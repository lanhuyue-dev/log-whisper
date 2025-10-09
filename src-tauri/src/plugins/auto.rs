use crate::plugins::{LogParser, ParseRequest, ParseResult, LogLine};
use std::collections::HashMap;

pub struct AutoParser;

impl LogParser for AutoParser {
    fn name(&self) -> &str {
        "auto"
    }

    fn description(&self) -> &str {
        "自动检测日志格式"
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["*".to_string()]
    }

    fn can_parse(&self, _content: &str, _file_path: Option<&str>) -> bool {
        true // Auto parser can always try
    }

    fn parse(&self, content: &str, _request: &ParseRequest) -> Result<ParseResult, String> {
        let lines: Vec<LogLine> = content
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let mut metadata = HashMap::new();
                let (level, timestamp) = extract_level_and_timestamp(line);

                if let Some(l) = &level {
                    metadata.insert("level".to_string(), l.clone());
                }
                if let Some(t) = &timestamp {
                    metadata.insert("timestamp".to_string(), t.clone());
                }

                LogLine {
                    line_number: i + 1,
                    content: line.to_string(),
                    level,
                    timestamp,
                    formatted_content: Some(line.trim().to_string()),
                    metadata,
                    processed_by: vec!["auto_parser".to_string()],
                }
            })
            .collect();

        Ok(ParseResult {
            lines,
            total_lines: content.lines().count(),
            detected_format: Some("auto".to_string()),
            parsing_errors: Vec::new(),
        })
    }
}

fn extract_level_and_timestamp(line: &str) -> (Option<String>, Option<String>) {
    let line_lower = line.to_lowercase();

    // 提取日志级别
    let level = if line_lower.contains("error") || line_lower.contains("err") {
        Some("ERROR".to_string())
    } else if line_lower.contains("warn") || line_lower.contains("warning") {
        Some("WARN".to_string())
    } else if line_lower.contains("info") {
        Some("INFO".to_string())
    } else if line_lower.contains("debug") {
        Some("DEBUG".to_string())
    } else if line_lower.contains("trace") {
        Some("TRACE".to_string())
    } else {
        None
    };

    // 简单的时间戳提取
    let timestamp = if line.len() > 20 {
        Some(line[..20].to_string())
    } else {
        None
    };

    (level, timestamp)
}