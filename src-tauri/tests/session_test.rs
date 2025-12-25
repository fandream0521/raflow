//! RaFlow 会话集成测试
//!
//! 测试会话模块的 API 和类型导出
//!
//! 注意：由于 RaFlowSession 需要 Tauri AppHandle 和网络连接，
//! 大多数功能测试需要在完整 Tauri 应用环境中进行。
//! 这里主要测试配置类型、事件类型、错误类型和 API 结构。

use raflow_lib::session::{SessionConfig, SessionError, SessionEvent};
use raflow_lib::input::InjectionStrategy;

// ==================== SessionConfig 测试 ====================

#[test]
fn test_session_config_default() {
    let config = SessionConfig::default();

    assert_eq!(config.injection_strategy, InjectionStrategy::Auto);
    assert_eq!(config.auto_threshold, 20);
    assert_eq!(config.paste_delay_ms, 100);
    assert_eq!(config.pre_injection_delay_ms, 50);
    assert!(config.auto_inject);
}

#[test]
fn test_session_config_clipboard_only() {
    let config = SessionConfig::clipboard_only();

    assert_eq!(config.injection_strategy, InjectionStrategy::ClipboardOnly);
    assert!(!config.auto_inject);
    // 其他值应该使用默认值
    assert_eq!(config.auto_threshold, 20);
    assert_eq!(config.paste_delay_ms, 100);
    assert_eq!(config.pre_injection_delay_ms, 50);
}

#[test]
fn test_session_config_keyboard_only() {
    let config = SessionConfig::keyboard_only();

    assert_eq!(config.injection_strategy, InjectionStrategy::Keyboard);
    assert!(config.auto_inject);
}

#[test]
fn test_session_config_clipboard_paste() {
    let config = SessionConfig::clipboard_paste();

    assert_eq!(config.injection_strategy, InjectionStrategy::Clipboard);
    assert!(config.auto_inject);
}

#[test]
fn test_session_config_clone() {
    let config = SessionConfig::default();
    let cloned = config.clone();

    assert_eq!(config.injection_strategy, cloned.injection_strategy);
    assert_eq!(config.auto_threshold, cloned.auto_threshold);
    assert_eq!(config.paste_delay_ms, cloned.paste_delay_ms);
    assert_eq!(config.pre_injection_delay_ms, cloned.pre_injection_delay_ms);
    assert_eq!(config.auto_inject, cloned.auto_inject);
}

#[test]
fn test_session_config_debug() {
    let config = SessionConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("SessionConfig"));
    assert!(debug_str.contains("injection_strategy"));
    assert!(debug_str.contains("auto_threshold"));
}

// ==================== SessionConfig 序列化测试 ====================

#[test]
fn test_session_config_serialization() {
    let config = SessionConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: SessionConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.injection_strategy, deserialized.injection_strategy);
    assert_eq!(config.auto_threshold, deserialized.auto_threshold);
    assert_eq!(config.paste_delay_ms, deserialized.paste_delay_ms);
    assert_eq!(config.pre_injection_delay_ms, deserialized.pre_injection_delay_ms);
    assert_eq!(config.auto_inject, deserialized.auto_inject);
}

#[test]
fn test_session_config_json_roundtrip() {
    let configs = vec![
        SessionConfig::default(),
        SessionConfig::clipboard_only(),
        SessionConfig::keyboard_only(),
        SessionConfig::clipboard_paste(),
    ];

    for config in configs {
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SessionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.injection_strategy, deserialized.injection_strategy);
    }
}

#[test]
fn test_session_config_custom_values() {
    // 测试自定义值的序列化和反序列化
    let json = r#"{
        "injection_strategy": "Keyboard",
        "auto_threshold": 50,
        "paste_delay_ms": 200,
        "pre_injection_delay_ms": 100,
        "auto_inject": false
    }"#;

    let config: SessionConfig = serde_json::from_str(json).unwrap();

    assert_eq!(config.injection_strategy, InjectionStrategy::Keyboard);
    assert_eq!(config.auto_threshold, 50);
    assert_eq!(config.paste_delay_ms, 200);
    assert_eq!(config.pre_injection_delay_ms, 100);
    assert!(!config.auto_inject);
}

// ==================== SessionEvent 测试 ====================

#[test]
fn test_session_event_started() {
    let event = SessionEvent::Started {
        session_id: "test-session-123".to_string(),
    };
    let json = serde_json::to_string(&event).unwrap();

    assert!(json.contains("Started"));
    assert!(json.contains("test-session-123"));
    assert!(json.contains("session_id"));
}

#[test]
fn test_session_event_partial_transcript() {
    let event = SessionEvent::PartialTranscript {
        text: "hello world".to_string(),
    };
    let json = serde_json::to_string(&event).unwrap();

    assert!(json.contains("PartialTranscript"));
    assert!(json.contains("hello world"));
}

#[test]
fn test_session_event_committed_transcript() {
    let event = SessionEvent::CommittedTranscript {
        text: "这是最终的转写文本".to_string(),
    };
    let json = serde_json::to_string(&event).unwrap();

    assert!(json.contains("CommittedTranscript"));
    assert!(json.contains("这是最终的转写文本"));
}

#[test]
fn test_session_event_text_injected() {
    let event = SessionEvent::TextInjected {
        text: "injected text".to_string(),
        strategy: "Auto".to_string(),
    };
    let json = serde_json::to_string(&event).unwrap();

    assert!(json.contains("TextInjected"));
    assert!(json.contains("injected text"));
    assert!(json.contains("Auto"));
    assert!(json.contains("strategy"));
}

#[test]
fn test_session_event_text_copied() {
    let event = SessionEvent::TextCopied {
        text: "copied text".to_string(),
    };
    let json = serde_json::to_string(&event).unwrap();

    assert!(json.contains("TextCopied"));
    assert!(json.contains("copied text"));
}

#[test]
fn test_session_event_stopped() {
    let event = SessionEvent::Stopped;
    let json = serde_json::to_string(&event).unwrap();

    assert!(json.contains("Stopped"));
}

#[test]
fn test_session_event_error() {
    let event = SessionEvent::Error {
        message: "Connection failed".to_string(),
    };
    let json = serde_json::to_string(&event).unwrap();

    assert!(json.contains("Error"));
    assert!(json.contains("Connection failed"));
}

#[test]
fn test_session_event_clone() {
    let event = SessionEvent::Started {
        session_id: "test".to_string(),
    };
    let cloned = event.clone();

    if let SessionEvent::Started { session_id } = cloned {
        assert_eq!(session_id, "test");
    } else {
        panic!("Event type changed after clone");
    }
}

#[test]
fn test_session_event_debug() {
    let event = SessionEvent::PartialTranscript {
        text: "test".to_string(),
    };
    let debug_str = format!("{:?}", event);

    assert!(debug_str.contains("PartialTranscript"));
    assert!(debug_str.contains("test"));
}

// ==================== SessionEvent 序列化格式测试 ====================

#[test]
fn test_session_event_tagged_format() {
    // 验证事件使用 tagged 格式序列化
    let event = SessionEvent::Started {
        session_id: "123".to_string(),
    };
    let json = serde_json::to_string(&event).unwrap();

    // 应该包含 "type" 字段
    assert!(json.contains(r#""type":"Started""#));
    // 应该包含 "payload" 字段
    assert!(json.contains(r#""payload""#));
}

#[test]
fn test_session_event_stopped_tagged_format() {
    let event = SessionEvent::Stopped;
    let json = serde_json::to_string(&event).unwrap();

    // Stopped 事件没有 payload
    assert!(json.contains(r#""type":"Stopped""#));
}

// ==================== SessionError 测试 ====================

#[test]
fn test_session_error_state() {
    let err = SessionError::StateError("invalid transition".to_string());
    let msg = err.to_string();

    assert!(msg.contains("State error"));
    assert!(msg.contains("invalid transition"));
}

#[test]
fn test_session_error_injection() {
    let err = SessionError::InjectionError("clipboard failed".to_string());
    let msg = err.to_string();

    assert!(msg.contains("Injection error"));
    assert!(msg.contains("clipboard failed"));
}

#[test]
fn test_session_error_no_text() {
    let err = SessionError::NoTextToInject;
    let msg = err.to_string();

    assert!(msg.contains("No text to inject"));
}

#[test]
fn test_session_error_not_running() {
    let err = SessionError::NotRunning;
    let msg = err.to_string();

    assert!(msg.contains("not running"));
}

#[test]
fn test_session_error_debug() {
    let err = SessionError::StateError("test".to_string());
    let debug_str = format!("{:?}", err);

    assert!(debug_str.contains("StateError"));
    assert!(debug_str.contains("test"));
}

// ==================== API 存在性测试 ====================

#[test]
fn test_session_module_exports() {
    // 验证模块导出了正确的类型
    use raflow_lib::session::{RaFlowSession, SessionConfig, SessionError, SessionEvent};

    // 类型存在即可
    fn _use_config(config: SessionConfig) {
        let _ = config.injection_strategy;
        let _ = config.auto_threshold;
        let _ = config.paste_delay_ms;
        let _ = config.pre_injection_delay_ms;
        let _ = config.auto_inject;
    }

    fn _use_event(event: SessionEvent) {
        let _ = format!("{:?}", event);
    }

    fn _use_error(err: SessionError) {
        let _ = err.to_string();
    }

    // RaFlowSession 需要 AppHandle，这里只验证类型存在
    fn _use_session(app: &tauri::AppHandle) {
        // 验证方法存在（不实际调用）
        let _ = std::mem::size_of::<RaFlowSession>();
        let _ = app;
    }
}

#[test]
fn test_raflow_session_methods_exist() {
    // 验证 RaFlowSession 的方法签名存在
    use raflow_lib::session::{RaFlowSession, SessionConfig};

    fn _verify_methods(app: &tauri::AppHandle) {
        // start 方法（async）
        async fn _start(app: &tauri::AppHandle) {
            let config = SessionConfig::default();
            let _result = RaFlowSession::start(app, "api-key", config).await;
        }

        // stop 方法（async）
        async fn _stop(session: &mut RaFlowSession) {
            let _result = session.stop().await;
        }

        // is_running 方法
        fn _is_running(session: &RaFlowSession) {
            let _running = session.is_running();
        }

        // current_state 方法
        fn _current_state(session: &RaFlowSession) {
            let _state = session.current_state();
        }

        // config 方法
        fn _config(session: &RaFlowSession) {
            let _config = session.config();
        }

        // last_committed_text 方法（async）
        async fn _last_committed(session: &RaFlowSession) {
            let _text = session.last_committed_text().await;
        }

        // inject_last_committed 方法（async）
        async fn _inject_last(session: &mut RaFlowSession) {
            let _result = session.inject_last_committed().await;
        }

        let _ = app;
    }
}

// ==================== 状态集成测试 ====================

#[test]
fn test_config_with_state_types() {
    // 测试 SessionConfig 与状态模块的集成
    use raflow_lib::state::{AppState, RecordingState};
    use raflow_lib::session::SessionConfig;

    let _config = SessionConfig::default();
    let _state = AppState::Recording(RecordingState::Listening);
}

// ==================== Unicode 支持测试 ====================

#[test]
fn test_session_event_unicode_support() {
    let event = SessionEvent::CommittedTranscript {
        text: "你好世界 🎤 مرحبا".to_string(),
    };
    let json = serde_json::to_string(&event).unwrap();

    // JSON 应该正确编码 Unicode
    assert!(json.contains("你好世界") || json.contains("\\u"));
}

#[test]
fn test_session_event_empty_text() {
    let event = SessionEvent::PartialTranscript {
        text: "".to_string(),
    };
    let json = serde_json::to_string(&event).unwrap();

    assert!(json.contains("PartialTranscript"));
    assert!(json.contains(r#""text":"""#));
}

#[test]
fn test_session_event_long_text() {
    let long_text = "a".repeat(10000);
    let event = SessionEvent::CommittedTranscript {
        text: long_text.clone(),
    };
    let json = serde_json::to_string(&event).unwrap();

    assert!(json.contains(&long_text));
}

// ==================== 错误处理边界测试 ====================

#[test]
fn test_session_error_empty_message() {
    let err = SessionError::StateError("".to_string());
    let msg = err.to_string();

    assert!(msg.contains("State error"));
}

#[test]
fn test_session_error_long_message() {
    let long_msg = "error".repeat(1000);
    let err = SessionError::InjectionError(long_msg.clone());
    let msg = err.to_string();

    assert!(msg.contains(&long_msg));
}

#[test]
fn test_session_error_unicode_message() {
    let err = SessionError::StateError("状态转换失败：无效操作".to_string());
    let msg = err.to_string();

    assert!(msg.contains("状态转换失败"));
}

// ==================== 配置边界值测试 ====================

#[test]
fn test_session_config_extreme_values() {
    // 测试极端配置值的序列化
    let json = r#"{
        "injection_strategy": "Auto",
        "auto_threshold": 0,
        "paste_delay_ms": 0,
        "pre_injection_delay_ms": 0,
        "auto_inject": true
    }"#;

    let config: SessionConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.auto_threshold, 0);
    assert_eq!(config.paste_delay_ms, 0);
    assert_eq!(config.pre_injection_delay_ms, 0);
}

#[test]
fn test_session_config_large_values() {
    let json = r#"{
        "injection_strategy": "Clipboard",
        "auto_threshold": 1000000,
        "paste_delay_ms": 60000,
        "pre_injection_delay_ms": 10000,
        "auto_inject": false
    }"#;

    let config: SessionConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.auto_threshold, 1000000);
    assert_eq!(config.paste_delay_ms, 60000);
    assert_eq!(config.pre_injection_delay_ms, 10000);
}
