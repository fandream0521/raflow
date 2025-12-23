/// Async tasks for sending and receiving WebSocket messages
///
/// This module provides task functions that can run concurrently
/// to handle audio data transmission and transcript reception.

use crate::network::connection::{WsReader, WsWriter};
use crate::network::error::{NetworkError, NetworkResult};
use crate::network::messages::{ClientMessage, InputAudioChunk, ServerMessage};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};

/// Sender task that reads audio data from a channel and sends it via WebSocket
///
/// This task continuously reads Base64-encoded audio chunks from the provided
/// receiver channel and sends them to the server via WebSocket. It runs until
/// either the channel is closed or an error occurs.
///
/// # Arguments
/// * `ws_writer` - The write half of the WebSocket stream
/// * `audio_rx` - Receiver for Base64-encoded audio data
///
/// # Returns
/// `Ok(())` if the task completes normally (channel closed), or an error
///
/// # Example
/// ```no_run
/// use tokio::sync::mpsc;
/// use raflow_lib::network::{ScribeConnection, ConnectionConfig};
/// use raflow_lib::network::tasks::sender_task;
///
/// #[tokio::main]
/// async fn main() {
///     let config = ConnectionConfig::new(16000);
///     let conn = ScribeConnection::connect("api-key", &config).await.unwrap();
///     let (writer, _reader) = conn.split();
///
///     let (audio_tx, audio_rx) = mpsc::channel(100);
///
///     tokio::spawn(async move {
///         sender_task(writer, audio_rx).await
///     });
/// }
/// ```
pub async fn sender_task(
    mut ws_writer: WsWriter,
    mut audio_rx: mpsc::Receiver<String>,
) -> NetworkResult<()> {
    info!("Sender task started");

    let mut chunk_count = 0u64;

    while let Some(audio_base64) = audio_rx.recv().await {
        chunk_count += 1;
        debug!(
            "Sending audio chunk #{} (size: {} bytes)",
            chunk_count,
            audio_base64.len()
        );

        // Create audio chunk message
        let chunk = InputAudioChunk::new(audio_base64);

        // For the first chunk, include sample rate
        let message = if chunk_count == 1 {
            chunk.with_sample_rate(16000)
        } else {
            chunk
        };

        // Serialize to JSON
        let json = serde_json::to_string(&ClientMessage::InputAudioChunk(message))
            .map_err(NetworkError::SerializationError)?;

        // Send via WebSocket
        ws_writer
            .send(Message::Text(json.into()))
            .await
            .map_err(NetworkError::WebSocketError)?;

        debug!("Audio chunk #{} sent successfully", chunk_count);
    }

    info!(
        "Sender task completed: {} chunks sent, channel closed",
        chunk_count
    );

    // Send close message before shutting down
    if let Err(e) = ws_writer.close().await {
        warn!("Failed to close WebSocket writer: {}", e);
    }

    Ok(())
}

/// Receiver task that reads messages from WebSocket and forwards them to a channel
///
/// This task continuously reads messages from the WebSocket stream, deserializes
/// them into `ServerMessage` types, and forwards them through the provided sender
/// channel. It handles ping/pong and close frames automatically.
///
/// # Arguments
/// * `ws_reader` - The read half of the WebSocket stream
/// * `message_tx` - Sender for forwarding received server messages
///
/// # Returns
/// `Ok(())` if the connection closes gracefully, or an error
///
/// # Example
/// ```no_run
/// use tokio::sync::mpsc;
/// use raflow_lib::network::{ScribeConnection, ConnectionConfig};
/// use raflow_lib::network::tasks::receiver_task;
///
/// #[tokio::main]
/// async fn main() {
///     let config = ConnectionConfig::new(16000);
///     let conn = ScribeConnection::connect("api-key", &config).await.unwrap();
///     let (_writer, reader) = conn.split();
///
///     let (msg_tx, mut msg_rx) = mpsc::channel(100);
///
///     tokio::spawn(async move {
///         receiver_task(reader, msg_tx).await
///     });
/// }
/// ```
pub async fn receiver_task(
    mut ws_reader: WsReader,
    message_tx: mpsc::Sender<ServerMessage>,
) -> NetworkResult<()> {
    info!("Receiver task started");

    let mut message_count = 0u64;

    while let Some(msg_result) = ws_reader.next().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                debug!("Received text message: {} bytes", text.len());

                // Deserialize the message
                match serde_json::from_str::<ServerMessage>(&text) {
                    Ok(server_msg) => {
                        message_count += 1;
                        debug!(
                            "Parsed message #{}: {:?}",
                            message_count,
                            std::mem::discriminant(&server_msg)
                        );

                        // Forward to channel
                        if let Err(e) = message_tx.send(server_msg).await {
                            warn!("Failed to forward message: receiver dropped ({})", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Failed to deserialize message: {}", e);
                        return Err(NetworkError::SerializationError(e));
                    }
                }
            }
            Ok(Message::Close(frame)) => {
                info!("Received close frame: {:?}", frame);
                break;
            }
            Ok(Message::Ping(data)) => {
                debug!("Received ping, length: {} bytes", data.len());
                // Pong is handled automatically by the underlying library
            }
            Ok(Message::Pong(_)) => {
                debug!("Received pong");
            }
            Ok(Message::Binary(data)) => {
                warn!("Received unexpected binary message: {} bytes", data.len());
            }
            Ok(Message::Frame(_)) => {
                // Raw frames are typically not exposed in normal operation
                debug!("Received raw frame");
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                return Err(NetworkError::WebSocketError(e));
            }
        }
    }

    info!(
        "Receiver task completed: {} messages received, stream ended",
        message_count
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audio_chunk_serialization() {
        let chunk = InputAudioChunk::new("dGVzdA==".to_string()).with_sample_rate(16000);

        let msg = ClientMessage::InputAudioChunk(chunk);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("input_audio_chunk"));
        assert!(json.contains("dGVzdA=="));
        assert!(json.contains("16000"));
    }

    #[tokio::test]
    async fn test_message_channel_behavior() {
        let (tx, mut rx) = mpsc::channel::<ServerMessage>(10);

        // Send a test message
        let test_msg = ServerMessage::PartialTranscript {
            text: "test".to_string(),
        };

        tx.send(test_msg.clone()).await.unwrap();

        // Receive the message
        let received = rx.recv().await.unwrap();
        assert_eq!(received, test_msg);
    }

    #[test]
    fn test_session_started_message_structure() {
        let json = r#"{
            "message_type": "session_started",
            "session_id": "test-123",
            "config": {
                "sample_rate": 16000,
                "audio_format": "pcm_16000",
                "model_id": "scribe_v2_realtime"
            }
        }"#;

        let msg: ServerMessage = serde_json::from_str(json).unwrap();

        match msg {
            ServerMessage::SessionStarted {
                session_id,
                config,
            } => {
                assert_eq!(session_id, "test-123");
                assert!(config.is_some());
            }
            _ => panic!("Expected SessionStarted message"),
        }
    }

    #[tokio::test]
    async fn test_channel_capacity() {
        // Test that channels work with the expected capacity
        let (tx, _rx) = mpsc::channel::<String>(100);

        // Should be able to send up to capacity without blocking
        for i in 0..100 {
            tx.send(format!("msg_{}", i)).await.unwrap();
        }
    }

    #[test]
    fn test_input_audio_chunk_builder() {
        let chunk = InputAudioChunk::new("base64data".to_string())
            .with_sample_rate(16000)
            .with_previous_text("context".to_string());

        assert_eq!(chunk.audio_base_64, "base64data");
        assert_eq!(chunk.sample_rate, Some(16000));
        assert_eq!(chunk.previous_text, Some("context".to_string()));
    }
}
