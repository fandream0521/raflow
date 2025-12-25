use raflow_lib::state::{
    AppState, RecordingState, StateChangeEvent, StateError, StateManager, TransitionError,
    DEFAULT_PROCESSING_TIMEOUT_SECS,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_state_manager_default() {
    let manager = StateManager::default();
    assert!(manager.current().is_idle());
}

#[tokio::test]
async fn test_complete_workflow() {
    let manager = StateManager::new();

    // 1. Idle -> Connecting
    assert!(manager.transition(AppState::connecting()).is_ok());
    assert!(manager.current().is_connecting());

    // 2. Connecting -> Recording (Listening)
    assert!(manager
        .transition(AppState::recording_listening())
        .is_ok());
    assert!(manager.current().is_recording());

    // 3. Recording (Listening) -> Recording (Transcribing)
    assert!(manager
        .transition(AppState::recording_transcribing(
            "hello world".to_string(),
            0.95
        ))
        .is_ok());

    let current = manager.current();
    assert!(current.is_recording());
    if let Some(state) = current.recording_state() {
        assert!(state.is_transcribing());
        assert_eq!(state.partial_text(), Some("hello world"));
        assert_eq!(state.confidence(), Some(0.95));
    }

    // 4. Recording -> Processing
    assert!(manager.transition(AppState::processing()).is_ok());
    assert!(manager.current().is_processing());

    // 5. Processing -> Injecting
    assert!(manager.transition(AppState::injecting()).is_ok());
    assert!(manager.current().is_injecting());

    // 6. Injecting -> Idle
    assert!(manager.transition(AppState::idle()).is_ok());
    assert!(manager.current().is_idle());
}

#[tokio::test]
async fn test_error_recovery() {
    let manager = StateManager::new();

    // 进入连接状态
    manager.transition(AppState::connecting()).unwrap();

    // 连接失败 -> Error
    assert!(manager
        .transition(AppState::error("Connection timeout"))
        .is_ok());
    assert!(manager.current().is_error());
    assert_eq!(
        manager.current().error_message(),
        Some("Connection timeout")
    );

    // Error -> Idle (恢复)
    assert!(manager.transition(AppState::idle()).is_ok());
    assert!(manager.current().is_idle());
}

#[tokio::test]
async fn test_cancel_during_recording() {
    let manager = StateManager::new();

    // 进入录音状态
    manager.transition(AppState::connecting()).unwrap();
    manager.transition(AppState::recording_listening()).unwrap();
    manager
        .transition(AppState::recording_transcribing(
            "test".to_string(),
            0.8,
        ))
        .unwrap();

    // 用户取消 -> Idle
    assert!(manager.transition(AppState::idle()).is_ok());
    assert!(manager.current().is_idle());
}

#[tokio::test]
async fn test_processing_timeout() {
    let manager = StateManager::new();

    // 进入处理状态
    manager.transition(AppState::connecting()).unwrap();
    manager.transition(AppState::recording_listening()).unwrap();
    manager.transition(AppState::processing()).unwrap();

    // 超时 -> Idle
    assert!(manager.transition(AppState::idle()).is_ok());
    assert!(manager.current().is_idle());
}

#[tokio::test]
async fn test_invalid_transition_idle_to_processing() {
    let manager = StateManager::new();

    let result = manager.transition(AppState::processing());
    assert!(result.is_err());

    match result.unwrap_err() {
        StateError::InvalidTransition { from, to } => {
            assert!(from.is_idle());
            assert!(to.is_processing());
        }
        _ => panic!("Expected InvalidTransition error"),
    }
}

#[tokio::test]
async fn test_invalid_transition_connecting_to_injecting() {
    let manager = StateManager::new();

    manager.transition(AppState::connecting()).unwrap();

    let result = manager.transition(AppState::injecting());
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        StateError::InvalidTransition { .. }
    ));
}

#[tokio::test]
async fn test_state_listener_notifications() {
    let manager = Arc::new(StateManager::new());
    let mut rx = manager.subscribe().await;

    // 在后台任务中改变状态
    let manager_clone = Arc::clone(&manager);
    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        let _ = manager_clone.transition(AppState::connecting());

        sleep(Duration::from_millis(50)).await;
        let _ = manager_clone.transition(AppState::recording_listening());

        sleep(Duration::from_millis(50)).await;
        let _ = manager_clone.transition(AppState::processing());
    });

    // 接收状态变更通知
    let mut states = Vec::new();
    let timeout = Duration::from_millis(500);
    let start = tokio::time::Instant::now();

    while start.elapsed() < timeout {
        match tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Some(state)) => states.push(state),
            Ok(None) => break,
            Err(_) => break,
        }
    }

    // 验证收到了状态变更通知
    assert!(!states.is_empty(), "Should receive state notifications");

    // 验证状态序列
    assert!(states.iter().any(|s| s.is_connecting()));
    assert!(states.iter().any(|s| s.is_recording()));
    assert!(states.iter().any(|s| s.is_processing()));
}

#[tokio::test]
async fn test_multiple_listeners() {
    let manager = Arc::new(StateManager::new());

    let mut rx1 = manager.subscribe().await;
    let mut rx2 = manager.subscribe().await;
    let mut rx3 = manager.subscribe().await;

    assert_eq!(manager.listener_count().await, 3);

    // 改变状态
    let manager_clone = Arc::clone(&manager);
    tokio::spawn(async move {
        sleep(Duration::from_millis(10)).await;
        let _ = manager_clone.transition(AppState::connecting());
    });

    // 所有监听者都应该收到通知
    let timeout = Duration::from_millis(200);

    let result1 = tokio::time::timeout(timeout, rx1.recv()).await;
    let result2 = tokio::time::timeout(timeout, rx2.recv()).await;
    let result3 = tokio::time::timeout(timeout, rx3.recv()).await;

    assert!(result1.is_ok() && result1.unwrap().is_some());
    assert!(result2.is_ok() && result2.unwrap().is_some());
    assert!(result3.is_ok() && result3.unwrap().is_some());
}

#[tokio::test]
async fn test_listener_cleanup() {
    let manager = StateManager::new();

    let rx1 = manager.subscribe().await;
    let rx2 = manager.subscribe().await;

    assert_eq!(manager.listener_count().await, 2);

    // 关闭一个接收器
    drop(rx1);

    // 清理
    manager.cleanup_listeners().await;

    // 应该只剩一个
    assert_eq!(manager.listener_count().await, 1);

    // 关闭最后一个
    drop(rx2);
    manager.cleanup_listeners().await;
    assert_eq!(manager.listener_count().await, 0);
}

#[tokio::test]
async fn test_force_set_and_reset() {
    let manager = StateManager::new();

    // 使用 force_set 跳过验证
    manager.force_set(AppState::injecting());
    assert!(manager.current().is_injecting());

    // 可以强制设置任何状态
    manager.force_set(AppState::error("forced error".to_string()));
    assert!(manager.current().is_error());

    // reset 恢复到 Idle
    manager.reset();
    assert!(manager.current().is_idle());
}

#[tokio::test]
async fn test_concurrent_state_transitions() {
    let manager = Arc::new(StateManager::new());

    // 正常的状态转换序列
    manager.transition(AppState::connecting()).unwrap();

    let manager1 = Arc::clone(&manager);
    let manager2 = Arc::clone(&manager);

    let handle1 = tokio::spawn(async move {
        sleep(Duration::from_millis(10)).await;
        manager1.transition(AppState::recording_listening())
    });

    let handle2 = tokio::spawn(async move {
        sleep(Duration::from_millis(20)).await;
        manager2.transition(AppState::processing())
    });

    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    // 第一个转换应该成功
    assert!(result1.is_ok());

    // 第二个转换可能成功或失败，取决于时序
    // 如果 handle1 先执行，则 Recording -> Processing 是合法的
    // 这个测试主要验证并发安全性，不验证具体结果
    let _ = result2;
}

#[tokio::test]
async fn test_recording_state_details() {
    let listening = RecordingState::listening();
    assert!(listening.is_listening());
    assert!(!listening.is_transcribing());
    assert_eq!(listening.partial_text(), None);
    assert_eq!(listening.confidence(), None);

    let transcribing = RecordingState::transcribing("test text".to_string(), 0.85);
    assert!(!transcribing.is_listening());
    assert!(transcribing.is_transcribing());
    assert_eq!(transcribing.partial_text(), Some("test text"));
    assert_eq!(transcribing.confidence(), Some(0.85));
}

#[tokio::test]
async fn test_app_state_helper_methods() {
    let idle = AppState::idle();
    assert_eq!(idle.name(), "Idle");
    assert!(idle.is_idle());
    assert!(!idle.is_connecting());
    assert!(!idle.is_recording());
    assert!(!idle.is_processing());
    assert!(!idle.is_injecting());
    assert!(!idle.is_error());

    let connecting = AppState::connecting();
    assert_eq!(connecting.name(), "Connecting");
    assert!(connecting.is_connecting());

    let recording = AppState::recording_transcribing("hello".to_string(), 0.9);
    assert_eq!(recording.name(), "Recording::Transcribing");
    assert!(recording.is_recording());
    assert!(recording.recording_state().is_some());
    assert_eq!(
        recording.recording_state().unwrap().partial_text(),
        Some("hello")
    );

    let error = AppState::error("test error");
    assert_eq!(error.name(), "Error");
    assert!(error.is_error());
    assert_eq!(error.error_message(), Some("test error"));
}

#[tokio::test]
async fn test_state_transitions_with_listener() {
    let manager = Arc::new(StateManager::new());
    let mut rx = manager.subscribe().await;

    let manager_clone = Arc::clone(&manager);

    // 执行完整的状态转换序列
    tokio::spawn(async move {
        sleep(Duration::from_millis(10)).await;
        let _ = manager_clone.transition(AppState::connecting());

        sleep(Duration::from_millis(10)).await;
        let _ = manager_clone.transition(AppState::recording_listening());

        sleep(Duration::from_millis(10)).await;
        let _ = manager_clone.transition(AppState::recording_transcribing(
            "partial".to_string(),
            0.8,
        ));

        sleep(Duration::from_millis(10)).await;
        let _ = manager_clone.transition(AppState::processing());

        sleep(Duration::from_millis(10)).await;
        let _ = manager_clone.transition(AppState::injecting());

        sleep(Duration::from_millis(10)).await;
        let _ = manager_clone.transition(AppState::idle());
    });

    // 收集所有状态变更
    let mut received_states = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_millis(500);

    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(50), rx.recv()).await {
            Ok(Some(state)) => {
                received_states.push(state);
            }
            Ok(None) => break,
            Err(_) => continue,
        }
    }

    // 验证至少收到了一些状态变更
    assert!(
        !received_states.is_empty(),
        "Should receive state change notifications"
    );
}

#[test]
fn test_state_error_display() {
    let error = StateError::InvalidTransition {
        from: AppState::idle(),
        to: AppState::processing(),
    };
    let msg = format!("{}", error);
    assert!(msg.contains("Invalid state transition"));
    assert!(msg.contains("Idle"));
    assert!(msg.contains("Processing"));
}

#[test]
fn test_recording_state_equality() {
    let listening1 = RecordingState::listening();
    let listening2 = RecordingState::listening();
    assert_eq!(listening1, listening2);

    let transcribing1 = RecordingState::transcribing("test".to_string(), 0.9);
    let transcribing2 = RecordingState::transcribing("test".to_string(), 0.9);
    assert_eq!(transcribing1, transcribing2);

    let transcribing3 = RecordingState::transcribing("other".to_string(), 0.9);
    assert_ne!(transcribing1, transcribing3);
}

#[test]
fn test_app_state_equality() {
    let idle1 = AppState::idle();
    let idle2 = AppState::idle();
    assert_eq!(idle1, idle2);

    let error1 = AppState::error("test");
    let error2 = AppState::error("test");
    assert_eq!(error1, error2);

    let error3 = AppState::error("other");
    assert_ne!(error1, error3);
}

// ==================== P2-T4: State Transition Tests ====================

#[test]
fn test_state_change_event_all_variants() {
    // Idle
    let event = StateChangeEvent::from(&AppState::idle());
    assert!(event.is_idle);
    assert!(!event.is_connecting);
    assert!(!event.is_recording);
    assert!(!event.is_processing);
    assert!(!event.is_injecting);
    assert!(!event.is_error);

    // Connecting
    let event = StateChangeEvent::from(&AppState::connecting());
    assert!(!event.is_idle);
    assert!(event.is_connecting);

    // Recording::Listening
    let event = StateChangeEvent::from(&AppState::recording_listening());
    assert!(event.is_recording);
    assert!(event.partial_text.is_none());

    // Recording::Transcribing
    let event = StateChangeEvent::from(&AppState::recording_transcribing(
        "partial text".to_string(),
        0.9,
    ));
    assert!(event.is_recording);
    assert_eq!(event.partial_text, Some("partial text".to_string()));

    // Processing
    let event = StateChangeEvent::from(&AppState::processing());
    assert!(event.is_processing);

    // Injecting
    let event = StateChangeEvent::from(&AppState::injecting());
    assert!(event.is_injecting);

    // Error
    let event = StateChangeEvent::from(&AppState::error("test error"));
    assert!(event.is_error);
    assert_eq!(event.error_message, Some("test error".to_string()));
}

#[test]
fn test_state_change_event_json_serialization() {
    let state = AppState::recording_transcribing("hello world".to_string(), 0.95);
    let event = StateChangeEvent::from(&state);

    // Serialize to JSON
    let json = serde_json::to_string(&event).expect("Should serialize to JSON");

    // Deserialize back
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should parse JSON");

    assert_eq!(parsed["state"], "Recording::Transcribing");
    assert_eq!(parsed["is_recording"], true);
    assert_eq!(parsed["is_idle"], false);
    assert_eq!(parsed["partial_text"], "hello world");
}

#[test]
fn test_transition_error_variants() {
    // TransitionFailed
    let err = TransitionError::TransitionFailed("invalid state".to_string());
    assert!(err.to_string().contains("invalid state"));

    // InvalidState
    let err = TransitionError::InvalidState {
        current: "Processing".to_string(),
        action: "cancel".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("Processing"));
    assert!(msg.contains("cancel"));
}

#[test]
fn test_default_processing_timeout_value() {
    // Verify the default timeout is reasonable (30 seconds)
    assert_eq!(DEFAULT_PROCESSING_TIMEOUT_SECS, 30);
    assert!(DEFAULT_PROCESSING_TIMEOUT_SECS > 10); // Not too short
    assert!(DEFAULT_PROCESSING_TIMEOUT_SECS <= 60); // Not too long
}

#[test]
fn test_app_state_serialization() {
    // AppState should be serializable for Tauri events
    let states = vec![
        AppState::idle(),
        AppState::connecting(),
        AppState::recording_listening(),
        AppState::recording_transcribing("test".to_string(), 0.8),
        AppState::processing(),
        AppState::injecting(),
        AppState::error("error message".to_string()),
    ];

    for state in states {
        let json = serde_json::to_string(&state).expect("Should serialize AppState");
        assert!(!json.is_empty());
    }
}

#[test]
fn test_recording_state_serialization() {
    let listening = RecordingState::listening();
    let json = serde_json::to_string(&listening).expect("Should serialize RecordingState");
    assert!(json.contains("Listening"));

    let transcribing = RecordingState::transcribing("test".to_string(), 0.9);
    let json = serde_json::to_string(&transcribing).expect("Should serialize RecordingState");
    assert!(json.contains("Transcribing"));
    assert!(json.contains("test"));
}
