//! 热键模块集成测试

use raflow_lib::hotkey::{HotkeyConfig, HotkeyError, HotkeyEvent, HotkeyManager};

// ============================================================================
// HotkeyConfig 测试
// ============================================================================

#[test]
fn test_hotkey_config_default_values() {
    let config = HotkeyConfig::default();
    assert_eq!(config.push_to_talk, "CommandOrControl+Shift+.");
    assert_eq!(config.cancel, "Escape");
    assert!(config.toggle_mode.is_none());
}

#[test]
fn test_hotkey_config_custom_values() {
    let config = HotkeyConfig::new("Alt+Space", "Ctrl+C");
    assert_eq!(config.push_to_talk, "Alt+Space");
    assert_eq!(config.cancel, "Ctrl+C");
}

#[test]
fn test_hotkey_config_builder_chain() {
    let config = HotkeyConfig::default()
        .with_push_to_talk("F1")
        .with_cancel("F2")
        .with_toggle_mode("F3");

    assert_eq!(config.push_to_talk, "F1");
    assert_eq!(config.cancel, "F2");
    assert_eq!(config.toggle_mode, Some("F3".to_string()));
}

#[test]
fn test_hotkey_config_all_hotkeys_without_toggle() {
    let config = HotkeyConfig::default();
    let hotkeys = config.all_hotkeys();

    assert_eq!(hotkeys.len(), 2);
    assert!(hotkeys.contains(&config.push_to_talk.as_str()));
    assert!(hotkeys.contains(&config.cancel.as_str()));
}

#[test]
fn test_hotkey_config_all_hotkeys_with_toggle() {
    let config = HotkeyConfig::default().with_toggle_mode("Ctrl+M");
    let hotkeys = config.all_hotkeys();

    assert_eq!(hotkeys.len(), 3);
    assert!(hotkeys.contains(&"Ctrl+M"));
}

#[test]
fn test_hotkey_config_identification() {
    let config = HotkeyConfig::default().with_toggle_mode("Ctrl+T");

    // Push-to-Talk 识别
    assert!(config.is_push_to_talk("CommandOrControl+Shift+."));
    assert!(!config.is_push_to_talk("Escape"));
    assert!(!config.is_push_to_talk("Ctrl+T"));

    // Cancel 识别
    assert!(config.is_cancel("Escape"));
    assert!(!config.is_cancel("CommandOrControl+Shift+."));
    assert!(!config.is_cancel("Ctrl+T"));

    // Toggle Mode 识别
    assert!(config.is_toggle_mode("Ctrl+T"));
    assert!(!config.is_toggle_mode("Escape"));
    assert!(!config.is_toggle_mode("CommandOrControl+Shift+."));
}

#[test]
fn test_hotkey_config_without_toggle_mode() {
    let config = HotkeyConfig::default();

    // 没有 toggle_mode 时
    assert!(!config.is_toggle_mode("Any"));
    assert!(config.toggle_mode.is_none());
}

#[test]
fn test_hotkey_config_serialization_roundtrip() {
    let original = HotkeyConfig::default()
        .with_push_to_talk("Ctrl+Alt+R")
        .with_toggle_mode("Ctrl+M");

    // 序列化
    let json = serde_json::to_string(&original).unwrap();

    // 反序列化
    let restored: HotkeyConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(original, restored);
}

#[test]
fn test_hotkey_config_json_format() {
    let config = HotkeyConfig::new("Ctrl+Space", "Escape").with_toggle_mode("Ctrl+M");

    let json = serde_json::to_string_pretty(&config).unwrap();

    // 验证 JSON 包含所有字段
    assert!(json.contains("push_to_talk"));
    assert!(json.contains("cancel"));
    assert!(json.contains("toggle_mode"));
    assert!(json.contains("Ctrl+Space"));
    assert!(json.contains("Escape"));
    assert!(json.contains("Ctrl+M"));
}

#[test]
fn test_hotkey_config_equality() {
    let config1 = HotkeyConfig::default();
    let config2 = HotkeyConfig::default();
    let config3 = HotkeyConfig::new("Different", "Escape");

    assert_eq!(config1, config2);
    assert_ne!(config1, config3);
}

#[test]
fn test_hotkey_config_clone() {
    let original = HotkeyConfig::default().with_toggle_mode("Ctrl+T");
    let cloned = original.clone();

    assert_eq!(original, cloned);
}

// ============================================================================
// HotkeyError 测试
// ============================================================================

#[test]
fn test_hotkey_error_invalid_format() {
    let error = HotkeyError::InvalidFormat("bad key".to_string());
    let message = format!("{}", error);
    assert!(message.contains("Invalid hotkey format"));
    assert!(message.contains("bad key"));
}

#[test]
fn test_hotkey_error_registration_failed() {
    let error = HotkeyError::RegistrationFailed {
        hotkey: "Ctrl+A".to_string(),
        reason: "already in use".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("Failed to register hotkey"));
    assert!(message.contains("Ctrl+A"));
    assert!(message.contains("already in use"));
}

#[test]
fn test_hotkey_error_unregistration_failed() {
    let error = HotkeyError::UnregistrationFailed {
        hotkey: "Ctrl+B".to_string(),
        reason: "not found".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("Failed to unregister hotkey"));
    assert!(message.contains("Ctrl+B"));
}

#[test]
fn test_hotkey_error_already_registered() {
    let error = HotkeyError::AlreadyRegistered("Ctrl+C".to_string());
    let message = format!("{}", error);
    assert!(message.contains("already registered"));
    assert!(message.contains("Ctrl+C"));
}

#[test]
fn test_hotkey_error_not_registered() {
    let error = HotkeyError::NotRegistered("Ctrl+D".to_string());
    let message = format!("{}", error);
    assert!(message.contains("not registered"));
    assert!(message.contains("Ctrl+D"));
}

#[test]
fn test_hotkey_error_occupied() {
    let error = HotkeyError::Occupied("Ctrl+E".to_string());
    let message = format!("{}", error);
    assert!(message.contains("occupied"));
    assert!(message.contains("Ctrl+E"));
}

#[test]
fn test_hotkey_error_plugin_not_available() {
    let error = HotkeyError::PluginNotAvailable;
    let message = format!("{}", error);
    assert!(message.contains("plugin is not available"));
}

#[test]
fn test_hotkey_error_config_error() {
    let error = HotkeyError::ConfigError("invalid value".to_string());
    let message = format!("{}", error);
    assert!(message.contains("configuration error"));
    assert!(message.contains("invalid value"));
}

#[test]
fn test_hotkey_error_equality() {
    let error1 = HotkeyError::InvalidFormat("test".to_string());
    let error2 = HotkeyError::InvalidFormat("test".to_string());
    let error3 = HotkeyError::InvalidFormat("other".to_string());

    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
}

#[test]
fn test_hotkey_error_clone() {
    let original = HotkeyError::RegistrationFailed {
        hotkey: "Ctrl+F".to_string(),
        reason: "test".to_string(),
    };
    let cloned = original.clone();

    assert_eq!(original, cloned);
}

// ============================================================================
// HotkeyEvent 测试
// ============================================================================

#[test]
fn test_hotkey_event_types() {
    let events = vec![
        HotkeyEvent::PushToTalkPressed,
        HotkeyEvent::PushToTalkReleased,
        HotkeyEvent::CancelPressed,
        HotkeyEvent::ToggleModePressed,
    ];

    // 验证所有事件类型不同
    for (i, event1) in events.iter().enumerate() {
        for (j, event2) in events.iter().enumerate() {
            if i == j {
                assert_eq!(event1, event2);
            } else {
                assert_ne!(event1, event2);
            }
        }
    }
}

#[test]
fn test_hotkey_event_clone() {
    let event = HotkeyEvent::PushToTalkPressed;
    let cloned = event.clone();
    assert_eq!(event, cloned);
}

#[test]
fn test_hotkey_event_debug() {
    let event = HotkeyEvent::PushToTalkPressed;
    let debug = format!("{:?}", event);
    assert!(debug.contains("PushToTalkPressed"));
}

// ============================================================================
// HotkeyManager 测试
// ============================================================================

#[test]
fn test_hotkey_manager_creation() {
    let config = HotkeyConfig::default();
    let manager = HotkeyManager::new(config.clone());

    assert_eq!(manager.config(), &config);
    assert!(manager.registered_shortcuts().is_empty());
}

#[test]
fn test_hotkey_manager_custom_config() {
    let config = HotkeyConfig::new("F1", "F2").with_toggle_mode("F3");
    let manager = HotkeyManager::new(config.clone());

    assert_eq!(manager.config().push_to_talk, "F1");
    assert_eq!(manager.config().cancel, "F2");
    assert_eq!(manager.config().toggle_mode, Some("F3".to_string()));
}

#[test]
fn test_hotkey_manager_update_config() {
    let config1 = HotkeyConfig::default();
    let mut manager = HotkeyManager::new(config1);

    let config2 = HotkeyConfig::new("Alt+Space", "Ctrl+X");
    manager.update_config(config2.clone());

    assert_eq!(manager.config(), &config2);
}

#[test]
fn test_hotkey_manager_config_reference() {
    let config = HotkeyConfig::default().with_toggle_mode("Ctrl+M");
    let manager = HotkeyManager::new(config);

    // 验证可以通过引用访问配置
    assert!(manager.config().is_push_to_talk("CommandOrControl+Shift+."));
    assert!(manager.config().is_cancel("Escape"));
    assert!(manager.config().is_toggle_mode("Ctrl+M"));
}

// ============================================================================
// 综合场景测试
// ============================================================================

#[test]
fn test_config_workflow() {
    // 模拟用户自定义配置的工作流程

    // 1. 从默认配置开始
    let mut config = HotkeyConfig::default();
    assert_eq!(config.push_to_talk, "CommandOrControl+Shift+.");

    // 2. 用户修改 Push-to-Talk 热键
    config = config.with_push_to_talk("Ctrl+Shift+R");
    assert_eq!(config.push_to_talk, "Ctrl+Shift+R");

    // 3. 用户添加切换模式热键
    config = config.with_toggle_mode("Ctrl+Shift+M");
    assert!(config.toggle_mode.is_some());

    // 4. 保存配置
    let json = serde_json::to_string(&config).unwrap();
    assert!(!json.is_empty());

    // 5. 重新加载配置
    let loaded: HotkeyConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, loaded);
}

#[test]
fn test_multiple_hotkey_formats() {
    // 测试各种热键格式
    let formats = vec![
        "Ctrl+A",
        "Alt+B",
        "Shift+C",
        "CommandOrControl+D",
        "Ctrl+Shift+E",
        "Ctrl+Alt+F",
        "Alt+Shift+G",
        "Ctrl+Alt+Shift+H",
        "F1",
        "F12",
        "Escape",
        "Space",
        "Enter",
    ];

    for format in formats {
        let config = HotkeyConfig::new(format, "Escape");
        assert_eq!(config.push_to_talk, format);
    }
}

#[test]
fn test_config_persistence_scenario() {
    // 模拟配置持久化场景
    let original_config = HotkeyConfig::default()
        .with_push_to_talk("Ctrl+Shift+V")
        .with_cancel("Ctrl+Escape")
        .with_toggle_mode("Ctrl+Shift+T");

    // 序列化为 JSON（模拟保存到文件）
    let json = serde_json::to_string_pretty(&original_config).unwrap();

    // 反序列化（模拟从文件加载）
    let loaded_config: HotkeyConfig = serde_json::from_str(&json).unwrap();

    // 验证所有字段正确恢复
    assert_eq!(original_config.push_to_talk, loaded_config.push_to_talk);
    assert_eq!(original_config.cancel, loaded_config.cancel);
    assert_eq!(original_config.toggle_mode, loaded_config.toggle_mode);
}

// ============================================================================
// 需要 Tauri 环境的测试（标记为 ignore）
// ============================================================================

#[test]
#[ignore = "Requires Tauri application environment"]
fn test_register_hotkeys_integration() {
    // 此测试需要完整的 Tauri 应用环境
    // 在 CI 中应该使用 tauri-test 框架进行测试
}

#[test]
#[ignore = "Requires Tauri application environment"]
fn test_unregister_hotkeys_integration() {
    // 此测试需要完整的 Tauri 应用环境
}

#[test]
#[ignore = "Requires Tauri application environment"]
fn test_hotkey_event_handling_integration() {
    // 此测试需要完整的 Tauri 应用环境和真实的热键事件
}
