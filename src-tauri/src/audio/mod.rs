/// Audio capture and streaming
pub mod capture;

/// Audio device enumeration and management
pub mod device;

/// Audio-related error types
pub mod error;

/// Audio resampling
pub mod resampler;

/// Audio processing pipeline
pub mod pipeline;

/// High-performance audio buffer (ring buffer)
pub mod buffer;

// Re-export commonly used types
pub use buffer::{AudioBufferConsumer, AudioBufferProducer, AudioRingBuffer, BufferPool, PcmBuffer};
pub use capture::AudioCapture;
pub use device::{get_default_input_device, get_device_config, list_input_devices, AudioDevice};
pub use error::{AudioError, AudioResult};
pub use pipeline::AudioPipeline;
pub use resampler::AudioResampler;
