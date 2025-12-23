/// Network communication and WebSocket handling
///
/// This module provides types and utilities for communicating with
/// the ElevenLabs Scribe v2 Realtime API via WebSocket.

/// WebSocket connection management
pub mod connection;

/// Network error types
pub mod error;

/// WebSocket message type definitions
pub mod messages;

/// Async tasks for concurrent send/receive operations
pub mod tasks;

// Re-export commonly used types
pub use connection::{ConnectionConfig, ScribeConnection, WsReader, WsWriter};
pub use error::{NetworkError, NetworkResult};
pub use messages::{
    ClientMessage, CloseMessage, CommitMessage, InputAudioChunk, ServerMessage, SessionConfig,
    VadConfig, WordTimestamp,
};
