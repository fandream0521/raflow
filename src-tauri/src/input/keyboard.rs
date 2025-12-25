//! 键盘模拟模块
//!
//! 提供键盘输入模拟功能，用于将转写文本注入到目标应用
//!
//! # 功能
//!
//! - 文本输入：逐字符模拟键盘输入
//! - 粘贴操作：模拟 Ctrl+V (Windows/Linux) 或 Cmd+V (macOS)
//! - 按键组合：支持自定义按键组合
//!
//! # 使用示例
//!
//! ```ignore
//! use raflow_lib::input::keyboard::KeyboardSimulator;
//!
//! // 创建键盘模拟器
//! let mut keyboard = KeyboardSimulator::new()?;
//!
//! // 输入文本
//! keyboard.type_text("Hello, World!")?;
//!
//! // 模拟粘贴
//! keyboard.paste()?;
//! ```
//!
//! # 平台支持
//!
//! | 平台 | 文本输入 | 粘贴操作 | 备注 |
//! |------|----------|----------|------|
//! | Windows | ✅ | ✅ | 无需特殊权限 |
//! | macOS | ✅ | ✅ | 需要辅助功能权限 |
//! | Linux (X11) | ✅ | ✅ | 需要 X11 |
//! | Linux (Wayland) | ⚠️ | ⚠️ | 受限支持 |

use super::error::{InputError, InputResult};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

/// 键盘模拟器
///
/// 封装 enigo 库，提供跨平台的键盘模拟功能
pub struct KeyboardSimulator {
    /// enigo 实例
    enigo: Enigo,
}

impl KeyboardSimulator {
    /// 创建新的键盘模拟器
    ///
    /// # Returns
    ///
    /// 返回键盘模拟器实例
    ///
    /// # Errors
    ///
    /// - `InputError::KeyboardSimulationFailed` - 初始化失败
    ///
    /// # Example
    ///
    /// ```ignore
    /// let keyboard = KeyboardSimulator::new()?;
    /// ```
    pub fn new() -> InputResult<Self> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to initialize: {}", e)))?;

        tracing::debug!("Keyboard simulator initialized");

        Ok(Self { enigo })
    }

    /// 输入文本
    ///
    /// 使用键盘模拟逐字符输入文本
    ///
    /// # Arguments
    ///
    /// * `text` - 要输入的文本
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    ///
    /// # Errors
    ///
    /// - `InputError::KeyboardSimulationFailed` - 输入失败
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut keyboard = KeyboardSimulator::new()?;
    /// keyboard.type_text("Hello, World!")?;
    /// ```
    ///
    /// # 注意
    ///
    /// - 短文本（< 20 字符）推荐使用此方法
    /// - 长文本建议使用剪贴板粘贴以提高速度
    /// - 输入过程中用户移动鼠标可能导致输入位置改变
    pub fn type_text(&mut self, text: &str) -> InputResult<()> {
        if text.is_empty() {
            return Ok(());
        }

        tracing::debug!(text_len = text.len(), "Typing text");

        self.enigo
            .text(text)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to type text: {}", e)))?;

        tracing::debug!("Text typed successfully");

        Ok(())
    }

    /// 模拟粘贴操作
    ///
    /// 根据平台发送相应的粘贴快捷键：
    /// - Windows/Linux: Ctrl+V
    /// - macOS: Cmd+V
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    ///
    /// # Errors
    ///
    /// - `InputError::KeyboardSimulationFailed` - 粘贴失败
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut keyboard = KeyboardSimulator::new()?;
    /// // 先将文本复制到剪贴板
    /// keyboard.paste()?;
    /// ```
    pub fn paste(&mut self) -> InputResult<()> {
        tracing::debug!("Simulating paste operation");

        #[cfg(target_os = "macos")]
        {
            self.paste_macos()?;
        }

        #[cfg(target_os = "windows")]
        {
            self.paste_windows()?;
        }

        #[cfg(target_os = "linux")]
        {
            self.paste_linux()?;
        }

        tracing::debug!("Paste operation completed");

        Ok(())
    }

    /// macOS 粘贴实现 (Cmd+V)
    #[cfg(target_os = "macos")]
    fn paste_macos(&mut self) -> InputResult<()> {
        self.enigo
            .key(Key::Meta, Direction::Press)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to press Meta: {}", e)))?;

        self.enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to click 'v': {}", e)))?;

        self.enigo
            .key(Key::Meta, Direction::Release)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to release Meta: {}", e)))?;

        Ok(())
    }

    /// Windows 粘贴实现 (Ctrl+V)
    #[cfg(target_os = "windows")]
    fn paste_windows(&mut self) -> InputResult<()> {
        self.enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to press Control: {}", e)))?;

        self.enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to click 'v': {}", e)))?;

        self.enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to release Control: {}", e)))?;

        Ok(())
    }

    /// Linux 粘贴实现 (Ctrl+V)
    #[cfg(target_os = "linux")]
    fn paste_linux(&mut self) -> InputResult<()> {
        self.enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to press Control: {}", e)))?;

        self.enigo
            .key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to click 'v': {}", e)))?;

        self.enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to release Control: {}", e)))?;

        Ok(())
    }

    /// 模拟复制操作
    ///
    /// 根据平台发送相应的复制快捷键：
    /// - Windows/Linux: Ctrl+C
    /// - macOS: Cmd+C
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    ///
    /// # Errors
    ///
    /// - `InputError::KeyboardSimulationFailed` - 复制失败
    pub fn copy(&mut self) -> InputResult<()> {
        tracing::debug!("Simulating copy operation");

        #[cfg(target_os = "macos")]
        {
            self.copy_macos()?;
        }

        #[cfg(target_os = "windows")]
        {
            self.copy_windows()?;
        }

        #[cfg(target_os = "linux")]
        {
            self.copy_linux()?;
        }

        tracing::debug!("Copy operation completed");

        Ok(())
    }

    /// macOS 复制实现 (Cmd+C)
    #[cfg(target_os = "macos")]
    fn copy_macos(&mut self) -> InputResult<()> {
        self.enigo
            .key(Key::Meta, Direction::Press)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to press Meta: {}", e)))?;

        self.enigo
            .key(Key::Unicode('c'), Direction::Click)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to click 'c': {}", e)))?;

        self.enigo
            .key(Key::Meta, Direction::Release)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to release Meta: {}", e)))?;

        Ok(())
    }

    /// Windows 复制实现 (Ctrl+C)
    #[cfg(target_os = "windows")]
    fn copy_windows(&mut self) -> InputResult<()> {
        self.enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to press Control: {}", e)))?;

        self.enigo
            .key(Key::Unicode('c'), Direction::Click)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to click 'c': {}", e)))?;

        self.enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to release Control: {}", e)))?;

        Ok(())
    }

    /// Linux 复制实现 (Ctrl+C)
    #[cfg(target_os = "linux")]
    fn copy_linux(&mut self) -> InputResult<()> {
        self.enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to press Control: {}", e)))?;

        self.enigo
            .key(Key::Unicode('c'), Direction::Click)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to click 'c': {}", e)))?;

        self.enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to release Control: {}", e)))?;

        Ok(())
    }

    /// 模拟全选操作
    ///
    /// 根据平台发送相应的全选快捷键：
    /// - Windows/Linux: Ctrl+A
    /// - macOS: Cmd+A
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    ///
    /// # Errors
    ///
    /// - `InputError::KeyboardSimulationFailed` - 全选失败
    pub fn select_all(&mut self) -> InputResult<()> {
        tracing::debug!("Simulating select all operation");

        #[cfg(target_os = "macos")]
        {
            self.select_all_macos()?;
        }

        #[cfg(target_os = "windows")]
        {
            self.select_all_windows()?;
        }

        #[cfg(target_os = "linux")]
        {
            self.select_all_linux()?;
        }

        tracing::debug!("Select all operation completed");

        Ok(())
    }

    /// macOS 全选实现 (Cmd+A)
    #[cfg(target_os = "macos")]
    fn select_all_macos(&mut self) -> InputResult<()> {
        self.enigo
            .key(Key::Meta, Direction::Press)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to press Meta: {}", e)))?;

        self.enigo
            .key(Key::Unicode('a'), Direction::Click)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to click 'a': {}", e)))?;

        self.enigo
            .key(Key::Meta, Direction::Release)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to release Meta: {}", e)))?;

        Ok(())
    }

    /// Windows 全选实现 (Ctrl+A)
    #[cfg(target_os = "windows")]
    fn select_all_windows(&mut self) -> InputResult<()> {
        self.enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to press Control: {}", e)))?;

        self.enigo
            .key(Key::Unicode('a'), Direction::Click)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to click 'a': {}", e)))?;

        self.enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to release Control: {}", e)))?;

        Ok(())
    }

    /// Linux 全选实现 (Ctrl+A)
    #[cfg(target_os = "linux")]
    fn select_all_linux(&mut self) -> InputResult<()> {
        self.enigo
            .key(Key::Control, Direction::Press)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to press Control: {}", e)))?;

        self.enigo
            .key(Key::Unicode('a'), Direction::Click)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to click 'a': {}", e)))?;

        self.enigo
            .key(Key::Control, Direction::Release)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to release Control: {}", e)))?;

        Ok(())
    }

    /// 按下单个按键
    ///
    /// # Arguments
    ///
    /// * `key` - 要按下的按键
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    pub fn press_key(&mut self, key: Key) -> InputResult<()> {
        self.enigo
            .key(key, Direction::Press)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to press key: {}", e)))
    }

    /// 释放单个按键
    ///
    /// # Arguments
    ///
    /// * `key` - 要释放的按键
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    pub fn release_key(&mut self, key: Key) -> InputResult<()> {
        self.enigo
            .key(key, Direction::Release)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to release key: {}", e)))
    }

    /// 点击单个按键（按下并释放）
    ///
    /// # Arguments
    ///
    /// * `key` - 要点击的按键
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    pub fn click_key(&mut self, key: Key) -> InputResult<()> {
        self.enigo
            .key(key, Direction::Click)
            .map_err(|e| InputError::KeyboardSimulationFailed(format!("Failed to click key: {}", e)))
    }

    /// 按下 Enter 键
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    pub fn press_enter(&mut self) -> InputResult<()> {
        self.click_key(Key::Return)
    }

    /// 按下 Escape 键
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    pub fn press_escape(&mut self) -> InputResult<()> {
        self.click_key(Key::Escape)
    }

    /// 按下 Tab 键
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    pub fn press_tab(&mut self) -> InputResult<()> {
        self.click_key(Key::Tab)
    }

    /// 按下 Backspace 键
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    pub fn press_backspace(&mut self) -> InputResult<()> {
        self.click_key(Key::Backspace)
    }

    /// 按下 Delete 键
    ///
    /// # Returns
    ///
    /// 成功返回 `Ok(())`
    pub fn press_delete(&mut self) -> InputResult<()> {
        self.click_key(Key::Delete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_simulator_new() {
        // 这个测试在没有窗口系统的环境下可能失败
        // 仅测试不会 panic
        let result = KeyboardSimulator::new();
        // 在 CI 环境中可能没有显示服务器，所以不断言成功
        match result {
            Ok(_) => {
                // 成功创建
            }
            Err(e) => {
                // 在无头环境中可能失败，这是预期的
                assert!(
                    e.to_string().contains("Failed to initialize")
                        || e.to_string().contains("Keyboard simulation failed")
                );
            }
        }
    }

    #[test]
    fn test_type_text_empty() {
        // 测试空文本不会出错
        if let Ok(mut keyboard) = KeyboardSimulator::new() {
            let result = keyboard.type_text("");
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_input_error_keyboard_simulation_failed() {
        let error = InputError::KeyboardSimulationFailed("test error".to_string());
        assert!(error.to_string().contains("test error"));
    }
}
