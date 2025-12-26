//! 应用配置模块
//!
//! 提供应用程序配置的加载、保存和管理功能
//!
//! # 配置存储位置
//!
//! - Windows: `%APPDATA%/RaFlow/config.json`
//! - macOS: `~/Library/Application Support/com.raflow.app/config.json`
//! - Linux: `~/.config/raflow/config.json`
//!
//! # 使用示例
//!
//! ```ignore
//! use raflow_lib::state::config::{AppConfig, ConfigManager};
//!
//! // 加载配置
//! let config = ConfigManager::load(&app_handle)?;
//!
//! // 修改配置
//! let mut config = config;
//! config.behavior.show_overlay = false;
//!
//! // 保存配置
//! ConfigManager::save(&app_handle, &config)?;
//! ```

use std::path::PathBuf;
use std::sync::Arc;

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};
use thiserror::Error;

use crate::hotkey::HotkeyConfig;
use crate::input::InjectionStrategy;

/// 配置错误类型
#[derive(Error, Debug)]
pub enum ConfigError {
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化/反序列化错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// 路径错误
    #[error("Path error: {0}")]
    Path(String),

    /// Tauri 错误
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
}

/// 配置结果类型
pub type ConfigResult<T> = Result<T, ConfigError>;

/// 应用配置
///
/// 包含所有应用程序设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// API 配置
    pub api: ApiConfig,
    /// 音频配置
    pub audio: AudioConfig,
    /// 热键配置
    pub hotkeys: HotkeyConfig,
    /// 行为配置
    pub behavior: BehaviorConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            audio: AudioConfig::default(),
            hotkeys: HotkeyConfig::default(),
            behavior: BehaviorConfig::default(),
        }
    }
}

/// API 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
    /// ElevenLabs API 密钥（加密存储）
    #[serde(default)]
    pub api_key: String,
    /// 模型 ID
    pub model_id: String,
    /// 语言代码（可选，自动检测）
    pub language_code: Option<String>,
    /// 是否启用时间戳
    pub include_timestamps: bool,
    /// VAD 提交策略
    pub vad_commit_strategy: Option<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model_id: "scribe_v2_realtime".to_string(),
            language_code: Some("zh".to_string()),
            include_timestamps: false,
            vad_commit_strategy: None,
        }
    }
}

/// 音频配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioConfig {
    /// 输入设备 ID（None 表示默认设备）
    pub input_device_id: Option<String>,
    /// 输入设备名称（仅供显示）
    pub input_device_name: Option<String>,
    /// 音量增益（0.5 - 2.0）
    pub gain: f32,
    /// 是否启用噪声抑制（预留）
    pub noise_suppression: bool,
    /// 静音阈值（0.0 - 1.0）
    pub silence_threshold: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            input_device_id: None,
            input_device_name: None,
            gain: 1.0,
            noise_suppression: false,
            silence_threshold: 0.01,
        }
    }
}

/// 行为配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    /// 文本注入策略
    pub injection_strategy: InjectionStrategy,
    /// 自动策略阈值（字符数）
    pub auto_threshold: usize,
    /// 粘贴延迟（毫秒）
    pub paste_delay_ms: u64,
    /// 注入前延迟（毫秒）
    pub pre_injection_delay_ms: u64,
    /// 是否自动注入
    pub auto_inject: bool,
    /// 是否显示悬浮窗
    pub show_overlay: bool,
    /// 是否开机自启动
    pub auto_start: bool,
    /// 是否最小化到托盘
    pub minimize_to_tray: bool,
    /// 处理超时时间（秒）
    pub processing_timeout_secs: u64,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            injection_strategy: InjectionStrategy::Auto,
            auto_threshold: 20,
            paste_delay_ms: 100,
            pre_injection_delay_ms: 50,
            auto_inject: true,
            show_overlay: true,
            auto_start: false,
            minimize_to_tray: true,
            processing_timeout_secs: 30,
        }
    }
}

/// 配置管理器
///
/// 提供配置的加载、保存和管理功能
pub struct ConfigManager;

impl ConfigManager {
    /// 加载配置
    ///
    /// 从配置文件加载配置，如果文件不存在则返回默认配置
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri 应用句柄
    ///
    /// # Returns
    ///
    /// 返回加载的配置
    pub fn load<R: Runtime>(app: &AppHandle<R>) -> ConfigResult<AppConfig> {
        let path = Self::config_path(app)?;

        tracing::debug!(path = %path.display(), "Loading config");

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: AppConfig = serde_json::from_str(&content)?;
            tracing::info!(path = %path.display(), "Config loaded successfully");
            Ok(config)
        } else {
            tracing::info!("Config file not found, using defaults");
            Ok(AppConfig::default())
        }
    }

    /// 保存配置
    ///
    /// 将配置保存到配置文件
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri 应用句柄
    /// * `config` - 要保存的配置
    pub fn save<R: Runtime>(app: &AppHandle<R>, config: &AppConfig) -> ConfigResult<()> {
        let path = Self::config_path(app)?;

        tracing::debug!(path = %path.display(), "Saving config");

        // 确保目录存在
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let content = serde_json::to_string_pretty(config)?;
        std::fs::write(&path, content)?;

        tracing::info!(path = %path.display(), "Config saved successfully");
        Ok(())
    }

    /// 获取配置文件路径
    ///
    /// # Arguments
    ///
    /// * `app` - Tauri 应用句柄
    ///
    /// # Returns
    ///
    /// 返回配置文件路径
    pub fn config_path<R: Runtime>(app: &AppHandle<R>) -> ConfigResult<PathBuf> {
        let app_config_dir = app
            .path()
            .app_config_dir()
            .map_err(|e| ConfigError::Path(e.to_string()))?;

        Ok(app_config_dir.join("config.json"))
    }

    /// 获取配置目录路径
    pub fn config_dir<R: Runtime>(app: &AppHandle<R>) -> ConfigResult<PathBuf> {
        app.path()
            .app_config_dir()
            .map_err(|e| ConfigError::Path(e.to_string()))
    }

    /// 检查配置文件是否存在
    pub fn exists<R: Runtime>(app: &AppHandle<R>) -> ConfigResult<bool> {
        let path = Self::config_path(app)?;
        Ok(path.exists())
    }

    /// 删除配置文件
    pub fn delete<R: Runtime>(app: &AppHandle<R>) -> ConfigResult<()> {
        let path = Self::config_path(app)?;
        if path.exists() {
            std::fs::remove_file(&path)?;
            tracing::info!(path = %path.display(), "Config deleted");
        }
        Ok(())
    }

    /// 重置为默认配置
    pub fn reset<R: Runtime>(app: &AppHandle<R>) -> ConfigResult<AppConfig> {
        let config = AppConfig::default();
        Self::save(app, &config)?;
        tracing::info!("Config reset to defaults");
        Ok(config)
    }
}

/// 全局配置状态
///
/// 使用 ArcSwap 实现无锁读取
pub struct GlobalConfig {
    config: ArcSwap<AppConfig>,
}

impl GlobalConfig {
    /// 创建新的全局配置
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: ArcSwap::new(Arc::new(config)),
        }
    }

    /// 获取当前配置
    pub fn get(&self) -> Arc<AppConfig> {
        self.config.load_full()
    }

    /// 更新配置
    pub fn update(&self, config: AppConfig) {
        self.config.store(Arc::new(config));
    }

    /// 更新 API 密钥
    pub fn set_api_key(&self, api_key: String) {
        let mut config = (*self.config.load_full()).clone();
        config.api.api_key = api_key;
        self.config.store(Arc::new(config));
    }

    /// 获取 API 密钥
    pub fn api_key(&self) -> String {
        self.config.load_full().api.api_key.clone()
    }

    /// 检查 API 密钥是否已配置
    pub fn has_api_key(&self) -> bool {
        !self.config.load_full().api.api_key.is_empty()
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self::new(AppConfig::default())
    }
}

/// 初始化配置系统
///
/// 加载配置并注册到应用状态
///
/// # Arguments
///
/// * `app` - Tauri 应用句柄
///
/// # Returns
///
/// 返回全局配置实例
pub fn init_config<R: Runtime>(app: &AppHandle<R>) -> ConfigResult<Arc<GlobalConfig>> {
    tracing::info!("Initializing config system");

    // 加载配置
    let config = ConfigManager::load(app)?;

    // 创建全局配置
    let global_config = Arc::new(GlobalConfig::new(config));

    // 注册到应用状态
    app.manage(Arc::clone(&global_config));

    tracing::info!("Config system initialized");
    Ok(global_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();

        assert!(config.api.api_key.is_empty());
        assert_eq!(config.api.model_id, "scribe_v2_realtime");
        assert_eq!(config.api.language_code, Some("zh".to_string()));

        assert!(config.audio.input_device_id.is_none());
        assert_eq!(config.audio.gain, 1.0);

        assert_eq!(config.behavior.injection_strategy, InjectionStrategy::Auto);
        assert!(config.behavior.show_overlay);
        assert!(config.behavior.auto_inject);
    }

    #[test]
    fn test_app_config_serialization() {
        let config = AppConfig::default();

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.api.model_id, deserialized.api.model_id);
        assert_eq!(config.audio.gain, deserialized.audio.gain);
        assert_eq!(
            config.behavior.injection_strategy,
            deserialized.behavior.injection_strategy
        );
    }

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();

        assert!(config.api_key.is_empty());
        assert_eq!(config.model_id, "scribe_v2_realtime");
        assert!(!config.include_timestamps);
    }

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();

        assert!(config.input_device_id.is_none());
        assert_eq!(config.gain, 1.0);
        assert!(!config.noise_suppression);
        assert_eq!(config.silence_threshold, 0.01);
    }

    #[test]
    fn test_behavior_config_default() {
        let config = BehaviorConfig::default();

        assert_eq!(config.injection_strategy, InjectionStrategy::Auto);
        assert_eq!(config.auto_threshold, 20);
        assert_eq!(config.paste_delay_ms, 100);
        assert!(config.auto_inject);
        assert!(config.show_overlay);
        assert!(!config.auto_start);
        assert!(config.minimize_to_tray);
    }

    #[test]
    fn test_global_config() {
        let config = GlobalConfig::default();

        assert!(!config.has_api_key());

        config.set_api_key("test-key".to_string());
        assert!(config.has_api_key());
        assert_eq!(config.api_key(), "test-key");
    }

    #[test]
    fn test_global_config_update() {
        let global = GlobalConfig::default();

        let mut new_config = AppConfig::default();
        new_config.api.api_key = "new-key".to_string();
        new_config.behavior.show_overlay = false;

        global.update(new_config);

        assert_eq!(global.api_key(), "new-key");
        assert!(!global.get().behavior.show_overlay);
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::Path("test error".to_string());
        assert!(err.to_string().contains("test error"));

        let err = ConfigError::Json(serde_json::from_str::<AppConfig>("invalid").unwrap_err());
        assert!(err.to_string().contains("JSON"));
    }

    #[test]
    fn test_config_partial_json() {
        // 测试部分 JSON 能够正确反序列化（使用默认值填充缺失字段）
        let json = r#"{
            "api": {
                "api_key": "test-key"
            }
        }"#;

        let config: AppConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.api.api_key, "test-key");
        assert_eq!(config.api.model_id, "scribe_v2_realtime"); // 默认值
        assert_eq!(config.audio.gain, 1.0); // 默认值
        assert!(config.behavior.show_overlay); // 默认值
    }
}
