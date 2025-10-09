pub mod auto;
pub mod mybatis;
pub mod docker_json;
pub mod raw;
pub mod springboot;
pub mod manager;
pub mod core;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Legacy type alias for compatibility
pub type LogEntry = LogLine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    pub line_number: usize,
    pub content: String,
    pub level: Option<String>,
    pub timestamp: Option<String>,
    pub formatted_content: Option<String>,
    pub metadata: HashMap<String, String>,
    pub processed_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub lines: Vec<LogLine>,
    pub total_lines: usize,
    pub detected_format: Option<String>,
    pub parsing_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseRequest {
    pub content: String,
    pub plugin: Option<String>,
    pub file_path: Option<String>,
    pub chunk_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub description: String,
    pub supported_extensions: Vec<String>,
    pub auto_detectable: bool,
}

pub trait LogParser {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn supported_extensions(&self) -> Vec<String>;
    fn can_parse(&self, content: &str, file_path: Option<&str>) -> bool;
    fn parse(&self, content: &str, request: &ParseRequest) -> Result<ParseResult, String>;
}