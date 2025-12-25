//! 状态相关的 Tauri 命令
//!
//! 提供前端访问和控制应用状态的命令

use std::sync::Arc;

use tauri::{command, AppHandle, Manager};

use crate::state::{AppState, StateChangeEvent, StateManager};

/// 获取当前应用状态
///
/// 返回当前状态的详细信息
#[command]
pub fn get_current_state(app: AppHandle) -> Result<StateChangeEvent, String> {
    let state_manager = app
        .try_state::<Arc<StateManager>>()
        .ok_or_else(|| "StateManager not available".to_string())?;

    let current = state_manager.current();
    Ok(StateChangeEvent::from(current.as_ref()))
}

/// 获取当前状态名称
#[command]
pub fn get_state_name(app: AppHandle) -> Result<String, String> {
    let state_manager = app
        .try_state::<Arc<StateManager>>()
        .ok_or_else(|| "StateManager not available".to_string())?;

    Ok(state_manager.current().name().to_string())
}

/// 检查是否处于空闲状态
#[command]
pub fn is_idle(app: AppHandle) -> Result<bool, String> {
    let state_manager = app
        .try_state::<Arc<StateManager>>()
        .ok_or_else(|| "StateManager not available".to_string())?;

    Ok(state_manager.current().is_idle())
}

/// 检查是否处于录音状态
#[command]
pub fn is_recording(app: AppHandle) -> Result<bool, String> {
    let state_manager = app
        .try_state::<Arc<StateManager>>()
        .ok_or_else(|| "StateManager not available".to_string())?;

    Ok(state_manager.current().is_recording())
}

/// 检查是否处于错误状态
#[command]
pub fn is_error(app: AppHandle) -> Result<bool, String> {
    let state_manager = app
        .try_state::<Arc<StateManager>>()
        .ok_or_else(|| "StateManager not available".to_string())?;

    Ok(state_manager.current().is_error())
}

/// 重置状态为空闲
///
/// 用于错误恢复或取消操作
#[command]
pub fn reset_state(app: AppHandle) -> Result<(), String> {
    let state_manager = app
        .try_state::<Arc<StateManager>>()
        .ok_or_else(|| "StateManager not available".to_string())?;

    state_manager.reset();
    tracing::info!("State reset to Idle via command");
    Ok(())
}

/// 从错误状态恢复
#[command]
pub fn recover_from_error(app: AppHandle) -> Result<(), String> {
    let state_manager = app
        .try_state::<Arc<StateManager>>()
        .ok_or_else(|| "StateManager not available".to_string())?;

    if state_manager.current().is_error() {
        state_manager
            .transition(AppState::idle())
            .map_err(|e| e.to_string())?;
        tracing::info!("Recovered from error state via command");
        Ok(())
    } else {
        Err("Not in error state".to_string())
    }
}

#[cfg(test)]
mod tests {
    // 注意：这些测试需要 Tauri 应用环境，在集成测试中运行
}
