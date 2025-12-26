//! 音频采集测试
//!
//! 测试麦克风音频采集功能
//!
//! 运行: cargo run --example test_audio

use raflow_lib::audio::AudioCapture;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("=== 音频采集测试 ===\n");

    // 列出可用设备
    println!("可用输入设备:");
    let devices = raflow_lib::audio::list_input_devices()?;
    for (i, device) in devices.iter().enumerate() {
        println!("  {}: {} ({})", i, device.name, device.id);
    }
    println!();

    // 获取默认设备
    let default = raflow_lib::audio::get_default_input_device()?;
    println!("默认设备: {}\n", default.name);

    // 测试音频采集
    println!("开始采集音频 (3秒)...");
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut capture = AudioCapture::new(None)?;
    capture.start(tx)?;

    let mut sample_count = 0;
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(3) {
        if let Ok(samples) = rx.try_recv() {
            sample_count += samples.len();

            // 计算音量级别
            let max_sample = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
            let level = (max_sample * 50.0) as usize;
            let bar: String = "█".repeat(level.min(50));
            print!("\r音量: {:50} ({:.3})", bar, max_sample);
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    capture.stop();
    println!("\n\n采集完成! 共采集 {} 个样本", sample_count);

    Ok(())
}
