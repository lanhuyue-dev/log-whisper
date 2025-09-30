use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::models::LogEntry;
use crate::plugins::{
    LogRenderer, PluginInfo, PluginStatus, PluginResult
};
use super::{MyBatisRenderer, JsonRepairRenderer, ErrorHighlighterRenderer, RawRenderer, DockerJsonRenderer};

/// 插件注册中心
pub struct PluginRegistry {
    plugins: HashMap<String, Arc<RwLock<dyn LogRenderer + Send + Sync>>>,
    plugin_info: HashMap<String, PluginInfo>,
    default_plugin: String,
}

impl PluginRegistry {
    /// 创建新的插件注册中心
    pub fn new() -> Self {
        let mut registry = Self {
            plugins: HashMap::new(),
            plugin_info: HashMap::new(),
            default_plugin: "Auto".to_string(),
        };
        
        // 注册默认插件
        registry.register_default_plugins();
        
        registry
    }
    
    /// 注册默认插件
    fn register_default_plugins(&mut self) {
        // 注册MyBatis插件
        let mybatis = Arc::new(RwLock::new(MyBatisRenderer::new()));
        let mybatis_info = PluginInfo::new(
            "MyBatis".to_string(),
            "MyBatis SQL 解析器".to_string(),
            "1.0.0".to_string(),
            "LogWhisper Team".to_string(),
        ).with_priority(10);
        
        self.plugins.insert("MyBatis".to_string(), mybatis);
        self.plugin_info.insert("MyBatis".to_string(), mybatis_info);
        
        // 注册Docker JSON插件
        let docker_json = Arc::new(RwLock::new(DockerJsonRenderer::new()));
        let docker_json_info = PluginInfo::new(
            "DockerJSON".to_string(),
            "Docker容器JSON日志解析器".to_string(),
            "1.0.0".to_string(),
            "LogWhisper Team".to_string(),
        ).with_priority(15);
        
        self.plugins.insert("DockerJSON".to_string(), docker_json);
        self.plugin_info.insert("DockerJSON".to_string(), docker_json_info);
        
        // 注册JSON修复插件
        let json_repair = Arc::new(RwLock::new(JsonRepairRenderer::new()));
        let json_info = PluginInfo::new(
            "JSON".to_string(),
            "JSON 修复和格式化".to_string(),
            "1.0.0".to_string(),
            "LogWhisper Team".to_string(),
        ).with_priority(20);
        
        self.plugins.insert("JSON".to_string(), json_repair);
        self.plugin_info.insert("JSON".to_string(), json_info);
        
        // 注册错误高亮插件
        let error_highlighter = Arc::new(RwLock::new(ErrorHighlighterRenderer::new()));
        let error_info = PluginInfo::new(
            "ErrorHighlighter".to_string(),
            "错误高亮插件".to_string(),
            "1.0.0".to_string(),
            "LogWhisper Team".to_string(),
        ).with_priority(5);
        
        self.plugins.insert("ErrorHighlighter".to_string(), error_highlighter);
        self.plugin_info.insert("ErrorHighlighter".to_string(), error_info);
        
        // 注册原始文本插件
        let raw = Arc::new(RwLock::new(RawRenderer::new()));
        let raw_info = PluginInfo::new(
            "Raw".to_string(),
            "原始文本显示".to_string(),
            "1.0.0".to_string(),
            "LogWhisper Team".to_string(),
        ).with_priority(1000);
        
        self.plugins.insert("Raw".to_string(), raw);
        self.plugin_info.insert("Raw".to_string(), raw_info);
    }
    
    /// 注册插件
    pub fn register_plugin(
        &mut self,
        name: String,
        plugin: Arc<RwLock<dyn LogRenderer + Send + Sync>>,
        info: PluginInfo,
    ) -> Result<(), String> {
        if self.plugins.contains_key(&name) {
            return Err(format!("Plugin '{}' already registered", name));
        }
        
        self.plugins.insert(name.clone(), plugin);
        self.plugin_info.insert(name, info);
        
        Ok(())
    }
    
    /// 获取插件
    pub fn get_plugin(&self, name: &str) -> Option<Arc<RwLock<dyn LogRenderer + Send + Sync>>> {
        self.plugins.get(name).cloned()
    }
    
    /// 获取插件信息
    pub fn get_plugin_info(&self, name: &str) -> Option<&PluginInfo> {
        self.plugin_info.get(name)
    }
    
    /// 获取所有插件名称
    pub fn get_plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
    
    /// 获取启用的插件名称
    pub fn get_enabled_plugin_names(&self) -> Vec<String> {
        self.plugin_info
            .iter()
            .filter(|(_, info)| info.enabled)
            .map(|(name, _)| name.clone())
            .collect()
    }
    
    /// 启用插件
    pub fn enable_plugin(&mut self, name: &str) -> Result<(), String> {
        if let Some(info) = self.plugin_info.get_mut(name) {
            info.enabled = true;
            info.update_status(PluginStatus::Initialized);
            Ok(())
        } else {
            Err(format!("Plugin '{}' not found", name))
        }
    }
    
    /// 禁用插件
    pub fn disable_plugin(&mut self, name: &str) -> Result<(), String> {
        if let Some(info) = self.plugin_info.get_mut(name) {
            info.enabled = false;
            info.update_status(PluginStatus::Stopped);
            Ok(())
        } else {
            Err(format!("Plugin '{}' not found", name))
        }
    }
    
    /// 设置默认插件
    pub fn set_default_plugin(&mut self, name: &str) -> Result<(), String> {
        if self.plugins.contains_key(name) {
            self.default_plugin = name.to_string();
            Ok(())
        } else {
            Err(format!("Plugin '{}' not found", name))
        }
    }
    
    /// 获取默认插件
    pub fn get_default_plugin(&self) -> &str {
        &self.default_plugin
    }
    
    /// 处理日志条目（自动选择最佳插件）
    pub fn process_entry(&self, entry: &LogEntry) -> PluginResult {
        let start_time = std::time::Instant::now();
        
        // 按优先级排序插件
        let mut sorted_plugins: Vec<_> = self.plugin_info.iter().collect();
        sorted_plugins.sort_by_key(|(_, info)| info.priority);
        
        for (name, info) in sorted_plugins {
            if !info.enabled {
                continue;
            }
            
            if let Some(plugin) = self.plugins.get(name) {
                if let Ok(plugin_guard) = plugin.read() {
                    if plugin_guard.can_handle(entry) {
                        let blocks = plugin_guard.render(entry);
                        let process_time = start_time.elapsed().as_millis() as u64;
                        let confidence = if blocks.is_empty() { 0.0 } else { 0.8 };
                        
                        return PluginResult::success(blocks, process_time, confidence);
                    }
                }
            }
        }
        
        // 如果没有插件能处理，使用原始文本插件
        if let Some(raw_plugin) = self.plugins.get("Raw") {
            if let Ok(plugin_guard) = raw_plugin.read() {
                let blocks = plugin_guard.render(entry);
                let process_time = start_time.elapsed().as_millis() as u64;
                return PluginResult::success(blocks, process_time, 1.0);
            }
        }
        
        let process_time = start_time.elapsed().as_millis() as u64;
        PluginResult::error("No plugin available".to_string(), process_time)
    }
    
    /// 使用指定插件处理日志条目
    pub fn process_entry_with_plugin(&self, entry: &LogEntry, plugin_name: &str) -> PluginResult {
        let start_time = std::time::Instant::now();
        
        if let Some(plugin) = self.plugins.get(plugin_name) {
            if let Ok(plugin_guard) = plugin.read() {
                if plugin_guard.can_handle(entry) {
                    let blocks = plugin_guard.render(entry);
                    let process_time = start_time.elapsed().as_millis() as u64;
                    let confidence = if blocks.is_empty() { 0.0 } else { 0.8 };
                    
                    return PluginResult::success(blocks, process_time, confidence);
                }
            }
        }
        
        let process_time = start_time.elapsed().as_millis() as u64;
        PluginResult::error(format!("Plugin '{}' not found or cannot handle entry", plugin_name), process_time)
    }
    
    /// 初始化所有插件
    pub fn initialize_all_plugins(&mut self) -> Result<(), String> {
        // 插件初始化在创建时已经完成，这里只是更新状态
        for (_name, info) in self.plugin_info.iter_mut() {
            info.update_status(PluginStatus::Initialized);
        }
        Ok(())
    }
    
    /// 清理所有插件
    pub fn cleanup_all_plugins(&mut self) -> Result<(), String> {
        // 更新插件状态
        for (_name, info) in self.plugin_info.iter_mut() {
            info.update_status(PluginStatus::Stopped);
        }
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
