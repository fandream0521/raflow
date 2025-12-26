//! 系统托盘模块
//!
//! 提供系统托盘图标和菜单功能
//!
//! # 功能
//!
//! - 托盘图标显示
//! - 右键菜单（设置、退出）
//! - 左键点击显示主窗口
//! - 状态图标更新

use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime, Wry,
};
use thiserror::Error;

/// 托盘错误类型
#[derive(Error, Debug)]
pub enum TrayError {
    /// Tauri 错误
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    /// 菜单创建失败
    #[error("Failed to create menu: {0}")]
    MenuCreation(String),

    /// 托盘图标创建失败
    #[error("Failed to create tray icon: {0}")]
    TrayCreation(String),

    /// 窗口未找到
    #[error("Window not found: {0}")]
    WindowNotFound(String),

    /// 图标加载失败
    #[error("Failed to load icon: {0}")]
    IconLoad(String),
}

/// 托盘结果类型
pub type TrayResult<T> = Result<T, TrayError>;

/// 托盘菜单项 ID
pub mod menu_ids {
    pub const SHOW_SETTINGS: &str = "show_settings";
    pub const TOGGLE_OVERLAY: &str = "toggle_overlay";
    pub const SEPARATOR: &str = "separator";
    pub const QUIT: &str = "quit";
}

/// 设置系统托盘
///
/// 创建托盘图标和菜单，注册事件处理器
///
/// # Arguments
///
/// * `app` - Tauri 应用句柄
///
/// # Returns
///
/// 成功返回 `Ok(())`，失败返回错误
///
/// # Example
///
/// ```ignore
/// use raflow_lib::tray::setup_tray;
///
/// // 在 Tauri setup 中调用
/// tauri::Builder::default()
///     .setup(|app| {
///         setup_tray(app.handle())?;
///         Ok(())
///     })
///     .run(tauri::generate_context!())
///     .expect("error while running tauri application");
/// ```
pub fn setup_tray(app: &AppHandle<Wry>) -> TrayResult<()> {
    tracing::info!("Setting up system tray");

    // 创建菜单项
    let show_settings = MenuItem::with_id(
        app,
        menu_ids::SHOW_SETTINGS,
        "Settings...",
        true,
        None::<&str>,
    )
    .map_err(|e| TrayError::MenuCreation(e.to_string()))?;

    let toggle_overlay = MenuItem::with_id(
        app,
        menu_ids::TOGGLE_OVERLAY,
        "Toggle Overlay",
        true,
        None::<&str>,
    )
    .map_err(|e| TrayError::MenuCreation(e.to_string()))?;

    let separator = PredefinedMenuItem::separator(app)
        .map_err(|e| TrayError::MenuCreation(e.to_string()))?;

    let quit =
        MenuItem::with_id(app, menu_ids::QUIT, "Quit RaFlow", true, None::<&str>)
            .map_err(|e| TrayError::MenuCreation(e.to_string()))?;

    // 创建菜单
    let menu = Menu::with_items(app, &[&show_settings, &toggle_overlay, &separator, &quit])
        .map_err(|e| TrayError::MenuCreation(e.to_string()))?;

    // 获取图标
    let icon = get_tray_icon(app)?;

    // 创建托盘图标
    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("RaFlow - Real-time Speech-to-Text")
        .on_menu_event(move |app, event| {
            handle_menu_event(app, event.id.as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            handle_tray_event(tray.app_handle(), event);
        })
        .build(app)
        .map_err(|e| TrayError::TrayCreation(e.to_string()))?;

    tracing::info!("System tray setup complete");
    Ok(())
}

/// 处理菜单事件
fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, menu_id: &str) {
    tracing::debug!(menu_id = %menu_id, "Tray menu event");

    match menu_id {
        menu_ids::SHOW_SETTINGS => {
            show_settings_window(app);
        }
        menu_ids::TOGGLE_OVERLAY => {
            toggle_overlay_window(app);
        }
        menu_ids::QUIT => {
            tracing::info!("User requested quit from tray menu");
            app.exit(0);
        }
        _ => {
            tracing::warn!(menu_id = %menu_id, "Unknown menu event");
        }
    }
}

/// 处理托盘图标事件
fn handle_tray_event<R: Runtime>(app: &AppHandle<R>, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } => {
            tracing::debug!("Tray left click");
            show_settings_window(app);
        }
        TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        } => {
            tracing::debug!("Tray double click");
            show_settings_window(app);
        }
        _ => {}
    }
}

/// 显示设置窗口
pub fn show_settings_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        tracing::debug!("Showing settings window");

        // 如果窗口是隐藏的，显示它
        if window.is_visible().unwrap_or(false) {
            let _ = window.set_focus();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    } else {
        tracing::warn!("Main window not found");
    }
}

/// 隐藏设置窗口
pub fn hide_settings_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        tracing::debug!("Hiding settings window");
        let _ = window.hide();
    }
}

/// 切换 Overlay 窗口显示状态
pub fn toggle_overlay_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("overlay") {
        let is_visible = window.is_visible().unwrap_or(false);
        tracing::debug!(is_visible = %is_visible, "Toggling overlay window");

        if is_visible {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.center();
        }
    } else {
        tracing::warn!("Overlay window not found");
    }
}

/// 显示 Overlay 窗口
pub fn show_overlay_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("overlay") {
        tracing::debug!("Showing overlay window");
        let _ = window.show();
        let _ = window.center();
    }
}

/// 隐藏 Overlay 窗口
pub fn hide_overlay_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("overlay") {
        tracing::debug!("Hiding overlay window");
        let _ = window.hide();
    }
}

/// 获取托盘图标
fn get_tray_icon(app: &AppHandle<Wry>) -> TrayResult<Image<'static>> {
    // 尝试使用默认窗口图标
    match app.default_window_icon() {
        Some(icon) => {
            // 获取 RGBA 数据并创建新的拥有所有权的 Image
            let rgba = icon.rgba().to_vec();
            let width = icon.width();
            let height = icon.height();
            Ok(Image::new_owned(rgba, width, height))
        }
        None => Err(TrayError::IconLoad("No default icon available".to_string())),
    }
}

/// 更新托盘图标状态
///
/// 根据应用状态更新托盘图标（预留功能）
///
/// # Arguments
///
/// * `app` - Tauri 应用句柄
/// * `status` - 状态名称
#[allow(dead_code)]
pub fn update_tray_status<R: Runtime>(_app: &AppHandle<R>, status: &str) {
    tracing::debug!(status = %status, "Updating tray status");
    // 预留：未来可以根据状态更新托盘图标
    // 例如：录音中显示红色图标，空闲时显示灰色图标
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_error_display() {
        let err = TrayError::MenuCreation("test error".to_string());
        assert!(err.to_string().contains("test error"));

        let err = TrayError::TrayCreation("creation failed".to_string());
        assert!(err.to_string().contains("creation failed"));

        let err = TrayError::WindowNotFound("main".to_string());
        assert!(err.to_string().contains("main"));

        let err = TrayError::IconLoad("icon not found".to_string());
        assert!(err.to_string().contains("icon not found"));
    }

    #[test]
    fn test_menu_ids() {
        assert_eq!(menu_ids::SHOW_SETTINGS, "show_settings");
        assert_eq!(menu_ids::TOGGLE_OVERLAY, "toggle_overlay");
        assert_eq!(menu_ids::QUIT, "quit");
    }
}
