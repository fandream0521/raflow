//! 文本注入测试
//!
//! 测试键盘模拟和窗口检测功能 (不需要 API Key)
//!
//! 运行: cargo run --example test_input
//!
//! 注意: 运行后会模拟键盘输入，请确保光标在安全的位置

use raflow_lib::input::{
    KeyboardSimulator, get_focused_app_name, get_focused_window, get_focused_window_title,
    has_focused_window, is_text_input_context,
};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 文本注入测试 ===\n");

    // 1. 窗口检测测试
    println!("1. 窗口检测测试");
    println!("{}", "-".repeat(40));

    if has_focused_window() {
        println!("  有焦点窗口: 是");

        if let Ok(info) = get_focused_window() {
            println!("  应用名称: {}", info.app_name);
            println!("  窗口标题: {}", info.title);
            println!("  进程 ID: {}", info.process_id);
            println!("  可执行文件: {}", info.exec_name);
        }

        if let Some(name) = get_focused_app_name() {
            println!("  (便捷) 应用: {}", name);
        }

        if let Some(title) = get_focused_window_title() {
            println!("  (便捷) 标题: {}", title);
        }

        let is_text = is_text_input_context();
        println!("  是文本输入环境: {}", if is_text { "是" } else { "否" });
    } else {
        println!("  没有检测到焦点窗口");
    }

    println!();

    // 2. 键盘模拟测试
    println!("2. 键盘模拟测试");
    println!("{}", "-".repeat(40));
    println!("  请在 3 秒内将光标移到文本框中...");

    std::thread::sleep(Duration::from_secs(3));

    println!("  开始输入测试文本...");

    let mut keyboard = KeyboardSimulator::new()?;

    // 输入测试文本
    keyboard.type_text("Hello from RaFlow! ")?;
    std::thread::sleep(Duration::from_millis(100));

    keyboard.type_text("你好世界 ")?;
    std::thread::sleep(Duration::from_millis(100));

    keyboard.type_text("语音转写测试")?;

    println!("  输入完成!");
    println!();

    // 3. 快捷键测试
    println!("3. 快捷键测试 (Ctrl+A 全选)");
    println!("{}", "-".repeat(40));
    println!("  2 秒后执行全选...");

    std::thread::sleep(Duration::from_secs(2));

    keyboard.select_all()?;
    println!("  全选完成!");

    println!("\n测试完成!");

    Ok(())
}
