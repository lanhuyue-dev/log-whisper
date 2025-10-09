use crate::plugins::{LogParser, ParseRequest, ParseResult, LogLine};
use std::collections::HashMap;
use serde_json;

pub struct DockerJsonParser;

impl LogParser for DockerJsonParser {
    fn name(&self) -> &str {
        "docker_json"
    }

    fn description(&self) -> &str {
        "Docker JSON 日志解析器"
    }

    fn supported_extensions(&self) -> Vec<String> {
        vec!["json".to_string(), "log".to_string()]
    }

    fn can_parse(&self, content: &str, _file_path: Option<&str>) -> bool {
        content.lines().next().map_or(false, |first_line| {
            first_line.trim_start().starts_with('{') &&
            (first_line.contains("\"log\"") || first_line.contains("\"stream\""))
        })
    }

    fn parse(&self, content: &str, _request: &ParseRequest) -> Result<ParseResult, String> {
        let mut lines = Vec::new();
        let mut parsing_errors = Vec::new();

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;

            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(json) => {
                    let mut metadata = HashMap::new();

                    if let Some(log) = json.get("log").and_then(|v| v.as_str()) {
                        metadata.insert("log".to_string(), log.to_string());
                    }

                    if let Some(stream) = json.get("stream").and_then(|v| v.as_str()) {
                        metadata.insert("stream".to_string(), stream.to_string());
                    }

                    if let Some(timestamp) = json.get("time").and_then(|v| v.as_str()) {
                        metadata.insert("timestamp".to_string(), timestamp.to_string());
                    }

                    let log_content = json.get("log")
                        .and_then(|v| v.as_str())
                        .unwrap_or(line)
                        .trim_end_matches('\n')
                        .to_string();

                    let level = extract_level_from_log(&log_content);

                    lines.push(LogLine {
                        line_number: line_num,
                        content: log_content,
                        level,
                        timestamp: json.get("time").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        formatted_content: Some(line.trim().to_string()),
                        metadata,
                        processed_by: vec!["docker_json_parser".to_string()],
                    });
                }
                Err(e) => {
                    parsing_errors.push(format!("Line {}: Failed to parse JSON: {}", line_num, e));
                    // 仍然添加原始行
                    lines.push(LogLine {
                        line_number: line_num,
                        content: line.to_string(),
                        level: None,
                        timestamp: None,
                        formatted_content: Some(line.trim().to_string()),
                        metadata: HashMap::new(),
                        processed_by: vec!["docker_json_parser".to_string()],
                    });
                }
            }
        }

        Ok(ParseResult {
            lines,
            total_lines: content.lines().count(),
            detected_format: Some("docker_json".to_string()),
            parsing_errors,
        })
    }
}

fn extract_level_from_log(log: &str) -> Option<String> {
    let log_lower = log.to_lowercase();
    if log_lower.contains("error") || log_lower.contains("err") {
        Some("ERROR".to_string())
    } else if log_lower.contains("warn") || log_lower.contains("warning") {
        Some("WARN".to_string())
    } else if log_lower.contains("info") {
        Some("INFO".to_string())
    } else if log_lower.contains("debug") {
        Some("DEBUG".to_string())
    } else {
        None
    }
}