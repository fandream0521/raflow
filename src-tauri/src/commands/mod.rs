//! Tauri 命令模块
//!
//! 提供前端可调用的 Tauri 命令
//!
//! # 模块结构
//!
//! - `config` - 配置管理命令
//! - `state` - 状态管理命令
//! - `window` - 窗口管理命令

pub mod config;
pub mod state;
pub mod window;

pub use config::*;
pub use state::*;
pub use window::*;
