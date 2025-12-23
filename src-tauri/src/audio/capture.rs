use crate::audio::error::{AudioError, AudioResult};
use crate::audio::device::find_device_by_id;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Audio capture manager
///
/// Manages audio input stream and provides methods to start/stop capture.
/// Audio data is sent through a channel as Vec<f32> samples.
pub struct AudioCapture {
    /// The audio input stream (None when stopped)
    stream: Option<Stream>,
    /// Sample rate of the input device
    sample_rate: u32,
    /// Number of channels
    channels: u16,
    /// Device being used for capture
    device: Device,
    /// Stream configuration
    config: StreamConfig,
}

impl AudioCapture {
    /// Create a new AudioCapture instance
    ///
    /// # Arguments
    /// * `device_id` - Optional device ID. If None, uses the default input device.
    ///
    /// # Returns
    /// A new `AudioCapture` instance ready to start capturing.
    ///
    /// # Errors
    /// Returns `AudioError::DeviceNotFound` if no device is available.
    /// Returns `AudioError::DefaultConfigError` if unable to get device config.
    ///
    /// # Example
    /// ```no_run
    /// use raflow_lib::audio::capture::AudioCapture;
    ///
    /// // Use default device
    /// let capture = AudioCapture::new(None).unwrap();
    ///
    /// // Use specific device
    /// let capture = AudioCapture::new(Some("My Microphone")).unwrap();
    /// ```
    pub fn new(device_id: Option<&str>) -> AudioResult<Self> {
        let host = cpal::default_host();

        let device = if let Some(id) = device_id {
            find_device_by_id(&host, id)?
        } else {
            host.default_input_device()
                .ok_or(AudioError::DeviceNotFound)?
        };

        let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        info!("Using audio input device: {}", device_name);

        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        info!(
            "Device config: {} Hz, {} channels",
            sample_rate, channels
        );

        let stream_config = StreamConfig {
            channels,
            sample_rate: config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        Ok(Self {
            stream: None,
            sample_rate,
            channels,
            device,
            config: stream_config,
        })
    }

    /// Start capturing audio
    ///
    /// # Arguments
    /// * `sender` - Channel sender to send audio data to
    ///
    /// # Returns
    /// Ok(()) if the stream started successfully.
    ///
    /// # Errors
    /// Returns `AudioError::StreamBuildFailed` if unable to build the stream.
    ///
    /// # Example
    /// ```no_run
    /// use raflow_lib::audio::capture::AudioCapture;
    /// use tokio::sync::mpsc;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, mut rx) = mpsc::channel(100);
    ///     let mut capture = AudioCapture::new(None).unwrap();
    ///
    ///     capture.start(tx).unwrap();
    ///
    ///     // Receive audio data
    ///     while let Some(data) = rx.recv().await {
    ///         println!("Received {} samples", data.len());
    ///     }
    /// }
    /// ```
    pub fn start(&mut self, sender: mpsc::Sender<Vec<f32>>) -> AudioResult<()> {
        if self.stream.is_some() {
            warn!("Audio capture already started");
            return Ok(());
        }

        info!("Starting audio capture");

        // Create an Arc to share the sender across the audio callback
        let sender = Arc::new(sender);
        let sender_clone = Arc::clone(&sender);

        // Build the input stream
        let stream = self
            .device
            .build_input_stream(
                &self.config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Use try_send to avoid blocking the audio thread
                    // If the channel is full, we'll just drop this batch
                    if let Err(_) = sender_clone.try_send(data.to_vec()) {
                        // Silently drop if channel is full to avoid blocking
                        // This is expected behavior under high load
                    }
                },
                move |err| {
                    error!("Audio stream error: {}", err);
                },
                None,
            )
            .map_err(|e| AudioError::StreamBuildFailed(e.to_string()))?;

        // Start the stream
        stream
            .play()
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        self.stream = Some(stream);
        info!("Audio capture started successfully");

        Ok(())
    }

    /// Stop capturing audio
    ///
    /// This method stops the audio stream and releases resources.
    /// It's safe to call this multiple times.
    ///
    /// # Example
    /// ```no_run
    /// use raflow_lib::audio::capture::AudioCapture;
    /// use tokio::sync::mpsc;
    ///
    /// let (tx, _rx) = mpsc::channel(100);
    /// let mut capture = AudioCapture::new(None).unwrap();
    /// capture.start(tx).unwrap();
    ///
    /// // Later...
    /// capture.stop();
    /// ```
    pub fn stop(&mut self) {
        if let Some(stream) = self.stream.take() {
            info!("Stopping audio capture");
            drop(stream);
            debug!("Audio capture stopped");
        }
    }

    /// Get the sample rate of the input device
    ///
    /// # Returns
    /// The sample rate in Hz
    ///
    /// # Example
    /// ```no_run
    /// use raflow_lib::audio::capture::AudioCapture;
    ///
    /// let capture = AudioCapture::new(None).unwrap();
    /// println!("Sample rate: {} Hz", capture.sample_rate());
    /// ```
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the number of channels
    ///
    /// # Returns
    /// The number of audio channels
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// Check if capture is currently active
    ///
    /// # Returns
    /// true if currently capturing, false otherwise
    pub fn is_capturing(&self) -> bool {
        self.stream.is_some()
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_audio_capture_creation() {
        let result = AudioCapture::new(None);
        match result {
            Ok(capture) => {
                assert!(capture.sample_rate() > 0, "Sample rate should be positive");
                assert!(capture.channels() > 0, "Should have at least one channel");
                assert!(!capture.is_capturing(), "Should not be capturing initially");
                println!("Created AudioCapture: {} Hz, {} channels",
                         capture.sample_rate(), capture.channels());
            }
            Err(e) => {
                eprintln!("Warning: Could not create AudioCapture: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_audio_capture_start_stop() {
        let (tx, mut rx) = mpsc::channel(100);

        let result = AudioCapture::new(None);
        if result.is_err() {
            eprintln!("Warning: No audio device available for testing");
            return;
        }

        let mut capture = result.unwrap();

        // Start capture
        assert!(capture.start(tx).is_ok(), "Should start successfully");
        assert!(capture.is_capturing(), "Should be capturing after start");

        // Wait a bit and try to receive some data
        let timeout_result = tokio::time::timeout(
            Duration::from_secs(2),
            rx.recv()
        ).await;

        match timeout_result {
            Ok(Some(data)) => {
                println!("Received {} samples", data.len());
                assert!(!data.is_empty(), "Should receive non-empty data");
            }
            Ok(None) => {
                eprintln!("Warning: Channel closed unexpectedly");
            }
            Err(_) => {
                eprintln!("Warning: Timeout waiting for audio data");
            }
        }

        // Stop capture
        capture.stop();
        assert!(!capture.is_capturing(), "Should not be capturing after stop");

        // Should be safe to call stop again
        capture.stop();
    }

    #[tokio::test]
    async fn test_audio_capture_sample_rate() {
        let result = AudioCapture::new(None);
        if let Ok(capture) = result {
            let sample_rate = capture.sample_rate();
            println!("Sample rate: {}", sample_rate);

            // Check common sample rates
            let common_rates = [8000, 16000, 22050, 32000, 44100, 48000, 96000];
            let is_common = common_rates.contains(&sample_rate);

            if is_common {
                println!("✓ Common sample rate detected");
            }

            assert!(
                sample_rate >= 8000 && sample_rate <= 192000,
                "Sample rate should be in reasonable range"
            );
        }
    }

    #[tokio::test]
    async fn test_audio_capture_double_start() {
        let (tx, _rx) = mpsc::channel(100);

        if let Ok(mut capture) = AudioCapture::new(None) {
            // First start
            assert!(capture.start(tx.clone()).is_ok());

            // Second start should be ok (just logs a warning)
            assert!(capture.start(tx).is_ok());

            capture.stop();
        }
    }

    #[tokio::test]
    async fn test_audio_capture_with_specific_device() {
        // This test tries to use a specific device
        // It may fail if the device doesn't exist, which is expected

        let result = AudioCapture::new(Some("NonExistentDevice"));
        assert!(result.is_err(), "Should fail with non-existent device");
    }
}
