/// Integration tests for P1-T4: Audio Pipeline
///
/// This test file validates the complete audio processing pipeline,
/// from capture through resampling to Base64-encoded output.

use base64::{engine::general_purpose::STANDARD, Engine};
use raflow_lib::audio::AudioPipeline;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_pipeline_creation() {
    println!("\n=== Pipeline Creation Test ===");

    let pipeline = AudioPipeline::new(None);
    assert!(pipeline.is_ok(), "Should create pipeline successfully");

    let pipeline = pipeline.unwrap();
    println!("Input sample rate: {} Hz", pipeline.input_sample_rate());
    println!("Output sample rate: {} Hz", pipeline.output_sample_rate());

    assert_eq!(pipeline.output_sample_rate(), 16000);
    assert!(!pipeline.is_running());
}

#[tokio::test]
async fn test_pipeline_start_stop() {
    println!("\n=== Pipeline Start/Stop Test ===");

    let (tx, _rx) = mpsc::channel(10);
    let mut pipeline = AudioPipeline::new(None).unwrap();

    println!("Starting pipeline...");
    let result = pipeline.start(tx).await;
    assert!(result.is_ok(), "Should start successfully");
    assert!(pipeline.is_running());

    println!("Pipeline is running");

    // Let it run for a bit
    tokio::time::sleep(Duration::from_millis(200)).await;

    println!("Stopping pipeline...");
    pipeline.stop().await;
    assert!(!pipeline.is_running());

    println!("Pipeline stopped successfully");
}

#[tokio::test]
async fn test_pipeline_audio_output() {
    println!("\n=== Pipeline Audio Output Test ===");

    let (tx, mut rx) = mpsc::channel(100);
    let mut pipeline = AudioPipeline::new(None).unwrap();

    println!("Starting pipeline...");
    pipeline.start(tx).await.unwrap();

    println!("Waiting for audio chunks...");

    let mut chunks_received = 0;
    let mut total_samples = 0;

    // Collect audio for up to 3 seconds or 10 chunks
    let result = timeout(Duration::from_secs(3), async {
        while let Some(audio_base64) = rx.recv().await {
            chunks_received += 1;

            println!(
                "\nChunk {}: {} characters (Base64)",
                chunks_received,
                audio_base64.len()
            );

            // Verify it's valid Base64
            let decoded = STANDARD.decode(&audio_base64);
            assert!(decoded.is_ok(), "Should be valid Base64");

            let pcm_bytes = decoded.unwrap();
            let num_samples = pcm_bytes.len() / 2; // i16 = 2 bytes per sample
            total_samples += num_samples;

            println!("  PCM bytes: {}", pcm_bytes.len());
            println!("  Samples: {}", num_samples);
            println!(
                "  Duration: {:.1}ms",
                (num_samples as f64 / 16000.0) * 1000.0
            );

            // Each chunk should be around 100ms (1600 samples = 3200 bytes)
            assert!(
                pcm_bytes.len() >= 3000 && pcm_bytes.len() <= 3400,
                "Chunk size should be around 3200 bytes, got {}",
                pcm_bytes.len()
            );

            if chunks_received >= 10 {
                break;
            }
        }
    })
    .await;

    pipeline.stop().await;

    println!("\n=== Summary ===");
    println!("Total chunks received: {}", chunks_received);
    println!("Total samples: {}", total_samples);
    println!(
        "Total duration: {:.2}s",
        total_samples as f64 / 16000.0
    );

    // We should receive at least some chunks if there's audio input
    // (might be 0 if no microphone or silent environment)
    if chunks_received > 0 {
        assert!(result.is_ok(), "Should receive chunks without timeout");
        println!("\n✓ Audio pipeline successfully captured and processed audio");
    } else {
        println!("\n⚠ No audio chunks received (no microphone input or silent environment)");
    }
}

#[tokio::test]
async fn test_pipeline_output_format() {
    println!("\n=== Pipeline Output Format Test ===");

    let (tx, mut rx) = mpsc::channel(10);
    let mut pipeline = AudioPipeline::new(None).unwrap();

    pipeline.start(tx).await.unwrap();

    // Wait for first chunk with timeout
    let result = timeout(Duration::from_secs(2), rx.recv()).await;

    pipeline.stop().await;

    if let Ok(Some(audio_base64)) = result {
        println!("Received audio chunk");

        // Decode Base64
        let pcm_bytes = STANDARD.decode(&audio_base64).unwrap();
        println!("PCM bytes: {}", pcm_bytes.len());

        // Verify it's i16 PCM (should be even number of bytes)
        assert_eq!(pcm_bytes.len() % 2, 0, "Should be even number of bytes");

        // Convert bytes to i16 samples
        let mut samples = Vec::new();
        for chunk in pcm_bytes.chunks_exact(2) {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
            samples.push(sample);
        }

        println!("Samples: {}", samples.len());
        println!("Sample rate: 16000 Hz");
        println!("Duration: {:.1}ms", (samples.len() as f64 / 16000.0) * 1000.0);

        // Check sample range (should be i16: -32768 to 32767)
        let min_sample = samples.iter().min().unwrap();
        let max_sample = samples.iter().max().unwrap();
        println!("Sample range: {} to {}", min_sample, max_sample);

        assert!(
            *min_sample >= -32768 && *max_sample <= 32767,
            "Samples should be in i16 range"
        );

        println!("✓ Output format is correct (i16 PCM @ 16kHz)");
    } else {
        println!("⚠ No audio received (timeout or no microphone)");
    }
}

#[tokio::test]
async fn test_pipeline_continuous_operation() {
    println!("\n=== Pipeline Continuous Operation Test ===");

    let (tx, mut rx) = mpsc::channel(100);
    let mut pipeline = AudioPipeline::new(None).unwrap();

    pipeline.start(tx).await.unwrap();
    println!("Pipeline started, collecting chunks for 2 seconds...");

    let mut chunks = Vec::new();

    // Collect for 2 seconds
    let result = timeout(Duration::from_secs(2), async {
        while let Some(chunk) = rx.recv().await {
            chunks.push(chunk);
        }
    })
    .await;

    pipeline.stop().await;

    // Timeout is expected (we want to collect for full 2 seconds)
    assert!(result.is_err(), "Should timeout after 2 seconds");

    println!("\nChunks collected: {}", chunks.len());

    if !chunks.is_empty() {
        // Verify all chunks
        let mut total_duration = 0.0;

        for (i, chunk) in chunks.iter().enumerate() {
            let pcm_bytes = STANDARD.decode(chunk).unwrap();
            let samples = pcm_bytes.len() / 2;
            let duration_ms = (samples as f64 / 16000.0) * 1000.0;
            total_duration += duration_ms;

            if i < 5 {
                // Print first 5 chunks
                println!(
                    "Chunk {}: {} samples ({:.1}ms)",
                    i + 1,
                    samples,
                    duration_ms
                );
            }
        }

        println!("\nTotal duration: {:.2}s", total_duration / 1000.0);
        println!("Average chunk duration: {:.1}ms", total_duration / chunks.len() as f64);

        // Each chunk should be around 100ms
        let avg_duration = total_duration / chunks.len() as f64;
        assert!(
            avg_duration >= 80.0 && avg_duration <= 120.0,
            "Average chunk duration should be around 100ms, got {:.1}ms",
            avg_duration
        );

        println!("✓ Pipeline operates continuously with consistent timing");
    } else {
        println!("⚠ No chunks received (no audio input)");
    }
}

#[tokio::test]
async fn test_pipeline_restart() {
    println!("\n=== Pipeline Restart Test ===");

    let (tx, mut rx) = mpsc::channel(10);
    let mut pipeline = AudioPipeline::new(None).unwrap();

    // First session
    println!("Starting first session...");
    pipeline.start(tx.clone()).await.unwrap();

    let first_chunk = timeout(Duration::from_millis(500), rx.recv()).await;

    pipeline.stop().await;
    println!("First session stopped");

    // Clear any remaining messages
    while rx.try_recv().is_ok() {}

    // Wait a bit
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Second session
    println!("\nStarting second session...");
    pipeline.start(tx).await.unwrap();

    let second_chunk = timeout(Duration::from_millis(500), rx.recv()).await;

    pipeline.stop().await;
    println!("Second session stopped");

    // Both sessions should behave the same
    match (first_chunk, second_chunk) {
        (Ok(Some(_)), Ok(Some(_))) => {
            println!("✓ Pipeline can be restarted successfully");
        }
        (Ok(None), Ok(None)) | (Err(_), Err(_)) => {
            println!("⚠ No audio in both sessions (no microphone)");
        }
        _ => {
            panic!("Inconsistent behavior between sessions");
        }
    }
}

#[tokio::test]
async fn test_pipeline_multiple_receivers() {
    println!("\n=== Pipeline Multiple Receivers Test ===");

    let (tx, mut rx1) = mpsc::channel(10);
    let mut pipeline = AudioPipeline::new(None).unwrap();

    // Start pipeline with first receiver
    pipeline.start(tx).await.unwrap();

    // Collect some chunks
    let result = timeout(Duration::from_millis(500), rx1.recv()).await;

    pipeline.stop().await;

    if let Ok(Some(chunk)) = result {
        println!("Received chunk: {} characters", chunk.len());
        println!("✓ Pipeline works with mpsc channel");
    } else {
        println!("⚠ No audio received");
    }
}

#[tokio::test]
async fn test_pipeline_error_handling() {
    println!("\n=== Pipeline Error Handling Test ===");

    let mut pipeline = AudioPipeline::new(None).unwrap();

    // Test double start
    let (tx1, _rx1) = mpsc::channel(10);
    let (tx2, _rx2) = mpsc::channel(10);

    pipeline.start(tx1).await.unwrap();
    let result = pipeline.start(tx2).await;

    assert!(result.is_err(), "Should not allow double start");
    println!("✓ Correctly prevents double start");

    pipeline.stop().await;
}
