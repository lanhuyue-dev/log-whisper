use crate::models::{LogEntry, RenderedBlock, BlockMetadata};
use crate::plugins::{LogRenderer, PluginLifecycle, PluginCapabilities, PerformanceRating, MemoryUsageRating};

/// 原始文本插件
pub struct RawRenderer {
    enabled: bool,
}

impl RawRenderer {
    /// 创建新的原始文本渲染器
    pub fn new() -> Self {
        Self {
            enabled: true,
        }
    }
    
    /// 格式化原始文本
    fn format_raw_text(&self, content: &str) -> String {
        // 简单的文本格式化
        let mut formatted = content.to_string();
        
        // 保留原始格式，只做基本的清理
        formatted = formatted.trim().to_string();
        
        formatted
    }
}

impl LogRenderer for RawRenderer {
    fn can_handle(&self, _entry: &LogEntry) -> bool {
        // 原始文本插件总是可以处理任何内容
        self.enabled
    }
    
    fn render(&self, entry: &LogEntry) -> Vec<RenderedBlock> {
        let block = RenderedBlock::raw(
            format!("raw_{}", entry.line_number),
            self.format_raw_text(&entry.content),
        ).with_metadata(BlockMetadata {
            line_start: entry.line_number,
            line_end: entry.line_number,
            char_start: 0,
            char_end: entry.content.len(),
            confidence: 1.0,
        });
        
        vec![block]
    }
    
    fn name(&self) -> &str {
        "Raw Text"
    }
    
    fn description(&self) -> &str {
        "显示原始文本内容，不进行任何解析或格式化"
    }
    
    fn priority(&self) -> u32 {
        1000 // 最低优先级
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl PluginLifecycle for RawRenderer {
    fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), String> {
        Ok(())
    }
}

impl PluginCapabilities for RawRenderer {
    fn supported_file_types(&self) -> Vec<String> {
        vec!["*".to_string()]
    }
    
    fn max_file_size(&self) -> Option<usize> {
        None
    }
    
    fn performance_rating(&self) -> PerformanceRating {
        PerformanceRating::VeryHigh
    }
    
    fn memory_usage_rating(&self) -> MemoryUsageRating {
        MemoryUsageRating::Low
    }
}
