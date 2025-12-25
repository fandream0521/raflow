//! 键盘模拟集成测试
//!
//! 测试键盘模拟模块的功能

use raflow_lib::input::{InputError, KeyboardSimulator};

// ==================== KeyboardSimulator 创建测试 ====================

#[test]
fn test_keyboard_simulator_creation() {
    // 在没有显示服务器的环境下可能失败
    // 测试函数不会 panic
    let result = KeyboardSimulator::new();

    match result {
        Ok(_keyboard) => {
            // 成功创建键盘模拟器
        }
        Err(e) => {
            // 在无头环境中可能失败，验证错误类型正确
            match e {
                InputError::KeyboardSimulationFailed(_) => {
                    // 这是预期的错误
                }
                _ => panic!("Unexpected error type: {:?}", e),
            }
        }
    }
}

// ==================== 文本输入测试 ====================

#[test]
fn test_type_text_empty_string() {
    // 空字符串应该成功
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        let result = keyboard.type_text("");
        assert!(result.is_ok(), "Empty string should succeed");
    }
}

#[test]
fn test_type_text_integration() {
    // 这个测试在有窗口系统的环境下运行
    // 由于测试环境可能没有窗口，我们只检查函数不会 panic
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        // 注意：这个测试会实际模拟键盘输入
        // 在 CI 环境中可能需要跳过
        let result = keyboard.type_text("test");

        // 不断言成功，因为在某些环境中可能失败
        match result {
            Ok(()) => {
                // 成功输入
            }
            Err(e) => {
                // 验证错误类型
                match e {
                    InputError::KeyboardSimulationFailed(_) => {
                        // 这是预期的错误
                    }
                    _ => panic!("Unexpected error type: {:?}", e),
                }
            }
        }
    }
}

// ==================== 粘贴操作测试 ====================

#[test]
fn test_paste_integration() {
    // 测试粘贴操作不会 panic
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        let result = keyboard.paste();

        // 不断言成功，因为在某些环境中可能失败
        match result {
            Ok(()) => {
                // 成功粘贴
            }
            Err(e) => {
                match e {
                    InputError::KeyboardSimulationFailed(_) => {
                        // 这是预期的错误
                    }
                    _ => panic!("Unexpected error type: {:?}", e),
                }
            }
        }
    }
}

// ==================== 复制操作测试 ====================

#[test]
fn test_copy_integration() {
    // 测试复制操作不会 panic
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        let result = keyboard.copy();

        match result {
            Ok(()) => {
                // 成功复制
            }
            Err(e) => {
                match e {
                    InputError::KeyboardSimulationFailed(_) => {
                        // 这是预期的错误
                    }
                    _ => panic!("Unexpected error type: {:?}", e),
                }
            }
        }
    }
}

// ==================== 全选操作测试 ====================

#[test]
fn test_select_all_integration() {
    // 测试全选操作不会 panic
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        let result = keyboard.select_all();

        match result {
            Ok(()) => {
                // 成功全选
            }
            Err(e) => {
                match e {
                    InputError::KeyboardSimulationFailed(_) => {
                        // 这是预期的错误
                    }
                    _ => panic!("Unexpected error type: {:?}", e),
                }
            }
        }
    }
}

// ==================== 特殊按键测试 ====================

#[test]
fn test_press_enter_integration() {
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        let result = keyboard.press_enter();

        match result {
            Ok(()) => {}
            Err(InputError::KeyboardSimulationFailed(_)) => {}
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }
}

#[test]
fn test_press_escape_integration() {
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        let result = keyboard.press_escape();

        match result {
            Ok(()) => {}
            Err(InputError::KeyboardSimulationFailed(_)) => {}
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }
}

#[test]
fn test_press_tab_integration() {
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        let result = keyboard.press_tab();

        match result {
            Ok(()) => {}
            Err(InputError::KeyboardSimulationFailed(_)) => {}
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }
}

#[test]
fn test_press_backspace_integration() {
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        let result = keyboard.press_backspace();

        match result {
            Ok(()) => {}
            Err(InputError::KeyboardSimulationFailed(_)) => {}
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }
}

#[test]
fn test_press_delete_integration() {
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        let result = keyboard.press_delete();

        match result {
            Ok(()) => {}
            Err(InputError::KeyboardSimulationFailed(_)) => {}
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }
}

// ==================== 按键控制测试 ====================

#[test]
fn test_press_and_release_key() {
    use enigo::Key;

    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        // 测试按下
        let press_result = keyboard.press_key(Key::Shift);
        match press_result {
            Ok(()) => {
                // 测试释放
                let release_result = keyboard.release_key(Key::Shift);
                match release_result {
                    Ok(()) => {}
                    Err(InputError::KeyboardSimulationFailed(_)) => {}
                    Err(e) => panic!("Unexpected error type: {:?}", e),
                }
            }
            Err(InputError::KeyboardSimulationFailed(_)) => {}
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }
}

#[test]
fn test_click_key() {
    use enigo::Key;

    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        let result = keyboard.click_key(Key::Space);

        match result {
            Ok(()) => {}
            Err(InputError::KeyboardSimulationFailed(_)) => {}
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }
}

// ==================== 错误类型测试 ====================

#[test]
fn test_input_error_keyboard_simulation_failed() {
    let error = InputError::KeyboardSimulationFailed("test reason".to_string());
    let msg = error.to_string();
    assert!(msg.contains("test reason"));
    assert!(msg.to_lowercase().contains("keyboard") || msg.to_lowercase().contains("simulation"));
}

#[test]
fn test_input_error_equality() {
    let error1 = InputError::KeyboardSimulationFailed("error".to_string());
    let error2 = InputError::KeyboardSimulationFailed("error".to_string());
    assert_eq!(error1, error2);

    let error3 = InputError::KeyboardSimulationFailed("other".to_string());
    assert_ne!(error1, error3);
}

// ==================== Unicode 文本测试 ====================

#[test]
fn test_type_text_unicode() {
    // 测试 Unicode 文本
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        // 中文
        let result = keyboard.type_text("你好世界");
        match result {
            Ok(()) => {}
            Err(InputError::KeyboardSimulationFailed(_)) => {}
            Err(e) => panic!("Unexpected error type for Chinese: {:?}", e),
        }

        // 日文
        let result = keyboard.type_text("こんにちは");
        match result {
            Ok(()) => {}
            Err(InputError::KeyboardSimulationFailed(_)) => {}
            Err(e) => panic!("Unexpected error type for Japanese: {:?}", e),
        }

        // 表情符号
        let result = keyboard.type_text("🎉👍");
        match result {
            Ok(()) => {}
            Err(InputError::KeyboardSimulationFailed(_)) => {}
            Err(e) => panic!("Unexpected error type for emoji: {:?}", e),
        }
    }
}

// ==================== 多行文本测试 ====================

#[test]
fn test_type_text_multiline() {
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        let multiline_text = "Line 1\nLine 2\nLine 3";
        let result = keyboard.type_text(multiline_text);

        match result {
            Ok(()) => {}
            Err(InputError::KeyboardSimulationFailed(_)) => {}
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }
}

// ==================== 长文本测试 ====================

#[test]
fn test_type_text_long_string() {
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        let long_text = "a".repeat(100);
        let result = keyboard.type_text(&long_text);

        match result {
            Ok(()) => {}
            Err(InputError::KeyboardSimulationFailed(_)) => {}
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }
}

// ==================== 特殊字符测试 ====================

#[test]
fn test_type_text_special_characters() {
    if let Ok(mut keyboard) = KeyboardSimulator::new() {
        // 特殊字符
        let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
        let result = keyboard.type_text(special_chars);

        match result {
            Ok(()) => {}
            Err(InputError::KeyboardSimulationFailed(_)) => {}
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }
}
