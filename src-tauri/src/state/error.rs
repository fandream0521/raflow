use thiserror::Error;

use super::app_state::AppState;

/// 状态相关错误
#[derive(Error, Debug, Clone, PartialEq)]
pub enum StateError {
    /// 无效的状态转换
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition { from: AppState, to: AppState },

    /// 监听器已满
    #[error("Listener queue is full")]
    ListenerQueueFull,

    /// 监听器未找到
    #[error("Listener with id {0} not found")]
    ListenerNotFound(String),
}

/// 状态模块的结果类型
pub type StateResult<T> = Result<T, StateError>;
