use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 应用配置模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub id: Option<i64>,
    pub key: String,
    pub value: String,
    pub category: String,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 主题配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub mode: ThemeMode,
    pub primary_color: String,
    pub accent_color: String,
    pub font_size: u32,
    pub font_family: String,
}

/// 主题模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    Auto,
}

/// 解析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseConfig {
    pub auto_parse: bool,
    pub show_line_numbers: bool,
    pub max_file_size: u64, // MB
    pub chunk_size: usize,
    pub timeout_seconds: u64,
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub auto_update: bool,
    pub enable_notifications: bool,
    pub plugin_directory: String,
    pub max_plugins: u32,
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
    pub always_on_top: bool,
    pub remember_position: bool,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub log_level: String,
    pub log_file: String,
    pub max_log_size: u64, // MB
    pub log_rotation: bool,
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub enable_gpu_acceleration: bool,
    pub max_memory_usage: u64, // MB
    pub cache_size: u64, // MB
    pub enable_compression: bool,
}

/// 配置分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigCategory {
    Theme,
    Parse,
    Plugin,
    Window,
    Log,
    Performance,
    General,
}

impl std::fmt::Display for ConfigCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigCategory::Theme => write!(f, "Theme"),
            ConfigCategory::Parse => write!(f, "Parse"),
            ConfigCategory::Plugin => write!(f, "Plugin"),
            ConfigCategory::Window => write!(f, "Window"),
            ConfigCategory::Log => write!(f, "Log"),
            ConfigCategory::Performance => write!(f, "Performance"),
            ConfigCategory::General => write!(f, "General"),
        }
    }
}

/// 配置更新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigUpdateRequest {
    pub key: String,
    pub value: String,
    pub category: ConfigCategory,
}

/// 配置响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub message: String,
    pub config: Option<AppConfig>,
}

/// 批量配置更新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfigUpdateRequest {
    pub configs: Vec<ConfigUpdateRequest>,
}

/// 配置查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigQueryRequest {
    pub category: Option<ConfigCategory>,
    pub key: Option<String>,
}

/// 配置列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigListResponse {
    pub success: bool,
    pub message: String,
    pub configs: Vec<AppConfig>,
    pub total: usize,
}
