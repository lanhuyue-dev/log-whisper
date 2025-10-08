//! SpringBoot 日志处理增强模块
//! 
//! 专门处理SpringBoot应用的各种日志格式

use super::{Plugin, LogEntry, PluginError, PluginType};
use serde_json::Value;
use std::collections::HashMap;
use regex::Regex;

/// SpringBoot JSON 日志解析器
/// 处理容器化SpringBoot应用的JSON格式日志
pub struct SpringBootJsonParser {
    name: String,
    version: String,
    description: String,
    priority: u32,
}

impl SpringBootJsonParser {
    pub fn new() -> Self {
        Self {
            name: "springboot_json".to_string(),
            version: "1.0.0".to_string(),
            description: "SpringBoot JSON格式日志解析器".to_string(),
            priority: 8,
        }
    }
    
    /// 检查是否为SpringBoot JSON日志
    fn is_springboot_json(&self, line: &str) -> bool {
        // 检查是否为JSON格式
        if !line.trim().starts_with('{') || !line.trim().ends_with('}') {
            return false;
        }
        
        // 尝试解析JSON
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            // 检查是否包含SpringBoot日志特征
            if let Some(log_content) = json.get("log").and_then(|v| v.as_str()) {
                // 检查日志内容是否包含SpringBoot特征
                return self.has_springboot_characteristics(log_content);
            }
        }
        
        false
    }
    
    /// 检查日志内容是否具有SpringBoot特征
    fn has_springboot_characteristics(&self, content: &str) -> bool {
        // 检查时间戳格式
        let timestamp_pattern = r"\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}\.\d{3}";
        let timestamp_regex = Regex::new(timestamp_pattern).unwrap();
        
        // 检查日志级别
        let level_pattern = r"\b(TRACE|DEBUG|INFO|WARN|ERROR|FATAL)\b";
        let level_regex = Regex::new(level_pattern).unwrap();
        
        // 检查线程信息
        let has_thread = content.contains('[') && content.contains(']') && content.contains("---");
        
        timestamp_regex.is_match(content) && 
        level_regex.is_match(content) && 
        has_thread
    }
    
    /// 解析SpringBoot JSON日志
    fn parse_springboot_json(&self, line: &str) -> Result<SpringBootLogData, PluginError> {
        let json: Value = serde_json::from_str(line)
            .map_err(|e| PluginError::ProcessingFailed(format!("JSON解析失败: {}", e)))?;
        
        let log_content = json.get("log")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::ProcessingFailed("缺少log字段".to_string()))?;
        
        let stream = json.get("stream")
            .and_then(|v| v.as_str())
            .unwrap_or("stdout");
        
        let timestamp = json.get("time")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // 解析日志内容
        let parsed_content = self.parse_log_content(log_content)?;
        
        Ok(SpringBootLogData {
            content: log_content.to_string(),
            stream: stream.to_string(),
            timestamp: timestamp.to_string(),
            level: parsed_content.level,
            thread: parsed_content.thread,
            class: parsed_content.class,
            message: parsed_content.message,
        })
    }
    
    /// 解析日志内容
    fn parse_log_content(&self, content: &str) -> Result<ParsedLogContent, PluginError> {
        // 时间戳正则
        let timestamp_regex = Regex::new(r"\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}\.\d{3}").unwrap();
        // 级别正则
        let level_regex = Regex::new(r"\b(TRACE|DEBUG|INFO|WARN|ERROR|FATAL)\b").unwrap();
        // 线程正则
        let thread_regex = Regex::new(r"\[([^\]]+)\]").unwrap();
        
        let level = level_regex.find(content)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "INFO".to_string());
        
        let thread = thread_regex.find(content)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "".to_string());
        
        // 提取类名（在级别和线程之后）
        let class = if let Some(level_pos) = level_regex.find(content) {
            let after_level = &content[level_pos.end()..];
            if let Some(thread_pos) = thread_regex.find(after_level) {
                let after_thread = &after_level[thread_pos.end()..];
                if let Some(class_end) = after_thread.find(" :") {
                    after_thread[..class_end].trim().to_string()
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };
        
        // 提取消息内容
        let message = if let Some(colon_pos) = content.find(" : ") {
            content[colon_pos + 3..].trim().to_string()
        } else {
            content.to_string()
        };
        
        Ok(ParsedLogContent {
            level,
            thread,
            class,
            message,
        })
    }
}

impl Plugin for SpringBootJsonParser {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn priority(&self) -> u32 {
        self.priority
    }
    
    fn plugin_type(&self) -> PluginType {
        PluginType::Parser
    }
    
    fn can_handle(&self, log_line: &str) -> bool {
        self.is_springboot_json(log_line)
    }
    
    fn process(&self, log_entry: &mut LogEntry) -> Result<(), PluginError> {
        if !self.can_handle(&log_entry.content) {
            return Ok(());
        }
        
        let springboot_data = self.parse_springboot_json(&log_entry.content)?;
        
        // 更新日志条目
        log_entry.content = springboot_data.message;
        log_entry.timestamp = Some(springboot_data.timestamp);
        log_entry.level = Some(springboot_data.level);
        
        // 添加元数据
        log_entry.add_metadata("stream".to_string(), springboot_data.stream);
        log_entry.add_metadata("thread".to_string(), springboot_data.thread);
        log_entry.add_metadata("class".to_string(), springboot_data.class);
        log_entry.add_metadata("format".to_string(), "springboot_json".to_string());
        log_entry.add_metadata("original_content".to_string(), springboot_data.content);
        
        log_entry.mark_processed(self.name.clone());
        Ok(())
    }
    
    fn initialize(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn cleanup(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// SpringBoot 异常聚合器
/// 专门处理SpringBoot应用的异常堆栈跟踪
pub struct SpringBootExceptionAggregator {
    name: String,
    version: String,
    description: String,
    priority: u32,
}

impl SpringBootExceptionAggregator {
    pub fn new() -> Self {
        Self {
            name: "springboot_exception_aggregator".to_string(),
            version: "1.0.0".to_string(),
            description: "SpringBoot异常堆栈跟踪聚合器".to_string(),
            priority: 12,
        }
    }
    
    /// 检查是否为异常开始行
    fn is_exception_start(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // 检查是否包含异常关键词
        trimmed.contains("Exception:") ||
        trimmed.contains("Error:") ||
        trimmed.ends_with("Exception") ||
        trimmed.ends_with("Error") ||
        trimmed.starts_with("Caused by:") ||
        trimmed.starts_with("Suppressed:")
    }
    
    /// 检查是否为堆栈跟踪行
    fn is_stack_trace_line(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        // 标准Java堆栈跟踪格式
        trimmed.starts_with("at ") ||
        trimmed.starts_with("\tat ") ||
        trimmed.contains("common frames omitted") ||
        trimmed.contains("more")
    }
    
    /// 检查是否为异常继续行
    fn is_exception_continuation(&self, line: &str) -> bool {
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            return false;
        }
        
        // 没有时间戳的非堆栈跟踪行，可能是异常消息
        if !self.has_timestamp(line) && !trimmed.starts_with("at ") {
            // 检查是否是下一个正常日志的开始
            let level_keywords = ["INFO", "DEBUG", "WARN", "ERROR", "TRACE"];
            let has_level_keyword = level_keywords.iter().any(|keyword| line.contains(keyword));
            
            if !has_level_keyword {
                return true;
            }
        }
        
        false
    }
    
    /// 检查是否包含时间戳
    fn has_timestamp(&self, line: &str) -> bool {
        let timestamp_regex = Regex::new(r"\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}\.\d{3}").unwrap();
        timestamp_regex.is_match(line)
    }
}

impl Plugin for SpringBootExceptionAggregator {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn priority(&self) -> u32 {
        self.priority
    }
    
    fn plugin_type(&self) -> PluginType {
        PluginType::Filter
    }
    
    fn can_handle(&self, log_line: &str) -> bool {
        self.is_exception_start(log_line) ||
        self.is_stack_trace_line(log_line) ||
        self.is_exception_continuation(log_line)
    }
    
    fn process(&self, log_entry: &mut LogEntry) -> Result<(), PluginError> {
        if !self.can_handle(&log_entry.content) {
            return Ok(());
        }
        
        // 标记异常类型
        if self.is_exception_start(&log_entry.content) {
            log_entry.add_metadata("exception_type".to_string(), "exception_start".to_string());
        } else if self.is_stack_trace_line(&log_entry.content) {
            log_entry.add_metadata("exception_type".to_string(), "stack_trace".to_string());
        } else if self.is_exception_continuation(&log_entry.content) {
            log_entry.add_metadata("exception_type".to_string(), "exception_message".to_string());
        }
        
        // 添加异常处理标记
        log_entry.add_metadata("processed_by".to_string(), "springboot_exception_aggregator".to_string());
        log_entry.add_metadata("format".to_string(), "springboot_exception".to_string());
        
        log_entry.mark_processed(self.name.clone());
        Ok(())
    }
    
    fn initialize(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn cleanup(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// SpringBoot 日志数据结构
#[derive(Debug, Clone)]
struct SpringBootLogData {
    content: String,
    stream: String,
    timestamp: String,
    level: String,
    thread: String,
    class: String,
    message: String,
}

/// 解析后的日志内容
#[derive(Debug, Clone)]
struct ParsedLogContent {
    level: String,
    thread: String,
    class: String,
    message: String,
}
