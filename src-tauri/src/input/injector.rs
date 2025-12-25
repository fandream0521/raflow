//! 文本注入器模块
//!
//! 提供统一的文本注入接口，支持多种注入策略
//!
//! # 功能
//!
//! - 自动策略：根据文本长度自动选择最佳注入方式
//! - 键盘模拟：逐字符模拟键盘输入
//! - 剪贴板粘贴：通过剪贴板快速注入长文本
//! - 仅复制：只复制到剪贴板，不执行粘贴
//!
//! # 使用示例
//!
//! ```ignore
//! use raflow_lib::input::{TextInjector, InjectionStrategy};
//!
//! // 创建注入器（自动策略）
//! let mut injector = TextInjector::new(&app_handle, InjectionStrategy::Auto)?;
//!
//! // 注入文本
//! injector.inject("Hello, World!").await?;
//! ```
//!
//! # 策略说明
//!
//! | 策略 | 适用场景 | 优点 | 缺点 |
//! |------|----------|------|------|
//! | Auto | 通用场景 | 自动优化 | - |
//! | Keyboard | 短文本、密码框 | 兼容性好 | 速度慢 |
//! | Clipboard | 长文本 | 速度快 | 可能覆盖剪贴板 |
//! | ClipboardOnly | 手动粘贴 | 不干扰焦点 | 需要手动粘贴 |

use super::clipboard::ClipboardManager;
use super::error::InputResult;
use super::keyboard::KeyboardSimulator;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::AppHandle;

/// 文本注入策略
///
/// 定义如何将转写文本注入到目标应用
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InjectionStrategy {
    /// 自动选择策略
    ///
    /// - 短文本（< 20 字符）：使用键盘模拟
    /// - 长文本（>= 20 字符）：使用剪贴板粘贴
    #[default]
    Auto,

    /// 始终使用键盘模拟
    ///
    /// 适用于：
    /// - 密码输入框（可能禁用粘贴）
    /// - 需要触发输入事件的场景
    /// - 短文本输入
    Keyboard,

    /// 始终使用剪贴板粘贴
    ///
    /// 适用于：
    /// - 长文本输入
    /// - 需要快速输入的场景
    ///
    /// 注意：会临时修改剪贴板内容，完成后自动恢复
    Clipboard,

    /// 仅复制到剪贴板
    ///
    /// 适用于：
    /// - 用户希望手动控制粘贴时机
    /// - 目标应用不支持自动输入
    ///
    /// 注意：不会自动执行粘贴操作
    ClipboardOnly,
}

impl InjectionStrategy {
    /// 获取策略的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Auto => "自动",
            Self::Keyboard => "键盘模拟",
            Self::Clipboard => "剪贴板粘贴",
            Self::ClipboardOnly => "仅复制",
        }
    }

    /// 获取策略的描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Auto => "根据文本长度自动选择最佳方式",
            Self::Keyboard => "逐字符模拟键盘输入，兼容性好但速度较慢",
            Self::Clipboard => "通过剪贴板粘贴，速度快但会临时占用剪贴板",
            Self::ClipboardOnly => "只复制到剪贴板，需要手动粘贴",
        }
    }
}

/// 自动策略的文本长度阈值
///
/// 小于此长度使用键盘模拟，大于等于此长度使用剪贴板
pub const AUTO_STRATEGY_THRESHOLD: usize = 20;

/// 粘贴操作后的等待时间（毫秒）
///
/// 等待目标应用处理粘贴内容
pub const PASTE_DELAY_MS: u64 = 100;

/// 文本注入器
///
/// 统一的文本注入接口，根据配置的策略选择合适的注入方式
pub struct TextInjector<'a> {
    /// Tauri 应用句柄
    app: &'a AppHandle,
    /// 注入策略
    strategy: InjectionStrategy,
    /// 键盘模拟器
    keyboard: KeyboardSimulator,
    /// 自动策略阈值（可自定义）
    auto_threshold: usize,
    /// 粘贴延迟（可自定义）
    paste_delay: Duration,
}

impl<'a> TextInjector<'a> {
    /// 创建新的文本注入器
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri 应用句柄
    /// * `strategy` - 注入策略
    ///
    /// # Returns
    ///
    /// 返回文本注入器实例
    ///
    /// # Errors
    ///
    /// - `InputError::KeyboardSimulationFailed` - 键盘模拟器初始化失败
    ///
    /// # Example
    ///
    /// ```ignore
    /// let injector = TextInjector::new(&app_handle, InjectionStrategy::Auto)?;
    /// ```
    pub fn new(app: &'a AppHandle, strategy: InjectionStrategy) -> InputResult<Self> {
        let keyboard = KeyboardSimulator::new()?;

        tracing::debug!(
            strategy = ?strategy,
            "Created text injector"
        );

        Ok(Self {
            app,
            strategy,
            keyboard,
            auto_threshold: AUTO_STRATEGY_THRESHOLD,
            paste_delay: Duration::from_millis(PASTE_DELAY_MS),
        })
    }

    /// 创建带自定义配置的文本注入器
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri 应用句柄
    /// * `strategy` - 注入策略
    /// * `auto_threshold` - 自动策略阈值
    /// * `paste_delay_ms` - 粘贴延迟（毫秒）
    pub fn with_config(
        app: &'a AppHandle,
        strategy: InjectionStrategy,
        auto_threshold: usize,
        paste_delay_ms: u64,
    ) -> InputResult<Self> {
        let keyboard = KeyboardSimulator::new()?;

        Ok(Self {
            app,
            strategy,
            keyboard,
            auto_threshold,
            paste_delay: Duration::from_millis(paste_delay_ms),
        })
    }

    /// 注入文本到当前焦点应用
    ///
    /// 根据配置的策略选择合适的注入方式
    ///
    /// # Arguments
    ///
    /// * `text` - 要注入的文本
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    ///
    /// # Errors
    ///
    /// - `InputError::KeyboardSimulationFailed` - 键盘模拟失败
    /// - `InputError::ClipboardFailed` - 剪贴板操作失败
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut injector = TextInjector::new(&app_handle, InjectionStrategy::Auto)?;
    /// injector.inject("Hello, World!").await?;
    /// ```
    pub async fn inject(&mut self, text: &str) -> InputResult<()> {
        if text.is_empty() {
            tracing::debug!("Empty text, skipping injection");
            return Ok(());
        }

        tracing::info!(
            strategy = ?self.strategy,
            text_len = text.len(),
            "Injecting text"
        );

        let result = match self.strategy {
            InjectionStrategy::Auto => {
                if text.chars().count() < self.auto_threshold {
                    tracing::debug!("Auto strategy: using keyboard (short text)");
                    self.inject_via_keyboard(text)
                } else {
                    tracing::debug!("Auto strategy: using clipboard (long text)");
                    self.inject_via_clipboard(text).await
                }
            }
            InjectionStrategy::Keyboard => self.inject_via_keyboard(text),
            InjectionStrategy::Clipboard => self.inject_via_clipboard(text).await,
            InjectionStrategy::ClipboardOnly => {
                self.copy_to_clipboard(text)?;
                tracing::info!("Text copied to clipboard (ClipboardOnly mode)");
                Ok(())
            }
        };

        match &result {
            Ok(()) => tracing::info!("Text injection successful"),
            Err(e) => tracing::error!(error = %e, "Text injection failed"),
        }

        result
    }

    /// 通过键盘模拟注入文本
    ///
    /// 逐字符模拟键盘输入
    ///
    /// # Arguments
    ///
    /// * `text` - 要注入的文本
    fn inject_via_keyboard(&mut self, text: &str) -> InputResult<()> {
        tracing::debug!(text_len = text.len(), "Injecting via keyboard");
        self.keyboard.type_text(text)
    }

    /// 通过剪贴板注入文本
    ///
    /// 保存当前剪贴板 -> 写入文本 -> 粘贴 -> 恢复剪贴板
    ///
    /// # Arguments
    ///
    /// * `text` - 要注入的文本
    async fn inject_via_clipboard(&mut self, text: &str) -> InputResult<()> {
        tracing::debug!(text_len = text.len(), "Injecting via clipboard");

        let mut clipboard = ClipboardManager::new(self.app);

        // 保存当前剪贴板内容
        clipboard.save()?;

        // 写入新内容
        clipboard.write(text)?;

        // 模拟粘贴
        self.keyboard.paste()?;

        // 等待粘贴完成
        tokio::time::sleep(self.paste_delay).await;

        // 恢复剪贴板
        clipboard.restore()?;

        tracing::debug!("Clipboard injection completed");

        Ok(())
    }

    /// 仅复制文本到剪贴板
    ///
    /// 不执行粘贴操作
    ///
    /// # Arguments
    ///
    /// * `text` - 要复制的文本
    fn copy_to_clipboard(&self, text: &str) -> InputResult<()> {
        tracing::debug!(text_len = text.len(), "Copying to clipboard only");

        let clipboard = ClipboardManager::new(self.app);
        clipboard.write(text)
    }

    /// 获取当前策略
    pub fn strategy(&self) -> InjectionStrategy {
        self.strategy
    }

    /// 设置注入策略
    pub fn set_strategy(&mut self, strategy: InjectionStrategy) {
        tracing::debug!(old = ?self.strategy, new = ?strategy, "Changing injection strategy");
        self.strategy = strategy;
    }

    /// 获取自动策略阈值
    pub fn auto_threshold(&self) -> usize {
        self.auto_threshold
    }

    /// 设置自动策略阈值
    pub fn set_auto_threshold(&mut self, threshold: usize) {
        self.auto_threshold = threshold;
    }

    /// 获取粘贴延迟
    pub fn paste_delay(&self) -> Duration {
        self.paste_delay
    }

    /// 设置粘贴延迟
    pub fn set_paste_delay(&mut self, delay: Duration) {
        self.paste_delay = delay;
    }
}

/// 注入结果
///
/// 包含注入操作的详细信息
#[derive(Debug, Clone)]
pub struct InjectionResult {
    /// 使用的策略
    pub strategy_used: InjectionStrategy,
    /// 注入的文本长度
    pub text_length: usize,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果失败）
    pub error_message: Option<String>,
}

impl InjectionResult {
    /// 创建成功结果
    pub fn success(strategy: InjectionStrategy, text_length: usize) -> Self {
        Self {
            strategy_used: strategy,
            text_length,
            success: true,
            error_message: None,
        }
    }

    /// 创建失败结果
    pub fn failure(strategy: InjectionStrategy, text_length: usize, error: &str) -> Self {
        Self {
            strategy_used: strategy,
            text_length,
            success: false,
            error_message: Some(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_injection_strategy_default() {
        let strategy = InjectionStrategy::default();
        assert_eq!(strategy, InjectionStrategy::Auto);
    }

    #[test]
    fn test_injection_strategy_display_name() {
        assert_eq!(InjectionStrategy::Auto.display_name(), "自动");
        assert_eq!(InjectionStrategy::Keyboard.display_name(), "键盘模拟");
        assert_eq!(InjectionStrategy::Clipboard.display_name(), "剪贴板粘贴");
        assert_eq!(InjectionStrategy::ClipboardOnly.display_name(), "仅复制");
    }

    #[test]
    fn test_injection_strategy_description() {
        assert!(!InjectionStrategy::Auto.description().is_empty());
        assert!(!InjectionStrategy::Keyboard.description().is_empty());
        assert!(!InjectionStrategy::Clipboard.description().is_empty());
        assert!(!InjectionStrategy::ClipboardOnly.description().is_empty());
    }

    #[test]
    fn test_injection_strategy_equality() {
        assert_eq!(InjectionStrategy::Auto, InjectionStrategy::Auto);
        assert_ne!(InjectionStrategy::Auto, InjectionStrategy::Keyboard);
    }

    #[test]
    fn test_injection_strategy_clone() {
        let strategy = InjectionStrategy::Clipboard;
        let cloned = strategy.clone();
        assert_eq!(strategy, cloned);
    }

    #[test]
    fn test_injection_strategy_serialization() {
        let strategy = InjectionStrategy::Auto;
        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: InjectionStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(strategy, deserialized);
    }

    #[test]
    fn test_injection_result_success() {
        let result = InjectionResult::success(InjectionStrategy::Keyboard, 10);
        assert!(result.success);
        assert_eq!(result.strategy_used, InjectionStrategy::Keyboard);
        assert_eq!(result.text_length, 10);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_injection_result_failure() {
        let result = InjectionResult::failure(InjectionStrategy::Clipboard, 100, "test error");
        assert!(!result.success);
        assert_eq!(result.strategy_used, InjectionStrategy::Clipboard);
        assert_eq!(result.text_length, 100);
        assert_eq!(result.error_message, Some("test error".to_string()));
    }

    #[test]
    fn test_auto_strategy_threshold_constant() {
        assert_eq!(AUTO_STRATEGY_THRESHOLD, 20);
    }

    #[test]
    fn test_paste_delay_constant() {
        assert_eq!(PASTE_DELAY_MS, 100);
    }
}
