//! 窗口检测集成测试
//!
//! 测试窗口检测模块的功能

use raflow_lib::input::{
    format_window_info, get_focused_app_name, get_focused_window, get_focused_window_title,
    has_focused_window, is_text_input_context, InputError, WindowInfo,
};

// ==================== WindowInfo 测试 ====================

#[test]
fn test_window_info_creation() {
    let window = WindowInfo {
        app_name: "Test App".to_string(),
        title: "Test Window".to_string(),
        process_id: 1234,
        exec_name: "test".to_string(),
        exec_path: "/usr/bin/test".to_string(),
        window_id: 5678,
    };

    assert_eq!(window.app_name, "Test App");
    assert_eq!(window.title, "Test Window");
    assert_eq!(window.process_id, 1234);
    assert_eq!(window.exec_name, "test");
    assert_eq!(window.exec_path, "/usr/bin/test");
    assert_eq!(window.window_id, 5678);
}

#[test]
fn test_window_info_equality() {
    let window1 = WindowInfo {
        app_name: "App".to_string(),
        title: "Title".to_string(),
        process_id: 100,
        exec_name: "app".to_string(),
        exec_path: "/path".to_string(),
        window_id: 200,
    };

    let window2 = window1.clone();
    assert_eq!(window1, window2);

    let window3 = WindowInfo {
        app_name: "Other App".to_string(),
        title: "Title".to_string(),
        process_id: 100,
        exec_name: "app".to_string(),
        exec_path: "/path".to_string(),
        window_id: 200,
    };
    assert_ne!(window1, window3);
}

#[test]
fn test_window_info_is_app_multiple_matches() {
    let window = WindowInfo {
        app_name: "Microsoft Word".to_string(),
        title: "Document.docx".to_string(),
        process_id: 1,
        exec_name: "word".to_string(),
        exec_path: "".to_string(),
        window_id: 1,
    };

    // 应该匹配 "word"
    assert!(window.is_app(&["word"]));
    assert!(window.is_app(&["microsoft"]));
    assert!(window.is_app(&["notepad", "word"]));

    // 不应该匹配
    assert!(!window.is_app(&["excel"]));
    assert!(!window.is_app(&["notepad", "chrome"]));
}

#[test]
fn test_window_info_is_app_empty_list() {
    let window = WindowInfo {
        app_name: "Any App".to_string(),
        title: "Title".to_string(),
        process_id: 1,
        exec_name: "app".to_string(),
        exec_path: "".to_string(),
        window_id: 1,
    };

    // 空列表应该返回 false
    assert!(!window.is_app(&[]));
}

#[test]
fn test_window_info_title_contains_special_chars() {
    let window = WindowInfo {
        app_name: "VS Code".to_string(),
        title: "main.rs - [RaFlow] (Running)".to_string(),
        process_id: 1,
        exec_name: "code".to_string(),
        exec_path: "".to_string(),
        window_id: 1,
    };

    assert!(window.title_contains("[raflow]"));
    assert!(window.title_contains("(Running)"));
    assert!(window.title_contains("main.rs"));
}

// ==================== 窗口检测功能测试 ====================

#[test]
fn test_get_focused_window_integration() {
    // 这个测试在有窗口系统的环境下运行
    // 由于测试环境可能没有窗口，我们只检查函数不会 panic
    let result = get_focused_window();

    // 结果要么成功要么是特定错误类型
    match result {
        Ok(window) => {
            // 如果成功，验证基本字段不为空
            assert!(!window.app_name.is_empty() || !window.title.is_empty());
            assert!(window.process_id > 0 || window.window_id > 0);
        }
        Err(e) => {
            // 验证错误类型
            match e {
                InputError::NoFocusedWindow => {
                    // 这是预期的错误
                }
                InputError::WindowDetectionFailed(_) => {
                    // 这也是可接受的错误
                }
                _ => panic!("Unexpected error type: {:?}", e),
            }
        }
    }
}

#[test]
fn test_has_focused_window_integration() {
    // 这个函数不应该 panic
    let _ = has_focused_window();
}

#[test]
fn test_is_text_input_context_integration() {
    // 这个函数不应该 panic
    let _ = is_text_input_context();
}

#[test]
fn test_get_focused_app_name_integration() {
    // 这个函数不应该 panic
    let result = get_focused_app_name();

    // 结果要么是 Some 要么是 None
    if let Some(name) = result {
        // 如果有结果，应该是非空字符串或空字符串
        assert!(name.len() >= 0);
    }
}

#[test]
fn test_get_focused_window_title_integration() {
    // 这个函数不应该 panic
    let result = get_focused_window_title();

    // 结果要么是 Some 要么是 None
    if let Some(title) = result {
        // 标题可以是空字符串（某些系统权限问题）
        assert!(title.len() >= 0);
    }
}

// ==================== 格式化测试 ====================

#[test]
fn test_format_window_info_all_fields() {
    let window = WindowInfo {
        app_name: "TestApp".to_string(),
        title: "TestTitle".to_string(),
        process_id: 12345,
        exec_name: "testexec".to_string(),
        exec_path: "/path/to/test".to_string(),
        window_id: 67890,
    };

    let formatted = format_window_info(&window);

    assert!(formatted.contains("TestApp"));
    assert!(formatted.contains("TestTitle"));
    assert!(formatted.contains("12345"));
    assert!(formatted.contains("testexec"));
    assert!(formatted.contains("/path/to/test"));
}

#[test]
fn test_format_window_info_empty_fields() {
    let window = WindowInfo {
        app_name: "".to_string(),
        title: "".to_string(),
        process_id: 0,
        exec_name: "".to_string(),
        exec_path: "".to_string(),
        window_id: 0,
    };

    let formatted = format_window_info(&window);

    // 应该生成有效的字符串，即使字段为空
    assert!(formatted.contains("Window"));
    assert!(formatted.contains("0"));
}

// ==================== 错误类型测试 ====================

#[test]
fn test_input_error_no_focused_window() {
    let error = InputError::NoFocusedWindow;
    let msg = error.to_string();
    assert!(msg.to_lowercase().contains("window"));
}

#[test]
fn test_input_error_window_detection_failed() {
    let error = InputError::WindowDetectionFailed("test reason".to_string());
    let msg = error.to_string();
    assert!(msg.contains("test reason"));
}

#[test]
fn test_input_error_permission_denied() {
    let error = InputError::PermissionDenied;
    let msg = error.to_string();
    assert!(msg.to_lowercase().contains("permission"));
}

#[test]
fn test_input_error_injection_failed() {
    let error = InputError::InjectionFailed("injection error".to_string());
    let msg = error.to_string();
    assert!(msg.contains("injection error"));
}

#[test]
fn test_input_error_clipboard_failed() {
    let error = InputError::ClipboardFailed("clipboard error".to_string());
    let msg = error.to_string();
    assert!(msg.contains("clipboard error"));
}

// ==================== 文本输入上下文测试 ====================

#[test]
fn test_text_input_apps_editors() {
    let editors = vec![
        ("Visual Studio Code", true),
        ("Sublime Text", true),
        ("Notepad++", true),
        ("vim", true),
        ("Emacs", true),
    ];

    for (app_name, expected) in editors {
        let window = WindowInfo {
            app_name: app_name.to_string(),
            title: "test.txt".to_string(),
            process_id: 1,
            exec_name: "".to_string(),
            exec_path: "".to_string(),
            window_id: 1,
        };

        let is_text = window.is_app(&["code", "sublime", "notepad", "vim", "emacs"]);
        assert_eq!(
            is_text, expected,
            "Failed for app: {}, expected: {}",
            app_name, expected
        );
    }
}

#[test]
fn test_text_input_apps_browsers() {
    let browsers = vec![
        ("Google Chrome", true),
        ("Mozilla Firefox", true),
        ("Safari", true),
        ("Microsoft Edge", true),
    ];

    for (app_name, expected) in browsers {
        let window = WindowInfo {
            app_name: app_name.to_string(),
            title: "Web Page".to_string(),
            process_id: 1,
            exec_name: "".to_string(),
            exec_path: "".to_string(),
            window_id: 1,
        };

        let is_browser = window.is_app(&["chrome", "firefox", "safari", "edge"]);
        assert_eq!(
            is_browser, expected,
            "Failed for app: {}, expected: {}",
            app_name, expected
        );
    }
}

#[test]
fn test_text_input_apps_communication() {
    let apps = vec![
        ("Slack", true),
        ("Discord", true),
        ("Microsoft Teams", true),
        ("微信", true),
        ("WeChat", true),
    ];

    for (app_name, expected) in apps {
        let window = WindowInfo {
            app_name: app_name.to_string(),
            title: "Chat".to_string(),
            process_id: 1,
            exec_name: "".to_string(),
            exec_path: "".to_string(),
            window_id: 1,
        };

        let is_comm =
            window.is_app(&["slack", "discord", "teams", "微信", "wechat"]);
        assert_eq!(
            is_comm, expected,
            "Failed for app: {}, expected: {}",
            app_name, expected
        );
    }
}

#[test]
fn test_non_text_input_apps() {
    let non_text_apps = vec![
        "System Preferences",
        "Finder",
        "Activity Monitor",
        "Calculator",
        "Preview",
    ];

    for app_name in non_text_apps {
        let window = WindowInfo {
            app_name: app_name.to_string(),
            title: "Window".to_string(),
            process_id: 1,
            exec_name: "".to_string(),
            exec_path: "".to_string(),
            window_id: 1,
        };

        // 这些应用不应该匹配常见的文本输入应用
        let is_text = window.is_app(&["code", "chrome", "word", "slack"]);
        assert!(
            !is_text,
            "App '{}' should not be detected as text input app",
            app_name
        );
    }
}
