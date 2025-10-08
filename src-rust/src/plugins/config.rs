//! 插件配置管理
//! 
//! 负责插件系统的配置管理和持久化

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 插件配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSystemConfig {
    /// 是否启用插件系统
    pub enabled: bool,
    /// 插件目录
    pub plugin_directory: String,
    /// 最大插件数量
    pub max_plugins: usize,
    /// 插件执行超时时间（毫秒）
    pub execution_timeout_ms: u64,
    /// 是否启用插件统计
    pub enable_stats: bool,
    /// 插件配置
    pub plugins: HashMap<String, PluginConfig>,
}

/// 单个插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// 插件名称
    pub name: String,
    /// 是否启用
    pub enabled: bool,
    /// 插件优先级
    pub priority: u32,
    /// 插件特定配置
    pub settings: HashMap<String, serde_json::Value>,
}

impl Default for PluginSystemConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            plugin_directory: "./plugins".to_string(),
            max_plugins: 50,
            execution_timeout_ms: 5000,
            enable_stats: true,
            plugins: HashMap::new(),
        }
    }
}

impl PluginSystemConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 添加插件配置
    pub fn add_plugin_config(&mut self, name: String, config: PluginConfig) {
        self.plugins.insert(name, config);
    }
    
    /// 获取插件配置
    pub fn get_plugin_config(&self, name: &str) -> Option<&PluginConfig> {
        self.plugins.get(name)
    }
    
    /// 更新插件配置
    pub fn update_plugin_config(&mut self, name: &str, config: PluginConfig) {
        self.plugins.insert(name.to_string(), config);
    }
    
    /// 移除插件配置
    pub fn remove_plugin_config(&mut self, name: &str) -> Option<PluginConfig> {
        self.plugins.remove(name)
    }
    
    /// 获取启用的插件列表
    pub fn get_enabled_plugins(&self) -> Vec<&PluginConfig> {
        self.plugins.values().filter(|p| p.enabled).collect()
    }
    
    /// 获取按优先级排序的插件配置
    pub fn get_plugins_by_priority(&self) -> Vec<&PluginConfig> {
        let mut plugins: Vec<_> = self.plugins.values().collect();
        plugins.sort_by_key(|p| p.priority);
        plugins
    }
}
