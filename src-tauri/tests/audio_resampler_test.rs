/// Integration tests for P1-T3: Audio Resampler
///
/// This test file validates that the audio resampler works correctly
/// with real-world usage patterns.

use raflow_lib::audio::AudioResampler;

#[test]
fn test_resampler_48khz_to_16khz_integration() {
    println!("\n=== Resampler Integration Test: 48kHz -> 16kHz ===");

    let mut resampler = AudioResampler::new(48000, 16000).unwrap();

    println!("Configuration:");
    println!("  Input rate: {} Hz", resampler.input_rate());
    println!("  Output rate: {} Hz", resampler.output_rate());
    println!("  Chunk size: {} samples", resampler.chunk_size());
    println!("  Expected output size: ~{} samples", resampler.output_chunk_size());

    // Generate a test signal: 1 second of 440Hz sine wave
    let duration_secs = 1.0;
    let sample_rate = 48000.0;
    let frequency = 440.0; // A4 note

    let total_samples = (duration_secs * sample_rate) as usize;
    let mut all_input = Vec::new();

    for i in 0..total_samples {
        let t = i as f32 / sample_rate;
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin();
        all_input.push(sample);
    }

    println!("\nProcessing {} samples of input ({:.1}s @ {} Hz)",
             all_input.len(), duration_secs, 48000);

    // Process in chunks
    let chunk_size = resampler.chunk_size();
    let mut all_output = Vec::new();
    let mut chunk_count = 0;

    for chunk in all_input.chunks_exact(chunk_size) {
        let output = resampler.process(chunk).unwrap();
        all_output.extend(output);
        chunk_count += 1;
    }

    println!("\nResults:");
    println!("  Chunks processed: {}", chunk_count);
    println!("  Total output samples: {}", all_output.len());
    println!("  Output duration: {:.2}s", all_output.len() as f32 / 16000.0);

    // Expected output: approximately 1 second at 16kHz = 16000 samples
    let expected_samples = 16000;
    let tolerance = 1000; // Allow 10% variation

    assert!(
        (all_output.len() as i32 - expected_samples).abs() < tolerance,
        "Expected ~{} output samples, got {}",
        expected_samples,
        all_output.len()
    );

    // Verify signal quality
    let max_amplitude = all_output.iter().map(|&v| v.abs()).fold(0.0f32, f32::max);
    println!("  Max amplitude: {:.4}", max_amplitude);

    assert!(
        max_amplitude > 0.5 && max_amplitude <= 1.01,
        "Signal amplitude should be preserved"
    );
}

#[test]
fn test_resampler_44khz_to_16khz_integration() {
    println!("\n=== Resampler Integration Test: 44.1kHz -> 16kHz ===");

    let mut resampler = AudioResampler::new(44100, 16000).unwrap();

    println!("Configuration:");
    println!("  Input rate: {} Hz", resampler.input_rate());
    println!("  Output rate: {} Hz", resampler.output_rate());
    println!("  Chunk size: {} samples", resampler.chunk_size());

    // Process multiple chunks
    let chunk_size = resampler.chunk_size();
    let mut total_output = 0;

    for _ in 0..100 {
        let input: Vec<f32> = (0..chunk_size)
            .map(|i| (i as f32 * 0.01).sin())
            .collect();

        let output = resampler.process(&input).unwrap();
        total_output += output.len();
    }

    println!("\nProcessed 100 chunks:");
    println!("  Total input: {} samples", chunk_size * 100);
    println!("  Total output: {} samples", total_output);

    // Ratio should be approximately 44100/16000 = 2.75625
    let expected_ratio = 44100.0 / 16000.0;
    let actual_ratio = (chunk_size * 100) as f32 / total_output as f32;

    println!("  Expected ratio: {:.4}", expected_ratio);
    println!("  Actual ratio: {:.4}", actual_ratio);

    // Allow 20% variation
    assert!(
        (actual_ratio - expected_ratio).abs() < 0.5,
        "Ratio mismatch: expected {:.4}, got {:.4}",
        expected_ratio,
        actual_ratio
    );
}

#[test]
fn test_resampler_continuous_stream() {
    println!("\n=== Resampler Continuous Stream Test ===");

    let mut resampler = AudioResampler::new(48000, 16000).unwrap();
    let chunk_size = resampler.chunk_size();

    println!("Simulating continuous audio stream...");
    println!("  Chunk size: {} samples", chunk_size);

    // Simulate 10 seconds of continuous audio
    let duration_seconds = 10;
    let chunks_per_second = 48000 / chunk_size;
    let total_chunks = chunks_per_second * duration_seconds;

    println!("  Total chunks: {}", total_chunks);

    let mut output_sizes = Vec::new();

    for chunk_idx in 0..total_chunks {
        // Generate a chunk with varying frequency
        let input: Vec<f32> = (0..chunk_size)
            .map(|i| {
                let t = (chunk_idx * chunk_size + i) as f32 / 48000.0;
                let freq = 200.0 + (t * 10.0).sin() * 100.0;
                (2.0 * std::f32::consts::PI * freq * t).sin()
            })
            .collect();

        let output = resampler.process(&input).unwrap();
        output_sizes.push(output.len());
    }

    // Calculate statistics
    let total_output: usize = output_sizes.iter().sum();
    let avg_output: f32 = total_output as f32 / output_sizes.len() as f32;
    let min_output = *output_sizes.iter().min().unwrap();
    let max_output = *output_sizes.iter().max().unwrap();

    println!("\nOutput statistics:");
    println!("  Total output samples: {}", total_output);
    println!("  Average per chunk: {:.2}", avg_output);
    println!("  Min/Max per chunk: {} / {}", min_output, max_output);

    // Expected: ~10 seconds at 16kHz = ~160,000 samples
    let expected = 160000;
    let tolerance = 10000;

    assert!(
        (total_output as i32 - expected).abs() < tolerance,
        "Expected ~{} samples, got {}",
        expected,
        total_output
    );
}

#[test]
fn test_resampler_reset_integration() {
    println!("\n=== Resampler Reset Integration Test ===");

    let mut resampler = AudioResampler::new(48000, 16000).unwrap();
    let chunk_size = resampler.chunk_size();

    // Process some data
    let input1: Vec<f32> = vec![1.0; chunk_size];
    let output1 = resampler.process(&input1).unwrap();
    println!("Output 1 size: {}", output1.len());

    // Reset
    resampler.reset();
    println!("Resampler reset");

    // Process same data again
    let output2 = resampler.process(&input1).unwrap();
    println!("Output 2 size: {}", output2.len());

    // Sizes should be the same
    assert_eq!(
        output1.len(),
        output2.len(),
        "Output sizes should match after reset"
    );

    // Process different data
    let input2: Vec<f32> = vec![0.5; chunk_size];
    let output3 = resampler.process(&input2).unwrap();
    println!("Output 3 size: {}", output3.len());

    assert!(!output3.is_empty(), "Should produce output after reset");
}

#[test]
fn test_resampler_buffered_integration() {
    println!("\n=== Resampler Buffered Processing Test ===");

    let mut resampler = AudioResampler::new(48000, 16000).unwrap();
    let mut buffer = Vec::new();

    // Simulate variable-length input chunks
    let input_sizes = vec![100, 200, 300, 150, 250, 400, 180];

    println!("Processing variable-length inputs:");

    let mut total_output = 0;

    for (idx, size) in input_sizes.iter().enumerate() {
        let input: Vec<f32> = vec![0.5; *size];
        let output = resampler.process_buffered(&input, &mut buffer).unwrap();

        println!("  Chunk {}: input={}, output={}, buffer={}",
                 idx + 1, size, output.len(), buffer.len());

        total_output += output.len();
    }

    println!("\nResults:");
    println!("  Total output: {}", total_output);
    println!("  Remaining in buffer: {}", buffer.len());

    assert!(total_output > 0, "Should have produced some output");
    println!("\nBuffered processing test completed successfully");
}

#[test]
fn test_resampler_frequency_preservation() {
    println!("\n=== Resampler Frequency Preservation Test ===");

    let mut resampler = AudioResampler::new(48000, 16000).unwrap();
    let chunk_size = resampler.chunk_size();

    // Generate a 1kHz test tone
    let test_freq = 1000.0;
    let input_rate = 48000.0;

    println!("Generating {}Hz test tone @ {} Hz", test_freq, input_rate as u32);

    let mut all_output = Vec::new();

    // Generate and process 100ms of audio
    let total_samples = (input_rate * 0.1) as usize;

    for start in (0..total_samples).step_by(chunk_size) {
        if start + chunk_size > total_samples {
            break;
        }

        let input: Vec<f32> = (start..start + chunk_size)
            .map(|i| {
                let t = i as f32 / input_rate;
                (2.0 * std::f32::consts::PI * test_freq * t).sin()
            })
            .collect();

        let output = resampler.process(&input).unwrap();
        all_output.extend(output);
    }

    println!("Output samples: {}", all_output.len());

    // Check that we still have a strong signal at the test frequency
    // (Simple check: verify there are zero crossings)
    let mut zero_crossings = 0;
    for i in 1..all_output.len() {
        if (all_output[i - 1] < 0.0 && all_output[i] >= 0.0)
            || (all_output[i - 1] >= 0.0 && all_output[i] < 0.0)
        {
            zero_crossings += 1;
        }
    }

    println!("Zero crossings: {}", zero_crossings);

    // At 1kHz for 100ms @ 16kHz output, we expect roughly 200 zero crossings
    // (2 per cycle, 100 cycles)
    assert!(
        zero_crossings > 100 && zero_crossings < 300,
        "Expected ~200 zero crossings for 1kHz tone, got {}",
        zero_crossings
    );

    println!("Frequency preservation test passed");
}
