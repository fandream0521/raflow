//! Phase 3 集成测试 - 用户界面
//!
//! 测试窗口配置、系统托盘和配置持久化功能

use raflow_lib::state::config::{
    ApiConfig, AppConfig, AudioConfig, BehaviorConfig, ConfigError, GlobalConfig,
};
use raflow_lib::input::InjectionStrategy;
use raflow_lib::tray::{menu_ids, TrayError};

// ==================== AppConfig 测试 ====================

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();

    // API 配置默认值
    assert!(config.api.api_key.is_empty());
    assert_eq!(config.api.model_id, "scribe_v2_realtime");
    assert_eq!(config.api.language_code, Some("zh".to_string()));
    assert!(!config.api.include_timestamps);

    // 音频配置默认值
    assert!(config.audio.input_device_id.is_none());
    assert_eq!(config.audio.gain, 1.0);
    assert!(!config.audio.noise_suppression);

    // 行为配置默认值
    assert_eq!(config.behavior.injection_strategy, InjectionStrategy::Auto);
    assert_eq!(config.behavior.auto_threshold, 20);
    assert!(config.behavior.show_overlay);
    assert!(config.behavior.auto_inject);
}

#[test]
fn test_app_config_serialization_roundtrip() {
    let config = AppConfig::default();

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.api.model_id, deserialized.api.model_id);
    assert_eq!(config.api.language_code, deserialized.api.language_code);
    assert_eq!(config.audio.gain, deserialized.audio.gain);
    assert_eq!(
        config.behavior.injection_strategy,
        deserialized.behavior.injection_strategy
    );
}

#[test]
fn test_app_config_partial_json_parsing() {
    // 只提供部分字段，其余使用默认值
    let json = r#"{
        "api": {
            "api_key": "test-api-key-12345"
        },
        "behavior": {
            "show_overlay": false
        }
    }"#;

    let config: AppConfig = serde_json::from_str(json).unwrap();

    // 指定的值
    assert_eq!(config.api.api_key, "test-api-key-12345");
    assert!(!config.behavior.show_overlay);

    // 默认值
    assert_eq!(config.api.model_id, "scribe_v2_realtime");
    assert_eq!(config.audio.gain, 1.0);
    assert!(config.behavior.auto_inject);
}

#[test]
fn test_app_config_full_json() {
    let json = r#"{
        "api": {
            "api_key": "sk-test-key",
            "model_id": "scribe_v2_realtime",
            "language_code": "en",
            "include_timestamps": true,
            "vad_commit_strategy": "auto"
        },
        "audio": {
            "input_device_id": "device-123",
            "input_device_name": "Test Microphone",
            "gain": 1.5,
            "noise_suppression": true,
            "silence_threshold": 0.02
        },
        "hotkeys": {
            "push_to_talk": "CommandOrControl+Shift+Space",
            "cancel": "Escape",
            "toggle_mode": null
        },
        "behavior": {
            "injection_strategy": "Clipboard",
            "auto_threshold": 30,
            "paste_delay_ms": 150,
            "pre_injection_delay_ms": 100,
            "auto_inject": false,
            "show_overlay": true,
            "auto_start": true,
            "minimize_to_tray": false,
            "processing_timeout_secs": 60
        }
    }"#;

    let config: AppConfig = serde_json::from_str(json).unwrap();

    // 验证所有值
    assert_eq!(config.api.api_key, "sk-test-key");
    assert_eq!(config.api.language_code, Some("en".to_string()));
    assert!(config.api.include_timestamps);

    assert_eq!(config.audio.input_device_id, Some("device-123".to_string()));
    assert_eq!(config.audio.gain, 1.5);
    assert!(config.audio.noise_suppression);

    assert_eq!(config.behavior.injection_strategy, InjectionStrategy::Clipboard);
    assert_eq!(config.behavior.auto_threshold, 30);
    assert!(!config.behavior.auto_inject);
    assert!(config.behavior.auto_start);
}

// ==================== GlobalConfig 测试 ====================

#[test]
fn test_global_config_creation() {
    let config = GlobalConfig::default();
    assert!(!config.has_api_key());
}

#[test]
fn test_global_config_api_key_operations() {
    let config = GlobalConfig::default();

    assert!(!config.has_api_key());
    assert!(config.api_key().is_empty());

    config.set_api_key("test-key-123".to_string());

    assert!(config.has_api_key());
    assert_eq!(config.api_key(), "test-key-123");
}

#[test]
fn test_global_config_update() {
    let global = GlobalConfig::default();

    let mut new_config = AppConfig::default();
    new_config.api.api_key = "updated-key".to_string();
    new_config.behavior.show_overlay = false;
    new_config.audio.gain = 2.0;

    global.update(new_config);

    let config = global.get();
    assert_eq!(config.api.api_key, "updated-key");
    assert!(!config.behavior.show_overlay);
    assert_eq!(config.audio.gain, 2.0);
}

#[test]
fn test_global_config_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let config = Arc::new(GlobalConfig::default());

    // 多线程并发读取
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let config = Arc::clone(&config);
            thread::spawn(move || {
                for _ in 0..100 {
                    let _ = config.get();
                    let _ = config.has_api_key();
                    if i == 0 {
                        config.set_api_key(format!("key-{}", i));
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // 应该没有崩溃
    assert!(config.has_api_key());
}

// ==================== ApiConfig 测试 ====================

#[test]
fn test_api_config_default() {
    let config = ApiConfig::default();

    assert!(config.api_key.is_empty());
    assert_eq!(config.model_id, "scribe_v2_realtime");
    assert_eq!(config.language_code, Some("zh".to_string()));
    assert!(!config.include_timestamps);
    assert!(config.vad_commit_strategy.is_none());
}

#[test]
fn test_api_config_serialization() {
    let config = ApiConfig {
        api_key: "sk-test".to_string(),
        model_id: "scribe_v2_realtime".to_string(),
        language_code: Some("ja".to_string()),
        include_timestamps: true,
        vad_commit_strategy: Some("auto".to_string()),
    };

    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("sk-test"));
    assert!(json.contains("ja"));

    let deserialized: ApiConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.api_key, deserialized.api_key);
    assert_eq!(config.language_code, deserialized.language_code);
}

// ==================== AudioConfig 测试 ====================

#[test]
fn test_audio_config_default() {
    let config = AudioConfig::default();

    assert!(config.input_device_id.is_none());
    assert!(config.input_device_name.is_none());
    assert_eq!(config.gain, 1.0);
    assert!(!config.noise_suppression);
    assert_eq!(config.silence_threshold, 0.01);
}

#[test]
fn test_audio_config_gain_range() {
    let json = r#"{"gain": 0.5}"#;
    let config: AudioConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.gain, 0.5);

    let json = r#"{"gain": 2.0}"#;
    let config: AudioConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.gain, 2.0);
}

// ==================== BehaviorConfig 测试 ====================

#[test]
fn test_behavior_config_default() {
    let config = BehaviorConfig::default();

    assert_eq!(config.injection_strategy, InjectionStrategy::Auto);
    assert_eq!(config.auto_threshold, 20);
    assert_eq!(config.paste_delay_ms, 100);
    assert_eq!(config.pre_injection_delay_ms, 50);
    assert!(config.auto_inject);
    assert!(config.show_overlay);
    assert!(!config.auto_start);
    assert!(config.minimize_to_tray);
    assert_eq!(config.processing_timeout_secs, 30);
}

#[test]
fn test_behavior_config_injection_strategies() {
    for strategy in [
        InjectionStrategy::Auto,
        InjectionStrategy::Keyboard,
        InjectionStrategy::Clipboard,
        InjectionStrategy::ClipboardOnly,
    ] {
        let config = BehaviorConfig {
            injection_strategy: strategy,
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: BehaviorConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.injection_strategy, deserialized.injection_strategy);
    }
}

// ==================== ConfigError 测试 ====================

#[test]
fn test_config_error_display() {
    let err = ConfigError::Path("invalid path".to_string());
    assert!(err.to_string().contains("invalid path"));

    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err = ConfigError::Io(io_err);
    assert!(err.to_string().contains("IO error"));
}

// ==================== TrayError 测试 ====================

#[test]
fn test_tray_error_display() {
    let err = TrayError::MenuCreation("menu error".to_string());
    assert!(err.to_string().contains("menu error"));

    let err = TrayError::TrayCreation("tray error".to_string());
    assert!(err.to_string().contains("tray error"));

    let err = TrayError::WindowNotFound("main".to_string());
    assert!(err.to_string().contains("main"));

    let err = TrayError::IconLoad("icon not found".to_string());
    assert!(err.to_string().contains("icon not found"));
}

// ==================== Menu IDs 测试 ====================

#[test]
fn test_menu_ids() {
    assert_eq!(menu_ids::SHOW_SETTINGS, "show_settings");
    assert_eq!(menu_ids::TOGGLE_OVERLAY, "toggle_overlay");
    assert_eq!(menu_ids::QUIT, "quit");
}

// ==================== 配置文件格式测试 ====================

#[test]
fn test_config_json_pretty_format() {
    let config = AppConfig::default();
    let json = serde_json::to_string_pretty(&config).unwrap();

    // 应该是格式化的 JSON
    assert!(json.contains('\n'));
    assert!(json.contains("  ")); // 缩进
}

#[test]
fn test_config_empty_optional_fields() {
    let config = AppConfig::default();
    let json = serde_json::to_string(&config).unwrap();

    // None 值应该被序列化
    // 这取决于 serde 的配置，但我们检查它是有效的 JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn test_config_unicode_values() {
    let mut config = AppConfig::default();
    config.audio.input_device_name = Some("麦克风 (中文设备名)".to_string());

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(
        deserialized.audio.input_device_name,
        Some("麦克风 (中文设备名)".to_string())
    );
}
