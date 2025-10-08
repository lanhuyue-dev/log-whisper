//! 插件注册中心
//! 
//! 负责插件的注册、管理和生命周期控制

use super::{Plugin, PluginInfo, PluginType, PluginError};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// 插件注册中心
pub struct PluginRegistry {
    /// 已注册的插件
    plugins: HashMap<String, Box<dyn Plugin>>,
    /// 插件信息缓存
    plugin_info: HashMap<String, PluginInfo>,
}

impl PluginRegistry {
    /// 创建新的插件注册中心
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_info: HashMap::new(),
        }
    }
    
    /// 注册插件
    pub async fn register(&mut self, mut plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let name = plugin.name().to_string();
        
        // 初始化插件
        plugin.initialize()?;
        
        // 创建插件信息
        let info = PluginInfo {
            name: name.clone(),
            version: plugin.version().to_string(),
            description: plugin.description().to_string(),
            plugin_type: plugin.plugin_type(),
            priority: plugin.priority(),
            enabled: true,
        };
        
        // 注册插件
        self.plugins.insert(name.clone(), plugin);
        self.plugin_info.insert(name, info);
        
        Ok(())
    }
    
    /// 获取插件
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }
    
    /// 获取插件列表
    pub fn get_plugin_list(&self) -> Vec<PluginInfo> {
        self.plugin_info.values().cloned().collect()
    }
    
    /// 获取按优先级排序的插件列表
    pub fn get_plugins_by_priority(&self) -> Vec<&dyn Plugin> {
        let mut plugins: Vec<_> = self.plugins.values().map(|p| p.as_ref()).collect();
        plugins.sort_by_key(|p| p.priority());
        plugins
    }
    
    /// 获取指定类型的插件
    pub fn get_plugins_by_type(&self, plugin_type: PluginType) -> Vec<&dyn Plugin> {
        self.plugins
            .values()
            .filter(|p| p.plugin_type() == plugin_type)
            .map(|p| p.as_ref())
            .collect()
    }
    
    /// 启用/禁用插件
    pub fn set_plugin_enabled(&mut self, name: &str, enabled: bool) -> Result<(), PluginError> {
        if let Some(info) = self.plugin_info.get_mut(name) {
            info.enabled = enabled;
            Ok(())
        } else {
            Err(PluginError::ConfigurationError(format!("插件 {} 不存在", name)))
        }
    }
    
    /// 移除插件
    pub fn unregister(&mut self, name: &str) -> Result<(), PluginError> {
        if let Some(mut plugin) = self.plugins.remove(name) {
            plugin.cleanup()?;
        }
        self.plugin_info.remove(name);
        Ok(())
    }
    
    /// 清理所有插件
    pub fn cleanup_all(&mut self) -> Result<(), PluginError> {
        for (_, mut plugin) in self.plugins.drain() {
            plugin.cleanup()?;
        }
        self.plugin_info.clear();
        Ok(())
    }
}
