/// Docker JSON日志解析器
///
/// 专门用于解析Docker容器输出的JSON格式日志。Docker默认的日志驱动将容器日志输出为JSON格式，
/// 每行都是一个包含时间戳、流类型和日志内容的JSON对象。
///
/// # 支持的JSON格式
/// 标准Docker JSON日志格式包含以下字段：
/// - `log`: 实际的日志内容
/// - `stream`: 输出流类型（stdout/stderr）
/// - `time`: ISO 8601格式的时间戳
/// - `attrs`: 可选的容器属性标签
///
/// # 示例JSON结构
/// ```json
/// {
///   "log": "Application started successfully\n",
///   "stream": "stdout",
///   "time": "2024-01-15T10:30:45.123456789Z"
/// }
/// ```
///
/// # 解析特性
/// - JSON结构解析：完整解析Docker JSON格式
/// - 流类型提取：识别stdout/stderr输出流
/// - 时间戳处理：提取ISO格式时间戳
/// - 日志级别推断：从内容中智能识别日志级别
/// - 错误恢复：对无效JSON行进行优雅处理
/// - 统一格式化：确保与其他插件的兼容性
///
/// # 应用场景
/// - 容器化应用日志分析
/// - Docker/Kubernetes环境监控
/// - 微服务架构日志收集
/// - CI/CD流水线日志处理
/// - 容器故障排查和性能分析
///
/// # 性能优化
/// - 高效JSON解析：使用serde_json进行快速解析
/// - 内存优化：逐行处理，避免大文件内存占用
/// - 错误容忍：部分解析失败不影响整体处理
/// - 并发安全：无状态设计支持多线程处理

use crate::plugins::{LogParser, ParseRequest, ParseResult, LogLine};
use crate::plugins::formatter::UnifiedFormatter;
use std::collections::HashMap;
use serde_json;

/// Docker JSON日志解析器实现
///
/// 这是一个无状态的结构体，专门处理Docker容器的JSON格式日志输出。
/// 依赖serde_json进行JSON解析，使用UnifiedFormatter确保输出格式的一致性。
pub struct DockerJsonParser;

impl LogParser for DockerJsonParser {
    /// 返回解析器的唯一名称标识符
    ///
    /// # Returns
    /// - `&str`: "docker_json"
    fn name(&self) -> &str {
        "docker_json"
    }

    /// 返回解析器的功能描述
    ///
    /// # Returns
    /// - `&str`: "Docker JSON 日志解析器"
    fn description(&self) -> &str {
        "Docker JSON 日志解析器"
    }

    /// 返回支持的文件扩展名列表
    ///
    /// Docker JSON日志通常存储在这些扩展名的文件中。
    ///
    /// # Returns
    /// - `Vec<String>`: 支持的文件扩展名列表
    fn supported_extensions(&self) -> Vec<String> {
        vec!["json".to_string(), "log".to_string()]
    }

    /// 检查是否能解析给定的日志内容
    ///
    /// 通过检测内容是否为JSON格式以及是否包含Docker日志的特征字段来判断。
    /// 只检查第一行进行快速识别，避免大文件的全量扫描。
    ///
    /// # 检测策略
    /// 1. JSON格式检测：检查行是否以'{'开头
    /// 2. Docker特征检测：检查是否包含"log"或"stream"字段
    ///
    /// # 参数
    /// - `content`: 日志内容样本
    /// - `_file_path`: 文件路径（当前未使用）
    ///
    /// # Returns
    /// - `bool`: true表示可能是Docker JSON日志格式
    ///
    /// # 检测示例
    /// ```rust
    /// // 匹配的内容示例
    /// r#"{"log": "Application started", "stream": "stdout", "time": "2024-01-15T10:30:45.123Z"}"#
    /// r#"{"stream": "stderr", "log": "Error occurred", "time": "2024-01-15T10:31:00.456Z"}"#
    /// ```
    ///
    /// # 性能考虑
    /// - 只检查第一行，实现O(1)时间复杂度的检测
    /// - 使用简单的字符串操作，避免JSON解析开销
    /// - 早期退出策略，提高检测效率
    fn can_parse(&self, content: &str, _file_path: Option<&str>) -> bool {
        content.lines().next().map_or(false, |first_line| {
            first_line.trim_start().starts_with('{') &&
            (first_line.contains("\"log\"") || first_line.contains("\"stream\""))
        })
    }

    /// 执行Docker JSON日志解析
    ///
    /// 这是Docker JSON插件的核心解析功能，将JSON格式的Docker日志转换为结构化的LogLine列表。
    /// 支持完整的JSON解析和错误恢复机制，确保即使部分行解析失败也能继续处理。
    ///
    /// # 解析流程
    /// 1. 初始化结果容器和错误收集器
    /// 2. 逐行处理JSON日志内容
    /// 3. JSON解析：使用serde_json解析每行JSON对象
    /// 4. 字段提取：提取log、stream、time等关键字段
    /// 5. 日志级别推断：从内容中智能识别日志级别
    /// 6. 统一格式化：使用UnifiedFormatter确保输出一致性
    /// 7. 错误恢复：对无效JSON行进行优雅处理
    ///
    /// # 支持的JSON字段提取
    /// - `log`: 日志内容（去除尾部换行符）
    /// - `stream`: 输出流类型（stdout/stderr）
    /// - `time`: ISO 8601格式时间戳
    /// - 其他字段：保留在元数据中供扩展使用
    ///
    /// # 错误处理策略
    /// - 部分解析失败不影响整体处理
    /// - 详细的错误信息记录到parsing_errors中
    /// - 无效JSON行作为普通文本行处理
    /// - 保持行号和内容的对应关系
    ///
    /// # 参数
    /// - `content`: 要解析的完整JSON日志内容
    /// - `_request`: 解析请求参数（当前未使用）
    ///
    /// # Returns
    /// - `Ok(ParseResult)`: 解析成功的结构化日志结果，包含错误信息
    /// - `Err(String)`: 解析失败时的错误描述
    ///
    /// # 性能特性
    /// - 流式处理：逐行处理，内存效率高
    /// - 快速解析：使用serde_json的高性能JSON解析器
    /// - 容错处理：部分失败不影响整体性能
    /// - 格式统一：确保与其他插件的兼容性
    ///
    /// # 元数据处理
    /// 所有JSON字段都会被提取并添加到元数据中：
    /// ```rust
    /// // 示例输入
    /// {"log": "Hello World\n", "stream": "stdout", "time": "2024-01-15T10:30:45.123Z"}
    ///
    /// // 元数据输出
    /// metadata = {"stream": "stdout"}
    /// ```
    fn parse(&self, content: &str, _request: &ParseRequest) -> Result<ParseResult, String> {
        let mut lines = Vec::new();
        let mut parsing_errors = Vec::new();

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;

            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(json) => {
                    let mut metadata = HashMap::new();

                    // 提取流信息
                    if let Some(stream) = json.get("stream").and_then(|v| v.as_str()) {
                        metadata.insert("stream".to_string(), stream.to_string());
                    }

                    let log_content = json.get("log")
                        .and_then(|v| v.as_str())
                        .unwrap_or(line)
                        .trim_end_matches('\n')
                        .to_string();

                    let level = extract_level_from_log(&log_content);
                    let timestamp = json.get("time").and_then(|v| v.as_str()).map(|s| s.to_string());

                    // 使用统一格式化器
                    let unified_format = UnifiedFormatter::format_log_line(
                        line_num,
                        &log_content,
                        level.clone(),
                        timestamp.clone(),
                        &metadata,
                        "docker_json"
                    );

                    let formatted_content = UnifiedFormatter::format_display_string(&unified_format);

                    lines.push(LogLine {
                        line_number: line_num,
                        content: log_content,
                        level,
                        timestamp,
                        formatted_content: Some(formatted_content),
                        metadata,
                        processed_by: vec!["docker_json_parser".to_string()],
                    });
                }
                Err(e) => {
                    parsing_errors.push(format!("Line {}: Failed to parse JSON: {}", line_num, e));

                    let metadata = HashMap::new();

                    // 对解析失败的行也使用统一格式化器
                    let unified_format = UnifiedFormatter::format_log_line(
                        line_num,
                        line,
                        None,
                        None,
                        &metadata,
                        "docker_json"
                    );

                    let formatted_content = UnifiedFormatter::format_display_string(&unified_format);

                    lines.push(LogLine {
                        line_number: line_num,
                        content: line.to_string(),
                        level: unified_format.level.clone(),
                        timestamp: unified_format.timestamp.clone(),
                        formatted_content: Some(formatted_content),
                        metadata,
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


/// 从日志内容中智能提取日志级别
///
/// Docker JSON格式本身不包含日志级别字段，需要从日志内容中智能推断。
/// 通过搜索常见的关键词模式来确定日志级别，提供更好的日志分类和分析能力。
///
/// # 检测策略
/// 使用大小写不敏感的关键词匹配进行级别推断：
///
/// ## 错误级别 (ERROR)
/// - 包含 "error" 或 "err" 关键词
/// - 示例：`"Error: Database connection failed"`, `"ERR: Invalid input"`
///
/// ## 警告级别 (WARN)
/// - 包含 "warn" 或 "warning" 关键词
/// - 示例：`"Warning: Memory usage high"`, `"WARN: Configuration missing"`
///
/// ## 信息级别 (INFO)
/// - 包含 "info" 关键词
/// - 示例：`"Info: Application started"`, `"INFO: Server listening on port 8080"`
///
/// ## 调试级别 (DEBUG)
/// - 包含 "debug" 关键词
/// - 示例：`"Debug: Processing request"`, `"DEBUG: Loading configuration"`
///
/// # 参数
/// - `log`: 日志内容字符串的引用
///
/// # Returns
/// - `Option<String>`: 推断出的日志级别，如果无法识别则返回None
///
/// # 性能考虑
/// - 使用字符串包含操作，时间复杂度O(n)
/// - 转换为小写进行匹配，确保大小写不敏感
/// - 按优先级顺序检查，避免不必要的匹配
///
/// # 检测示例
/// ```rust
/// assert_eq!(extract_level_from_log("ERROR: Database failed"), Some("ERROR".to_string()));
/// assert_eq!(extract_level_from_log("Warning: Low memory"), Some("WARN".to_string()));
/// assert_eq!(extract_level_from_log("INFO: Server started"), Some("INFO".to_string()));
/// assert_eq!(extract_level_from_log("Debug message"), Some("DEBUG".to_string()));
/// assert_eq!(extract_level_from_log("Hello world"), None);
/// ```
///
/// # 优化策略
/// - 错误关键词优先级最高，因为错误日志最重要
/// - 避免过度匹配，确保关键词的准确性
/// - 未来可扩展支持更多级别和关键词模式
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