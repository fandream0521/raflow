//! 热键配置模块
//!
//! 定义热键配置结构和默认值

use serde::{Deserialize, Serialize};

/// 热键配置
///
/// 存储应用程序使用的所有热键设置
///
/// # Examples
///
/// ```
/// use raflow_lib::hotkey::HotkeyConfig;
///
/// // 使用默认配置
/// let config = HotkeyConfig::default();
/// assert!(!config.push_to_talk.is_empty());
///
/// // 自定义配置
/// let config = HotkeyConfig::new("Ctrl+Shift+R", "Escape");
/// assert_eq!(config.push_to_talk, "Ctrl+Shift+R");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// Push-to-Talk 热键
    ///
    /// 按住此键开始录音，松开结束录音
    /// 默认值: "CommandOrControl+Shift+."
    pub push_to_talk: String,

    /// 取消热键
    ///
    /// 按下此键取消当前录音会话
    /// 默认值: "Escape"
    pub cancel: String,

    /// 切换模式热键（可选）
    ///
    /// 用于切换应用程序模式（如静音模式）
    /// 默认值: None
    pub toggle_mode: Option<String>,
}

impl HotkeyConfig {
    /// 创建新的热键配置
    ///
    /// # Arguments
    ///
    /// * `push_to_talk` - Push-to-Talk 热键
    /// * `cancel` - 取消热键
    ///
    /// # Examples
    ///
    /// ```
    /// use raflow_lib::hotkey::HotkeyConfig;
    ///
    /// let config = HotkeyConfig::new("Ctrl+Alt+Space", "Escape");
    /// assert_eq!(config.push_to_talk, "Ctrl+Alt+Space");
    /// assert_eq!(config.cancel, "Escape");
    /// assert!(config.toggle_mode.is_none());
    /// ```
    pub fn new(push_to_talk: impl Into<String>, cancel: impl Into<String>) -> Self {
        Self {
            push_to_talk: push_to_talk.into(),
            cancel: cancel.into(),
            toggle_mode: None,
        }
    }

    /// 设置切换模式热键
    ///
    /// # Examples
    ///
    /// ```
    /// use raflow_lib::hotkey::HotkeyConfig;
    ///
    /// let config = HotkeyConfig::default()
    ///     .with_toggle_mode("CommandOrControl+Shift+/");
    /// assert_eq!(config.toggle_mode, Some("CommandOrControl+Shift+/".to_string()));
    /// ```
    pub fn with_toggle_mode(mut self, hotkey: impl Into<String>) -> Self {
        self.toggle_mode = Some(hotkey.into());
        self
    }

    /// 设置 Push-to-Talk 热键
    pub fn with_push_to_talk(mut self, hotkey: impl Into<String>) -> Self {
        self.push_to_talk = hotkey.into();
        self
    }

    /// 设置取消热键
    pub fn with_cancel(mut self, hotkey: impl Into<String>) -> Self {
        self.cancel = hotkey.into();
        self
    }

    /// 获取所有已配置的热键列表
    ///
    /// # Examples
    ///
    /// ```
    /// use raflow_lib::hotkey::HotkeyConfig;
    ///
    /// let config = HotkeyConfig::default();
    /// let hotkeys = config.all_hotkeys();
    /// assert!(hotkeys.len() >= 2); // push_to_talk 和 cancel
    /// ```
    pub fn all_hotkeys(&self) -> Vec<&str> {
        let mut hotkeys = vec![self.push_to_talk.as_str(), self.cancel.as_str()];
        if let Some(ref toggle) = self.toggle_mode {
            hotkeys.push(toggle.as_str());
        }
        hotkeys
    }

    /// 检查热键是否为 Push-to-Talk 热键
    pub fn is_push_to_talk(&self, hotkey: &str) -> bool {
        self.push_to_talk == hotkey
    }

    /// 检查热键是否为取消热键
    pub fn is_cancel(&self, hotkey: &str) -> bool {
        self.cancel == hotkey
    }

    /// 检查热键是否为切换模式热键
    pub fn is_toggle_mode(&self, hotkey: &str) -> bool {
        self.toggle_mode.as_deref() == Some(hotkey)
    }
}

impl Default for HotkeyConfig {
    /// 创建默认热键配置
    ///
    /// - Push-to-Talk: `CommandOrControl+Shift+.`
    /// - Cancel: `Escape`
    /// - Toggle Mode: None
    fn default() -> Self {
        Self {
            push_to_talk: "CommandOrControl+Shift+.".to_string(),
            cancel: "Escape".to_string(),
            toggle_mode: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = HotkeyConfig::default();
        assert_eq!(config.push_to_talk, "CommandOrControl+Shift+.");
        assert_eq!(config.cancel, "Escape");
        assert!(config.toggle_mode.is_none());
    }

    #[test]
    fn test_config_new() {
        let config = HotkeyConfig::new("Ctrl+Space", "Escape");
        assert_eq!(config.push_to_talk, "Ctrl+Space");
        assert_eq!(config.cancel, "Escape");
        assert!(config.toggle_mode.is_none());
    }

    #[test]
    fn test_config_builder_pattern() {
        let config = HotkeyConfig::default()
            .with_push_to_talk("Alt+R")
            .with_cancel("Ctrl+C")
            .with_toggle_mode("Ctrl+M");

        assert_eq!(config.push_to_talk, "Alt+R");
        assert_eq!(config.cancel, "Ctrl+C");
        assert_eq!(config.toggle_mode, Some("Ctrl+M".to_string()));
    }

    #[test]
    fn test_all_hotkeys() {
        let config = HotkeyConfig::default();
        let hotkeys = config.all_hotkeys();
        assert_eq!(hotkeys.len(), 2);
        assert!(hotkeys.contains(&"CommandOrControl+Shift+."));
        assert!(hotkeys.contains(&"Escape"));

        let config_with_toggle = config.with_toggle_mode("Ctrl+T");
        let hotkeys = config_with_toggle.all_hotkeys();
        assert_eq!(hotkeys.len(), 3);
    }

    #[test]
    fn test_hotkey_identification() {
        let config = HotkeyConfig::default().with_toggle_mode("Ctrl+M");

        assert!(config.is_push_to_talk("CommandOrControl+Shift+."));
        assert!(!config.is_push_to_talk("Escape"));

        assert!(config.is_cancel("Escape"));
        assert!(!config.is_cancel("CommandOrControl+Shift+."));

        assert!(config.is_toggle_mode("Ctrl+M"));
        assert!(!config.is_toggle_mode("Escape"));
    }

    #[test]
    fn test_config_serialization() {
        let config = HotkeyConfig::default().with_toggle_mode("Ctrl+T");
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HotkeyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_equality() {
        let config1 = HotkeyConfig::default();
        let config2 = HotkeyConfig::default();
        assert_eq!(config1, config2);

        let config3 = HotkeyConfig::new("Different", "Escape");
        assert_ne!(config1, config3);
    }
}
