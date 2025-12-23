# RaFlow 实现计划

## 1. 概述

本文档基于 [0002-design.md](./0002-design.md) 详细设计，提供分阶段的实现计划。项目采用增量开发模式，每个阶段都产出可测试的功能模块。

### 1.1 实现原则

- **垂直切片优先**：每阶段实现完整的端到端功能
- **最小可行产品**：Phase 1 结束即可进行基本语音转写
- **持续集成**：每个任务完成后都进行测试验证
- **平台渐进**：先实现 Windows，再扩展 macOS/Linux

### 1.2 阶段概览

```mermaid
gantt
    title RaFlow 实现阶段
    dateFormat X
    axisFormat %s

    section Phase 0
    项目初始化           :p0, 0, 1

    section Phase 1
    核心音频流水线        :p1a, after p0, 2
    WebSocket 通信       :p1b, after p0, 2
    基础集成             :p1c, after p1a p1b, 1

    section Phase 2
    状态管理             :p2a, after p1c, 1
    全局热键             :p2b, after p1c, 1
    文本注入             :p2c, after p2a, 2

    section Phase 3
    前端 UI              :p3a, after p2c, 2
    系统托盘             :p3b, after p2c, 1
    配置持久化           :p3c, after p3a, 1

    section Phase 4
    性能优化             :p4a, after p3c, 1
    错误处理             :p4b, after p3c, 1
    多平台适配           :p4c, after p4a p4b, 2
```

---

## 2. Phase 0: 项目初始化

### 2.1 目标

建立项目基础结构，配置开发环境，确保所有依赖可用。

### 2.2 任务清单

```mermaid
flowchart TD
    P0T1[P0-T1: 创建 Tauri v2 项目] --> P0T2[P0-T2: 配置 Cargo workspace]
    P0T2 --> P0T3[P0-T3: 添加核心依赖]
    P0T3 --> P0T4[P0-T4: 配置 Capabilities]
    P0T4 --> P0T5[P0-T5: 设置日志系统]
    P0T5 --> P0T6[P0-T6: 验证构建]
```

#### P0-T1: 创建 Tauri v2 项目

**描述**: 使用 Tauri CLI 创建新项目

**步骤**:
```bash
# 安装 Tauri CLI
cargo install tauri-cli --version "^2.0"

# 创建项目
cargo tauri init
```

**验收标准**:
- [x] `cargo tauri dev` 能够启动空白窗口
- [x] 项目结构符合 Tauri v2 规范

---

#### P0-T2: 配置 Cargo Workspace

**描述**: 设置 workspace 管理依赖版本

**文件**: `Cargo.toml` (workspace root)

```toml
[workspace]
members = ["src-tauri"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/user/raflow"

[workspace.dependencies]
# Tauri
tauri = { version = "2.9", features = ["tray-icon", "protocol-asset"] }
tauri-build = "2.0"
tauri-plugin-global-shortcut = "2.3"
tauri-plugin-clipboard-manager = "2.3"
tauri-plugin-dialog = "2.2"
tauri-plugin-fs = "2.2"
tauri-plugin-shell = "2.2"

# Async Runtime
tokio = { version = "1.42", features = ["full"] }

# WebSocket
tokio-tungstenite = { version = "0.28", features = ["rustls-tls-native-roots"] }
futures-util = "0.3"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Audio
cpal = "0.15"
rubato = "0.16"

# Encoding
base64 = "0.22"

# Input Simulation
enigo = "0.6"

# Window Detection
x-win = "5.3"

# State Management
arc-swap = "1.7"

# Error Handling
thiserror = "2.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**验收标准**:
- [x] 所有依赖版本统一管理
- [x] `cargo check` 通过

**完成状态**: ✅ 已完成 (2025-12-22)

---

#### P0-T3: 添加核心依赖

**描述**: 在 `src-tauri/Cargo.toml` 中引用 workspace 依赖

**文件**: `src-tauri/Cargo.toml`

```toml
[package]
name = "raflow"
version.workspace = true
edition.workspace = true

[lib]
name = "raflow_lib"
crate-type = ["lib", "cdylib", "staticlib"]

[build-dependencies]
tauri-build = { workspace = true }

[dependencies]
tauri = { workspace = true }
tauri-plugin-global-shortcut = { workspace = true }
tauri-plugin-clipboard-manager = { workspace = true }
tauri-plugin-dialog = { workspace = true }
tauri-plugin-fs = { workspace = true }
tauri-plugin-shell = { workspace = true }

tokio = { workspace = true }
tokio-tungstenite = { workspace = true }
futures-util = { workspace = true }

serde = { workspace = true }
serde_json = { workspace = true }

cpal = { workspace = true }
rubato = { workspace = true }
base64 = { workspace = true }

enigo = { workspace = true }
x-win = { workspace = true }

arc-swap = { workspace = true }
thiserror = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

[target.'cfg(target_os = "macos")'.dependencies]
macos-accessibility-client = "0.0.1"

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.59", features = [
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_UI_WindowsAndMessaging"
]}
```

**验收标准**:
- [x] 所有依赖版本统一管理
- [x] `cargo check` 通过

**完成状态**: ✅ 已完成 (2025-12-22)
 
---

#### P0-T4: 配置 Capabilities

**描述**: 设置 Tauri v2 权限系统

**文件**: `src-tauri/capabilities/default.json`

**验收标准**:
- [x] 支持 main 和 overlay 两个窗口
- [x] 包含所有必需的权限配置
- [x] `cargo check` 通过

**完成状态**: ✅ 已完成 (2025-12-23)

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "RaFlow default capabilities",
  "windows": ["main", "overlay"],
  "permissions": [
    "core:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-focus",
    "core:window:allow-center",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "global-shortcut:allow-is-registered",
    "clipboard-manager:allow-write-text",
    "clipboard-manager:allow-read-text",
    "dialog:allow-message",
    "dialog:allow-ask",
    "fs:allow-read-text-file",
    "fs:allow-write-text-file",
    "shell:allow-open"
  ]
}
```

---

#### P0-T5: 设置日志系统

**描述**: 配置 tracing 日志

**文件**: `src-tauri/src/utils/logging.rs`

```rust
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("raflow=debug,warn"));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    tracing::info!("RaFlow logging initialized");
}
```

**验收标准**:
- [x] 日志系统模块化到 utils/logging.rs
- [x] 在 lib.rs 中正确初始化
- [x] 支持 RUST_LOG 环境变量配置
- [x] 测试通过，日志输出正常

**完成状态**: ✅ 已完成 (2025-12-23)

---

#### P0-T6: 验证构建

**描述**: 确保项目能够成功构建和运行

**验收标准**:
- [x] `cargo build --release` 成功（3分26秒完成）
- [x] 集成测试通过（验证所有依赖配置正确）
- [x] 日志输出正常（带时间戳、级别、模块路径）

**测试结果**:
```bash
# Release 构建
$ cargo build --release
Finished `release` profile [optimized] target(s) in 3m 26s

# 集成测试
$ cargo test --test integration_test
running 6 tests
test phase0_validation::test_tauri_available ... ok
test phase0_validation::test_serde_available ... ok
test phase0_validation::test_tracing_available ... ok
test test_project_compiles ... ok
test test_logging_module_exists ... ok
test phase0_validation::test_tokio_available ... ok

test result: ok. 6 passed; 0 failed; 0 ignored

# 日志输出验证
2025-12-23T12:23:52.043995Z INFO raflow_lib::utils::logging: RaFlow logging initialized
```

**完成状态**: ✅ 已完成 (2025-12-23)

---

## 3. Phase 1: 核心数据流

### 3.1 目标

实现音频采集到 WebSocket 传输的完整数据流，可在命令行验证转写功能。

### 3.2 任务依赖图

```mermaid
flowchart TD
    subgraph "音频模块"
        P1T1[P1-T1: 音频设备枚举]
        P1T2[P1-T2: 音频采集 Stream]
        P1T3[P1-T3: 重采样器]
        P1T4[P1-T4: 音频处理管道]
    end

    subgraph "网络模块"
        P1T5[P1-T5: 消息类型定义]
        P1T6[P1-T6: WebSocket 连接]
        P1T7[P1-T7: 发送/接收任务]
    end

    subgraph "集成"
        P1T8[P1-T8: 端到端集成]
    end

    P1T1 --> P1T2
    P1T2 --> P1T3
    P1T3 --> P1T4

    P1T5 --> P1T6
    P1T6 --> P1T7

    P1T4 --> P1T8
    P1T7 --> P1T8
```

### 3.3 详细任务

#### P1-T1: 音频设备枚举

**模块**: `src-tauri/src/audio/device.rs`

**功能**:
- 列出所有可用输入设备
- 获取默认输入设备
- 查询设备支持的配置

**接口定义**:
```rust
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub sample_rates: Vec<u32>,
}

pub fn list_input_devices() -> Result<Vec<AudioDevice>, AudioError>;
pub fn get_default_input_device() -> Result<AudioDevice, AudioError>;
pub fn get_device_config(device_id: &str) -> Result<cpal::StreamConfig, AudioError>;
```

**测试用例**:
```rust
#[test]
fn test_list_devices() {
    let devices = list_input_devices().unwrap();
    assert!(!devices.is_empty(), "Should have at least one input device");
}

#[test]
fn test_default_device() {
    let device = get_default_input_device().unwrap();
    assert!(!device.name.is_empty());
}
```

---

#### P1-T2: 音频采集 Stream

**模块**: `src-tauri/src/audio/capture.rs`

**功能**:
- 创建和管理 cpal 输入流
- 通过 channel 传递音频数据
- 支持启动/停止控制

**接口定义**:
```rust
pub struct AudioCapture {
    stream: Option<cpal::Stream>,
    sample_rate: u32,
}

impl AudioCapture {
    pub fn new(device_id: Option<&str>) -> Result<Self, AudioError>;
    pub fn start(&mut self, sender: mpsc::Sender<Vec<f32>>) -> Result<(), AudioError>;
    pub fn stop(&mut self);
    pub fn sample_rate(&self) -> u32;
}
```

**实现要点**:
```rust
// 音频回调中避免阻塞操作
let callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
    // 使用 try_send 避免阻塞
    let _ = sender.try_send(data.to_vec());
};
```

**测试用例**:
```rust
#[tokio::test]
async fn test_audio_capture() {
    let (tx, mut rx) = mpsc::channel(100);
    let mut capture = AudioCapture::new(None).unwrap();

    capture.start(tx).unwrap();

    // 等待接收一些音频数据
    let data = tokio::time::timeout(
        Duration::from_secs(1),
        rx.recv()
    ).await;

    assert!(data.is_ok());
    capture.stop();
}
```

---

#### P1-T3: 重采样器

**模块**: `src-tauri/src/audio/resampler.rs`

**功能**:
- 将任意采样率转换为 16kHz
- 支持常见采样率 (44.1k, 48k, 32k)
- 高质量 Sinc 插值

**接口定义**:
```rust
pub struct AudioResampler {
    resampler: rubato::SincFixedIn<f32>,
    input_buffer: Vec<Vec<f32>>,
    output_buffer: Vec<Vec<f32>>,
}

impl AudioResampler {
    pub fn new(input_rate: u32, output_rate: u32) -> Result<Self, AudioError>;
    pub fn process(&mut self, input: &[f32]) -> Result<Vec<f32>, AudioError>;
    pub fn reset(&mut self);
}
```

**实现要点**:
```rust
use rubato::{SincFixedIn, SincInterpolationType, SincInterpolationParameters, WindowFunction};

pub fn new(input_rate: u32, output_rate: u32) -> Result<Self, AudioError> {
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    let resampler = SincFixedIn::new(
        output_rate as f64 / input_rate as f64,
        2.0,  // max relative ratio
        params,
        1024, // chunk size
        1,    // channels
    )?;

    Ok(Self { resampler, ... })
}
```

**测试用例**:
```rust
#[test]
fn test_resample_48k_to_16k() {
    let mut resampler = AudioResampler::new(48000, 16000).unwrap();

    // 输入 480 samples (10ms @ 48kHz)
    let input: Vec<f32> = (0..480).map(|i| (i as f32 * 0.01).sin()).collect();
    let output = resampler.process(&input).unwrap();

    // 输出应约 160 samples (10ms @ 16kHz)
    assert!((output.len() as i32 - 160).abs() < 10);
}
```

---

#### P1-T4: 音频处理管道

**模块**: `src-tauri/src/audio/pipeline.rs`

**功能**:
- 整合采集、重采样、编码
- 输出 Base64 编码的 PCM 数据
- 批量处理 (累积 100ms)

**接口定义**:
```rust
pub struct AudioPipeline {
    capture: AudioCapture,
    resampler: AudioResampler,
    buffer: Vec<f32>,
    output_tx: mpsc::Sender<String>,  // Base64 encoded chunks
}

impl AudioPipeline {
    pub fn new(output_tx: mpsc::Sender<String>) -> Result<Self, AudioError>;
    pub async fn run(&mut self) -> Result<(), AudioError>;
    pub fn stop(&mut self);
}
```

**数据流**:
```mermaid
flowchart LR
    A[cpal 回调] -->|f32 @ 48k| B[缓冲区]
    B -->|1600 samples| C[重采样器]
    C -->|533 samples @ 16k| D[f32→i16]
    D -->|PCM bytes| E[Base64]
    E -->|String| F[输出 Channel]
```

---

#### P1-T5: 消息类型定义

**模块**: `src-tauri/src/network/messages.rs`

**功能**:
- 定义所有 WebSocket 消息类型
- 实现序列化/反序列化

**代码**:
```rust
use serde::{Deserialize, Serialize};

// ===== Client -> Server =====

#[derive(Serialize, Debug)]
pub struct InputAudioChunk {
    pub message_type: &'static str,
    pub audio_base_64: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,
}

impl InputAudioChunk {
    pub fn new(audio_base_64: String) -> Self {
        Self {
            message_type: "input_audio_chunk",
            audio_base_64,
            commit: None,
            sample_rate: None,
        }
    }

    pub fn with_sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = Some(rate);
        self
    }
}

#[derive(Serialize, Debug)]
pub struct CommitMessage {
    pub message_type: &'static str,
}

impl Default for CommitMessage {
    fn default() -> Self {
        Self { message_type: "commit" }
    }
}

// ===== Server -> Client =====

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "message_type")]
pub enum ServerMessage {
    #[serde(rename = "session_started")]
    SessionStarted {
        session_id: String,
    },

    #[serde(rename = "partial_transcript")]
    PartialTranscript {
        text: String,
    },

    #[serde(rename = "committed_transcript")]
    CommittedTranscript {
        text: String,
    },

    #[serde(rename = "input_error")]
    InputError {
        error_message: String,
    },
}
```

**测试用例**:
```rust
#[test]
fn test_serialize_audio_chunk() {
    let chunk = InputAudioChunk::new("SGVsbG8=".to_string())
        .with_sample_rate(16000);

    let json = serde_json::to_string(&chunk).unwrap();
    assert!(json.contains("input_audio_chunk"));
    assert!(json.contains("16000"));
}

#[test]
fn test_deserialize_server_message() {
    let json = r#"{"message_type": "partial_transcript", "text": "hello"}"#;
    let msg: ServerMessage = serde_json::from_str(json).unwrap();

    match msg {
        ServerMessage::PartialTranscript { text } => {
            assert_eq!(text, "hello");
        }
        _ => panic!("Wrong message type"),
    }
}
```

---

#### P1-T6: WebSocket 连接

**模块**: `src-tauri/src/network/connection.rs`

**功能**:
- 建立 WSS 连接
- 处理认证 Header
- 连接状态管理

**接口定义**:
```rust
pub struct ScribeConnection {
    write: SplitSink<WebSocketStream<...>, Message>,
    read: SplitStream<WebSocketStream<...>>,
}

impl ScribeConnection {
    pub async fn connect(api_key: &str, config: &ConnectionConfig) -> Result<Self, NetworkError>;
    pub async fn send(&mut self, message: impl Serialize) -> Result<(), NetworkError>;
    pub async fn recv(&mut self) -> Result<Option<ServerMessage>, NetworkError>;
    pub async fn close(&mut self) -> Result<(), NetworkError>;
}

pub struct ConnectionConfig {
    pub model_id: String,
    pub language_code: Option<String>,
    pub sample_rate: u32,
}
```

**实现要点**:
```rust
use tokio_tungstenite::{connect_async, tungstenite::http::Request};

pub async fn connect(api_key: &str, config: &ConnectionConfig) -> Result<Self, NetworkError> {
    let url = format!(
        "wss://api.elevenlabs.io/v1/speech-to-text/realtime?model_id={}&sample_rate={}",
        config.model_id,
        config.sample_rate
    );

    let request = Request::builder()
        .uri(&url)
        .header("xi-api-key", api_key)
        .header("Host", "api.elevenlabs.io")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", generate_key())
        .body(())?;

    let (ws_stream, _) = connect_async(request).await?;
    let (write, read) = ws_stream.split();

    Ok(Self { write, read })
}
```

---

#### P1-T7: 发送/接收任务

**模块**: `src-tauri/src/network/tasks.rs`

**功能**:
- 异步发送音频数据
- 异步接收服务端消息
- 通过 channel 与其他模块通信

**接口定义**:
```rust
pub async fn sender_task(
    mut ws_write: SplitSink<...>,
    mut audio_rx: mpsc::Receiver<String>,
    cancel: CancellationToken,
) -> Result<(), NetworkError>;

pub async fn receiver_task(
    mut ws_read: SplitStream<...>,
    transcript_tx: mpsc::Sender<ServerMessage>,
    cancel: CancellationToken,
) -> Result<(), NetworkError>;
```

**实现要点**:
```rust
pub async fn sender_task(...) -> Result<(), NetworkError> {
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("Sender task cancelled");
                break;
            }
            Some(audio_b64) = audio_rx.recv() => {
                let chunk = InputAudioChunk::new(audio_b64);
                let json = serde_json::to_string(&chunk)?;
                ws_write.send(Message::Text(json)).await?;
            }
        }
    }
    Ok(())
}
```

---

#### P1-T8: 端到端集成

**模块**: `src-tauri/src/transcription/mod.rs`

**功能**:
- 整合音频管道和网络通信
- 提供简单的 start/stop 接口
- 通过回调返回转写结果

**接口定义**:
```rust
pub struct TranscriptionSession {
    audio_pipeline: AudioPipeline,
    connection: ScribeConnection,
    cancel: CancellationToken,
}

impl TranscriptionSession {
    pub async fn start<F>(api_key: &str, on_transcript: F) -> Result<Self, AppError>
    where
        F: Fn(TranscriptEvent) + Send + 'static;

    pub async fn stop(&mut self) -> Result<(), AppError>;
}

pub enum TranscriptEvent {
    Partial(String),
    Committed(String),
    Error(String),
}
```

**集成测试**:
```rust
#[tokio::test]
#[ignore] // 需要有效 API Key
async fn test_e2e_transcription() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("ELEVENLABS_API_KEY").unwrap();

    let (tx, mut rx) = mpsc::channel(10);

    let session = TranscriptionSession::start(&api_key, move |event| {
        let _ = tx.blocking_send(event);
    }).await.unwrap();

    // 等待一些转写结果
    tokio::time::sleep(Duration::from_secs(5)).await;

    session.stop().await.unwrap();

    // 验证收到了消息
    assert!(rx.try_recv().is_ok());
}
```

---

## 4. Phase 2: 交互控制

### 4.1 目标

实现状态管理、全局热键和文本注入，使应用可通过热键触发转写并注入结果。

### 4.2 任务依赖图

```mermaid
flowchart TD
    P2T1[P2-T1: 状态机实现] --> P2T3[P2-T3: 热键处理器]
    P2T2[P2-T2: 全局热键注册] --> P2T3
    P2T3 --> P2T4[P2-T4: 状态转换逻辑]

    P2T5[P2-T5: 窗口检测] --> P2T7[P2-T7: 注入器集成]
    P2T6[P2-T6: 键盘模拟] --> P2T7
    P2T6B[P2-T6B: 剪贴板操作] --> P2T7

    P2T4 --> P2T8[P2-T8: 完整流程集成]
    P2T7 --> P2T8
```

### 4.3 详细任务

#### P2-T1: 状态机实现

**模块**: `src-tauri/src/state/app_state.rs`

**代码**:
```rust
use std::sync::Arc;
use arc_swap::ArcSwap;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Idle,
    Connecting,
    Recording { partial_text: String },
    Processing,
    Injecting,
    Error(String),
}

pub struct StateManager {
    state: ArcSwap<AppState>,
    listeners: Vec<mpsc::Sender<AppState>>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            state: ArcSwap::new(Arc::new(AppState::Idle)),
            listeners: Vec::new(),
        }
    }

    pub fn current(&self) -> Arc<AppState> {
        self.state.load_full()
    }

    pub fn transition(&self, new_state: AppState) -> Result<(), StateError> {
        let current = self.current();

        // 验证状态转换是否合法
        if !self.is_valid_transition(&current, &new_state) {
            return Err(StateError::InvalidTransition {
                from: (*current).clone(),
                to: new_state,
            });
        }

        self.state.store(Arc::new(new_state.clone()));

        // 通知监听者
        for listener in &self.listeners {
            let _ = listener.try_send(new_state.clone());
        }

        Ok(())
    }

    fn is_valid_transition(&self, from: &AppState, to: &AppState) -> bool {
        matches!(
            (from, to),
            (AppState::Idle, AppState::Connecting)
                | (AppState::Connecting, AppState::Recording { .. })
                | (AppState::Connecting, AppState::Error(_))
                | (AppState::Recording { .. }, AppState::Processing)
                | (AppState::Recording { .. }, AppState::Idle)  // 取消
                | (AppState::Processing, AppState::Injecting)
                | (AppState::Processing, AppState::Idle)  // 超时
                | (AppState::Injecting, AppState::Idle)
                | (AppState::Error(_), AppState::Idle)
        )
    }
}
```

---

#### P2-T2: 全局热键注册

**模块**: `src-tauri/src/hotkey/mod.rs`

**代码**:
```rust
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn register_hotkeys(app: &AppHandle) -> Result<(), HotkeyError> {
    let shortcut_manager = app.global_shortcut();

    // Push-to-Talk 热键
    let ptt_shortcut = "CommandOrControl+Shift+.".parse::<Shortcut>()?;

    shortcut_manager.on_shortcut(ptt_shortcut, |app, _shortcut, event| {
        let state_manager = app.state::<StateManager>();

        match event.state {
            ShortcutState::Pressed => {
                tracing::info!("PTT pressed");
                handle_ptt_pressed(app, &state_manager);
            }
            ShortcutState::Released => {
                tracing::info!("PTT released");
                handle_ptt_released(app, &state_manager);
            }
        }
    })?;

    // 取消热键
    let cancel_shortcut = "Escape".parse::<Shortcut>()?;
    shortcut_manager.on_shortcut(cancel_shortcut, |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            handle_cancel(app);
        }
    })?;

    tracing::info!("Global hotkeys registered");
    Ok(())
}
```

---

#### P2-T3: 热键处理器

**模块**: `src-tauri/src/hotkey/handlers.rs`

**代码**:
```rust
fn handle_ptt_pressed(app: &AppHandle, state_manager: &StateManager) {
    let current = state_manager.current();

    if *current != AppState::Idle {
        tracing::warn!("PTT pressed but not in Idle state: {:?}", current);
        return;
    }

    // 启动转写会话
    let app_handle = app.clone();
    tokio::spawn(async move {
        if let Err(e) = start_transcription_session(&app_handle).await {
            tracing::error!("Failed to start transcription: {}", e);
            let state_manager = app_handle.state::<StateManager>();
            let _ = state_manager.transition(AppState::Error(e.to_string()));
        }
    });
}

fn handle_ptt_released(app: &AppHandle, state_manager: &StateManager) {
    let current = state_manager.current();

    if !matches!(*current, AppState::Recording { .. }) {
        return;
    }

    // 发送 commit 并等待最终结果
    let app_handle = app.clone();
    tokio::spawn(async move {
        if let Err(e) = commit_and_process(&app_handle).await {
            tracing::error!("Failed to commit: {}", e);
        }
    });
}

fn handle_cancel(app: &AppHandle) {
    let state_manager = app.state::<StateManager>();
    let current = state_manager.current();

    if matches!(*current, AppState::Recording { .. } | AppState::Connecting) {
        let _ = state_manager.transition(AppState::Idle);
        // 取消当前会话
        cancel_current_session(app);
    }
}
```

---

#### P2-T5: 窗口检测

**模块**: `src-tauri/src/input/window.rs`

**代码**:
```rust
use x_win::{get_active_window, ActiveWindow};

pub struct WindowInfo {
    pub app_name: String,
    pub title: String,
    pub process_id: u32,
}

pub fn get_focused_window() -> Result<WindowInfo, InputError> {
    let window = get_active_window()
        .map_err(|_| InputError::NoFocusedWindow)?;

    Ok(WindowInfo {
        app_name: window.app_name,
        title: window.title,
        process_id: window.process_id,
    })
}

pub fn is_text_input_context() -> bool {
    // 基于应用名称的简单启发式判断
    if let Ok(window) = get_focused_window() {
        let text_apps = ["Code", "Notepad", "Word", "Chrome", "Firefox", "Safari"];
        text_apps.iter().any(|app| window.app_name.contains(app))
    } else {
        false
    }
}
```

---

#### P2-T6: 键盘模拟

**模块**: `src-tauri/src/input/keyboard.rs`

**代码**:
```rust
use enigo::{Enigo, Keyboard, Settings};

pub struct KeyboardSimulator {
    enigo: Enigo,
}

impl KeyboardSimulator {
    pub fn new() -> Result<Self, InputError> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| InputError::InjectionFailed(e.to_string()))?;
        Ok(Self { enigo })
    }

    pub fn type_text(&mut self, text: &str) -> Result<(), InputError> {
        self.enigo
            .text(text)
            .map_err(|e| InputError::InjectionFailed(e.to_string()))
    }

    pub fn paste(&mut self) -> Result<(), InputError> {
        #[cfg(target_os = "macos")]
        {
            use enigo::Key;
            self.enigo.key(Key::Meta, enigo::Direction::Press)?;
            self.enigo.key(Key::Unicode('v'), enigo::Direction::Click)?;
            self.enigo.key(Key::Meta, enigo::Direction::Release)?;
        }

        #[cfg(target_os = "windows")]
        {
            use enigo::Key;
            self.enigo.key(Key::Control, enigo::Direction::Press)?;
            self.enigo.key(Key::Unicode('v'), enigo::Direction::Click)?;
            self.enigo.key(Key::Control, enigo::Direction::Release)?;
        }

        Ok(())
    }
}
```

---

#### P2-T6B: 剪贴板操作

**模块**: `src-tauri/src/input/clipboard.rs`

**代码**:
```rust
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

pub struct ClipboardManager<'a> {
    app: &'a AppHandle,
    saved_content: Option<String>,
}

impl<'a> ClipboardManager<'a> {
    pub fn new(app: &'a AppHandle) -> Self {
        Self {
            app,
            saved_content: None,
        }
    }

    pub async fn save(&mut self) -> Result<(), InputError> {
        self.saved_content = self.app
            .clipboard()
            .read_text()
            .ok();
        Ok(())
    }

    pub async fn write(&self, text: &str) -> Result<(), InputError> {
        self.app
            .clipboard()
            .write_text(text)
            .map_err(|e| InputError::ClipboardFailed(e.to_string()))
    }

    pub async fn restore(&self) -> Result<(), InputError> {
        if let Some(content) = &self.saved_content {
            self.write(content).await?;
        }
        Ok(())
    }
}
```

---

#### P2-T7: 注入器集成

**模块**: `src-tauri/src/input/injector.rs`

**代码**:
```rust
pub enum InjectionStrategy {
    Auto,
    Keyboard,
    Clipboard,
    ClipboardOnly,
}

pub struct TextInjector<'a> {
    app: &'a AppHandle,
    strategy: InjectionStrategy,
    keyboard: KeyboardSimulator,
}

impl<'a> TextInjector<'a> {
    pub fn new(app: &'a AppHandle, strategy: InjectionStrategy) -> Result<Self, InputError> {
        Ok(Self {
            app,
            strategy,
            keyboard: KeyboardSimulator::new()?,
        })
    }

    pub async fn inject(&mut self, text: &str) -> Result<(), InputError> {
        match self.strategy {
            InjectionStrategy::Auto => {
                if text.len() < 10 {
                    self.inject_via_keyboard(text)
                } else {
                    self.inject_via_clipboard(text).await
                }
            }
            InjectionStrategy::Keyboard => self.inject_via_keyboard(text),
            InjectionStrategy::Clipboard => self.inject_via_clipboard(text).await,
            InjectionStrategy::ClipboardOnly => {
                self.copy_to_clipboard(text).await?;
                // 通知用户文本已复制
                Ok(())
            }
        }
    }

    fn inject_via_keyboard(&mut self, text: &str) -> Result<(), InputError> {
        self.keyboard.type_text(text)
    }

    async fn inject_via_clipboard(&mut self, text: &str) -> Result<(), InputError> {
        let mut clipboard = ClipboardManager::new(self.app);

        // 保存当前剪贴板
        clipboard.save().await?;

        // 写入新内容
        clipboard.write(text).await?;

        // 模拟粘贴
        self.keyboard.paste()?;

        // 等待粘贴完成
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 恢复剪贴板
        clipboard.restore().await?;

        Ok(())
    }

    async fn copy_to_clipboard(&self, text: &str) -> Result<(), InputError> {
        let clipboard = ClipboardManager::new(self.app);
        clipboard.write(text).await
    }
}
```

---

## 5. Phase 3: 用户界面

### 5.1 目标

实现悬浮窗 UI、系统托盘和配置界面。

### 5.2 任务依赖图

```mermaid
flowchart TD
    P3T1[P3-T1: 窗口配置] --> P3T2[P3-T2: 悬浮窗组件]
    P3T2 --> P3T3[P3-T3: 波形动画]
    P3T2 --> P3T4[P3-T4: 转写显示]

    P3T5[P3-T5: 系统托盘] --> P3T6[P3-T6: 托盘菜单]

    P3T7[P3-T7: 设置界面] --> P3T8[P3-T8: 配置持久化]

    P3T3 --> P3T9[P3-T9: 事件集成]
    P3T4 --> P3T9
    P3T6 --> P3T9
    P3T8 --> P3T9
```

### 5.3 详细任务

#### P3-T1: 窗口配置

**文件**: `src-tauri/tauri.conf.json`

```json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "RaFlow",
        "width": 600,
        "height": 400,
        "visible": false,
        "center": true,
        "resizable": true
      },
      {
        "label": "overlay",
        "title": "",
        "width": 400,
        "height": 100,
        "x": null,
        "y": null,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "visible": false,
        "focus": false
      }
    ],
    "trayIcon": {
      "iconPath": "icons/tray.png",
      "iconAsTemplate": true
    }
  }
}
```

---

#### P3-T2: 悬浮窗组件

**文件**: `src/components/Overlay/index.tsx`

```tsx
import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { Waveform } from './Waveform';
import { TranscriptDisplay } from './TranscriptDisplay';
import styles from './Overlay.module.css';

interface OverlayState {
  status: 'idle' | 'connecting' | 'recording' | 'processing';
  partialText: string;
  finalText: string;
  audioLevel: number;
}

export function Overlay() {
  const [state, setState] = useState<OverlayState>({
    status: 'idle',
    partialText: '',
    finalText: '',
    audioLevel: 0,
  });

  useEffect(() => {
    const unlistenPartial = listen<{ text: string }>('transcript:partial', (event) => {
      setState(prev => ({ ...prev, partialText: event.payload.text }));
    });

    const unlistenCommitted = listen<{ text: string }>('transcript:committed', (event) => {
      setState(prev => ({ ...prev, finalText: event.payload.text }));
    });

    const unlistenAudio = listen<{ level: number }>('audio:level', (event) => {
      setState(prev => ({ ...prev, audioLevel: event.payload.level }));
    });

    const unlistenState = listen<{ state: string }>('app:state', (event) => {
      setState(prev => ({ ...prev, status: event.payload.state as any }));
    });

    return () => {
      unlistenPartial.then(f => f());
      unlistenCommitted.then(f => f());
      unlistenAudio.then(f => f());
      unlistenState.then(f => f());
    };
  }, []);

  return (
    <div className={styles.overlay}>
      <div className={styles.statusBar}>
        <StatusIndicator status={state.status} />
        <Waveform level={state.audioLevel} />
      </div>
      <TranscriptDisplay
        partialText={state.partialText}
        finalText={state.finalText}
      />
    </div>
  );
}
```

---

#### P3-T3: 波形动画

**文件**: `src/components/Overlay/Waveform.tsx`

```tsx
import { useMemo } from 'react';
import styles from './Waveform.module.css';

interface WaveformProps {
  level: number; // 0-1
}

export function Waveform({ level }: WaveformProps) {
  const bars = useMemo(() => {
    const count = 5;
    return Array.from({ length: count }, (_, i) => {
      const baseHeight = 0.3;
      const variation = Math.sin((i / count) * Math.PI) * level;
      return Math.max(baseHeight, variation);
    });
  }, [level]);

  return (
    <div className={styles.waveform}>
      {bars.map((height, i) => (
        <div
          key={i}
          className={styles.bar}
          style={{
            height: `${height * 100}%`,
            animationDelay: `${i * 0.1}s`,
          }}
        />
      ))}
    </div>
  );
}
```

---

#### P3-T5: 系统托盘

**文件**: `src-tauri/src/tray.rs`

```rust
use tauri::{
    AppHandle, Manager,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{Menu, MenuItem},
};

pub fn setup_tray(app: &AppHandle) -> Result<(), TrayError> {
    let quit = MenuItem::with_id(app, "quit", "Quit RaFlow", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let separator = MenuItem::separator(app)?;

    let menu = Menu::with_items(app, &[&settings, &separator, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .menu_on_left_click(false)
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "quit" => {
                    app.exit(0);
                }
                "settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
```

---

#### P3-T8: 配置持久化

**文件**: `src-tauri/src/state/config.rs`

```rust
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub api: ApiConfig,
    pub audio: AudioConfig,
    pub hotkeys: HotkeyConfig,
    pub behavior: BehaviorConfig,
}

impl AppConfig {
    pub fn load(app: &AppHandle) -> Result<Self, ConfigError> {
        let path = Self::config_path(app)?;

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: Self = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, app: &AppHandle) -> Result<(), ConfigError> {
        let path = Self::config_path(app)?;

        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;

        Ok(())
    }

    fn config_path(app: &AppHandle) -> Result<PathBuf, ConfigError> {
        let app_data = app.path().app_config_dir()?;
        Ok(app_data.join("config.json"))
    }
}
```

---

## 6. Phase 4: 完善与优化

### 6.1 目标

性能优化、完善错误处理、多平台适配。

### 6.2 任务清单

#### P4-T1: 性能优化

| 优化项 | 实现方式 | 文件 |
|--------|----------|------|
| Ring Buffer | 使用 `ringbuf` crate | `audio/buffer.rs` |
| Buffer 复用 | 预分配 Vec 池 | `audio/pipeline.rs` |
| 批量发送 | 累积 100ms 数据 | `network/tasks.rs` |

#### P4-T2: 错误处理完善

**全局错误处理**:
```rust
// src-tauri/src/utils/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Input error: {0}")]
    Input(#[from] InputError),

    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
}

impl AppError {
    pub fn user_message(&self) -> String {
        match self {
            Self::Audio(AudioError::DeviceNotFound) =>
                "找不到麦克风设备，请检查音频设置".to_string(),
            Self::Network(NetworkError::AuthenticationFailed) =>
                "API Key 无效，请在设置中更新".to_string(),
            Self::Input(InputError::PermissionDenied) =>
                "需要辅助功能权限才能输入文本".to_string(),
            _ => self.to_string(),
        }
    }
}
```

#### P4-T3: macOS 适配

```rust
// src-tauri/src/input/platform/macos.rs
#[cfg(target_os = "macos")]
pub mod macos {
    use macos_accessibility_client::accessibility;

    pub fn check_accessibility_permission() -> bool {
        accessibility::application_is_trusted()
    }

    pub fn request_accessibility_permission() -> bool {
        accessibility::application_is_trusted_with_prompt()
    }
}
```

#### P4-T4: Linux 适配

```rust
// src-tauri/src/input/platform/linux.rs
#[cfg(target_os = "linux")]
pub mod linux {
    pub fn detect_display_server() -> DisplayServer {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            DisplayServer::Wayland
        } else {
            DisplayServer::X11
        }
    }

    pub enum DisplayServer {
        X11,
        Wayland,
    }
}
```

---

## 7. 验收标准

### 7.1 功能验收

| 功能 | 验收标准 |
|------|----------|
| 语音采集 | 能够检测到麦克风并采集音频 |
| 实时转写 | 说话时能看到 partial 文本更新 |
| 文本注入 | committed 后文本出现在目标应用 |
| 热键控制 | 按住热键录音，松开完成转写 |
| 取消操作 | Escape 能取消当前会话 |
| 系统托盘 | 托盘图标可见，菜单可操作 |
| 配置保存 | 设置修改后重启仍然生效 |

### 7.2 性能指标

| 指标 | 目标值 |
|------|--------|
| 首字延迟 | < 500ms |
| 内存占用 | < 100MB (空闲时) |
| CPU 占用 | < 5% (空闲时) |
| 热键响应 | < 50ms |

### 7.3 兼容性

| 平台 | 最低版本 |
|------|----------|
| Windows | Windows 10 1903+ |
| macOS | macOS 10.15+ |
| Linux | Ubuntu 22.04+ (X11) |

---

## 8. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| ElevenLabs API 变更 | 转写功能失效 | 抽象 API 层，便于切换 |
| 音频设备兼容性 | 部分用户无法使用 | 提供设备选择功能 |
| 系统权限问题 | 无法注入文本 | 提供清晰的权限引导 |
| 网络不稳定 | 转写中断 | 实现重连机制 |

---

## 9. 附录

### 9.1 开发环境要求

- Rust 1.75+
- Node.js 20+
- pnpm 8+
- Windows: Visual Studio Build Tools
- macOS: Xcode Command Line Tools
- Linux: `build-essential`, `libwebkit2gtk-4.1-dev`

### 9.2 测试命令

```bash
# 单元测试
cargo test --workspace

# 集成测试 (需要 API Key)
ELEVENLABS_API_KEY=xxx cargo test --workspace -- --ignored

# 前端测试
pnpm test

# 构建
cargo tauri build
```

---

*文档版本: 1.0.0*
*最后更新: 2025-12-21*
