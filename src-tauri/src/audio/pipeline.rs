use crate::audio::error::{AudioError, AudioResult};
use crate::audio::{AudioCapture, AudioResampler};
use base64::{engine::general_purpose::STANDARD, Engine};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// Audio processing pipeline that integrates capture, resampling, and encoding
///
/// This pipeline:
/// 1. Captures audio from microphone (via AudioCapture)
/// 2. Resamples to 16kHz (via AudioResampler)
/// 3. Converts f32 samples to i16 PCM
/// 4. Accumulates audio chunks (100ms batches)
/// 5. Base64 encodes the PCM data
/// 6. Sends encoded data through output channel
///
/// # Example
/// ```no_run
/// use raflow_lib::audio::AudioPipeline;
/// use tokio::sync::mpsc;
///
/// #[tokio::main]
/// async fn main() {
///     let (tx, mut rx) = mpsc::channel(10);
///
///     let mut pipeline = AudioPipeline::new(None).unwrap();
///     pipeline.start(tx).await.unwrap();
///
///     // Receive Base64-encoded audio chunks
///     while let Some(audio_base64) = rx.recv().await {
///         println!("Received {} bytes", audio_base64.len());
///     }
/// }
/// ```
pub struct AudioPipeline {
    /// Audio capture instance
    capture: AudioCapture,
    /// Processing task handle
    processing_task: Option<JoinHandle<()>>,
    /// Stop signal sender
    stop_signal: Option<tokio::sync::oneshot::Sender<()>>,
    /// Whether the pipeline is currently running
    is_running: bool,
}

impl AudioPipeline {
    /// Create a new audio processing pipeline
    ///
    /// # Arguments
    /// * `device_id` - Optional audio device ID (None for default device)
    ///
    /// # Returns
    /// A new `AudioPipeline` instance ready to start processing
    ///
    /// # Errors
    /// Returns error if audio capture or resampler initialization fails
    pub fn new(device_id: Option<&str>) -> AudioResult<Self> {
        info!("Creating audio pipeline");

        // Create audio capture
        let capture = AudioCapture::new(device_id)?;
        let input_rate = capture.sample_rate();

        info!("Audio capture created: {} Hz", input_rate);

        Ok(Self {
            capture,
            processing_task: None,
            stop_signal: None,
            is_running: false,
        })
    }

    /// Start the audio pipeline
    ///
    /// This starts audio capture and processing. Audio will be:
    /// 1. Captured from microphone
    /// 2. Resampled to 16kHz
    /// 3. Converted to i16 PCM
    /// 4. Accumulated to 100ms chunks
    /// 5. Base64 encoded
    /// 6. Sent through the output channel
    ///
    /// # Arguments
    /// * `output` - Channel to send Base64-encoded audio chunks
    ///
    /// # Errors
    /// Returns error if pipeline is already running or start fails
    pub async fn start(&mut self, output: mpsc::Sender<String>) -> AudioResult<()> {
        if self.is_running {
            return Err(AudioError::StreamBuildFailed(
                "Pipeline already running".to_string(),
            ));
        }

        info!("Starting audio pipeline");

        // Create internal channel for raw audio data
        let (internal_tx, internal_rx) = mpsc::channel(100);

        // Create stop signal
        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();

        // Start audio capture
        self.capture.start(internal_tx)?;

        // Spawn processing task
        let mut resampler = AudioResampler::new(
            self.capture.sample_rate(),
            16000,
        )?;

        let processing_task = tokio::spawn(async move {
            if let Err(e) = Self::processing_loop(
                internal_rx,
                output,
                stop_rx,
                &mut resampler,
            )
            .await
            {
                error!("Processing loop error: {}", e);
            }
        });

        self.processing_task = Some(processing_task);
        self.stop_signal = Some(stop_tx);
        self.is_running = true;

        info!("Audio pipeline started");
        Ok(())
    }

    /// Stop the audio pipeline
    ///
    /// This stops audio capture and processing tasks.
    pub async fn stop(&mut self) {
        if !self.is_running {
            return;
        }

        info!("Stopping audio pipeline");

        // Stop audio capture
        self.capture.stop();

        // Send stop signal to processing task
        if let Some(stop_tx) = self.stop_signal.take() {
            let _ = stop_tx.send(());
        }

        // Wait for processing task to finish
        if let Some(task) = self.processing_task.take() {
            let _ = task.await;
        }

        self.is_running = false;
        info!("Audio pipeline stopped");
    }

    /// Check if pipeline is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Get input sample rate
    pub fn input_sample_rate(&self) -> u32 {
        self.capture.sample_rate()
    }

    /// Get output sample rate (always 16000 Hz)
    pub fn output_sample_rate(&self) -> u32 {
        16000
    }

    /// Processing loop that handles audio data flow
    async fn processing_loop(
        mut input_rx: mpsc::Receiver<Vec<f32>>,
        output_tx: mpsc::Sender<String>,
        mut stop_rx: tokio::sync::oneshot::Receiver<()>,
        resampler: &mut AudioResampler,
    ) -> AudioResult<()> {
        // Buffer for accumulating resampled audio
        let mut resample_buffer = Vec::new();

        // Buffer for accumulating i16 PCM samples
        // 100ms @ 16kHz = 1600 samples = 3200 bytes
        let mut pcm_buffer: Vec<i16> = Vec::new();
        let target_samples = 1600; // 100ms @ 16kHz

        info!("Processing loop started");
        debug!(
            "Target accumulation: {} samples (100ms @ 16kHz)",
            target_samples
        );

        loop {
            tokio::select! {
                // Receive audio data
                Some(audio_data) = input_rx.recv() => {
                    // Resample audio using buffered processing
                    match resampler.process_buffered(&audio_data, &mut resample_buffer) {
                        Ok(resampled) => {
                            if resampled.is_empty() {
                                continue;
                            }

                            debug!("Resampled {} samples to {} samples", audio_data.len(), resampled.len());

                            // Convert f32 to i16 PCM
                            let pcm_samples = Self::f32_to_i16_pcm(&resampled);
                            pcm_buffer.extend(pcm_samples);

                            // Check if we have accumulated enough samples (100ms)
                            while pcm_buffer.len() >= target_samples {
                                // Take exactly target_samples
                                let chunk: Vec<i16> = pcm_buffer.drain(..target_samples).collect();

                                // Convert i16 to bytes
                                let pcm_bytes = Self::i16_to_bytes(&chunk);

                                // Base64 encode
                                let encoded = Self::encode_base64(&pcm_bytes);

                                debug!("Sending {} bytes (Base64: {} chars)", pcm_bytes.len(), encoded.len());

                                // Send to output channel
                                if output_tx.send(encoded).await.is_err() {
                                    warn!("Output channel closed, stopping processing loop");
                                    return Ok(());
                                }
                            }
                        }
                        Err(e) => {
                            error!("Resampling error: {}", e);
                        }
                    }
                }

                // Stop signal received
                _ = &mut stop_rx => {
                    info!("Stop signal received");
                    break;
                }

                // Input channel closed
                else => {
                    info!("Input channel closed");
                    break;
                }
            }
        }

        info!("Processing loop finished");
        Ok(())
    }

    /// Convert f32 samples (range: -1.0 to 1.0) to i16 PCM (range: -32768 to 32767)
    fn f32_to_i16_pcm(samples: &[f32]) -> Vec<i16> {
        samples
            .iter()
            .map(|&sample| {
                // Clamp to [-1.0, 1.0]
                let clamped = sample.clamp(-1.0, 1.0);
                // Scale to i16 range
                (clamped * 32767.0) as i16
            })
            .collect()
    }

    /// Convert i16 samples to little-endian bytes
    fn i16_to_bytes(samples: &[i16]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(samples.len() * 2);
        for &sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        bytes
    }

    /// Base64 encode PCM bytes
    fn encode_base64(data: &[u8]) -> String {
        STANDARD.encode(data)
    }
}

impl Drop for AudioPipeline {
    fn drop(&mut self) {
        if self.is_running {
            // Note: We can't call async stop() in Drop, but we can stop capture
            self.capture.stop();

            // Send stop signal
            if let Some(stop_tx) = self.stop_signal.take() {
                let _ = stop_tx.send(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = AudioPipeline::new(None);
        assert!(pipeline.is_ok());

        let pipeline = pipeline.unwrap();
        assert!(!pipeline.is_running());
        assert_eq!(pipeline.output_sample_rate(), 16000);
    }

    #[test]
    fn test_f32_to_i16_conversion() {
        let f32_samples = vec![-1.0, -0.5, 0.0, 0.5, 1.0];
        let i16_samples = AudioPipeline::f32_to_i16_pcm(&f32_samples);

        assert_eq!(i16_samples.len(), 5);
        assert_eq!(i16_samples[0], -32767); // -1.0
        assert_eq!(i16_samples[2], 0);      // 0.0
        assert_eq!(i16_samples[4], 32767);  // 1.0
    }

    #[test]
    fn test_f32_to_i16_clamping() {
        // Test values outside [-1.0, 1.0] range
        let f32_samples = vec![-2.0, -1.5, 1.5, 2.0];
        let i16_samples = AudioPipeline::f32_to_i16_pcm(&f32_samples);

        // Should be clamped to -32767 and 32767
        assert_eq!(i16_samples[0], -32767);
        assert_eq!(i16_samples[1], -32767);
        assert_eq!(i16_samples[2], 32767);
        assert_eq!(i16_samples[3], 32767);
    }

    #[test]
    fn test_i16_to_bytes() {
        let i16_samples = vec![0x1234, 0x5678, -1];
        let bytes = AudioPipeline::i16_to_bytes(&i16_samples);

        assert_eq!(bytes.len(), 6); // 3 samples * 2 bytes each

        // Check little-endian encoding
        assert_eq!(bytes[0], 0x34);
        assert_eq!(bytes[1], 0x12);
        assert_eq!(bytes[2], 0x78);
        assert_eq!(bytes[3], 0x56);
        assert_eq!(bytes[4], 0xFF);
        assert_eq!(bytes[5], 0xFF);
    }

    #[test]
    fn test_base64_encoding() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let encoded = AudioPipeline::encode_base64(&data);

        // Verify it's valid base64
        assert!(!encoded.is_empty());

        // Verify we can decode it back
        let decoded = STANDARD.decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_sample_rate_conversion() {
        let pipeline = AudioPipeline::new(None).unwrap();
        let input_rate = pipeline.input_sample_rate();
        let output_rate = pipeline.output_sample_rate();

        assert_eq!(output_rate, 16000);
        assert!(input_rate > 0);
    }

    #[tokio::test]
    async fn test_pipeline_start_stop() {
        let (tx, mut rx) = mpsc::channel(10);
        let mut pipeline = AudioPipeline::new(None).unwrap();

        // Start pipeline
        let result = pipeline.start(tx).await;
        assert!(result.is_ok());
        assert!(pipeline.is_running());

        // Let it run briefly
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Try to receive some data
        tokio::select! {
            _ = rx.recv() => {
                // Got data, good
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => {
                // Timeout, also acceptable (might not have audio input)
            }
        }

        // Stop pipeline
        pipeline.stop().await;
        assert!(!pipeline.is_running());
    }

    #[tokio::test]
    async fn test_pipeline_double_start() {
        let (tx, _rx) = mpsc::channel(10);
        let mut pipeline = AudioPipeline::new(None).unwrap();

        // First start should succeed
        assert!(pipeline.start(tx.clone()).await.is_ok());

        // Second start should fail
        assert!(pipeline.start(tx).await.is_err());

        pipeline.stop().await;
    }
}
