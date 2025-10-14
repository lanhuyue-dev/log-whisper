/// LogWhisper 插件系统模块
///
/// 这个模块提供了完整的日志解析插件架构，支持多种日志格式的解析和处理。
/// 插件系统采用可扩展的设计模式，允许轻松添加新的日志解析器。
///
/// # 架构组件
/// - **核心系统**: 增强插件管理器和基础插件管理器
/// - **解析器插件**: 针对特定日志格式的专用解析器
/// - **数据结构**: 统一的日志条目和解析结果格式
/// - **工具模块**: 格式化器和测试工具
///
/// # 支持的日志格式
/// - SpringBoot: Java应用日志格式
/// - MyBatis: SQL框架日志格式
/// - Docker JSON: 容器日志JSON格式
/// - Auto: 自动格式检测
/// - Raw: 原始文本格式
///
/// # 插件特性
/// - 自动格式检测
/// - 性能优化的大文件处理
/// - 错误恢复和优雅降级
/// - 可配置的解析策略

// ============================================================================
// 插件模块导入
// ============================================================================

// 核心解析器插件
pub mod auto;        // 自动格式检测插件 - 智能日志格式识别
pub mod mybatis;     // MyBatis SQL日志解析器 - SQL日志专用解析
pub mod docker_json; // Docker容器日志解析器 - Docker JSON格式解析
pub mod raw;         // 原始文本解析器 - 通用文本日志解析
pub mod springboot;  // SpringBoot日志解析器 - Java应用日志解析

// 系统管理模块
pub mod manager;     // 基础插件管理器 - 插件注册和调用核心
pub mod core;        // 增强插件管理器 - 高级插件管理功能
pub mod formatter;   // 格式化工具 - 统一日志格式化显示

// ============================================================================
// 核心数据结构
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 日志条目类型别名 (向后兼容)
///
/// 为了保持API兼容性而保留的类型别名。
/// 新代码应该直接使用 `LogLine` 类型。
pub type LogEntry = LogLine;

/// 日志行数据结构
///
/// 表示解析后的单行日志条目，包含原始内容和解析后的结构化信息。
/// 这是插件系统的核心数据结构，用于在解析器之间传递数据。
///
/// # 字段说明
/// - `line_number`: 在原文件中的行号（从1开始）
/// - `content`: 原始日志内容
/// - `level`: 解析出的日志级别（如INFO, ERROR等）
/// - `timestamp`: 解析出的时间戳
/// - `formatted_content`: 格式化后的显示内容
/// - `metadata`: 附加元数据（如线程ID、类名等）
/// - `processed_by`: 处理此条目的插件列表
///
/// # 设计特点
/// - 支持多种日志格式的统一表示
/// - 可选字段适应不同的解析需求
/// - 元数据支持扩展信息存储
/// - 处理链追踪用于调试和优化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    /// 在原文件中的行号（从1开始）
    pub line_number: usize,

    /// 原始日志内容（保持不变）
    pub content: String,

    /// 解析出的日志级别（如INFO, ERROR, WARN等）
    pub level: Option<String>,

    /// 解析出的时间戳（ISO格式或原始格式）
    pub timestamp: Option<String>,

    /// 格式化后的显示内容（可能包含高亮、结构化信息）
    pub formatted_content: Option<String>,

    /// 附加元数据（如线程ID、类名、方法名等）
    pub metadata: HashMap<String, String>,

    /// 处理此条目的插件名称列表（用于追踪处理链）
    pub processed_by: Vec<String>,
}

/// 解析结果数据结构
///
/// 包含日志解析的完整结果，包括解析的日志条目、统计信息和错误状态。
/// 这是所有解析器必须返回的统一结果格式。
///
/// # 字段说明
/// - `lines`: 解析后的日志行列表
/// - `total_lines`: 原始内容的总行数
/// - `detected_format`: 检测到的日志格式
/// - `parsing_errors`: 解析过程中遇到的错误列表
///
/// # 错误处理策略
/// - 非致命错误继续处理其他行
/// - 收集所有错误信息供用户参考
/// - 即使有错误也返回部分结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// 解析后的日志行列表
    pub lines: Vec<LogLine>,

    /// 原始内容的总行数（包括空行）
    pub total_lines: usize,

    /// 自动检测到的日志格式（如"SpringBoot", "DockerJson"等）
    pub detected_format: Option<String>,

    /// 解析过程中遇到的错误列表
    pub parsing_errors: Vec<String>,
}

/// 解析请求数据结构
///
/// 封装日志解析请求的所有参数，提供给插件解析器使用。
/// 支持多种解析模式和配置选项。
///
/// # 字段说明
/// - `content`: 要解析的日志内容
/// - `plugin`: 指定使用的插件名称（可选）
/// - `file_path`: 源文件路径（用于格式检测）
/// - `chunk_size`: 分块处理时的块大小
///
/// # 使用模式
/// - 指定插件模式：设置plugin字段使用特定解析器
/// - 自动检测模式：不设置plugin，让系统自动选择
/// - 分块处理模式：设置chunk_size用于大文件处理
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseRequest {
    /// 要解析的日志内容
    pub content: String,

    /// 指定使用的插件名称（可选，不指定则自动检测）
    pub plugin: Option<String>,

    /// 源文件路径（用于文件扩展名和格式检测）
    pub file_path: Option<String>,

    /// 分块处理时的块大小（用于大文件优化）
    pub chunk_size: Option<usize>,
}

impl Default for ParseRequest {
    /// 创建默认的解析请求
    ///
    /// 返回一个空的解析请求，适用于手动构建请求对象。
    fn default() -> Self {
        Self {
            content: String::new(),
            plugin: None,
            file_path: None,
            chunk_size: None,
        }
    }
}

/// 插件信息数据结构
///
/// 描述单个日志解析插件的基本信息和能力。
/// 用于插件发现、选择和用户界面展示。
///
/// # 字段说明
/// - `name`: 插件的唯一标识符
/// - `description`: 插件功能的用户友好描述
/// - `supported_extensions`: 支持的文件扩展名列表
/// - `auto_detectable`: 是否支持自动检测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// 插件的唯一标识符（用于API调用）
    pub name: String,

    /// 插件功能的用户友好描述
    pub description: String,

    /// 支持的文件扩展名列表（如[".log", ".txt"]）
    pub supported_extensions: Vec<String>,

    /// 是否支持自动格式检测
    pub auto_detectable: bool,
}

/// 日志解析器特征
///
/// 定义了所有日志解析插件必须实现的核心接口。
/// 这是插件系统的核心抽象，确保所有解析器的一致性。
///
/// # 方法说明
/// - `name()`: 返回解析器的唯一名称
/// - `description()`: 返回解析器的功能描述
/// - `supported_extensions()`: 返回支持的文件扩展名
/// - `can_parse()`: 检查是否能解析给定的内容
/// - `parse()`: 执行实际的日志解析
///
/// # 实现要求
/// - 线程安全：解析器必须支持多线程调用
/// - 错误处理：提供清晰的错误信息
/// - 性能考虑：支持大文件的高效处理
/// - 一致性：返回统一格式的解析结果
///
/// # 扩展指南
/// 实现此特征来添加新的日志格式支持：
/// 1. 实现所有必需的方法
/// 2. 在PluginManager中注册新解析器
/// 3. 添加相应的测试用例
pub trait LogParser {
    /// 返回解析器的唯一名称
    ///
    /// # Returns
    /// - `&str`: 解析器的名称标识符
    fn name(&self) -> &str;

    /// 返回解析器的功能描述
    ///
    /// # Returns
    /// - `&str`: 解析器的用户友好描述
    fn description(&self) -> &str;

    /// 返回支持的文件扩展名列表
    ///
    /// # Returns
    /// - `Vec<String>`: 支持的文件扩展名（如[".log", ".out"]）
    fn supported_extensions(&self) -> Vec<String>;

    /// 检查是否能解析给定的内容
    ///
    /// 通过分析内容和文件路径来判断此解析器是否适合处理该日志。
    /// 实现应该进行快速检查，避免昂贵的解析操作。
    ///
    /// # 参数
    /// - `content`: 日志内容的预览
    /// - `file_path`: 可选的文件路径信息
    ///
    /// # Returns
    /// - `bool`: true表示可以解析，false表示不适合
    fn can_parse(&self, content: &str, file_path: Option<&str>) -> bool;

    /// 执行日志解析
    ///
    /// 核心解析方法，将原始日志内容转换为结构化的LogLine列表。
    ///
    /// # 参数
    /// - `content`: 要解析的完整日志内容
    /// - `request`: 解析请求配置参数
    ///
    /// # Returns
    /// - `Ok(ParseResult)`: 解析成功，包含结构化日志行
    /// - `Err(String)`: 解析失败，包含错误描述
    ///
    /// # 错误处理
    /// - 提供清晰的错误信息
    /// - 非致命错误不应该中断整个解析过程
    /// - 在ParseResult中收集解析警告
    fn parse(&self, content: &str, request: &ParseRequest) -> Result<ParseResult, String>;
}