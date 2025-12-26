/// Audio processing modules
pub mod audio;

/// Tauri commands
pub mod commands;

/// Hotkey management modules
pub mod hotkey;

/// Input injection and window detection modules
pub mod input;

/// Network communication modules
pub mod network;

/// State management modules
pub mod state;

/// RaFlow session management (complete flow integration)
pub mod session;

/// System tray management
pub mod tray;

/// End-to-end transcription session management
pub mod transcription;

/// Utility modules
pub mod utils;

use std::sync::Arc;

use tauri::Manager;

use state::{init_config, GlobalConfig, StateManager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    utils::logging::init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            tracing::info!("Setting up RaFlow application");

            // Initialize state manager
            let state_manager = Arc::new(StateManager::new());
            app.manage(state_manager);

            // Initialize config
            match init_config(app.handle()) {
                Ok(config) => {
                    tracing::info!(
                        has_api_key = config.has_api_key(),
                        "Config initialized"
                    );
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to initialize config, using defaults");
                    app.manage(Arc::new(GlobalConfig::default()));
                }
            }

            // Setup system tray
            if let Err(e) = tray::setup_tray(app.handle()) {
                tracing::error!(error = %e, "Failed to setup system tray");
            }

            tracing::info!("RaFlow setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::state::get_current_state,
            commands::state::get_state_name,
            commands::state::is_idle,
            commands::state::is_recording,
            commands::state::is_error,
            commands::state::reset_state,
            commands::state::recover_from_error,
            commands::config::get_config,
            commands::config::save_config,
            commands::config::get_api_key,
            commands::config::set_api_key,
            commands::config::has_api_key,
            commands::config::reset_config,
            commands::window::show_overlay,
            commands::window::hide_overlay,
            commands::window::toggle_overlay,
            commands::window::show_settings,
            commands::window::hide_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
