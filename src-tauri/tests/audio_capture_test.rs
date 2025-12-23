/// Integration tests for P1-T2: Audio Capture Stream
///
/// This test file validates that the audio capture functionality
/// works correctly in real-world scenarios.

use raflow_lib::audio::AudioCapture;
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_audio_capture_integration() {
    let (tx, mut rx) = mpsc::channel(100);

    let result = AudioCapture::new(None);

    match result {
        Ok(mut capture) => {
            println!("\n=== Audio Capture Integration Test ===");
            println!("Sample rate: {} Hz", capture.sample_rate());
            println!("Channels: {}", capture.channels());
            println!("Initial capturing state: {}", capture.is_capturing());

            // Start capturing
            let start_result = capture.start(tx);
            assert!(start_result.is_ok(), "Should start capture successfully");
            assert!(capture.is_capturing(), "Should be capturing after start");

            println!("Capture started, waiting for audio data...");

            // Collect some audio samples
            let mut sample_count = 0;
            let mut total_samples = 0;
            let timeout = Duration::from_secs(3);

            let start_time = std::time::Instant::now();

            while start_time.elapsed() < timeout && sample_count < 10 {
                match tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
                    Ok(Some(data)) => {
                        sample_count += 1;
                        total_samples += data.len();
                        println!("Batch {}: {} samples", sample_count, data.len());

                        // Verify data
                        assert!(!data.is_empty(), "Audio data should not be empty");

                        // Check that values are in reasonable range for f32 audio
                        let max_val = data.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
                        println!("  Max absolute value: {:.4}", max_val);
                        assert!(max_val <= 1.0, "Audio samples should be normalized");
                    }
                    Ok(None) => {
                        println!("Channel closed");
                        break;
                    }
                    Err(_) => {
                        println!("Timeout waiting for data");
                    }
                }
            }

            println!("\nSummary:");
            println!("  Batches received: {}", sample_count);
            println!("  Total samples: {}", total_samples);

            if sample_count > 0 {
                let duration_secs = total_samples as f64 / capture.sample_rate() as f64;
                println!("  Audio duration: {:.2} seconds", duration_secs);
            }

            // Stop capturing
            capture.stop();
            assert!(!capture.is_capturing(), "Should not be capturing after stop");
            println!("\nCapture stopped successfully");

            assert!(sample_count > 0, "Should have received at least some audio data");
        }
        Err(e) => {
            eprintln!("Warning: Could not create AudioCapture: {}", e);
            eprintln!("This may be expected in CI environments without audio hardware");
        }
    }
}

#[tokio::test]
async fn test_audio_capture_channel_overflow() {
    let result = AudioCapture::new(None);

    if result.is_err() {
        eprintln!("Warning: No audio device available for testing");
        return;
    }

    let mut capture = result.unwrap();

    println!("\n=== Audio Capture Channel Overflow Test ===");

    // Create a small channel that will overflow quickly
    let (tx, mut rx) = mpsc::channel(5);

    capture.start(tx).unwrap();
    println!("Capture started with small channel (size: 5)");

    // Don't consume from the channel immediately, let it overflow
    tokio::time::sleep(Duration::from_millis(500)).await;

    println!("Checking if system is still responsive...");

    // Now try to read
    let mut count = 0;
    while let Ok(Some(_)) = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
        count += 1;
        if count >= 5 {
            break;
        }
    }

    println!("Received {} batches from overflowed channel", count);

    // The capture should still work even after overflow
    assert!(capture.is_capturing(), "Should still be capturing");

    capture.stop();
    println!("Test completed successfully - overflow handled gracefully");
}

#[tokio::test]
async fn test_audio_capture_start_stop_multiple_times() {
    let result = AudioCapture::new(None);

    if result.is_err() {
        eprintln!("Warning: No audio device available for testing");
        return;
    }

    let mut capture = result.unwrap();

    println!("\n=== Audio Capture Start/Stop Multiple Times Test ===");

    for i in 1..=3 {
        println!("\nIteration {}", i);

        let (tx, mut rx) = mpsc::channel(10);

        // Start
        capture.start(tx).unwrap();
        assert!(capture.is_capturing());
        println!("  Started");

        // Receive some data
        let timeout_result = tokio::time::timeout(Duration::from_secs(1), rx.recv()).await;
        if timeout_result.is_ok() {
            println!("  Received data");
        }

        // Stop
        capture.stop();
        assert!(!capture.is_capturing());
        println!("  Stopped");

        // Small delay between iterations
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("\nAll iterations completed successfully");
}

#[tokio::test]
async fn test_audio_capture_drop_while_capturing() {
    let result = AudioCapture::new(None);

    if result.is_err() {
        eprintln!("Warning: No audio device available for testing");
        return;
    }

    println!("\n=== Audio Capture Drop While Capturing Test ===");

    let (tx, mut rx) = mpsc::channel(10);

    {
        let mut capture = result.unwrap();
        capture.start(tx).unwrap();
        println!("Capture started");

        // Receive some data
        let _data = tokio::time::timeout(Duration::from_secs(1), rx.recv()).await;

        println!("Dropping capture while still capturing...");
        // capture drops here
    }

    println!("Capture dropped successfully (via Drop trait)");

    // Channel should eventually close
    tokio::time::sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_audio_capture_sample_characteristics() {
    let result = AudioCapture::new(None);

    if result.is_err() {
        eprintln!("Warning: No audio device available for testing");
        return;
    }

    let mut capture = result.unwrap();
    let (tx, mut rx) = mpsc::channel(100);

    println!("\n=== Audio Capture Sample Characteristics Test ===");
    println!("Device info:");
    println!("  Sample rate: {} Hz", capture.sample_rate());
    println!("  Channels: {}", capture.channels());

    capture.start(tx).unwrap();

    // Collect first batch of samples
    if let Ok(Some(data)) = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await {
        println!("\nSample analysis:");
        println!("  Batch size: {} samples", data.len());

        // Calculate statistics
        let mean = data.iter().sum::<f32>() / data.len() as f32;
        let variance = data.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / data.len() as f32;
        let std_dev = variance.sqrt();

        println!("  Mean: {:.6}", mean);
        println!("  Std dev: {:.6}", std_dev);

        // Find min/max
        let min = data.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        println!("  Range: [{:.6}, {:.6}]", min, max);

        // Check for clipping
        let clipped = data.iter().filter(|&&v| v.abs() >= 1.0).count();
        println!("  Clipped samples: {} ({:.2}%)", clipped,
                 100.0 * clipped as f32 / data.len() as f32);

        // Verify samples are normalized
        assert!(max <= 1.0 && min >= -1.0, "Samples should be in range [-1.0, 1.0]");
    }

    capture.stop();
}
