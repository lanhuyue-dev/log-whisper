use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::models::LogEntry;
use crate::plugins::{
    LogRenderer, LogAnalyzer, LogCorrelator, LogNavigator, LogFilter,
    PluginInfo, PluginStatus, PluginResult, PluginType,
    AnalysisResult, CorrelationResult, NavigationResult, FilterResult
};
use super::{
    MyBatisRenderer, JsonRepairRenderer, ErrorHighlighterRenderer, RawRenderer, DockerJsonRenderer,
    // ReplicaAnalyzer, CorrelationTracker, TimeDriftNavigator, LogDeduplicator // 暂时禁用
};

/// 插件注册中心
pub struct PluginRegistry {
    renderers: HashMap<String, Arc<RwLock<dyn LogRenderer + Send + Sync>>>,
    analyzers: HashMap<String, Arc<RwLock<dyn LogAnalyzer + Send + Sync>>>,
    correlators: HashMap<String, Arc<RwLock<dyn LogCorrelator + Send + Sync>>>,
    navigators: HashMap<String, Arc<RwLock<dyn LogNavigator + Send + Sync>>>,
    filters: HashMap<String, Arc<RwLock<dyn LogFilter + Send + Sync>>>,
    plugin_info: HashMap<String, PluginInfo>,
    default_plugin: String,
}

impl PluginRegistry {
    /// 创建新的插件注册中心
    pub fn new() -> Self {
        let mut registry = Self {
            renderers: HashMap::new(),
            analyzers: HashMap::new(),
            correlators: HashMap::new(),
            navigators: HashMap::new(),
            filters: HashMap::new(),
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
            PluginType::Renderer,
        ).with_priority(10);
        
        self.renderers.insert("MyBatis".to_string(), mybatis);
        self.plugin_info.insert("MyBatis".to_string(), mybatis_info);
        
        // 注册Docker JSON插件
        let docker_json = Arc::new(RwLock::new(DockerJsonRenderer::new()));
        let docker_json_info = PluginInfo::new(
            "DockerJSON".to_string(),
            "Docker容器JSON日志解析器".to_string(),
            "1.0.0".to_string(),
            "LogWhisper Team".to_string(),
            PluginType::Renderer,
        ).with_priority(15);
        
        self.renderers.insert("DockerJSON".to_string(), docker_json);
        self.plugin_info.insert("DockerJSON".to_string(), docker_json_info);
        
        // 注册JSON修复插件
        let json_repair = Arc::new(RwLock::new(JsonRepairRenderer::new()));
        let json_info = PluginInfo::new(
            "JSON".to_string(),
            "JSON 修复和格式化".to_string(),
            "1.0.0".to_string(),
            "LogWhisper Team".to_string(),
            PluginType::Renderer,
        ).with_priority(20);
        
        self.renderers.insert("JSON".to_string(), json_repair);
        self.plugin_info.insert("JSON".to_string(), json_info);
        
        // 注册错误高亮插件
        let error_highlighter = Arc::new(RwLock::new(ErrorHighlighterRenderer::new()));
        let error_info = PluginInfo::new(
            "ErrorHighlighter".to_string(),
            "错误高亮插件".to_string(),
            "1.0.0".to_string(),
            "LogWhisper Team".to_string(),
            PluginType::Renderer,
        ).with_priority(5);
        
        self.renderers.insert("ErrorHighlighter".to_string(), error_highlighter);
        self.plugin_info.insert("ErrorHighlighter".to_string(), error_info);
        
        // 注册原始文本插件
        let raw = Arc::new(RwLock::new(RawRenderer::new()));
        let raw_info = PluginInfo::new(
            "Raw".to_string(),
            "原始文本显示".to_string(),
            "1.0.0".to_string(),
            "LogWhisper Team".to_string(),
            PluginType::Renderer,
        ).with_priority(1000);
        
        self.renderers.insert("Raw".to_string(), raw);
        self.plugin_info.insert("Raw".to_string(), raw_info);
        
        // // 暂时禁用所有非渲染器插件 - 等待修复结构体不匹配问题
        // // 注册ReplicaAnalyzer插件
        // let replica_analyzer = Arc::new(RwLock::new(ReplicaAnalyzer::new()));
        // self.analyzers.insert("ReplicaAnalyzer".to_string(), replica_analyzer);
        
        // // 注册CorrelationTracker插件
        // let correlation_tracker = Arc::new(RwLock::new(CorrelationTracker::new()));
        // self.correlators.insert("CorrelationTracker".to_string(), correlation_tracker);
        
        // // 暂时禁用 - 等待修复结构体不匹配问题
        // // 注册TimeDriftNavigator插件
        // let time_navigator = Arc::new(RwLock::new(TimeDriftNavigator::new()));
        // let time_info = time_navigator.read().unwrap().plugin_info();
        // self.navigators.insert("TimeDriftNavigator".to_string(), time_navigator);
        // self.plugin_info.insert("TimeDriftNavigator".to_string(), time_info);
        
        // // 注册LogDeduplicator插件
        // let log_deduplicator = Arc::new(RwLock::new(LogDeduplicator::new()));
        // let dedup_info = log_deduplicator.read().unwrap().plugin_info();
        // self.filters.insert("LogDeduplicator".to_string(), log_deduplicator);
        // self.plugin_info.insert("LogDeduplicator".to_string(), dedup_info);
    }
    
    /// 注册渲染器插件
    pub fn register_renderer(
        &mut self,
        name: String,
        plugin: Arc<RwLock<dyn LogRenderer + Send + Sync>>,
        info: PluginInfo,
    ) -> Result<(), String> {
        if self.renderers.contains_key(&name) {
            return Err(format!("Renderer '{}' already registered", name));
        }
        
        self.renderers.insert(name.clone(), plugin);
        self.plugin_info.insert(name, info);
        
        Ok(())
    }
    
    /// 获取渲染器插件
    pub fn get_renderer(&self, name: &str) -> Option<Arc<RwLock<dyn LogRenderer + Send + Sync>>> {
        self.renderers.get(name).cloned()
    }
    
    /// 获取插件信息
    pub fn get_plugin_info(&self, name: &str) -> Option<&PluginInfo> {
        self.plugin_info.get(name)
    }
    
    /// 获取所有插件名称
    pub fn get_plugin_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        names.extend(self.renderers.keys().cloned());
        names.extend(self.analyzers.keys().cloned());
        names.extend(self.correlators.keys().cloned());
        names.extend(self.navigators.keys().cloned());
        names.extend(self.filters.keys().cloned());
        names
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
        if self.renderers.contains_key(name) {
            self.default_plugin = name.to_string();
            Ok(())
        } else {
            Err(format!("Renderer plugin '{}' not found", name))
        }
    }
    
    /// 获取默认插件
    pub fn get_default_plugin(&self) -> &str {
        &self.default_plugin
    }
    
    /// 处理日志条目（自动选择最佳渲染器插件）
    pub fn process_entry(&self, entry: &LogEntry) -> PluginResult {
        let start_time = std::time::Instant::now();
        
        // 按优先级排序渲染器插件
        let mut sorted_renderers: Vec<_> = self.plugin_info
            .iter()
            .filter(|(name, info)| {
                info.enabled && 
                info.plugin_type == PluginType::Renderer &&
                self.renderers.contains_key(*name)
            })
            .collect();
        sorted_renderers.sort_by_key(|(_, info)| info.priority);
        
        for (name, _info) in sorted_renderers {
            if let Some(plugin) = self.renderers.get(name) {
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
        if let Some(raw_plugin) = self.renderers.get("Raw") {
            if let Ok(plugin_guard) = raw_plugin.read() {
                let blocks = plugin_guard.render(entry);
                let process_time = start_time.elapsed().as_millis() as u64;
                return PluginResult::success(blocks, process_time, 1.0);
            }
        }
        
        let process_time = start_time.elapsed().as_millis() as u64;
        PluginResult::error("No renderer plugin available".to_string(), process_time)
    }
    
    /// 使用指定渲染器插件处理日志条目
    pub fn process_entry_with_plugin(&self, entry: &LogEntry, plugin_name: &str) -> PluginResult {
        let start_time = std::time::Instant::now();
        
        if let Some(plugin) = self.renderers.get(plugin_name) {
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
        PluginResult::error(format!("Renderer plugin '{}' not found or cannot handle entry", plugin_name), process_time)
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