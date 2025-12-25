//! 状态管理模块
//!
//! 提供应用程序状态机和状态管理功能
//!
//! # 模块结构
//!
//! - `app_state` - 核心状态定义和状态管理器
//! - `error` - 状态相关错误类型
//! - `transitions` - 状态转换逻辑和事件发射

mod app_state;
mod error;
mod transitions;

pub use app_state::{AppState, RecordingState, StateManager};
pub use error::{StateError, StateResult};
pub use transitions::{
    setup_state_transitions, ProcessingTimeoutHandler, StateChangeEvent, StateEventEmitter,
    StateTransitionContext, TransitionError, DEFAULT_PROCESSING_TIMEOUT_SECS,
};
