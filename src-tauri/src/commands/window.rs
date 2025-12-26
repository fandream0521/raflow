//! 窗口相关的 Tauri 命令
//!
//! 提供前端调用的窗口管理命令

use tauri::{command, AppHandle};

use crate::tray;

/// 显示悬浮窗
#[command]
pub fn show_overlay(app: AppHandle) -> Result<(), String> {
    tray::show_overlay_window(&app);
    Ok(())
}

/// 隐藏悬浮窗
#[command]
pub fn hide_overlay(app: AppHandle) -> Result<(), String> {
    tray::hide_overlay_window(&app);
    Ok(())
}

/// 切换悬浮窗显示状态
#[command]
pub fn toggle_overlay(app: AppHandle) -> Result<(), String> {
    tray::toggle_overlay_window(&app);
    Ok(())
}

/// 显示设置窗口
#[command]
pub fn show_settings(app: AppHandle) -> Result<(), String> {
    tray::show_settings_window(&app);
    Ok(())
}

/// 隐藏设置窗口
#[command]
pub fn hide_settings(app: AppHandle) -> Result<(), String> {
    tray::hide_settings_window(&app);
    Ok(())
}

#[cfg(test)]
mod tests {
    // 窗口命令测试需要 Tauri 环境
    #[test]
    fn test_command_signatures() {
        // 确保命令签名正确
    }
}
