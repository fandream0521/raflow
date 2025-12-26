//! 音频缓冲区模块
//!
//! 提供高性能的无锁环形缓冲区实现，用于音频数据传输
//!
//! # 特性
//!
//! - 无锁的生产者-消费者模式
//! - 零拷贝的批量读写操作
//! - 预分配内存，避免运行时分配
//! - 适用于实时音频处理场景
//!
//! # 使用示例
//!
//! ```
//! use raflow_lib::audio::buffer::AudioRingBuffer;
//!
//! // 创建容量为 4096 样本的环形缓冲区
//! let (producer, consumer) = AudioRingBuffer::new(4096);
//!
//! // 生产者写入数据
//! let audio_data = vec![0.5f32; 480];
//! producer.push_slice(&audio_data);
//!
//! // 消费者读取数据
//! let mut output = vec![0.0f32; 480];
//! consumer.pop_slice(&mut output);
//! ```

use ringbuf::{
    traits::{Consumer, Observer, Producer, Split},
    HeapRb,
};

/// 音频环形缓冲区
///
/// 封装 ringbuf 库的无锁环形缓冲区，提供音频专用的 API
pub struct AudioRingBuffer;

/// 音频缓冲区生产者
///
/// 用于向环形缓冲区写入音频数据
pub struct AudioBufferProducer {
    producer: ringbuf::HeapProd<f32>,
}

/// 音频缓冲区消费者
///
/// 用于从环形缓冲区读取音频数据
pub struct AudioBufferConsumer {
    consumer: ringbuf::HeapCons<f32>,
}

impl AudioRingBuffer {
    /// 创建新的音频环形缓冲区
    ///
    /// # Arguments
    ///
    /// * `capacity` - 缓冲区容量（样本数）
    ///
    /// # Returns
    ///
    /// 返回 (生产者, 消费者) 元组
    ///
    /// # Example
    ///
    /// ```
    /// use raflow_lib::audio::buffer::AudioRingBuffer;
    ///
    /// let (producer, consumer) = AudioRingBuffer::new(4096);
    /// ```
    pub fn new(capacity: usize) -> (AudioBufferProducer, AudioBufferConsumer) {
        let rb = HeapRb::<f32>::new(capacity);
        let (producer, consumer) = rb.split();

        (
            AudioBufferProducer { producer },
            AudioBufferConsumer { consumer },
        )
    }

    /// 创建具有默认容量的缓冲区
    ///
    /// 默认容量为 48000（1秒 @ 48kHz）
    pub fn with_default_capacity() -> (AudioBufferProducer, AudioBufferConsumer) {
        Self::new(48000)
    }

    /// 创建用于实时处理的缓冲区
    ///
    /// 容量为 9600（200ms @ 48kHz），适合低延迟场景
    pub fn for_realtime() -> (AudioBufferProducer, AudioBufferConsumer) {
        Self::new(9600) // 200ms @ 48kHz
    }
}

impl AudioBufferProducer {
    /// 写入单个样本
    ///
    /// # Returns
    ///
    /// 成功返回 `true`，缓冲区已满返回 `false`
    pub fn push(&mut self, sample: f32) -> bool {
        self.producer.try_push(sample).is_ok()
    }

    /// 批量写入样本
    ///
    /// # Arguments
    ///
    /// * `samples` - 要写入的样本切片
    ///
    /// # Returns
    ///
    /// 返回实际写入的样本数
    pub fn push_slice(&mut self, samples: &[f32]) -> usize {
        self.producer.push_slice(samples)
    }

    /// 尝试批量写入所有样本
    ///
    /// # Returns
    ///
    /// 如果所有样本都写入成功返回 `true`，否则返回 `false`（不会写入任何数据）
    pub fn try_push_all(&mut self, samples: &[f32]) -> bool {
        if self.available_space() >= samples.len() {
            self.push_slice(samples);
            true
        } else {
            false
        }
    }

    /// 获取可用写入空间
    pub fn available_space(&self) -> usize {
        self.producer.vacant_len()
    }

    /// 检查缓冲区是否已满
    pub fn is_full(&self) -> bool {
        self.producer.is_full()
    }

    /// 获取缓冲区容量
    pub fn capacity(&self) -> usize {
        self.producer.capacity().get()
    }
}

impl AudioBufferConsumer {
    /// 读取单个样本
    ///
    /// # Returns
    ///
    /// 成功返回样本值，缓冲区为空返回 `None`
    pub fn pop(&mut self) -> Option<f32> {
        self.consumer.try_pop()
    }

    /// 批量读取样本到切片
    ///
    /// # Arguments
    ///
    /// * `output` - 目标切片
    ///
    /// # Returns
    ///
    /// 返回实际读取的样本数
    pub fn pop_slice(&mut self, output: &mut [f32]) -> usize {
        self.consumer.pop_slice(output)
    }

    /// 读取所有可用样本到 Vec
    ///
    /// # Returns
    ///
    /// 返回包含所有可用样本的 Vec
    pub fn pop_all(&mut self) -> Vec<f32> {
        let len = self.available_samples();
        let mut output = vec![0.0f32; len];
        self.pop_slice(&mut output);
        output
    }

    /// 读取指定数量的样本，如果不够则返回 None
    pub fn pop_exact(&mut self, count: usize) -> Option<Vec<f32>> {
        if self.available_samples() >= count {
            let mut output = vec![0.0f32; count];
            self.pop_slice(&mut output);
            Some(output)
        } else {
            None
        }
    }

    /// 获取可读取的样本数
    pub fn available_samples(&self) -> usize {
        self.consumer.occupied_len()
    }

    /// 检查缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.consumer.is_empty()
    }

    /// 获取缓冲区容量
    pub fn capacity(&self) -> usize {
        self.consumer.capacity().get()
    }

    /// 跳过指定数量的样本
    pub fn skip(&mut self, count: usize) -> usize {
        self.consumer.skip(count)
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        let _ = self.consumer.clear();
    }
}

// 实现 Send 和 Sync（ringbuf 的类型本身是安全的）
unsafe impl Send for AudioBufferProducer {}
unsafe impl Send for AudioBufferConsumer {}

/// 预分配的可重用缓冲区池
///
/// 用于避免频繁内存分配
pub struct BufferPool {
    buffers: Vec<Vec<f32>>,
    buffer_size: usize,
}

impl BufferPool {
    /// 创建新的缓冲区池
    ///
    /// # Arguments
    ///
    /// * `pool_size` - 池中缓冲区数量
    /// * `buffer_size` - 每个缓冲区的大小
    pub fn new(pool_size: usize, buffer_size: usize) -> Self {
        let buffers = (0..pool_size)
            .map(|_| vec![0.0f32; buffer_size])
            .collect();

        Self {
            buffers,
            buffer_size,
        }
    }

    /// 获取一个可用缓冲区
    ///
    /// 如果池中有可用缓冲区则复用，否则创建新缓冲区
    pub fn get(&mut self) -> Vec<f32> {
        self.buffers.pop().unwrap_or_else(|| vec![0.0f32; self.buffer_size])
    }

    /// 归还缓冲区到池中
    ///
    /// 缓冲区内容会被清零
    pub fn put(&mut self, mut buffer: Vec<f32>) {
        // 清零缓冲区
        buffer.iter_mut().for_each(|s| *s = 0.0);
        // 确保大小正确
        buffer.resize(self.buffer_size, 0.0);
        self.buffers.push(buffer);
    }

    /// 获取池中可用缓冲区数量
    pub fn available(&self) -> usize {
        self.buffers.len()
    }

    /// 获取缓冲区大小
    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
}

/// 可重用的 PCM 缓冲区
///
/// 专门用于 PCM 数据处理的缓冲区，避免重复分配
pub struct PcmBuffer {
    /// f32 样本缓冲区
    pub samples: Vec<f32>,
    /// i16 PCM 缓冲区
    pub pcm: Vec<i16>,
    /// 字节缓冲区
    pub bytes: Vec<u8>,
    /// Base64 字符串缓冲区
    pub base64: String,
}

impl PcmBuffer {
    /// 创建新的 PCM 缓冲区
    ///
    /// # Arguments
    ///
    /// * `sample_capacity` - 样本容量（f32 样本数）
    pub fn new(sample_capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(sample_capacity),
            pcm: Vec::with_capacity(sample_capacity),
            bytes: Vec::with_capacity(sample_capacity * 2),
            base64: String::with_capacity(sample_capacity * 3), // Base64 大约是原始大小的 4/3
        }
    }

    /// 创建用于 100ms @ 16kHz 的缓冲区
    pub fn for_100ms() -> Self {
        Self::new(1600) // 100ms @ 16kHz
    }

    /// 清空所有缓冲区
    pub fn clear(&mut self) {
        self.samples.clear();
        self.pcm.clear();
        self.bytes.clear();
        self.base64.clear();
    }

    /// 将 f32 样本转换为 i16 PCM
    pub fn convert_to_pcm(&mut self) {
        self.pcm.clear();
        self.pcm.extend(self.samples.iter().map(|&s| {
            let clamped = s.clamp(-1.0, 1.0);
            (clamped * 32767.0) as i16
        }));
    }

    /// 将 i16 PCM 转换为字节
    pub fn convert_to_bytes(&mut self) {
        self.bytes.clear();
        self.bytes.reserve(self.pcm.len() * 2);
        for &sample in &self.pcm {
            self.bytes.extend_from_slice(&sample.to_le_bytes());
        }
    }

    /// 将字节编码为 Base64
    pub fn encode_base64(&mut self) {
        use base64::{engine::general_purpose::STANDARD, Engine};
        self.base64.clear();
        STANDARD.encode_string(&self.bytes, &mut self.base64);
    }

    /// 完整的处理流程：f32 -> i16 -> bytes -> base64
    pub fn process(&mut self) -> &str {
        self.convert_to_pcm();
        self.convert_to_bytes();
        self.encode_base64();
        &self.base64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_creation() {
        let (producer, consumer) = AudioRingBuffer::new(1024);
        assert_eq!(producer.capacity(), 1024);
        assert_eq!(consumer.capacity(), 1024);
    }

    #[test]
    fn test_ring_buffer_push_pop() {
        let (mut producer, mut consumer) = AudioRingBuffer::new(100);

        // 写入数据
        assert!(producer.push(0.5));
        assert!(producer.push(0.75));

        // 读取数据
        assert_eq!(consumer.pop(), Some(0.5));
        assert_eq!(consumer.pop(), Some(0.75));
        assert_eq!(consumer.pop(), None);
    }

    #[test]
    fn test_ring_buffer_slice_operations() {
        let (mut producer, mut consumer) = AudioRingBuffer::new(1024);

        // 批量写入
        let input: Vec<f32> = (0..100).map(|i| i as f32 * 0.01).collect();
        let written = producer.push_slice(&input);
        assert_eq!(written, 100);

        // 批量读取
        let mut output = vec![0.0f32; 50];
        let read = consumer.pop_slice(&mut output);
        assert_eq!(read, 50);

        // 验证数据
        for i in 0..50 {
            assert!((output[i] - i as f32 * 0.01).abs() < 0.0001);
        }
    }

    #[test]
    fn test_ring_buffer_overflow() {
        let (mut producer, _consumer) = AudioRingBuffer::new(10);

        // 填满缓冲区
        let data = vec![1.0f32; 10];
        producer.push_slice(&data);

        // 缓冲区应该已满
        assert!(producer.is_full());
        assert!(!producer.push(2.0));
    }

    #[test]
    fn test_ring_buffer_available() {
        let (mut producer, mut consumer) = AudioRingBuffer::new(100);

        // 初始状态
        assert_eq!(producer.available_space(), 100);
        assert_eq!(consumer.available_samples(), 0);

        // 写入 50 个样本
        let data = vec![0.5f32; 50];
        producer.push_slice(&data);

        assert_eq!(producer.available_space(), 50);
        assert_eq!(consumer.available_samples(), 50);
    }

    #[test]
    fn test_buffer_pool() {
        let mut pool = BufferPool::new(3, 480);

        assert_eq!(pool.available(), 3);
        assert_eq!(pool.buffer_size(), 480);

        // 获取缓冲区
        let buf1 = pool.get();
        assert_eq!(buf1.len(), 480);
        assert_eq!(pool.available(), 2);

        let buf2 = pool.get();
        assert_eq!(pool.available(), 1);

        // 归还缓冲区
        pool.put(buf1);
        assert_eq!(pool.available(), 2);

        pool.put(buf2);
        assert_eq!(pool.available(), 3);
    }

    #[test]
    fn test_pcm_buffer() {
        let mut buffer = PcmBuffer::new(100);

        // 添加样本
        buffer.samples = vec![-1.0, -0.5, 0.0, 0.5, 1.0];

        // 处理
        let base64 = buffer.process();
        assert!(!base64.is_empty());

        // 验证 PCM 转换
        assert_eq!(buffer.pcm.len(), 5);
        assert_eq!(buffer.pcm[0], -32767);
        assert_eq!(buffer.pcm[2], 0);
        assert_eq!(buffer.pcm[4], 32767);

        // 验证字节转换
        assert_eq!(buffer.bytes.len(), 10); // 5 samples * 2 bytes
    }

    #[test]
    fn test_pcm_buffer_clear() {
        let mut buffer = PcmBuffer::new(100);
        buffer.samples = vec![0.5; 50];
        buffer.process();

        assert!(!buffer.samples.is_empty());
        assert!(!buffer.pcm.is_empty());
        assert!(!buffer.bytes.is_empty());
        assert!(!buffer.base64.is_empty());

        buffer.clear();

        assert!(buffer.samples.is_empty());
        assert!(buffer.pcm.is_empty());
        assert!(buffer.bytes.is_empty());
        assert!(buffer.base64.is_empty());
    }

    #[test]
    fn test_default_capacity() {
        let (producer, _) = AudioRingBuffer::with_default_capacity();
        assert_eq!(producer.capacity(), 48000);
    }

    #[test]
    fn test_realtime_capacity() {
        let (producer, _) = AudioRingBuffer::for_realtime();
        assert_eq!(producer.capacity(), 9600);
    }

    #[test]
    fn test_pop_exact() {
        let (mut producer, mut consumer) = AudioRingBuffer::new(100);

        let data = vec![0.5f32; 30];
        producer.push_slice(&data);

        // 请求的数量小于可用数量
        let result = consumer.pop_exact(20);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 20);

        // 请求的数量等于剩余数量
        let result = consumer.pop_exact(10);
        assert!(result.is_some());

        // 请求的数量大于可用数量
        let result = consumer.pop_exact(10);
        assert!(result.is_none());
    }

    #[test]
    fn test_consumer_clear() {
        let (mut producer, mut consumer) = AudioRingBuffer::new(100);

        let data = vec![0.5f32; 50];
        producer.push_slice(&data);

        assert_eq!(consumer.available_samples(), 50);

        consumer.clear();

        assert_eq!(consumer.available_samples(), 0);
        assert!(consumer.is_empty());
    }
}
