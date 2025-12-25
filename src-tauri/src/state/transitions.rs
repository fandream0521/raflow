//! 状态转换与事件发射模块
//!
//! 提供状态转换的高级封装，包括：
//! - 状态变更事件发射到前端
//! - Processing 状态超时处理
//! - 状态转换的统一接口

use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;

use super::{AppState, StateManager};

/// 默认 Processing 超时时间（秒）
pub const DEFAULT_PROCESSING_TIMEOUT_SECS: u64 = 30;

/// 状态变更事件载荷
///
/// 用于发送到前端的状态变更通知
#[derive(Debug, Clone, Serialize)]
pub struct StateChangeEvent {
    /// 状态名称
    pub state: String,
    /// 是否空闲
    pub is_idle: bool,
    /// 是否连接中
    pub is_connecting: bool,
    /// 是否录音中
    pub is_recording: bool,
    /// 是否处理中
    pub is_processing: bool,
    /// 是否注入中
    pub is_injecting: bool,
    /// 是否错误状态
    pub is_error: bool,
    /// 错误消息（如果有）
    pub error_message: Option<String>,
    /// 部分转写文本（如果有）
    pub partial_text: Option<String>,
}

impl From<&AppState> for StateChangeEvent {
    fn from(state: &AppState) -> Self {
        Self {
            state: state.name().to_string(),
            is_idle: state.is_idle(),
            is_connecting: state.is_connecting(),
            is_recording: state.is_recording(),
            is_processing: state.is_processing(),
            is_injecting: state.is_injecting(),
            is_error: state.is_error(),
            error_message: state.error_message().map(|s| s.to_string()),
            partial_text: state
                .recording_state()
                .and_then(|rs| rs.partial_text().map(|s| s.to_string())),
        }
    }
}

/// 状态事件发射器
///
/// 监听 StateManager 的状态变更并发射 Tauri 事件到前端
pub struct StateEventEmitter {
    /// 停止信号发送器
    stop_tx: Option<mpsc::Sender<()>>,
}

impl StateEventEmitter {
    /// 创建并启动状态事件发射器
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri 应用句柄
    /// * `state_manager` - 状态管理器引用
    pub async fn start(app: &AppHandle, state_manager: Arc<StateManager>) -> Self {
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
        let mut state_rx = state_manager.subscribe().await;
        let app_handle = app.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 接收状态变更
                    Some(new_state) = state_rx.recv() => {
                        Self::emit_state_change(&app_handle, &new_state);
                    }
                    // 接收停止信号
                    _ = stop_rx.recv() => {
                        tracing::debug!("StateEventEmitter stopped");
                        break;
                    }
                }
            }
        });

        tracing::info!("StateEventEmitter started");
        Self {
            stop_tx: Some(stop_tx),
        }
    }

    /// 发射状态变更事件到前端
    fn emit_state_change(app: &AppHandle, state: &AppState) {
        let event = StateChangeEvent::from(state);

        // 发射通用状态变更事件
        if let Err(e) = app.emit("app:state_changed", &event) {
            tracing::warn!(error = %e, "Failed to emit state change event");
        }

        // 发射特定状态事件
        match state {
            AppState::Idle => {
                let _ = app.emit("app:idle", ());
            }
            AppState::Connecting => {
                let _ = app.emit("app:connecting", ());
            }
            AppState::Recording(rs) => {
                let _ = app.emit("app:recording", rs.is_transcribing());
                if let Some(text) = rs.partial_text() {
                    let _ = app.emit("transcript:partial", text);
                }
            }
            AppState::Processing => {
                let _ = app.emit("app:processing", ());
            }
            AppState::Injecting => {
                let _ = app.emit("app:injecting", ());
            }
            AppState::Error(msg) => {
                let _ = app.emit("app:error", msg);
            }
        }

        tracing::debug!(state = %state.name(), "Emitted state change event");
    }

    /// 停止事件发射器
    pub async fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(()).await;
        }
    }
}

impl Drop for StateEventEmitter {
    fn drop(&mut self) {
        // 同步关闭，尽量发送停止信号
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.try_send(());
        }
    }
}

/// Processing 状态超时处理器
///
/// 监控 Processing 状态，超时后自动转换到 Idle
pub struct ProcessingTimeoutHandler {
    /// 停止信号发送器
    stop_tx: Option<mpsc::Sender<()>>,
}

impl ProcessingTimeoutHandler {
    /// 创建并启动超时处理器
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri 应用句柄
    /// * `state_manager` - 状态管理器引用
    /// * `timeout_secs` - 超时时间（秒）
    pub async fn start(
        app: &AppHandle,
        state_manager: Arc<StateManager>,
        timeout_secs: u64,
    ) -> Self {
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
        let mut state_rx = state_manager.subscribe().await;
        let app_handle = app.clone();

        tokio::spawn(async move {
            let mut processing_start: Option<tokio::time::Instant> = None;

            loop {
                let check_interval = Duration::from_millis(500);

                tokio::select! {
                    // 接收状态变更
                    Some(new_state) = state_rx.recv() => {
                        match &new_state {
                            AppState::Processing => {
                                // 进入 Processing 状态，开始计时
                                processing_start = Some(tokio::time::Instant::now());
                                tracing::debug!("Processing timeout started");
                            }
                            _ => {
                                // 离开 Processing 状态，停止计时
                                if processing_start.is_some() {
                                    tracing::debug!("Processing timeout cancelled");
                                }
                                processing_start = None;
                            }
                        }
                    }
                    // 定期检查超时
                    _ = tokio::time::sleep(check_interval) => {
                        if let Some(start) = processing_start {
                            if start.elapsed() >= Duration::from_secs(timeout_secs) {
                                tracing::warn!(
                                    timeout_secs = timeout_secs,
                                    "Processing timeout, resetting to Idle"
                                );

                                // 超时，重置状态
                                state_manager.reset();
                                processing_start = None;

                                // 发射超时事件
                                let _ = app_handle.emit("app:processing_timeout", ());
                            }
                        }
                    }
                    // 接收停止信号
                    _ = stop_rx.recv() => {
                        tracing::debug!("ProcessingTimeoutHandler stopped");
                        break;
                    }
                }
            }
        });

        tracing::info!(timeout_secs = timeout_secs, "ProcessingTimeoutHandler started");
        Self {
            stop_tx: Some(stop_tx),
        }
    }

    /// 停止超时处理器
    pub async fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(()).await;
        }
    }
}

impl Drop for ProcessingTimeoutHandler {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.try_send(());
        }
    }
}

/// 状态转换上下文
///
/// 提供状态转换的便捷方法，自动处理事件发射
pub struct StateTransitionContext {
    state_manager: Arc<StateManager>,
    app_handle: AppHandle,
}

impl StateTransitionContext {
    /// 创建状态转换上下文
    pub fn new(app: &AppHandle, state_manager: Arc<StateManager>) -> Self {
        Self {
            state_manager,
            app_handle: app.clone(),
        }
    }

    /// 从 AppHandle 获取状态转换上下文
    pub fn from_app(app: &AppHandle) -> Option<Self> {
        let state_manager = app.try_state::<Arc<StateManager>>()?;
        Some(Self {
            state_manager: Arc::clone(&state_manager),
            app_handle: app.clone(),
        })
    }

    /// 获取当前状态
    pub fn current(&self) -> Arc<AppState> {
        self.state_manager.current()
    }

    /// 开始连接
    ///
    /// 从 Idle 转换到 Connecting
    pub fn start_connecting(&self) -> Result<(), TransitionError> {
        self.transition_with_event(AppState::connecting())
    }

    /// 连接成功，开始录音
    ///
    /// 从 Connecting 转换到 Recording(Listening)
    pub fn start_recording(&self) -> Result<(), TransitionError> {
        self.transition_with_event(AppState::recording_listening())
    }

    /// 更新转写文本
    ///
    /// 在 Recording 状态内更新部分转写
    pub fn update_partial(&self, text: String, confidence: f32) -> Result<(), TransitionError> {
        self.transition_with_event(AppState::recording_transcribing(text, confidence))
    }

    /// 开始处理
    ///
    /// 从 Recording 转换到 Processing
    pub fn start_processing(&self) -> Result<(), TransitionError> {
        self.transition_with_event(AppState::processing())
    }

    /// 开始注入
    ///
    /// 从 Processing 转换到 Injecting
    pub fn start_injecting(&self) -> Result<(), TransitionError> {
        self.transition_with_event(AppState::injecting())
    }

    /// 完成，回到空闲
    ///
    /// 从任何状态重置到 Idle
    pub fn complete(&self) {
        self.state_manager.reset();
    }

    /// 取消当前操作
    ///
    /// 从 Connecting/Recording 转换到 Idle
    pub fn cancel(&self) -> Result<(), TransitionError> {
        let current = self.current();
        if current.is_connecting() || current.is_recording() {
            self.transition_with_event(AppState::idle())
        } else {
            Err(TransitionError::InvalidState {
                current: current.name().to_string(),
                action: "cancel".to_string(),
            })
        }
    }

    /// 报告错误
    ///
    /// 转换到 Error 状态
    pub fn report_error(&self, message: impl Into<String>) -> Result<(), TransitionError> {
        let msg = message.into();
        self.transition_with_event(AppState::error(msg.clone()))?;

        // 发射错误事件
        let _ = self.app_handle.emit("transcription:error", &msg);

        Ok(())
    }

    /// 从错误恢复
    ///
    /// 从 Error 转换到 Idle
    pub fn recover_from_error(&self) -> Result<(), TransitionError> {
        if self.current().is_error() {
            self.transition_with_event(AppState::idle())
        } else {
            Err(TransitionError::InvalidState {
                current: self.current().name().to_string(),
                action: "recover".to_string(),
            })
        }
    }

    /// 执行状态转换并发射事件
    fn transition_with_event(&self, new_state: AppState) -> Result<(), TransitionError> {
        self.state_manager
            .transition(new_state)
            .map_err(|e| TransitionError::TransitionFailed(e.to_string()))
    }
}

/// 状态转换错误
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum TransitionError {
    /// 状态转换失败
    #[error("State transition failed: {0}")]
    TransitionFailed(String),

    /// 无效状态
    #[error("Invalid state for action '{action}': current state is {current}")]
    InvalidState { current: String, action: String },
}

/// 初始化状态转换系统
///
/// 设置 StateEventEmitter 和 ProcessingTimeoutHandler
///
/// # Arguments
///
/// * `app` - Tauri 应用句柄
/// * `state_manager` - 状态管理器引用
/// * `processing_timeout_secs` - Processing 超时时间（秒），使用 None 表示默认值
pub async fn setup_state_transitions(
    app: &AppHandle,
    state_manager: Arc<StateManager>,
    processing_timeout_secs: Option<u64>,
) -> (StateEventEmitter, ProcessingTimeoutHandler) {
    let timeout_secs = processing_timeout_secs.unwrap_or(DEFAULT_PROCESSING_TIMEOUT_SECS);

    let event_emitter = StateEventEmitter::start(app, Arc::clone(&state_manager)).await;
    let timeout_handler =
        ProcessingTimeoutHandler::start(app, Arc::clone(&state_manager), timeout_secs).await;

    tracing::info!("State transition system initialized");

    (event_emitter, timeout_handler)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_change_event_from_idle() {
        let state = AppState::idle();
        let event = StateChangeEvent::from(&state);

        assert_eq!(event.state, "Idle");
        assert!(event.is_idle);
        assert!(!event.is_connecting);
        assert!(!event.is_recording);
        assert!(!event.is_processing);
        assert!(!event.is_injecting);
        assert!(!event.is_error);
        assert!(event.error_message.is_none());
        assert!(event.partial_text.is_none());
    }

    #[test]
    fn test_state_change_event_from_connecting() {
        let state = AppState::connecting();
        let event = StateChangeEvent::from(&state);

        assert_eq!(event.state, "Connecting");
        assert!(!event.is_idle);
        assert!(event.is_connecting);
        assert!(!event.is_recording);
        assert!(!event.is_processing);
        assert!(event.error_message.is_none());
    }

    #[test]
    fn test_state_change_event_from_recording_listening() {
        let state = AppState::recording_listening();
        let event = StateChangeEvent::from(&state);

        assert_eq!(event.state, "Recording::Listening");
        assert!(!event.is_idle);
        assert!(event.is_recording);
        assert!(!event.is_processing);
        assert!(event.partial_text.is_none());
    }

    #[test]
    fn test_state_change_event_from_recording_transcribing() {
        let state = AppState::recording_transcribing("hello world".to_string(), 0.95);
        let event = StateChangeEvent::from(&state);

        assert_eq!(event.state, "Recording::Transcribing");
        assert!(!event.is_idle);
        assert!(event.is_recording);
        assert_eq!(event.partial_text, Some("hello world".to_string()));
    }

    #[test]
    fn test_state_change_event_from_processing() {
        let state = AppState::processing();
        let event = StateChangeEvent::from(&state);

        assert_eq!(event.state, "Processing");
        assert!(!event.is_idle);
        assert!(!event.is_recording);
        assert!(event.is_processing);
        assert!(!event.is_injecting);
        assert!(event.error_message.is_none());
    }

    #[test]
    fn test_state_change_event_from_injecting() {
        let state = AppState::injecting();
        let event = StateChangeEvent::from(&state);

        assert_eq!(event.state, "Injecting");
        assert!(!event.is_idle);
        assert!(!event.is_processing);
        assert!(event.is_injecting);
        assert!(event.error_message.is_none());
    }

    #[test]
    fn test_state_change_event_from_error() {
        let state = AppState::error("Connection failed");
        let event = StateChangeEvent::from(&state);

        assert_eq!(event.state, "Error");
        assert!(!event.is_idle);
        assert!(event.is_error);
        assert_eq!(event.error_message, Some("Connection failed".to_string()));
    }

    #[test]
    fn test_state_change_event_serialization() {
        let state = AppState::recording_transcribing("test text".to_string(), 0.85);
        let event = StateChangeEvent::from(&state);

        // Test that it can be serialized to JSON
        let json = serde_json::to_string(&event).expect("Should serialize");
        assert!(json.contains("Recording::Transcribing"));
        assert!(json.contains("test text"));
        assert!(json.contains("is_recording"));
    }

    #[test]
    fn test_transition_error_display() {
        let err = TransitionError::TransitionFailed("invalid".to_string());
        assert!(err.to_string().contains("invalid"));

        let err = TransitionError::InvalidState {
            current: "Idle".to_string(),
            action: "cancel".to_string(),
        };
        assert!(err.to_string().contains("Idle"));
        assert!(err.to_string().contains("cancel"));
    }

    #[test]
    fn test_transition_error_equality() {
        let err1 = TransitionError::TransitionFailed("test".to_string());
        let err2 = TransitionError::TransitionFailed("test".to_string());
        assert_eq!(err1, err2);

        let err3 = TransitionError::TransitionFailed("other".to_string());
        assert_ne!(err1, err3);

        let err4 = TransitionError::InvalidState {
            current: "Idle".to_string(),
            action: "test".to_string(),
        };
        let err5 = TransitionError::InvalidState {
            current: "Idle".to_string(),
            action: "test".to_string(),
        };
        assert_eq!(err4, err5);
    }

    #[test]
    fn test_default_processing_timeout() {
        assert_eq!(DEFAULT_PROCESSING_TIMEOUT_SECS, 30);
    }
}
