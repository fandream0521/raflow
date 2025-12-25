//! 热键注册模块
//!
//! 提供全局热键的注册和管理功能

use std::sync::Arc;

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use super::config::HotkeyConfig;
use super::error::{HotkeyError, HotkeyResult};
use super::handlers;

/// 热键事件类型
#[derive(Debug, Clone, PartialEq)]
pub enum HotkeyEvent {
    /// Push-to-Talk 按下
    PushToTalkPressed,
    /// Push-to-Talk 松开
    PushToTalkReleased,
    /// 取消按下
    CancelPressed,
    /// 切换模式按下
    ToggleModePressed,
}

/// 热键事件处理器类型
pub type HotkeyHandler = Arc<dyn Fn(HotkeyEvent) + Send + Sync>;

/// 热键管理器
///
/// 负责注册和管理全局热键
pub struct HotkeyManager {
    config: HotkeyConfig,
    registered_shortcuts: Vec<String>,
}

impl HotkeyManager {
    /// 创建新的热键管理器
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use raflow_lib::hotkey::{HotkeyManager, HotkeyConfig};
    ///
    /// let config = HotkeyConfig::default();
    /// let manager = HotkeyManager::new(config);
    /// ```
    pub fn new(config: HotkeyConfig) -> Self {
        Self {
            config,
            registered_shortcuts: Vec::new(),
        }
    }

    /// 获取当前配置
    pub fn config(&self) -> &HotkeyConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: HotkeyConfig) {
        self.config = config;
    }

    /// 获取已注册的热键列表
    pub fn registered_shortcuts(&self) -> &[String] {
        &self.registered_shortcuts
    }
}

/// 注册所有热键
///
/// 根据配置注册 Push-to-Talk、取消和切换模式热键
///
/// # Arguments
///
/// * `app` - Tauri 应用句柄
/// * `config` - 热键配置
///
/// # Returns
///
/// 成功返回 `Ok(())`，失败返回 `HotkeyError`
///
/// # Examples
///
/// ```ignore
/// use raflow_lib::hotkey::{register_hotkeys, HotkeyConfig};
///
/// // 在 Tauri setup 中调用
/// tauri::Builder::default()
///     .setup(|app| {
///         let config = HotkeyConfig::default();
///         register_hotkeys(app.handle(), &config)?;
///         Ok(())
///     })
/// ```
pub fn register_hotkeys(app: &AppHandle, config: &HotkeyConfig) -> HotkeyResult<()> {
    let shortcut_manager = app.global_shortcut();

    // 注册 Push-to-Talk 热键
    let ptt_shortcut = parse_shortcut(&config.push_to_talk)?;
    let app_handle = app.clone();

    shortcut_manager
        .on_shortcut(ptt_shortcut, move |app, _shortcut, event| {
            handle_ptt_event(app, &event.state);
        })
        .map_err(|e| HotkeyError::RegistrationFailed {
            hotkey: config.push_to_talk.clone(),
            reason: e.to_string(),
        })?;

    tracing::info!(
        hotkey = %config.push_to_talk,
        "Registered Push-to-Talk hotkey"
    );

    // 注册取消热键
    let cancel_shortcut = parse_shortcut(&config.cancel)?;
    let _app_handle_cancel = app_handle.clone();

    shortcut_manager
        .on_shortcut(cancel_shortcut, move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                handle_cancel(app);
            }
        })
        .map_err(|e| HotkeyError::RegistrationFailed {
            hotkey: config.cancel.clone(),
            reason: e.to_string(),
        })?;

    tracing::info!(
        hotkey = %config.cancel,
        "Registered Cancel hotkey"
    );

    // 注册切换模式热键（如果配置了）
    if let Some(ref toggle_hotkey) = config.toggle_mode {
        let toggle_shortcut = parse_shortcut(toggle_hotkey)?;

        shortcut_manager
            .on_shortcut(toggle_shortcut, move |app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    handle_toggle_mode(app);
                }
            })
            .map_err(|e| HotkeyError::RegistrationFailed {
                hotkey: toggle_hotkey.clone(),
                reason: e.to_string(),
            })?;

        tracing::info!(
            hotkey = %toggle_hotkey,
            "Registered Toggle Mode hotkey"
        );
    }

    tracing::info!("All global hotkeys registered successfully");
    Ok(())
}

/// 注销所有热键
///
/// # Arguments
///
/// * `app` - Tauri 应用句柄
/// * `config` - 热键配置
pub fn unregister_hotkeys(app: &AppHandle, config: &HotkeyConfig) -> HotkeyResult<()> {
    let shortcut_manager = app.global_shortcut();

    // 注销 Push-to-Talk 热键
    let ptt_shortcut = parse_shortcut(&config.push_to_talk)?;
    shortcut_manager
        .unregister(ptt_shortcut)
        .map_err(|e| HotkeyError::UnregistrationFailed {
            hotkey: config.push_to_talk.clone(),
            reason: e.to_string(),
        })?;

    // 注销取消热键
    let cancel_shortcut = parse_shortcut(&config.cancel)?;
    shortcut_manager
        .unregister(cancel_shortcut)
        .map_err(|e| HotkeyError::UnregistrationFailed {
            hotkey: config.cancel.clone(),
            reason: e.to_string(),
        })?;

    // 注销切换模式热键（如果配置了）
    if let Some(ref toggle_hotkey) = config.toggle_mode {
        let toggle_shortcut = parse_shortcut(toggle_hotkey)?;
        shortcut_manager
            .unregister(toggle_shortcut)
            .map_err(|e| HotkeyError::UnregistrationFailed {
                hotkey: toggle_hotkey.clone(),
                reason: e.to_string(),
            })?;
    }

    tracing::info!("All global hotkeys unregistered");
    Ok(())
}

/// 检查热键是否已注册
pub fn is_hotkey_registered(app: &AppHandle, hotkey: &str) -> HotkeyResult<bool> {
    let shortcut = parse_shortcut(hotkey)?;
    let shortcut_manager = app.global_shortcut();
    Ok(shortcut_manager.is_registered(shortcut))
}

/// 解析热键字符串为 Shortcut
fn parse_shortcut(hotkey: &str) -> HotkeyResult<Shortcut> {
    hotkey
        .parse::<Shortcut>()
        .map_err(|_| HotkeyError::InvalidFormat(hotkey.to_string()))
}

/// 处理 Push-to-Talk 事件
fn handle_ptt_event(app: &AppHandle, state: &ShortcutState) {
    match state {
        ShortcutState::Pressed => {
            tracing::info!("Push-to-Talk pressed");
            handlers::handle_ptt_pressed(app);
        }
        ShortcutState::Released => {
            tracing::info!("Push-to-Talk released");
            handlers::handle_ptt_released(app);
        }
    }
}

/// 处理取消事件
fn handle_cancel(app: &AppHandle) {
    tracing::info!("Cancel pressed");
    handlers::handle_cancel(app);
}

/// 处理切换模式事件
fn handle_toggle_mode(app: &AppHandle) {
    tracing::info!("Toggle mode pressed");
    handlers::handle_toggle_mode(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_shortcut_valid() {
        let result = parse_shortcut("CommandOrControl+Shift+.");
        assert!(result.is_ok());

        let result = parse_shortcut("Escape");
        assert!(result.is_ok());

        let result = parse_shortcut("Ctrl+Alt+Delete");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_shortcut_invalid() {
        let result = parse_shortcut("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HotkeyError::InvalidFormat(_)));

        let result = parse_shortcut("InvalidKey");
        assert!(result.is_err());
    }

    #[test]
    fn test_hotkey_manager_creation() {
        let config = HotkeyConfig::default();
        let manager = HotkeyManager::new(config.clone());

        assert_eq!(manager.config(), &config);
        assert!(manager.registered_shortcuts().is_empty());
    }

    #[test]
    fn test_hotkey_manager_update_config() {
        let config1 = HotkeyConfig::default();
        let mut manager = HotkeyManager::new(config1);

        let config2 = HotkeyConfig::new("Ctrl+Space", "Escape");
        manager.update_config(config2.clone());

        assert_eq!(manager.config(), &config2);
    }

    #[test]
    fn test_hotkey_event_equality() {
        assert_eq!(HotkeyEvent::PushToTalkPressed, HotkeyEvent::PushToTalkPressed);
        assert_ne!(HotkeyEvent::PushToTalkPressed, HotkeyEvent::PushToTalkReleased);
        assert_ne!(HotkeyEvent::CancelPressed, HotkeyEvent::ToggleModePressed);
    }
}
