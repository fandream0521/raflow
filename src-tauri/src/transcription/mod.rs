/// End-to-end transcription session management
///
/// This module integrates the audio pipeline and network communication
/// to provide a complete speech-to-text transcription service.

use crate::audio::AudioPipeline;
use crate::network::tasks::{receiver_task, sender_task};
use crate::network::{ConnectionConfig, NetworkError, ScribeConnection, ServerMessage};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// Events emitted during transcription
///
/// These events represent the different types of messages received
/// from the transcription service.
#[derive(Debug, Clone, PartialEq)]
pub enum TranscriptEvent {
    /// Session has started with the given session ID
    SessionStarted { session_id: String },

    /// Partial (real-time) transcription result
    Partial { text: String },

    /// Final (committed) transcription result
    Committed { text: String },

    /// Error occurred during transcription
    Error { message: String },

    /// Connection closed
    Closed,
}

/// Complete transcription session
///
/// Manages the entire lifecycle of a speech-to-text session, including:
/// - Audio capture from microphone
/// - Audio processing (resampling, encoding)
/// - WebSocket communication with transcription service
/// - Event callbacks for transcription results
///
/// # Example
/// ```no_run
/// use raflow_lib::transcription::{TranscriptionSession, TranscriptEvent};
///
/// #[tokio::main]
/// async fn main() {
///     let mut session = TranscriptionSession::start(
///         "your-api-key",
///         |event| {
///             match event {
///                 TranscriptEvent::Partial { text } => {
///                     println!("Transcribing: {}", text);
///                 }
///                 TranscriptEvent::Committed { text } => {
///                     println!("Final: {}", text);
///                 }
///                 _ => {}
///             }
///         }
///     ).await.unwrap();
///
///     // Let it run for a while
///     tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
///
///     session.stop().await.unwrap();
/// }
/// ```
pub struct TranscriptionSession {
    /// Audio pipeline handle
    audio_pipeline: AudioPipeline,

    /// Sender task handle
    sender_handle: Option<JoinHandle<Result<(), NetworkError>>>,

    /// Receiver task handle
    receiver_handle: Option<JoinHandle<Result<(), NetworkError>>>,

    /// Event handler task handle
    event_handler_handle: Option<JoinHandle<()>>,

    /// Whether the session is running
    is_running: bool,
}

impl TranscriptionSession {
    /// Start a new transcription session
    ///
    /// This method:
    /// 1. Establishes WebSocket connection to the transcription service
    /// 2. Starts audio capture and processing
    /// 3. Spawns tasks for concurrent send/receive operations
    /// 4. Sets up event handling with the provided callback
    ///
    /// # Arguments
    /// * `api_key` - ElevenLabs API key for authentication
    /// * `on_event` - Callback function for transcription events
    ///
    /// # Returns
    /// A running `TranscriptionSession` instance
    ///
    /// # Errors
    /// Returns error if connection fails or audio setup fails
    ///
    /// # Example
    /// ```no_run
    /// use raflow_lib::transcription::TranscriptionSession;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let session = TranscriptionSession::start("api-key", |event| {
    ///         println!("Event: {:?}", event);
    ///     }).await.unwrap();
    /// }
    /// ```
    pub async fn start<F>(api_key: &str, on_event: F) -> Result<Self, TranscriptionError>
    where
        F: Fn(TranscriptEvent) + Send + Sync + 'static,
    {
        info!("Starting transcription session");

        // 1. Create audio pipeline
        let mut audio_pipeline = AudioPipeline::new(None)
            .map_err(|e| TranscriptionError::AudioError(e.to_string()))?;

        let input_rate = audio_pipeline.input_sample_rate();
        let output_rate = audio_pipeline.output_sample_rate();
        debug!(
            "Audio pipeline created: {} Hz -> {} Hz",
            input_rate, output_rate
        );

        // 2. Establish WebSocket connection
        let config = ConnectionConfig::new(output_rate);
        let connection = ScribeConnection::connect(api_key, &config)
            .await
            .map_err(TranscriptionError::NetworkError)?;

        info!("WebSocket connection established");

        // 3. Split connection into read/write halves
        let (writer, reader) = connection.split();

        // 4. Create channels for communication
        let (audio_tx, audio_rx) = mpsc::channel::<String>(100);
        let (msg_tx, mut msg_rx) = mpsc::channel::<ServerMessage>(100);

        // 5. Start audio pipeline
        audio_pipeline
            .start(audio_tx)
            .await
            .map_err(|e| TranscriptionError::AudioError(e.to_string()))?;

        info!("Audio pipeline started");

        // 6. Spawn sender task
        let sender_handle = tokio::spawn(async move {
            debug!("Sender task starting");
            let result = sender_task(writer, audio_rx).await;
            debug!("Sender task completed: {:?}", result);
            result
        });

        // 7. Spawn receiver task
        let receiver_handle = tokio::spawn(async move {
            debug!("Receiver task starting");
            let result = receiver_task(reader, msg_tx).await;
            debug!("Receiver task completed: {:?}", result);
            result
        });

        // 8. Spawn event handler task
        let on_event = Arc::new(on_event);
        let event_handler_handle = tokio::spawn(async move {
            debug!("Event handler starting");

            while let Some(msg) = msg_rx.recv().await {
                debug!("Received server message: {:?}", std::mem::discriminant(&msg));

                let event = match msg {
                    ServerMessage::SessionStarted { session_id, .. } => {
                        info!("Session started: {}", session_id);
                        TranscriptEvent::SessionStarted { session_id }
                    }
                    ServerMessage::PartialTranscript { text } => {
                        debug!("Partial transcript: {}", text);
                        TranscriptEvent::Partial { text }
                    }
                    ServerMessage::CommittedTranscript { text } => {
                        info!("Committed transcript: {}", text);
                        TranscriptEvent::Committed { text }
                    }
                    ServerMessage::CommittedTranscriptWithTimestamps { text, .. } => {
                        info!("Committed transcript with timestamps: {}", text);
                        TranscriptEvent::Committed { text }
                    }
                    ServerMessage::InputError { error_message } => {
                        error!("Input error: {}", error_message);
                        TranscriptEvent::Error {
                            message: error_message,
                        }
                    }
                };

                // Call the user's callback
                on_event(event);
            }

            info!("Event handler completed: channel closed");
            on_event(TranscriptEvent::Closed);
        });

        info!("Transcription session started successfully");

        Ok(Self {
            audio_pipeline,
            sender_handle: Some(sender_handle),
            receiver_handle: Some(receiver_handle),
            event_handler_handle: Some(event_handler_handle),
            is_running: true,
        })
    }

    /// Stop the transcription session
    ///
    /// This method:
    /// 1. Stops audio capture
    /// 2. Waits for all tasks to complete
    /// 3. Cleans up resources
    ///
    /// # Errors
    /// Returns error if tasks fail to complete cleanly
    ///
    /// # Example
    /// ```no_run
    /// # use raflow_lib::transcription::TranscriptionSession;
    /// # async fn example(mut session: TranscriptionSession) {
    /// session.stop().await.unwrap();
    /// # }
    /// ```
    pub async fn stop(&mut self) -> Result<(), TranscriptionError> {
        if !self.is_running {
            warn!("Session already stopped");
            return Ok(());
        }

        info!("Stopping transcription session");

        // 1. Stop audio pipeline (this closes the audio_tx channel)
        self.audio_pipeline.stop().await;
        info!("Audio pipeline stopped");

        // 2. Wait for sender task to complete
        if let Some(handle) = self.sender_handle.take() {
            match handle.await {
                Ok(Ok(())) => debug!("Sender task completed successfully"),
                Ok(Err(e)) => warn!("Sender task completed with error: {}", e),
                Err(e) => error!("Sender task panicked: {}", e),
            }
        }

        // 3. Wait for receiver task to complete
        if let Some(handle) = self.receiver_handle.take() {
            match handle.await {
                Ok(Ok(())) => debug!("Receiver task completed successfully"),
                Ok(Err(e)) => warn!("Receiver task completed with error: {}", e),
                Err(e) => error!("Receiver task panicked: {}", e),
            }
        }

        // 4. Wait for event handler to complete
        if let Some(handle) = self.event_handler_handle.take() {
            match handle.await {
                Ok(()) => debug!("Event handler completed successfully"),
                Err(e) => error!("Event handler panicked: {}", e),
            }
        }

        self.is_running = false;
        info!("Transcription session stopped");

        Ok(())
    }

    /// Check if the session is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

/// Errors that can occur during transcription
#[derive(Debug, thiserror::Error)]
pub enum TranscriptionError {
    /// Audio-related error
    #[error("Audio error: {0}")]
    AudioError(String),

    /// Network-related error
    #[error("Network error: {0}")]
    NetworkError(#[from] NetworkError),

    /// Session is not running
    #[error("Session is not running")]
    NotRunning,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_event_types() {
        let events = vec![
            TranscriptEvent::SessionStarted {
                session_id: "test-123".to_string(),
            },
            TranscriptEvent::Partial {
                text: "hello".to_string(),
            },
            TranscriptEvent::Committed {
                text: "hello world".to_string(),
            },
            TranscriptEvent::Error {
                message: "test error".to_string(),
            },
            TranscriptEvent::Closed,
        ];

        // Verify all variants can be created
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_transcript_event_equality() {
        let event1 = TranscriptEvent::Partial {
            text: "test".to_string(),
        };
        let event2 = TranscriptEvent::Partial {
            text: "test".to_string(),
        };
        let event3 = TranscriptEvent::Partial {
            text: "different".to_string(),
        };

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
    }

    #[test]
    fn test_transcript_event_clone() {
        let event = TranscriptEvent::Committed {
            text: "test".to_string(),
        };
        let cloned = event.clone();

        assert_eq!(event, cloned);
    }
}
