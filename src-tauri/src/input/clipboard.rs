//! 剪贴板操作模块
//!
//! 提供剪贴板读写功能，支持保存和恢复剪贴板内容
//!
//! # 功能
//!
//! - 读取剪贴板文本
//! - 写入文本到剪贴板
//! - 保存当前剪贴板内容
//! - 恢复之前保存的内容
//!
//! # 使用示例
//!
//! ```ignore
//! use raflow_lib::input::clipboard::ClipboardManager;
//!
//! // 创建剪贴板管理器
//! let mut clipboard = ClipboardManager::new(&app_handle);
//!
//! // 保存当前剪贴板内容
//! clipboard.save()?;
//!
//! // 写入新内容
//! clipboard.write("Hello, World!")?;
//!
//! // 执行粘贴操作...
//!
//! // 恢复原来的剪贴板内容
//! clipboard.restore()?;
//! ```
//!
//! # 注意事项
//!
//! - 剪贴板操作需要 Tauri clipboard-manager 插件
//! - 保存/恢复功能用于避免覆盖用户原有的剪贴板内容
//! - 某些应用可能对快速剪贴板操作有限制

use super::error::{InputError, InputResult};
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

/// 剪贴板管理器
///
/// 封装 Tauri 剪贴板插件，提供文本读写和内容保存/恢复功能
pub struct ClipboardManager<'a> {
    /// Tauri 应用句柄
    app: &'a AppHandle,
    /// 保存的剪贴板内容
    saved_content: Option<String>,
}

impl<'a> ClipboardManager<'a> {
    /// 创建新的剪贴板管理器
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri 应用句柄
    ///
    /// # Example
    ///
    /// ```ignore
    /// let clipboard = ClipboardManager::new(&app_handle);
    /// ```
    pub fn new(app: &'a AppHandle) -> Self {
        Self {
            app,
            saved_content: None,
        }
    }

    /// 保存当前剪贴板内容
    ///
    /// 将当前剪贴板中的文本保存到内部缓冲区，以便后续恢复
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`，即使剪贴板为空或无法读取
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut clipboard = ClipboardManager::new(&app_handle);
    /// clipboard.save()?;
    /// // 现在可以安全地写入新内容
    /// ```
    pub fn save(&mut self) -> InputResult<()> {
        // 尝试读取当前剪贴板内容
        // 如果读取失败（例如剪贴板为空或包含非文本内容），保存为 None
        self.saved_content = self.app.clipboard().read_text().ok();

        tracing::debug!(
            has_content = self.saved_content.is_some(),
            content_len = self.saved_content.as_ref().map(|s| s.len()).unwrap_or(0),
            "Saved clipboard content"
        );

        Ok(())
    }

    /// 写入文本到剪贴板
    ///
    /// # Arguments
    ///
    /// * `text` - 要写入的文本
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    ///
    /// # Errors
    ///
    /// - `InputError::ClipboardFailed` - 写入失败
    ///
    /// # Example
    ///
    /// ```ignore
    /// let clipboard = ClipboardManager::new(&app_handle);
    /// clipboard.write("Hello, World!")?;
    /// ```
    pub fn write(&self, text: &str) -> InputResult<()> {
        tracing::debug!(text_len = text.len(), "Writing to clipboard");

        self.app
            .clipboard()
            .write_text(text)
            .map_err(|e| InputError::ClipboardFailed(format!("Failed to write: {}", e)))?;

        tracing::debug!("Clipboard write successful");

        Ok(())
    }

    /// 读取剪贴板文本
    ///
    /// # Returns
    ///
    /// 返回剪贴板中的文本，如果剪贴板为空或无法读取则返回 None
    ///
    /// # Example
    ///
    /// ```ignore
    /// let clipboard = ClipboardManager::new(&app_handle);
    /// if let Some(text) = clipboard.read() {
    ///     println!("Clipboard content: {}", text);
    /// }
    /// ```
    pub fn read(&self) -> Option<String> {
        let result = self.app.clipboard().read_text().ok();

        tracing::debug!(
            has_content = result.is_some(),
            content_len = result.as_ref().map(|s| s.len()).unwrap_or(0),
            "Read clipboard content"
        );

        result
    }

    /// 恢复之前保存的剪贴板内容
    ///
    /// 将之前通过 `save()` 保存的内容写回剪贴板
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`，如果之前没有保存内容则什么都不做
    ///
    /// # Errors
    ///
    /// - `InputError::ClipboardFailed` - 恢复失败
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut clipboard = ClipboardManager::new(&app_handle);
    /// clipboard.save()?;
    /// clipboard.write("temporary text")?;
    /// // 执行粘贴...
    /// clipboard.restore()?; // 恢复原来的内容
    /// ```
    pub fn restore(&self) -> InputResult<()> {
        if let Some(content) = &self.saved_content {
            tracing::debug!(content_len = content.len(), "Restoring clipboard content");
            self.write(content)?;
            tracing::debug!("Clipboard content restored");
        } else {
            tracing::debug!("No saved content to restore");
        }

        Ok(())
    }

    /// 检查是否有保存的内容
    ///
    /// # Returns
    ///
    /// 如果之前调用过 `save()` 并且剪贴板有内容则返回 `true`
    pub fn has_saved_content(&self) -> bool {
        self.saved_content.is_some()
    }

    /// 获取保存的内容（如果有）
    ///
    /// # Returns
    ///
    /// 返回之前保存的剪贴板内容的引用
    pub fn get_saved_content(&self) -> Option<&str> {
        self.saved_content.as_deref()
    }

    /// 清除保存的内容
    ///
    /// 清除内部缓冲区中保存的剪贴板内容
    pub fn clear_saved(&mut self) {
        self.saved_content = None;
        tracing::debug!("Cleared saved clipboard content");
    }

    /// 清空剪贴板
    ///
    /// 将剪贴板内容清空（写入空字符串）
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    ///
    /// # Errors
    ///
    /// - `InputError::ClipboardFailed` - 清空失败
    pub fn clear(&self) -> InputResult<()> {
        tracing::debug!("Clearing clipboard");
        self.write("")
    }
}

/// 便捷函数：写入文本到剪贴板
///
/// # Arguments
///
/// * `app` - Tauri 应用句柄
/// * `text` - 要写入的文本
///
/// # Returns
///
/// 成功返回 `Ok(())`
///
/// # Errors
///
/// - `InputError::ClipboardFailed` - 写入失败
pub fn write_to_clipboard(app: &AppHandle, text: &str) -> InputResult<()> {
    let manager = ClipboardManager::new(app);
    manager.write(text)
}

/// 便捷函数：读取剪贴板文本
///
/// # Arguments
///
/// * `app` - Tauri 应用句柄
///
/// # Returns
///
/// 返回剪贴板中的文本，如果无法读取则返回 None
pub fn read_from_clipboard(app: &AppHandle) -> Option<String> {
    let manager = ClipboardManager::new(app);
    manager.read()
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：由于这些测试需要 Tauri AppHandle，
    // 大多数测试在集成测试中进行

    #[test]
    fn test_clipboard_error_display() {
        let error = InputError::ClipboardFailed("test error".to_string());
        assert!(error.to_string().contains("test error"));
    }

    #[test]
    fn test_clipboard_error_equality() {
        let error1 = InputError::ClipboardFailed("error".to_string());
        let error2 = InputError::ClipboardFailed("error".to_string());
        assert_eq!(error1, error2);

        let error3 = InputError::ClipboardFailed("other".to_string());
        assert_ne!(error1, error3);
    }
}
