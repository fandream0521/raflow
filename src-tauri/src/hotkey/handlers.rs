//! 热键处理器模块
//!
//! 实现 PTT、取消等热键的具体处理逻辑

use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};

use super::session::{SessionController, SessionControllerError};
use crate::state::{
    setup_state_transitions, AppState, ProcessingTimeoutHandler, StateEventEmitter, StateManager,
};

/// API Key 持有者
///
/// 存储 API Key 供会话使用
pub struct ApiKeyHolder {
    api_key: tokio::sync::RwLock<Option<String>>,
}

impl ApiKeyHolder {
    /// 创建新的 API Key 持有者
    pub fn new() -> Self {
        Self {
            api_key: tokio::sync::RwLock::new(None),
        }
    }

    /// 设置 API Key
    pub async fn set(&self, api_key: impl Into<String>) {
        let mut key = self.api_key.write().await;
        *key = Some(api_key.into());
    }

    /// 获取 API Key
    pub async fn get(&self) -> Option<String> {
        self.api_key.read().await.clone()
    }
}

impl Default for ApiKeyHolder {
    fn default() -> Self {
        Self::new()
    }
}

/// 处理 Push-to-Talk 按下事件
///
/// 当用户按下 PTT 热键时：
/// 1. 检查当前状态是否为 Idle
/// 2. 转换状态为 Connecting
/// 3. 启动转写会话
pub fn handle_ptt_pressed(app: &AppHandle) {
    // 获取状态管理器
    let state_manager = match app.try_state::<Arc<StateManager>>() {
        Some(manager) => manager,
        None => {
            tracing::warn!("StateManager not available, ignoring PTT pressed event");
            return;
        }
    };

    let current = state_manager.current();

    // 只在 Idle 状态时响应
    if !current.is_idle() {
        tracing::warn!(
            current_state = %current.name(),
            "PTT pressed but not in Idle state, ignoring"
        );
        return;
    }

    // 转换到 Connecting 状态
    if let Err(e) = state_manager.transition(AppState::connecting()) {
        tracing::error!(error = %e, "Failed to transition to Connecting state");
        return;
    }

    tracing::info!("PTT pressed: transitioning to Connecting state");

    // 获取会话控制器
    let session_controller = match app.try_state::<Arc<SessionController>>() {
        Some(controller) => controller,
        None => {
            tracing::error!("SessionController not available");
            state_manager.reset();
            return;
        }
    };

    // 获取 API Key
    let api_key_holder = match app.try_state::<Arc<ApiKeyHolder>>() {
        Some(holder) => holder,
        None => {
            tracing::error!("ApiKeyHolder not available");
            let _ = state_manager.transition(AppState::error("API Key 未配置".to_string()));
            return;
        }
    };

    // 在后台启动会话
    let app_handle = app.clone();
    let controller = Arc::clone(&session_controller);
    let state_mgr = Arc::clone(&state_manager);
    let api_holder = Arc::clone(&api_key_holder);

    tokio::spawn(async move {
        // 获取 API Key
        let api_key = match api_holder.get().await {
            Some(key) => key,
            None => {
                tracing::error!("API Key not set");
                let _ = state_mgr.transition(AppState::error("API Key 未设置，请在设置中配置".to_string()));
                let _ = app_handle.emit("transcription:error", "API Key 未设置");
                return;
            }
        };

        match controller.start_session(&api_key).await {
            Ok(()) => {
                tracing::info!("Transcription session started successfully");
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to start transcription session");

                // 转换到错误状态
                let error_msg = match &e {
                    SessionControllerError::ApiKeyNotSet => {
                        "API Key 未设置，请在设置中配置".to_string()
                    }
                    SessionControllerError::SessionAlreadyActive => {
                        "会话已在运行中".to_string()
                    }
                    SessionControllerError::StartFailed(msg) => {
                        format!("启动失败: {}", msg)
                    }
                    _ => e.to_string(),
                };

                let _ = state_mgr.transition(AppState::error(error_msg));

                // 发送错误通知到前端
                let _ = app_handle.emit("transcription:error", e.to_string());
            }
        }
    });
}

/// 处理 Push-to-Talk 松开事件
///
/// 当用户松开 PTT 热键时：
/// 1. 检查当前状态是否为 Recording
/// 2. 转换状态为 Processing
/// 3. 停止会话并获取最终结果
/// 4. 转换状态为 Injecting（如果有结果）
pub fn handle_ptt_released(app: &AppHandle) {
    // 获取状态管理器
    let state_manager = match app.try_state::<Arc<StateManager>>() {
        Some(manager) => manager,
        None => {
            tracing::warn!("StateManager not available, ignoring PTT released event");
            return;
        }
    };

    let current = state_manager.current();

    // 只在 Recording 状态时响应
    if !current.is_recording() {
        tracing::debug!(
            current_state = %current.name(),
            "PTT released but not in Recording state, ignoring"
        );
        return;
    }

    // 转换到 Processing 状态
    if let Err(e) = state_manager.transition(AppState::processing()) {
        tracing::error!(error = %e, "Failed to transition to Processing state");
        return;
    }

    tracing::info!("PTT released: transitioning to Processing state");

    // 获取会话控制器
    let session_controller = match app.try_state::<Arc<SessionController>>() {
        Some(controller) => controller,
        None => {
            tracing::error!("SessionController not available");
            state_manager.reset();
            return;
        }
    };

    // 在后台停止会话并处理结果
    let app_handle = app.clone();
    let controller = Arc::clone(&session_controller);
    let state_mgr = Arc::clone(&state_manager);

    tokio::spawn(async move {
        match controller.stop_session().await {
            Ok(Some(text)) => {
                tracing::info!(text = %text, "Got committed transcript");

                // 转换到 Injecting 状态
                if let Err(e) = state_mgr.transition(AppState::injecting()) {
                    tracing::error!(error = %e, "Failed to transition to Injecting state");
                    state_mgr.reset();
                    return;
                }

                // 发送结果到前端
                let _ = app_handle.emit("transcription:committed", &text);

                // TODO: P2-T7 中实现文本注入
                // 这里先只是通知前端，实际注入在 P2-T7 中实现

                // 注入完成后返回 Idle
                // 暂时直接重置，等 P2-T7 实现后会在注入完成后重置
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                state_mgr.reset();
            }
            Ok(None) => {
                tracing::info!("No committed transcript received");
                state_mgr.reset();
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to stop session");
                state_mgr.reset();
            }
        }
    });
}

/// 处理取消事件
///
/// 当用户按下取消键时：
/// 1. 检查是否在可取消状态（Connecting 或 Recording）
/// 2. 取消当前会话
/// 3. 重置状态为 Idle
pub fn handle_cancel(app: &AppHandle) {
    // 获取状态管理器
    let state_manager = match app.try_state::<Arc<StateManager>>() {
        Some(manager) => manager,
        None => {
            tracing::warn!("StateManager not available, ignoring Cancel event");
            return;
        }
    };

    let current = state_manager.current();

    // 只在 Recording、Connecting 或 Processing 状态时响应
    if !current.is_recording() && !current.is_connecting() && !current.is_processing() {
        tracing::debug!(
            current_state = %current.name(),
            "Cancel pressed but not in cancellable state, ignoring"
        );
        return;
    }

    tracing::info!("Cancel pressed: cancelling session");

    // 获取会话控制器
    let session_controller = match app.try_state::<Arc<SessionController>>() {
        Some(controller) => controller,
        None => {
            // 没有会话控制器，直接重置状态
            state_manager.reset();
            return;
        }
    };

    // 在后台取消会话
    let controller = Arc::clone(&session_controller);
    let app_handle = app.clone();

    tokio::spawn(async move {
        if let Err(e) = controller.cancel_session().await {
            tracing::error!(error = %e, "Failed to cancel session");
        }

        // 发送取消通知到前端
        let _ = app_handle.emit("transcription:cancelled", ());
    });
}

/// 处理切换模式事件
///
/// 用于切换应用程序模式（如静音模式）
/// 这个功能在 MVP 中可能不需要，预留接口
pub fn handle_toggle_mode(app: &AppHandle) {
    tracing::info!("Toggle mode pressed");

    // 发送模式切换事件到前端
    let _ = app.emit("app:toggle_mode", ());

    // TODO: 实现模式切换逻辑
}

/// 状态转换系统持有者
///
/// 存储 StateEventEmitter 和 ProcessingTimeoutHandler
pub struct StateTransitionSystem {
    event_emitter: tokio::sync::Mutex<Option<StateEventEmitter>>,
    timeout_handler: tokio::sync::Mutex<Option<ProcessingTimeoutHandler>>,
}

impl StateTransitionSystem {
    /// 创建新实例
    fn new() -> Self {
        Self {
            event_emitter: tokio::sync::Mutex::new(None),
            timeout_handler: tokio::sync::Mutex::new(None),
        }
    }

    /// 初始化状态转换系统
    async fn initialize(&self, app: &AppHandle, state_manager: Arc<StateManager>) {
        let (emitter, handler) = setup_state_transitions(app, state_manager, None).await;

        *self.event_emitter.lock().await = Some(emitter);
        *self.timeout_handler.lock().await = Some(handler);
    }

    /// 停止状态转换系统
    pub async fn stop(&self) {
        if let Some(mut emitter) = self.event_emitter.lock().await.take() {
            emitter.stop().await;
        }
        if let Some(mut handler) = self.timeout_handler.lock().await.take() {
            handler.stop().await;
        }
    }
}

/// 初始化热键处理所需的状态
///
/// 在应用启动时调用，注册必要的状态管理器
pub fn setup_hotkey_state(app: &AppHandle) -> Result<(), HotkeyHandlerError> {
    // 创建状态管理器
    let state_manager = Arc::new(StateManager::new());
    app.manage(Arc::clone(&state_manager));

    // 创建会话控制器
    let session_controller = Arc::new(SessionController::new(Arc::clone(&state_manager)));
    app.manage(session_controller);

    // 创建 API Key 持有者
    let api_key_holder = Arc::new(ApiKeyHolder::new());
    app.manage(api_key_holder);

    // 创建状态转换系统（稍后异步初始化）
    let transition_system = Arc::new(StateTransitionSystem::new());
    app.manage(Arc::clone(&transition_system));

    // 在后台初始化状态转换系统
    let app_handle = app.clone();
    let state_mgr = Arc::clone(&state_manager);
    let trans_sys = Arc::clone(&transition_system);

    tokio::spawn(async move {
        trans_sys.initialize(&app_handle, state_mgr).await;
        tracing::info!("State transition system initialized");
    });

    tracing::info!("Hotkey state initialized");
    Ok(())
}

/// 设置 API Key
///
/// 在应用配置加载后调用
pub async fn set_api_key(app: &AppHandle, api_key: &str) -> Result<(), HotkeyHandlerError> {
    let api_key_holder = app
        .try_state::<Arc<ApiKeyHolder>>()
        .ok_or(HotkeyHandlerError::ControllerNotAvailable)?;

    api_key_holder.set(api_key).await;
    tracing::info!("API key configured");
    Ok(())
}

/// 热键处理器错误
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum HotkeyHandlerError {
    /// 状态管理器不可用
    #[error("State manager not available")]
    StateManagerNotAvailable,

    /// 会话控制器不可用
    #[error("Session controller not available")]
    ControllerNotAvailable,

    /// 状态转换失败
    #[error("State transition failed: {0}")]
    TransitionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_handler_error_display() {
        let error = HotkeyHandlerError::StateManagerNotAvailable;
        assert!(format!("{}", error).contains("State manager not available"));

        let error = HotkeyHandlerError::ControllerNotAvailable;
        assert!(format!("{}", error).contains("Session controller not available"));

        let error = HotkeyHandlerError::TransitionFailed("invalid transition".to_string());
        assert!(format!("{}", error).contains("invalid transition"));
    }

    #[test]
    fn test_hotkey_handler_error_equality() {
        let error1 = HotkeyHandlerError::StateManagerNotAvailable;
        let error2 = HotkeyHandlerError::StateManagerNotAvailable;
        assert_eq!(error1, error2);

        let error3 = HotkeyHandlerError::ControllerNotAvailable;
        assert_ne!(error1, error3);
    }

    #[tokio::test]
    async fn test_api_key_holder() {
        let holder = ApiKeyHolder::new();

        // 初始为空
        assert!(holder.get().await.is_none());

        // 设置后可获取
        holder.set("test-api-key").await;
        assert_eq!(holder.get().await, Some("test-api-key".to_string()));

        // 可以更新
        holder.set("new-api-key").await;
        assert_eq!(holder.get().await, Some("new-api-key".to_string()));
    }
}
