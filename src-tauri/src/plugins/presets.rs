/// 预定义插件链配置
///
/// 提供常用场景的预配置插件链，方便直接使用和参考。
/// 每个预设都针对特定的日志处理场景进行了优化配置。
///
/// # 支持的预设链
/// - **Docker容器日志链**: 专门处理Docker容器输出的复合日志
/// - **SpringBoot应用链**: 处理标准SpringBoot应用日志
/// - **通用文本链**: 处理普通文本格式日志
/// - **微服务链**: 处理微服务架构中的复杂日志格式
/// - **数据库链**: 专门处理数据库相关的SQL日志
///
/// # 使用方式
/// ```rust
/// let chain_manager = PluginChainManager::new();
/// register_preset_chains(&mut chain_manager);
/// let result = chain_manager.process(content, &request);
/// ```

use crate::plugins::chain::{PluginChain, ChainConditions, PluginChainManager};
use crate::plugins::filters::{
    DockerJsonFilter, SpringBootFilter, MyBatisFilter, JavaLogFilter,
    JsonStructureFilter, ContentEnhancerFilter
};
use std::sync::Arc;
use log::info;

/// 注册所有预定义的插件链
///
/// 创建并注册各种常用场景的插件链配置。
///
/// # 参数
/// - `manager`: 插件链管理器的可变引用
pub fn register_preset_chains(manager: &mut PluginChainManager) {
    info!("🔧 注册预定义插件链...");

    // Docker容器日志处理链
    register_docker_chain(manager);

    // SpringBoot应用日志处理链
    register_springboot_chain(manager);

    // 通用文本日志处理链
    register_generic_chain(manager);

    // 微服务架构日志处理链
    register_microservice_chain(manager);

    // 数据库SQL日志处理链
    register_database_chain(manager);

    // 设置默认链
    manager.set_default_chain("generic".to_string());

    info!("✅ 预定义插件链注册完成");
}

/// Docker容器日志处理链
///
/// 专门处理Docker容器输出的复合日志，典型的处理流程：
/// 1. Docker JSON格式解析 → 提取log内容
/// 2. SpringBoot格式解析 → 提取应用日志信息
/// 3. MyBatis SQL解析 → 识别和格式化SQL语句
/// 4. 内容增强 → 添加URL、邮箱等识别
/// 5. JSON结构化 → 统一输出格式
///
/// # 适用场景
/// - SpringBoot应用的Docker容器日志
/// - 包含SQL日志的微服务日志
/// - 需要多层解析的容器化应用日志
fn register_docker_chain(manager: &mut PluginChainManager) {
    let mut chain = PluginChain::new(
        "docker".to_string(),
        "Docker容器日志处理链，支持JSON格式解析和多层日志处理".to_string(),
    );

    // 设置执行条件
    let mut conditions = ChainConditions::new();
    conditions.file_patterns.push("docker".to_string());
    conditions.file_patterns.push("container".to_string());
    conditions.content_patterns.push("\"log\"".to_string()); // JSON格式特征
    conditions.content_patterns.push("\"stream\"".to_string()); // Docker特有字段
    conditions.content_patterns.push("\"time\"".to_string()); // Docker时间戳
    conditions.min_confidence = 0.7; // 提高置信度阈值，确保优先选择
    chain.conditions = Some(conditions);

    // 添加过滤器（按优先级顺序）
    chain.add_filter(Arc::new(DockerJsonFilter));
    chain.add_filter(Arc::new(SpringBootFilter));
    chain.add_filter(Arc::new(JavaLogFilter));
    chain.add_filter(Arc::new(MyBatisFilter));
    chain.add_filter(Arc::new(ContentEnhancerFilter));
    chain.add_filter(Arc::new(JsonStructureFilter));

    manager.register_chain(chain);
    info!("✅ 注册Docker容器日志链");
}

/// SpringBoot应用日志处理链
///
/// 专门处理SpringBoot应用的标准日志格式。
///
/// # 处理流程
/// 1. SpringBoot格式解析 → 提取时间戳、级别、线程等信息
/// 2. MyBatis SQL解析 → 识别SQL相关日志
/// 3. 内容增强 → 添加错误标记和链接识别
/// 4. JSON结构化 → 统一输出格式
///
/// # 适用场景
/// - SpringBoot应用的标准日志文件
/// - 直接运行的Java应用日志
/// - 不包含Docker JSON包装的应用日志
fn register_springboot_chain(manager: &mut PluginChainManager) {
    let mut chain = PluginChain::new(
        "springboot".to_string(),
        "SpringBoot应用日志处理链，专门处理SpringBoot标准日志格式".to_string(),
    );

    // 设置执行条件
    let mut conditions = ChainConditions::new();
    conditions.content_patterns.push("spring".to_string());
    conditions.content_patterns.push("application.start".to_string());
    conditions.content_patterns.push("INFO".to_string());
    conditions.content_patterns.push("ERROR".to_string());
    conditions.min_confidence = 0.5;
    chain.conditions = Some(conditions);

    // 添加过滤器
    chain.add_filter(Arc::new(SpringBootFilter));
    chain.add_filter(Arc::new(JavaLogFilter));
    chain.add_filter(Arc::new(MyBatisFilter));
    chain.add_filter(Arc::new(ContentEnhancerFilter));
    chain.add_filter(Arc::new(JsonStructureFilter));

    manager.register_chain(chain);
    info!("✅ 注册SpringBoot应用日志链");
}

/// 通用文本日志处理链
///
/// 处理普通的文本格式日志，提供基础的解析和格式化功能。
///
/// # 处理流程
/// 1. 内容增强 → 识别URL、邮箱等信息
/// 2. JSON结构化 → 统一输出格式
///
/// # 适用场景
/// - 普通的文本日志文件
/// - 不符合特定格式的应用日志
/// - 系统日志和工具输出
/// - 作为回退处理链
fn register_generic_chain(manager: &mut PluginChainManager) {
    let mut chain = PluginChain::new(
        "generic".to_string(),
        "通用文本日志处理链，提供基础的日志解析和格式化功能".to_string(),
    );

    // 通用链不设置特定条件，作为默认回退
    chain.conditions = None;

    // 添加过滤器
    chain.add_filter(Arc::new(JavaLogFilter));
    chain.add_filter(Arc::new(ContentEnhancerFilter));
    chain.add_filter(Arc::new(JsonStructureFilter));

    manager.register_chain(chain);
    info!("✅ 注册通用文本日志链");
}

/// 微服务架构日志处理链
///
/// 专门处理微服务架构中的复杂日志格式，包含分布式追踪信息。
///
/// # 处理流程
/// 1. SpringBoot格式解析 → 处理应用日志基础格式
/// 2. MyBatis SQL解析 → 识别数据库操作日志
/// 3. 内容增强 → 识别追踪ID、请求ID等微服务特有信息
/// 4. JSON结构化 → 统一输出格式
///
/// # 适用场景
/// - 微服务架构的应用日志
/// - 包含分布式追踪的日志
/// - 多个服务调用的复合日志
fn register_microservice_chain(manager: &mut PluginChainManager) {
    let mut chain = PluginChain::new(
        "microservice".to_string(),
        "微服务架构日志处理链，支持分布式追踪和多服务日志处理".to_string(),
    );

    // 设置执行条件
    let mut conditions = ChainConditions::new();
    conditions.content_patterns.push("trace".to_string());
    conditions.content_patterns.push("span".to_string());
    conditions.content_patterns.push("request".to_string());
    conditions.content_patterns.push("service".to_string());
    conditions.min_confidence = 0.4;
    chain.conditions = Some(conditions);

    // 添加过滤器
    chain.add_filter(Arc::new(SpringBootFilter));
    chain.add_filter(Arc::new(JavaLogFilter));
    chain.add_filter(Arc::new(MyBatisFilter));
    chain.add_filter(Arc::new(ContentEnhancerFilter));
    chain.add_filter(Arc::new(JsonStructureFilter));

    manager.register_chain(chain);
    info!("✅ 注册微服务架构日志链");
}

/// 数据库SQL日志处理链
///
/// 专门处理数据库相关的SQL日志，提供详细的SQL语句格式化。
///
/// # 处理流程
/// 1. MyBatis SQL解析 → 主要的SQL解析和格式化
/// 2. SpringBoot格式解析 → 处理包含SQL的应用日志
/// 3. 内容增强 → 识别SQL类型和性能指标
/// 4. JSON结构化 → 统一输出格式
///
/// # 适用场景
/// - MyBatis/MyBatis-Plus的SQL日志
/// - 数据库连接池日志
/// - ORM框架的SQL输出日志
/// - 数据库性能分析日志
fn register_database_chain(manager: &mut PluginChainManager) {
    let mut chain = PluginChain::new(
        "database".to_string(),
        "数据库SQL日志处理链，专门处理MyBatis等ORM框架的SQL日志".to_string(),
    );

    // 设置执行条件
    let mut conditions = ChainConditions::new();
    conditions.content_patterns.push("preparing".to_string());
    conditions.content_patterns.push("parameters".to_string());
    conditions.content_patterns.push("==>".to_string());
    conditions.content_patterns.push("sql".to_string());
    conditions.min_confidence = 0.7;
    chain.conditions = Some(conditions);

    // 添加过滤器（MyBatis过滤器优先级更高）
    chain.add_filter(Arc::new(MyBatisFilter));
    chain.add_filter(Arc::new(SpringBootFilter));
    chain.add_filter(Arc::new(JavaLogFilter));
    chain.add_filter(Arc::new(ContentEnhancerFilter));
    chain.add_filter(Arc::new(JsonStructureFilter));

    manager.register_chain(chain);
    info!("✅ 注册数据库SQL日志链");
}

/// 自定义链构建器
///
/// 提供便捷的API来构建自定义的插件链。
pub struct ChainBuilder {
    chain: PluginChain,
}

impl ChainBuilder {
    /// 创建新的链构建器
    ///
    /// # 参数
    /// - `name`: 链的名称
    /// - `description`: 链的描述
    ///
    /// # Returns
    /// - `Self`: 新的构建器实例
    pub fn new(name: String, description: String) -> Self {
        Self {
            chain: PluginChain::new(name, description),
        }
    }

    /// 添加Docker JSON过滤器
    pub fn with_docker_json(mut self) -> Self {
        self.chain.add_filter(Arc::new(DockerJsonFilter));
        self
    }

    /// 添加SpringBoot过滤器
    pub fn with_springboot(mut self) -> Self {
        self.chain.add_filter(Arc::new(SpringBootFilter));
        self
    }

    /// 添加MyBatis过滤器
    pub fn with_mybatis(mut self) -> Self {
        self.chain.add_filter(Arc::new(MyBatisFilter));
        self
    }

    /// 添加Java日志过滤器
    pub fn with_java_log(mut self) -> Self {
        self.chain.add_filter(Arc::new(JavaLogFilter));
        self
    }

    /// 添加内容增强过滤器
    pub fn with_content_enhancer(mut self) -> Self {
        self.chain.add_filter(Arc::new(ContentEnhancerFilter));
        self
    }

    /// 添加JSON结构化过滤器
    pub fn with_json_structure(mut self) -> Self {
        self.chain.add_filter(Arc::new(JsonStructureFilter));
        self
    }

    /// 设置执行条件
    pub fn with_conditions(mut self, conditions: ChainConditions) -> Self {
        self.chain.conditions = Some(conditions);
        self
    }

    /// 设置是否启用
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.chain.enabled = enabled;
        self
    }

    /// 构建最终的插件链
    ///
    /// # Returns
    /// - `PluginChain`: 构建完成的插件链
    pub fn build(self) -> PluginChain {
        self.chain
    }
}

/// 获取推荐的链配置
///
/// 根据日志内容和文件路径特征推荐最适合的链配置。
///
/// # 参数
/// - `content`: 日志内容样本
/// - `file_path`: 文件路径（可选）
///
/// # Returns
/// - `String`: 推荐的链名称
pub fn recommend_chain(content: &str, file_path: Option<&str>) -> String {
    let content_lower = content.to_lowercase();
    let _path_lower = file_path.unwrap_or("").to_lowercase();

    // Docker容器日志特征
    if content_lower.contains("{") &&
       content_lower.contains("\"log\"") &&
       content_lower.contains("\"stream\"") {
        return "docker".to_string();
    }

    // 数据库SQL日志特征
    if content_lower.contains("preparing:") ||
       content_lower.contains("parameters:") ||
       content_lower.contains("==>") {
        return "database".to_string();
    }

    // SpringBoot日志特征
    if content_lower.contains("spring") ||
       content_lower.contains("application.start") ||
       content_lower.contains("springframework") {
        return "springboot".to_string();
    }

    // 微服务日志特征
    if content_lower.contains("trace") ||
       content_lower.contains("span") ||
       content_lower.contains("request") ||
       content_lower.contains("service") {
        return "microservice".to_string();
    }

    // 默认返回通用链
    "generic".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommend_docker_chain() {
        let content = r#"{"log": "2024-01-15 10:30:25.123 [main] INFO Application started", "stream": "stdout", "time": "2024-01-15T10:30:25.123Z"}"#;
        let recommended = recommend_chain(content, None);
        assert_eq!(recommended, "docker");
    }

    #[test]
    fn test_recommend_springboot_chain() {
        let content = r#"2024-01-15 10:30:25.123 [main] INFO com.example.Application - Application started"#;
        let recommended = recommend_chain(content, Some("application.log"));
        assert_eq!(recommended, "springboot");
    }

    #[test]
    fn test_recommend_database_chain() {
        let content = r#"DEBUG - ==>  Preparing: SELECT * FROM users WHERE id = ?
DEBUG - ==> Parameters: 123(String)"#;
        let recommended = recommend_chain(content, None);
        assert_eq!(recommended, "database");
    }

    #[test]
    fn test_recommend_generic_chain() {
        let content = r#"This is a simple log message
Another log line here"#;
        let recommended = recommend_chain(content, None);
        assert_eq!(recommended, "generic");
    }

    #[test]
    fn test_chain_builder() {
        let chain = ChainBuilder::new(
            "test".to_string(),
            "Test chain".to_string(),
        )
        .with_docker_json()
        .with_springboot()
        .with_json_structure()
        .with_enabled(true)
        .build();

        assert_eq!(chain.name, "test");
        assert_eq!(chain.filters.len(), 3);
        assert!(chain.enabled);
    }
}