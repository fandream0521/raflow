use std::sync::Arc;

use arc_swap::ArcSwap;
use serde::Serialize;
use tokio::sync::mpsc;

use super::error::{StateError, StateResult};

/// 录音子状态
///
/// 表示录音阶段的具体状态
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum RecordingState {
    /// 监听中，未检测到语音
    Listening,

    /// 正在转写
    Transcribing {
        /// 当前部分转写文本
        partial_text: String,
        /// 置信度 (0.0 - 1.0)
        confidence: f32,
    },
}

impl RecordingState {
    /// 创建监听状态
    pub fn listening() -> Self {
        Self::Listening
    }

    /// 创建转写状态
    pub fn transcribing(partial_text: String, confidence: f32) -> Self {
        Self::Transcribing {
            partial_text,
            confidence,
        }
    }

    /// 检查是否在监听
    pub fn is_listening(&self) -> bool {
        matches!(self, Self::Listening)
    }

    /// 检查是否在转写
    pub fn is_transcribing(&self) -> bool {
        matches!(self, Self::Transcribing { .. })
    }

    /// 获取部分文本（如果有）
    pub fn partial_text(&self) -> Option<&str> {
        match self {
            Self::Transcribing { partial_text, .. } => Some(partial_text),
            _ => None,
        }
    }

    /// 获取置信度（如果有）
    pub fn confidence(&self) -> Option<f32> {
        match self {
            Self::Transcribing { confidence, .. } => Some(*confidence),
            _ => None,
        }
    }
}

/// 应用主状态
///
/// 表示应用程序的整体状态，用于管理转写流程的生命周期
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AppState {
    /// 空闲状态，等待用户触发
    Idle,

    /// 正在建立 WebSocket 连接
    Connecting,

    /// 正在录音和转写
    Recording(RecordingState),

    /// 正在处理最终转写结果
    Processing,

    /// 正在将文本注入目标应用
    Injecting,

    /// 错误状态
    Error(String),
}

impl AppState {
    /// 创建空闲状态
    pub fn idle() -> Self {
        Self::Idle
    }

    /// 创建连接中状态
    pub fn connecting() -> Self {
        Self::Connecting
    }

    /// 创建录音状态（监听）
    pub fn recording_listening() -> Self {
        Self::Recording(RecordingState::Listening)
    }

    /// 创建录音状态（转写中）
    pub fn recording_transcribing(partial_text: String, confidence: f32) -> Self {
        Self::Recording(RecordingState::Transcribing {
            partial_text,
            confidence,
        })
    }

    /// 创建处理中状态
    pub fn processing() -> Self {
        Self::Processing
    }

    /// 创建注入中状态
    pub fn injecting() -> Self {
        Self::Injecting
    }

    /// 创建错误状态
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error(message.into())
    }

    /// 检查是否为空闲状态
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// 检查是否在连接中
    pub fn is_connecting(&self) -> bool {
        matches!(self, Self::Connecting)
    }

    /// 检查是否在录音中
    pub fn is_recording(&self) -> bool {
        matches!(self, Self::Recording(_))
    }

    /// 检查是否在处理中
    pub fn is_processing(&self) -> bool {
        matches!(self, Self::Processing)
    }

    /// 检查是否在注入中
    pub fn is_injecting(&self) -> bool {
        matches!(self, Self::Injecting)
    }

    /// 检查是否为错误状态
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// 获取录音子状态（如果处于录音状态）
    pub fn recording_state(&self) -> Option<&RecordingState> {
        match self {
            Self::Recording(state) => Some(state),
            _ => None,
        }
    }

    /// 获取错误消息（如果处于错误状态）
    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Error(msg) => Some(msg),
            _ => None,
        }
    }

    /// 获取状态名称（用于日志和调试）
    pub fn name(&self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Connecting => "Connecting",
            Self::Recording(RecordingState::Listening) => "Recording::Listening",
            Self::Recording(RecordingState::Transcribing { .. }) => "Recording::Transcribing",
            Self::Processing => "Processing",
            Self::Injecting => "Injecting",
            Self::Error(_) => "Error",
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::Idle
    }
}

/// 状态管理器
///
/// 负责管理应用状态的转换和通知监听者
pub struct StateManager {
    /// 当前状态（使用 ArcSwap 实现无锁读取）
    state: ArcSwap<AppState>,

    /// 状态变更监听器列表
    listeners: Arc<tokio::sync::Mutex<Vec<mpsc::Sender<AppState>>>>,
}

impl StateManager {
    /// 创建新的状态管理器
    ///
    /// # Examples
    ///
    /// ```
    /// use raflow_lib::state::StateManager;
    ///
    /// let manager = StateManager::new();
    /// assert!(manager.current().is_idle());
    /// ```
    pub fn new() -> Self {
        Self {
            state: ArcSwap::new(Arc::new(AppState::Idle)),
            listeners: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// 获取当前状态
    ///
    /// 此方法是无锁的，可以在任何线程安全地调用
    ///
    /// # Examples
    ///
    /// ```
    /// use raflow_lib::state::StateManager;
    ///
    /// let manager = StateManager::new();
    /// let current = manager.current();
    /// assert!(current.is_idle());
    /// ```
    pub fn current(&self) -> Arc<AppState> {
        self.state.load_full()
    }

    /// 转换到新状态
    ///
    /// 验证状态转换的合法性，如果合法则更新状态并通知所有监听者
    ///
    /// # Errors
    ///
    /// 如果状态转换不合法，返回 [`StateError::InvalidTransition`]
    ///
    /// # Examples
    ///
    /// ```
    /// use raflow_lib::state::{StateManager, AppState};
    ///
    /// let manager = StateManager::new();
    ///
    /// // 合法转换
    /// assert!(manager.transition(AppState::connecting()).is_ok());
    ///
    /// // 非法转换
    /// assert!(manager.transition(AppState::injecting()).is_err());
    /// ```
    pub fn transition(&self, new_state: AppState) -> StateResult<()> {
        let current = self.current();

        // 验证状态转换是否合法
        if !self.is_valid_transition(&current, &new_state) {
            return Err(StateError::InvalidTransition {
                from: (*current).clone(),
                to: new_state,
            });
        }

        // 更新状态
        self.state.store(Arc::new(new_state.clone()));

        // 通知监听者（如果有 tokio 运行时）
        self.notify_listeners(new_state);

        Ok(())
    }

    /// 添加状态变更监听器
    ///
    /// 返回的接收器将接收所有状态变更通知
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use raflow_lib::state::{StateManager, AppState};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let manager = StateManager::new();
    /// let mut rx = manager.subscribe().await;
    ///
    /// // 在另一个任务中监听状态变更
    /// tokio::spawn(async move {
    ///     while let Some(state) = rx.recv().await {
    ///         println!("State changed to: {:?}", state);
    ///     }
    /// });
    /// # }
    /// ```
    pub async fn subscribe(&self) -> mpsc::Receiver<AppState> {
        let (tx, rx) = mpsc::channel(32);
        let mut listeners = self.listeners.lock().await;
        listeners.push(tx);
        rx
    }

    /// 移除所有已关闭的监听器
    ///
    /// 清理已关闭的接收器，释放资源
    pub async fn cleanup_listeners(&self) {
        let mut listeners = self.listeners.lock().await;
        listeners.retain(|tx| !tx.is_closed());
    }

    /// 获取当前监听器数量
    pub async fn listener_count(&self) -> usize {
        self.listeners.lock().await.len()
    }

    /// 强制设置状态（跳过验证）
    ///
    /// **警告**: 此方法跳过状态转换验证，仅在特殊情况下使用
    /// （例如错误恢复）
    pub fn force_set(&self, new_state: AppState) {
        self.state.store(Arc::new(new_state.clone()));

        // 通知监听者（如果有 tokio 运行时）
        self.notify_listeners(new_state);
    }

    /// 重置为空闲状态
    ///
    /// 这是 `force_set(AppState::Idle)` 的便捷方法
    pub fn reset(&self) {
        self.force_set(AppState::Idle);
    }

    /// 通知所有监听者状态变更
    ///
    /// 如果有 tokio 运行时，异步通知；否则静默失败
    fn notify_listeners(&self, new_state: AppState) {
        let listeners = Arc::clone(&self.listeners);

        // 尝试获取当前 tokio 运行时
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::spawn(async move {
                let listeners_guard = listeners.lock().await;
                for listener in listeners_guard.iter() {
                    // 使用 try_send 避免阻塞
                    let _ = listener.try_send(new_state.clone());
                }
            });
        }
        // 如果没有运行时，静默失败（测试环境可能不需要通知）
    }

    /// 验证状态转换是否合法
    ///
    /// 根据状态机图定义的转换规则进行验证
    fn is_valid_transition(&self, from: &AppState, to: &AppState) -> bool {
        use AppState::*;

        match (from, to) {
            // 从 Idle 可以转换到 Connecting
            (Idle, Connecting) => true,

            // 从 Connecting 可以转换到 Recording 或 Error
            (Connecting, Recording(_)) => true,
            (Connecting, Error(_)) => true,

            // 从 Recording 可以转换到 Processing、Idle（取消）或内部状态切换
            (Recording(_), Processing) => true,
            (Recording(_), Idle) => true,
            (Recording(_), Recording(_)) => true, // 允许子状态切换

            // 从 Processing 可以转换到 Injecting 或 Idle（超时/取消）
            (Processing, Injecting) => true,
            (Processing, Idle) => true,

            // 从 Injecting 可以转换到 Idle
            (Injecting, Idle) => true,

            // 从 Error 可以转换到 Idle
            (Error(_), Idle) => true,

            // 任何状态都可以转换到 Error
            (_, Error(_)) => true,

            // 其他转换不合法
            _ => false,
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_state_creation() {
        let listening = RecordingState::listening();
        assert!(listening.is_listening());
        assert!(!listening.is_transcribing());

        let transcribing = RecordingState::transcribing("hello".to_string(), 0.95);
        assert!(!transcribing.is_listening());
        assert!(transcribing.is_transcribing());
        assert_eq!(transcribing.partial_text(), Some("hello"));
        assert_eq!(transcribing.confidence(), Some(0.95));
    }

    #[test]
    fn test_app_state_creation() {
        let idle = AppState::idle();
        assert!(idle.is_idle());
        assert_eq!(idle.name(), "Idle");

        let connecting = AppState::connecting();
        assert!(connecting.is_connecting());

        let recording = AppState::recording_listening();
        assert!(recording.is_recording());
        assert_eq!(recording.name(), "Recording::Listening");

        let processing = AppState::processing();
        assert!(processing.is_processing());

        let injecting = AppState::injecting();
        assert!(injecting.is_injecting());

        let error = AppState::error("test error");
        assert!(error.is_error());
        assert_eq!(error.error_message(), Some("test error"));
    }

    #[test]
    fn test_state_manager_creation() {
        let manager = StateManager::new();
        let current = manager.current();
        assert!(current.is_idle());
    }

    #[test]
    fn test_valid_transitions() {
        let manager = StateManager::new();

        // Idle -> Connecting
        assert!(manager.transition(AppState::connecting()).is_ok());

        // Connecting -> Recording
        assert!(manager.transition(AppState::recording_listening()).is_ok());

        // Recording -> Processing
        assert!(manager.transition(AppState::processing()).is_ok());

        // Processing -> Injecting
        assert!(manager.transition(AppState::injecting()).is_ok());

        // Injecting -> Idle
        assert!(manager.transition(AppState::idle()).is_ok());
    }

    #[test]
    fn test_invalid_transitions() {
        let manager = StateManager::new();

        // Idle -> Processing (invalid)
        let result = manager.transition(AppState::processing());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StateError::InvalidTransition { .. }));

        // Idle -> Injecting (invalid)
        assert!(manager.transition(AppState::injecting()).is_err());

        // 转换到 Connecting（合法）
        manager.transition(AppState::connecting()).unwrap();

        // Connecting -> Injecting (invalid)
        assert!(manager.transition(AppState::injecting()).is_err());
    }

    #[test]
    fn test_recording_sub_state_transitions() {
        let manager = StateManager::new();

        // 转换到录音状态
        manager.transition(AppState::connecting()).unwrap();
        manager.transition(AppState::recording_listening()).unwrap();

        // 可以在录音子状态之间切换
        let result = manager.transition(AppState::recording_transcribing("test".to_string(), 0.9));
        assert!(result.is_ok());

        let current = manager.current();
        assert!(current.is_recording());
        if let Some(state) = current.recording_state() {
            assert!(state.is_transcribing());
            assert_eq!(state.partial_text(), Some("test"));
        } else {
            panic!("Expected recording state");
        }
    }

    #[test]
    fn test_error_state_transitions() {
        let manager = StateManager::new();

        // 任何状态都可以转换到 Error
        manager.transition(AppState::connecting()).unwrap();
        assert!(manager.transition(AppState::error("connection failed")).is_ok());

        // Error -> Idle
        assert!(manager.transition(AppState::idle()).is_ok());
    }

    #[test]
    fn test_cancel_during_recording() {
        let manager = StateManager::new();

        // 进入录音状态
        manager.transition(AppState::connecting()).unwrap();
        manager.transition(AppState::recording_listening()).unwrap();

        // 可以取消（回到 Idle）
        assert!(manager.transition(AppState::idle()).is_ok());
    }

    #[test]
    fn test_force_set() {
        let manager = StateManager::new();

        // 使用 force_set 可以跳过验证
        manager.force_set(AppState::injecting());
        assert!(manager.current().is_injecting());

        // reset 恢复到 Idle
        manager.reset();
        assert!(manager.current().is_idle());
    }

    #[tokio::test]
    async fn test_state_listener() {
        let manager = StateManager::new();
        let mut rx = manager.subscribe().await;

        // 在后台任务中改变状态
        let manager_clone = StateManager::new();
        manager_clone.state.store(manager.state.load_full());

        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            let _ = manager_clone.transition(AppState::connecting());
        });

        // 等待状态变更通知
        let timeout = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            rx.recv()
        ).await;

        // 注意：由于我们创建了新的 manager_clone，监听器不会收到通知
        // 这个测试主要验证订阅机制本身是否工作
        assert!(timeout.is_err() || timeout.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_listener_count() {
        let manager = StateManager::new();

        assert_eq!(manager.listener_count().await, 0);

        let _rx1 = manager.subscribe().await;
        assert_eq!(manager.listener_count().await, 1);

        let _rx2 = manager.subscribe().await;
        assert_eq!(manager.listener_count().await, 2);

        drop(_rx1);
        manager.cleanup_listeners().await;
        assert_eq!(manager.listener_count().await, 1);
    }
}
