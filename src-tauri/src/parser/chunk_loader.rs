use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::{info, warn, error, debug, trace};

use crate::models::{
    ParseResult, ParseError, ChunkLoaderConfig, ChunkInfo, ChunkLoadRequest, 
    ChunkLoadResponse, ChunkMetadata, ChunkPriority, MemoryManager, MemoryInfo,
    AdaptiveChunkSizer, MemoryPressureLevel
};
use crate::parser::{LogParser, FileReader};

/// 分块加载器
pub struct ChunkLoader {
    /// 配置
    config: ChunkLoaderConfig,
    /// 内存管理器
    memory_manager: Arc<Mutex<MemoryManager>>,
    /// 自适应块大小计算器
    chunk_sizer: AdaptiveChunkSizer,
    /// 缓存的块数据
    chunk_cache: Arc<Mutex<HashMap<String, HashMap<usize, Vec<ParseResult>>>>>,
    /// 文件元数据缓存
    metadata_cache: Arc<Mutex<HashMap<String, ChunkMetadata>>>,
    /// 日志解析器
    parser: Arc<LogParser>,
    /// 文件读取器
    file_reader: FileReader,
}

impl ChunkLoader {
    /// 创建新的分块加载器
    pub fn new(config: ChunkLoaderConfig) -> Self {
        let chunk_sizer = AdaptiveChunkSizer::new(config.clone());
        let memory_manager = Arc::new(Mutex::new(MemoryManager::new(config.clone())));
        
        Self {
            config: config.clone(),
            memory_manager,
            chunk_sizer,
            chunk_cache: Arc::new(Mutex::new(HashMap::new())),
            metadata_cache: Arc::new(Mutex::new(HashMap::new())),
            parser: Arc::new(LogParser::new()),
            file_reader: FileReader::new(),
        }
    }
    
    /// 使用自定义解析器创建分块加载器
    pub fn with_parser(config: ChunkLoaderConfig, parser: Arc<LogParser>) -> Self {
        let chunk_sizer = AdaptiveChunkSizer::new(config.clone());
        let memory_manager = Arc::new(Mutex::new(MemoryManager::new(config.clone())));
        
        Self {
            config: config.clone(),
            memory_manager,
            chunk_sizer,
            chunk_cache: Arc::new(Mutex::new(HashMap::new())),
            metadata_cache: Arc::new(Mutex::new(HashMap::new())),
            parser,
            file_reader: FileReader::new(),
        }
    }
    
    /// 初始化文件的分块元数据
    pub async fn initialize_file_chunks(&self, file_path: &str) -> Result<ChunkMetadata, ParseError> {
        info!("初始化文件分块: {}", file_path);
        
        // 获取文件信息
        let file_info = self.file_reader.get_file_info(file_path).await
            .map_err(|e| ParseError::FileReadError(e.to_string()))?;
        
        // 读取文件行数
        let lines = self.file_reader.read_lines(file_path).await
            .map_err(|e| ParseError::FileReadError(e.to_string()))?;
        
        let total_lines = lines.len();
        let file_size = file_info.size;
        
        // 计算最优块大小
        let chunk_size = self.chunk_sizer.calculate_optimal_chunk_size(total_lines);
        info!("文件 {} 使用块大小: {}, 总行数: {}", file_path, chunk_size, total_lines);
        
        // 创建元数据
        let metadata = ChunkMetadata::new(
            file_path.to_string(),
            total_lines,
            chunk_size,
            file_size
        );
        
        // 缓存元数据
        {
            let mut cache = self.metadata_cache.lock().unwrap();
            cache.insert(file_path.to_string(), metadata.clone());
        }
        
        debug!("文件分块初始化完成: {} 块", metadata.total_chunks);
        Ok(metadata)
    }
    
    /// 加载指定的数据块
    pub async fn load_chunks(&self, request: ChunkLoadRequest) -> Result<ChunkLoadResponse, ParseError> {
        info!("开始加载数据块: {} 个块", request.chunk_indices.len());
        debug!("加载请求: {:?}", request);
        
        // 检查内存压力
        let memory_info = self.get_memory_info().await;
        if memory_info.pressure_level == MemoryPressureLevel::Critical {
            warn!("内存压力严重，触发GC");
            self.force_gc().await?;
        }
        
        // 获取文件元数据
        let metadata = self.get_or_create_metadata(&request.file_path).await?;
        
        let mut loaded_chunks = HashMap::new();
        let mut chunk_infos = Vec::new();
        let mut total_memory_usage = 0;
        
        // 按优先级排序块索引
        let mut sorted_indices = request.chunk_indices.clone();
        if request.priority == ChunkPriority::Immediate {
            // 立即加载模式下，按顺序加载
        } else {
            // 其他模式下，可以按某种策略排序
            sorted_indices.sort();
        }
        
        for chunk_index in sorted_indices {
            if chunk_index >= metadata.total_chunks {
                warn!("块索引超出范围: {} >= {}", chunk_index, metadata.total_chunks);
                continue;
            }
            
            // 检查是否已在缓存中
            if let Some(cached_data) = self.get_cached_chunk(&request.file_path, chunk_index).await {
                loaded_chunks.insert(chunk_index, cached_data);
                if let Some(chunk_info) = metadata.get_chunk_info(chunk_index) {
                    chunk_infos.push(chunk_info.clone());
                    total_memory_usage += chunk_info.memory_usage;
                }
                debug!("使用缓存的块: {}", chunk_index);
                continue;
            }
            
            // 加载新块
            match self.load_single_chunk(&request.file_path, chunk_index, &request.plugin_name).await {
                Ok((chunk_data, memory_usage)) => {
                    loaded_chunks.insert(chunk_index, chunk_data.clone());
                    
                    // 缓存块数据
                    self.cache_chunk(&request.file_path, chunk_index, chunk_data).await;
                    
                    // 更新元数据
                    {
                        let mut cache = self.metadata_cache.lock().unwrap();
                        if let Some(meta) = cache.get_mut(&request.file_path) {
                            meta.mark_chunk_loaded(chunk_index, memory_usage);
                            if let Some(chunk_info) = meta.get_chunk_info(chunk_index) {
                                chunk_infos.push(chunk_info.clone());
                            }
                        }
                    }
                    
                    // 更新内存使用
                    {
                        let mut mem_mgr = self.memory_manager.lock().unwrap();
                        mem_mgr.add_memory_usage(memory_usage);
                    }
                    total_memory_usage += memory_usage;
                    
                    debug!("成功加载块: {}, 内存使用: {} bytes", chunk_index, memory_usage);
                }
                Err(e) => {
                    error!("加载块失败: {}, 错误: {}", chunk_index, e);
                    return Ok(ChunkLoadResponse {
                        success: false,
                        chunks: HashMap::new(),
                        chunk_infos: Vec::new(),
                        total_memory_usage: 0,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        
        info!("块加载完成: {} 个块, 总内存使用: {} bytes", loaded_chunks.len(), total_memory_usage);
        
        Ok(ChunkLoadResponse {
            success: true,
            chunks: loaded_chunks,
            chunk_infos,
            total_memory_usage,
            error: None,
        })
    }
    
    /// 加载单个数据块
    async fn load_single_chunk(&self, file_path: &str, chunk_index: usize, plugin_name: &str) -> Result<(Vec<ParseResult>, usize), ParseError> {
        debug!("加载单个块: {} - {}", file_path, chunk_index);
        
        // 获取元数据
        let metadata = self.get_metadata(file_path).await
            .ok_or_else(|| ParseError::ConfigError("文件元数据未找到".to_string()))?;
        
        let chunk_info = metadata.get_chunk_info(chunk_index)
            .ok_or_else(|| ParseError::ConfigError("块信息未找到".to_string()))?;
        
        // 读取块数据
        let _lines = self.file_reader.read_file_range(
            file_path,
            chunk_info.start_line as u64,
            chunk_info.end_line as u64
        ).map_err(|e| ParseError::FileReadError(e.to_string()))?;
        
        // 解析块数据
        let results = if plugin_name == "Auto" {
            self.parser.parse_file_stream(file_path)
                .map_err(|e| ParseError::ParseError(e.to_string()))?
        } else {
            // 这里应该使用指定插件解析，简化实现
            self.parser.parse_file_stream(file_path) 
                .map_err(|e| ParseError::ParseError(e.to_string()))?
        };
        
        // 估算内存使用量
        let memory_usage = self.estimate_memory_usage(&results);
        
        trace!("块 {} 解析完成: {} 条结果, {} bytes", chunk_index, results.len(), memory_usage);
        Ok((results, memory_usage))
    }
    
    /// 获取或创建文件元数据
    async fn get_or_create_metadata(&self, file_path: &str) -> Result<ChunkMetadata, ParseError> {
        // 先尝试从缓存获取
        if let Some(metadata) = self.get_metadata(file_path).await {
            return Ok(metadata);
        }
        
        // 如果缓存中没有，则初始化
        self.initialize_file_chunks(file_path).await
    }
    
    /// 获取文件元数据
    async fn get_metadata(&self, file_path: &str) -> Option<ChunkMetadata> {
        let cache = self.metadata_cache.lock().unwrap();
        cache.get(file_path).cloned()
    }
    
    /// 获取缓存的块数据
    async fn get_cached_chunk(&self, file_path: &str, chunk_index: usize) -> Option<Vec<ParseResult>> {
        let cache = self.chunk_cache.lock().unwrap();
        cache.get(file_path)?.get(&chunk_index).cloned()
    }
    
    /// 缓存块数据
    async fn cache_chunk(&self, file_path: &str, chunk_index: usize, data: Vec<ParseResult>) {
        let mut cache = self.chunk_cache.lock().unwrap();
        let file_cache = cache.entry(file_path.to_string()).or_insert_with(HashMap::new);
        file_cache.insert(chunk_index, data);
    }
    
    /// 清理最久未使用的块
    pub async fn cleanup_lru_chunks(&self, count: usize) -> Result<usize, ParseError> {
        debug!("开始清理LRU块: {} 个", count);
        let mut cleaned_count = 0;
        
        // 收集所有文件的LRU块
        let mut lru_chunks = Vec::new();
        {
            let metadata_cache = self.metadata_cache.lock().unwrap();
            for (file_path, metadata) in metadata_cache.iter() {
                if let Some(lru_index) = metadata.get_lru_chunk() {
                    lru_chunks.push((file_path.clone(), lru_index, metadata.chunks[lru_index].last_access_time));
                }
            }
        }
        
        // 按访问时间排序
        lru_chunks.sort_by_key(|(_, _, time)| *time);
        
        // 清理最久未使用的块
        for (file_path, chunk_index, _) in lru_chunks.into_iter().take(count) {
            if self.unload_chunk(&file_path, chunk_index).await.is_ok() {
                cleaned_count += 1;
                debug!("清理块: {} - {}", file_path, chunk_index);
            }
        }
        
        info!("LRU清理完成: {} 个块", cleaned_count);
        Ok(cleaned_count)
    }
    
    /// 卸载块
    async fn unload_chunk(&self, file_path: &str, chunk_index: usize) -> Result<(), ParseError> {
        // 从缓存中移除
        {
            let mut cache = self.chunk_cache.lock().unwrap();
            if let Some(file_cache) = cache.get_mut(file_path) {
                file_cache.remove(&chunk_index);
            }
        }
        
        // 更新元数据
        {
            let mut metadata_cache = self.metadata_cache.lock().unwrap();
            if let Some(metadata) = metadata_cache.get_mut(file_path) {
                let memory_usage = metadata.get_chunk_info(chunk_index)
                    .map(|info| info.memory_usage)
                    .unwrap_or(0);
                
                metadata.mark_chunk_unloaded(chunk_index);
                
                // 更新内存管理器
                {
                    let mut mem_mgr = self.memory_manager.lock().unwrap();
                    mem_mgr.subtract_memory_usage(memory_usage);
                }
            }
        }
        
        Ok(())
    }
    
    /// 强制垃圾回收
    pub async fn force_gc(&self) -> Result<usize, ParseError> {
        info!("开始强制垃圾回收");
        
        let memory_info = self.get_memory_info().await;
        let target_cleanup = memory_info.cached_chunks / 2; // 清理一半的缓存
        
        let cleaned = self.cleanup_lru_chunks(target_cleanup).await?;
        
        // 更新GC统计
        {
            let mut mem_mgr = self.memory_manager.lock().unwrap();
            mem_mgr.gc();
        }
        
        info!("垃圾回收完成: 清理了 {} 个块", cleaned);
        Ok(cleaned)
    }
    
    /// 获取内存信息
    pub async fn get_memory_info(&self) -> MemoryInfo {
        let mem_mgr = self.memory_manager.lock().unwrap();
        let cached_chunks = {
            let cache = self.chunk_cache.lock().unwrap();
            cache.values().map(|file_cache| file_cache.len()).sum()
        };
        
        mem_mgr.get_memory_info(cached_chunks, self.config.max_chunk_size)
    }
    
    /// 估算内存使用量
    fn estimate_memory_usage(&self, results: &[ParseResult]) -> usize {
        // 粗略估算，每个ParseResult约占用1KB
        results.len() * 1024
    }
    
    /// 清理所有缓存
    pub async fn clear_all_cache(&self) -> Result<(), ParseError> {
        info!("清理所有缓存");
        
        {
            let mut cache = self.chunk_cache.lock().unwrap();
            cache.clear();
        }
        
        {
            let mut metadata_cache = self.metadata_cache.lock().unwrap();
            metadata_cache.clear();
        }
        
        {
            let mut mem_mgr = self.memory_manager.lock().unwrap();
            let current = mem_mgr.current_usage;
            mem_mgr.subtract_memory_usage(current);
        }
        
        info!("缓存清理完成");
        Ok(())
    }
    
    /// 获取块状态信息
    pub async fn get_chunk_status(&self, file_path: &str) -> Option<Vec<ChunkInfo>> {
        let metadata_cache = self.metadata_cache.lock().unwrap();
        metadata_cache.get(file_path).map(|metadata| metadata.chunks.clone())
    }
    
    /// 预加载块
    pub async fn preload_chunks(&self, file_path: &str, start_chunk: usize, plugin_name: &str) -> Result<usize, ParseError> {
        let metadata = self.get_or_create_metadata(file_path).await?;
        let end_chunk = std::cmp::min(
            start_chunk + self.config.preload_chunk_count,
            metadata.total_chunks
        );
        
        let chunk_indices: Vec<usize> = (start_chunk..end_chunk).collect();
        let request = ChunkLoadRequest {
            file_path: file_path.to_string(),
            chunk_indices: chunk_indices.clone(),
            plugin_name: plugin_name.to_string(),
            priority: ChunkPriority::Low,
        };
        
        let response = self.load_chunks(request).await?;
        
        if response.success {
            info!("预加载完成: {} 个块", chunk_indices.len());
            Ok(chunk_indices.len())
        } else {
            error!("预加载失败: {}", response.error.unwrap_or_default());
            Ok(0)
        }
    }
}