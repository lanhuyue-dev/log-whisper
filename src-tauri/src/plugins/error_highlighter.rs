use regex::Regex;
use crate::models::{LogEntry, RenderedBlock, BlockMetadata};
use crate::plugins::{LogRenderer, PluginLifecycle, PluginCapabilities, PerformanceRating, MemoryUsageRating};

/// 错误高亮插件
pub struct ErrorHighlighterRenderer {
    error_patterns: Vec<Regex>,
    warning_patterns: Vec<Regex>,
    enabled: bool,
}

impl ErrorHighlighterRenderer {
    /// 创建新的错误高亮渲染器
    pub fn new() -> Self {
        let error_patterns = vec![
            Regex::new(r"(?i)\b(error|exception|failed|failure|fatal)\b").unwrap(),
            Regex::new(r"(?i)\b(500|timeout|null|undefined)\b").unwrap(),
            Regex::new(r"(?i)\b(crash|abort|panic)\b").unwrap(),
        ];
        
        let warning_patterns = vec![
            Regex::new(r"(?i)\b(warn|warning|deprecated)\b").unwrap(),
            Regex::new(r"(?i)\b(404|not found|missing)\b").unwrap(),
            Regex::new(r"(?i)\b(slow|timeout|retry)\b").unwrap(),
        ];
        
        Self {
            error_patterns,
            warning_patterns,
            enabled: true,
        }
    }
    
    /// 检查是否为错误日志
    fn is_error_log(&self, content: &str) -> bool {
        self.error_patterns.iter().any(|pattern| pattern.is_match(content))
    }
    
    /// 检查是否为警告日志
    fn is_warning_log(&self, content: &str) -> bool {
        self.warning_patterns.iter().any(|pattern| pattern.is_match(content))
    }
    
    /// 提取错误信息
    fn extract_error_info(&self, content: &str) -> Option<String> {
        for pattern in &self.error_patterns {
            if let Some(captures) = pattern.captures(content) {
                if let Some(matched) = captures.get(0) {
                    return Some(matched.as_str().to_string());
                }
            }
        }
        None
    }
    
    /// 提取警告信息
    fn extract_warning_info(&self, content: &str) -> Option<String> {
        for pattern in &self.warning_patterns {
            if let Some(captures) = pattern.captures(content) {
                if let Some(matched) = captures.get(0) {
                    return Some(matched.as_str().to_string());
                }
            }
        }
        None
    }
}

impl LogRenderer for ErrorHighlighterRenderer {
    fn can_handle(&self, entry: &LogEntry) -> bool {
        if !self.enabled {
            return false;
        }
        
        self.is_error_log(&entry.content) || self.is_warning_log(&entry.content)
    }
    
    fn render(&self, entry: &LogEntry) -> Vec<RenderedBlock> {
        let mut blocks = Vec::new();
        
        if self.is_error_log(&entry.content) {
            if let Some(error_info) = self.extract_error_info(&entry.content) {
                let block = RenderedBlock::error(
                    format!("error_{}", entry.line_number),
                    format!("错误: {}", error_info),
                ).with_metadata(BlockMetadata {
                    line_start: entry.line_number,
                    line_end: entry.line_number,
                    char_start: 0,
                    char_end: entry.content.len(),
                    confidence: 0.9,
                });
                blocks.push(block);
            }
        } else if self.is_warning_log(&entry.content) {
            if let Some(warning_info) = self.extract_warning_info(&entry.content) {
                let block = RenderedBlock::warning(
                    format!("warning_{}", entry.line_number),
                    format!("警告: {}", warning_info),
                ).with_metadata(BlockMetadata {
                    line_start: entry.line_number,
                    line_end: entry.line_number,
                    char_start: 0,
                    char_end: entry.content.len(),
                    confidence: 0.8,
                });
                blocks.push(block);
            }
        }
        
        blocks
    }
    
    fn name(&self) -> &str {
        "Error Highlighter"
    }
    
    fn description(&self) -> &str {
        "识别和高亮错误、警告日志，提供快速定位功能"
    }
    
    fn priority(&self) -> u32 {
        5
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl PluginLifecycle for ErrorHighlighterRenderer {
    fn initialize(&mut self) -> Result<(), String> {
        // 重新编译正则表达式
        self.error_patterns = vec![
            Regex::new(r"(?i)\b(error|exception|failed|failure|fatal)\b")
                .map_err(|e| format!("Failed to compile error regex: {}", e))?,
            Regex::new(r"(?i)\b(500|timeout|null|undefined)\b")
                .map_err(|e| format!("Failed to compile error regex: {}", e))?,
            Regex::new(r"(?i)\b(crash|abort|panic)\b")
                .map_err(|e| format!("Failed to compile error regex: {}", e))?,
        ];
        
        self.warning_patterns = vec![
            Regex::new(r"(?i)\b(warn|warning|deprecated)\b")
                .map_err(|e| format!("Failed to compile warning regex: {}", e))?,
            Regex::new(r"(?i)\b(404|not found|missing)\b")
                .map_err(|e| format!("Failed to compile warning regex: {}", e))?,
            Regex::new(r"(?i)\b(slow|timeout|retry)\b")
                .map_err(|e| format!("Failed to compile warning regex: {}", e))?,
        ];
        
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), String> {
        Ok(())
    }
}

impl PluginCapabilities for ErrorHighlighterRenderer {
    fn supported_file_types(&self) -> Vec<String> {
        vec!["*".to_string()]
    }
    
    fn max_file_size(&self) -> Option<usize> {
        None
    }
    
    fn performance_rating(&self) -> PerformanceRating {
        PerformanceRating::VeryHigh
    }
    
    fn memory_usage_rating(&self) -> MemoryUsageRating {
        MemoryUsageRating::Low
    }
}
