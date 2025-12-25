//! 文本注入器集成测试
//!
//! 测试注入器模块的功能
//!
//! 注意：由于 TextInjector 需要 Tauri AppHandle，
//! 大多数功能测试需要在 Tauri 应用环境中进行。
//! 这里主要测试枚举类型、结果类型和 API 结构。

use raflow_lib::input::{
    InjectionResult, InjectionStrategy, AUTO_STRATEGY_THRESHOLD, PASTE_DELAY_MS,
};

// ==================== InjectionStrategy 测试 ====================

#[test]
fn test_injection_strategy_default() {
    let strategy = InjectionStrategy::default();
    assert_eq!(strategy, InjectionStrategy::Auto);
}

#[test]
fn test_injection_strategy_variants() {
    // 测试所有变体都存在
    let _auto = InjectionStrategy::Auto;
    let _keyboard = InjectionStrategy::Keyboard;
    let _clipboard = InjectionStrategy::Clipboard;
    let _clipboard_only = InjectionStrategy::ClipboardOnly;
}

#[test]
fn test_injection_strategy_equality() {
    assert_eq!(InjectionStrategy::Auto, InjectionStrategy::Auto);
    assert_eq!(InjectionStrategy::Keyboard, InjectionStrategy::Keyboard);
    assert_eq!(InjectionStrategy::Clipboard, InjectionStrategy::Clipboard);
    assert_eq!(
        InjectionStrategy::ClipboardOnly,
        InjectionStrategy::ClipboardOnly
    );

    assert_ne!(InjectionStrategy::Auto, InjectionStrategy::Keyboard);
    assert_ne!(InjectionStrategy::Keyboard, InjectionStrategy::Clipboard);
    assert_ne!(InjectionStrategy::Clipboard, InjectionStrategy::ClipboardOnly);
}

#[test]
fn test_injection_strategy_clone() {
    let strategy = InjectionStrategy::Clipboard;
    let cloned = strategy.clone();
    assert_eq!(strategy, cloned);
}

#[test]
fn test_injection_strategy_copy() {
    let strategy = InjectionStrategy::Keyboard;
    let copied = strategy; // Copy, not move
    assert_eq!(strategy, copied);
}

#[test]
fn test_injection_strategy_debug() {
    let strategy = InjectionStrategy::Auto;
    let debug_str = format!("{:?}", strategy);
    assert!(debug_str.contains("Auto"));
}

// ==================== InjectionStrategy 显示名称测试 ====================

#[test]
fn test_injection_strategy_display_name() {
    assert_eq!(InjectionStrategy::Auto.display_name(), "自动");
    assert_eq!(InjectionStrategy::Keyboard.display_name(), "键盘模拟");
    assert_eq!(InjectionStrategy::Clipboard.display_name(), "剪贴板粘贴");
    assert_eq!(InjectionStrategy::ClipboardOnly.display_name(), "仅复制");
}

#[test]
fn test_injection_strategy_description() {
    // 验证所有描述都是非空的
    assert!(!InjectionStrategy::Auto.description().is_empty());
    assert!(!InjectionStrategy::Keyboard.description().is_empty());
    assert!(!InjectionStrategy::Clipboard.description().is_empty());
    assert!(!InjectionStrategy::ClipboardOnly.description().is_empty());
}

#[test]
fn test_injection_strategy_description_contains_keywords() {
    // Auto 描述应该提到"自动"或"长度"
    let auto_desc = InjectionStrategy::Auto.description();
    assert!(
        auto_desc.contains("自动") || auto_desc.contains("长度"),
        "Auto description: {}",
        auto_desc
    );

    // Keyboard 描述应该提到"键盘"
    let keyboard_desc = InjectionStrategy::Keyboard.description();
    assert!(
        keyboard_desc.contains("键盘"),
        "Keyboard description: {}",
        keyboard_desc
    );

    // Clipboard 描述应该提到"剪贴板"
    let clipboard_desc = InjectionStrategy::Clipboard.description();
    assert!(
        clipboard_desc.contains("剪贴板"),
        "Clipboard description: {}",
        clipboard_desc
    );

    // ClipboardOnly 描述应该提到"手动"或"复制"
    let clipboard_only_desc = InjectionStrategy::ClipboardOnly.description();
    assert!(
        clipboard_only_desc.contains("手动") || clipboard_only_desc.contains("复制"),
        "ClipboardOnly description: {}",
        clipboard_only_desc
    );
}

// ==================== InjectionStrategy 序列化测试 ====================

#[test]
fn test_injection_strategy_serialization() {
    let strategies = vec![
        InjectionStrategy::Auto,
        InjectionStrategy::Keyboard,
        InjectionStrategy::Clipboard,
        InjectionStrategy::ClipboardOnly,
    ];

    for strategy in strategies {
        let json = serde_json::to_string(&strategy).unwrap();
        let deserialized: InjectionStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(strategy, deserialized, "Failed for {:?}", strategy);
    }
}

#[test]
fn test_injection_strategy_json_format() {
    // 验证 JSON 格式
    let auto_json = serde_json::to_string(&InjectionStrategy::Auto).unwrap();
    assert!(
        auto_json.contains("Auto"),
        "JSON format: {}",
        auto_json
    );

    let keyboard_json = serde_json::to_string(&InjectionStrategy::Keyboard).unwrap();
    assert!(
        keyboard_json.contains("Keyboard"),
        "JSON format: {}",
        keyboard_json
    );
}

#[test]
fn test_injection_strategy_deserialization_from_string() {
    // 测试从字符串反序列化
    let auto: InjectionStrategy = serde_json::from_str("\"Auto\"").unwrap();
    assert_eq!(auto, InjectionStrategy::Auto);

    let keyboard: InjectionStrategy = serde_json::from_str("\"Keyboard\"").unwrap();
    assert_eq!(keyboard, InjectionStrategy::Keyboard);

    let clipboard: InjectionStrategy = serde_json::from_str("\"Clipboard\"").unwrap();
    assert_eq!(clipboard, InjectionStrategy::Clipboard);

    let clipboard_only: InjectionStrategy = serde_json::from_str("\"ClipboardOnly\"").unwrap();
    assert_eq!(clipboard_only, InjectionStrategy::ClipboardOnly);
}

// ==================== InjectionResult 测试 ====================

#[test]
fn test_injection_result_success() {
    let result = InjectionResult::success(InjectionStrategy::Keyboard, 10);

    assert!(result.success);
    assert_eq!(result.strategy_used, InjectionStrategy::Keyboard);
    assert_eq!(result.text_length, 10);
    assert!(result.error_message.is_none());
}

#[test]
fn test_injection_result_failure() {
    let result = InjectionResult::failure(InjectionStrategy::Clipboard, 100, "test error");

    assert!(!result.success);
    assert_eq!(result.strategy_used, InjectionStrategy::Clipboard);
    assert_eq!(result.text_length, 100);
    assert_eq!(result.error_message, Some("test error".to_string()));
}

#[test]
fn test_injection_result_clone() {
    let result = InjectionResult::success(InjectionStrategy::Auto, 50);
    let cloned = result.clone();

    assert_eq!(result.success, cloned.success);
    assert_eq!(result.strategy_used, cloned.strategy_used);
    assert_eq!(result.text_length, cloned.text_length);
    assert_eq!(result.error_message, cloned.error_message);
}

#[test]
fn test_injection_result_debug() {
    let result = InjectionResult::success(InjectionStrategy::Keyboard, 5);
    let debug_str = format!("{:?}", result);

    assert!(debug_str.contains("InjectionResult"));
    assert!(debug_str.contains("success"));
    assert!(debug_str.contains("Keyboard"));
}

#[test]
fn test_injection_result_with_various_lengths() {
    // 零长度
    let result0 = InjectionResult::success(InjectionStrategy::Auto, 0);
    assert_eq!(result0.text_length, 0);

    // 小长度
    let result_small = InjectionResult::success(InjectionStrategy::Keyboard, 5);
    assert_eq!(result_small.text_length, 5);

    // 大长度
    let result_large = InjectionResult::success(InjectionStrategy::Clipboard, 10000);
    assert_eq!(result_large.text_length, 10000);
}

#[test]
fn test_injection_result_with_various_errors() {
    // 空错误消息
    let result_empty = InjectionResult::failure(InjectionStrategy::Auto, 10, "");
    assert_eq!(result_empty.error_message, Some("".to_string()));

    // 长错误消息
    let long_error = "a".repeat(1000);
    let result_long = InjectionResult::failure(InjectionStrategy::Auto, 10, &long_error);
    assert_eq!(result_long.error_message, Some(long_error));

    // Unicode 错误消息
    let unicode_error = "注入失败：权限不足";
    let result_unicode = InjectionResult::failure(InjectionStrategy::Auto, 10, unicode_error);
    assert_eq!(result_unicode.error_message, Some(unicode_error.to_string()));
}

// ==================== 常量测试 ====================

#[test]
fn test_auto_strategy_threshold_constant() {
    // 验证阈值是一个合理的值
    assert!(AUTO_STRATEGY_THRESHOLD > 0);
    assert!(AUTO_STRATEGY_THRESHOLD < 1000);
    assert_eq!(AUTO_STRATEGY_THRESHOLD, 20); // 当前默认值
}

#[test]
fn test_paste_delay_constant() {
    // 验证延迟是一个合理的值
    assert!(PASTE_DELAY_MS > 0);
    assert!(PASTE_DELAY_MS < 10000);
    assert_eq!(PASTE_DELAY_MS, 100); // 当前默认值
}

// ==================== API 存在性测试 ====================

#[test]
fn test_injector_module_exports() {
    // 验证模块导出了正确的类型
    use raflow_lib::input::injector::{
        InjectionResult, InjectionStrategy, TextInjector, AUTO_STRATEGY_THRESHOLD, PASTE_DELAY_MS,
    };

    // 验证类型可以被引用
    fn _use_types(
        _strategy: InjectionStrategy,
        _result: InjectionResult,
        _threshold: usize,
        _delay: u64,
    ) {
        // 类型存在即可
    }

    fn _use_text_injector(app: &tauri::AppHandle) {
        let _ = TextInjector::new(app, InjectionStrategy::Auto);
    }

    let _ = AUTO_STRATEGY_THRESHOLD;
    let _ = PASTE_DELAY_MS;
}

#[test]
fn test_text_injector_methods_exist() {
    // 验证 TextInjector 的方法存在
    use raflow_lib::input::injector::{InjectionStrategy, TextInjector};
    use std::time::Duration;

    fn _verify_methods(app: &tauri::AppHandle) {
        // new 方法
        let mut injector = TextInjector::new(app, InjectionStrategy::Auto).unwrap();

        // with_config 方法
        let _injector2 = TextInjector::with_config(app, InjectionStrategy::Keyboard, 30, 200);

        // strategy getter
        let _s = injector.strategy();

        // strategy setter
        injector.set_strategy(InjectionStrategy::Clipboard);

        // auto_threshold getter/setter
        let _t = injector.auto_threshold();
        injector.set_auto_threshold(30);

        // paste_delay getter/setter
        let _d = injector.paste_delay();
        injector.set_paste_delay(Duration::from_millis(200));

        // inject 方法（需要异步上下文）
        // async fn _inject(injector: &mut TextInjector) {
        //     let _ = injector.inject("test").await;
        // }
    }
}

// ==================== Re-export 测试 ====================

#[test]
fn test_injector_reexports() {
    // 验证从 input 模块的 re-export
    use raflow_lib::input::{
        InjectionResult, InjectionStrategy, TextInjector, AUTO_STRATEGY_THRESHOLD, PASTE_DELAY_MS,
    };

    // 类型存在
    let _strategy = InjectionStrategy::Auto;
    let _result = InjectionResult::success(InjectionStrategy::Keyboard, 10);
    let _ = AUTO_STRATEGY_THRESHOLD;
    let _ = PASTE_DELAY_MS;

    fn _use_text_injector(app: &tauri::AppHandle) {
        let _ = TextInjector::new(app, InjectionStrategy::Auto);
    }
}

// ==================== 策略选择逻辑测试 ====================

#[test]
fn test_auto_strategy_threshold_logic() {
    // 测试自动策略的阈值逻辑
    let threshold = AUTO_STRATEGY_THRESHOLD;

    // 短文本应该使用键盘
    let short_text = "Hello";
    assert!(
        short_text.chars().count() < threshold,
        "Short text ({} chars) should be below threshold ({})",
        short_text.chars().count(),
        threshold
    );

    // 长文本应该使用剪贴板
    let long_text = "This is a longer text that exceeds the threshold value.";
    assert!(
        long_text.chars().count() >= threshold,
        "Long text ({} chars) should be at or above threshold ({})",
        long_text.chars().count(),
        threshold
    );
}

#[test]
fn test_auto_strategy_threshold_boundary() {
    let threshold = AUTO_STRATEGY_THRESHOLD;

    // 正好等于阈值的文本
    let boundary_text: String = "a".repeat(threshold);
    assert_eq!(boundary_text.chars().count(), threshold);

    // 阈值减一的文本
    let below_threshold: String = "a".repeat(threshold - 1);
    assert_eq!(below_threshold.chars().count(), threshold - 1);

    // 阈值加一的文本
    let above_threshold: String = "a".repeat(threshold + 1);
    assert_eq!(above_threshold.chars().count(), threshold + 1);
}

// ==================== Unicode 长度测试 ====================

#[test]
fn test_unicode_text_length_counting() {
    // 中文字符
    let chinese = "你好世界";
    assert_eq!(chinese.chars().count(), 4);

    // 表情符号
    let emoji = "👍🎉🔥";
    assert_eq!(emoji.chars().count(), 3);

    // 混合文本
    let mixed = "Hello你好👍";
    assert_eq!(mixed.chars().count(), 8);
}

// ==================== 错误处理测试 ====================

#[test]
fn test_injection_error_types() {
    use raflow_lib::input::InputError;

    // 验证相关错误类型存在
    let _keyboard_error = InputError::KeyboardSimulationFailed("test".to_string());
    let _clipboard_error = InputError::ClipboardFailed("test".to_string());
    let _injection_error = InputError::InjectionFailed("test".to_string());
}

// ==================== 文档测试 ====================

#[test]
fn test_documentation_compiles() {
    // 验证模块文档中的代码示例编译
    use raflow_lib::input::{InjectionResult, InjectionStrategy};

    // 创建策略
    let _strategy = InjectionStrategy::Auto;

    // 获取显示名称
    let _name = InjectionStrategy::Keyboard.display_name();

    // 获取描述
    let _desc = InjectionStrategy::Clipboard.description();

    // 创建结果
    let _success = InjectionResult::success(InjectionStrategy::Auto, 10);
    let _failure = InjectionResult::failure(InjectionStrategy::Keyboard, 20, "error");
}
