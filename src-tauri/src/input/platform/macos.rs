//! macOS 平台特定实现
//!
//! 提供 macOS 特定的功能：
//! - 辅助功能权限检测和请求
//! - 麦克风权限检测
//! - 系统设置打开
//!
//! # 辅助功能权限
//!
//! macOS 要求应用获得辅助功能权限才能：
//! - 模拟键盘输入
//! - 监听全局热键
//! - 检测其他应用的窗口
//!
//! # 使用示例
//!
//! ```ignore
//! use raflow_lib::input::platform::macos;
//!
//! // 检查权限
//! if !macos::check_accessibility_permission() {
//!     // 请求权限（显示系统对话框）
//!     macos::request_accessibility_permission();
//! }
//! ```

#![cfg(target_os = "macos")]

use crate::input::platform::PermissionStatus;

/// 检查辅助功能权限
///
/// 检查应用是否已获得辅助功能权限。
/// 此方法不会显示任何对话框。
///
/// # 返回
///
/// 如果应用已被信任，返回 `true`
///
/// # 示例
///
/// ```ignore
/// if macos::check_accessibility_permission() {
///     println!("Accessibility permission granted");
/// } else {
///     println!("Accessibility permission denied");
/// }
/// ```
pub fn check_accessibility_permission() -> bool {
    macos_accessibility_client::accessibility::application_is_trusted()
}

/// 请求辅助功能权限
///
/// 如果应用未被信任，会显示系统权限请求对话框。
/// 用户授权后需要重启应用才能生效。
///
/// # 返回
///
/// 如果应用已被信任或用户即将授权，返回 `true`
///
/// # 注意
///
/// 此方法会阻塞直到用户做出选择。
/// 建议在后台线程中调用。
///
/// # 示例
///
/// ```ignore
/// if !macos::check_accessibility_permission() {
///     let granted = macos::request_accessibility_permission();
///     if !granted {
///         println!("User denied accessibility permission");
///     }
/// }
/// ```
pub fn request_accessibility_permission() -> bool {
    macos_accessibility_client::accessibility::application_is_trusted_with_prompt()
}

/// 检查麦克风权限
///
/// 检查应用是否有权访问麦克风。
///
/// # 返回
///
/// 返回麦克风权限状态
pub fn check_microphone_permission() -> PermissionStatus {
    // macOS 的麦克风权限由系统自动管理
    // 当应用首次尝试访问麦克风时会显示权限对话框
    // 这里我们无法直接检查，所以返回 NotDetermined
    PermissionStatus::NotDetermined
}

/// 打开辅助功能设置
///
/// 打开系统偏好设置的"安全性与隐私" > "辅助功能"面板
pub fn open_accessibility_settings() {
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn();
}

/// 打开麦克风权限设置
///
/// 打开系统偏好设置的"安全性与隐私" > "麦克风"面板
pub fn open_microphone_settings() {
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
        .spawn();
}

/// 打开隐私设置主页面
///
/// 打开系统偏好设置的"安全性与隐私" > "隐私"面板
pub fn open_privacy_settings() {
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy")
        .spawn();
}

/// 获取 macOS 版本
///
/// # 返回
///
/// 返回格式如 "14.0" 的版本字符串
pub fn get_macos_version() -> Option<String> {
    let output = std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        None
    }
}

/// 检查是否在 Apple Silicon 上运行
///
/// # 返回
///
/// 如果在 ARM64 架构上运行，返回 `true`
pub fn is_apple_silicon() -> bool {
    std::env::consts::ARCH == "aarch64"
}

/// 检查是否通过 Rosetta 2 运行
///
/// # 返回
///
/// 如果是 x86_64 进程在 ARM64 上运行，返回 `true`
pub fn is_rosetta() -> bool {
    if std::env::consts::ARCH != "x86_64" {
        return false;
    }

    // 检查 sysctl.proc_translated
    let output = std::process::Command::new("sysctl")
        .arg("-n")
        .arg("sysctl.proc_translated")
        .output();

    match output {
        Ok(out) => {
            let value = String::from_utf8_lossy(&out.stdout);
            value.trim() == "1"
        }
        Err(_) => false,
    }
}

/// macOS 系统信息
#[derive(Debug, Clone)]
pub struct MacOSInfo {
    /// macOS 版本
    pub version: Option<String>,
    /// 是否是 Apple Silicon
    pub is_apple_silicon: bool,
    /// 是否通过 Rosetta 运行
    pub is_rosetta: bool,
    /// 辅助功能权限状态
    pub accessibility_granted: bool,
}

impl MacOSInfo {
    /// 获取当前系统信息
    pub fn current() -> Self {
        Self {
            version: get_macos_version(),
            is_apple_silicon: is_apple_silicon(),
            is_rosetta: is_rosetta(),
            accessibility_granted: check_accessibility_permission(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_accessibility_permission() {
        // 这个测试只验证函数可以被调用
        // 实际权限状态取决于系统设置
        let _ = check_accessibility_permission();
    }

    #[test]
    fn test_macos_info() {
        let info = MacOSInfo::current();
        // 版本应该存在
        assert!(info.version.is_some());
    }

    #[test]
    fn test_get_macos_version() {
        let version = get_macos_version();
        assert!(version.is_some());
        // 版本格式应该像 "14.0" 或 "13.5.1"
        let v = version.unwrap();
        assert!(v.contains('.'));
    }

    #[test]
    fn test_is_apple_silicon() {
        // 这个测试只验证函数可以被调用
        let _ = is_apple_silicon();
    }

    #[test]
    fn test_is_rosetta() {
        // 这个测试只验证函数可以被调用
        let _ = is_rosetta();
    }
}
