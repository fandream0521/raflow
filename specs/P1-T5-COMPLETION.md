# P1-T5: 消息类型定义 - 完成总结

## 概述

P1-T5（消息类型定义）已完成，实现了与 ElevenLabs Scribe v2 Realtime API 通信所需的所有 WebSocket 消息类型，包括客户端发送消息和服务端响应消息的完整定义。

## 实现的功能

### 1. 客户端消息类型 (`src-tauri/src/network/messages.rs`)

#### ✅ InputAudioChunk - 音频块消息

```rust
pub struct InputAudioChunk {
    pub message_type: &'static str,      // "input_audio_chunk"
    pub audio_base_64: String,            // Base64 编码的 PCM 数据
    pub commit: Option<bool>,             // 是否手动提交
    pub sample_rate: Option<u32>,         // 采样率（首次发送）
    pub previous_text: Option<String>,    // 上下文文本
}
```

**特性**:
- ✅ Builder 模式构造器
- ✅ 可选字段自动省略序列化
- ✅ 支持首次发送时指定采样率
- ✅ 支持上下文文本提高准确度

**构造方法**:
```rust
let chunk = InputAudioChunk::new(audio_base64)
    .with_sample_rate(16000)
    .with_commit()
    .with_previous_text("context".to_string());
```

#### ✅ CommitMessage - 手动提交消息

```rust
pub struct CommitMessage {
    pub message_type: &'static str,  // "commit"
}
```

**特性**:
- ✅ 简单的提交指令
- ✅ 触发服务器提交当前转写段

#### ✅ CloseMessage - 关闭连接消息

```rust
pub struct CloseMessage {
    pub message_type: &'static str,  // "close"
}
```

**特性**:
- ✅ 优雅关闭 WebSocket 连接
- ✅ 清理服务器资源

#### ✅ ClientMessage - 客户端消息联合类型

```rust
#[serde(untagged)]
pub enum ClientMessage {
    InputAudioChunk(InputAudioChunk),
    Commit(CommitMessage),
    Close(CloseMessage),
}
```

**特性**:
- ✅ 统一的序列化接口
- ✅ 无标签枚举（untagged）自动选择格式

### 2. 服务端消息类型

#### ✅ ServerMessage - 服务端消息枚举

```rust
#[serde(tag = "message_type")]
pub enum ServerMessage {
    SessionStarted { session_id, config },
    PartialTranscript { text },
    CommittedTranscript { text },
    CommittedTranscriptWithTimestamps { text, language_code, words },
    InputError { error_message },
}
```

**特性**:
- ✅ 基于 `message_type` 字段的标签枚举
- ✅ 自动反序列化到正确的变体
- ✅ 类型安全的模式匹配

#### ✅ SessionStarted - 会话开始

```rust
SessionStarted {
    session_id: String,           // 会话 ID
    config: Option<SessionConfig>, // 会话配置
}
```

**包含信息**:
- 唯一会话标识符
- 服务器配置（采样率、格式、语言、模型）

#### ✅ PartialTranscript - 部分转写

```rust
PartialTranscript {
    text: String,  // 实时部分转写文本
}
```

**用途**:
- 实时显示转写进度
- 更新悬浮窗 UI

#### ✅ CommittedTranscript - 最终转写

```rust
CommittedTranscript {
    text: String,  // 最终确认的转写文本
}
```

**用途**:
- 获取最终转写结果
- 触发文本注入

#### ✅ CommittedTranscriptWithTimestamps - 带时间戳转写

```rust
CommittedTranscriptWithTimestamps {
    text: String,              // 转写文本
    language_code: String,     // 检测到的语言
    words: Vec<WordTimestamp>, // 单词级时间戳
}
```

**用途**:
- 获取详细的单词级别时间信息
- 分析语音节奏和停顿

#### ✅ InputError - 输入错误

```rust
InputError {
    error_message: String,  // 错误描述
}
```

**用途**:
- 处理服务器端错误
- 显示错误信息给用户

### 3. 辅助类型

#### ✅ SessionConfig - 会话配置

```rust
pub struct SessionConfig {
    pub sample_rate: u32,                  // 采样率
    pub audio_format: String,              // 音频格式
    pub language_code: Option<String>,     // 语言代码
    pub model_id: String,                  // 模型 ID
    pub vad_commit_strategy: Option<VadConfig>, // VAD 配置
}
```

#### ✅ WordTimestamp - 单词时间戳

```rust
pub struct WordTimestamp {
    pub word: String,          // 单词文本
    pub start: f64,            // 开始时间（秒）
    pub end: f64,              // 结束时间（秒）
    pub word_type: String,     // 类型（word/punctuation）
    pub logprob: Option<f64>,  // 置信度分数
}
```

**方法**:
- `duration()` - 获取单词持续时间
- `is_punctuation()` - 判断是否为标点符号

#### ✅ VadConfig - VAD 配置

```rust
pub struct VadConfig {
    pub strategy: String,                     // VAD 策略
    pub silence_duration_ms: Option<u32>,     // 静音阈值
    pub min_speech_duration_ms: Option<u32>,  // 最小语音时长
}
```

### 4. 辅助方法

#### ServerMessage 帮助方法

```rust
impl ServerMessage {
    pub fn is_partial(&self) -> bool          // 是否为部分转写
    pub fn is_committed(&self) -> bool        // 是否为最终转写
    pub fn is_error(&self) -> bool            // 是否为错误
    pub fn text(&self) -> Option<&str>        // 获取文本
    pub fn error_message(&self) -> Option<&str>  // 获取错误信息
    pub fn session_id(&self) -> Option<&str>  // 获取会话 ID
}
```

**用途**:
- 简化消息类型判断
- 统一的文本提取接口
- 避免重复的模式匹配

## 测试覆盖

### ✅ 单元测试（14个）

1. **`test_input_audio_chunk_basic`**: 基本创建测试
2. **`test_input_audio_chunk_with_options`**: Builder 模式测试
3. **`test_input_audio_chunk_serialization`**: JSON 序列化测试
4. **`test_commit_message`**: 提交消息测试
5. **`test_close_message`**: 关闭消息测试
6. **`test_server_message_session_started`**: 会话开始反序列化
7. **`test_server_message_partial_transcript`**: 部分转写反序列化
8. **`test_server_message_committed_transcript`**: 最终转写反序列化
9. **`test_server_message_committed_with_timestamps`**: 带时间戳反序列化
10. **`test_server_message_input_error`**: 错误消息反序列化
11. **`test_word_timestamp_duration`**: 时间戳计算测试
12. **`test_word_timestamp_punctuation`**: 标点符号判断测试
13. **`test_session_config_deserialization`**: 配置反序列化测试
14. **`test_client_message_serialization`**: 客户端消息序列化测试

```bash
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
```

### ✅ 集成测试（14个）

1. **`test_input_audio_chunk_full_cycle`**: 完整周期测试
2. **`test_input_audio_chunk_minimal`**: 最小字段测试
3. **`test_commit_message`**: 提交消息集成测试
4. **`test_close_message`**: 关闭消息集成测试
5. **`test_client_message_variants`**: 客户端消息变体测试
6. **`test_session_started_deserialization`**: 会话开始集成测试
7. **`test_partial_transcript_deserialization`**: 部分转写集成测试
8. **`test_committed_transcript_deserialization`**: 最终转写集成测试
9. **`test_committed_with_timestamps_deserialization`**: 时间戳集成测试
10. **`test_input_error_deserialization`**: 错误处理集成测试
11. **`test_server_message_helper_methods`**: 辅助方法测试
12. **`test_real_world_session_flow`**: 真实会话流程测试
13. **`test_round_trip_serialization`**: 往返序列化测试
14. **`test_message_size_estimation`**: 消息大小估算测试

```bash
running 14 tests
test test_close_message ... ok
test test_commit_message ... ok
test test_client_message_variants ... ok
test test_committed_transcript_deserialization ... ok
test test_input_audio_chunk_full_cycle ... ok
test test_input_audio_chunk_minimal ... ok
test test_committed_with_timestamps_deserialization ... ok
test test_input_error_deserialization ... ok
test test_message_size_estimation ... ok
test test_partial_transcript_deserialization ... ok
test test_real_world_session_flow ... ok
test test_round_trip_serialization ... ok
test test_server_message_helper_methods ... ok
test test_session_started_deserialization ... ok

test result: ok. 14 passed; 0 failed; 0 ignored
```

## 测试数据分析

### JSON 序列化格式

#### 客户端消息示例

```json
// InputAudioChunk with all fields
{
  "message_type": "input_audio_chunk",
  "audio_base_64": "SGVsbG8gV29ybGQ=",
  "commit": true,
  "sample_rate": 16000,
  "previous_text": "Previous context"
}

// CommitMessage
{
  "message_type": "commit"
}

// CloseMessage
{
  "message_type": "close"
}
```

#### 服务端消息示例

```json
// SessionStarted
{
  "message_type": "session_started",
  "session_id": "sess_abc123",
  "config": {
    "sample_rate": 16000,
    "audio_format": "pcm_s16le",
    "language_code": "zh",
    "model_id": "scribe_v2_realtime"
  }
}

// PartialTranscript
{
  "message_type": "partial_transcript",
  "text": "你好世界"
}

// CommittedTranscript
{
  "message_type": "committed_transcript",
  "text": "这是最终的转写结果"
}

// CommittedTranscriptWithTimestamps
{
  "message_type": "committed_transcript_with_timestamps",
  "text": "Hello world.",
  "language_code": "en",
  "words": [
    {
      "word": "Hello",
      "start": 0.0,
      "end": 0.5,
      "type": "word",
      "logprob": -1.234
    },
    {
      "word": ".",
      "start": 1.0,
      "end": 1.05,
      "type": "punctuation"
    }
  ]
}

// InputError
{
  "message_type": "input_error",
  "error_message": "Invalid audio format: expected PCM 16kHz"
}
```

### 消息大小估算

```
PCM 数据: 3200 bytes (100ms @ 16kHz, i16)
  ↓
Base64 编码: ~4267 characters (4/3 of原始大小)
  ↓
JSON 封装: ~4350 bytes
  ↓
开销: ~83 bytes (message_type, 括号等)
```

## 项目结构更新

```
src-tauri/src/
├── audio/
│   └── ...
├── network/              # ✨ 新增：网络通信模块
│   ├── mod.rs            # 模块导出
│   └── messages.rs       # ✨ 消息类型定义
└── lib.rs                # 更新：添加 network 模块

tests/
├── audio_*.rs
└── network_messages_test.rs  # ✨ 新增：消息类型集成测试
```

## 使用示例

### 客户端发送音频

```rust
use raflow_lib::network::messages::*;

// 创建音频块消息
let chunk = InputAudioChunk::new(audio_base64)
    .with_sample_rate(16000);  // 首次发送时指定

// 序列化为 JSON
let json = serde_json::to_string(&chunk)?;

// 通过 WebSocket 发送
websocket.send(json).await?;
```

### 处理服务器响应

```rust
use raflow_lib::network::messages::ServerMessage;

// 接收 WebSocket 消息
let json = websocket.recv().await?;

// 反序列化
let msg: ServerMessage = serde_json::from_str(&json)?;

// 处理不同类型的消息
match msg {
    ServerMessage::SessionStarted { session_id, .. } => {
        println!("Session started: {}", session_id);
    }
    ServerMessage::PartialTranscript { text } => {
        // 更新 UI 显示实时转写
        update_overlay(&text);
    }
    ServerMessage::CommittedTranscript { text } => {
        // 获取最终结果，触发文本注入
        inject_text(&text)?;
    }
    ServerMessage::InputError { error_message } => {
        // 显示错误
        show_error(&error_message);
    }
    _ => {}
}

// 或使用辅助方法
if msg.is_partial() {
    println!("Partial: {}", msg.text().unwrap());
} else if msg.is_committed() {
    println!("Final: {}", msg.text().unwrap());
}
```

### 完整会话流程

```rust
use raflow_lib::network::messages::*;

// 1. 连接建立后，接收 SessionStarted
let session: ServerMessage = recv_message().await?;
if let Some(session_id) = session.session_id() {
    println!("Connected: {}", session_id);
}

// 2. 发送音频数据
for audio_chunk in audio_stream {
    let msg = InputAudioChunk::new(audio_chunk);
    send_message(&msg).await?;
}

// 3. 接收实时转写更新
while let Some(msg) = recv_message().await? {
    match msg {
        ServerMessage::PartialTranscript { text } => {
            println!("Transcribing: {}", text);
        }
        ServerMessage::CommittedTranscript { text } => {
            println!("Final: {}", text);
            break;
        }
        _ => {}
    }
}

// 4. 关闭连接
send_message(&CloseMessage::new()).await?;
```

## 后续任务准备

P1-T5 为后续 WebSocket 通信提供了完整的消息类型支持：

- ✅ **P1-T6**: WebSocket 连接（使用这些消息类型）
- ✅ **P1-T7**: 发送/接收任务（序列化和反序列化）
- ✅ **P1-T8**: 端到端集成（完整的通信流程）

## 验收标准达成

| 标准 | 状态 |
|------|------|
| 定义所有客户端消息类型 | ✅ (3 种) |
| 定义所有服务端消息类型 | ✅ (5 种) |
| 实现序列化/反序列化 | ✅ (serde) |
| 可选字段正确处理 | ✅ (skip_serializing_if) |
| 标签枚举自动分发 | ✅ (tag = "message_type") |
| Builder 模式构造器 | ✅ (with_* 方法) |
| 辅助方法 | ✅ (is_*, text(), 等) |
| 所有单元测试通过 | ✅ (14/14) |
| 集成测试通过 | ✅ (14/14) |
| 支持真实 API 格式 | ✅ (基于官方文档) |
| 类型安全 | ✅ (强类型枚举) |

## 技术亮点

### 1. 类型安全的消息处理

- 使用 Rust 枚举确保类型安全
- 编译时检查消息类型
- 模式匹配避免运行时错误

### 2. 自动序列化/反序列化

- serde 的 `tag` 属性自动分发
- 可选字段自动省略
- 无需手动编写序列化代码

### 3. 人性化 API

- Builder 模式构造器
- 辅助方法简化常见操作
- 清晰的文档和示例

### 4. 完整的错误处理

- 所有可能的服务器响应类型
- 错误消息类型化处理
- 优雅的错误传播

## 总结

P1-T5 成功实现了 WebSocket 消息类型定义：

- ✅ 完整的客户端消息类型（3 种）
- ✅ 完整的服务端消息类型（5 种）
- ✅ 辅助类型和配置（3 种）
- ✅ 自动序列化/反序列化
- ✅ 类型安全的消息处理
- ✅ Builder 模式 API
- ✅ 完善的测试覆盖（28 个测试）
- ✅ 符合 ElevenLabs API 规范

为 WebSocket 通信层（P1-T6, P1-T7）提供了坚实的类型基础。

---

**完成日期**: 2025-12-23
**测试状态**: 28/28 测试通过 (14 单元 + 14 集成)
**总测试数**: 95 个测试全部通过
**代码行数**: ~600 行（含测试 ~1100 行）
