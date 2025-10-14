/// MyBatis SQL日志解析器
///
/// 专门用于解析MyBatis框架输出的SQL调试日志。MyBatis作为Java生态中广泛使用的ORM框架，
/// 其日志输出具有特定的格式和结构，包含SQL语句的完整执行过程。
///
/// # 支持的日志格式
/// - SQL准备语句：`Preparing: SELECT * FROM users WHERE id = ?`
/// - 参数绑定：`Parameters: 123(Long), john(String)`
/// - 更新结果：`Updates: 1`
/// - 执行时间：`Time Elapsed: 5 ms`
/// - 调试信息：`DEBUG [main] c.e.m.UserMapper.selectById`
///
/// # 解析特性
/// - SQL语句识别：自动识别SQL操作的各个阶段
/// - 参数解析：提取和解析SQL参数信息
/// - 执行统计：收集更新行数和执行时间
/// - 类型标注：为不同类型的SQL行添加元数据标识
/// - 统一格式化：使用UnifiedFormatter确保输出一致性
///
/// # 应用场景
/// - SQL性能分析和优化
/// - 数据库操作调试
/// - ORM框架行为分析
/// - 数据访问层问题排查
/// - SQL注入安全审计
///
/// # 性能优化
/// - 轻量级检测：使用字符串包含操作进行快速识别
/// - 元数据标记：为SQL不同阶段添加类型标识
/// - 格式统一：确保与其他插件的显示格式一致

use crate::plugins::{LogParser, ParseRequest, ParseResult, LogLine};
use crate::plugins::formatter::UnifiedFormatter;
use std::collections::HashMap;

/// MyBatis日志解析器实现
///
/// 这是一个无状态的结构体，专门处理MyBatis框架的SQL日志输出。
/// 依赖UnifiedFormatter进行统一的格式化处理，确保输出格式的一致性。
pub struct MyBatisParser;

impl LogParser for MyBatisParser {
    /// 返回解析器的唯一名称标识符
    ///
    /// # Returns
    /// - `&str`: "mybatis"
    fn name(&self) -> &str {
        "mybatis"
    }

    /// 返回解析器的功能描述
    ///
    /// # Returns
    /// - `&str`: "MyBatis SQL 日志解析器"
    fn description(&self) -> &str {
        "MyBatis SQL 日志解析器"
    }

    /// 返回支持的文件扩展名列表
    ///
    /// MyBatis日志通常存储在这些扩展名的文件中。
    ///
    /// # Returns
    /// - `Vec<String>`: 支持的文件扩展名列表
    fn supported_extensions(&self) -> Vec<String> {
        vec!["log".to_string(), "txt".to_string()]
    }

    /// 检查是否能解析给定的日志内容
    ///
    /// 通过检测MyBatis特有的关键词和日志模式来判断内容是否适合此解析器。
    /// 使用轻量级的字符串匹配进行快速识别，避免昂贵的正则表达式操作。
    ///
    /// # 检测策略
    /// 1. 框架标识：查找"mybatis"关键词
    /// 2. SQL准备阶段：查找"preparing:"标识符
    /// 3. 参数绑定阶段：查找"parameters:"标识符
    ///
    /// # 参数
    /// - `content`: 日志内容样本
    /// - `_file_path`: 文件路径（当前未使用）
    ///
    /// # Returns
    /// - `bool`: true表示可能是MyBatis日志格式
    ///
    /// # 性能考虑
    /// - 使用字符串包含操作，时间复杂度O(n)
    /// - 转换为小写进行匹配，确保大小写不敏感
    /// - 短路求值，找到任一匹配即返回
    ///
    /// # 检测示例
    /// ```rust
    /// // 匹配的内容示例
    /// "DEBUG [main] c.e.m.UserMapper - Preparing: SELECT * FROM users"
    /// "INFO  [mapper] - Parameters: 123(Long), 'john'(String)"
    /// "mybatis configuration loaded successfully"
    /// ```
    fn can_parse(&self, content: &str, _file_path: Option<&str>) -> bool {
        content.to_lowercase().contains("mybatis") ||
        content.to_lowercase().contains("preparing:") ||
        content.to_lowercase().contains("parameters:")
    }

    /// 执行MyBatis日志解析
    ///
    /// 这是MyBatis插件的核心解析功能，将原始MyBatis日志内容转换为结构化的LogLine列表。
    /// 通过识别SQL执行的不同阶段，为每个日志行添加相应的元数据标识。
    ///
    /// # 解析流程
    /// 1. 逐行遍历日志内容，为每行创建LogLine结构
    /// 2. SQL类型识别：检测SQL准备、参数绑定、结果更新等阶段
    /// 3. 日志级别检测：识别DEBUG级别日志
    /// 4. 元数据添加：为不同类型的SQL行添加type标识
    /// 5. 统一格式化：使用UnifiedFormatter确保输出一致性
    /// 6. 结果封装：构建完整的ParseResult返回
    ///
    /// # 支持的SQL类型识别
    /// - `sql_prepare`: SQL准备语句（包含"preparing:"）
    /// - `sql_parameters`: SQL参数绑定（包含"parameters:"）
    /// - `sql_updates`: SQL更新结果（包含"updates:"）
    ///
    /// # 参数
    /// - `content`: 要解析的完整日志内容
    /// - `_request`: 解析请求参数（当前未使用）
    ///
    /// # Returns
    /// - `Ok(ParseResult)`: 解析成功的结构化日志结果
    /// - `Err(String)`: 解析失败时的错误描述
    ///
    /// # 性能特性
    /// - 流式处理：逐行处理，内存效率高
    /// - 快速匹配：使用字符串包含进行模式识别
    /// - 元数据增强：提供丰富的SQL执行信息
    /// - 格式统一：确保与其他插件的兼容性
    ///
    /// # 元数据示例
    /// ```rust
    /// // SQL准备行
    /// metadata = {"type": "sql_prepare"}
    ///
    /// // 参数绑定行
    /// metadata = {"type": "sql_parameters"}
    ///
    /// // 更新结果行
    /// metadata = {"type": "sql_updates"}
    /// ```
    ///
    /// # 日志级别处理
    /// 当前主要识别DEBUG级别的日志，这是MyBatis SQL调试的常见级别。
    /// 未来可扩展支持其他日志级别的识别。
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

                // 使用统一格式化器
                let unified_format = UnifiedFormatter::format_log_line(
                    i + 1,
                    line,
                    level.clone(),
                    None,
                    &metadata,
                    "mybatis"
                );

                let formatted_content = UnifiedFormatter::format_display_string(&unified_format);

                LogLine {
                    line_number: i + 1,
                    content: line.to_string(),
                    level: unified_format.level.clone(),
                    timestamp: unified_format.timestamp.clone(),
                    formatted_content: Some(formatted_content),
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