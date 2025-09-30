use serde::{Deserialize, Serialize};
use serde_json;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{LogEntry, RenderedBlock, BlockType, BlockMetadata};
use crate::plugins::{LogRenderer, PluginCapabilities, PerformanceRating, MemoryUsageRating};

/// Docker JSON日志格式的单条记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerLogRecord {
    /// 日志内容
    pub log: String,
    /// 流类型 (stdout, stderr)
    pub stream: String,
    /// 时间戳
    pub time: String,
}

/// Docker JSON日志解析器
pub struct DockerJsonRenderer {
    enabled: bool,
}

impl DockerJsonRenderer {
    /// 创建新的Docker JSON渲染器
    pub fn new() -> Self {
        Self {
            enabled: true,
        }
    }

    /// 解析Docker JSON日志行
    fn parse_docker_json_line(&self, line: &str) -> Option<DockerLogRecord> {
        // 尝试解析JSON格式的Docker日志
        serde_json::from_str::<DockerLogRecord>(line).ok()
    }

    /// 格式化Docker日志记录
    fn format_docker_record(&self, record: &DockerLogRecord) -> String {
        // 解析时间戳
        let formatted_time = if let Ok(dt) = DateTime::parse_from_rfc3339(&record.time) {
            dt.with_timezone(&Utc).format("%Y-%m-%d %H:%M:%S%.3f UTC").to_string()
        } else {
            record.time.clone()
        };

        // 根据流类型选择不同的格式
        let stream_indicator = match record.stream.as_str() {
            "stdout" => "📤",
            "stderr" => "❌", 
            _ => "📄",
        };

        format!(
            "{} {} [{}] {}",
            stream_indicator,
            formatted_time,
            record.stream.to_uppercase(),
            record.log.trim_end()
        )
    }

    /// 创建JSON内容块
    fn create_json_block(&self, record: &DockerLogRecord, line_number: usize) -> RenderedBlock {
        let id = Uuid::new_v4().to_string();
        
        // 格式化JSON内容
        let formatted_json = serde_json::to_string_pretty(record)
            .unwrap_or_else(|_| serde_json::to_string(record).unwrap_or_default());
        
        let metadata = BlockMetadata {
            line_start: line_number,
            line_end: line_number,
            char_start: 0,
            char_end: formatted_json.len(),
            confidence: 0.95,
        };

        RenderedBlock::new(
            id,
            BlockType::Json,
            "Docker JSON Log".to_string(),
            serde_json::to_string(record).unwrap_or_default(),
            formatted_json,
        ).with_metadata(metadata)
    }

    /// 创建格式化内容块
    fn create_formatted_block(&self, record: &DockerLogRecord, line_number: usize) -> RenderedBlock {
        let id = Uuid::new_v4().to_string();
        let formatted_content = self.format_docker_record(record);
        
        // 根据流类型确定块类型
        let block_type = match record.stream.as_str() {
            "stderr" => BlockType::Error,
            "stdout" => {
                // 检查内容是否包含错误信息
                let content_lower = record.log.to_lowercase();
                if content_lower.contains("error") || content_lower.contains("exception") {
                    BlockType::Error
                } else if content_lower.contains("warn") {
                    BlockType::Warning
                } else {
                    BlockType::Info
                }
            },
            _ => BlockType::Info,
        };
        
        let metadata = BlockMetadata {
            line_start: line_number,
            line_end: line_number,
            char_start: 0,
            char_end: formatted_content.len(),
            confidence: 0.9,
        };

        let title = match record.stream.as_str() {
            "stdout" => "Container Output (stdout)",
            "stderr" => "Container Error (stderr)",
            _ => "Container Log",
        };

        RenderedBlock::new(
            id,
            block_type,
            title.to_string(),
            record.log.clone(),
            formatted_content,
        ).with_metadata(metadata)
    }

    /// 检查是否为Docker JSON格式
    fn is_docker_json_format(&self, content: &str) -> bool {
        // Docker JSON日志必须包含这些字段
        content.trim_start().starts_with('{') && 
        content.contains("\"log\":") && 
        content.contains("\"stream\":") && 
        content.contains("\"time\":")
    }
}

impl Default for DockerJsonRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl LogRenderer for DockerJsonRenderer {
    fn can_handle(&self, entry: &LogEntry) -> bool {
        if !self.enabled {
            return false;
        }

        // 检查是否为Docker JSON格式
        self.is_docker_json_format(&entry.raw_line)
    }

    fn render(&self, entry: &LogEntry) -> Vec<RenderedBlock> {
        let mut blocks = Vec::new();

        if let Some(record) = self.parse_docker_json_line(&entry.raw_line) {
            // 创建格式化的内容块
            blocks.push(self.create_formatted_block(&record, entry.line_number));
            
            // 如果日志内容比较复杂，也提供原始JSON块
            if record.log.len() > 100 || record.log.contains('\n') {
                blocks.push(self.create_json_block(&record, entry.line_number));
            }
        }

        blocks
    }

    fn name(&self) -> &str {
        "DockerJSON"
    }

    fn description(&self) -> &str {
        "Docker容器JSON日志解析器，支持解析Docker容器标准JSON日志格式"
    }

    fn priority(&self) -> u32 {
        15 // 中等优先级，在MyBatis和JSON插件之间
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_config(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "enabled": self.enabled,
            "name": self.name(),
            "description": self.description(),
            "priority": self.priority(),
            "supported_formats": ["docker_json"],
            "features": {
                "parse_timestamps": true,
                "stream_detection": true,
                "error_highlighting": true,
                "json_formatting": true
            }
        }))
    }

    fn set_config(&mut self, config: serde_json::Value) -> Result<(), String> {
        if let Some(enabled) = config.get("enabled").and_then(|v| v.as_bool()) {
            self.enabled = enabled;
        }
        Ok(())
    }
}

impl PluginCapabilities for DockerJsonRenderer {
    fn supported_file_types(&self) -> Vec<String> {
        vec![
            "*.log".to_string(),
            "*.json".to_string(),
            "*-json.log".to_string(),
            "docker-*.log".to_string(),
        ]
    }

    fn max_file_size(&self) -> Option<usize> {
        Some(100 * 1024 * 1024) // 100MB
    }

    fn performance_rating(&self) -> PerformanceRating {
        PerformanceRating::High
    }

    fn memory_usage_rating(&self) -> MemoryUsageRating {
        MemoryUsageRating::Medium
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LogLevel;

    #[test]
    fn test_parse_docker_json_line() {
        let renderer = DockerJsonRenderer::new();
        let json_line = r#"{"log":"Hello world\n","stream":"stdout","time":"2023-01-01T10:00:00.123456789Z"}"#;
        
        let record = renderer.parse_docker_json_line(json_line);
        assert!(record.is_some());
        
        let record = record.unwrap();
        assert_eq!(record.log, "Hello world\n");
        assert_eq!(record.stream, "stdout");
        assert_eq!(record.time, "2023-01-01T10:00:00.123456789Z");
    }

    #[test]
    fn test_can_handle_docker_json() {
        let renderer = DockerJsonRenderer::new();
        
        // 有效的Docker JSON
        let valid_entry = LogEntry::new(1, r#"{"log":"test log\n","stream":"stdout","time":"2023-01-01T10:00:00Z"}"#.to_string());
        assert!(renderer.can_handle(&valid_entry));
        
        // 无效的JSON
        let invalid_entry = LogEntry::new(1, "This is not JSON".to_string());
        assert!(!renderer.can_handle(&invalid_entry));
        
        // 不完整的Docker JSON
        let incomplete_entry = LogEntry::new(1, r#"{"log":"test"}"#.to_string());
        assert!(!renderer.can_handle(&incomplete_entry));
    }

    #[test]
    fn test_render_docker_json() {
        let renderer = DockerJsonRenderer::new();
        let json_line = r#"{"log":"Error: Something went wrong\n","stream":"stderr","time":"2023-01-01T10:00:00Z"}"#;
        let entry = LogEntry::new(1, json_line.to_string());
        
        let blocks = renderer.render(&entry);
        assert!(!blocks.is_empty());
        
        // 应该生成格式化的内容块
        let formatted_block = &blocks[0];
        assert_eq!(formatted_block.block_type, BlockType::Error);
        assert!(formatted_block.title.contains("stderr"));
    }

    #[test]
    fn test_format_docker_record() {
        let renderer = DockerJsonRenderer::new();
        let record = DockerLogRecord {
            log: "Test message\n".to_string(),
            stream: "stdout".to_string(),
            time: "2023-01-01T10:00:00.123Z".to_string(),
        };
        
        let formatted = renderer.format_docker_record(&record);
        assert!(formatted.contains("📤")); // stdout indicator
        assert!(formatted.contains("STDOUT"));
        assert!(formatted.contains("Test message"));
    }
}