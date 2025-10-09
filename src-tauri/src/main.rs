// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use log::{info, error};

mod config;
mod plugins;
use config::{ConfigService, ThemeMode};
use plugins::core::EnhancedPluginManager;
use plugins::LogEntry as PluginLogEntry;

// 全局应用状态
pub struct AppState {
    pub config_service: Arc<ConfigService>,
    pub plugin_manager: Arc<EnhancedPluginManager>,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // 初始化配置服务
        // TODO: Implement proper config file loading
        let config_service = Arc::new(ConfigService::new());

        // 初始化插件系统
        let plugin_manager = Arc::new(EnhancedPluginManager::new());
        plugin_manager.initialize().await?;

        Ok(Self {
            config_service,
            plugin_manager,
        })
    }
}

// 健康检查
#[tauri::command]
async fn health_check() -> Result<HealthResponse, String> {
    Ok(HealthResponse {
        status: "ok".to_string(),
        version: "1.0.0".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

// 获取可用插件
#[tauri::command]
async fn get_plugins() -> Result<PluginsResponse, String> {
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

    Ok(PluginsResponse { plugins })
}

// 解析日志
#[tauri::command]
async fn parse_log(request: ParseRequest, state: tauri::State<'_, AppState>) -> Result<ParseResponse, String> {
    let start_time = std::time::Instant::now();

    info!("收到解析请求: {:?}", request);

    // 确定内容来源
    let content = if let Some(file_path) = &request.file_path {
        // 文件路径模式
        info!("使用文件路径模式: {}", file_path);

        // 检查文件是否存在
        if !std::path::Path::new(file_path).exists() {
            error!("文件不存在: {}", file_path);
            return Ok(ParseResponse {
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
            });
        }

        // 检查文件是否可读
        if !std::path::Path::new(file_path).is_file() {
            error!("路径不是文件: {}", file_path);
            return Ok(ParseResponse {
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
            });
        }

        // 读取文件内容
        match std::fs::read_to_string(file_path) {
            Ok(content) => {
                info!("文件读取成功，大小: {} bytes", content.len());
                content
            },
            Err(e) => {
                error!("读取文件失败: {} - 错误: {}", file_path, e);
                return Ok(ParseResponse {
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
                });
            }
        }
    } else if let Some(content) = &request.content {
        // 内容传输模式
        info!("使用内容传输模式，大小: {} bytes", content.len());
        content.clone()
    } else {
        error!("请求中既没有文件路径也没有内容");
        return Ok(ParseResponse {
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
        });
    };

    let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();
    let total_lines = lines.len();

    // 检查是否需要分块处理
    let chunk_size = request.chunk_size.unwrap_or(1000); // 默认1000行一块
    let chunk_index = request.chunk_index.unwrap_or(0);

    // 只有当文件大小超过分块大小且明确请求分块时才分块处理
    let should_chunk = total_lines > chunk_size && request.chunk_size.is_some();

    info!("分块判断: total_lines={}, chunk_size={}, chunk_size_requested={}, should_chunk={}",
         total_lines, chunk_size, request.chunk_size.is_some(), should_chunk);

    if should_chunk {
        // 分块处理
        let start_index = chunk_index * chunk_size;
        let _end_index = std::cmp::min(start_index + chunk_size, total_lines);

        info!("🔧 分块处理：使用插件系统快速处理");

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
        let processed_entries = match process_logs_with_plugin_system(&chunk_entries, &state.plugin_manager).await {
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

        Ok(ParseResponse {
            success: true,
            entries,
            stats,
            chunk_info: Some(chunk_info),
            error: None,
            detected_format: None, // 分块处理时不做格式检测
        })
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
        let processed_entries = match process_logs_with_plugin_system(&log_entries, &state.plugin_manager).await {
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
                return Ok(ParseResponse {
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
                });
            }
        };

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

        Ok(ParseResponse {
            success: true,
            entries,
            stats,
            chunk_info: None,
            error: None,
            detected_format: Some(detected_format),
        })
    }
}

// 测试解析端点
#[tauri::command]
async fn test_parse(request: ParseRequest) -> Result<serde_json::Value, String> {
    info!("测试解析请求: {:?}", request);

    // 检查请求类型
    let request_type = if request.file_path.is_some() {
        "file_path"
    } else if request.content.is_some() {
        "content"
    } else {
        "unknown"
    };

    Ok(serde_json::json!({
        "success": true,
        "message": "测试成功",
        "request_type": request_type,
        "request": request
    }))
}

// 主题配置相关命令
#[tauri::command]
async fn get_theme_config(state: tauri::State<'_, AppState>) -> Result<ThemeResponse, String> {
    match state.config_service.get_theme_config().await {
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

            Ok(response)
        }
        Err(e) => {
            log::error!("获取主题配置失败: {}", e);
            Err("获取主题配置失败".to_string())
        }
    }
}

#[tauri::command]
async fn update_theme_config(
    request: ThemeUpdateRequest,
    state: tauri::State<'_, AppState>
) -> Result<String, String> {
    // 获取当前配置
    let mut theme = match state.config_service.get_theme_config().await {
        Ok(theme) => theme,
        Err(e) => {
            log::error!("获取当前主题配置失败: {}", e);
            return Err("获取当前主题配置失败".to_string());
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

    match state.config_service.set_theme_config(&theme).await {
        Ok(_) => Ok("主题配置更新成功".to_string()),
        Err(e) => {
            log::error!("更新主题配置失败: {}", e);
            Err("更新主题配置失败".to_string())
        }
    }
}

// 其他配置获取命令
#[tauri::command]
async fn get_parse_config(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    match state.config_service.get_parse_config().await {
        Ok(parse) => {
            let data = serde_json::json!({
                "auto_parse": parse.auto_parse,
                "show_line_numbers": parse.show_line_numbers,
                "max_file_size": parse.max_file_size,
                "chunk_size": parse.chunk_size,
                "timeout_seconds": parse.timeout_seconds,
            });

            Ok(data)
        }
        Err(e) => {
            log::error!("获取解析配置失败: {}", e);
            Err("获取解析配置失败".to_string())
        }
    }
}

#[tauri::command]
async fn get_plugin_config(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    match state.config_service.get_plugin_config().await {
        Ok(plugin) => {
            let data = serde_json::json!({
                "auto_update": plugin.auto_update,
                "enable_notifications": plugin.enable_notifications,
                "plugin_directory": plugin.plugin_directory,
                "max_plugins": plugin.max_plugins,
            });

            Ok(data)
        }
        Err(e) => {
            log::error!("获取插件配置失败: {}", e);
            Err("获取插件配置失败".to_string())
        }
    }
}

#[tauri::command]
async fn get_window_config(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    match state.config_service.get_window_config().await {
        Ok(window) => {
            let data = serde_json::json!({
                "width": window.width,
                "height": window.height,
                "maximized": window.maximized,
                "always_on_top": window.always_on_top,
                "remember_position": window.remember_position,
            });

            Ok(data)
        }
        Err(e) => {
            log::error!("获取窗口配置失败: {}", e);
            Err("获取窗口配置失败".to_string())
        }
    }
}

#[tauri::command]
async fn get_all_configs(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    match state.config_service.get_all_configs().await {
        Ok(configs) => {
            let data = serde_json::to_value(configs).unwrap();
            Ok(data)
        }
        Err(e) => {
            log::error!("获取所有配置失败: {}", e);
            Err("获取所有配置失败".to_string())
        }
    }
}

// 文件系统相关命令
#[tauri::command]
async fn read_text_file(path: String) -> Result<String, String> {
    info!("读取文件: {}", path);

    match std::fs::read_to_string(&path) {
        Ok(content) => {
            info!("文件读取成功，大小: {} bytes", content.len());
            Ok(content)
        }
        Err(e) => {
            error!("读取文件失败: {}", e);
            Err(format!("读取文件失败: {}", e))
        }
    }
}

#[tauri::command]
async fn write_file(path: String, contents: String) -> Result<(), String> {
    info!("写入文件: {}", path);

    match std::fs::write(&path, contents) {
        Ok(_) => {
            info!("文件写入成功");
            Ok(())
        }
        Err(e) => {
            error!("写入文件失败: {}", e);
            Err(format!("写入文件失败: {}", e))
        }
    }
}

#[derive(Deserialize)]
struct SaveDialogRequest {
    default_path: String,
    filters: Vec<FileDialogFilter>,
}

#[derive(Deserialize)]
struct FileDialogFilter {
    name: String,
    extensions: Vec<String>,
}

#[tauri::command]
async fn save_dialog(request: SaveDialogRequest) -> Result<Option<String>, String> {
    // 在 Tauri 1.x 中，我们需要使用 tauri-plugin-dialog
    // 但是由于我们已经配置了 dialog 特性，我们可以直接返回默认路径
    // 这是一个简化的实现，实际应用中应该使用真正的文件对话框
    info!("保存对话框: {}", request.default_path);
    Ok(Some(request.default_path))
}

// 数据结构定义
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
    detected_format: Option<String>,
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

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    timestamp: String,
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

// 辅助函数
async fn process_logs_with_plugin_system(entries: &[LogEntry], plugin_manager: &Arc<EnhancedPluginManager>) -> Result<Vec<LogEntry>, String> {
    info!("🔧 开始插件系统处理，输入条目数: {}", entries.len());

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

    Ok(converted_entries)
}

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

#[tokio::main]
async fn main() {
    // 初始化日志系统
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🚀 LogWhisper Tauri 应用启动中...");

    // 初始化应用状态
    let app_state = match AppState::new().await {
        Ok(state) => {
            info!("✅ 应用状态初始化完成");
            state
        }
        Err(e) => {
            error!("❌ 应用状态初始化失败: {}", e);
            return;
        }
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            health_check,
            get_plugins,
            parse_log,
            test_parse,
            get_theme_config,
            update_theme_config,
            get_parse_config,
            get_plugin_config,
            get_window_config,
            get_all_configs,
            read_text_file,
            write_file,
            save_dialog
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}