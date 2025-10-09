use crate::plugins::{LogParser, ParseRequest, ParseResult, LogLine};
use std::collections::HashMap;

pub struct RawParser;

impl LogParser for RawParser {
    fn name(&self) -> &str {
        "raw"
    }

    fn description(&self) -> &str {
        "原始文本日志解析器"
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["log".to_string(), "txt".to_string(), "out".to_string()]
    }

    fn can_parse(&self, _content: &str, _file_path: Option<&str>) -> bool {
        true // Raw parser can parse any text
    }

    fn parse(&self, content: &str, _request: &ParseRequest) -> Result<ParseResult, String> {
        let lines: Vec<LogLine> = content
            .lines()
            .enumerate()
            .map(|(i, line)| {
                LogLine {
                    line_number: i + 1,
                    content: line.to_string(),
                    level: None,
                    timestamp: None,
                    formatted_content: Some(line.trim().to_string()),
                    metadata: HashMap::new(),
                    processed_by: vec!["raw_parser".to_string()],
                }
            })
            .collect();

        Ok(ParseResult {
            lines,
            total_lines: content.lines().count(),
            detected_format: Some("raw".to_string()),
            parsing_errors: Vec::new(),
        })
    }
}