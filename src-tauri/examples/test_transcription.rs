//! 语音转写测试
//!
//! 测试端到端语音转写功能 (需要 ElevenLabs API Key)
//!
//! 运行:
//!   set ELEVENLABS_API_KEY=your-api-key
//!   cargo run --example test_transcription

use raflow_lib::transcription::{TranscriptEvent, TranscriptionSession};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    println!("=== 语音转写测试 ===\n");

    // 获取 API Key
    let api_key = std::env::var("ELEVENLABS_API_KEY").map_err(|_| {
        "请设置 ELEVENLABS_API_KEY 环境变量\n\n\
         Windows CMD:\n  set ELEVENLABS_API_KEY=your-api-key\n\n\
         PowerShell:\n  $env:ELEVENLABS_API_KEY=\"your-api-key\""
    })?;

    println!(
        "API Key: {}...{}",
        &api_key[..8],
        &api_key[api_key.len() - 4..]
    );
    println!();

    println!("开始录音... 请说话 (5秒后自动停止)");
    println!("{}", "-".repeat(50));

    // 启动转写会话
    let mut session = TranscriptionSession::start(&api_key, |event| match event {
        TranscriptEvent::SessionStarted { session_id } => {
            println!("\n[会话开始] ID: {}", session_id);
        }
        TranscriptEvent::Partial { text } => {
            print!("\r[部分转写] {}                    ", text);
        }
        TranscriptEvent::Committed { text } => {
            println!("\n[最终结果] {}", text);
        }
        TranscriptEvent::Error { message } => {
            println!("\n[错误] {}", message);
        }
        TranscriptEvent::Closed => {
            println!("\n[会话关闭]");
        }
    })
    .await?;

    // 录制 5 秒
    tokio::time::sleep(Duration::from_secs(5)).await;

    println!("\n{}", "-".repeat(50));
    println!("停止录音...");

    // 停止会话
    session.stop().await?;

    println!("\n测试完成!");

    Ok(())
}
