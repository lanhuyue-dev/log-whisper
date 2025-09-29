use std::sync::Arc;
use std::time::Instant;
use crate::models::{LogEntry, ParseResult, ParseStats};
use crate::plugins::PluginRegistry;

/// 渲染引擎
pub struct RenderEngine {
    plugin_registry: Arc<PluginRegistry>,
    cache_enabled: bool,
}

impl RenderEngine {
    /// 创建新的渲染引擎
    pub fn new(plugin_registry: Arc<PluginRegistry>) -> Self {
        Self {
            plugin_registry,
            cache_enabled: true,
        }
    }
    
    /// 设置缓存启用状态
    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }
    
    /// 渲染单个日志条目
    pub fn render_entry(&self, entry: &LogEntry) -> Result<ParseResult, RenderError> {
        let start_time = Instant::now();
        
        // 使用插件注册中心处理条目
        let plugin_result = self.plugin_registry.process_entry(entry);
        
        let process_time = start_time.elapsed().as_millis() as u64;
        
        // 创建解析结果
        let mut parse_result = ParseResult::new(entry.clone());
        
        if plugin_result.success {
            parse_result = parse_result.add_blocks(plugin_result.blocks);
        }
        
        // 更新统计信息
        let stats = ParseStats {
            parse_time_ms: process_time,
            block_count: parse_result.block_count(),
            avg_confidence: plugin_result.confidence,
            success: plugin_result.success,
        };
        
        parse_result = parse_result.with_stats(stats);
        
        Ok(parse_result)
    }
    
    /// 使用指定插件渲染日志条目
    pub fn render_entry_with_plugin(&self, entry: &LogEntry, plugin_name: &str) -> Result<ParseResult, RenderError> {
        let start_time = Instant::now();
        
        // 使用指定插件处理条目
        let plugin_result = self.plugin_registry.process_entry_with_plugin(entry, plugin_name);
        
        let process_time = start_time.elapsed().as_millis() as u64;
        
        // 创建解析结果
        let mut parse_result = ParseResult::new(entry.clone());
        
        if plugin_result.success {
            parse_result = parse_result.add_blocks(plugin_result.blocks);
        }
        
        // 更新统计信息
        let stats = ParseStats {
            parse_time_ms: process_time,
            block_count: parse_result.block_count(),
            avg_confidence: plugin_result.confidence,
            success: plugin_result.success,
        };
        
        parse_result = parse_result.with_stats(stats);
        
        Ok(parse_result)
    }
    
    /// 批量渲染日志条目
    pub fn render_entries(&self, entries: Vec<LogEntry>) -> Result<Vec<ParseResult>, RenderError> {
        let mut results = Vec::new();
        
        for entry in entries {
            let result = self.render_entry(&entry)?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// 使用指定插件批量渲染日志条目
    pub fn render_entries_with_plugin(&self, entries: Vec<LogEntry>, plugin_name: &str) -> Result<Vec<ParseResult>, RenderError> {
        let mut results = Vec::new();
        
        for entry in entries {
            let result = self.render_entry_with_plugin(&entry, plugin_name)?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// 获取可用的插件列表
    pub fn get_available_plugins(&self) -> Vec<String> {
        self.plugin_registry.get_plugin_names()
    }
    
    /// 获取启用的插件列表
    pub fn get_enabled_plugins(&self) -> Vec<String> {
        self.plugin_registry.get_enabled_plugin_names()
    }
    
    /// 启用插件
    pub fn enable_plugin(&self, _plugin_name: &str) -> Result<(), String> {
        // 这里需要修改PluginRegistry以支持可变引用
        // 暂时返回成功
        Ok(())
    }
    
    /// 禁用插件
    pub fn disable_plugin(&self, _plugin_name: &str) -> Result<(), String> {
        // 这里需要修改PluginRegistry以支持可变引用
        // 暂时返回成功
        Ok(())
    }
    
    /// 设置默认插件
    pub fn set_default_plugin(&self, _plugin_name: &str) -> Result<(), String> {
        // 这里需要修改PluginRegistry以支持可变引用
        // 暂时返回成功
        Ok(())
    }
    
    /// 获取渲染统计信息
    pub fn get_render_stats(&self, results: &[ParseResult]) -> RenderStats {
        let total_entries = results.len();
        let successful_entries = results.iter().filter(|r| r.stats.success).count();
        let total_blocks: usize = results.iter().map(|r| r.block_count()).sum();
        let total_parse_time: u64 = results.iter().map(|r| r.stats.parse_time_ms).sum();
        let avg_confidence: f32 = if !results.is_empty() {
            results.iter().map(|r| r.stats.avg_confidence).sum::<f32>() / results.len() as f32
        } else {
            0.0
        };
        
        RenderStats {
            total_entries,
            successful_entries,
            total_blocks,
            total_parse_time_ms: total_parse_time,
            avg_parse_time_per_entry_ms: if total_entries > 0 {
                total_parse_time as f64 / total_entries as f64
            } else {
                0.0
            },
            avg_confidence,
            success_rate: if total_entries > 0 {
                successful_entries as f32 / total_entries as f32
            } else {
                0.0
            },
        }
    }
}

/// 渲染错误类型
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("渲染失败: {0}")]
    RenderFailed(String),
    
    #[error("插件错误: {0}")]
    PluginError(String),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
}

/// 渲染统计信息
#[derive(Debug, Clone)]
pub struct RenderStats {
    pub total_entries: usize,
    pub successful_entries: usize,
    pub total_blocks: usize,
    pub total_parse_time_ms: u64,
    pub avg_parse_time_per_entry_ms: f64,
    pub avg_confidence: f32,
    pub success_rate: f32,
}

impl RenderStats {
    /// 获取性能评级
    pub fn get_performance_rating(&self) -> PerformanceRating {
        if self.avg_parse_time_per_entry_ms < 1.0 {
            PerformanceRating::Excellent
        } else if self.avg_parse_time_per_entry_ms < 5.0 {
            PerformanceRating::Good
        } else if self.avg_parse_time_per_entry_ms < 10.0 {
            PerformanceRating::Fair
        } else {
            PerformanceRating::Poor
        }
    }
    
    /// 获取成功率评级
    pub fn get_success_rating(&self) -> SuccessRating {
        if self.success_rate >= 0.95 {
            SuccessRating::Excellent
        } else if self.success_rate >= 0.8 {
            SuccessRating::Good
        } else if self.success_rate >= 0.6 {
            SuccessRating::Fair
        } else {
            SuccessRating::Poor
        }
    }
}

/// 性能评级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceRating {
    Excellent,
    Good,
    Fair,
    Poor,
}

/// 成功率评级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuccessRating {
    Excellent,
    Good,
    Fair,
    Poor,
}
