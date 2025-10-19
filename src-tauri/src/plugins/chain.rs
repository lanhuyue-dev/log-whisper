/// 插件链处理器
///
/// 实现类似Java Web Filter机制的插件链处理系统，支持多个插件按顺序处理同一条日志。
/// 每个插件都可以对日志内容进行解析、增强和转换，然后将结果传递给链中的下一个插件。
///
/// # 设计理念
/// - **Filter Chain模式**：模仿Java Servlet Filter的实现方式
/// - **顺序处理**：插件按配置顺序依次处理
/// - **数据流转**：每个插件可以修改和传递数据
/// - **条件中断**：支持提前终止处理链
/// - **结果聚合**：收集所有插件的处理结果
///
/// # 典型应用场景
/// Docker容器日志处理链：
/// 1. docker_json插件：解析JSON格式，提取log字段
/// 2. springboot插件：解析SpringBoot格式，提取结构化信息
/// 3. mybatis插件：识别并格式化SQL语句
/// 4. json_formatter插件：最终JSON格式化输出
///
/// # 性能考虑
/// - **短路机制**：遇到错误可以提前终止
/// - **并行处理**：某些独立插件可以并行执行（未来功能）
/// - **内存优化**：流式处理，避免大量内存占用
/// - **缓存机制**：缓存常用处理结果（未来功能）

use crate::plugins::{ParseRequest, ParseResult, LogLine};
use std::collections::HashMap;
use std::sync::Arc;
use log::{debug, info, warn, error};

/// 插件链上下文
///
/// 包含插件链处理过程中的所有状态信息，包括原始数据、中间结果和最终输出。
/// 这个上下文会在插件链中传递，每个插件都可以读取和修改其中的数据。
///
/// # 字段说明
/// - `original_content`: 原始日志内容（只读，用于调试）
/// - `current_lines`: 当前处理的日志行列表（可修改）
/// - `processing_chain`: 已执行的插件名称列表
/// - `chain_metadata`: 插件链级别的元数据
/// - `should_continue`: 是否继续执行后续插件
/// - `errors`: 处理过程中收集的错误信息
#[derive(Debug, Clone)]
pub struct PluginChainContext {
    /// 原始日志内容（用于调试和回溯）
    pub original_content: String,

    /// 当前处理的日志行列表（会被插件修改）
    pub current_lines: Vec<LogLine>,

    /// 已执行的插件名称处理链
    pub processing_chain: Vec<String>,

    /// 插件链级别的元数据（跨插件共享）
    pub chain_metadata: HashMap<String, String>,

    /// 是否继续执行后续插件（可用于提前终止）
    pub should_continue: bool,

    /// 处理过程中收集的错误信息
    pub errors: Vec<String>,
}

impl PluginChainContext {
    /// 创建新的插件链上下文
    ///
    /// # 参数
    /// - `content`: 原始日志内容
    ///
    /// # Returns
    /// - `Self`: 新创建的上下文实例
    pub fn new(content: String) -> Self {
        Self {
            original_content: content.clone(),
            current_lines: Vec::new(),
            processing_chain: Vec::new(),
            chain_metadata: HashMap::new(),
            should_continue: true,
            errors: Vec::new(),
        }
    }

    /// 添加错误信息到上下文
    ///
    /// # 参数
    /// - `error`: 错误描述字符串
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// 设置链级别的元数据
    ///
    /// # 参数
    /// - `key`: 元数据键
    /// - `value`: 元数据值
    pub fn set_chain_metadata(&mut self, key: String, value: String) {
        self.chain_metadata.insert(key, value);
    }

    /// 获取链级别的元数据
    ///
    /// # 参数
    /// - `key`: 元数据键
    ///
    /// # Returns
    /// - `Option<&String>`: 元数据值的引用（如果存在）
    #[allow(dead_code)]
    pub fn get_chain_metadata(&self, key: &str) -> Option<&String> {
        self.chain_metadata.get(key)
    }

    /// 停止后续插件的执行
    ///
    /// 通常在遇到致命错误或完成所有必要处理时调用。
    #[allow(dead_code)]
    pub fn stop_chain(&mut self) {
        self.should_continue = false;
        debug!("🛑 插件链执行被停止");
    }
}

/// 插件链过滤器特征
///
/// 定义了插件链中每个处理器必须实现的接口。
/// 这类似于Java Web中的Filter接口，每个处理器都可以对请求进行处理和转换。
///
/// # 方法说明
/// - `name()`: 返回处理器的唯一名称
/// - `description()`: 返回处理器的功能描述
/// - `priority()`: 返回处理器的优先级（数值越小优先级越高）
/// - `should_process()`: 判断是否应该处理当前的上下文
/// - `process()`: 执行具体的处理逻辑
/// - `can_handle()`: 判断是否能处理特定类型的日志
///
/// # 实现要求
/// - **幂等性**：多次处理应该产生相同结果
/// - **线程安全**：处理器必须支持并发调用
/// - **错误处理**：提供清晰的错误信息
/// - **性能考虑**：避免阻塞和不必要的计算
pub trait PluginFilter {
    /// 返回过滤器的唯一名称
    ///
    /// # Returns
    /// - `&str`: 过滤器的名称标识符
    fn name(&self) -> &str;

    /// 返回过滤器的功能描述
    ///
    /// # Returns
    /// - `&str`: 过滤器的用户友好描述
    #[allow(dead_code)]
    fn description(&self) -> &str;

    /// 返回过滤器的优先级
    ///
    /// 数值越小优先级越高，用于确定过滤器在链中的执行顺序。
    ///
    /// # Returns
    /// - `i32`: 优先级数值
    fn priority(&self) -> i32;

    /// 判断是否应该处理当前的上下文
    ///
    /// 在执行process()之前调用，用于避免不必要的处理。
    /// 可以基于上下文内容、元数据等进行判断。
    ///
    /// # 参数
    /// - `context`: 当前插件链上下文的可变引用
    ///
    /// # Returns
    /// - `bool`: true表示应该处理，false表示跳过
    fn should_process(&self, context: &PluginChainContext) -> bool;

    /// 执行过滤器的处理逻辑
    ///
    /// 这是过滤器的核心方法，负责对日志内容进行处理和转换。
    /// 可以修改context中的current_lines、添加元数据、设置处理链信息等。
    ///
    /// # 参数
    /// - `context`: 当前插件链上下文的可变引用
    /// - `request`: 原始解析请求参数
    ///
    /// # Returns
    /// - `Result<(), String>`: 处理成功返回Ok(())，失败返回错误描述
    ///
    /// # 处理规范
    /// - 修改context.current_lines来更新数据
    /// - 在context.processing_chain中记录处理信息
    /// - 通过context.chain_metadata共享数据
    /// - 遇到错误时调用context.add_error()
    fn process(&self, context: &mut PluginChainContext, request: &ParseRequest) -> Result<(), String>;

    /// 判断是否能处理特定类型的日志内容
    ///
    /// 用于插件链的智能选择和优化。
    /// 基于日志内容特征判断是否适合使用此过滤器。
    ///
    /// # 参数
    /// - `content`: 日志内容样本
    /// - `file_path`: 可选的文件路径信息
    ///
    /// # Returns
    /// - `bool`: true表示可以处理，false表示不适合
    fn can_handle(&self, content: &str, file_path: Option<&str>) -> bool;
}

/// 插件链定义
///
/// 定义了一系列过滤器的处理链，包括执行顺序、条件和配置。
/// 支持基于不同场景的链配置，如Docker日志处理链、SpringBoot日志处理链等。
///
/// # 字段说明
/// - `name`: 链的名称（用于识别和配置）
/// - `description`: 链的功能描述
/// - `filters`: 按优先级排序的过滤器列表
/// - `enabled`: 是否启用此链
/// - `conditions`: 链的执行条件
#[derive(Clone)]
pub struct PluginChain {
    /// 链的唯一名称标识符
    pub name: String,

    /// 链的功能描述
    pub description: String,

    /// 按优先级排序的过滤器列表
    pub filters: Vec<Arc<dyn PluginFilter + Send + Sync>>,

    /// 是否启用此链
    pub enabled: bool,

    /// 链的执行条件（可选）
    pub conditions: Option<ChainConditions>,
}

/// 链执行条件
///
/// 定义了插件链执行的触发条件和约束。
///
/// # 字段说明
/// - `file_patterns`: 文件路径模式匹配
/// - `content_patterns`: 内容特征模式匹配
/// - `min_confidence`: 最小置信度阈值
#[derive(Debug, Clone)]
pub struct ChainConditions {
    /// 文件路径模式匹配列表
    pub file_patterns: Vec<String>,

    /// 内容特征模式匹配列表
    pub content_patterns: Vec<String>,

    /// 最小置信度阈值（0.0 - 1.0）
    pub min_confidence: f32,
}

impl ChainConditions {
    /// 创建新的链条件
    ///
    /// # Returns
    /// - `Self`: 新创建的条件实例
    pub fn new() -> Self {
        Self {
            file_patterns: Vec::new(),
            content_patterns: Vec::new(),
            min_confidence: 0.5,
        }
    }

    /// 检查是否满足执行条件
    ///
    /// # 参数
    /// - `content`: 日志内容
    /// - `file_path`: 文件路径（可选）
    ///
    /// # Returns
    /// - `bool`: true表示满足条件，false表示不满足
    pub fn matches(&self, content: &str, file_path: Option<&str>) -> bool {
        debug!("🔍 检查链条件匹配，内容长度: {}, 文件路径: {:?}", content.len(), file_path);

        // 检查文件路径模式
        if let Some(path) = file_path {
            if !self.file_patterns.is_empty() {
                let path_lower = path.to_lowercase();
                debug!("📁 检查文件路径: '{}'，模式: {:?}", path_lower, self.file_patterns);

                let matches_file = self.file_patterns.iter()
                    .any(|pattern| {
                        let pattern_lower = pattern.to_lowercase();
                        let matches = path_lower.contains(&pattern_lower);
                        debug!("  模式 '{}' -> 匹配: {}", pattern, matches);
                        matches
                    });

                debug!("📁 文件路径匹配结果: {}", matches_file);
                if !matches_file {
                    return false;
                }
            }
        }

        // 检查内容特征模式
        if !self.content_patterns.is_empty() {
            let content_lower = content.to_lowercase();
            debug!("📝 检查内容模式，模式: {:?}", self.content_patterns);

            let matches_content = self.content_patterns.iter()
                .any(|pattern| {
                    let pattern_lower = pattern.to_lowercase();
                    let matches = content_lower.contains(&pattern_lower);
                    debug!("  模式 '{}' -> 匹配: {}", pattern, matches);
                    matches
                });

            debug!("📝 内容匹配结果: {}", matches_content);
            if !matches_content {
                return false;
            }
        }

        debug!("✅ 链条件匹配成功");
        true
    }
}

impl Default for ChainConditions {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginChain {
    /// 创建新的插件链
    ///
    /// # 参数
    /// - `name`: 链的名称
    /// - `description`: 链的描述
    ///
    /// # Returns
    /// - `Self`: 新创建的链实例
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            filters: Vec::new(),
            enabled: true,
            conditions: None,
        }
    }

    /// 添加过滤器到链中
    ///
    /// 过滤器会按优先级自动排序。
    ///
    /// # 参数
    /// - `filter`: 要添加的过滤器
    pub fn add_filter(&mut self, filter: Arc<dyn PluginFilter + Send + Sync>) {
        self.filters.push(filter);
        // 按优先级排序（数值越小优先级越高）
        self.filters.sort_by_key(|f| f.priority());
    }

    /// 执行插件链处理
    ///
    /// 按顺序执行链中的所有过滤器，直到所有过滤器完成或链被停止。
    ///
    /// # 参数
    /// - `content`: 要处理的日志内容
    /// - `request`: 解析请求参数
    ///
    /// # Returns
    /// - `Result<ParseResult, String>`: 处理结果或错误信息
    ///
    /// # 执行流程
    /// 1. 创建处理上下文
    /// 2. 按优先级顺序执行每个过滤器
    /// 3. 收集处理结果和错误信息
    /// 4. 返回最终的解析结果
    pub fn process(&self, content: &str, request: &ParseRequest) -> Result<ParseResult, String> {
        info!("🔗 开始执行插件链: {}", self.name);
        let start_time = std::time::Instant::now();

        if !self.enabled {
            warn!("⚠️ 插件链 '{}' 已禁用，跳过处理", self.name);
            return Err(format!("插件链 '{}' 已禁用", self.name));
        }

        // 检查执行条件
        if let Some(conditions) = &self.conditions {
            if !conditions.matches(content, request.file_path.as_deref()) {
                debug!("🔍 插件链 '{}' 执行条件不匹配，跳过处理", self.name);
                return Err(format!("插件链 '{}' 执行条件不匹配", self.name));
            }
        }

        // 创建处理上下文
        let mut context = PluginChainContext::new(content.to_string());

        // 执行过滤器链
        for filter in &self.filters {
            // 检查是否应该继续执行
            if !context.should_continue {
                info!("🛑 插件链执行被停止在过滤器: {}", filter.name());
                break;
            }

            // 检查过滤器是否应该处理此上下文
            if !filter.should_process(&context) {
                debug!("⏭️ 过滤器 '{}' 跳过处理", filter.name());
                continue;
            }

            info!("🔄 执行过滤器: {}", filter.name());
            let filter_start = std::time::Instant::now();

            match filter.process(&mut context, request) {
                Ok(()) => {
                    let filter_time = filter_start.elapsed();
                    info!("✅ 过滤器 '{}' 执行成功，耗时: {}ms", filter.name(), filter_time.as_millis());
                    context.processing_chain.push(filter.name().to_string());
                }
                Err(e) => {
                    let filter_time = filter_start.elapsed();
                    error!("❌ 过滤器 '{}' 执行失败: {}, 耗时: {}ms", filter.name(), e, filter_time.as_millis());
                    context.add_error(format!("过滤器 '{}' 失败: {}", filter.name(), e));

                    // 根据错误策略决定是否继续
                    // 目前选择继续执行，可以配置为遇到错误就停止
                    continue;
                }
            }
        }

        let total_time = start_time.elapsed();
        info!("✅ 插件链 '{}' 执行完成，总耗时: {}ms", self.name, total_time.as_millis());
        info!("📊 处理统计: {} 个过滤器，{} 条日志，{} 个错误",
              context.processing_chain.len(), context.current_lines.len(), context.errors.len());

        // 构建最终结果
        Ok(ParseResult {
            lines: context.current_lines,
            total_lines: content.lines().count(),
            detected_format: Some(self.name.clone()),
            parsing_errors: context.errors,
        })
    }
}

/// 插件链管理器
///
/// 管理多个插件链，根据日志内容智能选择最适合的链进行处理。
/// 支持链的注册、选择、执行和配置管理。
///
/// # 功能特性
/// - **智能链选择**：基于内容特征自动选择最佳处理链
/// - **多链管理**：支持注册和管理多个处理链
/// - **配置驱动**：支持运行时配置和动态调整
/// - **性能监控**：提供详细的执行统计信息
/// - **错误恢复**：支持链执行失败时的回退策略
pub struct PluginChainManager {
    /// 注册的插件链列表
    chains: HashMap<String, PluginChain>,

    /// 默认链名称（当没有匹配链时使用）
    default_chain: Option<String>,

    /// 是否启用智能链选择
    smart_selection: bool,
}

impl PluginChainManager {
    /// 创建新的插件链管理器
    ///
    /// # Returns
    /// - `Self`: 新创建的管理器实例
    pub fn new() -> Self {
        Self {
            chains: HashMap::new(),
            default_chain: None,
            smart_selection: true,
        }
    }

    /// 注册插件链
    ///
    /// # 参数
    /// - `chain`: 要注册的插件链
    pub fn register_chain(&mut self, chain: PluginChain) {
        let name = chain.name.clone();
        self.chains.insert(name, chain);
    }

    /// 设置默认链
    ///
    /// # 参数
    /// - `chain_name`: 默认链的名称
    pub fn set_default_chain(&mut self, chain_name: String) {
        self.default_chain = Some(chain_name);
    }

    /// 智能选择最佳处理链
    ///
    /// 基于日志内容特征和文件路径信息选择最适合的处理链。
    ///
    /// # 参数
    /// - `content`: 日志内容
    /// - `file_path`: 文件路径（可选）
    ///
    /// # Returns
    /// - `Option<&PluginChain>`: 选择的链引用，如果没有匹配的则返回None
    pub fn select_best_chain(&self, content: &str, file_path: Option<&str>) -> Option<&PluginChain> {
        if !self.smart_selection {
            // 如果禁用智能选择，返回默认链
            return self.default_chain.as_ref().and_then(|name| self.chains.get(name));
        }

        // 优先检测Docker JSON格式（最高优先级）
        let content_lower = content.to_lowercase();
        if content_lower.contains("{") &&
           content_lower.contains("\"log\"") &&
           content_lower.contains("\"stream\"") {
            info!("🐳 检测到Docker JSON格式，优先选择Docker链");
            return self.chains.get("docker");
        }

        // 计算每个链的匹配度
        let mut best_chain = None;
        let mut best_score = 0.0;

        for (name, chain) in &self.chains {
            if !chain.enabled {
                continue;
            }

            let score = self.calculate_chain_score(chain, content, file_path);
            debug!("🔍 链 '{}' 匹配度: {:.2}", name, score);

            if score > best_score {
                best_score = score;
                best_chain = Some(chain);
            }
        }

        // 如果没有链匹配或匹配度太低，使用默认链
        if best_score < 0.3 {
            if let Some(default_name) = &self.default_chain {
                info!("⚠️ 没有找到高匹配度的链，使用默认链: {}", default_name);
                return self.chains.get(default_name);
            }
        }

        best_chain
    }

    /// 计算链与内容的匹配度
    ///
    /// # 参数
    /// - `chain`: 插件链引用
    /// - `content`: 日志内容
    /// - `file_path`: 文件路径（可选）
    ///
    /// # Returns
    /// - `f32`: 匹配度分数（0.0 - 1.0）
    fn calculate_chain_score(&self, chain: &PluginChain, content: &str, file_path: Option<&str>) -> f32 {
        let mut score = 0.0;
        let total_filters = chain.filters.len();

        if total_filters == 0 {
            return 0.0;
        }

        // 计算能处理此内容的过滤器比例
        let mut can_handle_count = 0;
        for filter in &chain.filters {
            if filter.can_handle(content, file_path) {
                can_handle_count += 1;
            }
        }

        score += (can_handle_count as f32) / (total_filters as f32);

        // 如果有条件，检查条件匹配度
        if let Some(conditions) = &chain.conditions {
            if conditions.matches(content, file_path) {
                score += 0.4; // 增加条件匹配加分，确保特定链优先于通用链
                debug!("✅ 链 '{}' 条件匹配加分，当前分数: {:.2}", chain.name, score);
            } else {
                debug!("❌ 链 '{}' 条件不匹配", chain.name);
                // 如果条件不匹配，大幅降低分数
                score *= 0.3;
                debug!("🔻 链 '{}' 条件不匹配，分数降至: {:.2}", chain.name, score);
            }
        } else {
            // 通用链没有条件，给予轻微惩罚，让特定链有优先权
            score *= 0.9;
            debug!("🔧 通用链 '{}' 分数调整: {:.2}", chain.name, score);
        }

        // 文件路径匹配加分
        if let Some(path) = file_path {
            for filter in &chain.filters {
                if filter.can_handle(content, Some(path)) {
                    score += 0.1;
                    break;
                }
            }
        }

        // 确保分数在0-1范围内
        score.min(1.0)
    }

    /// 处理日志内容
    ///
    /// 自动选择最佳链并执行处理。
    ///
    /// # 参数
    /// - `content`: 要处理的日志内容
    /// - `request`: 解析请求参数
    ///
    /// # Returns
    /// - `Result<ParseResult, String>`: 处理结果或错误信息
    pub fn process(&self, content: &str, request: &ParseRequest) -> Result<ParseResult, String> {
        // 选择最佳处理链
        let chain = self.select_best_chain(content, request.file_path.as_deref())
            .ok_or_else(|| "没有找到合适的处理链".to_string())?;

        info!("🎯 选择处理链: {}", chain.name);
        chain.process(content, request)
    }

    /// 获取所有已注册的链信息
    ///
    /// # Returns
    /// - `Vec<String>`: 所有链的名称列表
    pub fn get_available_chains(&self) -> Vec<String> {
        self.chains.keys().cloned().collect()
    }

    /// 启用或禁用智能链选择
    ///
    /// # 参数
    /// - `enabled`: 是否启用智能选择
    #[allow(dead_code)]
    pub fn set_smart_selection(&mut self, enabled: bool) {
        self.smart_selection = enabled;
    }
}

impl Default for PluginChainManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PluginChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginChain")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("filters_count", &self.filters.len())
            .field("enabled", &self.enabled)
            .field("conditions", &self.conditions.is_some())
            .finish()
    }
}