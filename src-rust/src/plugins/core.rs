//! 插件系统核心实现
//! 
//! 提供插件系统的核心功能和集成

use super::{Plugin, LogEntry, PluginError, PluginManager};
use super::registry::PluginRegistry;
use super::engine::PluginEngine;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 增强的插件管理器
pub struct EnhancedPluginManager {
    registry: Arc<RwLock<PluginRegistry>>,
    engine: Arc<RwLock<PluginEngine>>,
    config: PluginConfig,
}

/// 插件配置
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// 是否启用插件系统
    pub enabled: bool,
    /// 最大插件数量
    pub max_plugins: usize,
    /// 插件执行超时时间（毫秒）
    pub execution_timeout_ms: u64,
    /// 是否启用插件统计
    pub enable_stats: bool,
    /// 插件目录
    pub plugin_directory: String,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_plugins: 50,
            execution_timeout_ms: 5000,
            enable_stats: true,
            plugin_directory: "./plugins".to_string(),
        }
    }
}

impl EnhancedPluginManager {
    /// 创建新的增强插件管理器
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(PluginRegistry::new())),
            engine: Arc::new(RwLock::new(PluginEngine::new())),
            config: PluginConfig::default(),
        }
    }
    
    /// 使用配置创建插件管理器
    pub fn with_config(config: PluginConfig) -> Self {
        Self {
            registry: Arc::new(RwLock::new(PluginRegistry::new())),
            engine: Arc::new(RwLock::new(PluginEngine::new())),
            config,
        }
    }
    
    /// 初始化插件系统
    pub async fn initialize(&self) -> Result<(), PluginError> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // 注册内置插件
        self.register_builtin_plugins().await?;
        
        // 加载外部插件
        self.load_external_plugins().await?;
        
        Ok(())
    }
    
    /// 注册内置插件
    async fn register_builtin_plugins(&self) -> Result<(), PluginError> {
        use super::builtin::*;
        use super::springboot::*;
        use super::renderers::*;
        
        // 按优先级注册插件，确保正确的执行顺序
        
        // 1. 解析器阶段（优先级：10-20）
        // Docker JSON解析器（最高优先级）
        let docker_parser = Box::new(DockerJsonParser::new());
        self.registry.write().await.register(docker_parser).await?;
        
        // SpringBoot JSON解析器
        let springboot_json_parser = Box::new(SpringBootJsonParser::new());
        self.registry.write().await.register(springboot_json_parser).await?;
        
        // SpringBoot标准解析器
        let springboot_parser = Box::new(SpringBootParser::new()
            .map_err(|e| PluginError::InitializationFailed(format!("SpringBoot解析器初始化失败: {}", e)))?);
        self.registry.write().await.register(springboot_parser).await?;
        
        // MyBatis解析器
        let mybatis_parser = Box::new(MyBatisParser::new());
        self.registry.write().await.register(mybatis_parser).await?;
        
        // 2. 聚合器阶段（优先级：30-40）
        // SpringBoot异常聚合器
        let springboot_exception_aggregator = Box::new(SpringBootExceptionAggregator::new());
        self.registry.write().await.register(springboot_exception_aggregator).await?;
        
        // 通用堆栈跟踪聚合器
        let stack_aggregator = Box::new(StackTraceAggregator::new());
        self.registry.write().await.register(stack_aggregator).await?;
        
        // 3. 渲染器阶段（优先级：50-60）
        // JSON格式化器
        let json_formatter = Box::new(JsonFormatter::new());
        self.registry.write().await.register(json_formatter).await?;
        
        // 注册渲染器插件
        let stack_trace_renderer = Box::new(StackTraceRenderer::new());
        self.registry.write().await.register(stack_trace_renderer).await?;
        
        let sql_renderer = Box::new(SqlRenderer::new());
        self.registry.write().await.register(sql_renderer).await?;
        
        let error_highlighter = Box::new(ErrorHighlighter::new());
        self.registry.write().await.register(error_highlighter).await?;
        
        Ok(())
    }
    
    /// 加载外部插件
    async fn load_external_plugins(&self) -> Result<(), PluginError> {
        // TODO: 实现外部插件加载
        // 这里可以加载动态库插件或脚本插件
        Ok(())
    }
    
    /// 处理日志条目（优化版本）
    pub async fn process_log_entry(&self, mut log_entry: LogEntry) -> Result<LogEntry, PluginError> {
        if !self.config.enabled {
            return Ok(log_entry);
        }
        
        println!("🔧 [PluginSystem] 开始处理日志条目: {}", log_entry.content.chars().take(100).collect::<String>());
        
        let registry = self.registry.read().await;
        
        // 按类型分阶段处理，确保正确的执行顺序
        // 1. 首先运行解析器（Docker JSON, SpringBoot等）
        let parsers = registry.get_plugins_by_type(super::PluginType::Parser);
        println!("🔧 [PluginSystem] 找到 {} 个解析器插件", parsers.len());
        
        for plugin in parsers {
            println!("🔧 [PluginSystem] 检查解析器: {} - can_handle: {}", 
                     plugin.name(), plugin.can_handle(&log_entry.content));
            if plugin.can_handle(&log_entry.content) {
                println!("🔧 [PluginSystem] 运行解析器: {}", plugin.name());
                plugin.process(&mut log_entry)?;
                // 如果已经解析成功，跳过其他解析器
                if log_entry.is_processed_by("docker_json_parser") || 
                   log_entry.is_processed_by("springboot_parser") ||
                   log_entry.is_processed_by("mybatis") {
                    println!("🔧 [PluginSystem] 解析器处理完成，跳过其他解析器");
                    break;
                }
            }
        }
        
        // 2. 然后运行聚合器（异常堆栈聚合等）- 只对未处理的条目进行聚合
        if !log_entry.is_processed_by("docker_json_parser") {
            let aggregators = registry.get_plugins_by_type(super::PluginType::Filter);
            println!("🔧 [PluginSystem] 找到 {} 个聚合器插件", aggregators.len());
            for plugin in aggregators {
                if plugin.can_handle(&log_entry.content) {
                    println!("🔧 [PluginSystem] 运行聚合器: {}", plugin.name());
                    plugin.process(&mut log_entry)?;
                    break; // 只运行第一个匹配的聚合器
                }
            }
        }
        
        // 3. 最后运行渲染器（格式化、美化等）- 只对需要格式化的条目进行渲染
        if log_entry.content.len() > 100 || // 只对长内容进行渲染
           log_entry.is_processed_by("mybatis") ||
           log_entry.content.contains('{') {
            let renderers = registry.get_plugins_by_type(super::PluginType::Renderer);
            println!("🔧 [PluginSystem] 找到 {} 个渲染器插件", renderers.len());
            for plugin in renderers {
                if plugin.can_handle(&log_entry.content) {
                    println!("🔧 [PluginSystem] 运行渲染器: {}", plugin.name());
                    plugin.process(&mut log_entry)?;
                    break; // 只运行第一个匹配的渲染器
                }
            }
        }
        
        println!("🔧 [PluginSystem] 处理完成，formatted_content: {:?}", 
                 log_entry.formatted_content.as_ref().map(|s| s.chars().take(50).collect::<String>()));
        
        Ok(log_entry)
    }
    
    /// 批量处理日志条目
    pub async fn process_log_entries(&self, mut log_entries: Vec<LogEntry>) -> Result<Vec<LogEntry>, PluginError> {
        // 首先进行异常聚合处理
        let aggregated_entries = self.aggregate_exceptions(log_entries).await?;
        
        // 然后进行MyBatis SQL和参数合并
        let merged_entries = self.merge_mybatis_sql_params(aggregated_entries).await?;
        
        let mut results = Vec::new();
        
        for log_entry in merged_entries {
            match self.process_log_entry(log_entry).await {
                Ok(processed_entry) => results.push(processed_entry),
                Err(e) => {
                    // 记录错误但继续处理其他条目
                    eprintln!("处理日志条目失败: {}", e);
                }
            }
        }
        
        Ok(results)
    }
    
    /// 聚合异常堆栈跟踪（优化版本）
    async fn aggregate_exceptions(&self, log_entries: Vec<LogEntry>) -> Result<Vec<LogEntry>, PluginError> {
        let mut results = Vec::with_capacity(log_entries.len());
        let mut i = 0;
        
        while i < log_entries.len() {
            let current_entry = &log_entries[i];
            
            // 快速检查：只对SpringBoot格式的异常进行聚合
            if self.is_exception_start(&current_entry.content) && self.is_springboot_exception(&current_entry.content) {
                // 开始聚合异常
                let mut aggregated_content = String::with_capacity(current_entry.content.len() * 3);
                aggregated_content.push_str(&current_entry.content);
                let mut j = i + 1;
                
                // 收集后续的堆栈跟踪行（限制最大聚合行数，避免性能问题）
                let max_aggregate_lines = 50; // 限制最大聚合行数
                while j < log_entries.len() && 
                      (j - i) < max_aggregate_lines && 
                      self.is_exception_continuation(&log_entries[j].content) {
                    aggregated_content.push('\n');
                    aggregated_content.push_str(&log_entries[j].content);
                    j += 1;
                }
                
                // 创建聚合后的日志条目
                let mut aggregated_entry = current_entry.clone();
                aggregated_entry.content = aggregated_content;
                aggregated_entry.add_metadata("aggregated".to_string(), "true".to_string());
                aggregated_entry.add_metadata("stack_lines".to_string(), (j - i).to_string());
                
                results.push(aggregated_entry);
                i = j; // 跳过已聚合的行
            } else {
                // 普通日志行，直接添加
                results.push(current_entry.clone());
                i += 1;
            }
        }
        
        Ok(results)
    }
    
    /// 检查是否是异常开始行
    fn is_exception_start(&self, content: &str) -> bool {
        let trimmed = content.trim();
        trimmed.contains("Exception:") || 
        trimmed.contains("Error:") ||
        trimmed.starts_with("java.lang.") ||
        trimmed.starts_with("org.springframework.") ||
        trimmed.starts_with("com.example.")
    }
    
    /// 检查是否是异常延续行
    fn is_exception_continuation(&self, content: &str) -> bool {
        let trimmed = content.trim();
        trimmed.starts_with("at ") ||
        trimmed.starts_with("\tat ") ||
        trimmed.starts_with("Caused by:") ||
        trimmed.starts_with("Suppressed:") ||
        trimmed.starts_with("\t") && (trimmed.contains("(") && trimmed.contains(")"))
    }
    
    /// 检查是否是SpringBoot异常（排除容器JSON格式）
    fn is_springboot_exception(&self, content: &str) -> bool {
        let trimmed = content.trim();
        
        // 如果是JSON格式，不是SpringBoot异常
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return false;
        }
        
        // 检查是否包含SpringBoot特征
        trimmed.contains("java.lang.") ||
        trimmed.contains("org.springframework.") ||
        trimmed.contains("com.example.") ||
        trimmed.contains("Exception:") ||
        trimmed.contains("Error:")
    }
    
    /// 获取插件统计信息
    pub async fn get_plugin_stats(&self) -> PluginStats {
        let registry = self.registry.read().await;
        let plugin_list = registry.get_plugin_list();
        
        PluginStats {
            total_plugins: plugin_list.len(),
            enabled_plugins: plugin_list.iter().filter(|p| p.enabled).count(),
            plugin_types: self.count_plugin_types(&plugin_list),
        }
    }
    
    /// 统计插件类型
    fn count_plugin_types(&self, plugins: &[super::PluginInfo]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        for plugin in plugins {
            let type_name = format!("{:?}", plugin.plugin_type);
            *counts.entry(type_name).or_insert(0) += 1;
        }
        
        counts
    }
    
    /// 合并MyBatis SQL和参数
    async fn merge_mybatis_sql_params(&self, log_entries: Vec<LogEntry>) -> Result<Vec<LogEntry>, PluginError> {
        let mut results = Vec::new();
        let mut i = 0;
        
        while i < log_entries.len() {
            let current_entry = &log_entries[i];
            
            // 检查是否是MyBatis Preparing SQL
            if current_entry.content.contains("Preparing:") {
                
                println!("🔧 [MyBatis合并] 找到Preparing SQL: {}", current_entry.content.chars().take(100).collect::<String>());
                
                // 提取SQL语句
                let sql = self.extract_sql_from_preparing(&current_entry.content);
                println!("🔧 [MyBatis合并] 提取的SQL: {}", sql.chars().take(100).collect::<String>());
                
                // 查找后续的Parameters行
                let mut j = i + 1;
                let mut found_params = false;
                let mut final_sql = sql.clone();
                
                while j < log_entries.len() {
                    let next_entry = &log_entries[j];
                    
                    if next_entry.content.contains("Parameters:") {
                        println!("🔧 [MyBatis合并] 找到Parameters: {}", next_entry.content.chars().take(100).collect::<String>());
                        
                        // 提取参数
                        let params = self.extract_params_from_parameters(&next_entry.content);
                        println!("🔧 [MyBatis合并] 提取的参数: {:?}", params);
                        
                        // 将参数替换到SQL中
                        final_sql = self.replace_sql_params(&sql, &params);
                        println!("🔧 [MyBatis合并] 最终SQL: {}", final_sql.chars().take(200).collect::<String>());
                        
                        found_params = true;
                        i = j + 1; // 跳过已处理的Parameters行
                        break;
                    } else if next_entry.content.contains("Total:") {
                        // 如果遇到Total行，说明没有Parameters，直接使用SQL
                        i = j; // 处理Total行
                        break;
                    } else {
                        // 不是MyBatis相关的行，停止查找
                        i = j;
                        break;
                    }
                }
                
                if !found_params && j >= log_entries.len() {
                    // 没有找到Parameters，直接使用SQL
                    i += 1;
                }
                
                // 创建合并后的日志条目
                let mut merged_entry = current_entry.clone();
                merged_entry.content = final_sql.clone();
                merged_entry.formatted_content = Some(final_sql);
                merged_entry.add_metadata("merged".to_string(), "true".to_string());
                if found_params {
                    merged_entry.add_metadata("sql_with_params".to_string(), "true".to_string());
                } else {
                    merged_entry.add_metadata("sql_only".to_string(), "true".to_string());
                }
                
                results.push(merged_entry);
            } else {
                // 不是MyBatis Preparing行，直接添加
                results.push(current_entry.clone());
                i += 1;
            }
        }
        
        Ok(results)
    }
    
    /// 从Preparing行提取SQL
    fn extract_sql_from_preparing(&self, content: &str) -> String {
        if let Some(sql_start) = content.find("Preparing:") {
            let sql_part = &content[sql_start + 10..].trim();
            // 清理SQL，移除多余的换行和空格
            let cleaned_sql = sql_part
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<&str>>()
                .join(" ");
            // 保留Preparing:前缀
            format!("Preparing: {}", cleaned_sql)
        } else {
            content.to_string()
        }
    }
    
    /// 从Parameters行提取参数
    fn extract_params_from_parameters(&self, content: &str) -> Vec<String> {
        if let Some(params_start) = content.find("Parameters:") {
            let params_part = &content[params_start + 11..].trim();
            params_part
                .split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// 将参数替换到SQL中
    fn replace_sql_params(&self, sql: &str, params: &[String]) -> String {
        let mut result = sql.to_string();
        let mut param_index = 0;
        
        // 替换SQL中的?占位符为实际参数值
        while let Some(pos) = result.find('?') {
            if param_index < params.len() {
                let param = &params[param_index];
                // 提取参数值（去掉类型信息）
                let param_value = if let Some(open_paren) = param.find('(') {
                    param[..open_paren].trim().to_string()
                } else {
                    param.clone()
                };
                
                // 为字符串参数添加引号
                let formatted_param = if param_value.parse::<i64>().is_ok() || param_value.parse::<f64>().is_ok() {
                    // 数字参数不加引号
                    param_value
                } else {
                    // 字符串参数加单引号
                    format!("'{}'", param_value)
                };
                
                result.replace_range(pos..pos+1, &formatted_param);
                param_index += 1;
            } else {
                break;
            }
        }
        
        // 确保SQL以分号结尾
        if !result.trim().ends_with(';') {
            result.push(';');
        }
        
        result
    }
    
    /// 清理插件系统
    pub async fn cleanup(&self) -> Result<(), PluginError> {
        self.registry.write().await.cleanup_all()?;
        Ok(())
    }
}

/// 插件统计信息
#[derive(Debug, Clone)]
pub struct PluginStats {
    pub total_plugins: usize,
    pub enabled_plugins: usize,
    pub plugin_types: HashMap<String, usize>,
}
