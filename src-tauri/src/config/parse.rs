use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseConfig {
    pub chunk_size: usize,
    pub auto_detect_format: bool,
    pub default_plugin: String,
    pub max_file_size: u64, // bytes
    pub auto_parse: bool,
    pub show_line_numbers: bool,
    pub timeout_seconds: u64,
}

impl Default for ParseConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1000,
            auto_detect_format: true,
            default_plugin: "auto".to_string(),
            max_file_size: 100 * 1024 * 1024, // 100MB
            auto_parse: true,
            show_line_numbers: true,
            timeout_seconds: 30,
        }
    }
}