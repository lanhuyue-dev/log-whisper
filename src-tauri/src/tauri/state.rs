use std::sync::Arc;
use crate::parser::LogParser;

/// 应用状态
pub struct AppState {
    /// 日志解析器
    pub parser: Arc<LogParser>,
    /// 当前文件路径
    pub current_file: Option<String>,
    /// 当前插件
    pub current_plugin: String,
    /// 是否启用缓存
    pub cache_enabled: bool,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new() -> Self {
        Self {
            parser: Arc::new(LogParser::new()),
            current_file: None,
            current_plugin: "Auto".to_string(),
            cache_enabled: true,
        }
    }
    
    /// 设置当前文件
    pub fn set_current_file(&mut self, file_path: String) {
        self.current_file = Some(file_path);
    }
    
    /// 清除当前文件
    pub fn clear_current_file(&mut self) {
        self.current_file = None;
    }
    
    /// 设置当前插件
    pub fn set_current_plugin(&mut self, plugin_name: String) {
        self.current_plugin = plugin_name;
    }
    
    /// 设置缓存状态
    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
