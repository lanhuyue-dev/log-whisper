/// 原始文本日志解析器
///
/// 这是插件系统中最基础和通用的解析器，能够处理任何类型的文本日志内容。
/// 作为万能回退选项，当其他专用解析器无法识别日志格式时，RawParser确保系统总能提供解析结果。
///
/// # 设计理念
/// - **通用性**：能够处理任何文本格式的日志
/// - **简单性**：不进行复杂的格式解析，保持原始内容
/// - **可靠性**：作为其他解析器的最后回退选项
/// - **一致性**：使用UnifiedFormatter确保输出格式统一
///
/// # 功能特性
/// - 逐行处理：保持原始的行结构和顺序
/// - 内容保留：完整保留原始日志内容，不进行修改
/// - 元数据简单：不添加额外的解析元数据
/// - 格式统一：通过UnifiedFormatter提供一致的显示格式
/// - 零失败率：能够处理任何输入内容
///
/// # 应用场景
/// - 未知格式的日志文件
/// - 简单的文本日志内容
/// - 其他解析器的回退选项
/// - 调试和测试环境
/// - 自定义格式的预处理
/// - 系统日志和配置文件
///
/// # 在插件系统中的角色
/// RawParser在插件架构中扮演重要的"守门员"角色：
/// 1. **最终回退**：当所有专用解析器都无法处理时，RawParser保证至少能处理内容
/// 2. **格式基础**：为其他解析器提供基础的格式化框架
/// 3. **兼容性保障**：确保系统对任何日志输入都有响应
/// 4. **性能基准**：为其他解析器提供性能对比的基准线
///
/// # 性能特点
/// - 极高效率：只进行基本的行分割处理
/// - 低内存占用：不需要复杂的解析状态
/// - 线性时间复杂度：O(n)，n为行数
/// - 最小开销：不进行正则匹配或复杂分析

use crate::plugins::{LogParser, ParseRequest, ParseResult, LogLine};
use crate::plugins::formatter::UnifiedFormatter;
use std::collections::HashMap;

/// 原始文本日志解析器实现
///
/// 这是一个最简单的无状态结构体，提供最基本的日志解析功能。
/// 不进行任何格式分析或内容解析，只负责将文本内容分割为结构化的行。
pub struct RawParser;

impl LogParser for RawParser {
    /// 返回解析器的唯一名称标识符
    ///
    /// # Returns
    /// - `&str`: "raw"
    fn name(&self) -> &str {
        "raw"
    }

    /// 返回解析器的功能描述
    ///
    /// # Returns
    /// - `&str`: "原始文本日志解析器"
    fn description(&self) -> &str {
        "原始文本日志解析器"
    }

    /// 返回支持的文件扩展名列表
    ///
    /// 作为通用解析器，支持常见的文本文件扩展名。
    ///
    /// # Returns
    /// - `Vec<String>`: 支持的文件扩展名列表
    fn supported_extensions(&self) -> Vec<String> {
        vec!["log".to_string(), "txt".to_string(), "out".to_string()]
    }

    /// 检查是否能解析给定的日志内容
    ///
    /// RawParser作为万能解析器，能够处理任何文本内容。
    /// 这确保了在自动检测模式中，当其他专用解析器都无法识别格式时，
    /// RawParser总能提供解析结果，保证系统的健壮性。
    ///
    /// # 设计考虑
    /// - **万能回退**：确保系统总能处理任何输入
    /// - **零失败率**：从不拒绝解析任何内容
    /// - **兼容性保障**：作为插件系统的最后一道防线
    ///
    /// # 参数
    /// - `_content`: 日志内容样本（未使用，因为能处理任何内容）
    /// - `_file_path`: 文件路径（未使用）
    ///
    /// # Returns
    /// - `bool`: 始终返回true，表示能解析任何内容
    ///
    /// # 在插件系统中的作用
    /// 在自动检测模式下，RawParser通常作为最后的选项：
    /// 1. 首先尝试专用解析器（SpringBoot, MyBatis, Docker JSON等）
    /// 2. 如果所有专用解析器都无法识别，使用auto解析器
    /// 3. 如果auto解析器也失败，RawParser作为最终回退选项
    ///
    /// 这种设计确保了系统的**零失败率**和**最大兼容性**。
    fn can_parse(&self, _content: &str, _file_path: Option<&str>) -> bool {
        true // Raw parser can parse any text
    }

    /// 执行原始文本日志解析
    ///
    /// 这是RawParser的核心功能，提供最简单直接的日志解析。
    /// 不进行任何复杂的格式分析或内容处理，只负责将文本内容按行分割为结构化的LogLine列表。
    ///
    /// # 解析流程
    /// 1. 逐行遍历日志内容，为每行创建LogLine结构
    /// 2. 行号处理：为每行分配从1开始的行号
    /// 3. 内容保留：完整保留原始文本内容，不做任何修改
    /// 4. 元数据创建：创建空的元数据HashMap
    /// 5. 统一格式化：使用UnifiedFormatter确保输出格式一致
    /// 6. 结果封装：构建完整的ParseResult返回
    ///
    /// # 处理特点
    /// - **内容完整性**：完全保留原始内容，不进行任何修改或解析
    /// - **结构化输出**：将自由格式文本转换为结构化的LogLine列表
    /// - **行号追踪**：为每行分配准确的行号，便于定位和引用
    /// - **格式统一**：通过UnifiedFormatter确保与其他插件的显示格式一致
    ///
    /// # 输出特征
    /// - content: 原始行内容（完全保留）
    /// - level: 由UnifiedFormatter推断或默认值
    /// - timestamp: 由UnifiedFormatter提取或默认值
    /// - formatted_content: 统一格式的显示字符串
    /// - metadata: 空的HashMap（无额外元数据）
    /// - processed_by: ["raw_parser"]
    ///
    /// # 参数
    /// - `content`: 要解析的完整文本内容
    /// - `_request`: 解析请求参数（当前未使用）
    ///
    /// # Returns
    /// - `Ok(ParseResult)`: 解析成功的结构化日志结果
    /// - `Err(String)`: 解析失败时的错误描述（实际上总是成功）
    ///
    /// # 性能特性
    /// - **极简处理**：只进行基本的行分割和结构化
    /// - **高效内存使用**：不存储额外的解析状态
    /// - **线性时间复杂度**：O(n)，n为行数
    /// - **零错误率**：不会产生解析错误
    ///
    /// # 使用场景示例
    /// ```rust
    /// // 输入内容
    /// let content = "Line 1\nLine 2\nLine 3";
    ///
    /// // 解析结果
    /// // Line 1: content="Line 1", line_number=1
    /// // Line 2: content="Line 2", line_number=2
    /// // Line 3: content="Line 3", line_number=3
    /// ```
    ///
    /// # 设计原则
    /// RawParser遵循"做最少的事情，做得最好"的原则：
    /// - 不假设任何日志格式
    /// - 不修改任何原始内容
    /// - 不添加任何解析偏见
    /// - 确保最大的兼容性和可靠性
    fn parse(&self, content: &str, _request: &ParseRequest) -> Result<ParseResult, String> {
        let lines: Vec<LogLine> = content
            .lines()
            .enumerate()
            .map(|(i, line)| {
                let metadata = HashMap::new();

                // 使用统一格式化器
                let unified_format = UnifiedFormatter::format_log_line(
                    i + 1,
                    line,
                    None,
                    None,
                    &metadata,
                    "raw"
                );

                let formatted_content = UnifiedFormatter::format_display_string(&unified_format);

                LogLine {
                    line_number: i + 1,
                    content: line.to_string(),
                    level: unified_format.level.clone(),
                    timestamp: unified_format.timestamp.clone(),
                    formatted_content: Some(formatted_content),
                    metadata,
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