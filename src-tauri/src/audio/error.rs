use thiserror::Error;

/// Audio-related errors
#[derive(Error, Debug)]
pub enum AudioError {
    /// No audio input device found
    #[error("No audio input device found")]
    DeviceNotFound,

    /// Failed to build audio stream
    #[error("Failed to build audio stream: {0}")]
    StreamBuildFailed(String),

    /// Audio stream error
    #[error("Audio stream error: {0}")]
    StreamError(String),

    /// Resampling failed
    #[error("Resampling failed: {0}")]
    ResampleFailed(String),

    /// Device name is invalid
    #[error("Device name is invalid UTF-8")]
    InvalidDeviceName,

    /// Failed to get device configuration
    #[error("Failed to get device configuration: {0}")]
    ConfigError(String),

    /// cpal error
    #[error("cpal error: {0}")]
    CpalError(#[from] cpal::DevicesError),

    /// Default config error
    #[error("Default config error: {0}")]
    DefaultConfigError(#[from] cpal::DefaultStreamConfigError),

    /// Supported config error
    #[error("Supported config error: {0}")]
    SupportedConfigError(#[from] cpal::SupportedStreamConfigsError),
}

/// Result type for audio operations
pub type AudioResult<T> = Result<T, AudioError>;
