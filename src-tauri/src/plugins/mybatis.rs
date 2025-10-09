use crate::plugins::{LogParser, ParseRequest, ParseResult, LogLine};
use std::collections::HashMap;

pub struct MyBatisParser;

impl LogParser for MyBatisParser {
    fn name(&self) -> &str {
        "mybatis"
    }

    fn description(&self) -> &str {
        "MyBatis SQL 日志解析器"
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["log".to_string(), "txt".to_string()]
    }

    fn can_parse(&self, content: &str, _file_path: Option<&str>) -> bool {
        content.to_lowercase().contains("mybatis") ||
        content.to_lowercase().contains("preparing:") ||
        content.to_lowercase().contains("parameters:")
    }

    fn parse(&self, content: &str, _request: &ParseRequest) -> Result<ParseResult, String> {
        let lines: Vec<LogLine> = content
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let mut metadata = HashMap::new();

                // 检测 SQL 相关行
                if line.to_lowercase().contains("preparing:") {
                    metadata.insert("type".to_string(), "sql_prepare".to_string());
                } else if line.to_lowercase().contains("parameters:") {
                    metadata.insert("type".to_string(), "sql_parameters".to_string());
                } else if line.to_lowercase().contains("updates:") {
                    metadata.insert("type".to_string(), "sql_updates".to_string());
                }

                let level = if line.to_lowercase().contains("debug") {
                    Some("DEBUG".to_string())
                } else {
                    None
                };

                LogLine {
                    line_number: i + 1,
                    content: line.to_string(),
                    level,
                    timestamp: None,
                    formatted_content: Some(line.trim().to_string()),
                    metadata,
                    processed_by: vec!["mybatis_parser".to_string()],
                }
            })
            .collect();

        Ok(ParseResult {
            lines,
            total_lines: content.lines().count(),
            detected_format: Some("mybatis".to_string()),
            parsing_errors: Vec::new(),
        })
    }
}