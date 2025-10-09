use crate::plugins::{LogParser, ParseRequest, ParseResult, LogLine};
use std::collections::HashMap;
use regex::Regex;

pub struct SpringBootParser;

impl LogParser for SpringBootParser {
    fn name(&self) -> &str {
        "springboot"
    }

    fn description(&self) -> &str {
        "Spring Boot 应用日志解析器"
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["log".to_string(), "txt".to_string()]
    }

    fn can_parse(&self, content: &str, _file_path: Option<&str>) -> bool {
        content.to_lowercase().contains("spring") ||
        content.to_lowercase().contains("application.start") ||
        content.lines().any(|line| {
            regex::Regex::new(r"^\d{4}-\d{2}-\d{2}").unwrap().is_match(line)
        })
    }

    fn parse(&self, content: &str, _request: &ParseRequest) -> Result<ParseResult, String> {
        let mut lines = Vec::new();
        let mut parsing_errors = Vec::new();

        // Spring Boot 日志格式正则表达式
        let log_pattern = Regex::new(r"^(\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2}[.,]\d{3})\s+\[([^\]]+)\]\s+(\w+)\s+([^-]+)\s+-\s+(.*)$").unwrap();

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;
            let mut metadata = HashMap::new();

            if let Some(captures) = log_pattern.captures(line) {
                let timestamp = captures.get(1).map(|m| m.as_str().to_string());
                let thread = captures.get(2).map(|m| m.as_str().to_string());
                let level = captures.get(3).map(|m| m.as_str().to_uppercase().to_string());
                let logger = captures.get(4).map(|m| m.as_str().to_string());
                let message = captures.get(5).map(|m| m.as_str().to_string());

                if let Some(t) = &thread {
                    metadata.insert("thread".to_string(), t.clone());
                }
                if let Some(l) = &logger {
                    metadata.insert("logger".to_string(), l.clone());
                }

                lines.push(LogLine {
                    line_number: line_num,
                    content: message.unwrap_or(line.to_string()).to_string(),
                    level,
                    timestamp,
                    formatted_content: Some(line.trim().to_string()),
                    metadata,
                    processed_by: vec!["springboot_parser".to_string()],
                });
            } else {
                // 不匹配标准格式的行，可能是异常堆栈的一部分
                metadata.insert("type".to_string(), "stacktrace".to_string());

                lines.push(LogLine {
                    line_number: line_num,
                    content: line.to_string(),
                    level: None,
                    timestamp: None,
                    formatted_content: Some(line.trim().to_string()),
                    metadata,
                    processed_by: vec!["springboot_parser".to_string()],
                });
            }
        }

        Ok(ParseResult {
            lines,
            total_lines: content.lines().count(),
            detected_format: Some("springboot".to_string()),
            parsing_errors,
        })
    }
}