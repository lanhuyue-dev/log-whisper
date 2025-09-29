use std::collections::HashMap;
use std::sync::RwLock;
use crate::models::ParseResult;

/// 解析缓存
pub struct ParseCache {
    cache: RwLock<HashMap<String, Vec<ParseResult>>>,
    max_size: usize,
    max_entries: usize,
}

impl ParseCache {
    /// 创建新的解析缓存
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            max_size: 100, // 最大缓存条目数
            max_entries: 1000, // 最大日志条目数
        }
    }
    
    /// 设置最大缓存大小
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }
    
    /// 设置最大日志条目数
    pub fn with_max_entries(mut self, max_entries: usize) -> Self {
        self.max_entries = max_entries;
        self
    }
    
    /// 生成缓存键
    fn generate_cache_key(&self, file_path: &str, plugin_name: &str) -> String {
        format!("{}:{}", file_path, plugin_name)
    }
    
    /// 获取缓存结果
    pub fn get(&self, file_path: &str, plugin_name: &str) -> Option<Vec<ParseResult>> {
        let cache_key = self.generate_cache_key(file_path, plugin_name);
        
        if let Ok(cache_guard) = self.cache.read() {
            cache_guard.get(&cache_key).cloned()
        } else {
            None
        }
    }
    
    /// 设置缓存结果
    pub fn set(&self, file_path: &str, plugin_name: &str, results: Vec<ParseResult>) -> Result<(), CacheError> {
        let cache_key = self.generate_cache_key(file_path, plugin_name);
        
        // 检查缓存大小限制
        if results.len() > self.max_entries {
            return Err(CacheError::TooManyEntries(results.len(), self.max_entries));
        }
        
        if let Ok(mut cache_guard) = self.cache.write() {
            // 检查缓存条目数限制
            if cache_guard.len() >= self.max_size {
                // 简单的LRU策略：清除最旧的条目
                if let Some(oldest_key) = cache_guard.keys().next().cloned() {
                    cache_guard.remove(&oldest_key);
                }
            }
            
            cache_guard.insert(cache_key, results);
            Ok(())
        } else {
            Err(CacheError::LockError)
        }
    }
    
    /// 检查缓存是否存在
    pub fn contains(&self, file_path: &str, plugin_name: &str) -> bool {
        let cache_key = self.generate_cache_key(file_path, plugin_name);
        
        if let Ok(cache_guard) = self.cache.read() {
            cache_guard.contains_key(&cache_key)
        } else {
            false
        }
    }
    
    /// 移除缓存条目
    pub fn remove(&self, file_path: &str, plugin_name: &str) -> bool {
        let cache_key = self.generate_cache_key(file_path, plugin_name);
        
        if let Ok(mut cache_guard) = self.cache.write() {
            cache_guard.remove(&cache_key).is_some()
        } else {
            false
        }
    }
    
    /// 清空所有缓存
    pub fn clear(&self) -> Result<(), CacheError> {
        if let Ok(mut cache_guard) = self.cache.write() {
            cache_guard.clear();
            Ok(())
        } else {
            Err(CacheError::LockError)
        }
    }
    
    /// 获取缓存统计信息
    pub fn get_stats(&self) -> CacheStats {
        if let Ok(cache_guard) = self.cache.read() {
            let total_entries = cache_guard.len();
            let total_results: usize = cache_guard.values().map(|v| v.len()).sum();
            
            CacheStats {
                total_cached_files: total_entries,
                total_cached_results: total_results,
                max_size: self.max_size,
                max_entries: self.max_entries,
            }
        } else {
            CacheStats {
                total_cached_files: 0,
                total_cached_results: 0,
                max_size: self.max_size,
                max_entries: self.max_entries,
            }
        }
    }
    
    /// 清理过期缓存
    pub fn cleanup_expired(&self) -> Result<usize, CacheError> {
        // 简单的清理策略：如果缓存超过最大大小，移除最旧的条目
        if let Ok(mut cache_guard) = self.cache.write() {
            let mut removed_count = 0;
            
            while cache_guard.len() > self.max_size {
                if let Some(oldest_key) = cache_guard.keys().next().cloned() {
                    cache_guard.remove(&oldest_key);
                    removed_count += 1;
                } else {
                    break;
                }
            }
            
            Ok(removed_count)
        } else {
            Err(CacheError::LockError)
        }
    }
}

/// 缓存错误类型
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("缓存锁定错误")]
    LockError,
    
    #[error("条目数量过多: {0} (最大: {1})")]
    TooManyEntries(usize, usize),
    
    #[error("缓存已满")]
    CacheFull,
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_cached_files: usize,
    pub total_cached_results: usize,
    pub max_size: usize,
    pub max_entries: usize,
}

impl CacheStats {
    /// 获取缓存使用率
    pub fn get_usage_rate(&self) -> f32 {
        if self.max_size > 0 {
            self.total_cached_files as f32 / self.max_size as f32
        } else {
            0.0
        }
    }
    
    /// 检查缓存是否已满
    pub fn is_full(&self) -> bool {
        self.total_cached_files >= self.max_size
    }
}

impl Default for ParseCache {
    fn default() -> Self {
        Self::new()
    }
}
