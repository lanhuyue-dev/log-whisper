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
use config::{ConfigService, ThemeConfig, ThemeMode, ParseConfig, PluginConfig, WindowConfig, ConfigUpdateRequest};

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
        }));
    };
    
    let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();
    let total_lines = lines.len();
    
    // 检查是否需要分块处理
    let chunk_size = request.chunk_size.unwrap_or(1000); // 默认1000行一块
    let chunk_index = request.chunk_index.unwrap_or(0);
    
    let should_chunk = total_lines > chunk_size;
    
    if should_chunk {
        // 分块处理
        let start_index = chunk_index * chunk_size;
        let end_index = std::cmp::min(start_index + chunk_size, total_lines);
        
        let mut entries = Vec::new();
        for (global_index, line) in lines.iter().enumerate().skip(start_index).take(chunk_size) {
            let entry = LogEntry {
                line_number: global_index + 1,
                content: line.to_string(),
                timestamp: extract_timestamp(line),
                level: extract_log_level(line),
                formatted_content: Some(line.trim().to_string()),
            };
            entries.push(entry);
        }
        
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
        }))
    } else {
        // 传统全量处理（小文件）
        let mut entries = Vec::new();
        
        for (index, line) in lines.iter().enumerate() {
            let entry = LogEntry {
                line_number: index + 1,
                content: line.to_string(),
                timestamp: extract_timestamp(line),
                level: extract_log_level(line),
                formatted_content: Some(line.trim().to_string()),
            };
            entries.push(entry);
        }
        
        let parse_time = start_time.elapsed().as_millis() as u64;
        
        let stats = ParseStats {
            total_lines: lines.len(),
            success_lines: entries.len(),
            error_lines: 0,
            parse_time_ms: parse_time,
        };
        
        info!("全量解析完成: {} 行，耗时: {}ms", entries.len(), parse_time);
        
        Ok(Json(ParseResponse {
            success: true,
            entries,
            stats,
            chunk_info: None,
            error: None,
        }))
    }
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
        Some("Error".to_string())
    } else if line_lower.contains("warn") || line_lower.contains("warning") {
        Some("Warn".to_string())
    } else if line_lower.contains("info") {
        Some("Info".to_string())
    } else if line_lower.contains("debug") {
        Some("Debug".to_string())
    } else if line_lower.contains("trace") {
        Some("Trace".to_string())
    } else {
        Some("Info".to_string()) // 默认级别
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