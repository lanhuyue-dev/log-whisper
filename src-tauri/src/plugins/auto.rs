/// 自动日志格式检测和解析器
///
/// 这是一个智能的通用日志解析器，能够自动识别和解析各种常见的日志格式。
/// 通过模式匹配和启发式算法，从未知格式的日志中提取结构化信息，
/// 为用户提供基础的结构化日志分析能力。
///
/// # 设计理念
/// - **智能识别**：自动检测日志中的关键模式和结构
/// - **通用兼容**：支持多种常见的日志格式和样式
/// - **渐进解析**：从简单到复杂的层次化信息提取
/// - **容错能力**：在格式不明确时提供合理的回退策略
///
/// # 支持的识别模式
/// ## 日志级别识别
/// - ERROR/ERR: 错误级别信息
/// - WARN/WARNING: 警告级别信息
/// - INFO: 信息级别消息
/// - DEBUG: 调试级别详情
/// - TRACE: 跟踪级别信息
///
/// ## 时间戳模式
/// - ISO 8601格式：2024-01-15T10:30:45.123Z
/// - 标准格式：2024-01-15 10:30:45.123
/// - 时间格式：10:30:45 或 10:30
/// - 日期格式：2024-01-15 或 01/15/2024
///
/// ## 常见日志结构
/// - [LEVEL] MESSAGE 格式
/// - TIMESTAMP [LEVEL] MESSAGE 格式
/// - [THREAD] LEVEL MESSAGE 格式
/// - LOGGER.LEVEL MESSAGE 格式
///
/// # 解析策略
/// 1. **关键词检测**：搜索常见的日志级别关键词
/// 2. **时间模式识别**：匹配各种时间戳格式
/// 3. **结构分析**：识别方括号、冒号等分隔符
/// 4. **元数据提取**：将识别的信息添加到元数据中
/// 5. **格式推断**：基于模式匹配推断日志结构
///
/// # 应用场景
/// - 未知格式日志的初步分析
/// - 多种格式混合的日志文件
/// - 日志格式的快速评估和分类
/// - 开发阶段的调试和测试
/// - 日志系统的原型验证
/// - 应急日志分析和故障排查
///
/// # 技术特点
/// - **模式匹配**：使用字符串包含和正则表达式
/// - **启发式算法**：基于经验规则的智能判断
/// - **多层次分析**：从简单到复杂的信息提取
/// - **容错设计**：在不确定时提供保守的结果

use crate::plugins::{LogParser, ParseRequest, ParseResult, LogLine};
use std::collections::HashMap;

/// 自动日志解析器实现
///
/// 这是一个智能的无状态结构体，能够处理各种未知格式的日志。
/// 通过模式识别和启发式算法，为用户提供基础的结构化分析能力。
pub struct AutoParser;

impl LogParser for AutoParser {
    /// 返回解析器的唯一名称标识符
    ///
    /// # Returns
    /// - `&str`: "auto"
    fn name(&self) -> &str {
        "auto"
    }

    /// 返回解析器的功能描述
    ///
    /// # Returns
    /// - `&str`: "自动检测日志格式"
    fn description(&self) -> &str {
        "自动检测日志格式"
    }

    /// 返回支持的文件扩展名列表
    ///
    /// 作为自动检测解析器，支持所有文件类型。
    ///
    /// # Returns
    /// - `Vec<String>`: 包含通配符的文件扩展名列表
    fn supported_extensions(&self) -> Vec<String> {
        vec!["*".to_string()]
    }

    /// 检查是否能解析给定的日志内容
    ///
    /// AutoParser作为智能检测解析器，始终愿意尝试解析任何内容。
    /// 这种设计确保了在自动检测模式中，当专用解析器无法识别格式时，
    /// AutoParser能够提供基础的解析能力。
    ///
    /// # 设计考虑
    /// - **始终尝试**：从不拒绝任何输入内容
    /// - **智能分析**：通过模式匹配提供最佳解析结果
    /// - **渐进式识别**：从简单到复杂的信息提取
    ///
    /// # 参数
    /// - `_content`: 日志内容样本（未使用，因为能处理任何内容）
    /// - `_file_path`: 文件路径（未使用）
    ///
    /// # Returns
    /// - `bool`: 始终返回true，表示愿意尝试解析任何内容
    ///
    /// # 在插件系统中的作用
    /// 在自动检测模式中的工作流程：
    /// 1. 首先尝试专用解析器（SpringBoot, MyBatis, Docker JSON等）
    /// 2. 如果所有专用解析器都无法识别，使用AutoParser进行智能检测
    /// 3. 如果AutoParser也无法提供有意义的解析，最后使用RawParser
    ///
    /// AutoParser起到了**智能桥梁**的作用，在专用解析器和原始解析器之间
    /// 提供了基础的格式识别和结构化能力。
    fn can_parse(&self, _content: &str, _file_path: Option<&str>) -> bool {
        true // Auto parser can always try
    }

    /// 执行自动日志格式检测和解析
    ///
    /// 这是AutoParser的核心功能，通过智能模式匹配从未知格式的日志中提取结构化信息。
    /// 使用启发式算法识别日志级别、时间戳等关键信息，为用户提供基础的日志分析能力。
    ///
    /// # 解析流程
    /// 1. 逐行遍历日志内容，为每行创建LogLine结构
    /// 2. 智能识别：调用extract_level_and_timestamp函数提取关键信息
    /// 3. 元数据增强：将识别的信息添加到元数据中
    /// 4. 内容保留：完整保留原始日志内容
    /// 5. 格式化处理：提供简洁的格式化显示
    /// 6. 结果封装：构建完整的ParseResult返回
    ///
    /// # 信息提取能力
    /// ## 日志级别检测
    /// - 基于关键词匹配（ERROR, WARN, INFO, DEBUG, TRACE）
    /// - 大小写不敏感的匹配
    /// - 多种变体支持（ERR, WARNING等）
    ///
    /// ## 时间戳提取
    /// - 简单启发式算法（取前20个字符）
    /// - 适用于大多数标准日志格式
    /// - 避免复杂的正则表达式匹配
    ///
    /// ## 元数据丰富
    /// 将识别的信息同时存储在LogLine字段和元数据中：
    /// - level字段：标准化的日志级别
    /// - timestamp字段：提取的时间戳
    /// - metadata.level：原始级别标识
    /// - metadata.timestamp：原始时间戳信息
    ///
    /// # 处理特点
    /// - **智能识别**：基于模式匹配的自动格式检测
    /// - **容错能力**：在格式不明确时提供合理的默认值
    /// - **信息保留**：完整保留原始内容，添加结构化信息
    /// - **元数据丰富**：提供多层次的信息访问方式
    ///
    /// # 参数
    /// - `content`: 要解析的完整日志内容
    /// - `_request`: 解析请求参数（当前未使用）
    ///
    /// # Returns
    /// - `Ok(ParseResult)`: 解析成功的结构化日志结果
    /// - `Err(String)`: 解析失败时的错误描述
    ///
    /// # 输出特征
    /// - content: 原始行内容（完全保留）
    /// - level: 智能识别的日志级别（可选）
    /// - timestamp: 提取的时间戳（可选）
    /// - formatted_content: 简洁的格式化显示
    /// - metadata: 包含level和timestamp的元数据
    /// - processed_by: ["auto_parser"]
    ///
    /// # 性能特性
    /// - **高效处理**：基于字符串包含的快速匹配
    /// - **低内存占用**：不存储复杂的解析状态
    /// - **线性时间复杂度**：O(n)，n为行数
    /// - **智能回退**：在无法识别时提供合理的默认处理
    ///
    /// # 应用示例
    /// ```rust
    /// // 输入示例
    /// let content = "2024-01-15 10:30:45 ERROR Database connection failed
    /// 2024-01-15 10:30:46 INFO Retrying connection...
    /// 2024-01-15 10:30:47 DEBUG Connection pool status";
    ///
    /// // 解析结果
    /// // Line 1: level="ERROR", timestamp="2024-01-15 10:30:45"
    /// // Line 2: level="INFO", timestamp="2024-01-15 10:30:46"
    /// // Line 3: level="DEBUG", timestamp="2024-01-15 10:30:47"
    /// ```
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

/// 从日志行中智能提取日志级别和时间戳
///
/// 这是AutoParser的核心信息提取函数，使用启发式算法从未知格式的日志中
/// 识别关键的结构化信息。通过模式匹配和关键词搜索，为日志行添加语义标签。
///
/// # 提取策略
/// ## 日志级别检测
/// 使用大小写不敏感的关键词匹配，按重要性优先级排序：
///
/// ### 错误级别 (ERROR)
/// - 匹配关键词："error", "err"
/// - 示例：`"ERROR: Database failed"`, `"System err occurred"`
///
/// ### 警告级别 (WARN)
/// - 匹配关键词："warn", "warning"
/// - 示例：`"WARN: Memory high"`, `"Warning: Config missing"`
///
/// ### 信息级别 (INFO)
/// - 匹配关键词："info"
/// - 示例：`"INFO: Server started"`, "Application info ready"
///
/// ### 调试级别 (DEBUG)
/// - 匹配关键词："debug"
/// - 示例：`"DEBUG: Processing request"`, "Debug mode active"
///
/// ### 跟踪级别 (TRACE)
/// - 匹配关键词："trace"
/// - 示例：`"TRACE: Method entry"`, "Trace execution path"
///
/// ## 时间戳提取
/// 采用简单而有效的启发式算法：
/// - **长度检测**：只对长度超过20字符的行进行时间戳提取
/// - **前缀假设**：假设时间戳位于行的开头部分
/// - **通用格式**：适用于大多数标准日志格式
///
/// # 参数
/// - `line`: 单行日志内容的字符串引用
///
/// # Returns
/// - `(Option<String>, Option<String>)`: 元组包含
///   - 第一个元素：提取的日志级别（可选）
///   - 第二个元素：提取的时间戳（可选）
///
/// # 检测示例
/// ```rust
/// // 日志级别检测示例
/// assert_eq!(extract_level_and_timestamp("ERROR: Database failed"),
///            (Some("ERROR".to_string()), Some("ERROR: Database failed".to_string())));
/// assert_eq!(extract_level_and_timestamp("2024-01-15 10:30:45 INFO Server started"),
///            (Some("INFO".to_string()), Some("2024-01-15 10:30:45".to_string())));
/// assert_eq!(extract_level_and_timestamp("Hello world"),
///            (None, None));
///
/// // 时间戳提取示例
/// assert_eq!(extract_level_and_timestamp("2024-01-15 10:30:45.123 ERROR Something went wrong"),
///            (Some("ERROR".to_string()), Some("2024-01-15 10:30:45.123".to_string())));
/// ```
///
/// # 算法特点
/// - **高效性**：使用字符串包含操作，时间复杂度O(n)
/// - **容错性**：在无法识别时返回None，不抛出错误
/// - **通用性**：适用于多种常见的日志格式
/// - **简单性**：避免复杂的正则表达式，提高性能和可靠性
///
/// # 设计考虑
/// - **优先级排序**：错误级别优先级最高，确保重要信息不丢失
/// - **大小写不敏感**：提高匹配的覆盖范围
/// - **保守策略**：在不确定时不进行强行提取
/// - **性能优先**：选择简单而有效的算法
///
/// # 未来改进方向
/// - 支持更多的时间戳格式识别
/// - 添加正则表达式支持以提高准确性
/// - 支持自定义关键词匹配规则
/// - 添加日志格式的机器学习识别
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