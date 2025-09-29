use chrono::{DateTime, Utc};
use regex::Regex;
use crate::models::{LogEntry, LogLevel, ParseError};

/// 行解析器
pub struct LineParser {
    timestamp_patterns: Vec<Regex>,
    level_patterns: Vec<Regex>,
}

impl LineParser {
    /// 创建新的行解析器
    pub fn new() -> Self {
        let timestamp_patterns = vec![
            // Spring Boot格式: 2024-01-01 10:00:00.123
            Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}").unwrap(),
            // 标准格式: 2024-01-01 10:00:00
            Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap(),
            // ISO格式: 2024-01-01T10:00:00.123Z
            Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z").unwrap(),
            // ISO格式: 2024-01-01T10:00:00Z
            Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z").unwrap(),
            // Logback/Log4j格式: 2024-01-01 10:00:00,123
            Regex::new(r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2},\d{3}").unwrap(),
            // 简化格式: 10:00:00.123
            Regex::new(r"\d{2}:\d{2}:\d{2}\.\d{3}").unwrap(),
            // 简化格式: 10:00:00
            Regex::new(r"\d{2}:\d{2}:\d{2}").unwrap(),
            // 年月日格式: 2024/01/01 10:00:00
            Regex::new(r"\d{4}/\d{2}/\d{2} \d{2}:\d{2}:\d{2}").unwrap(),
            // 短年份格式: 24-01-01 10:00:00
            Regex::new(r"\d{2}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap(),
        ];
        
        let level_patterns = vec![
            // 标准日志级别
            Regex::new(r"(?i)\b(TRACE|DEBUG|INFO|WARN|WARNING|ERROR|FATAL)\b").unwrap(),
            // Spring Boot格式级别 (位于时间戳后面)
            Regex::new(r"\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}[.,]\d{3}\s+(TRACE|DEBUG|INFO|WARN|WARNING|ERROR|FATAL)").unwrap(),
            // Logback格式级别
            Regex::new(r"\[\w+\]\s+(TRACE|DEBUG|INFO|WARN|WARNING|ERROR|FATAL)").unwrap(),
            // Log4j格式级别
            Regex::new(r"(TRACE|DEBUG|INFO|WARN|WARNING|ERROR|FATAL)\s+\[\w+\]").unwrap(),
            // 中文日志级别
            Regex::new(r"\b(调试|信息|警告|错误|严重)\b").unwrap(),
        ];
        
        Self {
            timestamp_patterns,
            level_patterns,
        }
    }
    
    /// 解析单行日志
    pub fn parse_line(&self, line_number: usize, content: &str) -> Result<LogEntry, ParseError> {
        let timestamp = self.extract_timestamp(content);
        let level = self.extract_log_level(content);
        
        Ok(LogEntry {
            line_number,
            timestamp,
            level,
            content: content.to_string(),
            raw_line: content.to_string(),
        })
    }
    
    /// 批量解析日志行
    pub fn parse_lines(&self, lines: Vec<String>) -> Result<Vec<LogEntry>, ParseError> {
        let mut entries = Vec::new();
        
        for (index, line) in lines.iter().enumerate() {
            let entry = self.parse_line(index + 1, line)?;
            entries.push(entry);
        }
        
        Ok(entries)
    }
    
    /// 提取时间戳
    fn extract_timestamp(&self, content: &str) -> Option<DateTime<Utc>> {
        for pattern in &self.timestamp_patterns {
            if let Some(mat) = pattern.find(content) {
                let timestamp_str = mat.as_str();
                
                // 尝试解析不同的时间戳格式
                let formats = [
                    "%Y-%m-%d %H:%M:%S%.3f",        // 2024-01-01 10:00:00.123
                    "%Y-%m-%d %H:%M:%S",            // 2024-01-01 10:00:00
                    "%Y-%m-%dT%H:%M:%S%.3fZ",       // 2024-01-01T10:00:00.123Z
                    "%Y-%m-%dT%H:%M:%SZ",           // 2024-01-01T10:00:00Z
                    "%Y-%m-%d %H:%M:%S,%.3f",       // 2024-01-01 10:00:00,123 (Logback)
                    "%H:%M:%S%.3f",                // 10:00:00.123
                    "%H:%M:%S",                     // 10:00:00
                    "%Y/%m/%d %H:%M:%S",            // 2024/01/01 10:00:00
                    "%y-%m-%d %H:%M:%S",            // 24-01-01 10:00:00
                ];
                
                for format in &formats {
                    if let Ok(dt) = DateTime::parse_from_str(timestamp_str, format) {
                        return Some(dt.with_timezone(&Utc));
                    }
                }
            }
        }
        
        None
    }
    
    /// 提取日志级别
    fn extract_log_level(&self, content: &str) -> LogLevel {
        // 首先检查是否有明确的级别标识
        for pattern in &self.level_patterns {
            if let Some(captures) = pattern.captures(content) {
                if let Some(level_match) = captures.get(0) {
                    return LogLevel::from_str(level_match.as_str());
                }
            }
        }
        
        // 如果没有明确的级别标识，根据内容判断
        let content_lower = content.to_lowercase();
        
        // 检查错误相关关键词
        if content_lower.contains("error") || content_lower.contains("exception") || 
           content_lower.contains("failed") || content_lower.contains("fatal") ||
           content_lower.contains("错误") || content_lower.contains("异常") ||
           content_lower.contains("失败") || content_lower.contains("严重") {
            LogLevel::Error
        // 检查警告相关关键词
        } else if content_lower.contains("warn") || content_lower.contains("warning") ||
                  content_lower.contains("警告") {
            LogLevel::Warn
        // 检查信息相关关键词
        } else if content_lower.contains("info") || content_lower.contains("information") ||
                  content_lower.contains("信息") {
            LogLevel::Info
        // 检查调试相关关键词
        } else if content_lower.contains("debug") || content_lower.contains("trace") ||
                  content_lower.contains("调试") {
            LogLevel::Debug
        } else {
            LogLevel::Unknown
        }
    }
    
    /// 检查是否为多行日志的开始
    pub fn is_multiline_start(&self, content: &str) -> bool {
        // 检查是否包含时间戳和级别标识
        self.timestamp_patterns.iter().any(|pattern| pattern.is_match(content)) &&
        self.level_patterns.iter().any(|pattern| pattern.is_match(content))
    }
    
    /// 检查是否为多行日志的继续
    pub fn is_multiline_continuation(&self, content: &str) -> bool {
        // 检查是否不包含时间戳和级别标识，但包含日志内容
        !self.timestamp_patterns.iter().any(|pattern| pattern.is_match(content)) &&
        !self.level_patterns.iter().any(|pattern| pattern.is_match(content)) &&
        !content.trim().is_empty()
    }
    
    /// 合并多行日志
    pub fn merge_multiline_logs(&self, entries: Vec<LogEntry>) -> Vec<LogEntry> {
        let mut merged_entries = Vec::new();
        let mut current_entry: Option<LogEntry> = None;
        
        for entry in entries {
            if self.is_multiline_start(&entry.content) {
                // 保存之前的条目
                if let Some(prev_entry) = current_entry.take() {
                    merged_entries.push(prev_entry);
                }
                // 开始新的条目
                current_entry = Some(entry);
            } else if self.is_multiline_continuation(&entry.content) {
                // 合并到当前条目
                if let Some(ref mut current) = current_entry {
                    current.content.push('\n');
                    current.content.push_str(&entry.content);
                }
            } else {
                // 普通条目
                if let Some(prev_entry) = current_entry.take() {
                    merged_entries.push(prev_entry);
                }
                merged_entries.push(entry);
            }
        }
        
        // 处理最后一个条目
        if let Some(entry) = current_entry {
            merged_entries.push(entry);
        }
        
        merged_entries
    }
    
    /// 清理日志内容
    pub fn clean_log_content(&self, content: &str) -> String {
        let mut cleaned = content.to_string();
        
        // 移除多余的空白字符
        cleaned = cleaned.replace("  ", " ");
        cleaned = cleaned.replace("\t", " ");
        
        // 移除行首行尾空白
        cleaned = cleaned.trim().to_string();
        
        cleaned
    }
    
    /// 提取关键信息
    pub fn extract_key_info(&self, content: &str) -> Vec<String> {
        let mut key_info = Vec::new();
        
        // 提取IP地址
        let ip_pattern = Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap();
        if let Some(mat) = ip_pattern.find(content) {
            key_info.push(format!("IP: {}", mat.as_str()));
        }
        
        // 提取URL
        let url_pattern = Regex::new(r"https?://[^\s]+").unwrap();
        for mat in url_pattern.find_iter(content) {
            key_info.push(format!("URL: {}", mat.as_str()));
        }
        
        // 提取错误代码
        let error_code_pattern = Regex::new(r"\b\d{3,4}\b").unwrap();
        for mat in error_code_pattern.find_iter(content) {
            let code = mat.as_str();
            if code.parse::<u32>().map_or(false, |n| n >= 100 && n <= 599) {
                key_info.push(format!("Error Code: {}", code));
            }
        }
        
        key_info
    }
}

impl Default for LineParser {
    fn default() -> Self {
        Self::new()
    }
}
