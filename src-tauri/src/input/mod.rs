//! 输入模块
//!
//! 提供文本注入、窗口检测、键盘模拟和剪贴板操作功能
//!
//! # 子模块
//!
//! - [`error`] - 错误类型定义
//! - [`window`] - 窗口检测功能
//! - [`keyboard`] - 键盘模拟功能
//! - [`clipboard`] - 剪贴板操作功能
//! - [`injector`] - 文本注入器（整合键盘和剪贴板）
//!
//! # 功能概述
//!
//! 本模块实现了 RaFlow 的文本注入系统，包括：
//!
//! 1. **窗口检测** - 获取当前焦点窗口信息，判断是否为文本输入上下文
//! 2. **键盘模拟** - 模拟键盘输入，将转写结果注入到目标应用
//! 3. **剪贴板操作** - 通过剪贴板进行文本传输，支持保存和恢复
//! 4. **文本注入器** - 统一接口，自动选择最佳注入方式
//!
//! # 使用示例
//!
//! ## 推荐方式：使用 TextInjector
//!
//! ```ignore
//! use raflow_lib::input::{TextInjector, InjectionStrategy, is_text_input_context};
//!
//! // 检查是否为文本输入环境
//! if is_text_input_context() {
//!     // 创建注入器（自动策略）
//!     let mut injector = TextInjector::new(&app_handle, InjectionStrategy::Auto)?;
//!
//!     // 自动选择最佳方式注入文本
//!     injector.inject("转写文本").await?;
//! }
//! ```
//!
//! ## 底层方式：直接使用组件
//!
//! ```ignore
//! use raflow_lib::input::{KeyboardSimulator, ClipboardManager};
//!
//! // 方式1：直接键盘输入（短文本）
//! let mut keyboard = KeyboardSimulator::new()?;
//! keyboard.type_text("Hello!")?;
//!
//! // 方式2：剪贴板粘贴（长文本）
//! let mut clipboard = ClipboardManager::new(&app_handle);
//! clipboard.save()?;  // 保存原有内容
//! clipboard.write("Long text...")?;
//! keyboard.paste()?;  // 粘贴
//! clipboard.restore()?;  // 恢复原有内容
//! ```
//!
//! # 平台支持
//!
//! | 平台 | 窗口检测 | 键盘模拟 | 剪贴板 |
//! |------|----------|----------|--------|
//! | Windows | ✅ | ✅ | ✅ |
//! | macOS | ✅ * | ✅ * | ✅ |
//! | Linux (X11) | ✅ | ✅ | ✅ |
//! | Linux (Wayland) | ⚠️ | ⚠️ | ✅ |
//!
//! * macOS 需要辅助功能权限

pub mod clipboard;
pub mod error;
pub mod injector;
pub mod keyboard;
pub mod window;

// Re-export commonly used types
pub use clipboard::{read_from_clipboard, write_to_clipboard, ClipboardManager};
pub use error::{InputError, InputResult};
pub use injector::{InjectionResult, InjectionStrategy, TextInjector, AUTO_STRATEGY_THRESHOLD, PASTE_DELAY_MS};
pub use keyboard::KeyboardSimulator;
pub use window::{
    format_window_info, get_focused_app_name, get_focused_window, get_focused_window_title,
    has_focused_window, is_text_input_context, WindowInfo,
};
