//! 平台特定实现模块
//!
//! 提供跨平台的系统交互功能，包括：
//! - 辅助功能权限检测（macOS）
//! - 显示服务器检测（Linux）
//! - 平台能力查询
//!
//! # 架构
//!
//! ```text
//! platform/
//! ├── mod.rs          - 平台抽象和能力查询
//! ├── macos.rs        - macOS 特定实现
//! ├── linux.rs        - Linux 特定实现
//! └── windows.rs      - Windows 特定实现
//! ```

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

use serde::{Deserialize, Serialize};

/// 平台类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// Windows
    Windows,
    /// macOS
    MacOS,
    /// Linux
    Linux,
    /// 未知平台
    Unknown,
}

impl Platform {
    /// 获取当前平台
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[cfg(target_os = "macos")]
        return Platform::MacOS;

        #[cfg(target_os = "linux")]
        return Platform::Linux;

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return Platform::Unknown;
    }

    /// 获取平台名称
    pub fn name(&self) -> &'static str {
        match self {
            Platform::Windows => "Windows",
            Platform::MacOS => "macOS",
            Platform::Linux => "Linux",
            Platform::Unknown => "Unknown",
        }
    }
}

/// 平台能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    /// 平台类型
    pub platform: Platform,
    /// 是否支持全局热键
    pub global_shortcuts: bool,
    /// 是否支持键盘模拟
    pub keyboard_simulation: bool,
    /// 是否支持剪贴板操作
    pub clipboard: bool,
    /// 是否支持窗口检测
    pub window_detection: bool,
    /// 是否支持系统托盘
    pub system_tray: bool,
    /// 是否支持透明窗口
    pub transparent_windows: bool,
    /// 是否需要辅助功能权限
    pub requires_accessibility: bool,
    /// 显示服务器类型（Linux 专用）
    pub display_server: Option<String>,
}

impl PlatformCapabilities {
    /// 获取当前平台的能力
    pub fn current() -> Self {
        let platform = Platform::current();

        match platform {
            Platform::Windows => Self::windows(),
            Platform::MacOS => Self::macos(),
            Platform::Linux => Self::linux(),
            Platform::Unknown => Self::unknown(),
        }
    }

    /// Windows 平台能力
    fn windows() -> Self {
        Self {
            platform: Platform::Windows,
            global_shortcuts: true,
            keyboard_simulation: true,
            clipboard: true,
            window_detection: true,
            system_tray: true,
            transparent_windows: true,
            requires_accessibility: false,
            display_server: None,
        }
    }

    /// macOS 平台能力
    fn macos() -> Self {
        Self {
            platform: Platform::MacOS,
            global_shortcuts: true,
            keyboard_simulation: true,
            clipboard: true,
            window_detection: true,
            system_tray: true,
            transparent_windows: true,
            requires_accessibility: true,
            display_server: None,
        }
    }

    /// Linux 平台能力
    fn linux() -> Self {
        #[cfg(target_os = "linux")]
        let display_server = Some(linux::detect_display_server().name().to_string());

        #[cfg(not(target_os = "linux"))]
        let display_server = None;

        Self {
            platform: Platform::Linux,
            global_shortcuts: true,
            keyboard_simulation: true,
            clipboard: true,
            window_detection: true,
            system_tray: true,
            transparent_windows: true,
            requires_accessibility: false,
            display_server,
        }
    }

    /// 未知平台能力
    fn unknown() -> Self {
        Self {
            platform: Platform::Unknown,
            global_shortcuts: false,
            keyboard_simulation: false,
            clipboard: false,
            window_detection: false,
            system_tray: false,
            transparent_windows: false,
            requires_accessibility: false,
            display_server: None,
        }
    }
}

/// 权限状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionStatus {
    /// 已授权
    Granted,
    /// 被拒绝
    Denied,
    /// 未确定（需要请求）
    NotDetermined,
    /// 不适用（平台不需要此权限）
    NotApplicable,
}

/// 检查辅助功能权限
///
/// # 返回
///
/// 返回辅助功能权限状态
pub fn check_accessibility_permission() -> PermissionStatus {
    #[cfg(target_os = "macos")]
    {
        if macos::check_accessibility_permission() {
            PermissionStatus::Granted
        } else {
            PermissionStatus::Denied
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        PermissionStatus::NotApplicable
    }
}

/// 请求辅助功能权限
///
/// 在 macOS 上会显示系统权限请求对话框
///
/// # 返回
///
/// 返回是否成功获得权限
pub fn request_accessibility_permission() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos::request_accessibility_permission()
    }

    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// 检查麦克风权限
///
/// # 返回
///
/// 返回麦克风权限状态
pub fn check_microphone_permission() -> PermissionStatus {
    #[cfg(target_os = "macos")]
    {
        macos::check_microphone_permission()
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Windows 和 Linux 在使用时自动请求权限
        PermissionStatus::NotApplicable
    }
}

/// 打开系统权限设置
///
/// 在 macOS 上打开系统偏好设置的安全与隐私面板
pub fn open_permission_settings() {
    #[cfg(target_os = "macos")]
    {
        macos::open_accessibility_settings();
    }

    #[cfg(target_os = "windows")]
    {
        windows::open_settings();
    }

    #[cfg(target_os = "linux")]
    {
        // Linux 通常不需要特殊权限
        tracing::info!("Linux does not require special permission settings");
    }
}

/// 获取系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// 平台
    pub platform: Platform,
    /// 操作系统版本
    pub os_version: String,
    /// 架构
    pub arch: &'static str,
    /// 显示服务器（Linux）
    pub display_server: Option<String>,
}

impl SystemInfo {
    /// 获取当前系统信息
    pub fn current() -> Self {
        let platform = Platform::current();

        #[cfg(target_os = "linux")]
        let display_server = Some(linux::detect_display_server().name().to_string());

        #[cfg(not(target_os = "linux"))]
        let display_server = None;

        Self {
            platform,
            os_version: Self::get_os_version(),
            arch: std::env::consts::ARCH,
            display_server,
        }
    }

    fn get_os_version() -> String {
        std::env::consts::OS.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_current() {
        let platform = Platform::current();

        #[cfg(target_os = "windows")]
        assert_eq!(platform, Platform::Windows);

        #[cfg(target_os = "macos")]
        assert_eq!(platform, Platform::MacOS);

        #[cfg(target_os = "linux")]
        assert_eq!(platform, Platform::Linux);
    }

    #[test]
    fn test_platform_name() {
        assert_eq!(Platform::Windows.name(), "Windows");
        assert_eq!(Platform::MacOS.name(), "macOS");
        assert_eq!(Platform::Linux.name(), "Linux");
    }

    #[test]
    fn test_platform_capabilities() {
        let caps = PlatformCapabilities::current();
        assert!(caps.clipboard);
        assert!(caps.system_tray);
    }

    #[test]
    fn test_system_info() {
        let info = SystemInfo::current();
        assert!(!info.os_version.is_empty());
        assert!(!info.arch.is_empty());
    }

    #[test]
    fn test_permission_status_serialization() {
        let status = PermissionStatus::Granted;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"granted\"");
    }

    #[test]
    fn test_platform_serialization() {
        let platform = Platform::Windows;
        let json = serde_json::to_string(&platform).unwrap();
        assert_eq!(json, "\"windows\"");

        let platform = Platform::MacOS;
        let json = serde_json::to_string(&platform).unwrap();
        assert_eq!(json, "\"macos\"");
    }
}
