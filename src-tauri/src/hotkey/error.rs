//! 热键相关错误类型

use thiserror::Error;

/// 热键相关错误
#[derive(Error, Debug, Clone, PartialEq)]
pub enum HotkeyError {
    /// 无效的热键格式
    #[error("Invalid hotkey format: {0}")]
    InvalidFormat(String),

    /// 热键注册失败
    #[error("Failed to register hotkey '{hotkey}': {reason}")]
    RegistrationFailed { hotkey: String, reason: String },

    /// 热键注销失败
    #[error("Failed to unregister hotkey '{hotkey}': {reason}")]
    UnregistrationFailed { hotkey: String, reason: String },

    /// 热键已被注册
    #[error("Hotkey '{0}' is already registered")]
    AlreadyRegistered(String),

    /// 热键未注册
    #[error("Hotkey '{0}' is not registered")]
    NotRegistered(String),

    /// 热键被系统占用
    #[error("Hotkey '{0}' is occupied by system or another application")]
    Occupied(String),

    /// 全局快捷键插件不可用
    #[error("Global shortcut plugin is not available")]
    PluginNotAvailable,

    /// 配置错误
    #[error("Hotkey configuration error: {0}")]
    ConfigError(String),
}

/// 热键模块的结果类型
pub type HotkeyResult<T> = Result<T, HotkeyError>;
