//! 内置插件实现
//! 
//! 提供各种日志格式的解析和渲染插件

use super::{Plugin, LogEntry, PluginError, PluginType};
use serde_json::Value;
use std::collections::HashMap;
use regex::Regex;

/// Docker JSON 日志解析器
pub struct DockerJsonParser {
    name: String,
    version: String,
    description: String,
    priority: u32,
}

impl DockerJsonParser {
    pub fn new() -> Self {
        Self {
            name: "docker_json".to_string(),
            version: "1.0.0".to_string(),
            description: "Docker容器JSON格式日志解析器".to_string(),
            priority: 5, // 最高优先级，确保优先处理容器JSON
        }
    }
}

impl Plugin for DockerJsonParser {
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
        // 检查是否为Docker JSON格式
        log_line.trim().starts_with('{') && 
        log_line.trim().ends_with('}') &&
        log_line.contains("\"log\":") &&
        log_line.contains("\"stream\":") &&
        log_line.contains("\"time\":")
    }
    
    fn process(&self, log_entry: &mut LogEntry) -> Result<(), PluginError> {
        if !self.can_handle(&log_entry.content) {
            return Ok(());
        }
        
        match serde_json::from_str::<Value>(&log_entry.content) {
            Ok(json) => {
                // 提取日志内容
                if let Some(log_content) = json.get("log").and_then(|v| v.as_str()) {
                    log_entry.content = log_content.trim().to_string();
                }
                
                // 提取并解析时间戳
                if let Some(time) = json.get("time").and_then(|v| v.as_str()) {
                    // 尝试解析Docker时间格式
                    let parsed_time = self.parse_docker_time(time);
                    log_entry.timestamp = parsed_time;
                    log_entry.add_metadata("original_time".to_string(), time.to_string());
                }
                
                // 提取流类型并确定日志级别
                if let Some(stream) = json.get("stream").and_then(|v| v.as_str()) {
                    let level = if stream == "stderr" {
                        "ERROR"
                    } else {
                        // 根据日志内容确定级别
                        let content_lower = log_entry.content.to_lowercase();
                        if content_lower.contains("error") || content_lower.contains("exception") {
                            "ERROR"
                        } else if content_lower.contains("warn") {
                            "WARN"
                        } else if content_lower.contains("debug") {
                            "DEBUG"
                        } else {
                            "INFO"
                        }
                    };
                    log_entry.level = Some(level.to_string());
                }
                
                // 添加元数据
                log_entry.add_metadata("stream".to_string(), 
                    json.get("stream").and_then(|v| v.as_str()).unwrap_or("unknown").to_string());
                log_entry.add_metadata("format".to_string(), "docker_json".to_string());
                
                log_entry.mark_processed(self.name.clone());
                Ok(())
            }
            Err(e) => Err(PluginError::ProcessingFailed(format!("JSON解析失败: {}", e)))
        }
    }
    
    fn initialize(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn cleanup(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

impl DockerJsonParser {
    /// 解析Docker时间格式
    fn parse_docker_time(&self, time_str: &str) -> Option<String> {
        // Docker时间格式通常是 RFC3339 格式
        // 例如: "2025-09-16T08:17:32.172897326Z"
        if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(time_str) {
            // 转换为更易读的格式
            Some(parsed.format("%Y-%m-%d %H:%M:%S%.3f").to_string())
        } else {
            // 如果解析失败，返回原始时间
            Some(time_str.to_string())
        }
    }
}

/// SpringBoot 日志解析器
pub struct SpringBootParser {
    name: String,
    version: String,
    description: String,
    priority: u32,
    timestamp_regex: Regex,
    level_regex: Regex,
}

impl SpringBootParser {
    pub fn new() -> Result<Self, regex::Error> {
        Ok(Self {
            name: "springboot".to_string(),
            version: "1.0.0".to_string(),
            description: "SpringBoot标准日志格式解析器".to_string(),
            priority: 5,
            timestamp_regex: Regex::new(r"\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}\.\d{3}")?,
            level_regex: Regex::new(r"\b(TRACE|DEBUG|INFO|WARN|ERROR|FATAL)\b")?,
        })
    }
}

impl Plugin for SpringBootParser {
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
        // 检查SpringBoot日志格式
        self.timestamp_regex.is_match(log_line) && 
        self.level_regex.is_match(log_line) &&
        log_line.contains("---")
    }
    
    fn process(&self, log_entry: &mut LogEntry) -> Result<(), PluginError> {
        if !self.can_handle(&log_entry.content) {
            return Ok(());
        }
        
        // 提取时间戳
        if let Some(timestamp_match) = self.timestamp_regex.find(&log_entry.content) {
            log_entry.timestamp = Some(timestamp_match.as_str().to_string());
        }
        
        // 提取日志级别
        if let Some(level_match) = self.level_regex.find(&log_entry.content) {
            log_entry.level = Some(level_match.as_str().to_string());
        }
        
        // 提取线程信息
        if let Some(thread_start) = log_entry.content.find('[') {
            if let Some(thread_end) = log_entry.content[thread_start..].find(']') {
                let thread_info = &log_entry.content[thread_start..thread_start + thread_end + 1];
                log_entry.add_metadata("thread".to_string(), thread_info.to_string());
            }
        }
        
        // 提取类名
        if let Some(class_start) = log_entry.content.find("] ") {
            let after_bracket = &log_entry.content[class_start + 2..];
            if let Some(class_end) = after_bracket.find(" :") {
                let class_name = &after_bracket[..class_end];
                log_entry.add_metadata("class".to_string(), class_name.to_string());
            }
        }
        
        log_entry.add_metadata("format".to_string(), "springboot".to_string());
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

/// 堆栈跟踪聚合器
pub struct StackTraceAggregator {
    name: String,
    version: String,
    description: String,
    priority: u32,
}

impl StackTraceAggregator {
    pub fn new() -> Self {
        Self {
            name: "stack_trace_aggregator".to_string(),
            version: "1.0.0".to_string(),
            description: "Java堆栈跟踪聚合器".to_string(),
            priority: 15,
        }
    }
}

impl Plugin for StackTraceAggregator {
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
        // 检查是否为堆栈跟踪行
        let trimmed = log_line.trim();
        trimmed.starts_with("at ") ||
        trimmed.starts_with("Caused by:") ||
        trimmed.starts_with("Suppressed:") ||
        trimmed.contains("Exception:") ||
        trimmed.contains("Error:")
    }
    
    fn process(&self, log_entry: &mut LogEntry) -> Result<(), PluginError> {
        if !self.can_handle(&log_entry.content) {
            return Ok(());
        }
        
        // 标记为堆栈跟踪
        log_entry.add_metadata("type".to_string(), "stack_trace".to_string());
        
        // 如果是异常开始行，标记为异常
        if log_entry.content.contains("Exception:") || log_entry.content.contains("Error:") {
            log_entry.add_metadata("exception_type".to_string(), "exception_start".to_string());
        }
        
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

/// MyBatis SQL 解析器
pub struct MyBatisParser {
    name: String,
    version: String,
    description: String,
    priority: u32,
}

impl MyBatisParser {
    pub fn new() -> Self {
        Self {
            name: "mybatis".to_string(),
            version: "1.0.0".to_string(),
            description: "MyBatis SQL日志解析器".to_string(),
            priority: 20,
        }
    }
    
    /// 格式化SQL语句
    fn format_sql(&self, content: &str) -> String {
        if let Some(sql_start) = content.find("Preparing:") {
            let sql_part = &content[sql_start + 10..].trim();
            self.format_sql_indentation(sql_part)
        } else {
            content.to_string()
        }
    }
    
    /// 格式化参数
    fn format_parameters(&self, content: &str) -> String {
        if let Some(params_start) = content.find("Parameters:") {
            let params_part = &content[params_start + 11..].trim();
            self.format_parameter_list(params_part)
        } else {
            content.to_string()
        }
    }
    
    /// SQL缩进格式化 - 转换为单行可执行SQL
    fn format_sql_indentation(&self, sql: &str) -> String {
        // 将多行SQL合并为单行，并清理多余的空格
        let cleaned_sql = sql
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<&str>>()
            .join(" ");
        
        // 进一步清理SQL，确保可执行性
        self.clean_sql_for_execution(&cleaned_sql)
    }
    
    /// 清理SQL使其可执行
    fn clean_sql_for_execution(&self, sql: &str) -> String {
        let mut cleaned = sql.to_string();
        
        // 替换参数占位符为实际值（如果有参数的话）
        // 这里我们保持?占位符，因为实际参数在Parameters行中
        cleaned = cleaned
            .replace("  ", " ")  // 替换双空格为单空格
            .replace(" ,", ",")  // 清理逗号前的空格
            .replace("( ", "(")  // 清理括号后的空格
            .replace(" )", ")")  // 清理括号前的空格
            .replace(" = ", "=") // 清理等号周围的空格
            .replace(" < ", "<") // 清理小于号周围的空格
            .replace(" > ", ">") // 清理大于号周围的空格
            .replace(" != ", "!=") // 清理不等号周围的空格
            .replace(" <= ", "<=") // 清理小于等于号周围的空格
            .replace(" >= ", ">=") // 清理大于等于号周围的空格
            .replace(" AND ", " AND ") // 保持AND关键字
            .replace(" OR ", " OR ")   // 保持OR关键字
            .replace(" IN ", " IN ")   // 保持IN关键字
            .replace(" NOT ", " NOT ") // 保持NOT关键字
            .replace(" IS ", " IS ")  // 保持IS关键字
            .replace(" NULL", " NULL") // 保持NULL关键字
            .replace(" TRUE", " TRUE") // 保持TRUE关键字
            .replace(" FALSE", " FALSE"); // 保持FALSE关键字
        
        // 确保SQL以分号结尾（如果没有的话）
        if !cleaned.trim().ends_with(';') {
            cleaned.push(';');
        }
        
        cleaned
    }
    
    /// 参数列表格式化 - 转换为单行参数列表
    fn format_parameter_list(&self, params: &str) -> String {
        // 将参数列表格式化为单行，用逗号分隔
        let cleaned_params = params
            .split(',')
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
        
        format!("Parameters: {}", cleaned_params)
    }
}

impl Plugin for MyBatisParser {
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
        log_line.contains("Preparing:") || 
        log_line.contains("Parameters:") ||
        log_line.contains("Total:")
    }
    
    fn process(&self, log_entry: &mut LogEntry) -> Result<(), PluginError> {
        if !self.can_handle(&log_entry.content) {
            return Ok(());
        }
        
        println!("🔧 [MyBatis] 处理日志条目: {}", log_entry.content.chars().take(100).collect::<String>());
        
        if log_entry.content.contains("Preparing:") {
            println!("🔧 [MyBatis] 检测到Preparing SQL语句");
            log_entry.add_metadata("sql_type".to_string(), "preparing".to_string());
            // 只提取SQL语句，不包含参数（参数会在后续的Parameters行中处理）
            let formatted_sql = self.format_sql(&log_entry.content);
            println!("🔧 [MyBatis] 格式化后的SQL: {}", formatted_sql);
            log_entry.formatted_content = Some(formatted_sql);
        } else if log_entry.content.contains("Parameters:") {
            println!("🔧 [MyBatis] 检测到Parameters参数");
            log_entry.add_metadata("sql_type".to_string(), "parameters".to_string());
            // 对于参数行，我们只标记类型，不进行格式化
            // 实际的SQL+参数合并会在后续的聚合处理中完成
            log_entry.add_metadata("format".to_string(), "mybatis".to_string());
            log_entry.mark_processed(self.name.clone());
            return Ok(());
        } else if log_entry.content.contains("Total:") {
            println!("🔧 [MyBatis] 检测到Total结果");
            log_entry.add_metadata("sql_type".to_string(), "result".to_string());
        }
        
        log_entry.add_metadata("format".to_string(), "mybatis".to_string());
        log_entry.mark_processed(self.name.clone());
        
        println!("🔧 [MyBatis] 处理完成，formatted_content: {:?}", 
                 log_entry.formatted_content.as_ref().map(|s| s.chars().take(50).collect::<String>()));
        
        Ok(())
    }
    
    fn initialize(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn cleanup(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// JSON 格式化渲染器
pub struct JsonFormatter {
    name: String,
    version: String,
    description: String,
    priority: u32,
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self {
            name: "json_formatter".to_string(),
            version: "1.0.0".to_string(),
            description: "JSON格式化和美化渲染器".to_string(),
            priority: 25,
        }
    }
}

impl Plugin for JsonFormatter {
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
        PluginType::Renderer
    }
    
    fn can_handle(&self, log_line: &str) -> bool {
        // 检查是否包含JSON内容
        log_line.contains('{') && log_line.contains('}')
    }
    
    fn process(&self, log_entry: &mut LogEntry) -> Result<(), PluginError> {
        if !self.can_handle(&log_entry.content) {
            return Ok(());
        }
        
        // 尝试格式化JSON
        if let Ok(parsed) = serde_json::from_str::<Value>(&log_entry.content) {
            if let Ok(formatted) = serde_json::to_string_pretty(&parsed) {
                log_entry.formatted_content = Some(formatted);
                log_entry.add_metadata("formatted".to_string(), "true".to_string());
            }
        }
        
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
