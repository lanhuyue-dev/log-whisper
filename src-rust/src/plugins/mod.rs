//! 插件系统核心模块
//! 
//! 提供可扩展的插件架构，支持多种日志格式的解析和渲染

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 插件系统核心结构
pub mod core;
/// 插件注册中心
pub mod registry;
/// 插件执行引擎
pub mod engine;
/// 内置插件实现
pub mod builtin;
/// 插件配置管理
pub mod config;
/// SpringBoot 日志处理增强
pub mod springboot;
/// 渲染器插件
pub mod renderers;
/// 插件系统测试
#[cfg(test)]
pub mod test;

/// 插件接口定义
pub trait Plugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;
    
    /// 插件版本
    fn version(&self) -> &str;
    
    /// 插件描述
    fn description(&self) -> &str;
    
    /// 插件优先级（数值越小优先级越高）
    fn priority(&self) -> u32;
    
    /// 插件类型
    fn plugin_type(&self) -> PluginType;
    
    /// 检查是否支持该日志格式
    fn can_handle(&self, log_line: &str) -> bool;
    
    /// 处理日志条目
    fn process(&self, log_entry: &mut LogEntry) -> Result<(), PluginError>;
    
    /// 插件初始化
    fn initialize(&mut self) -> Result<(), PluginError>;
    
    /// 插件清理
    fn cleanup(&self) -> Result<(), PluginError>;
}

/// 插件类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    /// 解析器：负责解析日志格式
    Parser,
    /// 渲染器：负责美化显示
    Renderer,
    /// 过滤器：负责过滤和聚合
    Filter,
    /// 分析器：负责日志分析
    Analyzer,
    /// 关联器：负责日志关联
    Correlator,
}

/// 日志条目结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 行号
    pub line_number: usize,
    /// 原始内容
    pub content: String,
    /// 时间戳
    pub timestamp: Option<String>,
    /// 日志级别
    pub level: Option<String>,
    /// 格式化后的内容
    pub formatted_content: Option<String>,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 插件处理标记
    pub processed_by: Vec<String>,
}

/// 插件错误类型
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("插件初始化失败: {0}")]
    InitializationFailed(String),
    
    #[error("插件处理失败: {0}")]
    ProcessingFailed(String),
    
    #[error("插件配置错误: {0}")]
    ConfigurationError(String),
    
    #[error("插件依赖缺失: {0}")]
    DependencyMissing(String),
    
    #[error("插件版本不兼容: {0}")]
    VersionIncompatible(String),
}

/// 插件管理器
pub struct PluginManager {
    registry: Arc<RwLock<registry::PluginRegistry>>,
    engine: Arc<RwLock<engine::PluginEngine>>,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(registry::PluginRegistry::new())),
            engine: Arc::new(RwLock::new(engine::PluginEngine::new())),
        }
    }
    
    /// 注册插件
    pub async fn register_plugin(&self, plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let mut registry = self.registry.write().await;
        registry.register(plugin).await
    }
    
    /// 处理日志条目
    pub async fn process_log_entry(&self, mut log_entry: LogEntry) -> Result<LogEntry, PluginError> {
        let engine = self.engine.read().await;
        engine.process_entry(&mut log_entry).await?;
        Ok(log_entry)
    }
    
    /// 获取可用插件列表
    pub async fn get_available_plugins(&self) -> Vec<PluginInfo> {
        let registry = self.registry.read().await;
        registry.get_plugin_list()
    }
}

/// 插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub plugin_type: PluginType,
    pub priority: u32,
    pub enabled: bool,
}

impl Default for LogEntry {
    fn default() -> Self {
        Self {
            line_number: 0,
            content: String::new(),
            timestamp: None,
            level: None,
            formatted_content: None,
            metadata: HashMap::new(),
            processed_by: Vec::new(),
        }
    }
}

impl LogEntry {
    /// 创建新的日志条目
    pub fn new(line_number: usize, content: String) -> Self {
        Self {
            line_number,
            content,
            timestamp: None,
            level: None,
            formatted_content: None,
            metadata: HashMap::new(),
            processed_by: Vec::new(),
        }
    }
    
    /// 添加元数据
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// 标记为已处理
    pub fn mark_processed(&mut self, plugin_name: String) {
        self.processed_by.push(plugin_name);
    }
    
    /// 检查是否已被处理
    pub fn is_processed_by(&self, plugin_name: &str) -> bool {
        self.processed_by.contains(&plugin_name.to_string())
    }
}
