/// WebSocket connection to ElevenLabs Scribe v2 Realtime API
///
/// This module provides the WebSocket client for speech-to-text streaming.

use crate::network::error::{NetworkError, NetworkResult};
use crate::network::messages::ServerMessage;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        http::{Request, Uri},
        Message,
    },
    MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, info, warn};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Write half of the WebSocket stream
pub type WsWriter = SplitSink<WsStream, Message>;

/// Read half of the WebSocket stream
pub type WsReader = SplitStream<WsStream>;

/// Configuration for WebSocket connection
///
/// Contains parameters needed to establish a connection to the
/// ElevenLabs Scribe v2 API.
///
/// # Example
/// ```no_run
/// use raflow_lib::network::ConnectionConfig;
///
/// let config = ConnectionConfig::new(16000)
///     .with_model("scribe_v2_realtime")
///     .with_language("zh");
/// ```
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Model ID (default: "scribe_v2_realtime")
    pub model_id: String,

    /// Language code (e.g., "zh", "en")
    pub language_code: Option<String>,

    /// Audio sample rate in Hz (typically 16000)
    pub sample_rate: u32,

    /// Whether to include timestamps in results
    pub include_timestamps: bool,

    /// Voice activity detection strategy
    pub vad_commit_strategy: Option<String>,

    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
}

impl ConnectionConfig {
    /// Create a new configuration with the specified sample rate
    ///
    /// # Arguments
    /// * `sample_rate` - Audio sample rate in Hz (typically 16000)
    pub fn new(sample_rate: u32) -> Self {
        Self {
            model_id: "scribe_v2_realtime".to_string(),
            language_code: None,
            sample_rate,
            include_timestamps: false,
            vad_commit_strategy: None,
            timeout_ms: 10000, // 10 seconds default
        }
    }

    /// Set the model ID
    pub fn with_model(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = model_id.into();
        self
    }

    /// Set the language code
    pub fn with_language(mut self, language_code: impl Into<String>) -> Self {
        self.language_code = Some(language_code.into());
        self
    }

    /// Enable timestamps in transcription results
    pub fn with_timestamps(mut self) -> Self {
        self.include_timestamps = true;
        self
    }

    /// Set VAD commit strategy
    pub fn with_vad_strategy(mut self, strategy: impl Into<String>) -> Self {
        self.vad_commit_strategy = Some(strategy.into());
        self
    }

    /// Set connection timeout in milliseconds
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Build the WebSocket URL with query parameters
    pub fn build_url(&self) -> NetworkResult<String> {
        let mut url = format!(
            "wss://api.elevenlabs.io/v1/speech-to-text/realtime?model_id={}&sample_rate={}",
            self.model_id, self.sample_rate
        );

        if let Some(ref lang) = self.language_code {
            url.push_str(&format!("&language_code={}", lang));
        }

        if self.include_timestamps {
            url.push_str("&include_timestamps=true");
        }

        if let Some(ref vad) = self.vad_commit_strategy {
            url.push_str(&format!("&vad_commit_strategy={}", vad));
        }

        Ok(url)
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self::new(16000)
    }
}

/// WebSocket connection to ElevenLabs Scribe v2 API
///
/// Manages the WebSocket connection lifecycle and provides methods for
/// sending and receiving messages.
///
/// # Example
/// ```no_run
/// use raflow_lib::network::{ScribeConnection, ConnectionConfig};
///
/// #[tokio::main]
/// async fn main() {
///     let config = ConnectionConfig::new(16000);
///     let mut conn = ScribeConnection::connect("your-api-key", &config)
///         .await
///         .unwrap();
///
///     // Use the connection...
///     conn.close().await.unwrap();
/// }
/// ```
#[derive(Debug)]
pub struct ScribeConnection {
    /// WebSocket stream
    ws_stream: WsStream,

    /// Whether the connection is open
    is_open: bool,
}

impl ScribeConnection {
    /// Connect to the ElevenLabs Scribe v2 API
    ///
    /// Establishes a WebSocket connection with the provided API key and configuration.
    ///
    /// # Arguments
    /// * `api_key` - ElevenLabs API key for authentication
    /// * `config` - Connection configuration
    ///
    /// # Returns
    /// A connected `ScribeConnection` instance
    ///
    /// # Errors
    /// Returns `NetworkError` if connection fails
    pub async fn connect(api_key: &str, config: &ConnectionConfig) -> NetworkResult<Self> {
        info!("Connecting to ElevenLabs Scribe API");

        // Build URL
        let url = config.build_url()?;
        debug!("Connection URL: {}", url);

        // Parse URI
        let uri: Uri = url
            .parse()
            .map_err(|e| NetworkError::InvalidConfig(format!("Invalid URL: {}", e)))?;

        // Build request with authentication header
        let request = Request::builder()
            .uri(uri)
            .header("xi-api-key", api_key)
            .header("Host", "api.elevenlabs.io")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .body(())
            .map_err(|e| NetworkError::HttpError(e.to_string()))?;

        // Connect with timeout
        let connect_future = connect_async(request);
        let timeout = tokio::time::Duration::from_millis(config.timeout_ms);

        let (ws_stream, response) = tokio::time::timeout(timeout, connect_future)
            .await
            .map_err(|_| NetworkError::Timeout(config.timeout_ms))?
            .map_err(|e| {
                if let tokio_tungstenite::tungstenite::Error::Http(resp) = &e {
                    if resp.status() == 401 {
                        return NetworkError::AuthenticationFailed;
                    }
                }
                NetworkError::ConnectionFailed(e.to_string())
            })?;

        info!(
            "Connected to ElevenLabs API (status: {})",
            response.status()
        );
        debug!("Response headers: {:?}", response.headers());

        Ok(Self {
            ws_stream,
            is_open: true,
        })
    }

    /// Send a message to the server
    ///
    /// Serializes the message to JSON and sends it over the WebSocket.
    ///
    /// # Arguments
    /// * `message` - Any message that implements `Serialize`
    ///
    /// # Errors
    /// Returns `NetworkError` if serialization or sending fails
    pub async fn send<T: Serialize>(&mut self, message: &T) -> NetworkResult<()> {
        if !self.is_open {
            return Err(NetworkError::ConnectionClosed);
        }

        // Serialize message to JSON
        let json = serde_json::to_string(message)?;
        debug!("Sending message: {}", json);

        // Send as text message
        self.ws_stream
            .send(Message::Text(json.into()))
            .await
            .map_err(|e| NetworkError::WebSocketError(e))?;

        Ok(())
    }

    /// Receive a message from the server
    ///
    /// Waits for the next message from the server and deserializes it.
    ///
    /// # Returns
    /// * `Ok(Some(message))` - A message was received
    /// * `Ok(None)` - Connection closed gracefully
    /// * `Err(error)` - An error occurred
    ///
    /// # Errors
    /// Returns `NetworkError` if receiving or deserialization fails
    pub async fn recv(&mut self) -> NetworkResult<Option<ServerMessage>> {
        if !self.is_open {
            return Ok(None);
        }

        match self.ws_stream.next().await {
            Some(Ok(Message::Text(text))) => {
                debug!("Received message: {}", text);

                // Deserialize JSON to ServerMessage
                let message: ServerMessage = serde_json::from_str(&text)?;

                Ok(Some(message))
            }
            Some(Ok(Message::Close(frame))) => {
                info!("Received close frame: {:?}", frame);
                self.is_open = false;
                Ok(None)
            }
            Some(Ok(Message::Ping(data))) => {
                debug!("Received ping, sending pong");
                self.ws_stream.send(Message::Pong(data)).await?;
                // Recursively wait for next message
                Box::pin(self.recv()).await
            }
            Some(Ok(Message::Pong(_))) => {
                debug!("Received pong");
                // Recursively wait for next message
                Box::pin(self.recv()).await
            }
            Some(Ok(msg)) => {
                warn!("Received unexpected message type: {:?}", msg);
                // Recursively wait for next message
                Box::pin(self.recv()).await
            }
            Some(Err(e)) => {
                self.is_open = false;
                Err(NetworkError::WebSocketError(e))
            }
            None => {
                info!("WebSocket stream ended");
                self.is_open = false;
                Ok(None)
            }
        }
    }

    /// Close the WebSocket connection
    ///
    /// Sends a close frame and waits for the connection to close.
    ///
    /// # Errors
    /// Returns `NetworkError` if closing fails
    pub async fn close(&mut self) -> NetworkResult<()> {
        if !self.is_open {
            return Ok(());
        }

        info!("Closing WebSocket connection");

        self.ws_stream
            .close(None)
            .await
            .map_err(|e| NetworkError::WebSocketError(e))?;

        self.is_open = false;
        info!("WebSocket connection closed");

        Ok(())
    }

    /// Check if the connection is open
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Split the connection into separate read and write halves
    ///
    /// This consumes the connection and returns the read and write halves,
    /// which can be used independently in separate tasks.
    ///
    /// # Returns
    /// A tuple of `(WsWriter, WsReader)` that can be used for concurrent
    /// sending and receiving operations.
    ///
    /// # Example
    /// ```no_run
    /// use raflow_lib::network::{ScribeConnection, ConnectionConfig};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = ConnectionConfig::new(16000);
    ///     let conn = ScribeConnection::connect("your-api-key", &config)
    ///         .await
    ///         .unwrap();
    ///
    ///     let (writer, reader) = conn.split();
    ///     // Now you can use writer and reader in separate tasks
    /// }
    /// ```
    pub fn split(self) -> (WsWriter, WsReader) {
        self.ws_stream.split()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_config_new() {
        let config = ConnectionConfig::new(16000);

        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.model_id, "scribe_v2_realtime");
        assert_eq!(config.language_code, None);
        assert!(!config.include_timestamps);
    }

    #[test]
    fn test_connection_config_builder() {
        let config = ConnectionConfig::new(16000)
            .with_model("custom_model")
            .with_language("zh")
            .with_timestamps()
            .with_vad_strategy("auto")
            .with_timeout(5000);

        assert_eq!(config.model_id, "custom_model");
        assert_eq!(config.language_code, Some("zh".to_string()));
        assert!(config.include_timestamps);
        assert_eq!(config.vad_commit_strategy, Some("auto".to_string()));
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_connection_config_build_url() {
        let config = ConnectionConfig::new(16000);
        let url = config.build_url().unwrap();

        assert!(url.contains("wss://api.elevenlabs.io"));
        assert!(url.contains("model_id=scribe_v2_realtime"));
        assert!(url.contains("sample_rate=16000"));
    }

    #[test]
    fn test_connection_config_build_url_with_options() {
        let config = ConnectionConfig::new(16000)
            .with_language("zh")
            .with_timestamps()
            .with_vad_strategy("auto");

        let url = config.build_url().unwrap();

        assert!(url.contains("language_code=zh"));
        assert!(url.contains("include_timestamps=true"));
        assert!(url.contains("vad_commit_strategy=auto"));
    }

    #[test]
    fn test_connection_config_default() {
        let config = ConnectionConfig::default();

        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.model_id, "scribe_v2_realtime");
    }
}
