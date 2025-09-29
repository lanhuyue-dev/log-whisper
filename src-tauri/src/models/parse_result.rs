use serde::{Deserialize, Serialize};
use crate::models::{LogEntry, RenderedBlock};
use crate::parser::RenderError;

/// 解析结果结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// 原始日志条目
    pub original: LogEntry,
    /// 渲染块列表
    pub rendered_blocks: Vec<RenderedBlock>,
    /// 是否为错误日志
    pub is_error: bool,
    /// 是否为警告日志
    pub is_warning: bool,
    /// 解析统计信息
    pub stats: ParseStats,
}

/// 解析统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseStats {
    /// 解析耗时（毫秒）
    pub parse_time_ms: u64,
    /// 渲染块数量
    pub block_count: usize,
    /// 置信度平均值
    pub avg_confidence: f32,
    /// 是否成功解析
    pub success: bool,
}

impl Default for ParseStats {
    fn default() -> Self {
        Self {
            parse_time_ms: 0,
            block_count: 0,
            avg_confidence: 0.0,
            success: false,
        }
    }
}

impl ParseResult {
    /// 创建新的解析结果
    pub fn new(original: LogEntry) -> Self {
        let is_error = original.is_error();
        let is_warning = original.is_warning();
        
        Self {
            original,
            rendered_blocks: Vec::new(),
            is_error,
            is_warning,
            stats: ParseStats::default(),
        }
    }
    
    /// 添加渲染块
    pub fn add_block(mut self, block: RenderedBlock) -> Self {
        self.rendered_blocks.push(block);
        self
    }
    
    /// 添加多个渲染块
    pub fn add_blocks(mut self, blocks: Vec<RenderedBlock>) -> Self {
        self.rendered_blocks.extend(blocks);
        self
    }
    
    /// 设置解析统计信息
    pub fn with_stats(mut self, stats: ParseStats) -> Self {
        self.stats = stats;
        self
    }
    
    /// 检查是否有渲染块
    pub fn has_blocks(&self) -> bool {
        !self.rendered_blocks.is_empty()
    }
    
    /// 获取渲染块数量
    pub fn block_count(&self) -> usize {
        self.rendered_blocks.len()
    }
    
    /// 获取指定类型的渲染块
    pub fn get_blocks_by_type(&self, block_type: &crate::models::BlockType) -> Vec<&RenderedBlock> {
        self.rendered_blocks
            .iter()
            .filter(|block| &block.block_type == block_type)
            .collect()
    }
    
    /// 获取所有可复制的块
    pub fn get_copyable_blocks(&self) -> Vec<&RenderedBlock> {
        self.rendered_blocks
            .iter()
            .filter(|block| block.is_copyable)
            .collect()
    }
    
    /// 计算平均置信度
    pub fn calculate_avg_confidence(&mut self) {
        if !self.rendered_blocks.is_empty() {
            let total_confidence: f32 = self.rendered_blocks
                .iter()
                .map(|block| block.metadata.confidence)
                .sum();
            self.stats.avg_confidence = total_confidence / self.rendered_blocks.len() as f32;
        }
    }
    
    /// 更新统计信息
    pub fn update_stats(&mut self, parse_time_ms: u64) {
        self.stats.parse_time_ms = parse_time_ms;
        self.stats.block_count = self.rendered_blocks.len();
        self.stats.success = !self.rendered_blocks.is_empty();
        self.calculate_avg_confidence();
    }
}

/// 解析错误类型
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("文件读取错误: {0}")]
    FileReadError(String),
    
    #[error("解析错误: {0}")]
    ParseError(String),
    
    #[error("渲染错误: {0}")]
    RenderError(String),
    
    #[error("插件错误: {0}")]
    PluginError(String),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    #[error("内存不足: {0}")]
    OutOfMemory(String),
    
    #[error("超时错误: {0}")]
    TimeoutError(String),
}

/// 从RenderError转换为ParseError
impl From<RenderError> for ParseError {
    fn from(err: RenderError) -> Self {
        match err {
            RenderError::RenderFailed(msg) => ParseError::RenderError(msg),
            RenderError::PluginError(msg) => ParseError::PluginError(msg),
            RenderError::ConfigError(msg) => ParseError::ConfigError(msg),
        }
    }
}

/// 解析结果集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResultSet {
    /// 解析结果列表
    pub results: Vec<ParseResult>,
    /// 总体统计信息
    pub total_stats: TotalParseStats,
    /// 解析配置
    pub config: ParseConfig,
}

/// 总体解析统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalParseStats {
    /// 总行数
    pub total_lines: usize,
    /// 成功解析行数
    pub success_lines: usize,
    /// 错误行数
    pub error_lines: usize,
    /// 警告行数
    pub warning_lines: usize,
    /// 总渲染块数
    pub total_blocks: usize,
    /// 总解析时间（毫秒）
    pub total_parse_time_ms: u64,
    /// 平均每行解析时间（毫秒）
    pub avg_parse_time_per_line_ms: f64,
}

/// 解析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseConfig {
    /// 使用的插件名称
    pub plugin_name: String,
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 最大文件大小（字节）
    pub max_file_size: usize,
    /// 超时时间（毫秒）
    pub timeout_ms: u64,
}

impl Default for ParseConfig {
    fn default() -> Self {
        Self {
            plugin_name: "Auto".to_string(),
            enable_cache: true,
            max_file_size: 50 * 1024 * 1024, // 50MB
            timeout_ms: 30000, // 30秒
        }
    }
}

impl ParseResultSet {
    /// 创建新的解析结果集合
    pub fn new(config: ParseConfig) -> Self {
        Self {
            results: Vec::new(),
            total_stats: TotalParseStats {
                total_lines: 0,
                success_lines: 0,
                error_lines: 0,
                warning_lines: 0,
                total_blocks: 0,
                total_parse_time_ms: 0,
                avg_parse_time_per_line_ms: 0.0,
            },
            config,
        }
    }
    
    /// 添加解析结果
    pub fn add_result(&mut self, result: ParseResult) {
        self.results.push(result);
        self.update_total_stats();
    }
    
    /// 添加多个解析结果
    pub fn add_results(&mut self, results: Vec<ParseResult>) {
        self.results.extend(results);
        self.update_total_stats();
    }
    
    /// 更新总体统计信息
    fn update_total_stats(&mut self) {
        self.total_stats.total_lines = self.results.len();
        self.total_stats.success_lines = self.results.iter().filter(|r| r.stats.success).count();
        self.total_stats.error_lines = self.results.iter().filter(|r| r.is_error).count();
        self.total_stats.warning_lines = self.results.iter().filter(|r| r.is_warning).count();
        self.total_stats.total_blocks = self.results.iter().map(|r| r.block_count()).sum();
        
        if self.total_stats.total_lines > 0 {
            self.total_stats.avg_parse_time_per_line_ms = 
                self.total_stats.total_parse_time_ms as f64 / self.total_stats.total_lines as f64;
        }
    }
    
    /// 获取错误结果
    pub fn get_error_results(&self) -> Vec<&ParseResult> {
        self.results.iter().filter(|r| r.is_error).collect()
    }
    
    /// 获取警告结果
    pub fn get_warning_results(&self) -> Vec<&ParseResult> {
        self.results.iter().filter(|r| r.is_warning).collect()
    }
    
    /// 获取成功解析的结果
    pub fn get_success_results(&self) -> Vec<&ParseResult> {
        self.results.iter().filter(|r| r.stats.success).collect()
    }
    
    /// 清空结果
    pub fn clear(&mut self) {
        self.results.clear();
        self.total_stats = TotalParseStats {
            total_lines: 0,
            success_lines: 0,
            error_lines: 0,
            warning_lines: 0,
            total_blocks: 0,
            total_parse_time_ms: 0,
            avg_parse_time_per_line_ms: 0.0,
        };
    }
}
