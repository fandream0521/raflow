//! 剪贴板操作集成测试
//!
//! 测试剪贴板模块的功能
//!
//! 注意：由于 ClipboardManager 需要 Tauri AppHandle，
//! 大多数功能测试需要在 Tauri 应用环境中进行。
//! 这里主要测试错误类型和 API 结构。

use raflow_lib::input::InputError;

// ==================== 错误类型测试 ====================

#[test]
fn test_clipboard_error_display() {
    let error = InputError::ClipboardFailed("test error".to_string());
    let msg = error.to_string();
    assert!(msg.contains("test error"));
    assert!(
        msg.to_lowercase().contains("clipboard")
            || msg.to_lowercase().contains("failed")
    );
}

#[test]
fn test_clipboard_error_equality() {
    let error1 = InputError::ClipboardFailed("error".to_string());
    let error2 = InputError::ClipboardFailed("error".to_string());
    assert_eq!(error1, error2);

    let error3 = InputError::ClipboardFailed("other".to_string());
    assert_ne!(error1, error3);
}

#[test]
fn test_clipboard_error_clone() {
    let error = InputError::ClipboardFailed("clipboard error".to_string());
    let cloned = error.clone();
    assert_eq!(error, cloned);
}

#[test]
fn test_clipboard_error_debug() {
    let error = InputError::ClipboardFailed("debug test".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("ClipboardFailed"));
    assert!(debug_str.contains("debug test"));
}

// ==================== 错误类型区分测试 ====================

#[test]
fn test_clipboard_error_differs_from_other_errors() {
    let clipboard_error = InputError::ClipboardFailed("clipboard".to_string());
    let keyboard_error = InputError::KeyboardSimulationFailed("keyboard".to_string());
    let injection_error = InputError::InjectionFailed("injection".to_string());

    assert_ne!(clipboard_error, keyboard_error);
    assert_ne!(clipboard_error, injection_error);
}

#[test]
fn test_clipboard_error_with_various_messages() {
    // 空消息
    let error1 = InputError::ClipboardFailed("".to_string());
    assert!(error1.to_string().len() > 0);

    // 长消息
    let long_msg = "a".repeat(1000);
    let error2 = InputError::ClipboardFailed(long_msg.clone());
    assert!(error2.to_string().contains(&long_msg));

    // Unicode 消息
    let unicode_msg = "剪贴板错误：无法读取".to_string();
    let error3 = InputError::ClipboardFailed(unicode_msg.clone());
    assert!(error3.to_string().contains(&unicode_msg));

    // 特殊字符
    let special_msg = "Error: <>&\"'".to_string();
    let error4 = InputError::ClipboardFailed(special_msg.clone());
    assert!(error4.to_string().contains(&special_msg));
}

// ==================== API 存在性测试 ====================

#[test]
fn test_clipboard_module_exports() {
    // 验证模块导出了正确的类型
    // 这些测试确保 API 稳定性

    // ClipboardManager 类型存在
    use raflow_lib::input::clipboard::ClipboardManager;

    // 便捷函数存在
    use raflow_lib::input::clipboard::{read_from_clipboard, write_to_clipboard};

    // 验证类型可以被引用
    fn _use_types(_cm: &ClipboardManager, _app: &tauri::AppHandle) {
        // 类型存在即可
    }

    fn _use_convenience_fns(app: &tauri::AppHandle) {
        let _ = read_from_clipboard(app);
        let _ = write_to_clipboard(app, "test");
    }
}

#[test]
fn test_clipboard_manager_methods_exist() {
    // 验证 ClipboardManager 的方法存在
    // 通过编译时检查确保 API 稳定

    use raflow_lib::input::clipboard::ClipboardManager;

    // 这个函数的存在验证了所有方法签名
    fn _verify_methods(app: &tauri::AppHandle) {
        // new 方法
        let mut clipboard = ClipboardManager::new(app);

        // save 方法
        let _ = clipboard.save();

        // write 方法
        let _ = clipboard.write("test");

        // read 方法
        let _ = clipboard.read();

        // restore 方法
        let _ = clipboard.restore();

        // has_saved_content 方法
        let _ = clipboard.has_saved_content();

        // get_saved_content 方法
        let _ = clipboard.get_saved_content();

        // clear_saved 方法
        clipboard.clear_saved();

        // clear 方法
        let _ = clipboard.clear();
    }
}

// ==================== Re-export 测试 ====================

#[test]
fn test_clipboard_reexports() {
    // 验证从 input 模块的 re-export
    use raflow_lib::input::{read_from_clipboard, write_to_clipboard, ClipboardManager};

    // 验证类型可以被引用
    fn _use_reexports(app: &tauri::AppHandle) {
        let _ = ClipboardManager::new(app);
        let _ = read_from_clipboard(app);
        let _ = write_to_clipboard(app, "test");
    }
}

// ==================== InputResult 兼容性测试 ====================

#[test]
fn test_input_result_with_clipboard_error() {
    use raflow_lib::input::InputResult;

    fn returns_clipboard_error() -> InputResult<()> {
        Err(InputError::ClipboardFailed("test".to_string()))
    }

    let result = returns_clipboard_error();
    assert!(result.is_err());

    if let Err(InputError::ClipboardFailed(msg)) = result {
        assert_eq!(msg, "test");
    } else {
        panic!("Expected ClipboardFailed error");
    }
}

#[test]
fn test_input_result_with_clipboard_success() {
    use raflow_lib::input::InputResult;

    fn returns_success() -> InputResult<String> {
        Ok("clipboard content".to_string())
    }

    let result = returns_success();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "clipboard content");
}

// ==================== 错误转换测试 ====================

#[test]
fn test_clipboard_error_into_string() {
    let error = InputError::ClipboardFailed("conversion test".to_string());
    let error_string: String = error.to_string();
    assert!(error_string.contains("conversion test"));
}

#[test]
fn test_clipboard_error_pattern_matching() {
    let error = InputError::ClipboardFailed("pattern test".to_string());

    match error {
        InputError::ClipboardFailed(msg) => {
            assert_eq!(msg, "pattern test");
        }
        _ => panic!("Wrong error variant"),
    }
}

// ==================== 文档测试 ====================

/// 验证模块文档中的代码示例编译
/// 这个测试确保文档中的示例保持最新
#[test]
fn test_documentation_compiles() {
    // 从文档示例改编的代码
    use raflow_lib::input::InputError;

    // 创建错误
    let _error = InputError::ClipboardFailed("example error".to_string());

    // 错误可以转换为字符串
    let _msg = _error.to_string();

    // 错误可以克隆
    let _cloned = _error.clone();

    // 错误可以比较
    let _same = _error == _cloned;
}

// ==================== 边界条件测试 ====================

#[test]
fn test_clipboard_error_with_newlines() {
    let msg_with_newlines = "Line 1\nLine 2\nLine 3".to_string();
    let error = InputError::ClipboardFailed(msg_with_newlines.clone());
    assert!(error.to_string().contains(&msg_with_newlines));
}

#[test]
fn test_clipboard_error_with_tabs() {
    let msg_with_tabs = "Col1\tCol2\tCol3".to_string();
    let error = InputError::ClipboardFailed(msg_with_tabs.clone());
    assert!(error.to_string().contains(&msg_with_tabs));
}

#[test]
fn test_clipboard_error_preserves_whitespace() {
    let msg_with_spaces = "  leading and trailing spaces  ".to_string();
    let error = InputError::ClipboardFailed(msg_with_spaces.clone());
    assert!(error.to_string().contains(&msg_with_spaces));
}
