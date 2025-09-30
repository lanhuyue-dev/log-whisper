use std::sync::Arc;
use std::time::Instant;
use crate::models::{ParseResult, ParseResultSet, ParseConfig, ParseError};
use crate::parser::{FileReader, LineParser, RenderEngine, ParseCache};
use crate::plugins::PluginRegistry;
use log::{info, warn, error, debug, trace};

/// 流式解析迭代器
pub struct StreamingParseIterator {
    line_iterator: Box<dyn Iterator<Item = Result<String, ParseError>>>,
    line_parser: LineParser,
    render_engine: RenderEngine,
    config: ParseConfig,
    line_number: usize,
}

impl StreamingParseIterator {
    pub fn new(
        line_iterator: Box<dyn Iterator<Item = Result<String, ParseError>>>,
        line_parser: LineParser,
        render_engine: RenderEngine,
        config: ParseConfig,
    ) -> Self {
        Self {
            line_iterator,
            line_parser,
            render_engine,
            config,
            line_number: 0,
        }
    }
}

impl Iterator for StreamingParseIterator {
    type Item = Result<ParseResult, ParseError>;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.line_iterator.next() {
            Some(Ok(line)) => {
                self.line_number += 1;
                
                // 解析日志条目
                match self.line_parser.parse_line(self.line_number, &line) {
                    Ok(entry) => {
                        // 渲染结果
                        if self.config.plugin_name == "Auto" {
                            match self.render_engine.render_entry(&entry) {
                                Ok(result) => Some(Ok(result)),
                                Err(e) => Some(Err(e.into())),
                            }
                        } else {
                            match self.render_engine.render_entry_with_plugin(&entry, &self.config.plugin_name) {
                                Ok(result) => Some(Ok(result)),
                                Err(e) => Some(Err(e.into())),
                            }
                        }
                    }
                    Err(e) => Some(Err(e))
                }
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
}

/// 日志解析器
pub struct LogParser {
    file_reader: FileReader,
    line_parser: LineParser,
    render_engine: RenderEngine,
    cache: ParseCache,
    config: ParseConfig,
}

impl LogParser {
    /// 创建新的日志解析器
    pub fn new() -> Self {
        let plugin_registry = Arc::new(PluginRegistry::new());
        let render_engine = RenderEngine::new(plugin_registry);
        
        Self {
            file_reader: FileReader::new(),
            line_parser: LineParser::new(),
            render_engine,
            cache: ParseCache::new(),
            config: ParseConfig::default(),
        }
    }
    
    /// 使用配置创建日志解析器
    pub fn with_config(config: ParseConfig) -> Self {
        let plugin_registry = Arc::new(PluginRegistry::new());
        let render_engine = RenderEngine::new(plugin_registry);
        
        Self {
            file_reader: FileReader::new()
                .with_max_file_size(config.max_file_size),
            line_parser: LineParser::new(),
            render_engine,
            cache: ParseCache::new(),
            config,
        }
    }
    
    /// 解析文件
    pub async fn parse_file(&self, file_path: &str) -> Result<ParseResultSet, ParseError> {
        info!("开始解析文件: {}", file_path);
        debug!("解析配置: {:?}", self.config);
        
        let start_time = Instant::now();
        
        // 验证文件类型
        debug!("验证文件类型: {}", file_path);
        self.file_reader.validate_file_type(file_path)?;
        trace!("文件类型验证通过");
        
        // 检查缓存
        if self.config.enable_cache {
            debug!("检查缓存: {}", file_path);
            if let Some(cached_results) = self.cache.get(file_path, &self.config.plugin_name) {
                info!("使用缓存结果，条目数: {}", cached_results.len());
                let mut result_set = ParseResultSet::new(self.config.clone());
                result_set.add_results(cached_results);
                return Ok(result_set);
            }
            trace!("缓存未命中，继续解析");
        }
        
        // 读取文件
        debug!("读取文件内容: {}", file_path);
        let lines = self.file_reader.read_lines(file_path).await?;
        info!("文件读取完成，总行数: {}", lines.len());
        trace!("文件内容预览: {:?}", lines.iter().take(3).collect::<Vec<_>>());
        
        // 解析日志条目
        debug!("开始解析日志条目");
        let log_entries = self.line_parser.parse_lines(lines)?;
        info!("日志条目解析完成，条目数: {}", log_entries.len());
        trace!("解析的日志条目: {:?}", log_entries.iter().take(2).collect::<Vec<_>>());
        
        // 合并多行日志
        debug!("合并多行日志");
        let merged_entries = self.line_parser.merge_multiline_logs(log_entries);
        info!("多行日志合并完成，最终条目数: {}", merged_entries.len());
        
        // 渲染结果
        debug!("开始渲染结果，使用插件: {}", self.config.plugin_name);
        let parse_results = if self.config.plugin_name == "Auto" {
            debug!("使用自动插件选择");
            self.render_engine.render_entries(merged_entries)?
        } else {
            debug!("使用指定插件: {}", self.config.plugin_name);
            self.render_engine.render_entries_with_plugin(merged_entries, &self.config.plugin_name)?
        };
        info!("渲染完成，结果数: {}", parse_results.len());
        trace!("渲染结果预览: {:?}", parse_results.iter().take(2).collect::<Vec<_>>());
        
        // 创建结果集
        debug!("创建结果集");
        let mut result_set = ParseResultSet::new(self.config.clone());
        result_set.add_results(parse_results);
        
        // 更新总体统计信息
        let total_parse_time = start_time.elapsed().as_millis() as u64;
        result_set.total_stats.total_parse_time_ms = total_parse_time;
        info!("解析完成，总耗时: {}ms", total_parse_time);
        debug!("解析统计: {:?}", result_set.total_stats);
        
        // 缓存结果
        if self.config.enable_cache {
            debug!("缓存解析结果");
            let _ = self.cache.set(file_path, &self.config.plugin_name, result_set.results.clone());
            trace!("结果已缓存");
        }
        
        Ok(result_set)
    }
    
    /// 同步解析文件
    pub fn parse_file_sync(&self, file_path: &str) -> Result<ParseResultSet, ParseError> {
        let start_time = Instant::now();
        
        // 验证文件类型
        self.file_reader.validate_file_type(file_path)?;
        
        // 检查缓存
        if self.config.enable_cache {
            if let Some(cached_results) = self.cache.get(file_path, &self.config.plugin_name) {
                let mut result_set = ParseResultSet::new(self.config.clone());
                result_set.add_results(cached_results);
                return Ok(result_set);
            }
        }
        
        // 读取文件
        let lines = self.file_reader.read_lines_sync(file_path)?;
        
        // 解析日志条目
        let log_entries = self.line_parser.parse_lines(lines)?;
        
        // 合并多行日志
        let merged_entries = self.line_parser.merge_multiline_logs(log_entries);
        
        // 渲染结果
        let parse_results = if self.config.plugin_name == "Auto" {
            self.render_engine.render_entries(merged_entries)?
        } else {
            self.render_engine.render_entries_with_plugin(merged_entries, &self.config.plugin_name)?
        };
        
        // 创建结果集
        let mut result_set = ParseResultSet::new(self.config.clone());
        result_set.add_results(parse_results);
        
        // 更新总体统计信息
        let total_parse_time = start_time.elapsed().as_millis() as u64;
        result_set.total_stats.total_parse_time_ms = total_parse_time;
        
        // 缓存结果
        if self.config.enable_cache {
            let _ = self.cache.set(file_path, &self.config.plugin_name, result_set.results.clone());
        }
        
        Ok(result_set)
    }
    
    /// 流式解析文件（支持大文件）
    pub async fn parse_file_streaming(&self, file_path: &str) -> Result<Box<dyn Iterator<Item = Result<ParseResult, ParseError>>>, ParseError> {
        info!("开始流式解析文件: {}", file_path);
        debug!("解析配置: {:?}", self.config);
        
        // 验证文件类型
        self.file_reader.validate_file_type(file_path)?;
        
        // 获取流式读取迭代器
        let line_iterator = self.file_reader.read_lines_smart(file_path).await?;
        
        // 在这里我们直接使用简化的解析方式
        let iter = line_iterator.enumerate().filter_map(move |(index, line_result)| {
            match line_result {
                Ok(line) => {
                    // 简单解析 - 创建基本的ParseResult
                    let entry = crate::models::LogEntry::new(index + 1, line);
                    let parse_result = crate::models::ParseResult::new(entry);
                    Some(Ok(parse_result))
                }
                Err(e) => Some(Err(e))
            }
        });
        
        Ok(Box::new(iter))
    }
    
    /// 分块解析文件（用于虚拟滚动）
    pub async fn parse_file_chunked(&self, file_path: &str, chunk_count: usize) -> Result<Vec<ParseResult>, ParseError> {
        info!("开始分块解析文件: {}, 块数量: {}", file_path, chunk_count);
        
        // 验证文件类型
        self.file_reader.validate_file_type(file_path)?;
        
        // 获取文件分块信息
        let chunks = self.file_reader.get_file_chunks(file_path, chunk_count)?;
        info!("文件分为 {} 个块", chunks.len());
        
        let mut results = Vec::new();
        
        // 处理每个块
        for (chunk_index, chunk) in chunks.iter().enumerate() {
            debug!("处理第 {} 个块: {} - {}", chunk_index, chunk.start_offset, chunk.end_offset);
            
            // 读取块内容
            let lines = self.file_reader.read_file_range(file_path, chunk.start_offset, chunk.end_offset)?;
            
            // 解析日志条目
            let log_entries = self.line_parser.parse_lines(lines)?;
            
            // 合并多行日志
            let merged_entries = self.line_parser.merge_multiline_logs(log_entries);
            
            // 渲染结果
            let chunk_results = if self.config.plugin_name == "Auto" {
                self.render_engine.render_entries(merged_entries)?
            } else {
                self.render_engine.render_entries_with_plugin(merged_entries, &self.config.plugin_name)?
            };
            
            results.extend(chunk_results);
        }
        
        info!("分块解析完成，总结果数: {}", results.len());
        Ok(results)
    }
    
    /// 流式解析文件
    pub fn parse_file_stream(&self, file_path: &str) -> Result<Vec<ParseResult>, ParseError> {
        // 验证文件类型
        self.file_reader.validate_file_type(file_path)?;
        
        // 读取文件流
        let lines = self.file_reader.read_file_stream(file_path)?;
        
        // 解析和渲染每一行
        let results: Vec<ParseResult> = lines.enumerate().filter_map(|(index, line)| {
            let entry = self.line_parser.parse_line(index + 1, &line).ok()?;
            
            if self.config.plugin_name == "Auto" {
                self.render_engine.render_entry(&entry).ok()
            } else {
                self.render_engine.render_entry_with_plugin(&entry, &self.config.plugin_name).ok()
            }
        }).collect();
        
        Ok(results)
    }
    
    /// 获取文件信息
    pub async fn get_file_info(&self, file_path: &str) -> Result<crate::parser::FileInfo, ParseError> {
        self.file_reader.get_file_info(file_path).await
    }
    
    /// 获取可用的插件列表
    pub fn get_available_plugins(&self) -> Vec<String> {
        self.render_engine.get_available_plugins()
    }
    
    /// 获取启用的插件列表
    pub fn get_enabled_plugins(&self) -> Vec<String> {
        self.render_engine.get_enabled_plugins()
    }
    
    /// 设置插件
    pub fn set_plugin(&mut self, plugin_name: &str) -> Result<(), String> {
        if self.get_available_plugins().contains(&plugin_name.to_string()) {
            self.config.plugin_name = plugin_name.to_string();
            Ok(())
        } else {
            Err(format!("Plugin '{}' not found", plugin_name))
        }
    }
    
    /// 启用缓存
    pub fn enable_cache(&mut self) {
        self.config.enable_cache = true;
    }
    
    /// 禁用缓存
    pub fn disable_cache(&mut self) {
        self.config.enable_cache = false;
    }
    
    /// 清空缓存
    pub fn clear_cache(&self) -> Result<(), crate::parser::CacheError> {
        self.cache.clear()
    }
    
    /// 获取缓存统计信息
    pub fn get_cache_stats(&self) -> crate::parser::CacheStats {
        self.cache.get_stats()
    }
    
    /// 获取渲染统计信息
    pub fn get_render_stats(&self, results: &[ParseResult]) -> crate::parser::RenderStats {
        self.render_engine.get_render_stats(results)
    }
}

impl Default for LogParser {
    fn default() -> Self {
        Self::new()
    }
}
