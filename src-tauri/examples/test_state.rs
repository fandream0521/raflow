//! 状态机测试
//!
//! 测试状态管理和状态转换功能
//!
//! 运行: cargo run --example test_state

use raflow_lib::state::{AppState, RecordingState, StateManager};

fn main() {
    println!("=== 状态机测试 ===\n");

    // 1. 创建状态管理器
    println!("1. 创建状态管理器");
    println!("{}", "-".repeat(40));

    let manager = StateManager::new();
    println!("  初始状态: {:?}", *manager.current());
    println!();

    // 2. 测试有效状态转换
    println!("2. 测试有效状态转换");
    println!("{}", "-".repeat(40));

    let transitions = vec![
        AppState::Connecting,
        AppState::Recording(RecordingState::Listening),
        AppState::Recording(RecordingState::transcribing("测试文本".to_string(), 0.8)),
        AppState::Processing,
        AppState::Injecting,
        AppState::Idle,
    ];

    for new_state in transitions {
        let current = manager.current();
        match manager.transition(new_state.clone()) {
            Ok(()) => {
                println!("  OK {:?} -> {:?}", current.name(), new_state.name());
            }
            Err(e) => {
                println!("  !! {:?} -> {:?}: {}", current.name(), new_state.name(), e);
            }
        }
    }
    println!();

    // 3. 测试无效状态转换
    println!("3. 测试无效状态转换");
    println!("{}", "-".repeat(40));

    // 从 Idle 不能直接到 Recording
    let invalid_transitions = vec![
        (
            AppState::Idle,
            AppState::Recording(RecordingState::Listening),
        ),
        (AppState::Idle, AppState::Processing),
        (AppState::Connecting, AppState::Injecting),
    ];

    for (from, to) in invalid_transitions {
        manager.force_set(from.clone());
        match manager.transition(to.clone()) {
            Ok(()) => {
                println!("  ?? {:?} -> {:?}: 意外成功", from.name(), to.name());
            }
            Err(_) => {
                println!("  OK {:?} -> {:?}: 正确拒绝", from.name(), to.name());
            }
        }
    }
    println!();

    // 4. 测试状态属性
    println!("4. 测试状态属性");
    println!("{}", "-".repeat(40));

    let states = vec![
        AppState::Idle,
        AppState::Connecting,
        AppState::Recording(RecordingState::Listening),
        AppState::Recording(RecordingState::transcribing(
            "正在说话...".to_string(),
            0.95,
        )),
        AppState::Processing,
        AppState::Injecting,
        AppState::Error("测试错误".to_string()),
    ];

    for state in states {
        println!("  {:?}", state.name());
        println!("    is_idle: {}", state.is_idle());
        println!("    is_recording: {}", state.is_recording());
        println!("    is_error: {}", state.is_error());
        if let Some(rec_state) = state.recording_state() {
            println!("    recording_state: {:?}", rec_state);
            if let Some(text) = rec_state.partial_text() {
                println!("    partial_text: {}", text);
            }
        }
        if let Some(msg) = state.error_message() {
            println!("    error_message: {}", msg);
        }
        println!();
    }

    // 5. 测试错误状态恢复
    println!("5. 测试错误状态恢复");
    println!("{}", "-".repeat(40));

    manager.force_set(AppState::Recording(RecordingState::Listening));
    let error_state = AppState::Error("测试错误".to_string());
    let _ = manager.transition(error_state.clone());
    println!("  当前状态: {:?}", manager.current().name());

    // 从错误状态恢复
    let _ = manager.transition(AppState::Idle);
    println!("  恢复后状态: {:?}", manager.current().name());

    println!("\n测试完成!");
}
