//! Linux 平台特定实现
//!
//! 提供 Linux 特定的功能：
//! - 显示服务器检测（X11/Wayland）
//! - 输入方法检测
//! - 桌面环境检测
//!
//! # 显示服务器
//!
//! Linux 支持多种显示服务器：
//! - X11: 传统显示服务器，广泛支持
//! - Wayland: 现代显示协议，更安全但兼容性有限
//!
//! # 键盘模拟限制
//!
//! - X11: 完全支持 XTest 扩展
//! - Wayland: 由于安全限制，需要特殊处理（如 wlroots 协议或 libei）
//!
//! # 使用示例
//!
//! ```ignore
//! use raflow_lib::input::platform::linux;
//!
//! let display_server = linux::detect_display_server();
//! println!("Display server: {:?}", display_server);
//!
//! if display_server == linux::DisplayServer::Wayland {
//!     println!("Running on Wayland - some features may be limited");
//! }
//! ```

#![cfg(target_os = "linux")]

use serde::{Deserialize, Serialize};

/// 显示服务器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayServer {
    /// X11 显示服务器
    X11,
    /// Wayland 合成器
    Wayland,
    /// 未知或未检测到
    Unknown,
}

impl DisplayServer {
    /// 获取显示服务器名称
    pub fn name(&self) -> &'static str {
        match self {
            DisplayServer::X11 => "X11",
            DisplayServer::Wayland => "Wayland",
            DisplayServer::Unknown => "Unknown",
        }
    }

    /// 检查是否支持完整的键盘模拟
    pub fn supports_keyboard_simulation(&self) -> bool {
        match self {
            DisplayServer::X11 => true,
            DisplayServer::Wayland => false, // 需要特殊协议
            DisplayServer::Unknown => false,
        }
    }

    /// 检查是否支持窗口检测
    pub fn supports_window_detection(&self) -> bool {
        match self {
            DisplayServer::X11 => true,
            DisplayServer::Wayland => true, // 通过 xdg-foreign 或其他协议
            DisplayServer::Unknown => false,
        }
    }

    /// 获取建议的输入方法
    pub fn recommended_input_method(&self) -> &'static str {
        match self {
            DisplayServer::X11 => "xtest",
            DisplayServer::Wayland => "wlroots-virtual-keyboard",
            DisplayServer::Unknown => "clipboard",
        }
    }
}

/// 检测当前显示服务器
///
/// 通过检查环境变量确定当前使用的显示服务器。
///
/// # 检测逻辑
///
/// 1. 检查 `WAYLAND_DISPLAY` 环境变量
/// 2. 检查 `XDG_SESSION_TYPE` 环境变量
/// 3. 检查 `DISPLAY` 环境变量
///
/// # 返回
///
/// 返回检测到的显示服务器类型
///
/// # 示例
///
/// ```ignore
/// let server = linux::detect_display_server();
/// match server {
///     linux::DisplayServer::X11 => println!("Running on X11"),
///     linux::DisplayServer::Wayland => println!("Running on Wayland"),
///     linux::DisplayServer::Unknown => println!("Unknown display server"),
/// }
/// ```
pub fn detect_display_server() -> DisplayServer {
    // 优先检查 Wayland
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        return DisplayServer::Wayland;
    }

    // 检查 XDG_SESSION_TYPE
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        match session_type.to_lowercase().as_str() {
            "wayland" => return DisplayServer::Wayland,
            "x11" => return DisplayServer::X11,
            _ => {}
        }
    }

    // 检查 DISPLAY（X11）
    if std::env::var("DISPLAY").is_ok() {
        return DisplayServer::X11;
    }

    DisplayServer::Unknown
}

/// 桌面环境类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DesktopEnvironment {
    /// GNOME
    Gnome,
    /// KDE Plasma
    Kde,
    /// Xfce
    Xfce,
    /// Cinnamon
    Cinnamon,
    /// MATE
    Mate,
    /// LXQt
    Lxqt,
    /// i3 窗口管理器
    I3,
    /// Sway (Wayland i3)
    Sway,
    /// 其他或未知
    Other(String),
    /// 未检测到
    Unknown,
}

impl DesktopEnvironment {
    /// 获取桌面环境名称
    pub fn name(&self) -> &str {
        match self {
            DesktopEnvironment::Gnome => "GNOME",
            DesktopEnvironment::Kde => "KDE Plasma",
            DesktopEnvironment::Xfce => "Xfce",
            DesktopEnvironment::Cinnamon => "Cinnamon",
            DesktopEnvironment::Mate => "MATE",
            DesktopEnvironment::Lxqt => "LXQt",
            DesktopEnvironment::I3 => "i3",
            DesktopEnvironment::Sway => "Sway",
            DesktopEnvironment::Other(name) => name,
            DesktopEnvironment::Unknown => "Unknown",
        }
    }

    /// 检查是否是 Wayland 原生
    pub fn is_wayland_native(&self) -> bool {
        matches!(
            self,
            DesktopEnvironment::Gnome
                | DesktopEnvironment::Kde
                | DesktopEnvironment::Sway
        )
    }
}

/// 检测当前桌面环境
///
/// # 返回
///
/// 返回检测到的桌面环境类型
pub fn detect_desktop_environment() -> DesktopEnvironment {
    // 检查 XDG_CURRENT_DESKTOP
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        let desktop = desktop.to_lowercase();
        return match desktop.as_str() {
            "gnome" | "gnome-xorg" | "gnome-wayland" | "ubuntu:gnome" | "unity" => {
                DesktopEnvironment::Gnome
            }
            "kde" | "plasma" | "kde-plasma" => DesktopEnvironment::Kde,
            "xfce" | "xfce4" => DesktopEnvironment::Xfce,
            "cinnamon" | "x-cinnamon" => DesktopEnvironment::Cinnamon,
            "mate" => DesktopEnvironment::Mate,
            "lxqt" => DesktopEnvironment::Lxqt,
            "i3" | "i3wm" => DesktopEnvironment::I3,
            "sway" => DesktopEnvironment::Sway,
            _ => {
                if desktop.is_empty() {
                    DesktopEnvironment::Unknown
                } else {
                    DesktopEnvironment::Other(desktop)
                }
            }
        };
    }

    // 检查 DESKTOP_SESSION
    if let Ok(session) = std::env::var("DESKTOP_SESSION") {
        let session = session.to_lowercase();
        if session.contains("gnome") {
            return DesktopEnvironment::Gnome;
        }
        if session.contains("kde") || session.contains("plasma") {
            return DesktopEnvironment::Kde;
        }
        if session.contains("xfce") {
            return DesktopEnvironment::Xfce;
        }
    }

    DesktopEnvironment::Unknown
}

/// 检查是否在 Flatpak 沙箱中运行
///
/// # 返回
///
/// 如果在 Flatpak 中运行，返回 `true`
pub fn is_flatpak() -> bool {
    std::path::Path::new("/.flatpak-info").exists()
}

/// 检查是否在 Snap 沙箱中运行
///
/// # 返回
///
/// 如果在 Snap 中运行，返回 `true`
pub fn is_snap() -> bool {
    std::env::var("SNAP").is_ok()
}

/// 检查是否在容器/沙箱中运行
///
/// # 返回
///
/// 如果在 Flatpak 或 Snap 中运行，返回 `true`
pub fn is_sandboxed() -> bool {
    is_flatpak() || is_snap()
}

/// 检查 XTest 扩展是否可用
///
/// XTest 是 X11 上用于键盘模拟的扩展。
///
/// # 返回
///
/// 如果 XTest 可能可用，返回 `true`
pub fn is_xtest_available() -> bool {
    // 简单检查是否在 X11 下
    detect_display_server() == DisplayServer::X11
}

/// Linux 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxInfo {
    /// 显示服务器
    pub display_server: DisplayServer,
    /// 桌面环境
    pub desktop_environment: DesktopEnvironment,
    /// 是否在 Flatpak 中
    pub is_flatpak: bool,
    /// 是否在 Snap 中
    pub is_snap: bool,
    /// XTest 是否可用
    pub xtest_available: bool,
    /// 是否支持键盘模拟
    pub keyboard_simulation_supported: bool,
}

impl LinuxInfo {
    /// 获取当前系统信息
    pub fn current() -> Self {
        let display_server = detect_display_server();
        let desktop_environment = detect_desktop_environment();

        Self {
            display_server,
            desktop_environment,
            is_flatpak: is_flatpak(),
            is_snap: is_snap(),
            xtest_available: is_xtest_available(),
            keyboard_simulation_supported: display_server.supports_keyboard_simulation(),
        }
    }
}

/// 获取 Linux 发行版名称
///
/// 从 /etc/os-release 读取发行版信息
///
/// # 返回
///
/// 返回发行版名称，如 "Ubuntu 22.04"
pub fn get_distro_name() -> Option<String> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;

    for line in content.lines() {
        if line.starts_with("PRETTY_NAME=") {
            let name = line.strip_prefix("PRETTY_NAME=")?;
            // 去除引号
            return Some(name.trim_matches('"').to_string());
        }
    }

    None
}

/// 获取 Linux 内核版本
///
/// # 返回
///
/// 返回内核版本字符串
pub fn get_kernel_version() -> Option<String> {
    let output = std::process::Command::new("uname")
        .arg("-r")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_display_server() {
        let server = detect_display_server();
        // 应该返回某个有效值
        assert!(matches!(
            server,
            DisplayServer::X11 | DisplayServer::Wayland | DisplayServer::Unknown
        ));
    }

    #[test]
    fn test_display_server_name() {
        assert_eq!(DisplayServer::X11.name(), "X11");
        assert_eq!(DisplayServer::Wayland.name(), "Wayland");
        assert_eq!(DisplayServer::Unknown.name(), "Unknown");
    }

    #[test]
    fn test_display_server_capabilities() {
        // X11 支持完整的键盘模拟
        assert!(DisplayServer::X11.supports_keyboard_simulation());

        // Wayland 需要特殊协议
        assert!(!DisplayServer::Wayland.supports_keyboard_simulation());
    }

    #[test]
    fn test_detect_desktop_environment() {
        let de = detect_desktop_environment();
        // 应该返回某个有效值
        match de {
            DesktopEnvironment::Unknown => {}
            _ => {
                assert!(!de.name().is_empty());
            }
        }
    }

    #[test]
    fn test_desktop_environment_wayland_native() {
        assert!(DesktopEnvironment::Gnome.is_wayland_native());
        assert!(DesktopEnvironment::Kde.is_wayland_native());
        assert!(DesktopEnvironment::Sway.is_wayland_native());
        assert!(!DesktopEnvironment::I3.is_wayland_native());
        assert!(!DesktopEnvironment::Xfce.is_wayland_native());
    }

    #[test]
    fn test_sandbox_detection() {
        // 这些测试只验证函数可以被调用
        let _ = is_flatpak();
        let _ = is_snap();
        let _ = is_sandboxed();
    }

    #[test]
    fn test_linux_info() {
        let info = LinuxInfo::current();
        // 验证结构体可以正确创建
        assert!(matches!(
            info.display_server,
            DisplayServer::X11 | DisplayServer::Wayland | DisplayServer::Unknown
        ));
    }

    #[test]
    fn test_get_distro_name() {
        // 这个测试在 Linux 上应该返回有效值
        let name = get_distro_name();
        // 在 CI 环境中可能没有 /etc/os-release
        if name.is_some() {
            assert!(!name.unwrap().is_empty());
        }
    }

    #[test]
    fn test_get_kernel_version() {
        let version = get_kernel_version();
        // 应该能获取到内核版本
        if version.is_some() {
            assert!(!version.unwrap().is_empty());
        }
    }

    #[test]
    fn test_display_server_serialization() {
        let server = DisplayServer::Wayland;
        let json = serde_json::to_string(&server).unwrap();
        assert_eq!(json, "\"wayland\"");

        let deserialized: DisplayServer = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, DisplayServer::Wayland);
    }
}
