//! 配置相关的 Tauri 命令
//!
//! 提供前端调用的配置管理命令

use std::sync::Arc;

use tauri::{command, AppHandle, Manager};

use crate::state::{AppConfig, ConfigManager, GlobalConfig};

/// 获取当前配置
#[command]
pub fn get_config(app: AppHandle) -> Result<AppConfig, String> {
    let config = app
        .try_state::<Arc<GlobalConfig>>()
        .ok_or("Config not initialized")?;

    Ok((*config.get()).clone())
}

/// 保存配置
#[command]
pub fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    // 保存到文件
    ConfigManager::save(&app, &config).map_err(|e| e.to_string())?;

    // 更新全局配置
    if let Some(global) = app.try_state::<Arc<GlobalConfig>>() {
        global.update(config);
    }

    tracing::info!("Config saved via command");
    Ok(())
}

/// 获取 API 密钥
#[command]
pub fn get_api_key(app: AppHandle) -> Result<String, String> {
    let config = app
        .try_state::<Arc<GlobalConfig>>()
        .ok_or("Config not initialized")?;

    Ok(config.api_key())
}

/// 设置 API 密钥
#[command]
pub fn set_api_key(app: AppHandle, api_key: String) -> Result<(), String> {
    let global = app
        .try_state::<Arc<GlobalConfig>>()
        .ok_or("Config not initialized")?;

    // 更新内存中的配置
    global.set_api_key(api_key);

    // 保存到文件
    let config = (*global.get()).clone();
    ConfigManager::save(&app, &config).map_err(|e| e.to_string())?;

    tracing::info!("API key updated via command");
    Ok(())
}

/// 检查是否已配置 API 密钥
#[command]
pub fn has_api_key(app: AppHandle) -> Result<bool, String> {
    let config = app
        .try_state::<Arc<GlobalConfig>>()
        .ok_or("Config not initialized")?;

    Ok(config.has_api_key())
}

/// 重置配置为默认值
#[command]
pub fn reset_config(app: AppHandle) -> Result<AppConfig, String> {
    let config = ConfigManager::reset(&app).map_err(|e| e.to_string())?;

    // 更新全局配置
    if let Some(global) = app.try_state::<Arc<GlobalConfig>>() {
        global.update(config.clone());
    }

    tracing::info!("Config reset via command");
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 命令测试需要 Tauri 环境，这里只测试基本逻辑
    #[test]
    fn test_command_signatures() {
        // 确保命令签名正确
        // 实际测试需要 Tauri 测试环境
    }
}
