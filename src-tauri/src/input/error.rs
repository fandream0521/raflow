//! 输入模块错误类型
//!
//! 定义输入注入相关的错误类型

use thiserror::Error;

/// 输入操作错误
#[derive(Error, Debug, Clone, PartialEq)]
pub enum InputError {
    /// 辅助功能权限被拒绝
    #[error("Accessibility permission denied")]
    PermissionDenied,

    /// 没有焦点窗口
    #[error("No focused window found")]
    NoFocusedWindow,

    /// 文本注入失败
    #[error("Failed to inject text: {0}")]
    InjectionFailed(String),

    /// 剪贴板操作失败
    #[error("Clipboard operation failed: {0}")]
    ClipboardFailed(String),

    /// 窗口检测失败
    #[error("Window detection failed: {0}")]
    WindowDetectionFailed(String),

    /// 键盘模拟失败
    #[error("Keyboard simulation failed: {0}")]
    KeyboardSimulationFailed(String),

    /// 平台不支持
    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),
}

/// 输入操作结果类型
pub type InputResult<T> = Result<T, InputError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_error_display() {
        let error = InputError::NoFocusedWindow;
        assert!(error.to_string().contains("No focused window"));

        let error = InputError::PermissionDenied;
        assert!(error.to_string().contains("permission denied"));

        let error = InputError::InjectionFailed("test reason".to_string());
        assert!(error.to_string().contains("test reason"));

        let error = InputError::ClipboardFailed("clipboard error".to_string());
        assert!(error.to_string().contains("clipboard error"));
    }

    #[test]
    fn test_input_error_equality() {
        let error1 = InputError::NoFocusedWindow;
        let error2 = InputError::NoFocusedWindow;
        assert_eq!(error1, error2);

        let error3 = InputError::PermissionDenied;
        assert_ne!(error1, error3);

        let error4 = InputError::InjectionFailed("test".to_string());
        let error5 = InputError::InjectionFailed("test".to_string());
        assert_eq!(error4, error5);

        let error6 = InputError::InjectionFailed("other".to_string());
        assert_ne!(error4, error6);
    }

    #[test]
    fn test_input_error_clone() {
        let error = InputError::WindowDetectionFailed("detection error".to_string());
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }
}
