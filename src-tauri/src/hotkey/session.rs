//! 会话控制器模块
//!
//! 管理转写会话的生命周期，与热键处理器协作
//!
//! 由于 TranscriptionSession 包含 cpal::Stream（不是 Send + Sync），
//! 我们使用 channel 模式来控制会话，会话运行在专门的任务中。

use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, RwLock};

use crate::state::{AppState, StateManager};
use crate::transcription::{TranscriptEvent, TranscriptionSession};

/// 会话命令
#[derive(Debug)]
enum SessionCommand {
    /// 启动会话
    Start {
        api_key: String,
        response: oneshot::Sender<Result<(), SessionControllerError>>,
    },
    /// 停止会话
    Stop {
        response: oneshot::Sender<Result<Option<String>, SessionControllerError>>,
    },
    /// 取消会话
    Cancel {
        response: oneshot::Sender<Result<(), SessionControllerError>>,
    },
}

/// 会话状态
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    /// 空闲，没有活跃会话
    Idle,
    /// 正在启动会话
    Starting,
    /// 会话运行中
    Running,
    /// 正在停止会话
    Stopping,
}

/// 会话事件
///
/// 用于通知外部（如 UI）会话状态变化
#[derive(Debug, Clone)]
pub enum SessionEvent {
    /// 会话已启动
    Started { session_id: String },
    /// 收到部分转写
    PartialTranscript { text: String },
    /// 收到最终转写
    CommittedTranscript { text: String },
    /// 会话错误
    Error { message: String },
    /// 会话已关闭
    Closed,
}

/// 会话事件发送器类型
pub type SessionEventSender = mpsc::Sender<SessionEvent>;

/// 会话控制器
///
/// 管理转写会话的生命周期。
/// 由于 TranscriptionSession 不是 Send + Sync，
/// 我们使用 channel 模式，在专门的任务中运行会话。
pub struct SessionController {
    /// 命令发送器
    command_tx: mpsc::Sender<SessionCommand>,
    /// 会话状态
    state: Arc<RwLock<SessionState>>,
    /// 事件发送器（用于通知 UI）
    event_tx: Arc<RwLock<Option<SessionEventSender>>>,
    /// 最后的 committed 文本（用于注入）
    last_committed_text: Arc<RwLock<Option<String>>>,
}

impl SessionController {
    /// 创建新的会话控制器
    ///
    /// # Arguments
    ///
    /// * `state_manager` - 状态管理器引用
    pub fn new(state_manager: Arc<StateManager>) -> Self {
        let (command_tx, command_rx) = mpsc::channel::<SessionCommand>(16);
        let state = Arc::new(RwLock::new(SessionState::Idle));
        let event_tx = Arc::new(RwLock::new(None::<SessionEventSender>));
        let last_committed_text = Arc::new(RwLock::new(None::<String>));

        // 启动会话管理任务
        // 使用专用线程来运行会话任务，因为 TranscriptionSession 不是 Send
        let state_clone = Arc::clone(&state);
        let state_manager_clone = Arc::clone(&state_manager);
        let event_tx_clone = Arc::clone(&event_tx);
        let last_committed_clone = Arc::clone(&last_committed_text);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create session runtime");

            rt.block_on(async move {
                session_task(
                    command_rx,
                    state_clone,
                    state_manager_clone,
                    event_tx_clone,
                    last_committed_clone,
                )
                .await;
            });
        });

        Self {
            command_tx,
            state,
            event_tx,
            last_committed_text,
        }
    }

    /// 设置事件发送器
    pub async fn set_event_sender(&self, tx: SessionEventSender) {
        let mut event_tx = self.event_tx.write().await;
        *event_tx = Some(tx);
    }

    /// 获取当前会话状态
    pub async fn session_state(&self) -> SessionState {
        self.state.read().await.clone()
    }

    /// 获取最后的 committed 文本
    pub async fn take_last_committed_text(&self) -> Option<String> {
        let mut text = self.last_committed_text.write().await;
        text.take()
    }

    /// 启动转写会话
    pub async fn start_session(&self, api_key: &str) -> Result<(), SessionControllerError> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(SessionCommand::Start {
                api_key: api_key.to_string(),
                response: response_tx,
            })
            .await
            .map_err(|_| SessionControllerError::ChannelClosed)?;

        response_rx
            .await
            .map_err(|_| SessionControllerError::ChannelClosed)?
    }

    /// 停止转写会话并获取最终结果
    pub async fn stop_session(&self) -> Result<Option<String>, SessionControllerError> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(SessionCommand::Stop {
                response: response_tx,
            })
            .await
            .map_err(|_| SessionControllerError::ChannelClosed)?;

        response_rx
            .await
            .map_err(|_| SessionControllerError::ChannelClosed)?
    }

    /// 取消转写会话
    pub async fn cancel_session(&self) -> Result<(), SessionControllerError> {
        let (response_tx, response_rx) = oneshot::channel();

        self.command_tx
            .send(SessionCommand::Cancel {
                response: response_tx,
            })
            .await
            .map_err(|_| SessionControllerError::ChannelClosed)?;

        response_rx
            .await
            .map_err(|_| SessionControllerError::ChannelClosed)?
    }

    /// 检查会话是否正在运行
    pub async fn is_running(&self) -> bool {
        let state = self.state.read().await;
        *state == SessionState::Running
    }
}

/// 会话管理任务
///
/// 在专门的任务中运行，处理会话命令
async fn session_task(
    mut command_rx: mpsc::Receiver<SessionCommand>,
    state: Arc<RwLock<SessionState>>,
    state_manager: Arc<StateManager>,
    event_tx: Arc<RwLock<Option<SessionEventSender>>>,
    last_committed: Arc<RwLock<Option<String>>>,
) {
    let mut current_session: Option<TranscriptionSession> = None;

    while let Some(command) = command_rx.recv().await {
        match command {
            SessionCommand::Start { api_key, response } => {
                let result = handle_start(
                    &api_key,
                    &mut current_session,
                    &state,
                    &state_manager,
                    &event_tx,
                    &last_committed,
                )
                .await;
                let _ = response.send(result);
            }
            SessionCommand::Stop { response } => {
                let result = handle_stop(
                    &mut current_session,
                    &state,
                    &state_manager,
                    &last_committed,
                )
                .await;
                let _ = response.send(result);
            }
            SessionCommand::Cancel { response } => {
                let result = handle_cancel_session(
                    &mut current_session,
                    &state,
                    &state_manager,
                    &last_committed,
                )
                .await;
                let _ = response.send(result);
            }
        }
    }
}

/// 处理启动命令
async fn handle_start(
    api_key: &str,
    current_session: &mut Option<TranscriptionSession>,
    state: &Arc<RwLock<SessionState>>,
    state_manager: &Arc<StateManager>,
    event_tx: &Arc<RwLock<Option<SessionEventSender>>>,
    last_committed: &Arc<RwLock<Option<String>>>,
) -> Result<(), SessionControllerError> {
    // 检查当前状态
    {
        let current_state = state.read().await;
        if *current_state != SessionState::Idle {
            return Err(SessionControllerError::SessionAlreadyActive);
        }
    }

    // 更新状态为 Starting
    {
        let mut s = state.write().await;
        *s = SessionState::Starting;
    }

    tracing::info!("Starting transcription session");

    // 创建事件处理回调
    let state_manager_clone = Arc::clone(state_manager);
    let event_tx_clone = Arc::clone(event_tx);
    let last_committed_clone = Arc::clone(last_committed);

    let on_event = move |event: TranscriptEvent| {
        let state_manager = Arc::clone(&state_manager_clone);
        let event_tx = Arc::clone(&event_tx_clone);
        let last_committed = Arc::clone(&last_committed_clone);

        // 使用 spawn_blocking 处理异步操作
        tokio::spawn(async move {
            match &event {
                TranscriptEvent::SessionStarted { session_id } => {
                    tracing::info!(session_id = %session_id, "Transcription session started");

                    // 转换状态到 Recording
                    if let Err(e) = state_manager.transition(AppState::recording_listening()) {
                        tracing::error!(error = %e, "Failed to transition to Recording state");
                    }

                    // 发送事件
                    let tx = event_tx.read().await;
                    if let Some(tx) = tx.as_ref() {
                        let _ = tx.try_send(SessionEvent::Started {
                            session_id: session_id.clone(),
                        });
                    }
                }
                TranscriptEvent::Partial { text } => {
                    tracing::debug!(text = %text, "Partial transcript");

                    // 更新状态中的 partial_text
                    let _ = state_manager.transition(AppState::recording_transcribing(
                        text.clone(),
                        0.5,
                    ));

                    // 发送事件
                    let tx = event_tx.read().await;
                    if let Some(tx) = tx.as_ref() {
                        let _ = tx.try_send(SessionEvent::PartialTranscript { text: text.clone() });
                    }
                }
                TranscriptEvent::Committed { text } => {
                    tracing::info!(text = %text, "Committed transcript");

                    // 保存 committed 文本
                    {
                        let mut last = last_committed.write().await;
                        *last = Some(text.clone());
                    }

                    // 发送事件
                    let tx = event_tx.read().await;
                    if let Some(tx) = tx.as_ref() {
                        let _ = tx.try_send(SessionEvent::CommittedTranscript { text: text.clone() });
                    }
                }
                TranscriptEvent::Error { message } => {
                    tracing::error!(error = %message, "Transcription error");

                    // 转换到错误状态
                    let _ = state_manager.transition(AppState::error(message.clone()));

                    // 发送事件
                    let tx = event_tx.read().await;
                    if let Some(tx) = tx.as_ref() {
                        let _ = tx.try_send(SessionEvent::Error {
                            message: message.clone(),
                        });
                    }
                }
                TranscriptEvent::Closed => {
                    tracing::info!("Transcription session closed");

                    // 发送事件
                    let tx = event_tx.read().await;
                    if let Some(tx) = tx.as_ref() {
                        let _ = tx.try_send(SessionEvent::Closed);
                    }
                }
            }
        });
    };

    // 启动转写会话
    match TranscriptionSession::start(api_key, on_event).await {
        Ok(session) => {
            *current_session = Some(session);

            // 更新状态为 Running
            {
                let mut s = state.write().await;
                *s = SessionState::Running;
            }

            tracing::info!("Transcription session started successfully");
            Ok(())
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to start transcription session");

            // 重置状态
            {
                let mut s = state.write().await;
                *s = SessionState::Idle;
            }

            // 更新应用状态为错误
            let _ = state_manager.transition(AppState::error(e.to_string()));

            Err(SessionControllerError::StartFailed(e.to_string()))
        }
    }
}

/// 处理停止命令
async fn handle_stop(
    current_session: &mut Option<TranscriptionSession>,
    state: &Arc<RwLock<SessionState>>,
    state_manager: &Arc<StateManager>,
    last_committed: &Arc<RwLock<Option<String>>>,
) -> Result<Option<String>, SessionControllerError> {
    // 检查当前状态
    {
        let current_state = state.read().await;
        if *current_state != SessionState::Running {
            return Err(SessionControllerError::NoActiveSession);
        }
    }

    // 更新状态为 Stopping
    {
        let mut s = state.write().await;
        *s = SessionState::Stopping;
    }

    tracing::info!("Stopping transcription session");

    // 停止会话
    if let Some(session) = current_session {
        if let Err(e) = session.stop().await {
            tracing::warn!(error = %e, "Error while stopping session");
        }
    }
    *current_session = None;

    // 获取最后的 committed 文本
    let committed_text = last_committed.write().await.take();

    // 更新状态为 Idle
    {
        let mut s = state.write().await;
        *s = SessionState::Idle;
    }

    // 重置应用状态
    state_manager.reset();

    tracing::info!("Transcription session stopped");
    Ok(committed_text)
}

/// 处理取消命令
async fn handle_cancel_session(
    current_session: &mut Option<TranscriptionSession>,
    state: &Arc<RwLock<SessionState>>,
    state_manager: &Arc<StateManager>,
    last_committed: &Arc<RwLock<Option<String>>>,
) -> Result<(), SessionControllerError> {
    // 检查当前状态
    let current_state = state.read().await.clone();
    if current_state == SessionState::Idle {
        return Ok(());
    }

    tracing::info!("Cancelling transcription session");

    // 更新状态为 Stopping
    {
        let mut s = state.write().await;
        *s = SessionState::Stopping;
    }

    // 停止会话
    if let Some(session) = current_session {
        let _ = session.stop().await;
    }
    *current_session = None;

    // 清除 last_committed_text
    {
        let mut text = last_committed.write().await;
        *text = None;
    }

    // 更新状态为 Idle
    {
        let mut s = state.write().await;
        *s = SessionState::Idle;
    }

    // 重置应用状态
    state_manager.reset();

    tracing::info!("Transcription session cancelled");
    Ok(())
}

/// 会话控制器错误
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum SessionControllerError {
    /// API Key 未设置
    #[error("API key not set")]
    ApiKeyNotSet,

    /// 会话已经在运行
    #[error("A session is already active")]
    SessionAlreadyActive,

    /// 没有活跃的会话
    #[error("No active session")]
    NoActiveSession,

    /// 启动会话失败
    #[error("Failed to start session: {0}")]
    StartFailed(String),

    /// 停止会话失败
    #[error("Failed to stop session: {0}")]
    StopFailed(String),

    /// Channel 已关闭
    #[error("Session controller channel closed")]
    ChannelClosed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_equality() {
        assert_eq!(SessionState::Idle, SessionState::Idle);
        assert_ne!(SessionState::Idle, SessionState::Running);
    }

    #[test]
    fn test_session_event_creation() {
        let events = vec![
            SessionEvent::Started {
                session_id: "test-123".to_string(),
            },
            SessionEvent::PartialTranscript {
                text: "hello".to_string(),
            },
            SessionEvent::CommittedTranscript {
                text: "hello world".to_string(),
            },
            SessionEvent::Error {
                message: "test error".to_string(),
            },
            SessionEvent::Closed,
        ];

        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_session_controller_error_display() {
        let error = SessionControllerError::ApiKeyNotSet;
        assert!(format!("{}", error).contains("API key not set"));

        let error = SessionControllerError::SessionAlreadyActive;
        assert!(format!("{}", error).contains("already active"));

        let error = SessionControllerError::StartFailed("connection error".to_string());
        assert!(format!("{}", error).contains("connection error"));

        let error = SessionControllerError::ChannelClosed;
        assert!(format!("{}", error).contains("channel closed"));
    }
}
