use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 日志条目结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 行号
    pub line_number: usize,
    /// 时间戳
    pub timestamp: Option<DateTime<Utc>>,
    /// 日志级别
    pub level: LogLevel,
    /// 日志内容
    pub content: String,
    /// 原始行内容
    pub raw_line: String,
}

/// 日志级别枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Unknown,
}

impl LogLevel {
    /// 从字符串解析日志级别
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" | "warning" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Unknown,
        }
    }
    
    /// 获取日志级别的颜色
    pub fn color(&self) -> &'static str {
        match self {
            LogLevel::Debug => "text-gray-500",
            LogLevel::Info => "text-blue-600",
            LogLevel::Warn => "text-yellow-600",
            LogLevel::Error => "text-red-600",
            LogLevel::Unknown => "text-gray-400",
        }
    }
    
    /// 获取日志级别的背景色
    pub fn background_color(&self) -> &'static str {
        match self {
            LogLevel::Debug => "bg-gray-50",
            LogLevel::Info => "bg-blue-50",
            LogLevel::Warn => "bg-yellow-50",
            LogLevel::Error => "bg-red-50",
            LogLevel::Unknown => "bg-gray-50",
        }
    }
}

impl LogEntry {
    /// 创建新的日志条目
    pub fn new(line_number: usize, content: String) -> Self {
        let level = Self::extract_log_level(&content);
        let timestamp = Self::extract_timestamp(&content);
        
        Self {
            line_number,
            timestamp,
            level,
            content: content.clone(),
            raw_line: content,
        }
    }
    
    /// 从字符串提取日志级别
    fn extract_log_level(content: &str) -> LogLevel {
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("error") || content_lower.contains("exception") {
            LogLevel::Error
        } else if content_lower.contains("warn") {
            LogLevel::Warn
        } else if content_lower.contains("info") {
            LogLevel::Info
        } else if content_lower.contains("debug") {
            LogLevel::Debug
        } else {
            LogLevel::Unknown
        }
    }
    
    /// 从字符串提取时间戳
    fn extract_timestamp(content: &str) -> Option<DateTime<Utc>> {
        // 常见的时间戳格式
        let timestamp_patterns = [
            "%Y-%m-%d %H:%M:%S%.3f",  // 2024-01-01 10:00:00.123
            "%Y-%m-%d %H:%M:%S",      // 2024-01-01 10:00:00
            "%Y-%m-%dT%H:%M:%S%.3fZ", // 2024-01-01T10:00:00.123Z
            "%Y-%m-%dT%H:%M:%SZ",     // 2024-01-01T10:00:00Z
        ];
        
        for pattern in &timestamp_patterns {
            if let Ok(dt) = chrono::DateTime::parse_from_str(content, pattern) {
                return Some(dt.with_timezone(&Utc));
            }
        }
        
        None
    }
    
    /// 检查是否为错误日志
    pub fn is_error(&self) -> bool {
        self.level == LogLevel::Error || 
        self.content.to_lowercase().contains("error") ||
        self.content.to_lowercase().contains("exception") ||
        self.content.to_lowercase().contains("failed")
    }
    
    /// 检查是否为警告日志
    pub fn is_warning(&self) -> bool {
        self.level == LogLevel::Warn || 
        self.content.to_lowercase().contains("warn")
    }
    
    /// 获取日志条目的显示文本
    pub fn display_text(&self) -> String {
        if let Some(ts) = &self.timestamp {
            format!("[{}] {}", ts.format("%H:%M:%S"), self.content)
        } else {
            self.content.clone()
        }
    }
}
