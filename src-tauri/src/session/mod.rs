//! RaFlow 完整会话管理模块
//!
//! 整合 Phase 1 和 Phase 2 的所有组件，提供端到端的语音转写与文本注入流程
//!
//! # 功能
//!
//! - 完整的语音转写会话管理
//! - 自动状态转换（Idle -> Connecting -> Recording -> Processing -> Injecting -> Idle）
//! - 转写文本自动注入到目标应用
//! - 前端事件通知
//!
//! # 使用示例
//!
//! ```ignore
//! use raflow_lib::session::{RaFlowSession, SessionConfig};
//!
//! // 创建会话配置
//! let config = SessionConfig::default();
//!
//! // 启动会话
//! let mut session = RaFlowSession::start(&app_handle, "api-key", config).await?;
//!
//! // 等待用户停止
//! // ...
//!
//! // 停止会话
//! session.stop().await?;
//! ```
//!
//! # 工作流程
//!
//! ```text
//! 1. 用户按下热键
//!    └── State: Idle -> Connecting
//!
//! 2. 建立 WebSocket 连接，启动音频采集
//!    └── State: Connecting -> Recording(Listening)
//!
//! 3. 接收部分转写
//!    └── State: Recording(Listening) -> Recording(Transcribing)
//!    └── Event: transcript:partial
//!
//! 4. 用户松开热键
//!    └── State: Recording -> Processing
//!
//! 5. 接收最终转写
//!    └── State: Processing -> Injecting
//!    └── 执行文本注入
//!
//! 6. 注入完成
//!    └── State: Injecting -> Idle
//!    └── Event: transcript:committed
//! ```

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{mpsc, oneshot, Mutex};

use crate::input::{InjectionStrategy, TextInjector};
use crate::state::{AppState, StateManager, StateTransitionContext};
use crate::transcription::{TranscriptEvent, TranscriptionError, TranscriptionSession};

/// 会话配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// 文本注入策略
    pub injection_strategy: InjectionStrategy,
    /// 自动策略阈值（字符数）
    pub auto_threshold: usize,
    /// 粘贴延迟（毫秒）
    pub paste_delay_ms: u64,
    /// 注入前延迟（毫秒）
    pub pre_injection_delay_ms: u64,
    /// 是否自动注入（false = 仅复制到剪贴板）
    pub auto_inject: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            injection_strategy: InjectionStrategy::Auto,
            auto_threshold: 20,
            paste_delay_ms: 100,
            pre_injection_delay_ms: 50,
            auto_inject: true,
        }
    }
}

impl SessionConfig {
    /// 创建仅复制到剪贴板的配置
    pub fn clipboard_only() -> Self {
        Self {
            injection_strategy: InjectionStrategy::ClipboardOnly,
            auto_inject: false,
            ..Default::default()
        }
    }

    /// 创建使用键盘注入的配置
    pub fn keyboard_only() -> Self {
        Self {
            injection_strategy: InjectionStrategy::Keyboard,
            ..Default::default()
        }
    }

    /// 创建使用剪贴板粘贴的配置
    pub fn clipboard_paste() -> Self {
        Self {
            injection_strategy: InjectionStrategy::Clipboard,
            ..Default::default()
        }
    }
}

/// 会话事件
///
/// 发送到前端的会话事件
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum SessionEvent {
    /// 会话开始
    Started { session_id: String },
    /// 部分转写
    PartialTranscript { text: String },
    /// 最终转写
    CommittedTranscript { text: String },
    /// 文本已注入
    TextInjected { text: String, strategy: String },
    /// 文本已复制到剪贴板
    TextCopied { text: String },
    /// 会话结束
    Stopped,
    /// 错误发生
    Error { message: String },
}

/// RaFlow 完整会话
///
/// 管理从语音输入到文本注入的完整流程
pub struct RaFlowSession {
    /// Tauri 应用句柄
    app: AppHandle,
    /// 会话配置
    config: SessionConfig,
    /// 转写会话
    transcription: Option<TranscriptionSession>,
    /// 状态管理器
    state_manager: Arc<StateManager>,
    /// 停止信号发送器
    stop_tx: Option<oneshot::Sender<()>>,
    /// 是否正在运行
    is_running: bool,
    /// 最后的最终转写文本
    last_committed_text: Arc<Mutex<Option<String>>>,
}

impl RaFlowSession {
    /// 启动新的 RaFlow 会话
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri 应用句柄
    /// * `api_key` - ElevenLabs API 密钥
    /// * `config` - 会话配置
    ///
    /// # Returns
    ///
    /// 返回正在运行的会话实例
    ///
    /// # Errors
    ///
    /// - `SessionError::StateError` - 状态转换失败
    /// - `SessionError::TranscriptionError` - 转写会话启动失败
    pub async fn start(
        app: &AppHandle,
        api_key: &str,
        config: SessionConfig,
    ) -> Result<Self, SessionError> {
        tracing::info!(strategy = ?config.injection_strategy, "Starting RaFlow session");

        // 获取或创建状态管理器
        let state_manager = Self::get_or_create_state_manager(app)?;

        // 创建状态转换上下文
        let ctx = StateTransitionContext::new(app, Arc::clone(&state_manager));

        // 转换到 Connecting 状态
        ctx.start_connecting()
            .map_err(|e| SessionError::StateError(e.to_string()))?;

        // 发射事件
        let _ = app.emit("session:connecting", ());

        // 创建共享数据
        let last_committed = Arc::new(Mutex::new(None::<String>));
        let last_committed_clone = Arc::clone(&last_committed);

        // 创建用于注入的 channel
        let (inject_tx, mut inject_rx) = mpsc::channel::<String>(10);

        let app_clone = app.clone();
        let state_manager_clone = Arc::clone(&state_manager);
        let config_clone = config.clone();

        // 启动转写会话
        let transcription = TranscriptionSession::start(api_key, move |event| {
            let ctx = StateTransitionContext::new(&app_clone, Arc::clone(&state_manager_clone));

            match event {
                TranscriptEvent::SessionStarted { session_id } => {
                    tracing::info!(session_id = %session_id, "Transcription session started");

                    // 转换到 Recording 状态
                    if let Err(e) = ctx.start_recording() {
                        tracing::error!(error = %e, "Failed to transition to Recording");
                    }

                    // 发射事件
                    let _ = app_clone.emit(
                        "session:event",
                        SessionEvent::Started {
                            session_id: session_id.clone(),
                        },
                    );
                }
                TranscriptEvent::Partial { text } => {
                    tracing::debug!(text = %text, "Partial transcript");

                    // 更新部分转写
                    if let Err(e) = ctx.update_partial(text.clone(), 0.5) {
                        tracing::warn!(error = %e, "Failed to update partial text");
                    }

                    // 发射事件
                    let _ = app_clone.emit(
                        "session:event",
                        SessionEvent::PartialTranscript { text },
                    );
                }
                TranscriptEvent::Committed { text } => {
                    tracing::info!(text = %text, "Committed transcript");

                    // 保存最终文本
                    {
                        if let Ok(mut guard) = last_committed_clone.try_lock() {
                            *guard = Some(text.clone());
                        }
                    }

                    // 转换到 Processing 状态
                    if let Err(e) = ctx.start_processing() {
                        tracing::error!(error = %e, "Failed to transition to Processing");
                    }

                    // 发送到注入 channel
                    if config_clone.auto_inject {
                        let _ = inject_tx.try_send(text.clone());
                    }

                    // 发射事件
                    let _ = app_clone.emit(
                        "session:event",
                        SessionEvent::CommittedTranscript { text },
                    );
                }
                TranscriptEvent::Error { message } => {
                    tracing::error!(error = %message, "Transcription error");

                    // 报告错误
                    if let Err(e) = ctx.report_error(&message) {
                        tracing::error!(error = %e, "Failed to report error");
                    }

                    // 发射事件
                    let _ = app_clone.emit("session:event", SessionEvent::Error { message });
                }
                TranscriptEvent::Closed => {
                    tracing::info!("Transcription session closed");
                }
            }
        })
        .await
        .map_err(SessionError::TranscriptionError)?;

        // 启动注入处理任务
        let app_inject = app.clone();
        let state_manager_inject = Arc::clone(&state_manager);
        let config_inject = config.clone();

        tokio::spawn(async move {
            while let Some(text) = inject_rx.recv().await {
                Self::handle_injection(
                    &app_inject,
                    &state_manager_inject,
                    &text,
                    &config_inject,
                )
                .await;
            }
            tracing::debug!("Injection handler stopped");
        });

        tracing::info!("RaFlow session started successfully");

        Ok(Self {
            app: app.clone(),
            config,
            transcription: Some(transcription),
            state_manager,
            stop_tx: None,
            is_running: true,
            last_committed_text: last_committed,
        })
    }

    /// 处理文本注入
    async fn handle_injection(
        app: &AppHandle,
        state_manager: &Arc<StateManager>,
        text: &str,
        config: &SessionConfig,
    ) {
        let ctx = StateTransitionContext::new(app, Arc::clone(state_manager));

        // 转换到 Injecting 状态
        if let Err(e) = ctx.start_injecting() {
            tracing::error!(error = %e, "Failed to transition to Injecting");
            return;
        }

        // 注入前延迟
        if config.pre_injection_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(config.pre_injection_delay_ms)).await;
        }

        // 执行注入
        let result = Self::inject_text(app, text, config).await;

        match result {
            Ok(strategy_name) => {
                tracing::info!(text_len = text.len(), strategy = %strategy_name, "Text injected");

                // 发射事件
                if config.injection_strategy == InjectionStrategy::ClipboardOnly {
                    let _ = app.emit(
                        "session:event",
                        SessionEvent::TextCopied {
                            text: text.to_string(),
                        },
                    );
                } else {
                    let _ = app.emit(
                        "session:event",
                        SessionEvent::TextInjected {
                            text: text.to_string(),
                            strategy: strategy_name,
                        },
                    );
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to inject text");

                // 报告错误
                let _ = ctx.report_error(e.to_string());
            }
        }

        // 完成，回到 Idle
        ctx.complete();
    }

    /// 执行文本注入
    async fn inject_text(
        app: &AppHandle,
        text: &str,
        config: &SessionConfig,
    ) -> Result<String, SessionError> {
        let mut injector = TextInjector::with_config(
            app,
            config.injection_strategy,
            config.auto_threshold,
            config.paste_delay_ms,
        )
        .map_err(|e| SessionError::InjectionError(e.to_string()))?;

        injector
            .inject(text)
            .await
            .map_err(|e| SessionError::InjectionError(e.to_string()))?;

        Ok(config.injection_strategy.display_name().to_string())
    }

    /// 停止会话
    ///
    /// 停止转写和所有相关任务
    pub async fn stop(&mut self) -> Result<(), SessionError> {
        if !self.is_running {
            return Ok(());
        }

        tracing::info!("Stopping RaFlow session");

        // 停止转写会话
        if let Some(mut transcription) = self.transcription.take() {
            transcription
                .stop()
                .await
                .map_err(SessionError::TranscriptionError)?;
        }

        // 重置状态
        self.state_manager.reset();

        // 发射停止事件
        let _ = self.app.emit("session:event", SessionEvent::Stopped);

        self.is_running = false;

        tracing::info!("RaFlow session stopped");

        Ok(())
    }

    /// 检查会话是否正在运行
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// 获取当前状态
    pub fn current_state(&self) -> Arc<AppState> {
        self.state_manager.current()
    }

    /// 获取最后的最终转写文本
    pub async fn last_committed_text(&self) -> Option<String> {
        self.last_committed_text.lock().await.clone()
    }

    /// 获取会话配置
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// 获取或创建状态管理器
    fn get_or_create_state_manager(app: &AppHandle) -> Result<Arc<StateManager>, SessionError> {
        // 尝试从应用状态获取
        if let Some(manager) = app.try_state::<Arc<StateManager>>() {
            return Ok(Arc::clone(&manager));
        }

        // 创建新的状态管理器
        let manager = Arc::new(StateManager::new());

        // 注册到应用状态
        app.manage(Arc::clone(&manager));

        Ok(manager)
    }

    /// 手动触发文本注入
    ///
    /// 用于在 ClipboardOnly 模式下手动注入已复制的文本
    pub async fn inject_last_committed(&mut self) -> Result<(), SessionError> {
        let text = self.last_committed_text.lock().await.clone();

        if let Some(text) = text {
            Self::handle_injection(&self.app, &self.state_manager, &text, &self.config).await;
            Ok(())
        } else {
            Err(SessionError::NoTextToInject)
        }
    }
}

impl Drop for RaFlowSession {
    fn drop(&mut self) {
        // 尝试同步停止
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// 会话错误
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// 状态错误
    #[error("State error: {0}")]
    StateError(String),

    /// 转写错误
    #[error("Transcription error: {0}")]
    TranscriptionError(#[from] TranscriptionError),

    /// 注入错误
    #[error("Injection error: {0}")]
    InjectionError(String),

    /// 没有文本可注入
    #[error("No text to inject")]
    NoTextToInject,

    /// 会话未运行
    #[error("Session is not running")]
    NotRunning,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.injection_strategy, InjectionStrategy::Auto);
        assert_eq!(config.auto_threshold, 20);
        assert_eq!(config.paste_delay_ms, 100);
        assert!(config.auto_inject);
    }

    #[test]
    fn test_session_config_clipboard_only() {
        let config = SessionConfig::clipboard_only();
        assert_eq!(config.injection_strategy, InjectionStrategy::ClipboardOnly);
        assert!(!config.auto_inject);
    }

    #[test]
    fn test_session_config_keyboard_only() {
        let config = SessionConfig::keyboard_only();
        assert_eq!(config.injection_strategy, InjectionStrategy::Keyboard);
        assert!(config.auto_inject);
    }

    #[test]
    fn test_session_config_clipboard_paste() {
        let config = SessionConfig::clipboard_paste();
        assert_eq!(config.injection_strategy, InjectionStrategy::Clipboard);
        assert!(config.auto_inject);
    }

    #[test]
    fn test_session_config_serialization() {
        let config = SessionConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SessionConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.injection_strategy, deserialized.injection_strategy);
        assert_eq!(config.auto_threshold, deserialized.auto_threshold);
        assert_eq!(config.paste_delay_ms, deserialized.paste_delay_ms);
        assert_eq!(config.auto_inject, deserialized.auto_inject);
    }

    #[test]
    fn test_session_event_started() {
        let event = SessionEvent::Started {
            session_id: "test-123".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Started"));
        assert!(json.contains("test-123"));
    }

    #[test]
    fn test_session_event_partial_transcript() {
        let event = SessionEvent::PartialTranscript {
            text: "hello world".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("PartialTranscript"));
        assert!(json.contains("hello world"));
    }

    #[test]
    fn test_session_event_committed_transcript() {
        let event = SessionEvent::CommittedTranscript {
            text: "final text".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("CommittedTranscript"));
        assert!(json.contains("final text"));
    }

    #[test]
    fn test_session_event_text_injected() {
        let event = SessionEvent::TextInjected {
            text: "injected text".to_string(),
            strategy: "Auto".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("TextInjected"));
        assert!(json.contains("injected text"));
        assert!(json.contains("Auto"));
    }

    #[test]
    fn test_session_event_text_copied() {
        let event = SessionEvent::TextCopied {
            text: "copied text".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("TextCopied"));
        assert!(json.contains("copied text"));
    }

    #[test]
    fn test_session_event_stopped() {
        let event = SessionEvent::Stopped;
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Stopped"));
    }

    #[test]
    fn test_session_event_error() {
        let event = SessionEvent::Error {
            message: "test error".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Error"));
        assert!(json.contains("test error"));
    }

    #[test]
    fn test_session_error_display() {
        let err = SessionError::StateError("test".to_string());
        assert!(err.to_string().contains("test"));

        let err = SessionError::InjectionError("injection failed".to_string());
        assert!(err.to_string().contains("injection failed"));

        let err = SessionError::NoTextToInject;
        assert!(err.to_string().contains("No text"));

        let err = SessionError::NotRunning;
        assert!(err.to_string().contains("not running"));
    }
}
