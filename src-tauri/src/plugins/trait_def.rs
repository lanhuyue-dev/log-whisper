use crate::models::{LogEntry, RenderedBlock};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 日志条目分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// 分析类型
    pub analysis_type: String,
    /// 分析结果数据
    pub data: serde_json::Value,
    /// 置信度
    pub confidence: f32,
    /// 分析耗时（毫秒）
    pub duration_ms: u64,
}

/// 关联分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    /// 关联ID
    pub correlation_id: String,
    /// 关联的日志条目
    pub related_entries: Vec<LogEntry>,
    /// 关联强度
    pub correlation_strength: f32,
    /// 关联类型
    pub correlation_type: String,
}

/// 导航结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationResult {
    /// 导航类型
    pub navigation_type: String,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 时间范围
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// 导航数据
    pub navigation_data: serde_json::Value,
    /// 数据
    pub data: serde_json::Value,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 置信度
    pub confidence: f32,
}

/// 过滤结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterResult {
    /// 过滤类型
    pub filter_type: String,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 过滤后的日志条目
    pub filtered_entries: Vec<LogEntry>,
    /// 过滤统计
    pub filter_stats: FilterStats,
    /// 元数据
    pub metadata: HashMap<String, String>,
    /// 置信度
    pub confidence: f32,
}

/// 过滤统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterStats {
    /// 原始条目数
    pub original_count: usize,
    /// 过滤后条目数
    pub filtered_count: usize,
    /// 移除的条目数
    pub removed_count: usize,
    /// 过滤规则
    pub filter_rules: Vec<String>,
}

/// 日志分析器Trait定义
pub trait LogAnalyzer: Send + Sync {
    /// 检查是否可以分析该日志条目集合
    fn can_analyze(&self, entries: &[LogEntry]) -> bool;
    
    /// 对日志条目集合进行分析
    fn analyze(&self, entries: &[LogEntry]) -> Vec<AnalysisResult>;
    
    /// 获取分析器名称
    fn name(&self) -> &str;
    
    /// 获取分析器描述
    fn description(&self) -> &str;
    
    /// 获取分析器优先级
    fn priority(&self) -> u32;
    
    /// 获取支持的分析类型
    fn supported_analysis_types(&self) -> Vec<String> {
        vec!["general".to_string()]
    }
    
    /// 获取最小分析样本数
    fn min_sample_size(&self) -> usize {
        1
    }
}

/// 日志关联器Trait定义
pub trait LogCorrelator: Send + Sync {
    /// 检查是否可以进行关联分析
    fn can_correlate(&self, entries: &[LogEntry]) -> bool;
    
    /// 进行关联分析
    fn correlate(&self, entries: &[LogEntry]) -> Vec<CorrelationResult>;
    
    /// 获取关联器名称
    fn name(&self) -> &str;
    
    /// 获取关联器描述
    fn description(&self) -> &str;
    
    /// 获取关联器优先级
    fn priority(&self) -> u32;
    
    /// 获取支持的关联类型
    fn supported_correlation_types(&self) -> Vec<String> {
        vec!["keyword".to_string()]
    }
    
    /// 获取最大关联距离（秒）
    fn max_correlation_distance_seconds(&self) -> u64 {
        300 // 5分钟
    }
}

/// 日志导航器Trait定义
pub trait LogNavigator: Send + Sync {
    /// 检查是否可以提供导航服务
    fn can_navigate(&self, entries: &[LogEntry]) -> bool;
    
    /// 生成导航结果
    fn navigate(&self, entries: &[LogEntry]) -> Vec<NavigationResult>;
    
    /// 获取导航器名称
    fn name(&self) -> &str;
    
    /// 获取导航器描述
    fn description(&self) -> &str;
    
    /// 获取导航器优先级
    fn priority(&self) -> u32;
    
    /// 获取支持的导航类型
    fn supported_navigation_types(&self) -> Vec<String> {
        vec!["timeline".to_string()]
    }
    
    /// 获取时间粒度（秒）
    fn time_granularity_seconds(&self) -> u64 {
        60 // 1分钟
    }
}

/// 日志过滤器Trait定义
pub trait LogFilter: Send + Sync {
    /// 检查是否可以过滤该日志条目集合
    fn can_filter(&self, entries: &[LogEntry]) -> bool;
    
    /// 过滤日志条目集合
    fn filter(&self, entries: &[LogEntry]) -> FilterResult;
    
    /// 获取过滤器名称
    fn name(&self) -> &str;
    
    /// 获取过滤器描述
    fn description(&self) -> &str;
    
    /// 获取过滤器优先级
    fn priority(&self) -> u32;
    
    /// 获取支持的过滤类型
    fn supported_filter_types(&self) -> Vec<String> {
        vec!["content".to_string()]
    }
    
    /// 是否保留原始数据
    fn preserve_original(&self) -> bool {
        true
    }
}

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

/// 插件类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    /// 渲染器插件
    Renderer,
    /// 分析器插件
    Analyzer,
    /// 关联器插件
    Correlator,
    /// 导航器插件
    Navigator,
    /// 过滤器插件
    Filter,
}

impl PluginType {
    /// 获取插件类型的描述
    pub fn description(&self) -> &'static str {
        match self {
            PluginType::Renderer => "日志内容渲染和显示",
            PluginType::Analyzer => "日志数据分析和挖掘",
            PluginType::Correlator => "日志关联和聚合分析",
            PluginType::Navigator => "日志导航和索引",
            PluginType::Filter => "日志过滤和筛选",
        }
    }
    
    /// 获取插件类型的默认优先级范围
    pub fn default_priority_range(&self) -> (u32, u32) {
        match self {
            PluginType::Renderer => (1, 100),
            PluginType::Analyzer => (101, 200),
            PluginType::Correlator => (201, 300),
            PluginType::Navigator => (301, 400),
            PluginType::Filter => (401, 500),
        }
    }
}

/// 优先级类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// 低优先级
    Low = 1,
    /// 中等优先级
    Medium = 2,
    /// 高优先级
    High = 3,
    /// 非常高优先级
    VeryHigh = 4,
}

impl Priority {
    /// 转换为数值
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
    
    /// 从数值创建优先级
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => Priority::Low,
            2 => Priority::Medium,
            3 => Priority::High,
            4 | _ => Priority::VeryHigh,
        }
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
    /// 插件类型
    pub plugin_type: PluginType,
}

impl PluginInfo {
    /// 创建新的插件信息
    pub fn new(name: String, description: String, version: String, author: String, plugin_type: PluginType) -> Self {
        let priority = plugin_type.default_priority_range().0;
        Self {
            name,
            description,
            version,
            author,
            status: PluginStatus::Uninitialized,
            priority,
            enabled: true,
            plugin_type,
        }
    }
    
    /// 从旧版本兼容性创建（默认为Renderer类型）
    pub fn new_renderer(name: String, description: String, version: String, author: String) -> Self {
        Self::new(name, description, version, author, PluginType::Renderer)
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

/// 插件统一结果类型
#[derive(Debug, Clone)]
pub enum PluginExecutionResult {
    /// 渲染结果
    Render(PluginResult),
    /// 分析结果
    Analysis(Vec<AnalysisResult>),
    /// 关联结果
    Correlation(Vec<CorrelationResult>),
    /// 导航结果
    Navigation(Vec<NavigationResult>),
    /// 过滤结果
    Filter(FilterResult),
}

impl PluginExecutionResult {
    /// 获取结果类型
    pub fn result_type(&self) -> &'static str {
        match self {
            PluginExecutionResult::Render(_) => "render",
            PluginExecutionResult::Analysis(_) => "analysis",
            PluginExecutionResult::Correlation(_) => "correlation",
            PluginExecutionResult::Navigation(_) => "navigation",
            PluginExecutionResult::Filter(_) => "filter",
        }
    }
    
    /// 检查结果是否成功
    pub fn is_success(&self) -> bool {
        match self {
            PluginExecutionResult::Render(result) => result.success,
            PluginExecutionResult::Analysis(results) => !results.is_empty(),
            PluginExecutionResult::Correlation(results) => !results.is_empty(),
            PluginExecutionResult::Navigation(results) => !results.is_empty(),
            PluginExecutionResult::Filter(result) => !result.filtered_entries.is_empty(),
        }
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
