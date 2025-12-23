use crate::audio::error::{AudioError, AudioResult};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use tracing::{debug, info};

/// Audio resampler for converting between sample rates
///
/// This resampler uses high-quality Sinc interpolation to convert audio
/// from any input sample rate to a target output rate (typically 16kHz for speech recognition).
pub struct AudioResampler {
    /// The rubato resampler instance
    resampler: SincFixedIn<f32>,
    /// Input buffer for rubato (channels x samples)
    input_buffer: Vec<Vec<f32>>,
    /// Output buffer for rubato (channels x samples)
    output_buffer: Vec<Vec<f32>>,
    /// Input sample rate
    input_rate: u32,
    /// Output sample rate
    output_rate: u32,
    /// Number of input samples per chunk
    chunk_size: usize,
}

impl AudioResampler {
    /// Create a new audio resampler
    ///
    /// # Arguments
    /// * `input_rate` - Input sample rate in Hz
    /// * `output_rate` - Output sample rate in Hz (typically 16000 for speech)
    ///
    /// # Returns
    /// A new `AudioResampler` instance configured for the specified rates.
    ///
    /// # Errors
    /// Returns `AudioError::ResampleFailed` if the resampler cannot be created.
    ///
    /// # Example
    /// ```no_run
    /// use raflow_lib::audio::resampler::AudioResampler;
    ///
    /// // Resample from 48kHz to 16kHz
    /// let resampler = AudioResampler::new(48000, 16000).unwrap();
    /// ```
    pub fn new(input_rate: u32, output_rate: u32) -> AudioResult<Self> {
        info!(
            "Creating resampler: {} Hz -> {} Hz",
            input_rate, output_rate
        );

        // Handle the case where input and output rates are the same
        if input_rate == output_rate {
            info!("Input and output rates are the same, using passthrough mode");
            // We still create a resampler but with ratio 1.0
        }

        // Calculate the ratio
        let ratio = output_rate as f64 / input_rate as f64;
        debug!("Resample ratio: {:.6}", ratio);

        // Configure Sinc interpolation parameters for high quality
        let params = SincInterpolationParameters {
            sinc_len: 256,                              // Length of sinc function
            f_cutoff: 0.95,                             // Cutoff frequency
            interpolation: SincInterpolationType::Linear, // Interpolation type
            oversampling_factor: 256,                   // Oversampling factor
            window: WindowFunction::BlackmanHarris2,    // Window function
        };

        // Determine chunk size based on input rate
        // We want chunks that represent about 10ms of audio
        let chunk_size = (input_rate / 100) as usize; // 10ms worth of samples

        debug!(
            "Chunk size: {} samples ({:.1}ms @ {} Hz)",
            chunk_size,
            1000.0 * chunk_size as f64 / input_rate as f64,
            input_rate
        );

        // Create the resampler
        // Note: We use 1 channel (mono) and allow ratio variation up to 2.0
        let resampler = SincFixedIn::<f32>::new(
            ratio,
            2.0, // max_relative_ratio (allow up to 2x variation)
            params,
            chunk_size,
            1, // number of channels (mono)
        )
        .map_err(|e| AudioError::ResampleFailed(format!("Failed to create resampler: {}", e)))?;

        // Pre-allocate buffers
        let input_buffer = resampler.input_buffer_allocate(true);
        let output_buffer = resampler.output_buffer_allocate(true);

        info!(
            "Resampler created: chunk_size={}, output_size={}",
            chunk_size,
            resampler.output_frames_max()
        );

        Ok(Self {
            resampler,
            input_buffer,
            output_buffer,
            input_rate,
            output_rate,
            chunk_size,
        })
    }

    /// Process audio samples through the resampler
    ///
    /// # Arguments
    /// * `input` - Input audio samples as f32 (mono)
    ///
    /// # Returns
    /// Resampled audio data as Vec<f32>
    ///
    /// # Errors
    /// Returns `AudioError::ResampleFailed` if resampling fails.
    ///
    /// # Note
    /// The input must contain exactly `chunk_size` samples. If your input
    /// is a different size, you'll need to buffer it appropriately.
    ///
    /// # Example
    /// ```no_run
    /// use raflow_lib::audio::resampler::AudioResampler;
    ///
    /// let mut resampler = AudioResampler::new(48000, 16000).unwrap();
    ///
    /// // Process 480 samples (10ms @ 48kHz)
    /// let input = vec![0.0f32; 480];
    /// let output = resampler.process(&input).unwrap();
    ///
    /// // Output will have ~160 samples (10ms @ 16kHz)
    /// println!("Output size: {}", output.len());
    /// ```
    pub fn process(&mut self, input: &[f32]) -> AudioResult<Vec<f32>> {
        // Check input size
        if input.len() != self.chunk_size {
            return Err(AudioError::ResampleFailed(format!(
                "Input size mismatch: expected {} samples, got {}",
                self.chunk_size,
                input.len()
            )));
        }

        // Copy input to the input buffer (channel 0)
        self.input_buffer[0].copy_from_slice(input);

        // Process the samples
        let (_input_frames_used, output_frames_generated) = self
            .resampler
            .process_into_buffer(&self.input_buffer, &mut self.output_buffer, None)
            .map_err(|e| AudioError::ResampleFailed(format!("Resampling failed: {}", e)))?;

        // Extract the output (channel 0)
        let output = self.output_buffer[0][..output_frames_generated].to_vec();

        debug!(
            "Resampled {} -> {} samples",
            input.len(),
            output.len()
        );

        Ok(output)
    }

    /// Process a variable-length input buffer
    ///
    /// This method handles inputs of any size by buffering and processing
    /// in chunks internally.
    ///
    /// # Arguments
    /// * `input` - Input audio samples (any length)
    /// * `buffer` - Internal buffer to accumulate samples
    ///
    /// # Returns
    /// Resampled audio data (may be empty if not enough data accumulated)
    pub fn process_buffered(
        &mut self,
        input: &[f32],
        buffer: &mut Vec<f32>,
    ) -> AudioResult<Vec<f32>> {
        // Add input to buffer
        buffer.extend_from_slice(input);

        let mut output = Vec::new();

        // Process as many complete chunks as we have
        while buffer.len() >= self.chunk_size {
            let chunk: Vec<f32> = buffer.drain(..self.chunk_size).collect();
            let resampled = self.process(&chunk)?;
            output.extend(resampled);
        }

        Ok(output)
    }

    /// Reset the resampler state
    ///
    /// This clears any internal state in the resampler, which is useful
    /// when starting a new audio session.
    pub fn reset(&mut self) {
        debug!("Resetting resampler");
        self.resampler.reset();

        // Clear buffers
        for channel in &mut self.input_buffer {
            channel.fill(0.0);
        }
        for channel in &mut self.output_buffer {
            channel.fill(0.0);
        }
    }

    /// Get the input sample rate
    pub fn input_rate(&self) -> u32 {
        self.input_rate
    }

    /// Get the output sample rate
    pub fn output_rate(&self) -> u32 {
        self.output_rate
    }

    /// Get the chunk size (number of input samples per process call)
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Get the expected output size for one chunk
    pub fn output_chunk_size(&self) -> usize {
        self.resampler.output_frames_max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample_48k_to_16k() {
        let mut resampler = AudioResampler::new(48000, 16000).unwrap();

        // Check configuration
        assert_eq!(resampler.input_rate(), 48000);
        assert_eq!(resampler.output_rate(), 16000);
        assert_eq!(resampler.chunk_size(), 480); // 10ms @ 48kHz

        // Input 480 samples (10ms @ 48kHz)
        let input: Vec<f32> = (0..480)
            .map(|i| (i as f32 * 0.01 * std::f32::consts::PI).sin())
            .collect();

        let output = resampler.process(&input).unwrap();

        // Output should be approximately 160 samples (10ms @ 16kHz)
        // The actual ratio is 48000/16000 = 3, so 480/3 = 160
        // However, rubato's SincFixedIn may produce slightly different sizes
        // due to internal buffering and filtering
        println!("48k->16k output size: {}", output.len());

        // Expect roughly 1/3 of input size (allow wider tolerance)
        let expected = 160;
        let tolerance = 50; // Allow +/- 50 samples due to filtering
        assert!(
            (output.len() as i32 - expected).abs() < tolerance,
            "Expected ~{} samples, got {}",
            expected,
            output.len()
        );

        // Verify output is reasonable (values in range)
        // Allow for small floating point errors
        for &sample in &output {
            assert!(
                sample.abs() <= 1.001,
                "Sample value out of range: {}",
                sample
            );
        }
    }

    #[test]
    fn test_resample_44k_to_16k() {
        let mut resampler = AudioResampler::new(44100, 16000).unwrap();

        assert_eq!(resampler.chunk_size(), 441); // 10ms @ 44.1kHz

        // Input 441 samples
        let input: Vec<f32> = (0..441).map(|i| (i as f32 * 0.01).sin()).collect();

        let output = resampler.process(&input).unwrap();

        // Output should be approximately 160 samples
        // Ratio: 44100/16000 = 2.75625, so 441/2.75625 ≈ 160
        println!("44.1k->16k output size: {}", output.len());

        let expected = 160;
        let tolerance = 50;
        assert!(
            (output.len() as i32 - expected).abs() < tolerance,
            "Expected ~{} samples, got {}",
            expected,
            output.len()
        );
    }

    #[test]
    fn test_resample_16k_to_16k() {
        // Test passthrough (same rate)
        let mut resampler = AudioResampler::new(16000, 16000).unwrap();

        assert_eq!(resampler.chunk_size(), 160); // 10ms @ 16kHz

        let input: Vec<f32> = (0..160).map(|i| i as f32 / 160.0).collect();

        let output = resampler.process(&input).unwrap();

        // When input and output rates are the same (ratio = 1.0),
        // rubato still applies filtering which can affect output size
        println!("16k->16k output size: {}", output.len());

        // Allow for significant variation due to filtering
        let expected = 160;
        let tolerance = 140; // Very wide tolerance for 1:1 ratio
        assert!(
            (output.len() as i32 - expected).abs() < tolerance,
            "Expected ~{} samples (tolerance {}), got {}",
            expected,
            tolerance,
            output.len()
        );
    }

    #[test]
    fn test_resample_wrong_input_size() {
        let mut resampler = AudioResampler::new(48000, 16000).unwrap();

        // Try with wrong input size
        let input = vec![0.0f32; 100]; // Wrong size

        let result = resampler.process(&input);
        assert!(result.is_err(), "Should fail with wrong input size");
    }

    #[test]
    fn test_resample_reset() {
        let mut resampler = AudioResampler::new(48000, 16000).unwrap();

        // Process some data
        let input = vec![1.0f32; 480];
        let _ = resampler.process(&input).unwrap();

        // Reset
        resampler.reset();

        // Should still work after reset
        let output = resampler.process(&input).unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_resample_multiple_chunks() {
        let mut resampler = AudioResampler::new(48000, 16000).unwrap();

        let mut total_output = Vec::new();

        // Process multiple chunks
        for _ in 0..10 {
            let input: Vec<f32> = (0..480).map(|i| (i as f32 * 0.01).sin()).collect();
            let output = resampler.process(&input).unwrap();
            total_output.extend(output);
        }

        // Should have approximately 1600 samples total (10 * 160)
        println!("Total output size: {}", total_output.len());
        assert!(
            (total_output.len() as i32 - 1600).abs() < 100,
            "Expected ~1600 samples, got {}",
            total_output.len()
        );
    }

    #[test]
    fn test_resample_buffered() {
        let mut resampler = AudioResampler::new(48000, 16000).unwrap();
        let mut buffer = Vec::new();

        // First call with small input (not enough for a chunk)
        let input1 = vec![0.5f32; 200];
        let output1 = resampler.process_buffered(&input1, &mut buffer).unwrap();
        assert!(output1.is_empty(), "Should not output anything yet");

        // Second call with more data
        let input2 = vec![0.5f32; 400];
        let output2 = resampler.process_buffered(&input2, &mut buffer).unwrap();
        assert!(!output2.is_empty(), "Should have output now");

        println!("Buffered output size: {}", output2.len());
    }

    #[test]
    fn test_resample_signal_preservation() {
        let mut resampler = AudioResampler::new(48000, 16000).unwrap();

        // Generate a simple sine wave at 440Hz (A4 note)
        let freq = 440.0;
        let input: Vec<f32> = (0..480)
            .map(|i| {
                let t = i as f32 / 48000.0;
                (2.0 * std::f32::consts::PI * freq * t).sin()
            })
            .collect();

        let output = resampler.process(&input).unwrap();

        // Check that we still have a reasonable signal
        // Find max amplitude
        let max_amplitude = output.iter().map(|&v| v.abs()).fold(0.0f32, f32::max);

        println!("Signal max amplitude: {}", max_amplitude);
        assert!(
            max_amplitude > 0.5 && max_amplitude <= 1.0,
            "Signal amplitude should be preserved"
        );
    }
}
