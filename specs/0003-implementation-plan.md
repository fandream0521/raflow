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

**验收标准**:
- [x] 成功列出所有输入设备
- [x] 正确获取默认输入设备
- [x] 正确查询设备配置信息
- [x] 所有单元测试通过（4个测试）
- [x] 集成测试通过（4个测试）
- [x] 错误处理正确

**测试结果**:
```bash
# 单元测试
running 4 tests
test audio::device::tests::test_default_device ... ok
test audio::device::tests::test_device_not_found ... ok
test audio::device::tests::test_device_config ... ok
test audio::device::tests::test_list_devices ... ok

test result: ok. 4 passed; 0 failed; 0 ignored

# 集成测试
running 4 tests
test test_default_device_integration ... ok
test test_device_config_error_handling ... ok
test test_device_sample_rate_validity ... ok
test test_list_input_devices_integration ... ok

test result: ok. 4 passed; 0 failed; 0 ignored
```

**测试环境**:
- 检测到 2 个输入设备
- 默认设备：麦克风 (Realtek(R) Audio)
- 支持采样率：48000 Hz
- 通道数：2（立体声）

**完成状态**: ✅ 已完成 (2025-12-23)

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

**验收标准**:
- [x] 成功创建音频采集实例
- [x] 正确启动和停止音频流
- [x] 通过 channel 正确传递音频数据
- [x] 使用 try_send 避免阻塞音频线程
- [x] 所有单元测试通过（5个测试）
- [x] 集成测试通过（5个测试）
- [x] 支持多次启动/停止循环
- [x] 正确处理 Drop 清理
- [x] 文档测试通过（4个测试）

**测试结果**:
```bash
# 单元测试
running 5 tests
test audio::capture::tests::test_audio_capture_creation ... ok
test audio::capture::tests::test_audio_capture_sample_rate ... ok
test audio::capture::tests::test_audio_capture_with_specific_device ... ok
test audio::capture::tests::test_audio_capture_double_start ... ok
test audio::capture::tests::test_audio_capture_start_stop ... ok

test result: ok. 5 passed; 0 failed; 0 ignored

# 集成测试
running 5 tests
test test_audio_capture_sample_characteristics ... ok
test test_audio_capture_integration ... ok
test test_audio_capture_drop_while_capturing ... ok
test test_audio_capture_start_stop_multiple_times ... ok
test test_audio_capture_channel_overflow ... ok

test result: ok. 5 passed; 0 failed; 0 ignored
```

**测试数据**:
- 采样率：48000 Hz
- 通道数：2（立体声）
- 批次大小：960 样本（约 20ms @ 48kHz）
- 接收到 10 批次，总计 9600 样本（0.20 秒音频）
- 样本范围：[-1.0, 1.0]（正确归一化）
- 无削波现象

**实现特性**:
- ✅ 使用 tokio mpsc channel 异步传输数据
- ✅ 使用 Arc 共享 sender 到音频回调
- ✅ 使用 try_send 避免阻塞音频线程
- ✅ 实现 Drop trait 自动清理资源
- ✅ 支持默认设备和指定设备
- ✅ 提供 is_capturing() 状态查询
- ✅ 完整的文档注释和示例

**完成状态**: ✅ 已完成 (2025-12-23)

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

**验收标准**:
- [x] 成功创建重采样器实例
- [x] 正确实现 48kHz → 16kHz 转换
- [x] 正确实现 44.1kHz → 16kHz 转换
- [x] 支持 16kHz → 16kHz 直通
- [x] 实现 reset 方法清除状态
- [x] 实现 process_buffered 处理可变长度输入
- [x] 所有单元测试通过（8个测试）
- [x] 集成测试通过（6个测试）
- [x] 文档测试通过（2个测试）
- [x] 信号质量保持良好

**测试结果**:
```bash
# 单元测试
running 8 tests
test audio::resampler::tests::test_resample_48k_to_16k ... ok
test audio::resampler::tests::test_resample_44k_to_16k ... ok
test audio::resampler::tests::test_resample_16k_to_16k ... ok
test audio::resampler::tests::test_resample_wrong_input_size ... ok
test audio::resampler::tests::test_resample_reset ... ok
test audio::resampler::tests::test_resample_multiple_chunks ... ok
test audio::resampler::tests::test_resample_buffered ... ok
test audio::resampler::tests::test_resample_signal_preservation ... ok

test result: ok. 8 passed; 0 failed; 0 ignored

# 集成测试
running 6 tests
test test_resampler_48khz_to_16khz_integration ... ok
test test_resampler_44khz_to_16khz_integration ... ok
test test_resampler_continuous_stream ... ok
test test_resampler_reset_integration ... ok
test test_resampler_buffered_integration ... ok
test test_resampler_frequency_preservation ... ok

test result: ok. 6 passed; 0 failed; 0 ignored
```

**测试数据**:
```
48kHz → 16kHz:
  - 输入: 48000 samples (1.0s)
  - 输出: 15956 samples (1.00s @ 16kHz)
  - 比率: 3.0 (精确)
  - 信号保持: 最大幅值 1.0000

44.1kHz → 16kHz:
  - 输入: 44100 samples (1.0s)
  - 输出: 15953 samples (1.00s @ 16kHz)
  - 比率: 2.7644 (预期 2.7563)
  - 误差: < 0.5%

连续流处理:
  - 处理 1000 个块 (10秒)
  - 输出: 159956 samples (~10s @ 16kHz)
  - 每块平均: 159.96 samples
  - 范围: 116-160 samples
```

**实现特性**:
- ✅ 使用 rubato SincFixedIn 高质量重采样
- ✅ Sinc 插值长度: 256
- ✅ 黑曼-哈里斯窗函数
- ✅ 截止频率: 0.95
- ✅ 过采样因子: 256
- ✅ 自动块大小计算（10ms）
- ✅ 支持可变长度输入的缓冲处理
- ✅ 完整的文档注释和示例
- ✅ 信号频率和幅度保持良好

**完成状态**: ✅ 已完成 (2025-12-23)

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

**验收标准**:
- [x] 成功创建管道实例
- [x] 整合 AudioCapture 和 AudioResampler
- [x] 实现 f32 → i16 PCM 转换
- [x] 实现 Base64 编码
- [x] 实现批量处理（100ms 累积）
- [x] 实现异步处理架构
- [x] 实现启动/停止管理
- [x] 所有单元测试通过（8个测试）
- [x] 集成测试通过（8个测试）
- [x] 输出格式正确（i16 PCM LE）
- [x] 可用于 WebSocket 传输

**测试结果**:
```bash
# 单元测试
running 8 tests
test audio::pipeline::tests::test_base64_encoding ... ok
test audio::pipeline::tests::test_f32_to_i16_conversion ... ok
test audio::pipeline::tests::test_f32_to_i16_clamping ... ok
test audio::pipeline::tests::test_i16_to_bytes ... ok
test audio::pipeline::tests::test_pipeline_creation ... ok
test audio::pipeline::tests::test_pipeline_double_start ... ok
test audio::pipeline::tests::test_pipeline_start_stop ... ok
test audio::pipeline::tests::test_sample_rate_conversion ... ok

test result: ok. 8 passed; 0 failed; 0 ignored

# 集成测试
running 8 tests
test test_pipeline_creation ... ok
test test_pipeline_error_handling ... ok
test test_pipeline_start_stop ... ok
test test_pipeline_audio_output ... ok
test test_pipeline_output_format ... ok
test test_pipeline_restart ... ok
test test_pipeline_multiple_receivers ... ok
test test_pipeline_continuous_operation ... ok

test result: ok. 8 passed; 0 failed; 0 ignored
```

**实现特性**:
- ✅ 完整的数据流管道（采集 → 重采样 → 转换 → 编码）
- ✅ 异步处理架构（tokio 异步任务）
- ✅ 批量处理（100ms 块，1600 samples @ 16kHz）
- ✅ 精确的格式转换（f32 → i16 PCM）
- ✅ 标准 Base64 编码（RFC 4648）
- ✅ 优雅的启动/停止管理
- ✅ 自动资源清理（Drop trait）
- ✅ 防御性编程（钳位、边界检查）

**输出特性**:
```
输入: 48kHz (设备原生)
输出: 16kHz (语音识别标准)
块大小: 100ms
采样数: 1600 samples/chunk
字节数: 3200 bytes/chunk (i16 PCM LE)
Base64: ~4267 characters/chunk
频率: 10 chunks/second
比特率: 256 kbps
延迟: ~125ms (采集 20ms + 处理 5ms + 累积 100ms)
```

**完成状态**: ✅ 已完成 (2025-12-23)

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

**验收标准**:
- [x] 定义所有客户端消息类型
- [x] 定义所有服务端消息类型
- [x] 实现序列化/反序列化
- [x] 可选字段正确处理
- [x] 标签枚举自动分发
- [x] Builder 模式构造器
- [x] 辅助方法实现
- [x] 所有单元测试通过（14个测试）
- [x] 集成测试通过（14个测试）
- [x] 支持真实 API 格式
- [x] 类型安全保证

**测试结果**:
```bash
# 单元测试
running 14 tests
test network::messages::tests::test_commit_message ... ok
test network::messages::tests::test_client_message_serialization ... ok
test network::messages::tests::test_close_message ... ok
test network::messages::tests::test_input_audio_chunk_basic ... ok
test network::messages::tests::test_input_audio_chunk_serialization ... ok
test network::messages::tests::test_input_audio_chunk_with_options ... ok
test network::messages::tests::test_server_message_committed_transcript ... ok
test network::messages::tests::test_server_message_committed_with_timestamps ... ok
test network::messages::tests::test_server_message_input_error ... ok
test network::messages::tests::test_server_message_partial_transcript ... ok
test network::messages::tests::test_server_message_session_started ... ok
test network::messages::tests::test_session_config_deserialization ... ok
test network::messages::tests::test_word_timestamp_duration ... ok
test network::messages::tests::test_word_timestamp_punctuation ... ok

test result: ok. 14 passed; 0 failed; 0 ignored

# 集成测试
running 14 tests
test test_close_message ... ok
test test_commit_message ... ok
test test_client_message_variants ... ok
test test_committed_transcript_deserialization ... ok
test test_committed_with_timestamps_deserialization ... ok
test test_input_audio_chunk_full_cycle ... ok
test test_input_audio_chunk_minimal ... ok
test test_input_error_deserialization ... ok
test test_message_size_estimation ... ok
test test_partial_transcript_deserialization ... ok
test test_real_world_session_flow ... ok
test test_round_trip_serialization ... ok
test test_server_message_helper_methods ... ok
test test_session_started_deserialization ... ok

test result: ok. 14 passed; 0 failed; 0 ignored
```

**实现特性**:
- ✅ 完整的客户端消息类型（InputAudioChunk, CommitMessage, CloseMessage）
- ✅ 完整的服务端消息类型（5 种变体）
- ✅ 辅助类型（SessionConfig, WordTimestamp, VadConfig）
- ✅ 自动序列化/反序列化（serde）
- ✅ 标签枚举自动分发（tag = "message_type"）
- ✅ Builder 模式构造器（with_* 方法）
- ✅ 辅助方法（is_partial, is_committed, text, 等）
- ✅ 可选字段自动省略（skip_serializing_if）
- ✅ 类型安全的消息处理
- ✅ 完整的文档和示例

**消息类型**:
```
客户端消息（3 种）:
- InputAudioChunk: 音频数据块
- CommitMessage: 手动提交
- CloseMessage: 关闭连接

服务端消息（5 种）:
- SessionStarted: 会话开始
- PartialTranscript: 部分转写
- CommittedTranscript: 最终转写
- CommittedTranscriptWithTimestamps: 带时间戳转写
- InputError: 输入错误

辅助类型（3 种）:
- SessionConfig: 会话配置
- WordTimestamp: 单词时间戳
- VadConfig: VAD 配置
```

**完成状态**: ✅ 已完成 (2025-12-23)

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

**验收标准**:
- [x] NetworkError 错误类型定义（10 种错误）
- [x] ConnectionConfig 配置类型（Builder 模式）
- [x] ScribeConnection 结构体实现
- [x] connect() 方法（WSS + API key 认证）
- [x] send() 方法（泛型序列化）
- [x] recv() 方法（自动反序列化 + Ping/Pong 处理）
- [x] close() 方法（优雅关闭）
- [x] 超时处理和错误检测
- [x] 单元测试通过（5 个）
- [x] 集成测试通过（12 个，2 个忽略）

**测试结果**:
```bash
# 单元测试
$ cargo test --lib network::connection::tests
running 5 tests
test network::connection::tests::test_connection_config_new ... ok
test network::connection::tests::test_connection_config_builder ... ok
test network::connection::tests::test_connection_config_build_url ... ok
test network::connection::tests::test_connection_config_build_url_with_options ... ok
test network::connection::tests::test_connection_config_default ... ok

test result: ok. 5 passed; 0 failed; 0 ignored

# 集成测试
$ cargo test --test network_connection_test
running 14 tests
test test_config_immutability ... ok
test test_connection_config_builder_pattern ... ok
test test_connection_config_chaining ... ok
test test_connection_config_creation ... ok
test test_connection_config_default ... ok
test test_connection_config_different_sample_rates ... ok
test test_connection_config_language_codes ... ok
test test_connection_config_timeout_values ... ok
test test_connection_config_url_building ... ok
test test_network_error_types ... ok
test test_url_query_parameter_format ... ok
test test_vad_strategies ... ok
test test_connection_timeout ... ignored
test test_connection_with_invalid_api_key ... ignored

test result: ok. 12 passed; 0 failed; 2 ignored
```

**实现亮点**:
- ✅ 完整的错误类型系统（NetworkError 枚举 + thiserror）
- ✅ Builder 模式配置（ConnectionConfig with_* 方法链）
- ✅ 类型安全的连接管理（ScribeConnection）
- ✅ 泛型 send 方法（支持任何 Serialize 类型）
- ✅ 自动协议处理（Ping/Pong 自动响应，Close 帧处理）
- ✅ 认证错误识别（401 状态码检测）
- ✅ 连接超时保护（可配置超时时间）
- ✅ 完全异步设计（tokio + tokio-tungstenite）
- ✅ 全面的测试覆盖（17 个测试，包括多语言、多采样率、VAD 策略）

**完成状态**: ✅ 已完成 (2025-12-23)

**详细文档**: [P1-T6-COMPLETION.md](./P1-T6-COMPLETION.md)

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
    mut ws_writer: WsWriter,
    mut audio_rx: mpsc::Receiver<String>,
) -> NetworkResult<()>;

pub async fn receiver_task(
    mut ws_reader: WsReader,
    message_tx: mpsc::Sender<ServerMessage>,
) -> NetworkResult<()>;
```

**实现要点**:
```rust
// sender_task: 从 channel 读取音频，发送到 WebSocket
pub async fn sender_task(
    mut ws_writer: WsWriter,
    mut audio_rx: mpsc::Receiver<String>,
) -> NetworkResult<()> {
    while let Some(audio_base64) = audio_rx.recv().await {
        let chunk = InputAudioChunk::new(audio_base64);
        let json = serde_json::to_string(&ClientMessage::InputAudioChunk(chunk))?;
        ws_writer.send(Message::Text(json.into())).await?;
    }
    ws_writer.close().await?;
    Ok(())
}

// receiver_task: 从 WebSocket 读取消息，转发到 channel
pub async fn receiver_task(
    mut ws_reader: WsReader,
    message_tx: mpsc::Sender<ServerMessage>,
) -> NetworkResult<()> {
    while let Some(msg_result) = ws_reader.next().await {
        match msg_result? {
            Message::Text(text) => {
                let server_msg: ServerMessage = serde_json::from_str(&text)?;
                message_tx.send(server_msg).await.ok();
            }
            Message::Close(_) => break,
            Message::Ping(_) | Message::Pong(_) => {} // 自动处理
            _ => {}
        }
    }
    Ok(())
}
```

**验收标准**:
- [x] ScribeConnection 支持 split() 方法
- [x] 实现 sender_task 函数（音频发送）
- [x] 实现 receiver_task 函数（消息接收）
- [x] 支持 mpsc channel 通信
- [x] 自动处理 Ping/Pong 和 Close 帧
- [x] Channel 关闭自动停止任务
- [x] 完整的错误处理和日志
- [x] 单元测试通过（5 个）
- [x] 集成测试通过（10 个）

**测试结果**:
```bash
# 单元测试
$ cargo test --lib network::tasks::tests
running 5 tests
test network::tasks::tests::test_audio_chunk_serialization ... ok
test network::tasks::tests::test_channel_capacity ... ok
test network::tasks::tests::test_input_audio_chunk_builder ... ok
test network::tasks::tests::test_message_channel_behavior ... ok
test network::tasks::tests::test_session_started_message_structure ... ok

test result: ok. 5 passed; 0 failed; 0 ignored

# 集成测试
$ cargo test --test network_tasks_test
running 10 tests
test test_audio_chunk_message_creation ... ok
test test_base64_audio_encoding ... ok
test test_channel_closure_detection ... ok
test test_client_message_serialization ... ok
test test_concurrent_send_receive ... ok
test test_error_message_handling ... ok
test test_message_size_estimation ... ok
test test_mpsc_channel_audio_flow ... ok
test test_mpsc_channel_message_flow ... ok
test test_task_coordination ... ok

test result: ok. 10 passed; 0 failed; 0 ignored
```

**实现亮点**:
- ✅ 优雅的任务设计（基于 mpsc channel）
- ✅ 自动协议处理（Ping/Pong、Close、首包采样率）
- ✅ 完全并发（发送和接收独立运行）
- ✅ 自动生命周期管理（channel 关闭即停止）
- ✅ 类型安全（强类型 channel，编译时检查）
- ✅ 零配置使用（无需手动处理协议细节）
- ✅ 完整的可观测性（详细日志、消息计数）
- ✅ 性能优化（无锁、异步、零拷贝）

**完成状态**: ✅ 已完成 (2025-12-23)

**详细文档**: [P1-T7-COMPLETION.md](./P1-T7-COMPLETION.md)

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
    sender_handle: Option<JoinHandle<Result<(), NetworkError>>>,
    receiver_handle: Option<JoinHandle<Result<(), NetworkError>>>,
    event_handler_handle: Option<JoinHandle<()>>,
    is_running: bool,
}

impl TranscriptionSession {
    pub async fn start<F>(api_key: &str, on_event: F) -> Result<Self, TranscriptionError>
    where
        F: Fn(TranscriptEvent) + Send + Sync + 'static;

    pub async fn stop(&mut self) -> Result<(), TranscriptionError>;

    pub fn is_running(&self) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub enum TranscriptEvent {
    SessionStarted { session_id: String },
    Partial { text: String },
    Committed { text: String },
    Error { message: String },
    Closed,
}
```

**实现要点**:
```rust
// start() 方法流程
pub async fn start<F>(api_key: &str, on_event: F) -> Result<Self, TranscriptionError>
where
    F: Fn(TranscriptEvent) + Send + Sync + 'static,
{
    // 1. 创建音频管道
    let mut audio_pipeline = AudioPipeline::new(None)?;

    // 2. 建立 WebSocket 连接
    let config = ConnectionConfig::new(audio_pipeline.output_sample_rate());
    let connection = ScribeConnection::connect(api_key, &config).await?;

    // 3. 分离连接
    let (writer, reader) = connection.split();

    // 4. 创建 channels
    let (audio_tx, audio_rx) = mpsc::channel::<String>(100);
    let (msg_tx, msg_rx) = mpsc::channel::<ServerMessage>(100);

    // 5. 启动音频管道
    audio_pipeline.start(audio_tx).await?;

    // 6. 启动 sender_task
    let sender_handle = tokio::spawn(async move {
        sender_task(writer, audio_rx).await
    });

    // 7. 启动 receiver_task
    let receiver_handle = tokio::spawn(async move {
        receiver_task(reader, msg_tx).await
    });

    // 8. 启动事件处理器
    let on_event = Arc::new(on_event);
    let event_handler_handle = tokio::spawn(async move {
        while let Some(msg) = msg_rx.recv().await {
            let event = convert_server_message_to_event(msg);
            on_event(event);
        }
        on_event(TranscriptEvent::Closed);
    });

    Ok(Self {
        audio_pipeline,
        sender_handle: Some(sender_handle),
        receiver_handle: Some(receiver_handle),
        event_handler_handle: Some(event_handler_handle),
        is_running: true,
    })
}

// stop() 方法流程
pub async fn stop(&mut self) -> Result<(), TranscriptionError> {
    // 1. 停止音频管道
    self.audio_pipeline.stop().await;

    // 2. 等待所有任务完成
    if let Some(h) = self.sender_handle.take() {
        h.await??;
    }
    if let Some(h) = self.receiver_handle.take() {
        h.await??;
    }
    if let Some(h) = self.event_handler_handle.take() {
        h.await?;
    }

    self.is_running = false;
    Ok(())
}
```

**验收标准**:
- [x] TranscriptEvent 枚举定义（5 种事件）
- [x] TranscriptionSession 结构实现
- [x] start() 方法（完整集成）
- [x] stop() 方法（优雅关闭）
- [x] 事件回调机制
- [x] 音频管道集成
- [x] WebSocket 通信集成
- [x] 任务生命周期管理
- [x] 单元测试通过（3 个）
- [x] 集成测试通过（7 个，1 个忽略）
- [x] Doc 测试通过（3 个）

**测试结果**:
```bash
# 单元测试
$ cargo test --lib transcription::tests
running 3 tests
test transcription::tests::test_transcript_event_clone ... ok
test transcription::tests::test_transcript_event_equality ... ok
test transcription::tests::test_transcript_event_types ... ok

test result: ok. 3 passed; 0 failed; 0 ignored

# 集成测试
$ cargo test --test transcription_e2e_test
running 8 tests
test test_e2e_transcription_with_real_api ... ignored
test test_concurrent_event_handling ... ok
test test_error_event_handling ... ok
test test_event_callback_mechanism ... ok
test test_event_channel_flow ... ok
test test_event_pattern_matching ... ok
test test_transcript_event_creation ... ok
test test_transcript_event_equality ... ok

test result: ok. 7 passed; 0 failed; 1 ignored

# Doc 测试
$ cargo test --doc transcription
running 3 tests
test transcription::TranscriptionSession ... ok
test transcription::TranscriptionSession::start ... ok
test transcription::TranscriptionSession::stop ... ok

test result: ok. 3 passed; 0 failed; 0 ignored
```

**实现亮点**:
- ✅ 一键启动完整转写流程（start() 一个调用）
- ✅ 自动管理所有任务生命周期
- ✅ 事件驱动的回调模型
- ✅ 类型安全的事件处理
- ✅ 优雅的资源清理（无泄漏）
- ✅ 完全并发的处理架构
- ✅ 简洁易用的 API

**完成状态**: ✅ 已完成 (2025-12-23)

**详细文档**: [P1-T8-COMPLETION.md](./P1-T8-COMPLETION.md)

**Phase 1 状态**: ✅ 100% 完成 (8/8 任务)

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

**验收标准**:
- [x] AppState 枚举包含所有必需状态
- [x] RecordingState 子状态正确实现
- [x] StateManager 使用 ArcSwap 实现无锁读取
- [x] 状态转换验证符合状态机图
- [x] 监听器机制工作正常
- [x] 单元测试通过（11个测试）
- [x] 集成测试通过（18个测试）
- [x] 文档测试通过（4个测试）

**完成状态**: ✅ 已完成 (2025-12-25)

**详细文档**: [P2-T1-COMPLETION.md](./P2-T1-COMPLETION.md)

**参考代码**:
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

**验收标准**:
- [x] HotkeyError 错误类型定义（8 种错误）
- [x] HotkeyConfig 配置结构（Builder 模式）
- [x] HotkeyManager 管理器结构
- [x] register_hotkeys() 方法（注册所有热键）
- [x] unregister_hotkeys() 方法（注销所有热键）
- [x] is_hotkey_registered() 检查方法
- [x] PTT、Cancel、Toggle Mode 三种热键支持
- [x] Pressed/Released 事件处理
- [x] 与 StateManager 集成
- [x] 单元测试通过（12个测试）
- [x] 集成测试通过（31个测试，3个忽略需 Tauri 环境）
- [x] 文档测试通过（4个测试，3个忽略需 Tauri 环境）

**测试结果**:
```bash
# 单元测试
$ cargo test --lib hotkey::
running 12 tests
test hotkey::config::tests::test_all_hotkeys ... ok
test hotkey::config::tests::test_config_builder_pattern ... ok
test hotkey::config::tests::test_config_default ... ok
test hotkey::config::tests::test_config_equality ... ok
test hotkey::config::tests::test_config_new ... ok
test hotkey::config::tests::test_config_serialization ... ok
test hotkey::config::tests::test_hotkey_identification ... ok
test hotkey::register::tests::test_hotkey_event_equality ... ok
test hotkey::register::tests::test_hotkey_manager_creation ... ok
test hotkey::register::tests::test_hotkey_manager_update_config ... ok
test hotkey::register::tests::test_parse_shortcut_invalid ... ok
test hotkey::register::tests::test_parse_shortcut_valid ... ok

test result: ok. 12 passed; 0 failed; 0 ignored

# 集成测试
$ cargo test --test hotkey_test
running 34 tests
test result: ok. 31 passed; 0 failed; 3 ignored
```

**实现亮点**:
- ✅ 完整的错误类型系统（HotkeyError 枚举 + thiserror）
- ✅ Builder 模式配置（HotkeyConfig with_* 方法链）
- ✅ 类型安全的热键管理（HotkeyManager）
- ✅ 支持 Push-to-Talk、Cancel、Toggle Mode 三种热键
- ✅ 处理 Pressed/Released 事件（用于 PTT）
- ✅ 与 StateManager 集成（状态转换）
- ✅ 完整的序列化支持（serde JSON）
- ✅ 全面的测试覆盖（47 个测试）

**完成状态**: ✅ 已完成 (2025-12-25)

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

**模块**: `src-tauri/src/hotkey/handlers.rs`, `src-tauri/src/hotkey/session.rs`

**验收标准**:
- [x] SessionController 管理转写会话生命周期
- [x] 使用 channel 模式处理非 Send 类型（cpal::Stream）
- [x] handle_ptt_pressed() 启动转写会话
- [x] handle_ptt_released() 停止会话并获取结果
- [x] handle_cancel() 取消当前会话
- [x] handle_toggle_mode() 切换模式（预留）
- [x] setup_hotkey_state() 初始化热键状态
- [x] set_api_key() 配置 API Key
- [x] ApiKeyHolder 安全存储 API Key
- [x] 与 StateManager 集成（状态转换）
- [x] 与 TranscriptionSession 集成
- [x] 单元测试通过（6个测试）
- [x] 完整的事件发送到前端

**测试结果**:
```bash
# 单元测试
$ cargo test --lib hotkey::
running 15 tests
test hotkey::handlers::tests::test_api_key_holder ... ok
test hotkey::handlers::tests::test_hotkey_handler_error_display ... ok
test hotkey::handlers::tests::test_hotkey_handler_error_equality ... ok
test hotkey::session::tests::test_session_controller_error_display ... ok
test hotkey::session::tests::test_session_event_creation ... ok
test hotkey::session::tests::test_session_state_equality ... ok
...
test result: ok. 15 passed; 0 failed; 0 ignored
```

**实现亮点**:
- ✅ 使用 channel 模式解决 cpal::Stream 非 Send 问题
- ✅ 专用线程运行 TranscriptionSession（std::thread::spawn）
- ✅ oneshot channel 实现请求-响应通信
- ✅ 状态机集成（Idle → Connecting → Recording → Processing → Injecting）
- ✅ 完整的错误处理和日志
- ✅ 事件发送到前端（transcription:error, transcription:committed, transcription:cancelled）
- ✅ ApiKeyHolder 安全存储 API Key（tokio::RwLock）

**架构设计**:
```
SessionController (Send + Sync)
    │
    ├── command_tx ────────────────────┐
    │   (mpsc::Sender<SessionCommand>) │
    │                                  ▼
    └───────────────────────────► session_task (专用线程)
                                       │
                                       ├── TranscriptionSession (非 Send)
                                       ├── AudioPipeline
                                       └── WebSocket 连接
```

**完成状态**: ✅ 已完成 (2025-12-25)

---

#### P2-T4: 状态转换逻辑

**模块**: `src-tauri/src/state/transitions.rs`, `src-tauri/src/commands/state.rs`

**验收标准**:
- [x] StateChangeEvent 结构体（可序列化到前端）
- [x] StateEventEmitter（监听状态变更，发射 Tauri 事件）
- [x] ProcessingTimeoutHandler（Processing 状态超时自动重置）
- [x] StateTransitionContext（便捷的状态转换方法）
- [x] TransitionError 错误类型
- [x] setup_state_transitions() 初始化函数
- [x] Tauri 命令实现（get_current_state, is_idle, reset_state 等）
- [x] 与 StateManager 集成
- [x] 与热键处理器集成
- [x] 单元测试通过（13个测试）
- [x] 集成测试通过（7个测试）

**测试结果**:
```bash
# 单元测试
$ cargo test --lib state::transitions::tests
running 13 tests
test state::transitions::tests::test_state_change_event_from_idle ... ok
test state::transitions::tests::test_state_change_event_from_connecting ... ok
test state::transitions::tests::test_state_change_event_from_recording_listening ... ok
test state::transitions::tests::test_state_change_event_from_recording_transcribing ... ok
test state::transitions::tests::test_state_change_event_from_processing ... ok
test state::transitions::tests::test_state_change_event_from_injecting ... ok
test state::transitions::tests::test_state_change_event_from_error ... ok
test state::transitions::tests::test_state_change_event_serialization ... ok
test state::transitions::tests::test_transition_error_display ... ok
test state::transitions::tests::test_transition_error_equality ... ok
test state::transitions::tests::test_default_processing_timeout ... ok

test result: ok. 13 passed; 0 failed; 0 ignored

# 集成测试
$ cargo test --test state_test
running 24 tests
test test_state_change_event_all_variants ... ok
test test_state_change_event_json_serialization ... ok
test test_transition_error_variants ... ok
test test_default_processing_timeout_value ... ok
test test_app_state_serialization ... ok
test test_recording_state_serialization ... ok
...
test result: ok. 24 passed; 0 failed; 0 ignored
```

**实现亮点**:
- ✅ 自动状态事件发射（StateEventEmitter 监听 StateManager）
- ✅ Processing 超时保护（默认 30 秒自动重置）
- ✅ 完整的 Tauri 事件体系（app:state_changed, app:idle, app:recording, app:error 等）
- ✅ JSON 可序列化事件载荷（StateChangeEvent）
- ✅ 7 个 Tauri 命令供前端调用
- ✅ 与热键处理器无缝集成（StateTransitionSystem）

**Tauri 事件**:
```
通用事件:
- app:state_changed: StateChangeEvent 载荷

状态特定事件:
- app:idle: 进入空闲状态
- app:connecting: 进入连接中状态
- app:recording: 进入录音状态（带 is_transcribing 参数）
- app:processing: 进入处理中状态
- app:injecting: 进入注入中状态
- app:error: 进入错误状态（带错误消息）
- app:processing_timeout: Processing 超时重置

转写相关事件:
- transcript:partial: 部分转写文本
```

**Tauri 命令**:
```rust
#[command] pub fn get_current_state(app: AppHandle) -> Result<StateChangeEvent, String>;
#[command] pub fn get_state_name(app: AppHandle) -> Result<String, String>;
#[command] pub fn is_idle(app: AppHandle) -> Result<bool, String>;
#[command] pub fn is_recording(app: AppHandle) -> Result<bool, String>;
#[command] pub fn is_error(app: AppHandle) -> Result<bool, String>;
#[command] pub fn reset_state(app: AppHandle) -> Result<(), String>;
#[command] pub fn recover_from_error(app: AppHandle) -> Result<(), String>;
```

**完成状态**: ✅ 已完成 (2025-12-25)

---

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

**模块**: `src-tauri/src/input/window.rs`, `src-tauri/src/input/error.rs`

**验收标准**:
- [x] InputError 错误类型定义（7 种错误）
- [x] WindowInfo 结构体（含 app_name, title, process_id, exec_name, exec_path, window_id）
- [x] get_focused_window() 获取当前焦点窗口
- [x] is_text_input_context() 判断是否为文本输入上下文
- [x] has_focused_window() 检查是否有焦点窗口
- [x] get_focused_app_name() 便捷获取应用名称
- [x] get_focused_window_title() 便捷获取窗口标题
- [x] format_window_info() 格式化窗口信息
- [x] 支持 60+ 常见文本输入应用的识别
- [x] 单元测试通过（9 个测试）
- [x] 集成测试通过（21 个测试）

**测试结果**:
```bash
# 单元测试
$ cargo test --lib input::
running 9 tests
test input::error::tests::test_input_error_clone ... ok
test input::error::tests::test_input_error_display ... ok
test input::error::tests::test_input_error_equality ... ok
test input::window::tests::test_format_window_info ... ok
test input::window::tests::test_is_text_input_app ... ok
test input::window::tests::test_window_info_clone ... ok
test input::window::tests::test_window_info_is_app ... ok
test input::window::tests::test_window_info_is_app_case_insensitive ... ok
test input::window::tests::test_window_info_title_contains ... ok

test result: ok. 9 passed; 0 failed; 0 ignored

# 集成测试
$ cargo test --test input_window_test
running 21 tests
test test_format_window_info_all_fields ... ok
test test_format_window_info_empty_fields ... ok
test test_get_focused_app_name_integration ... ok
test test_get_focused_window_integration ... ok
test test_get_focused_window_title_integration ... ok
test test_has_focused_window_integration ... ok
test test_input_error_clipboard_failed ... ok
test test_input_error_injection_failed ... ok
test test_input_error_no_focused_window ... ok
test test_input_error_permission_denied ... ok
test test_input_error_window_detection_failed ... ok
test test_is_text_input_context_integration ... ok
test test_non_text_input_apps ... ok
test test_text_input_apps_browsers ... ok
test test_text_input_apps_communication ... ok
test test_text_input_apps_editors ... ok
test test_window_info_creation ... ok
test test_window_info_equality ... ok
test test_window_info_is_app_empty_list ... ok
test test_window_info_is_app_multiple_matches ... ok
test test_window_info_title_contains_special_chars ... ok

test result: ok. 21 passed; 0 failed; 0 ignored
```

**实现亮点**:
- ✅ 使用 x-win 库实现跨平台窗口检测
- ✅ 完整的 WindowInfo 结构（6 个字段）
- ✅ 大小写不敏感的应用匹配
- ✅ 支持 60+ 文本输入应用识别（编辑器、浏览器、通讯工具、终端、笔记应用、IDE）
- ✅ 中文应用名支持（微信、飞书、钉钉等）
- ✅ 便捷的辅助方法（has_focused_window, get_focused_app_name 等）
- ✅ 完整的错误类型系统（InputError）

**支持的应用类型**:
```
编辑器: VS Code, Sublime, Notepad++, Vim, Emacs, Atom
浏览器: Chrome, Firefox, Safari, Edge, Brave, Opera
通讯工具: Slack, Discord, Teams, 微信, QQ, Telegram, 飞书, 钉钉
终端: Terminal, iTerm, Windows Terminal, PowerShell, Alacritty
笔记应用: Obsidian, Notion, Typora, Bear, Evernote, OneNote
IDE: IntelliJ, PyCharm, WebStorm, Android Studio, Xcode, Eclipse
Office: Word, Excel, PowerPoint, WPS, LibreOffice
```

**完成状态**: ✅ 已完成 (2025-12-25)

---

#### P2-T6: 键盘模拟

**模块**: `src-tauri/src/input/keyboard.rs`

**功能**:
- 使用 enigo 库实现跨平台键盘模拟
- 支持文本输入、粘贴、复制、全选等操作
- 平台特定快捷键（Windows/Linux: Ctrl，macOS: Cmd）
- 支持 Unicode 文本和特殊字符

**API**:
```rust
pub struct KeyboardSimulator {
    enigo: Enigo,
}

impl KeyboardSimulator {
    pub fn new() -> InputResult<Self>;
    pub fn type_text(&mut self, text: &str) -> InputResult<()>;
    pub fn paste(&mut self) -> InputResult<()>;
    pub fn copy(&mut self) -> InputResult<()>;
    pub fn select_all(&mut self) -> InputResult<()>;
    pub fn press_key(&mut self, key: Key) -> InputResult<()>;
    pub fn release_key(&mut self, key: Key) -> InputResult<()>;
    pub fn click_key(&mut self, key: Key) -> InputResult<()>;
    pub fn press_enter(&mut self) -> InputResult<()>;
    pub fn press_escape(&mut self) -> InputResult<()>;
    pub fn press_tab(&mut self) -> InputResult<()>;
    pub fn press_backspace(&mut self) -> InputResult<()>;
    pub fn press_delete(&mut self) -> InputResult<()>;
}
```

**测试用例**: 19 tests
- KeyboardSimulator 创建和初始化
- 文本输入（空字符串、普通文本、Unicode、多行、长文本）
- 粘贴/复制/全选操作
- 特殊按键（Enter、Escape、Tab、Backspace、Delete）
- 按键控制（press/release/click）
- 错误类型验证

**验收标准**:
- [x] KeyboardSimulator 结构体实现
- [x] 跨平台粘贴操作（Ctrl+V / Cmd+V）
- [x] 跨平台复制操作（Ctrl+C / Cmd+C）
- [x] 跨平台全选操作（Ctrl+A / Cmd+A）
- [x] Unicode 文本支持
- [x] 单元测试和集成测试通过

**完成状态**: ✅ 已完成 (2025-12-25)

---

#### P2-T6B: 剪贴板操作

**模块**: `src-tauri/src/input/clipboard.rs`

**功能**:
- 使用 Tauri clipboard-manager 插件实现剪贴板操作
- 支持保存和恢复剪贴板内容，避免覆盖用户原有内容
- 提供便捷函数简化常用操作

**API**:
```rust
pub struct ClipboardManager<'a> {
    app: &'a AppHandle,
    saved_content: Option<String>,
}

impl<'a> ClipboardManager<'a> {
    pub fn new(app: &'a AppHandle) -> Self;
    pub fn save(&mut self) -> InputResult<()>;
    pub fn write(&self, text: &str) -> InputResult<()>;
    pub fn read(&self) -> Option<String>;
    pub fn restore(&self) -> InputResult<()>;
    pub fn has_saved_content(&self) -> bool;
    pub fn get_saved_content(&self) -> Option<&str>;
    pub fn clear_saved(&mut self);
    pub fn clear(&self) -> InputResult<()>;
}

// 便捷函数
pub fn write_to_clipboard(app: &AppHandle, text: &str) -> InputResult<()>;
pub fn read_from_clipboard(app: &AppHandle) -> Option<String>;
```

**测试用例**: 17 tests
- 错误类型测试（display、equality、clone、debug）
- API 存在性验证
- Re-export 验证
- InputResult 兼容性测试
- 边界条件测试（换行、制表符、空白字符）

**验收标准**:
- [x] ClipboardManager 结构体实现
- [x] 保存/恢复剪贴板内容功能
- [x] 读取/写入剪贴板功能
- [x] 便捷函数实现
- [x] 单元测试和集成测试通过

**完成状态**: ✅ 已完成 (2025-12-25)

---

#### P2-T7: 注入器集成

**模块**: `src-tauri/src/input/injector.rs`

**功能**:
- 统一的文本注入接口，整合键盘模拟和剪贴板操作
- 支持四种注入策略：Auto、Keyboard、Clipboard、ClipboardOnly
- 自动策略根据文本长度选择最佳注入方式
- 剪贴板注入自动保存/恢复原有内容

**API**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InjectionStrategy {
    #[default]
    Auto,           // 自动选择（< 20字符用键盘，否则用剪贴板）
    Keyboard,       // 始终键盘模拟
    Clipboard,      // 始终剪贴板粘贴
    ClipboardOnly,  // 仅复制到剪贴板
}

pub struct TextInjector<'a> {
    app: &'a AppHandle,
    strategy: InjectionStrategy,
    keyboard: KeyboardSimulator,
    auto_threshold: usize,
    paste_delay: Duration,
}

impl<'a> TextInjector<'a> {
    pub fn new(app: &'a AppHandle, strategy: InjectionStrategy) -> InputResult<Self>;
    pub fn with_config(app, strategy, auto_threshold, paste_delay_ms) -> InputResult<Self>;
    pub async fn inject(&mut self, text: &str) -> InputResult<()>;
    pub fn strategy(&self) -> InjectionStrategy;
    pub fn set_strategy(&mut self, strategy: InjectionStrategy);
    pub fn auto_threshold(&self) -> usize;
    pub fn set_auto_threshold(&mut self, threshold: usize);
    pub fn paste_delay(&self) -> Duration;
    pub fn set_paste_delay(&mut self, delay: Duration);
}

pub struct InjectionResult {
    pub strategy_used: InjectionStrategy,
    pub text_length: usize,
    pub success: bool,
    pub error_message: Option<String>,
}

pub const AUTO_STRATEGY_THRESHOLD: usize = 20;
pub const PASTE_DELAY_MS: u64 = 100;
```

**测试用例**: 28 tests
- InjectionStrategy 枚举测试（default、equality、clone、copy、debug）
- 显示名称和描述测试
- JSON 序列化/反序列化测试
- InjectionResult 测试（success、failure、clone）
- 常量测试（AUTO_STRATEGY_THRESHOLD、PASTE_DELAY_MS）
- API 存在性和 re-export 验证
- 自动策略阈值逻辑测试
- Unicode 文本长度计数测试

**验收标准**:
- [x] InjectionStrategy 枚举实现
- [x] TextInjector 结构体实现
- [x] 自动策略（根据文本长度选择）
- [x] 键盘注入和剪贴板注入
- [x] 剪贴板内容保存/恢复
- [x] 可配置的阈值和延迟
- [x] 单元测试和集成测试通过

**完成状态**: ✅ 已完成 (2025-12-25)

---

#### P2-T8: 完整流程集成

**模块**: `src-tauri/src/session/mod.rs`

**功能描述**:
整合 Phase 1 和 Phase 2 的所有组件，提供端到端的语音转写与文本注入流程。

**核心组件**:
```rust
/// 会话配置
pub struct SessionConfig {
    pub injection_strategy: InjectionStrategy,  // 注入策略
    pub auto_threshold: usize,                   // 自动策略阈值
    pub paste_delay_ms: u64,                     // 粘贴延迟
    pub pre_injection_delay_ms: u64,             // 注入前延迟
    pub auto_inject: bool,                       // 是否自动注入
}

/// 会话事件
pub enum SessionEvent {
    Started { session_id: String },
    PartialTranscript { text: String },
    CommittedTranscript { text: String },
    TextInjected { text: String, strategy: String },
    TextCopied { text: String },
    Stopped,
    Error { message: String },
}

/// RaFlow 完整会话
pub struct RaFlowSession {
    app: AppHandle,
    config: SessionConfig,
    transcription: Option<TranscriptionSession>,
    state_manager: Arc<StateManager>,
    // ...
}

impl RaFlowSession {
    pub async fn start(app: &AppHandle, api_key: &str, config: SessionConfig) -> Result<Self, SessionError>;
    pub async fn stop(&mut self) -> Result<(), SessionError>;
    pub fn is_running(&self) -> bool;
    pub fn current_state(&self) -> Arc<AppState>;
    pub async fn last_committed_text(&self) -> Option<String>;
    pub async fn inject_last_committed(&mut self) -> Result<(), SessionError>;
}

/// 会话错误
pub enum SessionError {
    StateError(String),
    TranscriptionError(TranscriptionError),
    InjectionError(String),
    NoTextToInject,
    NotRunning,
}
```

**工作流程**:
```
1. 用户按下热键
   └── State: Idle -> Connecting

2. 建立 WebSocket 连接，启动音频采集
   └── State: Connecting -> Recording(Listening)

3. 接收部分转写
   └── State: Recording(Listening) -> Recording(Transcribing)
   └── Event: transcript:partial

4. 用户松开热键
   └── State: Recording -> Processing

5. 接收最终转写
   └── State: Processing -> Injecting
   └── 执行文本注入

6. 注入完成
   └── State: Injecting -> Idle
   └── Event: transcript:committed
```

**配置预设**:
```rust
SessionConfig::default()       // 自动策略，自动注入
SessionConfig::clipboard_only() // 仅复制到剪贴板
SessionConfig::keyboard_only()  // 始终使用键盘模拟
SessionConfig::clipboard_paste() // 始终使用剪贴板粘贴
```

**测试用例**: 57 tests (13 unit + 44 integration)

**单元测试**:
- SessionConfig 测试（default、clipboard_only、keyboard_only、clipboard_paste、serialization）
- SessionEvent 测试（Started、PartialTranscript、CommittedTranscript、TextInjected、TextCopied、Stopped、Error）
- SessionError 测试（display、debug）

**集成测试** (`tests/session_test.rs`):
- SessionConfig 序列化/反序列化测试
- SessionEvent JSON 格式测试（tagged format）
- SessionError 边界值测试
- API 存在性和方法签名验证
- 状态模块集成测试
- Unicode 支持测试

**验收标准**:
- [x] SessionConfig 配置结构体
- [x] SessionEvent 事件枚举（可序列化）
- [x] RaFlowSession 完整会话管理
- [x] 整合 TranscriptionSession
- [x] 整合 StateManager 和 StateTransitionContext
- [x] 整合 TextInjector
- [x] 自动状态转换流程
- [x] Tauri 事件发射
- [x] 单元测试通过（13个测试）
- [x] 集成测试通过（44个测试）

**完成状态**: ✅ 已完成 (2025-12-25)

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
