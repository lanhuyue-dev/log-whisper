use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 统一的日志输出格式标准
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedLogFormat {
    /// 时间戳（ISO 8601格式，简化到秒）
    pub timestamp: Option<String>,
    /// 日志级别
    pub level: Option<String>,
    /// 线程/进程信息
    pub thread: Option<String>,
    /// 日志来源/记录器名称
    pub source: Option<String>,
    /// 主要日志消息
    pub message: String,
    /// 附加元数据（如流信息、SQL类型等）
    pub metadata: HashMap<String, String>,
    /// 原始内容（用于搜索和参考）
    pub raw_content: String,
}

/// 统一日志格式化器
pub struct UnifiedFormatter;

impl UnifiedFormatter {
    /// 将LogLine转换为统一格式
    pub fn format_log_line(
        line_number: usize,
        content: &str,
        level: Option<String>,
        timestamp: Option<String>,
        metadata: &HashMap<String, String>,
        plugin_name: &str,
    ) -> UnifiedLogFormat {
        let mut unified_metadata = metadata.clone();

        // 添加处理插件信息
        unified_metadata.insert("processed_by".to_string(), plugin_name.to_string());
        unified_metadata.insert("line_number".to_string(), line_number.to_string());

        UnifiedLogFormat {
            timestamp: Self::normalize_timestamp(timestamp),
            level: Self::normalize_level(level),
            thread: metadata.get("thread").cloned(),
            source: metadata.get("logger").or_else(|| metadata.get("source")).cloned(),
            message: Self::extract_message(content, metadata),
            metadata: unified_metadata,
            raw_content: content.to_string(),
        }
    }

    /// 生成统一的格式化显示字符串
    pub fn format_display_string(format: &UnifiedLogFormat) -> String {
        let mut parts = Vec::new();

        // 时间戳（简化格式）
        if let Some(ts) = &format.timestamp {
            parts.push(ts.clone());
        }

        // 日志级别
        if let Some(level) = &format.level {
            parts.push(format!("[{}]", level));
        }

        // 线程信息（如果有且有意义）
        if let Some(thread) = &format.thread {
            if thread.len() <= 20 { // 限制线程名长度
                parts.push(format!("({})", thread));
            }
        }

        // 特殊标记（基于元数据）
        Self::add_metadata_tags(&mut parts, &format.metadata);

        // 主要消息
        parts.push(format.message.clone());

        parts.join(" ")
    }

    /// 标准化时间戳格式
    fn normalize_timestamp(timestamp: Option<String>) -> Option<String> {
        match timestamp {
            Some(ts) => {
                let trimmed = ts.trim();

                // 处理多种时间戳格式
                let normalized = if trimmed.contains('T') {
                    // ISO 8601 格式: 2024-09-30T08:00:07.890Z 或 2024-09-30T08:00:07.123456789Z
                    let without_ms = if trimmed.contains('.') {
                        trimmed.split('.').next().unwrap_or(trimmed)
                    } else {
                        trimmed
                    };

                    // 移除Z后缀如果有
                    without_ms.trim_end_matches('Z').to_string()
                } else if trimmed.contains('-') && trimmed.contains(':') {
                    // 标准格式: 2024-09-30 08:00:07,890 或 2024-09-30 08:00:07.890
                    let without_ms = if trimmed.contains('.') {
                        trimmed.split('.').next().unwrap_or(trimmed)
                    } else if trimmed.contains(',') {
                        trimmed.split(',').next().unwrap_or(trimmed)
                    } else {
                        trimmed
                    };
                    without_ms.to_string()
                } else if trimmed.contains('/') {
                    // 美式格式: 09/30/2024 08:00:07
                    trimmed.to_string()
                } else {
                    // 其他格式，直接返回
                    trimmed.to_string()
                };

                // 验证时间戳格式是否合理
                if normalized.len() >= 16 { // 至少包含 YYYY-MM-DD HH:MM
                    Some(normalized)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// 标准化日志级别
    fn normalize_level(level: Option<String>) -> Option<String> {
        match level {
            Some(l) => {
                let upper = l.to_uppercase();
                match upper.as_str() {
                    "ERROR" | "ERR" | "FATAL" | "SEVERE" => Some("ERROR".to_string()),
                    "WARN" | "WARNING" | "ALERT" => Some("WARN".to_string()),
                    "INFO" | "INFORMATION" | "NOTE" => Some("INFO".to_string()),
                    "DEBUG" | "TRACE" | "VERBOSE" => Some("DEBUG".to_string()),
                    _ => Some(upper),
                }
            }
            None => None,
        }
    }

    /// 提取主要消息内容
    fn extract_message(content: &str, metadata: &HashMap<String, String>) -> String {
        let mut cleaned = content.trim();

        // 移除常见错误前缀
        let error_prefixes = [
            "ERROR:", "Error:", "WARN:", "Warning:", "WARN", "INFO:", "Info:",
            "DEBUG:", "Debug:", "TRACE:", "Trace:", "FATAL:", "Fatal:",
            "SEVERE:", "Severe:", "CRITICAL:", "Critical:"
        ];

        for prefix in &error_prefixes {
            if cleaned.to_lowercase().starts_with(&prefix.to_lowercase()) {
                cleaned = &cleaned[prefix.len()..].trim_start();
                break;
            }
        }

        // 根据不同的日志类型提取主要消息
        match metadata.get("type").map(|s| s.as_str()) {
            Some("sql_prepare") => {
                // SQL准备语句，提取SQL部分
                if let Some(start) = cleaned.find("Preparing:") {
                    &cleaned[start..]
                } else {
                    cleaned
                }
            }
            Some("sql_parameters") => {
                // SQL参数，提取参数部分
                if let Some(start) = cleaned.find("Parameters:") {
                    &cleaned[start..]
                } else {
                    cleaned
                }
            }
            Some("stacktrace") => {
                // 堆栈跟踪，保持原样
                cleaned
            }
            _ => {
                // 默认情况，移除时间戳前缀（如果有）
                let without_timestamp = Self::remove_leading_timestamp(cleaned);
                without_timestamp
            }
        }.trim().to_string()
    }

    /// 移除行首的时间戳
    fn remove_leading_timestamp(content: &str) -> &str {
        // 尝试匹配时间戳模式
        if content.len() < 19 {
            return content;
        }

        // 检查常见时间戳格式的开始模式
        let has_timestamp_start =
            // YYYY-MM-DD 或 YYYY/MM/DD 或 DD.MM.YYYY
            (content.chars().nth(4) == Some('-') || content.chars().nth(4) == Some('/') || content.chars().nth(2) == Some('.')) &&
            // 后面跟时间部分
            (content.contains('T') || (content.len() >= 11 && content.chars().nth(10) == Some(' ') || content.chars().nth(13) == Some(' ')));

        if !has_timestamp_start {
            return content;
        }

        // 尝试找到时间戳结束后的位置
        if let Some(time_end) = Self::find_timestamp_end(content) {
            if time_end < content.len() {
                return &content[time_end..].trim_start();
            }
        }

        content
    }

    /// 查找时间戳结束位置
    fn find_timestamp_end(content: &str) -> Option<usize> {
        // 查找时间部分的结束位置
        let space_indices: Vec<usize> = content
            .char_indices()
            .filter(|(_, c)| *c == ' ')
            .map(|(i, _)| i)
            .collect();

        match space_indices.len() {
            0 => None,
            1 => {
                // 只有一个空格，时间戳结束
                Some(space_indices[0] + 1)
            }
            2 => {
                // 两个空格，可能是 "YYYY-MM-DD HH:MM:SS message"
                Some(space_indices[1] + 1)
            }
            3 => {
                // 三个空格，检查第三个空格后的内容是否是日志级别
                let after_third_space = &content[space_indices[2] + 1..];
                if after_third_space.len() >= 4 {
                    let first_word = after_third_space.split(' ').next().unwrap_or("");
                    if first_word.chars().all(|c| c.is_ascii_uppercase()) &&
                       (first_word == "ERROR" || first_word == "WARN" || first_word == "INFO" ||
                        first_word == "DEBUG" || first_word == "TRACE" || first_word == "FATAL") {
                        return Some(space_indices[2] + 1 + first_word.len() + 1);
                    }
                }
                Some(space_indices[2] + 1)
            }
            _ => {
                // 更多空格，找到时间部分后的第一个非时间内容
                for i in 2..space_indices.len() {
                    let potential_end = space_indices[i] + 1;
                    let after_space = &content[potential_end..];

                    // 检查是否是日志级别
                    if after_space.len() >= 4 {
                        let first_word = after_space.split(' ').next().unwrap_or("");
                        if first_word.chars().all(|c| c.is_ascii_uppercase()) &&
                           (first_word == "ERROR" || first_word == "WARN" || first_word == "INFO" ||
                            first_word == "DEBUG" || first_word == "TRACE" || first_word == "FATAL") {
                            return Some(potential_end + first_word.len() + 1);
                        }
                    }

                    // 如果不是时间格式，就返回这个位置
                    if !after_space.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                        return Some(potential_end);
                    }
                }
                None
            }
        }
    }

    /// 添加基于元数据的特殊标签
    fn add_metadata_tags(parts: &mut Vec<String>, metadata: &HashMap<String, String>) {
        // 流信息标签
        if let Some(stream) = metadata.get("stream") {
            parts.push(format!("[{}]", stream.to_uppercase()));
        }

        // 类型标签
        if let Some(log_type) = metadata.get("type") {
            match log_type.as_str() {
                "sql_prepare" => parts.push("[SQL]".to_string()),
                "sql_parameters" => parts.push("[PARAMS]".to_string()),
                "sql_updates" => parts.push("[UPDATE]".to_string()),
                "stacktrace" => parts.push("[STACK]".to_string()),
                _ => {}
            }
        }
    }

    /// 创建颜色样式（用于终端输出）
    #[allow(dead_code)]
    pub fn get_color_code(level: &Option<String>) -> &'static str {
        match level.as_deref() {
            Some("ERROR") => "\x1b[31m",    // 红色
            Some("WARN") => "\x1b[33m",     // 黄色
            Some("INFO") => "\x1b[32m",     // 绿色
            Some("DEBUG") => "\x1b[36m",    // 青色
            _ => "\x1b[37m",                // 白色
        }
    }

    /// 重置颜色样式
    #[allow(dead_code)]
    pub fn reset_color() -> &'static str {
        "\x1b[0m"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_timestamp() {
        assert_eq!(
            UnifiedFormatter::normalize_timestamp(Some("2024-09-30T08:00:07.890Z".to_string())),
            Some("2024-09-30T08:00:07".to_string())
        );
        assert_eq!(
            UnifiedFormatter::normalize_timestamp(Some("2024-09-30 08:00:07,890".to_string())),
            Some("2024-09-30 08:00:07".to_string())
        );
    }

    #[test]
    fn test_normalize_level() {
        assert_eq!(
            UnifiedFormatter::normalize_level(Some("error".to_string())),
            Some("ERROR".to_string())
        );
        assert_eq!(
            UnifiedFormatter::normalize_level(Some("warning".to_string())),
            Some("WARN".to_string())
        );
    }

    #[test]
    fn test_format_display_string() {
        let mut metadata = HashMap::new();
        metadata.insert("stream".to_string(), "stderr".to_string());

        let format = UnifiedLogFormat {
            timestamp: Some("2024-09-30T08:00:07".to_string()),
            level: Some("ERROR".to_string()),
            thread: None,
            source: None,
            message: "Database connection failed".to_string(),
            metadata,
            raw_content: "Error: Database connection failed".to_string(),
        };

        let result = UnifiedFormatter::format_display_string(&format);
        assert!(result.contains("2024-09-30T08:00:07"));
        assert!(result.contains("[ERROR]"));
        assert!(result.contains("[STDERR]"));
        assert!(result.contains("Database connection failed"));
    }

    #[test]
    fn test_extract_message_with_error_prefix() {
        let metadata = HashMap::new();

        // Test removing Error: prefix
        let result = UnifiedFormatter::extract_message(
            "Error: Failed to connect to Redis Connection timeout",
            &metadata
        );
        assert_eq!(result, "Failed to connect to Redis Connection timeout");

        // Test removing ERROR prefix
        let result = UnifiedFormatter::extract_message(
            "ERROR: Database connection lost",
            &metadata
        );
        assert_eq!(result, "Database connection lost");
    }

    #[test]
    fn test_normalize_timestamp_various_formats() {
        // Test ISO 8601 with nanoseconds and Z
        assert_eq!(
            UnifiedFormatter::normalize_timestamp(Some("2024-09-30T08:00:07.123456789Z".to_string())),
            Some("2024-09-30T08:00:07".to_string())
        );

        // Test standard format with comma milliseconds
        assert_eq!(
            UnifiedFormatter::normalize_timestamp(Some("2024-09-30 08:00:07,890".to_string())),
            Some("2024-09-30 08:00:07".to_string())
        );

        // Test US format
        assert_eq!(
            UnifiedFormatter::normalize_timestamp(Some("09/30/2024 08:00:07.123".to_string())),
            Some("09/30/2024 08:00:07.123".to_string())
        );
    }

    #[test]
    fn test_remove_leading_timestamp() {
        // Test removing timestamp from message content
        let content = "2024-09-30 08:00:07 ERROR Failed to connect to Redis";
        let result = UnifiedFormatter::remove_leading_timestamp(content);
        assert_eq!(result, "Failed to connect to Redis");

        // Test content without timestamp
        let content = "Simple log message without timestamp";
        let result = UnifiedFormatter::remove_leading_timestamp(content);
        assert_eq!(result, "Simple log message without timestamp");
    }
}