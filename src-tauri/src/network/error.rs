/// Network error types for WebSocket communication
///
/// This module defines error types used throughout the network layer.

use thiserror::Error;

/// Network-related errors
#[derive(Error, Debug)]
pub enum NetworkError {
    /// Failed to connect to server
    #[error("Failed to connect to server: {0}")]
    ConnectionFailed(String),

    /// Authentication failed (invalid API key)
    #[error("Authentication failed: invalid API key")]
    AuthenticationFailed,

    /// WebSocket protocol error
    #[error("WebSocket protocol error: {0}")]
    ProtocolError(String),

    /// Connection timeout
    #[error("Connection timeout after {0}ms")]
    Timeout(u64),

    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    /// Failed to serialize message
    #[error("Failed to serialize message: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Failed to build HTTP request
    #[error("Failed to build HTTP request: {0}")]
    HttpError(String),

    /// Connection closed unexpectedly
    #[error("Connection closed unexpectedly")]
    ConnectionClosed,

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Server returned an error message
    #[error("Server error: {0}")]
    ServerError(String),
}

/// Result type for network operations
pub type NetworkResult<T> = Result<T, NetworkError>;

impl From<tokio_tungstenite::tungstenite::http::Error> for NetworkError {
    fn from(err: tokio_tungstenite::tungstenite::http::Error) -> Self {
        NetworkError::HttpError(err.to_string())
    }
}
