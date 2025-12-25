//! 窗口检测模块
//!
//! 提供获取当前焦点窗口信息的功能，用于文本注入前的上下文检测
//!
//! # 功能
//!
//! - 获取当前活动窗口信息
//! - 判断窗口是否为文本输入上下文
//! - 跨平台支持 (Windows, macOS, Linux)
//!
//! # 使用示例
//!
//! ```ignore
//! use raflow_lib::input::window::{get_focused_window, is_text_input_context};
//!
//! // 获取当前焦点窗口
//! if let Ok(window) = get_focused_window() {
//!     println!("当前窗口: {} - {}", window.app_name, window.title);
//!     println!("进程 ID: {}", window.process_id);
//! }
//!
//! // 检查是否为文本输入环境
//! if is_text_input_context() {
//!     println!("可以进行文本注入");
//! }
//! ```
//!
//! # 平台说明
//!
//! - **Windows**: 使用 Windows API 获取窗口信息
//! - **macOS**: 需要屏幕录制权限才能获取窗口标题
//! - **Linux (X11)**: 直接支持
//! - **Linux (GNOME > 41)**: 需要安装并启用 x-win 扩展

use super::error::{InputError, InputResult};

/// 窗口信息
///
/// 包含当前活动窗口的基本信息
#[derive(Debug, Clone, PartialEq)]
pub struct WindowInfo {
    /// 应用程序名称
    pub app_name: String,
    /// 窗口标题
    pub title: String,
    /// 进程 ID
    pub process_id: u32,
    /// 可执行文件名称
    pub exec_name: String,
    /// 可执行文件路径
    pub exec_path: String,
    /// 窗口 ID
    pub window_id: u32,
}

impl WindowInfo {
    /// 检查窗口是否属于指定应用
    ///
    /// # Arguments
    ///
    /// * `app_names` - 应用名称列表（部分匹配）
    pub fn is_app(&self, app_names: &[&str]) -> bool {
        app_names
            .iter()
            .any(|name| self.app_name.to_lowercase().contains(&name.to_lowercase()))
    }

    /// 检查窗口标题是否包含指定文本
    ///
    /// # Arguments
    ///
    /// * `text` - 要搜索的文本
    pub fn title_contains(&self, text: &str) -> bool {
        self.title.to_lowercase().contains(&text.to_lowercase())
    }
}

/// 获取当前焦点窗口信息
///
/// # Returns
///
/// 返回当前活动窗口的信息，如果没有焦点窗口则返回错误
///
/// # Errors
///
/// - `InputError::NoFocusedWindow` - 没有找到焦点窗口
/// - `InputError::WindowDetectionFailed` - 窗口检测失败
///
/// # Example
///
/// ```ignore
/// use raflow_lib::input::window::get_focused_window;
///
/// match get_focused_window() {
///     Ok(window) => {
///         println!("当前窗口: {}", window.title);
///     }
///     Err(e) => {
///         eprintln!("获取窗口失败: {}", e);
///     }
/// }
/// ```
pub fn get_focused_window() -> InputResult<WindowInfo> {
    match x_win::get_active_window() {
        Ok(active_window) => {
            let info = WindowInfo {
                app_name: active_window.info.name.clone(),
                title: active_window.title.clone(),
                process_id: active_window.info.process_id,
                exec_name: active_window.info.exec_name.clone(),
                exec_path: active_window.info.path.clone(),
                window_id: active_window.id,
            };

            tracing::debug!(
                app = %info.app_name,
                title = %info.title,
                pid = info.process_id,
                "Got focused window"
            );

            Ok(info)
        }
        Err(e) => {
            tracing::warn!(error = ?e, "Failed to get active window");
            Err(InputError::WindowDetectionFailed(format!("{:?}", e)))
        }
    }
}

/// 检查当前焦点是否在文本输入上下文中
///
/// 基于应用程序名称的启发式判断，用于决定是否可以安全地进行文本注入
///
/// # Returns
///
/// 如果当前焦点可能在文本输入区域则返回 `true`
///
/// # 支持的应用类型
///
/// - 文本编辑器：VS Code, Notepad, Sublime Text, Vim, Emacs
/// - Office 应用：Word, Excel, PowerPoint, WPS
/// - 浏览器：Chrome, Firefox, Safari, Edge
/// - 通讯工具：Slack, Discord, Teams, WeChat, QQ
/// - 终端：Terminal, iTerm, Windows Terminal, PowerShell
/// - 其他：Obsidian, Notion, Typora
///
/// # Example
///
/// ```ignore
/// use raflow_lib::input::window::is_text_input_context;
///
/// if is_text_input_context() {
///     // 可以进行文本注入
///     inject_text("Hello, World!");
/// } else {
///     // 降级到剪贴板模式
///     copy_to_clipboard("Hello, World!");
/// }
/// ```
pub fn is_text_input_context() -> bool {
    if let Ok(window) = get_focused_window() {
        is_text_input_app(&window)
    } else {
        false
    }
}

/// 检查窗口是否为文本输入应用
///
/// # Arguments
///
/// * `window` - 窗口信息
fn is_text_input_app(window: &WindowInfo) -> bool {
    // 常见的文本输入应用
    const TEXT_INPUT_APPS: &[&str] = &[
        // 文本编辑器
        "code",
        "visual studio code",
        "notepad",
        "sublime",
        "vim",
        "nvim",
        "emacs",
        "atom",
        "textmate",
        "notepad++",
        "gedit",
        "kate",
        // Office 应用
        "word",
        "excel",
        "powerpoint",
        "wps",
        "libreoffice",
        "openoffice",
        "pages",
        "numbers",
        "keynote",
        // 浏览器
        "chrome",
        "firefox",
        "safari",
        "edge",
        "brave",
        "opera",
        "vivaldi",
        "arc",
        // 通讯工具
        "slack",
        "discord",
        "teams",
        "wechat",
        "微信",
        "qq",
        "telegram",
        "whatsapp",
        "signal",
        "zoom",
        "skype",
        "飞书",
        "feishu",
        "钉钉",
        "dingtalk",
        // 终端
        "terminal",
        "iterm",
        "windows terminal",
        "powershell",
        "cmd",
        "konsole",
        "gnome-terminal",
        "alacritty",
        "warp",
        "hyper",
        // 笔记应用
        "obsidian",
        "notion",
        "typora",
        "bear",
        "ulysses",
        "evernote",
        "onenote",
        "印象笔记",
        // IDE
        "idea",
        "intellij",
        "pycharm",
        "webstorm",
        "goland",
        "rider",
        "android studio",
        "xcode",
        "eclipse",
        // 其他
        "mail",
        "outlook",
        "thunderbird",
    ];

    window.is_app(TEXT_INPUT_APPS)
}

/// 获取窗口的详细信息用于调试
///
/// # Arguments
///
/// * `window` - 窗口信息
pub fn format_window_info(window: &WindowInfo) -> String {
    format!(
        "Window {{ app: \"{}\", title: \"{}\", pid: {}, exec: \"{}\", path: \"{}\" }}",
        window.app_name, window.title, window.process_id, window.exec_name, window.exec_path
    )
}

/// 检查是否有有效的焦点窗口
///
/// 用于快速判断是否可以进行窗口操作
pub fn has_focused_window() -> bool {
    get_focused_window().is_ok()
}

/// 获取焦点窗口的应用名称
///
/// 便捷方法，用于快速获取应用名称
pub fn get_focused_app_name() -> Option<String> {
    get_focused_window().ok().map(|w| w.app_name)
}

/// 获取焦点窗口的标题
///
/// 便捷方法，用于快速获取窗口标题
pub fn get_focused_window_title() -> Option<String> {
    get_focused_window().ok().map(|w| w.title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_info_is_app() {
        let window = WindowInfo {
            app_name: "Visual Studio Code".to_string(),
            title: "test.rs - RaFlow".to_string(),
            process_id: 1234,
            exec_name: "code".to_string(),
            exec_path: "/usr/bin/code".to_string(),
            window_id: 5678,
        };

        assert!(window.is_app(&["code", "vim"]));
        assert!(window.is_app(&["visual", "sublime"]));
        assert!(!window.is_app(&["notepad", "word"]));
    }

    #[test]
    fn test_window_info_is_app_case_insensitive() {
        let window = WindowInfo {
            app_name: "Google Chrome".to_string(),
            title: "GitHub".to_string(),
            process_id: 1234,
            exec_name: "chrome".to_string(),
            exec_path: "/usr/bin/chrome".to_string(),
            window_id: 5678,
        };

        assert!(window.is_app(&["CHROME"]));
        assert!(window.is_app(&["Chrome"]));
        assert!(window.is_app(&["google"]));
    }

    #[test]
    fn test_window_info_title_contains() {
        let window = WindowInfo {
            app_name: "VS Code".to_string(),
            title: "main.rs - RaFlow [Running]".to_string(),
            process_id: 1234,
            exec_name: "code".to_string(),
            exec_path: "/usr/bin/code".to_string(),
            window_id: 5678,
        };

        assert!(window.title_contains("raflow"));
        assert!(window.title_contains("Running"));
        assert!(window.title_contains("MAIN.RS"));
        assert!(!window.title_contains("test"));
    }

    #[test]
    fn test_window_info_clone() {
        let window = WindowInfo {
            app_name: "Test App".to_string(),
            title: "Test Window".to_string(),
            process_id: 1234,
            exec_name: "test".to_string(),
            exec_path: "/usr/bin/test".to_string(),
            window_id: 5678,
        };

        let cloned = window.clone();
        assert_eq!(window, cloned);
    }

    #[test]
    fn test_is_text_input_app() {
        // 测试编辑器
        let vscode = WindowInfo {
            app_name: "Visual Studio Code".to_string(),
            title: "test.rs".to_string(),
            process_id: 1,
            exec_name: "code".to_string(),
            exec_path: "".to_string(),
            window_id: 1,
        };
        assert!(is_text_input_app(&vscode));

        // 测试浏览器
        let chrome = WindowInfo {
            app_name: "Google Chrome".to_string(),
            title: "GitHub".to_string(),
            process_id: 2,
            exec_name: "chrome".to_string(),
            exec_path: "".to_string(),
            window_id: 2,
        };
        assert!(is_text_input_app(&chrome));

        // 测试通讯工具
        let wechat = WindowInfo {
            app_name: "微信".to_string(),
            title: "聊天".to_string(),
            process_id: 3,
            exec_name: "wechat".to_string(),
            exec_path: "".to_string(),
            window_id: 3,
        };
        assert!(is_text_input_app(&wechat));

        // 测试未知应用
        let unknown = WindowInfo {
            app_name: "Some Random App".to_string(),
            title: "Unknown".to_string(),
            process_id: 4,
            exec_name: "unknown".to_string(),
            exec_path: "".to_string(),
            window_id: 4,
        };
        assert!(!is_text_input_app(&unknown));
    }

    #[test]
    fn test_format_window_info() {
        let window = WindowInfo {
            app_name: "Test".to_string(),
            title: "Title".to_string(),
            process_id: 123,
            exec_name: "test".to_string(),
            exec_path: "/usr/bin/test".to_string(),
            window_id: 456,
        };

        let formatted = format_window_info(&window);
        assert!(formatted.contains("Test"));
        assert!(formatted.contains("Title"));
        assert!(formatted.contains("123"));
        assert!(formatted.contains("/usr/bin/test"));
    }
}
