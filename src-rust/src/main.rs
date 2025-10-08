use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use log::{info, error, warn};
use std::sync::Arc;
use std::env;
use std::fs;
use std::path::Path;

mod config;
mod plugins;
use config::{ConfigService, ThemeConfig, ThemeMode, ParseConfig, PluginConfig, WindowConfig, ConfigUpdateRequest};
use plugins::core::EnhancedPluginManager;
use plugins::LogEntry as PluginLogEntry;

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    port: u16,
    log_file: String,
    log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 3030,
            log_file: "logs/log-whisper.log".to_string(),
            log_level: "info".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParseRequest {
    #[serde(default)]
    file_path: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    plugin: Option<String>,
    #[serde(default)]
    chunk_size: Option<usize>,
    #[serde(default)]
    chunk_index: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParseResponse {
    success: bool,
    entries: Vec<LogEntry>,
    stats: ParseStats,
    chunk_info: Option<ChunkInfo>,
    error: Option<String>,
    detected_format: Option<String>, // 新增：检测到的日志格式
}

#[derive(Debug, Serialize, Deserialize)]
struct ChunkInfo {
    total_chunks: usize,
    current_chunk: usize,
    has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    line_number: usize,
    content: String,
    timestamp: Option<String>,
    level: Option<String>,
    formatted_content: Option<String>,
    metadata: std::collections::HashMap<String, String>,
    processed_by: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParseStats {
    total_lines: usize,
    success_lines: usize,
    error_lines: usize,
    parse_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Plugin {
    name: String,
    description: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginsResponse {
    plugins: Vec<Plugin>,
}

// 配置相关结构体
#[derive(Debug, Serialize, Deserialize)]
struct ConfigResponse {
    success: bool,
    message: String,
    data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThemeResponse {
    mode: String,
    primary_color: String,
    accent_color: String,
    font_size: u32,
    font_family: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThemeUpdateRequest {
    mode: String,
    primary_color: Option<String>,
    accent_color: Option<String>,
    font_size: Option<u32>,
    font_family: Option<String>,
}

// 健康检查
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "1.0.0".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

// 获取可用插件
async fn get_plugins() -> Json<PluginsResponse> {
    let plugins = vec![
        Plugin {
            name: "auto".to_string(),
            description: "自动检测".to_string(),
            version: "1.0.0".to_string(),
        },
        Plugin {
            name: "mybatis".to_string(),
            description: "MyBatis SQL 解析器".to_string(),
            version: "1.0.0".to_string(),
        },
        Plugin {
            name: "docker_json".to_string(),
            description: "Docker JSON 日志".to_string(),
            version: "1.0.0".to_string(),
        },
        Plugin {
            name: "raw".to_string(),
            description: "原始文本".to_string(),
            version: "1.0.0".to_string(),
        },
    ];
    
    Json(PluginsResponse { plugins })
}

// 解析日志
async fn parse_log(Json(request): Json<ParseRequest>) -> Result<Json<ParseResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    info!("收到解析请求: {:?}", request);
    
    // 确定内容来源
    let content = if let Some(file_path) = &request.file_path {
        // 文件路径模式
        info!("使用文件路径模式: {}", file_path);
        
        // 检查文件是否存在
        if !std::path::Path::new(file_path).exists() {
            error!("文件不存在: {}", file_path);
            return Ok(Json(ParseResponse {
                success: false,
                entries: vec![],
                stats: ParseStats {
                    total_lines: 0,
                    success_lines: 0,
                    error_lines: 0,
                    parse_time_ms: 0,
                },
                chunk_info: None,
                error: Some(format!("文件不存在: {}", file_path)),
                detected_format: None,
            }));
        }
        
        // 检查文件是否可读
        if !std::path::Path::new(file_path).is_file() {
            error!("路径不是文件: {}", file_path);
            return Ok(Json(ParseResponse {
                success: false,
                entries: vec![],
                stats: ParseStats {
                    total_lines: 0,
                    success_lines: 0,
                    error_lines: 0,
                    parse_time_ms: 0,
                },
                chunk_info: None,
                error: Some(format!("路径不是文件: {}", file_path)),
                detected_format: None,
            }));
        }
        
        // 读取文件内容
        match std::fs::read_to_string(file_path) {
            Ok(content) => {
                info!("文件读取成功，大小: {} bytes", content.len());
                content
            },
            Err(e) => {
                error!("读取文件失败: {} - 错误: {}", file_path, e);
                return Ok(Json(ParseResponse {
                    success: false,
                    entries: vec![],
                    stats: ParseStats {
                        total_lines: 0,
                        success_lines: 0,
                        error_lines: 0,
                        parse_time_ms: 0,
                    },
                    chunk_info: None,
                    error: Some(format!("读取文件失败: {} - 错误: {}", file_path, e)),
                    detected_format: None,
                }));
            }
        }
    } else if let Some(content) = &request.content {
        // 内容传输模式
        info!("使用内容传输模式，大小: {} bytes", content.len());
        content.clone()
    } else {
        error!("请求中既没有文件路径也没有内容");
        return Ok(Json(ParseResponse {
            success: false,
            entries: vec![],
            stats: ParseStats {
                total_lines: 0,
                success_lines: 0,
                error_lines: 0,
                parse_time_ms: 0,
            },
            chunk_info: None,
            error: Some("请求中既没有文件路径也没有内容".to_string()),
            detected_format: None,
        }));
    };
    
    let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();
    let total_lines = lines.len();
    
    // 添加详细的行调试信息
    info!("过滤后有效行数: {}", total_lines);
    for (i, line) in lines.iter().take(5).enumerate() {
        info!("行 {}: 长度={}, 内容={}", i + 1, line.len(), line.chars().take(200).collect::<String>());
    }
    
    // 检查是否需要分块处理
    let chunk_size = request.chunk_size.unwrap_or(1000); // 默认1000行一块
    let chunk_index = request.chunk_index.unwrap_or(0);
    
    // 只有当文件大小超过分块大小且明确请求分块时才分块处理
    let should_chunk = total_lines > chunk_size && request.chunk_size.is_some();
    
    info!("分块判断: total_lines={}, chunk_size={}, chunk_size_requested={}, should_chunk={}", 
         total_lines, chunk_size, request.chunk_size.is_some(), should_chunk);
    
    if should_chunk {
        // 分块处理 - 使用插件系统进行快速处理
        let start_index = chunk_index * chunk_size;
        let _end_index = std::cmp::min(start_index + chunk_size, total_lines);
        
        info!("🔧 分块处理：使用插件系统快速处理容器JSON格式");
        
        // 将分块行转换为LogEntry
        let chunk_entries: Vec<LogEntry> = lines.iter().enumerate().skip(start_index).take(chunk_size)
            .map(|(global_index, line)| LogEntry {
                line_number: global_index + 1,
                content: line.to_string(),
                timestamp: extract_timestamp(line),
                level: extract_log_level(line),
                formatted_content: Some(line.trim().to_string()),
                metadata: std::collections::HashMap::new(),
                processed_by: vec!["generic_parser".to_string()],
            })
            .collect();
        
        // 使用插件系统处理分块
        let processed_entries = match process_logs_with_plugin_system(&chunk_entries).await {
            Ok(entries) => entries,
            Err(e) => {
                error!("分块插件系统处理失败: {}", e);
                // 回退到传统处理
                chunk_entries
            }
        };
        
        let entries = processed_entries;
        
        let total_chunks = (total_lines + chunk_size - 1) / chunk_size; // 向上取整
        let has_more = chunk_index + 1 < total_chunks;
        
        let parse_time = start_time.elapsed().as_millis() as u64;
        
        let stats = ParseStats {
            total_lines,
            success_lines: entries.len(),
            error_lines: 0,
            parse_time_ms: parse_time,
        };
        
        let chunk_info = ChunkInfo {
            total_chunks,
            current_chunk: chunk_index,
            has_more,
        };
        
        info!("分块解析完成: 第{}/{}块，{}行，耗时: {}ms", 
              chunk_index + 1, total_chunks, entries.len(), parse_time);
        
        Ok(Json(ParseResponse {
            success: true,
            entries,
            stats,
            chunk_info: Some(chunk_info),
            error: None,
            detected_format: None, // 分块处理时不做格式检测
        }))
    } else {
        // 使用增强插件系统处理（小文件）
        info!("使用增强插件系统处理日志");
        
        // 将字符串行转换为LogEntry
        let log_entries: Vec<LogEntry> = lines.iter().enumerate()
            .map(|(index, line)| LogEntry {
                line_number: index + 1,
                content: line.to_string(),
                timestamp: extract_timestamp(line),
                level: extract_log_level(line),
                formatted_content: Some(line.trim().to_string()),
                metadata: std::collections::HashMap::new(),
                processed_by: vec!["generic_parser".to_string()],
            })
            .collect();
        
        // 使用插件系统处理
        let processed_entries = match process_logs_with_plugin_system(&log_entries).await {
            Ok(entries) => entries,
            Err(e) => {
                error!("插件系统处理失败: {}", e);
                // 回退到传统处理
                let mut fallback_entries = Vec::new();
                for (index, line) in lines.iter().enumerate() {
                    fallback_entries.push(LogEntry {
                        line_number: index + 1,
                        content: line.to_string(),
                        timestamp: extract_timestamp(line),
                        level: extract_log_level(line),
                        formatted_content: Some(line.trim().to_string()),
                        metadata: std::collections::HashMap::new(),
                        processed_by: vec!["generic_parser".to_string()],
                    });
                }
                return Ok(Json(ParseResponse {
                    success: true,
                    entries: fallback_entries,
                    stats: ParseStats {
                        total_lines: lines.len(),
                        success_lines: lines.len(),
                        error_lines: 0,
                        parse_time_ms: start_time.elapsed().as_millis() as u64,
                    },
                    chunk_info: None,
                    error: Some(format!("插件系统处理失败: {}", e)),
                    detected_format: None,
                }));
            }
        };
        
        // 插件系统已经返回LogEntry格式，直接使用
        let entries = processed_entries;
        
        let parse_time = start_time.elapsed().as_millis() as u64;
        
        let stats = ParseStats {
            total_lines: lines.len(),
            success_lines: entries.len(),
            error_lines: 0,
            parse_time_ms: parse_time,
        };
        
        // 检测日志格式
        let detected_format = detect_log_format(&lines);
        
        info!("全量解析完成: {} 行，处理为 {} 条目，耗时: {}ms，检测格式: {:?}", 
              lines.len(), entries.len(), parse_time, detected_format);
        
        Ok(Json(ParseResponse {
            success: true,
            entries,
            stats,
            chunk_info: None,
            error: None,
            detected_format: Some(detected_format),
        }))
    }
}

// 全局插件管理器缓存
static mut PLUGIN_MANAGER: Option<Arc<EnhancedPluginManager>> = None;
static INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

// 使用插件系统处理日志
async fn process_logs_with_plugin_system(entries: &[LogEntry]) -> Result<Vec<LogEntry>, String> {
    info!("🔧 开始插件系统处理，输入条目数: {}", entries.len());
    
    // 输出前几个条目的内容用于调试
    for (i, entry) in entries.iter().take(3).enumerate() {
        info!("🔧 输入条目 {}: 行号={}, 内容前100字符={}", 
              i + 1, entry.line_number, entry.content.chars().take(100).collect::<String>());
    }
    
    // 使用全局缓存的插件管理器
    let plugin_manager = get_or_init_plugin_manager().await
        .map_err(|e| format!("插件系统初始化失败: {}", e))?;
    
    info!("🔧 使用缓存的插件系统");
    
    // 转换LogEntry到PluginLogEntry
    let plugin_entries: Vec<PluginLogEntry> = entries.iter().map(|entry| {
        PluginLogEntry {
            line_number: entry.line_number,
            content: entry.content.clone(),
            timestamp: entry.timestamp.clone(),
            level: entry.level.clone(),
            formatted_content: entry.formatted_content.clone(),
            metadata: std::collections::HashMap::new(),
            processed_by: Vec::new(),
        }
    }).collect();
    
    // 处理日志条目
    let result = plugin_manager.process_log_entries(plugin_entries).await
        .map_err(|e| format!("插件处理失败: {}", e))?;
    
    info!("🔧 插件系统处理完成，输出条目数: {}", result.len());
    
    // 转换回LogEntry
    let converted_entries: Vec<LogEntry> = result.into_iter().map(|entry| {
        LogEntry {
            line_number: entry.line_number,
            content: entry.content,
            timestamp: entry.timestamp,
            level: entry.level,
            formatted_content: entry.formatted_content,
            metadata: entry.metadata,
            processed_by: entry.processed_by,
        }
    }).collect();
    
    // 输出前几个条目的调试信息
    for (i, entry) in converted_entries.iter().take(3).enumerate() {
        info!("🔧 处理后的条目 {}: 行号={}, 内容长度={}, formatted_content: {:?}", 
              i + 1, entry.line_number, entry.content.len(),
              entry.formatted_content.as_ref().map(|s| s.chars().take(50).collect::<String>()));
    }
    
    Ok(converted_entries)
}

// 获取或初始化全局插件管理器
async fn get_or_init_plugin_manager() -> Result<Arc<EnhancedPluginManager>, String> {
    if INITIALIZED.load(std::sync::atomic::Ordering::Relaxed) {
        unsafe {
            if let Some(ref manager) = PLUGIN_MANAGER {
                return Ok(manager.clone());
            }
        }
    }
    
    // 初始化插件管理器
    let plugin_manager = Arc::new(EnhancedPluginManager::new());
    plugin_manager.initialize().await
        .map_err(|e| format!("插件系统初始化失败: {}", e))?;
    
    unsafe {
        PLUGIN_MANAGER = Some(plugin_manager.clone());
    }
    INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);
    
    Ok(plugin_manager)
}

// 检测日志格式
fn detect_log_format(lines: &[&str]) -> String {
    if lines.is_empty() {
        return "Unknown".to_string();
    }
    
    // 检查SpringBoot格式
    let springboot_count = lines.iter()
        .filter(|line| {
            line.contains("INFO") || line.contains("ERROR") || line.contains("WARN") || line.contains("DEBUG")
        })
        .count();
    
    if springboot_count > lines.len() / 2 {
        return "SpringBoot".to_string();
    }
    
    // 检查Docker JSON格式
    let docker_json_count = lines.iter()
        .filter(|line| line.trim().starts_with('{') && line.contains("\"log\":") && line.contains("\"stream\":"))
        .count();
    
    if docker_json_count > lines.len() / 2 {
        return "DockerJson".to_string();
    }
    
    // 检查MyBatis格式
    let mybatis_count = lines.iter()
        .filter(|line| line.contains("Preparing:") || line.contains("Parameters:") || line.contains("==>"))
        .count();
    
    if mybatis_count > 0 {
        return "MyBatis".to_string();
    }
    
    "Unknown".to_string()
}

// 测试解析端点
async fn test_parse(Json(request): Json<ParseRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("测试解析请求: {:?}", request);
    
    // 检查请求类型
    let request_type = if request.file_path.is_some() {
        "file_path"
    } else if request.content.is_some() {
        "content"
    } else {
        "unknown"
    };
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "测试成功",
        "request_type": request_type,
        "request": request
    })))
}

// 使用增强插件系统处理日志

// 日志格式枚举
#[derive(Debug, PartialEq)]
enum LogFormat {
    DockerJson,
    SpringBoot,
    Generic,
}


// 检测是否为 Docker JSON 格式行
fn is_docker_json_line(line: &str) -> bool {
    let trimmed = line.trim();
    
    // 详细检测日志
    let starts_with_brace = trimmed.starts_with('{');
    let ends_with_brace = trimmed.ends_with('}');
    let has_log = trimmed.contains("\"log\":");
    let has_stream = trimmed.contains("\"stream\":");
    let has_time = trimmed.contains("\"time\":");
    
    info!("检测 Docker JSON 行: 开始{{={}, 结束}}={}, log={}, stream={}, time={} | 内容: {:?}", 
         starts_with_brace, ends_with_brace, has_log, has_stream, has_time, trimmed.chars().take(200).collect::<String>());
    
    // 必须以 { 开始和 } 结尾
    if !starts_with_brace || !ends_with_brace {
        return false;
    }
    
    // 检查是否包含 Docker JSON 的关键字段
    has_log && has_stream && has_time
}

// 检测是否为 Spring Boot 格式行
fn is_spring_boot_line(line: &str) -> bool {
    // Spring Boot 日志通常包含时间戳和日志级别
    let has_timestamp = extract_timestamp(line).is_some();
    let has_level = extract_log_level(line).is_some();
    let has_thread = line.contains("---") && line.contains("[");
    
    // 标准Spring Boot格式
    if has_timestamp && has_level && has_thread {
        return true;
    }
    
    // 检查是否是异常堆栈相关行（这些也应该用Spring Boot解析器处理）
    let trimmed = line.trim();
    
    // 异常类名行
    if trimmed.contains("Exception:") || trimmed.contains("Error:") || 
       trimmed.ends_with("Exception") || trimmed.ends_with("Error") ||
       trimmed.starts_with("Caused by:") {
        return true;
    }
    
    // 堆栈跟踪行
    if trimmed.starts_with("at ") && trimmed.contains("(") && trimmed.contains(")") {
        return true;
    }
    
    false
}

// 处理 Docker JSON 格式日志
fn process_docker_json_lines(lines: &[&str]) -> Vec<LogEntry> {
    let mut entries = Vec::new();
    info!("开始处理 Docker JSON 格式，总行数: {}", lines.len());
    
    for (index, line) in lines.iter().enumerate() {
        if let Some(parsed) = parse_docker_json_line(line) {
            info!("成功解析第 {} 行 Docker JSON: level={}, stream={}, content={}", 
                 index + 1, parsed.level, parsed.stream, parsed.log_content.chars().take(50).collect::<String>());
            
            let entry = LogEntry {
                line_number: index + 1,
                content: parsed.log_content.trim().to_string(), // 只返回日志内容，去掉换行符
                timestamp: Some(parsed.timestamp.clone()),
                level: Some(parsed.level.clone()),
                formatted_content: Some(parsed.log_content.trim().to_string()), // 简化格式化内容
                metadata: std::collections::HashMap::new(),
                processed_by: vec!["docker_json_parser".to_string()],
            };
            entries.push(entry);
        } else {
            info!("第 {} 行解析失败，作为普通文本处理: {}", 
                 index + 1, line.chars().take(100).collect::<String>());
            
            // 如果解析失败，作为普通文本处理
            let entry = LogEntry {
                line_number: index + 1,
                content: line.to_string(),
                timestamp: extract_timestamp(line),
                level: extract_log_level(line),
                formatted_content: Some(line.trim().to_string()),
                metadata: std::collections::HashMap::new(),
                processed_by: vec!["generic_parser".to_string()],
            };
            entries.push(entry);
        }
    }
    
    info!("Docker JSON 处理完成，生成 {} 个条目", entries.len());
    entries
}

// Docker JSON 解析结果结构
struct DockerJsonLog {
    log_content: String,
    stream: String,
    timestamp: String,
    level: String,
}

// 解析单行 Docker JSON
fn parse_docker_json_line(line: &str) -> Option<DockerJsonLog> {
    use serde_json::Value;
    
    if let Ok(json) = serde_json::from_str::<Value>(line) {
        let log_content = json.get("log")?.as_str()?.to_string();
        let stream = json.get("stream")?.as_str()?.to_string();
        let timestamp = json.get("time")?.as_str()?.to_string();
        
        // 根据流类型和内容确定日志级别
        let level = if stream == "stderr" {
            "ERROR".to_string()
        } else {
            // 检查日志内容中的级别关键词
            let content_lower = log_content.to_lowercase();
            if content_lower.contains("error") || content_lower.contains("exception") {
                "ERROR".to_string()
            } else if content_lower.contains("warn") {
                "WARN".to_string()
            } else if content_lower.contains("debug") {
                "DEBUG".to_string()
            } else {
                "INFO".to_string()
            }
        };
        
        return Some(DockerJsonLog {
            log_content,
            stream,
            timestamp,
            level,
        });
    }
    
    None
}

// 处理通用格式日志
fn process_generic_lines(lines: &[&str]) -> Vec<LogEntry> {
    let mut entries = Vec::new();
    
    for (index, line) in lines.iter().enumerate() {
        let entry = LogEntry {
            line_number: index + 1,
            content: line.to_string(),
            timestamp: extract_timestamp(line),
            level: extract_log_level(line),
            formatted_content: Some(line.trim().to_string()),
            metadata: std::collections::HashMap::new(),
            processed_by: vec!["generic_parser".to_string()],
        };
        entries.push(entry);
    }
    
    entries
}

// 聚合异常堆栈跟踪行
fn aggregate_exception_lines(lines: &[&str]) -> Vec<LogEntry> {
    let mut entries = Vec::new();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        
        // 检查是否是异常开始行（包含时间戳和异常信息）
        if is_exception_start_line(line) {
            let mut exception_content = vec![line.to_string()];
            let mut j = i + 1;
            
            // 收集所有后续的堆栈跟踪行
            while j < lines.len() && (is_stack_trace_line(lines[j]) || is_exception_continuation_line(lines[j])) {
                exception_content.push(lines[j].to_string());
                j += 1;
            }
            
            // 创建聚合的异常条目
            let full_content = exception_content.join("\n");
            let entry = LogEntry {
                line_number: i + 1,
                content: full_content,
                timestamp: extract_timestamp(line),
                level: extract_log_level(line),
                formatted_content: Some(exception_content.join("\n")),
                metadata: std::collections::HashMap::new(),
                processed_by: vec!["exception_aggregator".to_string()],
            };
            entries.push(entry);
            
            // 跳过已处理的行
            i = j;
        } else {
            // 普通日志行
            let entry = LogEntry {
                line_number: i + 1,
                content: line.to_string(),
                timestamp: extract_timestamp(line),
                level: extract_log_level(line),
                formatted_content: Some(line.trim().to_string()),
                metadata: std::collections::HashMap::new(),
                processed_by: vec!["generic_parser".to_string()],
            };
            entries.push(entry);
            i += 1;
        }
    }
    
    entries
}

// 检查是否是异常开始行
fn is_exception_start_line(line: &str) -> bool {
    let has_timestamp = extract_timestamp(line).is_some();
    let has_error_level = extract_log_level(line) == Some("ERROR".to_string());
    
    // 情况1：标准日志行 + ERROR级别（异常的正式开始）
    if has_timestamp && has_error_level {
        return true;
    }
    
    // 情况2：直接的异常类名行（紧接在ERROR日志后）
    let trimmed = line.trim();
    if !has_timestamp && (
        trimmed.contains("Exception:") || 
        trimmed.contains("Error:") ||
        trimmed.ends_with("Exception") ||
        trimmed.ends_with("Error") ||
        trimmed.starts_with("Caused by:")
    ) {
        return true;
    }
    
    false
}

// 检查是否是堆栈跟踪行
fn is_stack_trace_line(line: &str) -> bool {
    let trimmed = line.trim();
    
    // 标准Java堆栈跟踪格式
    if trimmed.starts_with("at ") {
        return true;
    }
    
    // Caused by 行
    if trimmed.starts_with("Caused by:") {
        return true;
    }
    
    // Suppressed 行
    if trimmed.starts_with("Suppressed:") {
        return true;
    }
    
    // common frames omitted
    if trimmed.contains("common frames omitted") || trimmed.contains("more") {
        return true;
    }
    
    false
}

// 检查是否是异常继续行
fn is_exception_continuation_line(line: &str) -> bool {
    let trimmed = line.trim();
    
    if trimmed.is_empty() {
        return false;
    }
    
    // 没有时间戳的非堆栈跟踪行，可能是异常消息
    if !extract_timestamp(line).is_some() && !trimmed.starts_with("at ") {
        // 检查是否是下一个正常日志的开始
        // 如果包含日志级别关键词，则不是异常继续行
        let level_keywords = ["INFO", "DEBUG", "WARN", "ERROR", "TRACE"];
        let has_level_keyword = level_keywords.iter().any(|keyword| line.contains(keyword));
        
        if !has_level_keyword {
            return true;
        }
    }
    
    false
}

// 提取时间戳
fn extract_timestamp(line: &str) -> Option<String> {
    use regex::Regex;
    
    // 常见的时间戳格式
    let patterns = vec![
        r"\d{4}-\d{2}-\d{2}[\s\T]\d{2}:\d{2}:\d{2}",
        r"\d{2}/\d{2}/\d{4}\s+\d{2}:\d{2}:\d{2}",
        r"\d{2}-\d{2}-\d{4}\s+\d{2}:\d{2}:\d{2}",
    ];
    
    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(captures) = re.find(line) {
                return Some(captures.as_str().to_string());
            }
        }
    }
    
    None
}

// 提取日志级别
fn extract_log_level(line: &str) -> Option<String> {
    let line_lower = line.to_lowercase();
    
    if line_lower.contains("error") || line_lower.contains("err") {
        Some("ERROR".to_string())
    } else if line_lower.contains("warn") || line_lower.contains("warning") {
        Some("WARN".to_string())
    } else if line_lower.contains("info") {
        Some("INFO".to_string())
    } else if line_lower.contains("debug") {
        Some("DEBUG".to_string())
    } else if line_lower.contains("trace") {
        Some("TRACE".to_string())
    } else {
        Some("INFO".to_string()) // 默认级别
    }
}

// 配置相关API处理函数
async fn get_theme_config(
    State(config_service): State<Arc<ConfigService>>
) -> Result<Json<ConfigResponse>, StatusCode> {
    match config_service.get_theme_config().await {
        Ok(theme) => {
            let response = ThemeResponse {
                mode: match theme.mode {
                    ThemeMode::Light => "light".to_string(),
                    ThemeMode::Dark => "dark".to_string(),
                    ThemeMode::Auto => "auto".to_string(),
                },
                primary_color: theme.primary_color,
                accent_color: theme.accent_color,
                font_size: theme.font_size,
                font_family: theme.font_family,
            };
            
            Ok(Json(ConfigResponse {
                success: true,
                message: "主题配置获取成功".to_string(),
                data: Some(serde_json::to_value(response).unwrap()),
            }))
        }
        Err(e) => {
            log::error!("获取主题配置失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_theme_config(
    State(config_service): State<Arc<ConfigService>>,
    Json(request): Json<ThemeUpdateRequest>
) -> Result<Json<ConfigResponse>, StatusCode> {
    // 获取当前配置
    let mut theme = match config_service.get_theme_config().await {
        Ok(theme) => theme,
        Err(e) => {
            log::error!("获取当前主题配置失败: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // 更新配置
    theme.mode = match request.mode.as_str() {
        "light" => ThemeMode::Light,
        "dark" => ThemeMode::Dark,
        "auto" => ThemeMode::Auto,
        _ => ThemeMode::Auto,
    };

    if let Some(primary_color) = request.primary_color {
        theme.primary_color = primary_color;
    }
    if let Some(accent_color) = request.accent_color {
        theme.accent_color = accent_color;
    }
    if let Some(font_size) = request.font_size {
        theme.font_size = font_size;
    }
    if let Some(font_family) = request.font_family {
        theme.font_family = font_family;
    }

    match config_service.set_theme_config(&theme).await {
        Ok(_) => Ok(Json(ConfigResponse {
            success: true,
            message: "主题配置更新成功".to_string(),
            data: None,
        })),
        Err(e) => {
            log::error!("更新主题配置失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_parse_config(
    State(config_service): State<Arc<ConfigService>>
) -> Result<Json<ConfigResponse>, StatusCode> {
    match config_service.get_parse_config().await {
        Ok(parse) => {
            let data = serde_json::json!({
                "auto_parse": parse.auto_parse,
                "show_line_numbers": parse.show_line_numbers,
                "max_file_size": parse.max_file_size,
                "chunk_size": parse.chunk_size,
                "timeout_seconds": parse.timeout_seconds,
            });
            
            Ok(Json(ConfigResponse {
                success: true,
                message: "解析配置获取成功".to_string(),
                data: Some(data),
            }))
        }
        Err(e) => {
            log::error!("获取解析配置失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_plugin_config(
    State(config_service): State<Arc<ConfigService>>
) -> Result<Json<ConfigResponse>, StatusCode> {
    match config_service.get_plugin_config().await {
        Ok(plugin) => {
            let data = serde_json::json!({
                "auto_update": plugin.auto_update,
                "enable_notifications": plugin.enable_notifications,
                "plugin_directory": plugin.plugin_directory,
                "max_plugins": plugin.max_plugins,
            });
            
            Ok(Json(ConfigResponse {
                success: true,
                message: "插件配置获取成功".to_string(),
                data: Some(data),
            }))
        }
        Err(e) => {
            log::error!("获取插件配置失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_window_config(
    State(config_service): State<Arc<ConfigService>>
) -> Result<Json<ConfigResponse>, StatusCode> {
    match config_service.get_window_config().await {
        Ok(window) => {
            let data = serde_json::json!({
                "width": window.width,
                "height": window.height,
                "maximized": window.maximized,
                "always_on_top": window.always_on_top,
                "remember_position": window.remember_position,
            });
            
            Ok(Json(ConfigResponse {
                success: true,
                message: "窗口配置获取成功".to_string(),
                data: Some(data),
            }))
        }
        Err(e) => {
            log::error!("获取窗口配置失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_all_configs(
    State(config_service): State<Arc<ConfigService>>
) -> Result<Json<ConfigResponse>, StatusCode> {
    match config_service.get_all_configs().await {
        Ok(configs) => {
            let data = serde_json::to_value(configs).unwrap();
            
            Ok(Json(ConfigResponse {
                success: true,
                message: "所有配置获取成功".to_string(),
                data: Some(data),
            }))
        }
        Err(e) => {
            log::error!("获取所有配置失败: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// 加载配置
fn load_config() -> AppConfig {
    // 从环境变量或配置文件加载
    let port = env::var("LOGWHISPER_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3030);
    
    let log_file = env::var("LOGWHISPER_LOG_FILE")
        .unwrap_or_else(|_| "logs/log-whisper.log".to_string());
    
    let log_level = env::var("LOGWHISPER_LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string());
    
    AppConfig {
        port,
        log_file,
        log_level,
    }
}

// 初始化日志
fn init_logger(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 确保日志目录存在
    if let Some(log_dir) = Path::new(&config.log_file).parent() {
        fs::create_dir_all(log_dir)?;
    }
    
    // 设置日志级别
    let log_level = match config.log_level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };
    
    // 初始化日志系统
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .target(env_logger::Target::Stdout)
        .init();
    
    info!("日志系统初始化完成");
    info!("日志级别: {}", config.log_level);
    info!("日志文件: {}", config.log_file);
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载配置
    let config = load_config();
    
    // 初始化日志系统
    init_logger(&config)?;
    
    info!("🚀 LogWhisper API 服务器启动中...");
    info!("📋 配置信息:");
    info!("  - 端口: {}", config.port);
    info!("  - 日志文件: {}", config.log_file);
    info!("  - 日志级别: {}", config.log_level);
    
    // 初始化配置服务
    let config_service = Arc::new(ConfigService::new("./config.db").map_err(|e| format!("配置服务初始化失败: {}", e))?);
    info!("✅ 配置服务初始化完成");
    
    // 初始化插件系统
    let plugin_manager = Arc::new(EnhancedPluginManager::new());
    plugin_manager.initialize().await.map_err(|e| format!("插件系统初始化失败: {}", e))?;
    info!("✅ 插件系统初始化完成");
    
    // 创建 CORS 层
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);
    
    // 构建路由
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/plugins", get(get_plugins))
        .route("/api/parse", post(parse_log))
        .route("/api/test", post(test_parse))
        // 配置相关路由
        .route("/api/config/theme", get(get_theme_config))
        .route("/api/config/theme", post(update_theme_config))
        .route("/api/config/parse", get(get_parse_config))
        .route("/api/config/plugin", get(get_plugin_config))
        .route("/api/config/window", get(get_window_config))
        .route("/api/config/all", get(get_all_configs))
        .with_state(config_service)
        .layer(cors);
    
    // 启动服务器
    let addr = format!("127.0.0.1:{}", config.port);
    
    info!("🌐 API 服务器启动在: http://{}", addr);
    info!("📋 API 端点:");
    info!("  - GET  /health");
    info!("  - GET  /api/plugins");
    info!("  - POST /api/parse");
    info!("  - GET  /api/config/theme");
    info!("  - POST /api/config/theme");
    info!("  - GET  /api/config/parse");
    info!("  - GET  /api/config/plugin");
    info!("  - GET  /api/config/window");
    info!("  - GET  /api/config/all");
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}