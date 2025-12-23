/// WebSocket message types for ElevenLabs Scribe v2 Realtime API
///
/// This module defines all message types used in communication with the
/// ElevenLabs speech-to-text WebSocket API.
///
/// Reference: https://elevenlabs.io/docs/api-reference/speech-to-text/v-1-speech-to-text-realtime

use serde::{Deserialize, Serialize};

// ============================================================================
// Client -> Server Messages
// ============================================================================

/// Audio chunk message sent from client to server
///
/// Contains Base64-encoded PCM audio data to be transcribed.
///
/// # Example
/// ```
/// use raflow_lib::network::messages::InputAudioChunk;
///
/// let chunk = InputAudioChunk::new("SGVsbG8gV29ybGQ=".to_string())
///     .with_sample_rate(16000);
///
/// let json = serde_json::to_string(&chunk).unwrap();
/// ```
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct InputAudioChunk {
    /// Message type identifier (always "input_audio_chunk")
    pub message_type: &'static str,

    /// Base64-encoded PCM audio data (i16 little-endian)
    pub audio_base_64: String,

    /// Whether to manually commit this segment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<bool>,

    /// Sample rate in Hz (should be sent with first chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,

    /// Previous context text for improved accuracy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_text: Option<String>,
}

impl InputAudioChunk {
    /// Create a new audio chunk message
    ///
    /// # Arguments
    /// * `audio_base_64` - Base64-encoded PCM audio data
    pub fn new(audio_base_64: String) -> Self {
        Self {
            message_type: "input_audio_chunk",
            audio_base_64,
            commit: None,
            sample_rate: None,
            previous_text: None,
        }
    }

    /// Set sample rate (typically sent with first chunk)
    pub fn with_sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = Some(rate);
        self
    }

    /// Mark this chunk to be manually committed
    pub fn with_commit(mut self) -> Self {
        self.commit = Some(true);
        self
    }

    /// Add previous context text for improved accuracy
    pub fn with_previous_text(mut self, text: String) -> Self {
        self.previous_text = Some(text);
        self
    }
}

/// Manual commit message
///
/// Tells the server to commit the current transcription segment.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct CommitMessage {
    /// Message type identifier (always "commit")
    pub message_type: &'static str,
}

impl Default for CommitMessage {
    fn default() -> Self {
        Self {
            message_type: "commit",
        }
    }
}

impl CommitMessage {
    /// Create a new commit message
    pub fn new() -> Self {
        Self::default()
    }
}

/// Close connection message
///
/// Gracefully closes the WebSocket connection.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct CloseMessage {
    /// Message type identifier (always "close")
    pub message_type: &'static str,
}

impl Default for CloseMessage {
    fn default() -> Self {
        Self {
            message_type: "close",
        }
    }
}

impl CloseMessage {
    /// Create a new close message
    pub fn new() -> Self {
        Self::default()
    }
}

/// Union type for all client messages
///
/// This makes it easier to serialize any client message.
#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ClientMessage {
    /// Audio chunk message
    InputAudioChunk(InputAudioChunk),
    /// Manual commit message
    Commit(CommitMessage),
    /// Close connection message
    Close(CloseMessage),
}

// ============================================================================
// Server -> Client Messages
// ============================================================================

/// Messages received from the server
///
/// Uses serde's tagged enum feature to automatically deserialize
/// based on the `message_type` field.
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "message_type")]
pub enum ServerMessage {
    /// Session has been started
    #[serde(rename = "session_started")]
    SessionStarted {
        /// Unique session identifier
        session_id: String,
        /// Session configuration
        #[serde(default)]
        config: Option<SessionConfig>,
    },

    /// Partial transcription result (real-time updates)
    #[serde(rename = "partial_transcript")]
    PartialTranscript {
        /// Partial transcription text
        text: String,
    },

    /// Final committed transcription
    #[serde(rename = "committed_transcript")]
    CommittedTranscript {
        /// Final transcription text
        text: String,
    },

    /// Committed transcription with word-level timestamps
    #[serde(rename = "committed_transcript_with_timestamps")]
    CommittedTranscriptWithTimestamps {
        /// Transcription text
        text: String,
        /// Detected language code
        language_code: String,
        /// Word-level timing information
        words: Vec<WordTimestamp>,
    },

    /// Input error from the server
    #[serde(rename = "input_error")]
    InputError {
        /// Error message description
        error_message: String,
    },
}

impl ServerMessage {
    /// Check if this is a partial transcript
    pub fn is_partial(&self) -> bool {
        matches!(self, ServerMessage::PartialTranscript { .. })
    }

    /// Check if this is a committed transcript
    pub fn is_committed(&self) -> bool {
        matches!(
            self,
            ServerMessage::CommittedTranscript { .. }
                | ServerMessage::CommittedTranscriptWithTimestamps { .. }
        )
    }

    /// Check if this is an error
    pub fn is_error(&self) -> bool {
        matches!(self, ServerMessage::InputError { .. })
    }

    /// Get the transcript text if this is a transcript message
    pub fn text(&self) -> Option<&str> {
        match self {
            ServerMessage::PartialTranscript { text } => Some(text),
            ServerMessage::CommittedTranscript { text } => Some(text),
            ServerMessage::CommittedTranscriptWithTimestamps { text, .. } => Some(text),
            _ => None,
        }
    }

    /// Get the error message if this is an error
    pub fn error_message(&self) -> Option<&str> {
        match self {
            ServerMessage::InputError { error_message } => Some(error_message),
            _ => None,
        }
    }

    /// Get the session ID if this is a session started message
    pub fn session_id(&self) -> Option<&str> {
        match self {
            ServerMessage::SessionStarted { session_id, .. } => Some(session_id),
            _ => None,
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Session configuration returned by the server
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SessionConfig {
    /// Sample rate in Hz
    #[serde(default)]
    pub sample_rate: u32,

    /// Audio format (e.g., "pcm_s16le")
    #[serde(default)]
    pub audio_format: String,

    /// Language code (e.g., "zh", "en")
    #[serde(default)]
    pub language_code: Option<String>,

    /// Model identifier
    #[serde(default)]
    pub model_id: String,

    /// Voice activity detection configuration
    #[serde(default)]
    pub vad_commit_strategy: Option<VadConfig>,
}

/// Voice Activity Detection (VAD) configuration
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct VadConfig {
    /// VAD strategy ("auto", "manual", etc.)
    pub strategy: String,

    /// Silence duration threshold in milliseconds
    #[serde(default)]
    pub silence_duration_ms: Option<u32>,

    /// Minimum speech duration in milliseconds
    #[serde(default)]
    pub min_speech_duration_ms: Option<u32>,
}

/// Word-level timestamp information
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct WordTimestamp {
    /// The word text
    pub word: String,

    /// Start time in seconds
    pub start: f64,

    /// End time in seconds
    pub end: f64,

    /// Word type ("word", "punctuation", etc.)
    #[serde(rename = "type")]
    pub word_type: String,

    /// Log probability (confidence score)
    #[serde(default)]
    pub logprob: Option<f64>,
}

impl WordTimestamp {
    /// Get the duration of this word in seconds
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }

    /// Check if this is a punctuation mark
    pub fn is_punctuation(&self) -> bool {
        self.word_type == "punctuation"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_audio_chunk_basic() {
        let chunk = InputAudioChunk::new("SGVsbG8=".to_string());

        assert_eq!(chunk.message_type, "input_audio_chunk");
        assert_eq!(chunk.audio_base_64, "SGVsbG8=");
        assert_eq!(chunk.commit, None);
        assert_eq!(chunk.sample_rate, None);
    }

    #[test]
    fn test_input_audio_chunk_with_options() {
        let chunk = InputAudioChunk::new("SGVsbG8=".to_string())
            .with_sample_rate(16000)
            .with_commit()
            .with_previous_text("Previous text".to_string());

        assert_eq!(chunk.sample_rate, Some(16000));
        assert_eq!(chunk.commit, Some(true));
        assert_eq!(chunk.previous_text, Some("Previous text".to_string()));
    }

    #[test]
    fn test_input_audio_chunk_serialization() {
        let chunk = InputAudioChunk::new("SGVsbG8=".to_string()).with_sample_rate(16000);

        let json = serde_json::to_string(&chunk).unwrap();

        assert!(json.contains("\"message_type\":\"input_audio_chunk\""));
        assert!(json.contains("\"audio_base_64\":\"SGVsbG8=\""));
        assert!(json.contains("\"sample_rate\":16000"));
        assert!(!json.contains("\"commit\"")); // Should be omitted
    }

    #[test]
    fn test_commit_message() {
        let msg = CommitMessage::new();

        assert_eq!(msg.message_type, "commit");

        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, "{\"message_type\":\"commit\"}");
    }

    #[test]
    fn test_close_message() {
        let msg = CloseMessage::new();

        assert_eq!(msg.message_type, "close");

        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, "{\"message_type\":\"close\"}");
    }

    #[test]
    fn test_server_message_session_started() {
        let json = r#"{
            "message_type": "session_started",
            "session_id": "test-session-123"
        }"#;

        let msg: ServerMessage = serde_json::from_str(json).unwrap();

        match &msg {
            ServerMessage::SessionStarted { session_id, .. } => {
                assert_eq!(session_id, "test-session-123");
            }
            _ => panic!("Expected SessionStarted"),
        }

        assert_eq!(msg.session_id(), Some("test-session-123"));
    }

    #[test]
    fn test_server_message_partial_transcript() {
        let json = r#"{
            "message_type": "partial_transcript",
            "text": "Hello world"
        }"#;

        let msg: ServerMessage = serde_json::from_str(json).unwrap();

        assert!(msg.is_partial());
        assert_eq!(msg.text(), Some("Hello world"));

        match msg {
            ServerMessage::PartialTranscript { text } => {
                assert_eq!(text, "Hello world");
            }
            _ => panic!("Expected PartialTranscript"),
        }
    }

    #[test]
    fn test_server_message_committed_transcript() {
        let json = r#"{
            "message_type": "committed_transcript",
            "text": "Final text"
        }"#;

        let msg: ServerMessage = serde_json::from_str(json).unwrap();

        assert!(msg.is_committed());
        assert_eq!(msg.text(), Some("Final text"));
    }

    #[test]
    fn test_server_message_committed_with_timestamps() {
        let json = r#"{
            "message_type": "committed_transcript_with_timestamps",
            "text": "Hello world",
            "language_code": "en",
            "words": [
                {
                    "word": "Hello",
                    "start": 0.0,
                    "end": 0.5,
                    "type": "word",
                    "logprob": -1.5
                },
                {
                    "word": "world",
                    "start": 0.6,
                    "end": 1.0,
                    "type": "word"
                }
            ]
        }"#;

        let msg: ServerMessage = serde_json::from_str(json).unwrap();

        assert!(msg.is_committed());
        assert_eq!(msg.text(), Some("Hello world"));

        match msg {
            ServerMessage::CommittedTranscriptWithTimestamps {
                text,
                language_code,
                words,
            } => {
                assert_eq!(text, "Hello world");
                assert_eq!(language_code, "en");
                assert_eq!(words.len(), 2);
                assert_eq!(words[0].word, "Hello");
                assert_eq!(words[0].duration(), 0.5);
                assert!(!words[0].is_punctuation());
            }
            _ => panic!("Expected CommittedTranscriptWithTimestamps"),
        }
    }

    #[test]
    fn test_server_message_input_error() {
        let json = r#"{
            "message_type": "input_error",
            "error_message": "Invalid audio format"
        }"#;

        let msg: ServerMessage = serde_json::from_str(json).unwrap();

        assert!(msg.is_error());
        assert_eq!(msg.error_message(), Some("Invalid audio format"));
    }

    #[test]
    fn test_word_timestamp_duration() {
        let word = WordTimestamp {
            word: "test".to_string(),
            start: 1.0,
            end: 1.5,
            word_type: "word".to_string(),
            logprob: None,
        };

        assert_eq!(word.duration(), 0.5);
        assert!(!word.is_punctuation());
    }

    #[test]
    fn test_word_timestamp_punctuation() {
        let word = WordTimestamp {
            word: ".".to_string(),
            start: 1.0,
            end: 1.05,
            word_type: "punctuation".to_string(),
            logprob: None,
        };

        assert!(word.is_punctuation());
    }

    #[test]
    fn test_session_config_deserialization() {
        let json = r#"{
            "sample_rate": 16000,
            "audio_format": "pcm_s16le",
            "language_code": "zh",
            "model_id": "scribe_v2_realtime"
        }"#;

        let config: SessionConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.audio_format, "pcm_s16le");
        assert_eq!(config.language_code, Some("zh".to_string()));
        assert_eq!(config.model_id, "scribe_v2_realtime");
    }

    #[test]
    fn test_client_message_serialization() {
        let chunk = InputAudioChunk::new("test".to_string());
        let msg = ClientMessage::InputAudioChunk(chunk);

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"message_type\":\"input_audio_chunk\""));
    }
}
