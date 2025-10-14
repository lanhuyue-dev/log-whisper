/// 基础插件管理器
///
/// 负责管理所有日志解析插件的生命周期，包括插件的注册、发现和调用。
/// 提供统一的插件访问接口和自动格式检测功能。
///
/// # 核心功能
/// - 插件注册和管理
/// - 自动格式检测和解析器选择
/// - 插件信息查询和元数据管理
/// - 线程安全的插件调用
///
/// # 设计特点
/// - 使用HashMap进行高效的插件查找
/// - Arc包装确保线程安全
/// - 支持插件的动态注册
/// - 提供自动回退机制
///
/// # 插件注册策略
/// - 构造时自动注册所有内置插件
/// - 每个插件都有唯一的名称标识符
/// - 支持插件的热替换（未来功能）

use crate::plugins::{LogParser, PluginInfo, ParseRequest, ParseResult};
use std::collections::HashMap;
use std::sync::Arc;
use log::{debug, error};

/// 插件管理器主结构
///
/// 内部使用HashMap存储插件实例，键为插件名称，值为插件对象的Arc引用。
/// 这种设计确保了插件的线程安全访问和高效查找。
pub struct PluginManager {
    /// 插件注册表，键为插件名称，值为插件实例
    parsers: HashMap<String, Arc<dyn LogParser + Send + Sync>>,
}

impl PluginManager {
    /// 创建新的插件管理器实例
    ///
    /// 自动注册所有内置的日志解析插件。
    /// 每个插件都有其特定的用途和适用场景。
    ///
    /// # 注册的插件
    /// - `auto`: 自动格式检测插件（万能解析器）
    /// - `mybatis`: MyBatis SQL日志专用解析器
    /// - `docker_json`: Docker容器JSON日志解析器
    /// - `raw`: 原始文本日志解析器（最基础）
    /// - `springboot`: SpringBoot应用日志解析器
    ///
    /// # Returns
    /// - `Self`: 初始化完成的插件管理器实例
    ///
    /// # 插件优先级
    /// 在自动检测时，按注册顺序优先级进行检测，
    /// 专用解析器优先于通用解析器。
    pub fn new() -> Self {
        let mut parsers: HashMap<String, Arc<dyn LogParser + Send + Sync>> = HashMap::new();

        // 注册内置解析器
        // 按照优先级和专用性排序

        // 自动检测插件 - 作为万能解析器，优先级较低
        parsers.insert("auto".to_string(), Arc::new(crate::plugins::auto::AutoParser));

        // 专用格式解析器 - 具有高检测准确性
        parsers.insert("mybatis".to_string(), Arc::new(crate::plugins::mybatis::MyBatisParser));
        parsers.insert("docker_json".to_string(), Arc::new(crate::plugins::docker_json::DockerJsonParser));
        parsers.insert("springboot".to_string(), Arc::new(crate::plugins::springboot::SpringBootParser));

        // 原始文本解析器 - 作为最后的回退选项
        parsers.insert("raw".to_string(), Arc::new(crate::plugins::raw::RawParser));

        Self { parsers }
    }

    /// 获取所有可用插件的详细信息
    ///
    /// 返回系统中所有已注册插件的元数据信息，
    /// 包括插件名称、描述、支持的文件扩展名等。
    /// 这个方法主要用于插件发现和用户界面展示。
    ///
    /// # Returns
    /// - `Vec<PluginInfo>`: 所有插件的详细信息列表
    ///
    /// # 用途
    /// - 前端插件选择界面
    /// - 插件能力查询
    /// - 系统状态报告
    pub fn get_available_plugins(&self) -> Vec<PluginInfo> {
        self.parsers.values().map(|parser| {
            PluginInfo {
                name: parser.name().to_string(),
                description: parser.description().to_string(),
                supported_extensions: parser.supported_extensions(),
                auto_detectable: true, // 当前所有插件都支持自动检测
            }
        }).collect()
    }

    /// 使用指定插件解析日志内容
    ///
    /// 根据插件名称查找对应的解析器并执行解析操作。
    /// 这是显式指定插件的解析方式，不进行自动检测。
    ///
    /// # 参数
    /// - `plugin_name`: 要使用的插件名称
    /// - `request`: 解析请求参数
    ///
    /// # Returns
    /// - `Ok(ParseResult)`: 解析成功，包含结构化日志
    /// - `Err(String)`: 插件不存在或解析失败
    ///
    /// # 错误处理
    /// - 插件不存在时返回明确的错误信息
    /// - 解析失败时传递解析器的错误信息
    ///
    /// # 使用场景
    /// - 用户明确指定日志格式
    /// - 测试特定解析器的性能
    /// - 强制使用特定解析策略
    pub fn parse_with_plugin(&self, plugin_name: &str, request: &ParseRequest) -> Result<ParseResult, String> {
        let parser = self.parsers.get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;

        debug!("🔧 使用插件 '{}' 解析日志内容", plugin_name);
        parser.parse(&request.content, request)
    }

    /// 自动检测格式并解析日志内容
    ///
    /// 智能地选择最适合的解析器来处理日志内容。
    /// 这是推荐的使用方式，能够处理大多数常见的日志格式。
    ///
    /// # 检测策略
    /// 1. 首先尝试专用解析器（mybatis, docker_json, springboot）
    /// 2. 如果没有匹配，使用auto解析器进行通用检测
    /// 3. 最后使用raw解析器作为回退选项
    ///
    /// # 参数
    /// - `request`: 解析请求参数
    ///
    /// # Returns
    /// - `Ok(ParseResult)`: 解析成功，包含检测结果和解析内容
    /// - `Err(String)`: 所有解析器都失败
    ///
    /// # 检测优先级
    /// 专用解析器 > auto解析器 > raw解析器
    ///
    /// # 性能考虑
    /// - 快速检测：使用can_parse()进行预检
    /// - 早期退出：找到合适的解析器立即返回
    /// - 回退机制：确保总能解析某种格式的内容
    pub fn auto_detect_and_parse(&self, request: &ParseRequest) -> Result<ParseResult, String> {
        let content = &request.content;
        let file_path = request.file_path.as_deref();

        debug!("🔍 开始自动检测日志格式，内容长度: {}", content.len());

        // 第一步：尝试专用解析器
        // 这些解析器针对特定格式进行了优化，准确性更高
        for (name, parser) in &self.parsers {
            // 跳过通用解析器，优先使用专用解析器
            if name != "auto" && name != "raw" && parser.can_parse(content, file_path) {
                debug!("✅ 检测到专用格式: {}", name);
                let mut result = parser.parse(content, request)?;
                result.detected_format = Some(name.clone());
                return Ok(result);
            }
        }

        // 第二步：使用auto解析器
        // 这是一个万能解析器，可以处理大多数标准格式
        if let Some(auto_parser) = self.parsers.get("auto") {
            debug!("🔧 使用auto解析器进行通用检测");
            let mut result = auto_parser.parse(content, request)?;
            result.detected_format = Some("auto".to_string());
            return Ok(result);
        }

        // 第三步：使用raw解析器作为最后回退
        // 这是最基础的解析器，不会失败但信息最少
        if let Some(raw_parser) = self.parsers.get("raw") {
            debug!("🔧 使用raw解析器作为回退选项");
            let mut result = raw_parser.parse(content, request)?;
            result.detected_format = Some("raw".to_string());
            return Ok(result);
        }

        // 如果连raw解析器都没有，说明系统配置有问题
        error!("❌ 系统错误：没有找到任何可用的解析器");
        Err("No suitable parser found".to_string())
    }

  }

/// 插件管理器的默认实现
///
/// 提供Default trait实现，允许使用PluginManager::default()创建实例。
/// 这在需要延迟初始化或配置驱动的场景中很有用。
impl Default for PluginManager {
    /// 创建默认的插件管理器实例
    ///
    /// 等同于调用PluginManager::new()，自动注册所有内置插件。
    ///
    /// # Returns
    /// - `Self`: 配置完整的插件管理器实例
    fn default() -> Self {
        Self::new()
    }
}