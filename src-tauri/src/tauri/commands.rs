use tauri::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::parser::LogParser;
use crate::models::{ParseResultSet, ParseConfig};
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
    
    let plugin_info: Vec<PluginInfo> = plugins.into_iter().map(|name| {
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
    request: CopyToClipboardRequest,
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
    use std::path::Path;
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
