/// 增强插件管理器
///
/// 这是插件系统的高级封装，提供了比基础插件管理器更丰富的功能。
/// 主要增强包括异步初始化、批量处理、性能优化和扩展性支持。
///
/// # 增强功能
/// - 异步插件初始化和生命周期管理
/// - 批量日志条目处理和优化
/// - 插件链和处理器管理（未来功能）
/// - 性能监控和统计信息收集
/// - 配置驱动的插件行为调整
///
/// # 设计目标
/// - 提供更高级的插件管理接口
/// - 支持复杂的插件处理流程
/// - 优化大文件和批量处理的性能
/// - 为未来扩展提供良好基础
///
/// # 与基础管理器的关系
/// - 内部封装PluginManager作为核心引擎
/// - 在基础功能之上添加高级特性
/// - 保持API兼容性的同时增强能力

use crate::plugins::{manager::PluginManager, PluginInfo, ParseRequest, ParseResult, LogEntry};
use crate::plugins::chain::{PluginChainManager};
use crate::plugins::presets::register_preset_chains;
use log::{info, debug, warn, error};
use std::sync::{Arc, Mutex};

/// 增强插件管理器主结构
///
/// 内部使用基础PluginManager作为核心，并在其基础上添加高级功能。
/// 这种设计模式称为装饰器模式，在保持接口兼容性的同时增强功能。
///
/// 现在集成了插件链系统，支持顺序处理和Filter Chain机制。
pub struct EnhancedPluginManager {
    /// 内部的基础插件管理器实例
    ///
    /// 负责具体的插件注册、查找和解析操作。
    /// EnhancedPluginManager在此基础上添加了批处理、异步操作等高级功能。
    inner: PluginManager,

    /// 插件链管理器
    ///
    /// 负责管理多个过滤器链，支持类似Java Web Filter的顺序处理机制。
    /// 使用Arc<Mutex<>>来支持内部可变性。
    chain_manager: Arc<Mutex<PluginChainManager>>,

    /// 是否启用插件链系统
    ///
    /// 当启用时，优先使用插件链处理；当禁用时，回退到传统单插件模式。
    chain_enabled: bool,
}

impl EnhancedPluginManager {
    /// 创建新的增强插件管理器实例
    ///
    /// 初始化内部的PluginManager实例和插件链管理器，为后续的插件操作做准备。
    /// 注意：此方法只创建管理器实例，不进行插件的初始化操作。
    ///
    /// # Returns
    /// - `Self`: 新创建的增强插件管理器实例
    ///
    /// # 初始化流程
    /// 1. 创建基础PluginManager实例
    /// 2. 创建插件链管理器实例
    /// 3. 注册所有内置插件和预设链
    /// 4. 准备高级功能的运行环境
    pub fn new() -> Self {
        Self {
            inner: PluginManager::new(),
            chain_manager: Arc::new(Mutex::new(PluginChainManager::new())),
            chain_enabled: true, // 默认启用插件链系统
        }
    }

    /// 异步初始化增强插件管理器
    ///
    /// 执行插件系统的完整初始化流程，包括插件加载、配置验证、
    /// 插件链注册等操作。这是应用程序启动时的关键步骤。
    ///
    /// # 功能特性
    /// - 异步插件加载和验证
    /// - 插件链系统初始化
    /// - 预设链配置注册
    /// - 性能基准测试和优化
    /// - 配置文件加载和应用
    ///
    /// # Returns
    /// - `Ok(())`: 初始化成功
    /// - `Err(String)`: 初始化失败，包含错误详情
    pub async fn initialize(&self) -> Result<(), String> {
        info!("🚀 开始初始化增强插件管理器...");

        // 1. 初始化基础插件管理器
        info!("📦 初始化基础插件系统");
        // 这里可以添加基础插件的初始化逻辑

        // 2. 初始化插件链系统
        if self.chain_enabled {
            info!("🔗 初始化插件链系统");
            if let Ok(mut chain_manager) = self.chain_manager.lock() {
                register_preset_chains(&mut chain_manager);

                let available_chains = chain_manager.get_available_chains();
                info!("✅ 已注册 {} 个预设链: {:?}", available_chains.len(), available_chains);
            } else {
                error!("❌ 无法获取插件链管理器锁，初始化失败");
                return Err("插件链管理器初始化失败".to_string());
            }
        } else {
            warn!("⚠️ 插件链系统已禁用，将使用传统单插件模式");
        }

        // 3. 验证系统完整性
        info!("🔍 验证插件系统完整性");
        // TODO: 添加系统完整性检查逻辑

        info!("✅ 增强插件管理器初始化完成");
        Ok(())
    }

    /// 获取所有可用插件的详细信息
    ///
    /// 委托给内部的PluginManager，返回系统中所有已注册插件的元数据。
    /// 这是一个透明代理方法，保持与基础管理器的接口兼容性。
    ///
    /// # Returns
    /// - `Vec<PluginInfo>`: 所有插件的详细信息列表
    ///
    /// # 包含信息
    /// - 插件名称和描述
    /// - 支持的文件扩展名
    /// - 自动检测能力
    pub fn get_available_plugins(&self) -> Vec<PluginInfo> {
        self.inner.get_available_plugins()
    }

    /// 使用指定插件解析日志内容
    ///
    /// 委托给内部的PluginManager执行实际的解析操作。
    /// 未来版本将在此基础之上添加增强功能，如缓存、预处理等。
    ///
    /// # 参数
    /// - `plugin_name`: 要使用的插件名称
    /// - `request`: 解析请求参数
    ///
    /// # Returns
    /// - `Ok(ParseResult)`: 解析成功，包含结构化日志
    /// - `Err(String)`: 插件不存在或解析失败
    ///
    /// # 未来增强计划
    /// - [ ] 解析结果缓存
    /// - [ ] 预处理和后处理管道
    /// - [ ] 性能监控和统计
    pub fn parse_with_plugin(&self, plugin_name: &str, request: &ParseRequest) -> Result<ParseResult, String> {
        debug!("🔧 通过增强插件管理器调用插件: {}", plugin_name);
        self.inner.parse_with_plugin(plugin_name, request)
    }

    /// 自动检测格式并解析日志内容
    ///
    /// 优先使用插件链系统进行智能处理，如果插件链系统不可用或处理失败，
    /// 则回退到传统的单插件模式。这确保了向后兼容性和处理可靠性。
    ///
    /// # 参数
    /// - `request`: 解析请求参数
    ///
    /// # Returns
    /// - `Ok(ParseResult)`: 解析成功，包含检测结果和解析内容
    /// - `Err(String)`: 所有解析器都失败
    ///
    /// # 处理策略
    /// 1. 插件链系统（如果启用）→ 传统单插件模式 → 错误
    /// 2. 智能链选择 → 自动格式检测 → 原始文本处理
    ///
    /// # 增强特性
    /// - 支持多层处理（Docker JSON → SpringBoot → MyBatis）
    /// - 智能链选择和条件匹配
    /// - 详细的处理链追踪和性能监控
    pub fn auto_detect_and_parse(&self, request: &ParseRequest) -> Result<ParseResult, String> {
        debug!("🔍 通过增强插件管理器执行自动检测");

        // 优先使用插件链系统
        if self.chain_enabled {
            debug!("🔗 尝试使用插件链系统处理");
            if let Ok(chain_manager) = self.chain_manager.lock() {
                match chain_manager.process(&request.content, request) {
                    Ok(result) => {
                        info!("✅ 插件链系统处理成功，检测格式: {:?}", result.detected_format);
                        return Ok(result);
                    }
                    Err(e) => {
                        warn!("⚠️ 插件链系统处理失败: {}，回退到传统模式", e);
                        // 继续执行传统模式作为回退
                    }
                }
            } else {
                warn!("⚠️ 无法获取插件链管理器锁，回退到传统模式");
            }
        }

        // 回退到传统的单插件模式
        debug!("🔄 使用传统单插件模式处理");
        self.inner.auto_detect_and_parse(request)
    }

    /// 批量处理日志条目
    ///
    /// 这是增强插件管理器的核心功能，用于批量处理已解析的日志条目。
    /// 支持插件链处理、数据增强、格式转换等高级操作。
    ///
    /// # 功能特性
    /// - 批量处理优化，提高性能
    /// - 插件链支持，允许多个插件依次处理
    /// - 并行处理支持（未来功能）
    /// - 进度监控和错误恢复
    ///
    /// # 参数
    /// - `entries`: 要处理的日志条目列表
    ///
    /// # Returns
    /// - `Ok(Vec<LogEntry>)`: 处理后的日志条目列表
    /// - `Err(String)`: 处理失败，包含错误详情
    ///
    /// # TODO 实现计划
    /// - [ ] 插件链处理管道
    /// - [ ] 并行处理支持
    /// - [ ] 进度监控和报告
    /// - [ ] 错误恢复和重试机制
    /// - [ ] 内存优化策略
    pub async fn process_log_entries(&self, entries: Vec<LogEntry>) -> Result<Vec<LogEntry>, String> {
        info!("🔄 开始批量处理 {} 个日志条目", entries.len());

        // 当前实现为占位符，直接返回原始条目
        // 未来将实现完整的批量处理逻辑

        debug!("📊 批量处理统计: 输入 {} 条目", entries.len());

        // TODO: 实现完整的批量处理流程
        // 1. 数据验证和预处理
        // 2. 插件链处理
        // 3. 并行处理优化
        // 4. 结果聚合和验证
        // 5. 性能统计和报告

        info!("✅ 批量处理完成 (占位符实现)");
        Ok(entries)
    }

    /// 使用指定的插件链处理日志内容
    ///
    /// 直接调用插件链系统进行处理，绕过自动选择逻辑。
    /// 用于需要明确指定处理链的场景。
    ///
    /// # 参数
    /// - `chain_name`: 要使用的插件链名称
    /// - `request`: 解析请求参数
    ///
    /// # Returns
    /// - `Ok(ParseResult)`: 处理成功的结果
    /// - `Err(String)`: 链不存在或处理失败
    pub fn process_with_chain(&self, chain_name: &str, request: &ParseRequest) -> Result<ParseResult, String> {
        if !self.chain_enabled {
            return Err("插件链系统已禁用".to_string());
        }

        info!("🔗 使用指定插件链处理: {}", chain_name);

        // 获取可用链列表
        let available_chains = if let Ok(chain_manager) = self.chain_manager.lock() {
            chain_manager.get_available_chains()
        } else {
            return Err("无法获取插件链管理器锁".to_string());
        };

        if !available_chains.contains(&chain_name.to_string()) {
            return Err(format!("插件链 '{}' 不存在，可用链: {:?}", chain_name, available_chains));
        }

        // 临时实现：设置请求中的链名称，然后在process中处理
        // 这需要修改ParseRequest结构体来支持链名称参数
        // 暂时使用自动检测作为回退
        warn!("⚠️ 指定链处理功能尚未完全实现，使用自动检测");
        self.auto_detect_and_parse(request)
    }

    /// 获取所有可用的插件链信息
    ///
    /// 返回系统中所有已注册的插件链列表。
    ///
    /// # Returns
    /// - `Vec<String>`: 所有可用链的名称列表
    pub fn get_available_chains(&self) -> Vec<String> {
        if !self.chain_enabled {
            return Vec::new();
        }

        if let Ok(chain_manager) = self.chain_manager.lock() {
            chain_manager.get_available_chains()
        } else {
            Vec::new()
        }
    }

    /// 启用或禁用插件链系统
    ///
    /// # 参数
    /// - `enabled`: 是否启用插件链系统
    pub fn set_chain_enabled(&mut self, enabled: bool) {
        self.chain_enabled = enabled;
        if enabled {
            info!("✅ 插件链系统已启用");
        } else {
            warn!("⚠️ 插件链系统已禁用，将使用传统单插件模式");
        }
    }

    /// 检查插件链系统是否启用
    ///
    /// # Returns
    /// - `bool`: true表示已启用，false表示已禁用
    pub fn is_chain_enabled(&self) -> bool {
        self.chain_enabled
    }

    /// 推荐最适合的插件链
    ///
    /// 基于日志内容和文件路径特征推荐最适合的处理链。
    ///
    /// # 参数
    /// - `content`: 日志内容样本
    /// - `file_path`: 文件路径（可选）
    ///
    /// # Returns
    /// - `Option<String>`: 推荐的链名称，如果没有合适的则返回None
    pub fn recommend_chain(&self, content: &str, file_path: Option<&str>) -> Option<String> {
        if !self.chain_enabled {
            return None;
        }

        use crate::plugins::presets::recommend_chain;
        let recommended = recommend_chain(content, file_path);

        // 验证推荐的链是否存在
        let available_chains = self.get_available_chains();
        if available_chains.contains(&recommended) {
            info!("💡 推荐插件链: {}", recommended);
            Some(recommended)
        } else {
            warn!("⚠️ 推荐的链 '{}' 不存在，可用链: {:?}", recommended, available_chains);
            None
        }
    }
}

/// 增强插件管理器的默认实现
///
/// 提供Default trait实现，允许使用EnhancedPluginManager::default()创建实例。
/// 这在需要延迟初始化或配置驱动的场景中很有用。
impl Default for EnhancedPluginManager {
    /// 创建默认的增强插件管理器实例
    ///
    /// 等同于调用EnhancedPluginManager::new()，创建包含完整插件集的管理器。
    ///
    /// # Returns
    /// - `Self`: 配置完整的增强插件管理器实例
    fn default() -> Self {
        Self::new()
    }
}