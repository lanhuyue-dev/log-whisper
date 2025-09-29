use crate::models::{LogEntry, RenderedBlock};

/// 日志渲染器Trait定义
pub trait LogRenderer: Send + Sync {
    /// 检查是否可以处理该日志条目
    fn can_handle(&self, entry: &LogEntry) -> bool;
    
    /// 处理日志条目，返回渲染块
    fn render(&self, entry: &LogEntry) -> Vec<RenderedBlock>;
    
    /// 获取插件名称
    fn name(&self) -> &str;
    
    /// 获取插件描述
    fn description(&self) -> &str;
    
    /// 获取插件优先级（数字越小优先级越高）
    fn priority(&self) -> u32;
    
    /// 获取插件版本
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    /// 检查插件是否启用
    fn is_enabled(&self) -> bool {
        true
    }
    
    /// 设置插件启用状态
    fn set_enabled(&mut self, _enabled: bool) {
        // 默认实现，子类可以重写
    }
    
    /// 获取插件配置
    fn get_config(&self) -> Option<serde_json::Value> {
        None
    }
    
    /// 设置插件配置
    fn set_config(&mut self, _config: serde_json::Value) -> Result<(), String> {
        Ok(())
    }
}

/// 插件生命周期Trait
pub trait PluginLifecycle {
    /// 插件初始化
    fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }
    
    /// 插件清理
    fn cleanup(&mut self) -> Result<(), String> {
        Ok(())
    }
    
    /// 插件重置
    fn reset(&mut self) -> Result<(), String> {
        Ok(())
    }
}

/// 插件能力接口
pub trait PluginCapabilities {
    /// 获取支持的文件类型
    fn supported_file_types(&self) -> Vec<String> {
        vec!["*".to_string()]
    }
    
    /// 获取支持的文件大小限制
    fn max_file_size(&self) -> Option<usize> {
        None
    }
    
    /// 获取处理速度评级
    fn performance_rating(&self) -> PerformanceRating {
        PerformanceRating::Medium
    }
    
    /// 获取内存使用评级
    fn memory_usage_rating(&self) -> MemoryUsageRating {
        MemoryUsageRating::Medium
    }
}

/// 性能评级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceRating {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// 内存使用评级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryUsageRating {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// 插件状态枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    /// 未初始化
    Uninitialized,
    /// 已初始化
    Initialized,
    /// 运行中
    Running,
    /// 已停止
    Stopped,
    /// 错误状态
    Error(String),
}

/// 插件信息结构体
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// 插件名称
    pub name: String,
    /// 插件描述
    pub description: String,
    /// 插件版本
    pub version: String,
    /// 插件作者
    pub author: String,
    /// 插件状态
    pub status: PluginStatus,
    /// 插件优先级
    pub priority: u32,
    /// 是否启用
    pub enabled: bool,
}

impl PluginInfo {
    /// 创建新的插件信息
    pub fn new(name: String, description: String, version: String, author: String) -> Self {
        Self {
            name,
            description,
            version,
            author,
            status: PluginStatus::Uninitialized,
            priority: 100,
            enabled: true,
        }
    }
    
    /// 设置优先级
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
    
    /// 设置启用状态
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// 更新状态
    pub fn update_status(&mut self, status: PluginStatus) {
        self.status = status;
    }
    
    /// 检查插件是否可用
    pub fn is_available(&self) -> bool {
        self.enabled && self.status != PluginStatus::Error("".to_string())
    }
}

/// 插件结果结构体
#[derive(Debug, Clone)]
pub struct PluginResult {
    /// 是否成功
    pub success: bool,
    /// 渲染块列表
    pub blocks: Vec<RenderedBlock>,
    /// 处理时间（毫秒）
    pub process_time_ms: u64,
    /// 错误信息
    pub error: Option<String>,
    /// 置信度
    pub confidence: f32,
}

impl PluginResult {
    /// 创建成功结果
    pub fn success(blocks: Vec<RenderedBlock>, process_time_ms: u64, confidence: f32) -> Self {
        Self {
            success: true,
            blocks,
            process_time_ms,
            error: None,
            confidence,
        }
    }
    
    /// 创建失败结果
    pub fn error(error: String, process_time_ms: u64) -> Self {
        Self {
            success: false,
            blocks: Vec::new(),
            process_time_ms,
            error: Some(error),
            confidence: 0.0,
        }
    }
    
    /// 检查是否有渲染块
    pub fn has_blocks(&self) -> bool {
        !self.blocks.is_empty()
    }
    
    /// 获取渲染块数量
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }
}
