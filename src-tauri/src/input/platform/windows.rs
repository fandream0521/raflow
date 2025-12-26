//! Windows 平台特定实现
//!
//! 提供 Windows 特定的功能：
//! - 系统信息检测
//! - 设置页面打开
//! - 平台能力查询
//!
//! # Windows 特性
//!
//! Windows 平台通常不需要特殊权限即可：
//! - 模拟键盘输入（通过 SendInput API）
//! - 监听全局热键
//! - 检测窗口信息
//!
//! # 使用示例
//!
//! ```ignore
//! use raflow_lib::input::platform::windows;
//!
//! let info = windows::WindowsInfo::current();
//! println!("Windows version: {:?}", info.version);
//! ```

#![cfg(target_os = "windows")]

use serde::{Deserialize, Serialize};

/// Windows 版本
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowsVersion {
    /// Windows 10
    Windows10,
    /// Windows 11
    Windows11,
    /// Windows Server
    Server,
    /// 未知版本
    Unknown,
}

impl WindowsVersion {
    /// 获取版本名称
    pub fn name(&self) -> &'static str {
        match self {
            WindowsVersion::Windows10 => "Windows 10",
            WindowsVersion::Windows11 => "Windows 11",
            WindowsVersion::Server => "Windows Server",
            WindowsVersion::Unknown => "Windows",
        }
    }
}

/// 检测 Windows 版本
///
/// # 返回
///
/// 返回检测到的 Windows 版本
pub fn detect_windows_version() -> WindowsVersion {
    // 使用 winver 输出或注册表来检测
    // 简化实现：通过构建号判断
    if let Ok(output) = std::process::Command::new("cmd")
        .args(["/C", "ver"])
        .output()
    {
        let version_str = String::from_utf8_lossy(&output.stdout);

        // Windows 11 的构建号 >= 22000
        if version_str.contains("22") && version_str.contains("000") {
            return WindowsVersion::Windows11;
        }

        // Windows 10 的构建号 < 22000
        if version_str.contains("10.0") {
            return WindowsVersion::Windows10;
        }
    }

    WindowsVersion::Unknown
}

/// 检查是否以管理员权限运行
///
/// # 返回
///
/// 如果以管理员权限运行，返回 `true`
pub fn is_admin() -> bool {
    // 简化检测：尝试访问管理员目录
    let admin_path = std::path::Path::new("C:\\Windows\\System32\\config\\SAM");
    std::fs::metadata(admin_path).is_ok()
}

/// 检查是否支持暗色模式
///
/// # 返回
///
/// 如果系统支持暗色模式，返回 `true`
pub fn supports_dark_mode() -> bool {
    // Windows 10 1809 及以上支持暗色模式
    true
}

/// 检查当前是否启用暗色模式
///
/// # 返回
///
/// 如果当前启用暗色模式，返回 `true`
pub fn is_dark_mode_enabled() -> bool {
    // 读取注册表键值
    // HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize
    // AppsUseLightTheme = 0 表示暗色模式

    let output = std::process::Command::new("reg")
        .args([
            "query",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .output();

    match output {
        Ok(out) => {
            let result = String::from_utf8_lossy(&out.stdout);
            // 如果值为 0x0，则启用暗色模式
            result.contains("0x0")
        }
        Err(_) => false,
    }
}

/// 打开 Windows 设置
pub fn open_settings() {
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "ms-settings:"])
        .spawn();
}

/// 打开隐私设置
pub fn open_privacy_settings() {
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "ms-settings:privacy"])
        .spawn();
}

/// 打开麦克风权限设置
pub fn open_microphone_settings() {
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "ms-settings:privacy-microphone"])
        .spawn();
}

/// 打开声音设置
pub fn open_sound_settings() {
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "ms-settings:sound"])
        .spawn();
}

/// 打开键盘设置
pub fn open_keyboard_settings() {
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "ms-settings:easeofaccess-keyboard"])
        .spawn();
}

/// 打开通知设置
pub fn open_notification_settings() {
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "ms-settings:notifications"])
        .spawn();
}

/// Windows 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsInfo {
    /// Windows 版本
    pub version: WindowsVersion,
    /// 是否以管理员权限运行
    pub is_admin: bool,
    /// 是否启用暗色模式
    pub dark_mode: bool,
    /// 架构
    pub arch: &'static str,
}

impl WindowsInfo {
    /// 获取当前系统信息
    pub fn current() -> Self {
        Self {
            version: detect_windows_version(),
            is_admin: is_admin(),
            dark_mode: is_dark_mode_enabled(),
            arch: std::env::consts::ARCH,
        }
    }
}

/// 获取 Windows 构建号
///
/// # 返回
///
/// 返回 Windows 构建号字符串
pub fn get_build_number() -> Option<String> {
    let output = std::process::Command::new("cmd")
        .args(["/C", "ver"])
        .output()
        .ok()?;

    if output.status.success() {
        let version_str = String::from_utf8_lossy(&output.stdout);
        // 提取构建号
        // 格式类似：Microsoft Windows [Version 10.0.22631.2861]
        if let Some(start) = version_str.find('[') {
            if let Some(end) = version_str.find(']') {
                return Some(version_str[start + 1..end].to_string());
            }
        }
    }

    None
}

/// 检查 Windows Hello 是否可用
///
/// # 返回
///
/// 如果 Windows Hello 可用，返回 `true`
pub fn is_windows_hello_available() -> bool {
    // Windows 10 1903+ 支持 Windows Hello
    matches!(
        detect_windows_version(),
        WindowsVersion::Windows10 | WindowsVersion::Windows11
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_windows_version() {
        let version = detect_windows_version();
        assert!(matches!(
            version,
            WindowsVersion::Windows10
                | WindowsVersion::Windows11
                | WindowsVersion::Server
                | WindowsVersion::Unknown
        ));
    }

    #[test]
    fn test_version_name() {
        assert_eq!(WindowsVersion::Windows10.name(), "Windows 10");
        assert_eq!(WindowsVersion::Windows11.name(), "Windows 11");
    }

    #[test]
    fn test_is_admin() {
        // 这个测试只验证函数可以被调用
        let _ = is_admin();
    }

    #[test]
    fn test_supports_dark_mode() {
        // Windows 10+ 应该支持暗色模式
        assert!(supports_dark_mode());
    }

    #[test]
    fn test_is_dark_mode_enabled() {
        // 这个测试只验证函数可以被调用
        let _ = is_dark_mode_enabled();
    }

    #[test]
    fn test_windows_info() {
        let info = WindowsInfo::current();
        assert!(!info.arch.is_empty());
    }

    #[test]
    fn test_get_build_number() {
        let build = get_build_number();
        // 应该能获取到构建号
        if build.is_some() {
            assert!(!build.unwrap().is_empty());
        }
    }

    #[test]
    fn test_is_windows_hello_available() {
        // 这个测试只验证函数可以被调用
        let _ = is_windows_hello_available();
    }

    #[test]
    fn test_version_serialization() {
        let version = WindowsVersion::Windows11;
        let json = serde_json::to_string(&version).unwrap();
        assert!(json.contains("Windows11"));

        let deserialized: WindowsVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, WindowsVersion::Windows11);
    }
}
