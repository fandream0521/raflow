//! 热键管理模块
//!
//! 提供全局热键的注册、管理和事件处理功能
//!
//! # 功能
//!
//! - Push-to-Talk 热键：按住开始录音，松开结束录音
//! - 取消热键：取消当前录音会话
//! - 切换模式热键：切换应用程序模式（可选）
//!
//! # 使用方法
//!
//! ```ignore
//! use raflow_lib::hotkey::{HotkeyConfig, register_hotkeys, setup_hotkey_state};
//!
//! // 在 Tauri setup 中初始化状态和注册热键
//! tauri::Builder::default()
//!     .setup(|app| {
//!         // 初始化热键所需状态
//!         setup_hotkey_state(app.handle())?;
//!
//!         // 创建配置并注册热键
//!         let config = HotkeyConfig::default();
//!         register_hotkeys(app.handle(), &config)?;
//!         Ok(())
//!     })
//! ```

mod config;
mod error;
mod handlers;
mod register;
mod session;

pub use config::HotkeyConfig;
pub use error::{HotkeyError, HotkeyResult};
pub use handlers::{
    handle_cancel, handle_ptt_pressed, handle_ptt_released, handle_toggle_mode, set_api_key,
    setup_hotkey_state, HotkeyHandlerError, StateTransitionSystem,
};
pub use register::{
    is_hotkey_registered, register_hotkeys, unregister_hotkeys, HotkeyEvent, HotkeyHandler,
    HotkeyManager,
};
pub use session::{
    SessionController, SessionControllerError, SessionEvent, SessionEventSender, SessionState,
};
