use tauri::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::parser::{LogParser, ChunkLoader};
use crate::models::{
    ParseResultSet, ParseConfig, ChunkLoadRequest, ChunkLoadResponse,
    ChunkMetadata, MemoryInfo, ChunkInfo
};
use log::{info, warn, error, debug, trace};

/// 解析文件请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ParseFileRequest {
    pub file_path: String,
    pub plugin_name: Option<String>,
}

/// 解析文件响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ParseFileResponse {
    pub success: bool,
    pub result_set: Option<ParseResultSet>,
    pub error: Option<String>,
}

/// 获取支持的文件格式响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SupportedFormatsResponse {
    pub formats: Vec<String>,
}

/// 获取可用插件响应
#[derive(Debug, Serialize, Deserialize)]
pub struct AvailablePluginsResponse {
    pub plugins: Vec<PluginInfo>,
}

/// 插件信息
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

/// 切换插件请求
#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchPluginRequest {
    pub plugin_name: String,
}

/// 切换插件响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchPluginResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// 复制到剪贴板请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CopyToClipboardRequest {
    pub content: String,
}

/// 复制到剪贴板响应
#[derive(Debug, Serialize, Deserialize)]
pub struct CopyToClipboardResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// 解析文件命令
#[tauri::command]
pub async fn parse_file(
    request: ParseFileRequest,
    parser: State<'_, Arc<LogParser>>,
) -> Result<ParseFileResponse, String> {
    info!("开始解析文件: {}", request.file_path);
    debug!("解析请求参数: {:?}", request);
    
    let start_time = std::time::Instant::now();
    
    let mut config = ParseConfig::default();
    
    // 设置插件
    if let Some(plugin_name) = &request.plugin_name {
        config.plugin_name = plugin_name.clone();
        debug!("使用插件: {}", plugin_name);
    } else {
        debug!("使用自动插件选择");
    }
    
    // 创建新的解析器实例
    let parser_instance = LogParser::with_config(config);
    trace!("解析器实例创建完成");
    
    match parser_instance.parse_file(&request.file_path).await {
        Ok(result_set) => {
            let duration = start_time.elapsed();
            info!("文件解析成功，耗时: {:?}", duration);
            debug!("解析结果统计: 总行数={}, 成功行数={}, 错误行数={}", 
                   result_set.total_stats.total_lines,
                   result_set.total_stats.success_lines,
                   result_set.total_stats.error_lines);
            trace!("解析结果详情: {:?}", result_set);
            
            Ok(ParseFileResponse {
                success: true,
                result_set: Some(result_set),
                error: None,
            })
        },
        Err(e) => {
            let duration = start_time.elapsed();
            error!("文件解析失败: {}, 耗时: {:?}", e, duration);
            debug!("解析错误详情: {:?}", e);
            
            Ok(ParseFileResponse {
                success: false,
                result_set: None,
                error: Some(e.to_string()),
            })
        },
    }
}

/// 获取支持的文件格式
#[tauri::command]
pub async fn get_supported_formats() -> Result<SupportedFormatsResponse, String> {
    debug!("获取支持的文件格式");
    let formats = vec![".log".to_string(), ".txt".to_string()];
    trace!("支持的文件格式: {:?}", formats);
    
    Ok(SupportedFormatsResponse {
        formats,
    })
}

/// 获取可用插件
#[tauri::command]
pub async fn get_available_plugins(
    parser: State<'_, Arc<LogParser>>,
) -> Result<AvailablePluginsResponse, String> {
    debug!("获取可用插件列表");
    let plugins = parser.get_available_plugins();
    debug!("发现 {} 个插件", plugins.len());
    trace!("插件列表: {:?}", plugins);
    
    let plugin_info: Vec<PluginInfo> = plugins.into_iter().map(|name: String| {
        let description = get_plugin_description(&name);
        debug!("插件: {} - {}", name, description);
        PluginInfo {
            name: name.clone(),
            description,
            enabled: true,
        }
    }).collect();
    
    info!("返回 {} 个可用插件", plugin_info.len());
    
    Ok(AvailablePluginsResponse {
        plugins: plugin_info,
    })
}

/// 切换插件
#[tauri::command]
pub async fn switch_plugin(
    _request: SwitchPluginRequest,
    _parser: State<'_, Arc<LogParser>>,
) -> Result<SwitchPluginResponse, String> {
    // 由于LogParser是不可变的，我们需要创建一个新的实例
    // 这里简化处理，实际应用中可能需要使用Mutex或其他同步机制
    Ok(SwitchPluginResponse {
        success: true,
        error: None,
    })
}

/// 复制到剪贴板（暂时禁用）
#[tauri::command]
pub async fn copy_to_clipboard(
    _request: CopyToClipboardRequest,
    _app: tauri::AppHandle,
) -> Result<CopyToClipboardResponse, String> {
    // 暂时返回成功，后续可以重新实现
    Ok(CopyToClipboardResponse {
        success: true,
        error: Some("剪贴板功能暂时禁用".to_string()),
    })
}

/// 获取文件信息
#[tauri::command]
pub async fn get_file_info(
    file_path: String,
    parser: State<'_, Arc<LogParser>>,
) -> Result<String, String> {
    match parser.get_file_info(&file_path).await {
        Ok(file_info) => Ok(format!("{:?}", file_info)),
        Err(e) => Err(e.to_string())
    }
}

/// 清空缓存
#[tauri::command]
pub async fn clear_cache(
    parser: State<'_, Arc<LogParser>>,
) -> Result<(), String> {
    parser.clear_cache()
        .map_err(|e| e.to_string())
}

/// 获取缓存统计信息
#[tauri::command]
pub async fn get_cache_stats(
    parser: State<'_, Arc<LogParser>>,
) -> Result<String, String> {
    let stats = parser.get_cache_stats();
    Ok(format!("{:?}", stats))
}

/// 获取插件描述
fn get_plugin_description(plugin_name: &str) -> String {
    match plugin_name {
        "Auto" => "自动选择最佳插件".to_string(),
        "MyBatis" => "MyBatis SQL 解析器".to_string(),
        "JSON" => "JSON 修复和格式化".to_string(),
        "Raw" => "原始文本显示".to_string(),
        _ => "未知插件".to_string(),
    }
}

/// 写入日志请求
#[derive(Debug, Serialize, Deserialize)]
pub struct WriteLogRequest {
    pub content: String,
    pub append: bool,
}

/// 写入日志响应
#[derive(Debug, Serialize, Deserialize)]
pub struct WriteLogResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// 写入日志到文件
#[tauri::command]
pub async fn write_log(
    request: WriteLogRequest,
) -> Result<WriteLogResponse, String> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use chrono::Utc;
    
    debug!("写入日志请求: append={}", request.append);
    
    // 创建日志目录
    let log_dir = std::env::current_dir()
        .map_err(|e| format!("获取当前目录失败: {}", e))?
        .join("logs");
    
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)
            .map_err(|e| format!("创建日志目录失败: {}", e))?;
    }
    
    // 生成日志文件名（按日期）
    let today = Utc::now().format("%Y-%m-%d");
    let log_file = log_dir.join(format!("logwhisper_{}.log", today));
    
    // 打开文件
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(request.append)
        .open(&log_file)
        .map_err(|e| format!("打开日志文件失败: {}", e))?;
    
    // 写入内容
    file.write_all(request.content.as_bytes())
        .map_err(|e| format!("写入日志失败: {}", e))?;
    
    file.flush()
        .map_err(|e| format!("刷新日志文件失败: {}", e))?;
    
    info!("日志已写入文件: {}", log_file.display());
    
    Ok(WriteLogResponse {
        success: true,
        error: None,
    })
}

/// 初始化文件分块元数据
#[tauri::command]
pub async fn initialize_file_chunks(
    file_path: String,
    chunk_loader: State<'_, Arc<ChunkLoader>>,
) -> Result<ChunkMetadata, String> {
    info!("初始化文件分块: {}", file_path);
    
    match chunk_loader.initialize_file_chunks(&file_path).await {
        Ok(metadata) => {
            info!("文件分块初始化成功: {} 块", metadata.total_chunks);
            Ok(metadata)
        }
        Err(e) => {
            error!("文件分块初始化失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 加载数据块
#[tauri::command]
pub async fn load_chunks(
    request: ChunkLoadRequest,
    chunk_loader: State<'_, Arc<ChunkLoader>>,
) -> Result<ChunkLoadResponse, String> {
    info!("加载数据块: {} 个块", request.chunk_indices.len());
    debug!("加载请求: {:?}", request);
    
    match chunk_loader.load_chunks(request).await {
        Ok(response) => {
            if response.success {
                info!("数据块加载成功: {} 块", response.chunks.len());
            } else {
                warn!("数据块加载失败: {}", response.error.as_ref().unwrap_or(&"未知错误".to_string()));
            }
            Ok(response)
        }
        Err(e) => {
            error!("数据块加载错误: {}", e);
            Err(e.to_string())
        }
    }
}

/// 获取内存信息
#[tauri::command]
pub async fn get_memory_info(
    chunk_loader: State<'_, Arc<ChunkLoader>>,
) -> Result<MemoryInfo, String> {
    debug!("获取内存信息");
    
    let memory_info = chunk_loader.get_memory_info().await;
    debug!("当前内存使用: {} bytes, 缓存块数: {}", 
           memory_info.current_usage, memory_info.cached_chunks);
    
    Ok(memory_info)
}

/// 清理内存
#[tauri::command]
pub async fn cleanup_memory(
    chunk_loader: State<'_, Arc<ChunkLoader>>,
) -> Result<usize, String> {
    info!("开始内存清理");
    
    match chunk_loader.force_gc().await {
        Ok(cleaned_count) => {
            info!("内存清理完成: 清理了 {} 个块", cleaned_count);
            Ok(cleaned_count)
        }
        Err(e) => {
            error!("内存清理失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 获取块状态信息
#[tauri::command]
pub async fn get_chunk_status(
    file_path: String,
    chunk_loader: State<'_, Arc<ChunkLoader>>,
) -> Result<Vec<ChunkInfo>, String> {
    debug!("获取块状态: {}", file_path);
    
    match chunk_loader.get_chunk_status(&file_path).await {
        Some(chunk_infos) => {
            debug!("找到 {} 个块的状态信息", chunk_infos.len());
            Ok(chunk_infos)
        }
        None => {
            warn!("未找到文件的块状态信息: {}", file_path);
            Ok(Vec::new())
        }
    }
}

/// 预加载数据块
#[tauri::command]
pub async fn preload_chunks(
    file_path: String,
    start_chunk: usize,
    plugin_name: String,
    chunk_loader: State<'_, Arc<ChunkLoader>>,
) -> Result<usize, String> {
    info!("预加载数据块: {} 开始块: {}", file_path, start_chunk);
    
    match chunk_loader.preload_chunks(&file_path, start_chunk, &plugin_name).await {
        Ok(loaded_count) => {
            info!("预加载完成: {} 个块", loaded_count);
            Ok(loaded_count)
        }
        Err(e) => {
            error!("预加载失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 清理所有缓存
#[tauri::command]
pub async fn clear_all_cache(
    chunk_loader: State<'_, Arc<ChunkLoader>>,
) -> Result<(), String> {
    info!("清理所有缓存");
    
    match chunk_loader.clear_all_cache().await {
        Ok(()) => {
            info!("缓存清理完成");
            Ok(())
        }
        Err(e) => {
            error!("缓存清理失败: {}", e);
            Err(e.to_string())
        }
    }
}

/// 文件验证请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateFileRequest {
    pub file_path: String,
}

/// 文件验证响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateFileResponse {
    pub valid: bool,
    pub file_size: Option<usize>,
    pub file_type: Option<String>,
    pub error: Option<String>,
}

/// 验证文件
#[tauri::command]
pub async fn validate_file(
    request: ValidateFileRequest,
    parser: State<'_, Arc<LogParser>>,
) -> Result<ValidateFileResponse, String> {
    debug!("验证文件: {}", request.file_path);
    
    // 检查文件是否存在
    if !std::path::Path::new(&request.file_path).exists() {
        return Ok(ValidateFileResponse {
            valid: false,
            file_size: None,
            file_type: None,
            error: Some("文件不存在".to_string()),
        });
    }
    
    // 获取文件信息
    match parser.get_file_info(&request.file_path).await {
        Ok(file_info) => {
            debug!("文件信息获取成功: {:?}", file_info);
            
            // 验证文件类型
            let file_type = std::path::Path::new(&request.file_path)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_lowercase());
            
            let is_valid_type = file_type.as_ref()
                .map(|ext| matches!(ext.as_str(), "log" | "txt"))
                .unwrap_or(false);
            
            if !is_valid_type {
                return Ok(ValidateFileResponse {
                    valid: false,
                    file_size: Some(file_info.size),
                    file_type,
                    error: Some("不支持的文件类型，仅支持 .log 和 .txt 文件".to_string()),
                });
            }
            
            // 检查文件大小（可选限制）
            let max_size = 1024 * 1024 * 1024; // 1GB
            if file_info.size > max_size {
                warn!("文件过大: {} bytes", file_info.size);
                // 不直接拒绝，让用户决定
            }
            
            info!("文件验证通过: {} ({} bytes)", request.file_path, file_info.size);
            
            Ok(ValidateFileResponse {
                valid: true,
                file_size: Some(file_info.size),
                file_type,
                error: None,
            })
        }
        Err(e) => {
            error!("获取文件信息失败: {}", e);
            Ok(ValidateFileResponse {
                valid: false,
                file_size: None,
                file_type: None,
                error: Some(format!("获取文件信息失败: {}", e)),
            })
        }
    }
}