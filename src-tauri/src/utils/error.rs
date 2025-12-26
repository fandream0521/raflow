//! 全局错误处理模块
//!
//! 提供统一的应用错误类型和用户友好的错误消息
//!
//! # 功能
//!
//! - 统一的 `AppError` 类型，聚合所有模块错误
//! - 用户友好的错误消息（支持多语言）
//! - 错误代码用于前端处理
//! - 错误恢复建议
//!
//! # 使用示例
//!
//! ```
//! use raflow_lib::utils::error::{AppError, ErrorContext};
//!
//! fn example() -> Result<(), AppError> {
//!     // 从其他错误类型自动转换
//!     // let result = some_audio_operation()?;
//!     Ok(())
//! }
//! ```

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::audio::error::AudioError;
use crate::input::error::InputError;
use crate::network::error::NetworkError;
use crate::state::config::ConfigError;
use crate::transcription::TranscriptionError;
use crate::session::SessionError;

/// 应用错误类型
///
/// 聚合所有模块的错误类型，提供统一的错误处理接口
#[derive(Error, Debug)]
pub enum AppError {
    /// 音频错误
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    /// 网络错误
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    /// 输入错误
    #[error("Input error: {0}")]
    Input(#[from] InputError),

    /// 配置错误
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    /// 转写错误
    #[error("Transcription error: {0}")]
    Transcription(#[from] TranscriptionError),

    /// 会话错误
    #[error("Session error: {0}")]
    Session(#[from] SessionError),

    /// 内部错误
    #[error("Internal error: {0}")]
    Internal(String),

    /// 用户取消操作
    #[error("Operation cancelled")]
    Cancelled,

    /// 超时错误
    #[error("Operation timed out after {0}ms")]
    Timeout(u64),
}

/// 错误代码
///
/// 用于前端识别和处理特定错误
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // 音频错误 (1xxx)
    /// 找不到麦克风设备
    AudioDeviceNotFound,
    /// 音频流错误
    AudioStreamError,
    /// 重采样失败
    AudioResampleFailed,

    // 网络错误 (2xxx)
    /// 连接失败
    NetworkConnectionFailed,
    /// 认证失败（API Key 无效）
    NetworkAuthFailed,
    /// 协议错误
    NetworkProtocolError,
    /// 连接超时
    NetworkTimeout,

    // 输入错误 (3xxx)
    /// 权限被拒绝
    InputPermissionDenied,
    /// 没有焦点窗口
    InputNoFocusedWindow,
    /// 注入失败
    InputInjectionFailed,
    /// 剪贴板操作失败
    InputClipboardFailed,

    // 配置错误 (4xxx)
    /// 配置加载失败
    ConfigLoadFailed,
    /// 配置保存失败
    ConfigSaveFailed,
    /// 配置无效
    ConfigInvalid,

    // 会话错误 (5xxx)
    /// 会话已在运行
    SessionAlreadyRunning,
    /// 会话未运行
    SessionNotRunning,
    /// 没有可注入的文本
    SessionNoText,

    // 通用错误 (9xxx)
    /// 内部错误
    InternalError,
    /// 操作取消
    OperationCancelled,
    /// 操作超时
    OperationTimeout,
    /// 未知错误
    Unknown,
}

/// 错误上下文信息
///
/// 提供用户友好的错误信息和恢复建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// 错误代码
    pub code: ErrorCode,
    /// 用户友好的错误消息
    pub message: String,
    /// 详细错误信息（用于日志）
    pub detail: Option<String>,
    /// 恢复建议
    pub recovery_hint: Option<String>,
    /// 是否可恢复
    pub recoverable: bool,
}

impl ErrorContext {
    /// 创建新的错误上下文
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            detail: None,
            recovery_hint: None,
            recoverable: true,
        }
    }

    /// 设置详细信息
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// 设置恢复建议
    pub fn with_recovery_hint(mut self, hint: impl Into<String>) -> Self {
        self.recovery_hint = Some(hint.into());
        self
    }

    /// 标记为不可恢复
    pub fn not_recoverable(mut self) -> Self {
        self.recoverable = false;
        self
    }
}

impl AppError {
    /// 获取错误代码
    pub fn code(&self) -> ErrorCode {
        match self {
            // 音频错误
            AppError::Audio(AudioError::DeviceNotFound) => ErrorCode::AudioDeviceNotFound,
            AppError::Audio(AudioError::StreamError(_)) => ErrorCode::AudioStreamError,
            AppError::Audio(AudioError::StreamBuildFailed(_)) => ErrorCode::AudioStreamError,
            AppError::Audio(AudioError::ResampleFailed(_)) => ErrorCode::AudioResampleFailed,
            AppError::Audio(_) => ErrorCode::AudioStreamError,

            // 网络错误
            AppError::Network(NetworkError::ConnectionFailed(_)) => ErrorCode::NetworkConnectionFailed,
            AppError::Network(NetworkError::AuthenticationFailed) => ErrorCode::NetworkAuthFailed,
            AppError::Network(NetworkError::ProtocolError(_)) => ErrorCode::NetworkProtocolError,
            AppError::Network(NetworkError::Timeout(_)) => ErrorCode::NetworkTimeout,
            AppError::Network(_) => ErrorCode::NetworkConnectionFailed,

            // 输入错误
            AppError::Input(InputError::PermissionDenied) => ErrorCode::InputPermissionDenied,
            AppError::Input(InputError::NoFocusedWindow) => ErrorCode::InputNoFocusedWindow,
            AppError::Input(InputError::InjectionFailed(_)) => ErrorCode::InputInjectionFailed,
            AppError::Input(InputError::ClipboardFailed(_)) => ErrorCode::InputClipboardFailed,
            AppError::Input(_) => ErrorCode::InputInjectionFailed,

            // 配置错误
            AppError::Config(ConfigError::Io(_)) => ErrorCode::ConfigLoadFailed,
            AppError::Config(ConfigError::Json(_)) => ErrorCode::ConfigInvalid,
            AppError::Config(_) => ErrorCode::ConfigLoadFailed,

            // 会话错误
            AppError::Session(SessionError::NotRunning) => ErrorCode::SessionNotRunning,
            AppError::Session(SessionError::NoTextToInject) => ErrorCode::SessionNoText,
            AppError::Session(_) => ErrorCode::InternalError,

            // 转写错误
            AppError::Transcription(_) => ErrorCode::InternalError,

            // 通用错误
            AppError::Internal(_) => ErrorCode::InternalError,
            AppError::Cancelled => ErrorCode::OperationCancelled,
            AppError::Timeout(_) => ErrorCode::OperationTimeout,
        }
    }

    /// 获取用户友好的错误消息
    ///
    /// 返回适合直接显示给用户的错误消息
    pub fn user_message(&self) -> String {
        match self {
            // 音频错误
            AppError::Audio(AudioError::DeviceNotFound) => {
                "找不到麦克风设备，请检查音频设置".to_string()
            }
            AppError::Audio(AudioError::StreamBuildFailed(_)) => {
                "无法启动音频录制，请检查麦克风权限".to_string()
            }
            AppError::Audio(AudioError::StreamError(_)) => {
                "音频录制出错，请重试".to_string()
            }
            AppError::Audio(AudioError::ResampleFailed(_)) => {
                "音频处理失败，请重试".to_string()
            }
            AppError::Audio(_) => {
                "音频错误，请检查麦克风设置".to_string()
            }

            // 网络错误
            AppError::Network(NetworkError::ConnectionFailed(_)) => {
                "无法连接到服务器，请检查网络连接".to_string()
            }
            AppError::Network(NetworkError::AuthenticationFailed) => {
                "API Key 无效，请在设置中更新".to_string()
            }
            AppError::Network(NetworkError::ProtocolError(_)) => {
                "通信协议错误，请重试".to_string()
            }
            AppError::Network(NetworkError::Timeout(_)) => {
                "连接超时，请检查网络状况".to_string()
            }
            AppError::Network(NetworkError::ConnectionClosed) => {
                "连接已断开，请重试".to_string()
            }
            AppError::Network(_) => {
                "网络错误，请检查网络连接".to_string()
            }

            // 输入错误
            AppError::Input(InputError::PermissionDenied) => {
                "需要辅助功能权限才能输入文本".to_string()
            }
            AppError::Input(InputError::NoFocusedWindow) => {
                "请先点击要输入文字的位置".to_string()
            }
            AppError::Input(InputError::InjectionFailed(_)) => {
                "文本输入失败，已复制到剪贴板".to_string()
            }
            AppError::Input(InputError::ClipboardFailed(_)) => {
                "剪贴板操作失败".to_string()
            }
            AppError::Input(InputError::PlatformNotSupported(_)) => {
                "当前系统不支持此功能".to_string()
            }
            AppError::Input(_) => {
                "文本注入错误".to_string()
            }

            // 配置错误
            AppError::Config(ConfigError::Io(_)) => {
                "无法读取配置文件".to_string()
            }
            AppError::Config(ConfigError::Json(_)) => {
                "配置文件格式错误".to_string()
            }
            AppError::Config(_) => {
                "配置错误".to_string()
            }

            // 会话错误
            AppError::Session(SessionError::NotRunning) => {
                "没有正在运行的会话".to_string()
            }
            AppError::Session(SessionError::NoTextToInject) => {
                "没有可输入的文本".to_string()
            }
            AppError::Session(_) => {
                "会话错误，请重试".to_string()
            }

            // 转写错误
            AppError::Transcription(_) => {
                "语音识别错误，请重试".to_string()
            }

            // 通用错误
            AppError::Internal(msg) => {
                format!("内部错误: {}", msg)
            }
            AppError::Cancelled => {
                "操作已取消".to_string()
            }
            AppError::Timeout(ms) => {
                format!("操作超时 ({}ms)", ms)
            }
        }
    }

    /// 获取完整的错误上下文
    pub fn context(&self) -> ErrorContext {
        let code = self.code();
        let message = self.user_message();

        let mut ctx = ErrorContext::new(code, message)
            .with_detail(self.to_string());

        // 添加恢复建议
        ctx.recovery_hint = self.recovery_hint();

        // 某些错误不可恢复
        if matches!(
            self,
            AppError::Config(_) | AppError::Internal(_)
        ) {
            ctx = ctx.not_recoverable();
        }

        ctx
    }

    /// 获取恢复建议
    pub fn recovery_hint(&self) -> Option<String> {
        match self {
            AppError::Audio(AudioError::DeviceNotFound) => {
                Some("请确保麦克风已连接，并在系统设置中选择正确的输入设备".to_string())
            }
            AppError::Audio(AudioError::StreamBuildFailed(_)) => {
                Some("请在系统设置中允许应用访问麦克风".to_string())
            }
            AppError::Network(NetworkError::AuthenticationFailed) => {
                Some("请前往设置页面，输入正确的 ElevenLabs API Key".to_string())
            }
            AppError::Network(NetworkError::ConnectionFailed(_)) => {
                Some("请检查网络连接，或稍后重试".to_string())
            }
            AppError::Input(InputError::PermissionDenied) => {
                Some("请在系统设置中为 RaFlow 开启辅助功能权限".to_string())
            }
            AppError::Input(InputError::NoFocusedWindow) => {
                Some("请先点击文本框或输入区域，然后再次尝试".to_string())
            }
            _ => None,
        }
    }

    /// 检查错误是否可恢复
    pub fn is_recoverable(&self) -> bool {
        !matches!(
            self,
            AppError::Config(_) | AppError::Internal(_)
        )
    }

    /// 检查是否是用户取消的操作
    pub fn is_cancelled(&self) -> bool {
        matches!(self, AppError::Cancelled)
    }

    /// 检查是否是超时错误
    pub fn is_timeout(&self) -> bool {
        matches!(self, AppError::Timeout(_))
    }

    /// 检查是否是认证错误
    pub fn is_auth_error(&self) -> bool {
        matches!(self, AppError::Network(NetworkError::AuthenticationFailed))
    }

    /// 检查是否是权限错误
    pub fn is_permission_error(&self) -> bool {
        matches!(self, AppError::Input(InputError::PermissionDenied))
    }
}

/// 应用结果类型
pub type AppResult<T> = Result<T, AppError>;

/// 将任意错误转换为内部错误
impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Internal(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Internal(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code() {
        let err = AppError::Audio(AudioError::DeviceNotFound);
        assert_eq!(err.code(), ErrorCode::AudioDeviceNotFound);

        let err = AppError::Network(NetworkError::AuthenticationFailed);
        assert_eq!(err.code(), ErrorCode::NetworkAuthFailed);

        let err = AppError::Input(InputError::PermissionDenied);
        assert_eq!(err.code(), ErrorCode::InputPermissionDenied);
    }

    #[test]
    fn test_user_message() {
        let err = AppError::Audio(AudioError::DeviceNotFound);
        assert!(err.user_message().contains("麦克风"));

        let err = AppError::Network(NetworkError::AuthenticationFailed);
        assert!(err.user_message().contains("API Key"));

        let err = AppError::Input(InputError::PermissionDenied);
        assert!(err.user_message().contains("权限"));
    }

    #[test]
    fn test_error_context() {
        let err = AppError::Network(NetworkError::AuthenticationFailed);
        let ctx = err.context();

        assert_eq!(ctx.code, ErrorCode::NetworkAuthFailed);
        assert!(!ctx.message.is_empty());
        assert!(ctx.detail.is_some());
        assert!(ctx.recovery_hint.is_some());
        assert!(ctx.recoverable);
    }

    #[test]
    fn test_recoverable() {
        // 可恢复的错误
        let err = AppError::Network(NetworkError::ConnectionFailed("test".to_string()));
        assert!(err.is_recoverable());

        // 不可恢复的错误
        let err = AppError::Internal("fatal".to_string());
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_error_predicates() {
        let err = AppError::Cancelled;
        assert!(err.is_cancelled());
        assert!(!err.is_timeout());

        let err = AppError::Timeout(5000);
        assert!(err.is_timeout());
        assert!(!err.is_cancelled());

        let err = AppError::Network(NetworkError::AuthenticationFailed);
        assert!(err.is_auth_error());

        let err = AppError::Input(InputError::PermissionDenied);
        assert!(err.is_permission_error());
    }

    #[test]
    fn test_from_string() {
        let err: AppError = "test error".into();
        match err {
            AppError::Internal(msg) => assert_eq!(msg, "test error"),
            _ => panic!("Expected Internal error"),
        }
    }

    #[test]
    fn test_error_code_serialization() {
        let code = ErrorCode::AudioDeviceNotFound;
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, "\"AUDIO_DEVICE_NOT_FOUND\"");

        let deserialized: ErrorCode = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, code);
    }

    #[test]
    fn test_error_context_serialization() {
        let ctx = ErrorContext::new(ErrorCode::NetworkAuthFailed, "Test message")
            .with_detail("Detailed error")
            .with_recovery_hint("Try again");

        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("NETWORK_AUTH_FAILED"));
        assert!(json.contains("Test message"));

        let deserialized: ErrorContext = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, ErrorCode::NetworkAuthFailed);
        assert_eq!(deserialized.message, "Test message");
    }
}
