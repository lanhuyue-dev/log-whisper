use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::ParseResult;

/// 分块加载器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkLoaderConfig {
    /// 默认块大小
    pub default_chunk_size: usize,
    /// 最大块大小
    pub max_chunk_size: usize,
    /// 最小块大小
    pub min_chunk_size: usize,
    /// 是否启用自适应块大小
    pub adaptive_chunk_size: bool,
    /// 内存阈值（字节）
    pub memory_threshold: usize,
    /// 预加载块数量
    pub preload_chunk_count: usize,
    /// GC触发阈值（字节）
    pub gc_threshold: usize,
}

impl Default for ChunkLoaderConfig {
    fn default() -> Self {
        Self {
            default_chunk_size: 100,
            max_chunk_size: 1000,
            min_chunk_size: 50,
            adaptive_chunk_size: true,
            memory_threshold: 500 * 1024 * 1024, // 500MB
            preload_chunk_count: 5,
            gc_threshold: 200 * 1024 * 1024, // 200MB
        }
    }
}

/// 数据块信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    /// 块索引
    pub index: usize,
    /// 起始行号
    pub start_line: usize,
    /// 结束行号
    pub end_line: usize,
    /// 块大小（行数）
    pub size: usize,
    /// 是否已加载
    pub is_loaded: bool,
    /// 内存使用量（字节）
    pub memory_usage: usize,
    /// 最后访问时间
    pub last_access_time: std::time::SystemTime,
}

/// 分块加载请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkLoadRequest {
    /// 文件路径
    pub file_path: String,
    /// 块索引列表
    pub chunk_indices: Vec<usize>,
    /// 插件名称
    pub plugin_name: String,
    /// 优先级
    pub priority: ChunkPriority,
}

/// 块加载优先级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChunkPriority {
    /// 低优先级
    Low,
    /// 正常优先级
    Normal,
    /// 高优先级
    High,
    /// 立即加载
    Immediate,
}

/// 分块加载响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkLoadResponse {
    /// 是否成功
    pub success: bool,
    /// 加载的块数据
    pub chunks: HashMap<usize, Vec<ParseResult>>,
    /// 块信息
    pub chunk_infos: Vec<ChunkInfo>,
    /// 总内存使用量
    pub total_memory_usage: usize,
    /// 错误信息
    pub error: Option<String>,
}

/// 内存管理信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// 当前内存使用量（字节）
    pub current_usage: usize,
    /// 最大内存使用量（字节）
    pub max_usage: usize,
    /// 已缓存的块数量
    pub cached_chunks: usize,
    /// 最大缓存块数量
    pub max_cached_chunks: usize,
    /// 最后一次GC时间
    pub last_gc_time: std::time::SystemTime,
    /// GC触发次数
    pub gc_count: usize,
    /// 内存压力级别
    pub pressure_level: MemoryPressureLevel,
}

/// 内存压力级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryPressureLevel {
    /// 正常
    Normal,
    /// 中等压力
    Moderate,
    /// 高压力
    High,
    /// 危险
    Critical,
}

/// 分块元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    /// 文件路径
    pub file_path: String,
    /// 总行数
    pub total_lines: usize,
    /// 总块数
    pub total_chunks: usize,
    /// 块大小
    pub chunk_size: usize,
    /// 文件大小（字节）
    pub file_size: usize,
    /// 创建时间
    pub created_at: std::time::SystemTime,
    /// 块信息列表
    pub chunks: Vec<ChunkInfo>,
}

impl ChunkMetadata {
    /// 创建新的分块元数据
    pub fn new(file_path: String, total_lines: usize, chunk_size: usize, file_size: usize) -> Self {
        let total_chunks = (total_lines + chunk_size - 1) / chunk_size;
        let mut chunks = Vec::with_capacity(total_chunks);
        
        for i in 0..total_chunks {
            let start_line = i * chunk_size;
            let end_line = std::cmp::min(start_line + chunk_size, total_lines);
            chunks.push(ChunkInfo {
                index: i,
                start_line,
                end_line,
                size: end_line - start_line,
                is_loaded: false,
                memory_usage: 0,
                last_access_time: std::time::SystemTime::now(),
            });
        }
        
        Self {
            file_path,
            total_lines,
            total_chunks,
            chunk_size,
            file_size,
            created_at: std::time::SystemTime::now(),
            chunks,
        }
    }
    
    /// 获取指定索引的块信息
    pub fn get_chunk_info(&self, index: usize) -> Option<&ChunkInfo> {
        self.chunks.get(index)
    }
    
    /// 获取指定索引的块信息（可变）
    pub fn get_chunk_info_mut(&mut self, index: usize) -> Option<&mut ChunkInfo> {
        self.chunks.get_mut(index)
    }
    
    /// 标记块为已加载
    pub fn mark_chunk_loaded(&mut self, index: usize, memory_usage: usize) {
        if let Some(chunk) = self.get_chunk_info_mut(index) {
            chunk.is_loaded = true;
            chunk.memory_usage = memory_usage;
            chunk.last_access_time = std::time::SystemTime::now();
        }
    }
    
    /// 标记块为未加载
    pub fn mark_chunk_unloaded(&mut self, index: usize) {
        if let Some(chunk) = self.get_chunk_info_mut(index) {
            chunk.is_loaded = false;
            chunk.memory_usage = 0;
        }
    }
    
    /// 获取已加载的块数量
    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.iter().filter(|c| c.is_loaded).count()
    }
    
    /// 获取总内存使用量
    pub fn total_memory_usage(&self) -> usize {
        self.chunks.iter().map(|c| c.memory_usage).sum()
    }
    
    /// 获取最久未访问的块
    pub fn get_lru_chunk(&self) -> Option<usize> {
        self.chunks
            .iter()
            .filter(|c| c.is_loaded)
            .min_by_key(|c| c.last_access_time)
            .map(|c| c.index)
    }
}

/// 自适应块大小计算器
pub struct AdaptiveChunkSizer {
    config: ChunkLoaderConfig,
}

impl AdaptiveChunkSizer {
    pub fn new(config: ChunkLoaderConfig) -> Self {
        Self { config }
    }
    
    /// 根据文件大小计算最优块大小
    pub fn calculate_optimal_chunk_size(&self, total_lines: usize) -> usize {
        if !self.config.adaptive_chunk_size {
            return self.config.default_chunk_size;
        }
        
        let chunk_size = if total_lines > 100_000 {
            // 超大文件：使用较小的块大小
            std::cmp::max(
                std::cmp::min(total_lines / 1000, self.config.max_chunk_size),
                self.config.min_chunk_size
            )
        } else if total_lines > 10_000 {
            // 大文件：使用中等块大小
            std::cmp::max(
                std::cmp::min(total_lines / 100, self.config.max_chunk_size),
                self.config.min_chunk_size
            )
        } else {
            // 普通文件：使用默认块大小
            self.config.default_chunk_size
        };
        
        // 确保块大小在配置范围内
        std::cmp::max(
            std::cmp::min(chunk_size, self.config.max_chunk_size),
            self.config.min_chunk_size
        )
    }
}

/// 内存管理器
pub struct MemoryManager {
    config: ChunkLoaderConfig,
    pub current_usage: usize,
    max_usage: usize,
    gc_count: usize,
    last_gc_time: std::time::SystemTime,
}

impl MemoryManager {
    pub fn new(config: ChunkLoaderConfig) -> Self {
        Self {
            config,
            current_usage: 0,
            max_usage: 0,
            gc_count: 0,
            last_gc_time: std::time::SystemTime::now(),
        }
    }
    
    /// 添加内存使用量
    pub fn add_memory_usage(&mut self, usage: usize) {
        self.current_usage += usage;
        if self.current_usage > self.max_usage {
            self.max_usage = self.current_usage;
        }
    }
    
    /// 减少内存使用量
    pub fn subtract_memory_usage(&mut self, usage: usize) {
        self.current_usage = self.current_usage.saturating_sub(usage);
    }
    
    /// 检查是否需要进行垃圾回收
    pub fn should_gc(&self) -> bool {
        self.current_usage > self.config.gc_threshold
    }
    
    /// 获取内存压力级别
    pub fn get_pressure_level(&self) -> MemoryPressureLevel {
        let usage_ratio = self.current_usage as f64 / self.config.memory_threshold as f64;
        
        if usage_ratio > 0.9 {
            MemoryPressureLevel::Critical
        } else if usage_ratio > 0.7 {
            MemoryPressureLevel::High
        } else if usage_ratio > 0.5 {
            MemoryPressureLevel::Moderate
        } else {
            MemoryPressureLevel::Normal
        }
    }
    
    /// 执行垃圾回收
    pub fn gc(&mut self) {
        self.gc_count += 1;
        self.last_gc_time = std::time::SystemTime::now();
    }
    
    /// 获取内存信息
    pub fn get_memory_info(&self, cached_chunks: usize, max_cached_chunks: usize) -> MemoryInfo {
        MemoryInfo {
            current_usage: self.current_usage,
            max_usage: self.max_usage,
            cached_chunks,
            max_cached_chunks,
            last_gc_time: self.last_gc_time,
            gc_count: self.gc_count,
            pressure_level: self.get_pressure_level(),
        }
    }
}