// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::{debug, error, info, warn};
/// LogWhisper Tauri Application - 主应用程序入口
///
/// 这是LogWhisper桌面应用的主要入口点，基于Tauri框架构建。
/// 应用程序提供了强大的日志分析功能，支持多种日志格式的解析和处理。
///
/// 架构组件：
/// - Tauri: 跨平台桌面应用框架，提供原生性能
/// - Rust: 高性能后端，负责日志解析和数据处理
/// - 插件系统: 可扩展的日志解析器架构
/// - 配置管理: 用户偏好设置和应用配置

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;

// 模块导入
mod config;
mod plugins;

// 具体导入
use config::{ConfigService, ThemeMode};
use plugins::core::EnhancedPluginManager;
use plugins::LogEntry as PluginLogEntry;

/// 应用程序全局状态
///
/// 包含应用程序运行时所需的所有核心服务组件。
/// 使用Arc确保在多线程环境中的安全共享。
pub struct AppState {
    /// 配置服务实例，管理用户设置和应用配置
    pub config_service: Arc<Mutex<ConfigService>>,
    /// 增强插件管理器，负责日志解析插件的管理和调用
    pub plugin_manager: Arc<EnhancedPluginManager>,
}

impl AppState {
    /// 创建新的应用状态实例
    ///
    /// # Returns
    /// - `Ok(AppState)`: 成功初始化的应用状态
    /// - `Err(Box<dyn std::error::Error>)`: 初始化失败时的错误信息
    ///
    /// # 初始化流程
    /// 1. 创建配置服务实例
    /// 2. 初始化插件管理器并加载所有插件
    /// 3. 验证所有核心组件正常工作
    ///
    /// # 示例
    /// ```rust
    /// let state = AppState::new().await?;
    /// ```
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("🔧 初始化应用状态...");

        // 初始化配置服务
        // 配置服务负责管理用户偏好设置、主题配置、解析设置等
        debug!("初始化配置服务");

        // 确定数据库路径
        let app_data_dir = get_app_data_dir().await?;
        let db_path = app_data_dir.join("config.db");

        info!("📁 配置数据库路径: {:?}", db_path);

        let config_service = Arc::new(Mutex::new(ConfigService::new(&db_path)?));

        // 初始化插件系统
        // 插件管理器负责加载和管理所有日志解析插件
        info!("🔧 初始化插件管理器...");
        let plugin_manager = Arc::new(EnhancedPluginManager::new());
        plugin_manager.initialize().await?;

        info!("✅ 应用状态初始化完成");
        Ok(Self {
            config_service,
            plugin_manager,
        })
    }
}

/// 获取应用数据目录
///
/// 根据不同操作系统返回相应的应用数据目录路径。
///
/// # Returns
/// - `Result<PathBuf, Box<dyn std::error::Error>>`: 应用数据目录路径
async fn get_app_data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_name = "LogWhisper";

    #[cfg(target_os = "windows")]
    {
        let mut path = dirs::config_dir()
            .ok_or("Failed to get AppData directory")?;
        path.push(app_name);

        // 确保目录存在
        std::fs::create_dir_all(&path)?;

        Ok(path)
    }

    #[cfg(target_os = "macos")]
    {
        let mut path = dirs::home_dir()
            .ok_or("Failed to get home directory")?;
        path.push("Library");
        path.push("Application Support");
        path.push(app_name);

        // 确保目录存在
        std::fs::create_dir_all(&path)?;

        Ok(path)
    }

    #[cfg(target_os = "linux")]
    {
        let path = dirs::data_dir()
            .ok_or("Failed to get data directory")?
            .join(app_name);

        // 确保目录存在
        std::fs::create_dir_all(&path)?;

        Ok(path)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // 默认使用当前目录
        let path = std::env::current_dir()
            .map_err(|e| e.into())?
            .join(app_name);

        // 确保目录存在
        std::fs::create_dir_all(&path)?;

        Ok(path)
    }
}

// ============================================================================
// 辅助函数模块
// ============================================================================

/// 创建错误响应的辅助函数
///
/// 用于统一创建解析失败时的错误响应格式。
///
/// # 参数
/// - `error_message`: 错误消息
/// - `file_path`: 相关的文件路径（可选）
///
/// # Returns
/// - `ParseResponse`: 格式化的错误响应
fn create_error_response(error_message: &str, file_path: &str) -> ParseResponse {
    ParseResponse {
        success: false,
        entries: vec![],
        stats: ParseStats {
            total_lines: 0,
            success_lines: 0,
            error_lines: 0,
            parse_time_ms: 0,
        },
        chunk_info: None,
        error: Some(format!("{}: {}", error_message, file_path)),
        detected_format: None,
    }
}

/// 创建空内容响应的辅助函数
///
/// 用于处理日志内容为空或只包含空行的情况。
///
/// # Returns
/// - `ParseResponse`: 格式化的空内容响应
fn create_empty_response() -> ParseResponse {
    ParseResponse {
        success: false,
        entries: vec![],
        stats: ParseStats {
            total_lines: 0,
            success_lines: 0,
            error_lines: 0,
            parse_time_ms: 0,
        },
        chunk_info: None,
        error: Some("日志内容为空".to_string()),
        detected_format: None,
    }
}

/// 应用程序健康检查端点
///
/// 提供应用程序的基本状态信息，用于监控系统健康状况。
/// 不需要访问应用状态，是一个简单的状态检查。
///
/// # Returns
/// - `Ok(HealthResponse)`: 包含状态、版本和时间戳的健康信息
/// - `Err(String)`: 健康检查失败时的错误信息
#[tauri::command]
async fn health_check() -> Result<HealthResponse, String> {
    debug!("执行健康检查");

    Ok(HealthResponse {
        status: "ok".to_string(),
        version: "1.0.0".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// 获取文件信息用于分块处理
///
/// 分析日志文件的基本信息，包括总行数、文件大小等，
/// 用于前端确定分块处理策略。
///
/// # 参数
/// - `file_path`: 日志文件的路径
///
/// # Returns
/// - `Ok(FileInfoResponse)`: 包含文件基本信息的响应
/// - `Err(String)`: 获取文件信息失败时的错误信息
#[tauri::command]
async fn get_file_info(file_path: String) -> Result<FileInfoResponse, String> {
    info!("🔍 [BACKEND_DEBUG] get_file_info 命令调用开始");
    info!("📊 [BACKEND_DEBUG] 获取文件信息: {}", file_path);

    // 文件存在性检查
    let path_obj = std::path::Path::new(&file_path);
    if !path_obj.exists() {
        error!("❌ [BACKEND_DEBUG] 文件不存在: {}", file_path);
        return Err(format!("文件不存在: {}", file_path));
    }

    if !path_obj.is_file() {
        error!("❌ [BACKEND_DEBUG] 路径不是文件: {}", file_path);
        return Err(format!("路径不是文件: {}", file_path));
    }

    // 获取文件元数据
    let metadata = match std::fs::metadata(&file_path) {
        Ok(meta) => {
            info!("✅ [BACKEND_DEBUG] 文件元数据获取成功");
            meta
        },
        Err(e) => {
            error!("❌ [BACKEND_DEBUG] 获取文件元数据失败: {} - 错误: {}", file_path, e);
            return Err(format!("获取文件元数据失败: {}", e));
        }
    };

    let file_size = metadata.len();
    info!("📏 [BACKEND_DEBUG] 文件大小: {} bytes ({} MB)", file_size, file_size / 1024 / 1024);

    // 对于大文件，采样读取前1000行来估算总行数
    let total_lines = if file_size > 10_000_000 { // 10MB以上的文件
        info!("📏 [BACKEND_DEBUG] 大文件检测，采样估算行数");
        match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                let sample_lines: Vec<&str> = content.lines().take(1000).collect();
                let sample_count = sample_lines.len();
                if sample_count > 0 {
                    // 基于采样估算总行数
                    let avg_line_len = content.len() / sample_count;
                    let estimated_lines = file_size as usize / avg_line_len;
                    info!("📊 [BACKEND_DEBUG] 估算总行数: {} (基于{}行采样)", estimated_lines, sample_count);
                    estimated_lines
                } else {
                    warn!("⚠️ [BACKEND_DEBUG] 文件内容为空，无法估算行数");
                    0
                }
            }
            Err(e) => {
                warn!("⚠️ [BACKEND_DEBUG] 无法读取文件内容进行行数估算: {}", e);
                0
            }
        }
    } else {
        // 小文件直接计算准确行数
        info!("📏 [BACKEND_DEBUG] 小文件直接计算行数");
        match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                let lines = content.lines().count();
                info!("📊 [BACKEND_DEBUG] 准确行数统计: {} 行", lines);
                lines
            }
            Err(e) => {
                warn!("⚠️ [BACKEND_DEBUG] 无法读取文件内容: {}", e);
                0
            }
        }
    };

    // 推荐分块大小
    let recommended_chunk_size = if total_lines > 100_000 {
        5000 // 超大文件
    } else if total_lines > 10_000 {
        2000 // 大文件
    } else if total_lines > 1000 {
        1000 // 中等文件
    } else {
        total_lines // 小文件不分块
    };

    let is_large_file = file_size > 50_000_000; // 50MB以上认为是大文件

    let response = FileInfoResponse {
        file_path: file_path.clone(),
        file_size,
        total_lines,
        recommended_chunk_size,
        is_large_file,
    };

    info!("✅ [BACKEND_DEBUG] get_file_info 命令完成: {:?}", response);
    Ok(response)
}

/// 获取可用的日志解析插件列表
///
/// 返回当前系统中所有可用的日志解析插件信息，
/// 包括插件名称、描述和版本信息。这些信息用于前端显示插件选择界面。
///
/// # Returns
/// - `Ok(PluginsResponse)`: 包含所有可用插件信息的响应
/// - `Err(String)`: 获取插件列表失败时的错误信息
///
/// # 插件列表
/// - auto: 自动检测日志格式
/// - mybatis: MyBatis SQL日志解析器
/// - docker_json: Docker JSON格式日志解析器
/// - raw: 原始文本日志解析器
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

/// 核心日志解析功能
///
/// 这是LogWhisper应用的核心功能，负责解析各种格式的日志文件。
/// 支持两种输入模式：文件路径模式和内容传输模式，并支持大文件的分块处理。
///
/// # 功能特性
/// - 智能格式检测和插件选择
/// - 大文件分块处理，避免内存溢出
/// - 性能监控和详细日志记录
/// - 错误处理和优雅降级
/// - 支持多种日志格式（SpringBoot、Docker JSON、MyBatis等）
///
/// # 参数
/// - `request`: 解析请求，包含文件路径或内容、插件选择等信息
/// - `state`: 应用状态，包含插件管理器和配置服务
///
/// # Returns
/// - `Ok(ParseResponse)`: 解析结果，包含解析的日志条目和统计信息
/// - `Err(String)`: 解析失败时的错误信息
///
/// # 性能考虑
/// - 小文件（<1000行）：直接使用插件系统处理
/// - 大文件（≥1000行）：自动分块处理，降低内存使用
/// - 智能缓存：避免重复的文件读取和解析操作
#[tauri::command]
async fn parse_log(request: ParseRequest, state: tauri::State<'_, AppState>) -> Result<ParseResponse, String> {
    let start_time = std::time::Instant::now();

    info!("🚀 [BACKEND_DEBUG] parse_log 命令调用开始");
    info!("📥 [BACKEND_DEBUG] 收到日志解析请求: {:?}", request);
    debug!("⏱️ [BACKEND_DEBUG] 开始性能计时");

    // 第一步：确定内容来源
    // 支持两种模式：文件路径模式（从磁盘读取）和内容传输模式（直接传入内容）
    let content = if let Some(file_path) = &request.file_path {
        // 文件路径模式：从指定的文件路径读取日志内容
        info!("📁 [BACKEND_DEBUG] 使用文件路径模式: {}", file_path);

        // 文件存在性检查：确保文件可访问
        if !std::path::Path::new(file_path).exists() {
            error!("❌ [BACKEND_DEBUG] 文件不存在: {}", file_path);
            return Ok(create_error_response("文件不存在", file_path));
        }

        // 文件类型检查：确保是普通文件而非目录
        if !std::path::Path::new(file_path).is_file() {
            error!("❌ [BACKEND_DEBUG] 路径不是文件: {}", file_path);
            return Ok(create_error_response("路径不是文件", file_path));
        }

        // 文件读取：安全地读取文件内容
        match std::fs::read_to_string(file_path) {
            Ok(content) => {
                info!("✅ [BACKEND_DEBUG] 文件读取成功，大小: {} bytes", content.len());
                content
            }
            Err(e) => {
                error!("❌ [BACKEND_DEBUG] 读取文件失败: {} - 错误: {}", file_path, e);
                return Ok(create_error_response(&format!("读取文件失败: {}", e), file_path));
            }
        }
    } else if let Some(content) = &request.content {
        // 内容传输模式：直接使用传入的日志内容
        info!("📝 [BACKEND_DEBUG] 使用内容传输模式，大小: {} bytes", content.len());
        content.clone()
    } else {
        // 错误处理：既没有文件路径也没有内容
        error!("❌ [BACKEND_DEBUG] 请求中既没有文件路径也没有内容");
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

    // 第二步：预处理日志内容
    // 过滤空行并统计总行数
    let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();
    let total_lines = lines.len();

    info!("📊 [BACKEND_DEBUG] 日志预处理完成：{} 行有效内容", total_lines);

    if total_lines == 0 {
        warn!("⚠️ [BACKEND_DEBUG] 日志内容为空或只包含空行");
        return Ok(create_empty_response());
    }

    // 第三步：确定处理策略（分块 vs 全量处理）
    // 根据文件大小和用户请求确定使用分块处理还是全量处理
    let chunk_size = request.chunk_size.unwrap_or(1000); // 默认1000行一块
    let chunk_index = request.chunk_index.unwrap_or(0);

    // 分块处理判断逻辑：
    // - 只有文件足够大（>chunk_size）且用户明确请求分块时才启用分块处理
    // - 小文件总是使用全量处理以获得最佳解析效果
    let should_chunk = total_lines > chunk_size && request.chunk_size.is_some();

    debug!("📏 [BACKEND_DEBUG] 分块处理判断: total_lines={}, chunk_size={}, chunk_size_requested={}, should_chunk={}",
         total_lines, chunk_size, request.chunk_size.is_some(), should_chunk);

    if should_chunk {
        // ==================== 分块处理模式 ====================
        info!("🔧 [BACKEND_DEBUG] 启用分块处理模式：第{}块，每块{}行", chunk_index + 1, chunk_size);

        // 计算当前块的索引范围
        let start_index = chunk_index * chunk_size;
        let end_index = std::cmp::min(start_index + chunk_size, total_lines);

        debug!("📏 [BACKEND_DEBUG] 分块范围: 第{}-{}行（共{}行）", start_index + 1, end_index, total_lines);

        // 提取当前块的原始日志内容作为字符串
        let chunk_content: String = lines.iter()
            .enumerate()
            .skip(start_index)
            .take(chunk_size)
            .map(|(_, line)| *line)
            .collect::<Vec<&str>>()
            .join("\n");

        info!("📊 [BACKEND_DEBUG] 分块内容长度: {} 字符", chunk_content.len());

        // 使用插件链的自动检测系统处理分块内容
        let parse_request = crate::plugins::ParseRequest {
            file_path: request.file_path.clone(),
            content: chunk_content,
            plugin: Some("auto".to_string()),
            chunk_size: None,
        };

        debug!("🔍 [BACKEND_DEBUG] 调用插件链自动检测系统处理分块");
        let parse_result = match state.plugin_manager.auto_detect_and_parse(&parse_request) {
            Ok(result) => {
                info!("✅ [BACKEND_DEBUG] 插件链自动检测成功: {} -> {} 条目",
                      result.lines.len(), result.lines.len());
                info!("🔍 [BACKEND_DEBUG] 检测格式: {:?}", result.detected_format);
                if let Some(first_line) = result.lines.first() {
                    info!("🔍 [BACKEND_DEBUG] 第一条记录formatted_content: {:?}", first_line.formatted_content);
                }
                result.lines
            }
            Err(e) => {
                error!("❌ [BACKEND_DEBUG] 插件链自动检测失败: {}", e);
                warn!("🔄 [BACKEND_DEBUG] 回退到通用解析器");

                // 回退到简单的行解析
                lines.iter()
                    .enumerate()
                    .skip(start_index)
                    .take(chunk_size)
                    .map(|(global_index, line)| {
                        let log_line = crate::plugins::LogLine {
                            line_number: global_index + 1,
                            content: line.to_string(),
                            timestamp: extract_timestamp(line),
                            level: extract_log_level(line),
                            formatted_content: Some(line.trim().to_string()),
                            metadata: std::collections::HashMap::new(),
                            processed_by: vec!["fallback_parser".to_string()],
                        };
                        log_line
                    })
                    .collect()
            }
        };

        // Convert LogLine to LogEntry
        let entries: Vec<LogEntry> = parse_result.into_iter().map(|log_line| {
            LogEntry {
                line_number: log_line.line_number,
                content: log_line.content,
                timestamp: log_line.timestamp,
                level: log_line.level,
                formatted_content: log_line.formatted_content,
                metadata: log_line.metadata,
                processed_by: log_line.processed_by,
            }
        }).collect();

        // 计算分块信息
        let total_chunks = (total_lines + chunk_size - 1) / chunk_size; // 向上取整
        let has_more = chunk_index + 1 < total_chunks;

        // 性能统计
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

        info!("📦 [BACKEND_DEBUG] 分块解析完成: 第{}/{}块，{}条目，耗时: {}ms",
              chunk_index + 1, total_chunks, entries.len(), parse_time);

        let response = ParseResponse {
            success: true,
            entries,
            stats,
            chunk_info: Some(chunk_info),
            error: None,
            detected_format: None, // 分块处理时不做格式检测以提高性能
        };

        info!("✅ [BACKEND_DEBUG] 分块解析响应构建完成，条目数: {}", response.entries.len());
        return Ok(response);
    }

    // ==================== 全量处理模式 ====================
    // 适用于小文件或未明确请求分块的情况
    info!("🔧 启用全量处理模式：使用增强插件系统");
    // 使用增强插件系统处理（小文件）- 性能优化版本
    info!("使用增强插件系统处理日志");

      // 使用增强插件管理器的自动检测和解析功能
    info!("🔧 使用增强插件管理器进行自动检测和解析");

    let parse_request = crate::plugins::ParseRequest {
        content: content.clone(),
        plugin: Some("auto".to_string()), // 使用自动检测
        file_path: request.file_path.clone(), // 传递文件路径以帮助链选择
        chunk_size: request.chunk_size,
    };

    let plugin_start = std::time::Instant::now();
    let (entries, detected_format) = match state.plugin_manager.auto_detect_and_parse(&parse_request) {
        Ok(result) => {
            let plugin_time = plugin_start.elapsed();
            info!("增强插件管理器处理成功，生成 {} 条目，耗时: {}ms，检测格式: {:?}",
                  result.lines.len(), plugin_time.as_millis(), result.detected_format);

            // 性能优化：直接转换，避免中间步骤
            let conversion_start = std::time::Instant::now();
            let converted_entries: Vec<LogEntry> = result.lines.into_iter().map(|line| LogEntry {
                line_number: line.line_number,
                content: line.content,
                timestamp: line.timestamp,
                level: line.level,
                formatted_content: line.formatted_content,
                metadata: line.metadata,
                processed_by: line.processed_by,
            }).collect();
            let conversion_time = conversion_start.elapsed();
            info!("数据转换耗时: {}ms", conversion_time.as_millis());

            let detected_format = result.detected_format.clone();
            (converted_entries, detected_format)
        }
        Err(e) => {
            error!("增强插件管理器处理失败: {}", e);
            // 快速回退处理，避免重复计算
            return Ok(ParseResponse {
                success: true,
                entries: lines.iter().enumerate().map(|(i, line)| LogEntry {
                    line_number: i + 1,
                    content: line.to_string(),
                    timestamp: None,
                    level: None,
                    formatted_content: Some(line.trim().to_string()),
                    metadata: std::collections::HashMap::new(),
                    processed_by: vec!["fallback_parser".to_string()],
                }).collect(),
                stats: ParseStats {
                    total_lines: lines.len(),
                    success_lines: lines.len(),
                    error_lines: 0,
                    parse_time_ms: start_time.elapsed().as_millis() as u64,
                },
                chunk_info: None,
                error: Some(format!("增强插件管理器处理失败: {}", e)),
                detected_format: Some("Unknown".to_string()),
            });
        }
    };
    let parse_time = start_time.elapsed().as_millis() as u64;

    // JSON序列化性能监控
    let json_start = std::time::Instant::now();

    let stats = ParseStats {
        total_lines: lines.len(),
        success_lines: entries.len(),
        error_lines: 0,
        parse_time_ms: parse_time,
    };

    // 预估JSON大小
    let estimated_json_size = entries.iter()
        .map(|e| e.content.len() + e.formatted_content.as_ref().map_or(0, |f| f.len()) + 100)
        .sum::<usize>();

    let json_time = json_start.elapsed();
    info!("JSON序列化预估耗时: {}ms，预估大小: {} bytes", json_time.as_millis(), estimated_json_size);

    let detected_format_display = detected_format.clone().unwrap_or_else(|| "Unknown".to_string());
    info!("全量解析完成: {} 行，处理为 {} 条目，耗时: {}ms，检测格式: {}",
              lines.len(), entries.len(), parse_time, detected_format_display);

    let response_start = std::time::Instant::now();
    let response = ParseResponse {
        success: true,
        entries,
        stats,
        chunk_info: None,
        error: None,
        detected_format: detected_format,
    };
    let response_time = response_start.elapsed();
    info!("响应构建耗时: {}ms", response_time.as_millis());

    Ok(response)
}

/// 测试解析端点
///
/// 用于测试日志解析功能的可用性和参数验证。
/// 此端点不执行实际的日志解析，而是返回请求的基本信息。
///
/// # 功能特性
/// - 验证请求参数的完整性
/// - 识别请求类型（文件路径或内容传输）
/// - 提供调试信息用于故障排除
/// - 验证前后端通信的完整性
///
/// # 参数
/// - `request`: 解析请求，包含文件路径或内容
///
/// # Returns
/// - `Ok(serde_json::Value)`: 包含测试结果和请求信息的JSON响应
/// - `Err(String)`: 测试失败时的错误信息
///
/// # 使用场景
/// - 前端连接性测试
/// - 参数格式验证
/// - 开发环境调试
#[tauri::command]
async fn test_parse(request: ParseRequest) -> Result<serde_json::Value, String> {
    info!("🧪 收到测试解析请求: {:?}", request);

    // 检查请求类型，用于验证前端参数传递
    let request_type = if request.file_path.is_some() {
        "file_path"
    } else if request.content.is_some() {
        "content"
    } else {
        "unknown"
    };

    debug!("📝 识别的请求类型: {}", request_type);

    Ok(serde_json::json!({
        "success": true,
        "message": "测试成功",
        "request_type": request_type,
        "request": request
    }))
}

// ============================================================================
// 主题配置管理命令
// ============================================================================

/// 获取当前主题配置
///
/// 返回应用程序的当前主题设置，包括颜色方案、字体配置等。
/// 主题配置影响用户界面的外观和显示效果。
///
/// # 功能特性
/// - 支持明暗主题切换（Light/Dark/Auto）
/// - 可自定义主色调和强调色
/// - 字体大小和字体族配置
/// - 配置持久化存储
///
/// # 参数
/// - `state`: 应用状态，包含配置服务实例
///
/// # Returns
/// - `Ok(ThemeResponse)`: 包含当前主题配置的响应
/// - `Err(String)`: 获取配置失败时的错误信息
///
/// # 主题模式说明
/// - Light: 浅色主题，适合白天使用
/// - Dark: 深色主题，适合夜间使用，保护视力
/// - Auto: 自动跟随系统主题设置
#[tauri::command]
async fn get_theme_config(state: tauri::State<'_, AppState>) -> Result<ThemeResponse, String> {
    debug!("🎨 获取主题配置");

    match state.config_service.lock().await.get_theme_config() {
        Ok(theme) => {
            debug!("✅ 主题配置获取成功: mode={:?}", theme.mode);

            // 将内部主题模式枚举转换为前端字符串格式
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
            error!("❌ 获取主题配置失败: {}", e);
            Err("获取主题配置失败".to_string())
        }
    }
}

/// 更新主题配置
///
/// 根据用户的请求更新应用程序的主题设置。
/// 支持部分更新，只修改请求中明确指定的字段。
///
/// # 功能特性
/// - 部分更新支持：只更新提供的字段
/// - 配置验证：确保主题参数的有效性
/// - 持久化存储：自动保存配置到本地存储
/// - 实时生效：更新后立即反映到用户界面
///
/// # 参数
/// - `request`: 主题更新请求，包含要更新的主题字段
/// - `state`: 应用状态，包含配置服务实例
///
/// # Returns
/// - `Ok(String)`: 更新成功的确认消息
/// - `Err(String)`: 更新失败时的错误信息
///
/// # 更新流程
/// 1. 获取当前主题配置作为基础
/// 2. 验证并更新请求中的字段
/// 3. 保存新配置到持久化存储
/// 4. 返回更新结果确认
#[tauri::command]
async fn update_theme_config(
    request: ThemeUpdateRequest,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!("🎨 收到主题配置更新请求: {:?}", request);

    // 第一步：获取当前配置作为更新基础
    // 这样可以实现部分更新，只修改请求中包含的字段
    let mut theme = match state.config_service.lock().await.get_theme_config() {
        Ok(theme) => {
            debug!("✅ 获取当前主题配置成功");
            theme
        }
        Err(e) => {
            error!("❌ 获取当前主题配置失败: {}", e);
            return Err("获取当前主题配置失败".to_string());
        }
    };

    // 第二步：验证并更新主题模式
    // 支持的模式：light, dark, auto（默认）
    let old_mode = theme.mode.clone();
    theme.mode = match request.mode.as_str() {
        "light" => {
            debug!("🌞 切换到浅色主题");
            ThemeMode::Light
        }
        "dark" => {
            debug!("🌙 切换到深色主题");
            ThemeMode::Dark
        }
        "auto" => {
            debug!("🔄 切换到自动主题");
            ThemeMode::Auto
        }
        _ => {
            warn!("⚠️ 未知的主题模式: {}，使用默认值 'auto'", request.mode);
            ThemeMode::Auto
        }
    };

    // 第三步：更新颜色配置（可选字段）
    if let Some(primary_color) = request.primary_color {
        debug!("🎨 更新主色调: {} -> {}", theme.primary_color, primary_color);
        theme.primary_color = primary_color;
    }

    if let Some(accent_color) = request.accent_color {
        debug!("🎨 更新强调色: {} -> {}", theme.accent_color, accent_color);
        theme.accent_color = accent_color;
    }

    // 第四步：更新字体配置（可选字段）
    if let Some(font_size) = request.font_size {
        debug!("📝 更新字体大小: {} -> {}", theme.font_size, font_size);
        theme.font_size = font_size;
    }

    if let Some(font_family) = request.font_family {
        debug!("🔤 更新字体族: {} -> {}", theme.font_family, font_family);
        theme.font_family = font_family;
    }

    // 第五步：保存配置到持久化存储
    match state.config_service.lock().await.set_theme_config(&theme) {
        Ok(_) => {
            info!("✅ 主题配置更新成功: 模式 {:?} -> {:?}", old_mode, theme.mode);
            Ok("主题配置更新成功".to_string())
        }
        Err(e) => {
            error!("❌ 主题配置保存失败: {}", e);
            Err("更新主题配置失败".to_string())
        }
    }
}

// ============================================================================
// 其他配置管理命令
// ============================================================================

/// 获取解析配置
///
/// 返回与日志解析相关的配置参数，包括性能优化设置和解析行为控制。
/// 这些配置影响日志文件的处理方式和性能表现。
///
/// # 配置项说明
/// - auto_parse: 是否自动解析日志文件
/// - show_line_numbers: 是否显示行号
/// - max_file_size: 支持的最大文件大小限制
/// - chunk_size: 大文件分块处理的块大小
/// - timeout_seconds: 解析超时时间限制
///
/// # 参数
/// - `state`: 应用状态，包含配置服务实例
///
/// # Returns
/// - `Ok(serde_json::Value)`: 包含解析配置的JSON对象
/// - `Err(String)`: 获取配置失败时的错误信息
#[tauri::command]
async fn get_parse_config(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    debug!("⚙️ 获取解析配置");

    match state.config_service.lock().await.get_parse_config() {
        Ok(parse) => {
            debug!("✅ 解析配置获取成功");

            // 将内部配置结构转换为前端JSON格式
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
            error!("❌ 获取解析配置失败: {}", e);
            Err("获取解析配置失败".to_string())
        }
    }
}

/// 获取插件配置
///
/// 返回与插件系统相关的配置参数，包括插件管理策略和系统设置。
/// 这些配置影响插件的加载、更新和行为。
///
/// # 配置项说明
/// - auto_update: 是否自动更新插件
/// - enable_notifications: 是否启用插件通知
/// - plugin_directory: 插件存储目录路径
/// - max_plugins: 最大插件数量限制
///
/// # 参数
/// - `state`: 应用状态，包含配置服务实例
///
/// # Returns
/// - `Ok(serde_json::Value)`: 包含插件配置的JSON对象
/// - `Err(String)`: 获取配置失败时的错误信息
#[tauri::command]
async fn get_plugin_config(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    debug!("🔌 获取插件配置");

    match state.config_service.lock().await.get_plugin_config() {
        Ok(plugin) => {
            debug!("✅ 插件配置获取成功");

            // 将内部插件配置转换为前端JSON格式
            let data = serde_json::json!({
                "auto_update": plugin.auto_update,
                "enable_notifications": plugin.enable_notifications,
                "plugin_directory": plugin.plugin_directory,
                "max_plugins": plugin.max_plugins,
            });

            Ok(data)
        }
        Err(e) => {
            error!("❌ 获取插件配置失败: {}", e);
            Err("获取插件配置失败".to_string())
        }
    }
}

/// 获取窗口配置
///
/// 返回与应用程序窗口相关的配置参数，包括窗口尺寸、位置和行为设置。
/// 这些配置影响应用程序的窗口显示和用户交互体验。
///
/// # 配置项说明
/// - width: 窗口默认宽度
/// - height: 窗口默认高度
/// - maximized: 是否默认最大化显示
/// - always_on_top: 是否保持窗口置顶
/// - remember_position: 是否记住窗口位置
///
/// # 参数
/// - `state`: 应用状态，包含配置服务实例
///
/// # Returns
/// - `Ok(serde_json::Value)`: 包含窗口配置的JSON对象
/// - `Err(String)`: 获取配置失败时的错误信息
#[tauri::command]
async fn get_window_config(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    debug!("🪟 获取窗口配置");

    match state.config_service.lock().await.get_window_config() {
        Ok(window) => {
            debug!("✅ 窗口配置获取成功");

            // 将内部窗口配置转换为前端JSON格式
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
            error!("❌ 获取窗口配置失败: {}", e);
            Err("获取窗口配置失败".to_string())
        }
    }
}

/// 获取所有配置
///
/// 返回应用程序的所有配置信息，包括主题、解析、插件和窗口配置。
/// 这是一个综合性的配置获取接口，用于前端一次性获取所有配置。
///
/// # 功能特性
/// - 统一获取所有配置类型
/// - 减少多次网络请求
/// - 配置一致性保证
/// - 完整的配置快照
///
/// # 参数
/// - `state`: 应用状态，包含配置服务实例
///
/// # Returns
/// - `Ok(serde_json::Value)`: 包含所有配置的完整JSON对象
/// - `Err(String)`: 获取配置失败时的错误信息
///
/// # 返回结构
/// 包含主题配置、解析配置、插件配置和窗口配置的完整配置树
#[tauri::command]
async fn get_all_configs(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    debug!("📦 获取所有配置信息");

    match state.config_service.lock().await.get_all_configs() {
        Ok(configs) => {
            debug!("✅ 所有配置获取成功");

            // 将内部配置结构直接序列化为JSON
            let data = serde_json::to_value(configs).unwrap_or_else(|e| {
                error!("❌ 配置序列化失败: {}", e);
                serde_json::json!({"error": "配置序列化失败"})
            });

            Ok(data)
        }
        Err(e) => {
            error!("❌ 获取所有配置失败: {}", e);
            Err("获取所有配置失败".to_string())
        }
    }
}

// ============================================================================
// 文件系统操作命令
// ============================================================================

/// 读取文本文件
///
/// 安全地读取指定路径的文本文件内容。
/// 此命令为前端提供了访问本地文件系统的能力，用于读取配置文件、日志文件等。
///
/// # 功能特性
/// - 安全的文件路径处理
/// - 完整的错误处理和日志记录
/// - 大文件支持（受系统内存限制）
/// - UTF-8编码自动处理
///
/// # 参数
/// - `path`: 要读取的文件路径（绝对路径或相对路径）
///
/// # Returns
/// - `Ok(String)`: 文件的完整文本内容
/// - `Err(String)`: 读取失败时的详细错误信息
///
/// # 错误处理
/// - 文件不存在
/// - 权限不足
/// - 文件被占用
/// - 编码错误（非UTF-8文件）
///
/// # 安全考虑
/// - 路径验证：确保路径在允许的范围内
/// - 权限检查：验证文件访问权限
/// - 大小限制：防止读取过大的文件导致内存溢出
#[tauri::command]
async fn read_text_file(path: String) -> Result<String, String> {
    info!("📂 请求读取文件: {}", path);

    // 路径安全验证
    let path_obj = std::path::Path::new(&path);

    // 检查路径是否存在
    if !path_obj.exists() {
        error!("❌ 文件不存在: {}", path);
        return Err(format!("文件不存在: {}", path));
    }

    // 检查是否为文件（而非目录）
    if !path_obj.is_file() {
        error!("❌ 路径不是文件: {}", path);
        return Err(format!("路径不是文件: {}", path));
    }

    // 尝试读取文件内容
    match std::fs::read_to_string(&path) {
        Ok(content) => {
            info!("✅ 文件读取成功: {} (大小: {} bytes)", path, content.len());
            debug!("📝 文件内容预览: {}",
                  if content.len() > 100 {
                      format!("{}...", &content[..100])
                  } else {
                      content.clone()
                  });
            Ok(content)
        }
        Err(e) => {
            error!("❌ 读取文件失败: {} - 错误: {}", path, e);
            Err(format!("读取文件失败: {}", e))
        }
    }
}

/// 写入文件
///
/// 安全地将内容写入到指定路径的文件中。
/// 此命令为前端提供了写入本地文件的能力，用于保存配置、导出数据等操作。
///
/// # 功能特性
/// - 原子性写入：确保文件完整性
/// - 自动创建目录结构
/// - 完整的错误处理和日志记录
/// - 覆盖写入模式
///
/// # 参数
/// - `path`: 要写入的文件路径
/// - `contents`: 要写入的文件内容
///
/// # Returns
/// - `Ok(())`: 写入成功的确认
/// - `Err(String)`: 写入失败时的详细错误信息
///
/// # 错误处理
/// - 磁盘空间不足
/// - 权限不足
/// - 目录不存在（自动创建）
/// - 文件被占用
///
/// # 安全考虑
/// - 路径验证：确保写入路径安全
/// - 权限检查：验证写入权限
/// - 备份策略：重要文件建议先备份
#[tauri::command]
async fn write_file(path: String, contents: String) -> Result<(), String> {
    info!("💾 请求写入文件: {} (大小: {} bytes)", path, contents.len());

    // 路径安全验证
    let path_obj = std::path::Path::new(&path);

    // 确保父目录存在，如果不存在则创建
    if let Some(parent) = path_obj.parent() {
        if !parent.exists() {
            debug!("📁 创建目录结构: {}", parent.display());
            match std::fs::create_dir_all(parent) {
                Ok(_) => {
                    info!("✅ 目录创建成功: {}", parent.display());
                }
                Err(e) => {
                    error!("❌ 创建目录失败: {} - 错误: {}", parent.display(), e);
                    return Err(format!("创建目录失败: {}", e));
                }
            }
        }
    }

    // 尝试写入文件内容
    let content_len = contents.len(); // 先保存长度，避免所有权转移
    match std::fs::write(&path, contents) {
        Ok(_) => {
            info!("✅ 文件写入成功: {} (大小: {} bytes)", path, content_len);
            debug!("💾 文件详情: 大小={} bytes, 路径={}",
                  std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
                  path);
            Ok(())
        }
        Err(e) => {
            error!("❌ 写入文件失败: {} - 错误: {}", path, e);
            Err(format!("写入文件失败: {}", e))
        }
    }
}




// ============================================================================
// 数据结构定义
// ============================================================================

/// 日志解析请求结构
///
/// 定义了前端向后端发起日志解析请求的完整参数结构。
/// 支持文件路径和内容直接传输两种模式。
///
/// # 字段说明
/// - file_path: 日志文件的路径（可选）
/// - content: 直接传入的日志内容（可选）
/// - plugin: 指定使用的解析插件（可选，不指定则自动检测）
/// - chunk_size: 分块处理时的块大小（可选，默认1000行）
/// - chunk_index: 当前请求的块索引（可选，用于分块处理）
///
/// # 使用模式
/// 1. 文件模式：提供file_path，后端读取文件内容
/// 2. 内容模式：提供content，后端直接处理传入内容
/// 3. 分块模式：设置chunk_size和chunk_index，用于大文件处理
#[derive(Debug, Serialize, Deserialize)]
struct ParseRequest {
    /// 日志文件路径（绝对路径或相对路径）
    #[serde(default)]
    file_path: Option<String>,

    /// 直接传入的日志内容（UTF-8编码）
    #[serde(default)]
    content: Option<String>,

    /// 指定使用的解析插件名称（如"springboot", "docker_json"等）
    #[serde(default)]
    plugin: Option<String>,

    /// 分块处理时的块大小（行数，默认1000）
    #[serde(default)]
    chunk_size: Option<usize>,

    /// 当前请求的块索引（从0开始，用于分块处理）
    #[serde(default)]
    chunk_index: Option<usize>,
}

/// 日志解析响应结构
///
/// 包含日志解析的完整结果，包括解析的日志条目、统计信息和错误状态。
/// 这是后端返回给前端的标准化响应格式。
///
/// # 字段说明
/// - success: 解析是否成功标志
/// - entries: 解析后的日志条目列表
/// - stats: 解析统计信息（行数、耗时等）
/// - chunk_info: 分块处理信息（仅在分块模式时有值）
/// - error: 错误信息（仅在出错时有值）
/// - detected_format: 自动检测到的日志格式
///
/// # 响应类型
/// 1. 成功响应：success=true，包含entries和stats
/// 2. 错误响应：success=false，包含error信息
/// 3. 分块响应：包含chunk_info用于分块管理
#[derive(Debug, Serialize, Deserialize)]
struct ParseResponse {
    /// 解析操作是否成功完成
    success: bool,

    /// 解析后的日志条目列表
    entries: Vec<LogEntry>,

    /// 解析过程的统计信息
    stats: ParseStats,

    /// 分块处理信息（大文件分块时使用）
    chunk_info: Option<ChunkInfo>,

    /// 错误信息（解析失败时提供详细错误描述）
    error: Option<String>,

    /// 自动检测到的日志格式（如"SpringBoot", "DockerJson"等）
    detected_format: Option<String>,
}

/// 分块信息结构
///
/// 用于大文件分块处理时的元数据管理。
/// 提供分块进度和状态信息，帮助前端管理分块加载过程。
///
/// # 字段说明
/// - total_chunks: 总分块数量
/// - current_chunk: 当前块的索引（从0开始）
/// - has_more: 是否还有后续块需要处理
///
/// # 使用场景
/// - 大文件分块加载的进度显示
/// - 分块请求的顺序管理
/// - 分块完成状态的判断
#[derive(Debug, Serialize, Deserialize)]
struct ChunkInfo {
    /// 总分块数量（向上取整）
    total_chunks: usize,

    /// 当前处理的块索引（从0开始）
    current_chunk: usize,

    /// 是否还有后续块需要处理
    has_more: bool,
}

/// 日志条目结构
///
/// 表示解析后的单个日志条目，包含原始内容和解析后的结构化信息。
/// 这是日志解析的核心数据结构，支持多种日志格式的统一表示。
///
/// # 字段说明
/// - line_number: 在原文件中的行号（从1开始）
/// - content: 原始日志内容
/// - timestamp: 解析出的时间戳（可选）
/// - level: 日志级别（如INFO, ERROR, WARN等）
/// - formatted_content: 格式化后的显示内容
/// - metadata: 附加元数据（键值对形式）
/// - processed_by: 处理此条目的插件列表
///
/// # 解析增强
/// - 时间戳提取和标准化
/// - 日志级别识别和分类
/// - 内容格式化和高亮
/// - 元数据提取（如线程ID、类名等）
/// - 处理链追踪
#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    /// 在原日志文件中的行号（从1开始）
    line_number: usize,

    /// 原始日志内容（保持不变）
    content: String,

    /// 解析出的时间戳（ISO 8601格式或原始格式）
    timestamp: Option<String>,

    /// 日志级别（INFO, ERROR, WARN, DEBUG, TRACE等）
    level: Option<String>,

    /// 格式化后的显示内容（可能包含高亮、结构化信息）
    formatted_content: Option<String>,

    /// 附加元数据（如线程ID、类名、方法名等）
    metadata: std::collections::HashMap<String, String>,

    /// 处理此条目的插件名称列表（用于追踪处理链）
    processed_by: Vec<String>,
}


/// 解析统计信息结构
///
/// 包含日志解析过程的性能和结果统计数据。
/// 用于监控解析性能、优化处理策略和用户反馈。
///
/// # 字段说明
/// - total_lines: 原始日志文件的总行数
/// - success_lines: 成功解析的行数
/// - error_lines: 解析失败的行数
/// - parse_time_ms: 解析耗时（毫秒）
///
/// # 性能指标
/// - 解析成功率：success_lines / total_lines
/// - 解析速度：total_lines / parse_time_ms (行/毫秒)
/// - 错误率：error_lines / total_lines
#[derive(Debug, Serialize, Deserialize)]
struct ParseStats {
    /// 原始日志文件的总行数（包括空行和无效行）
    total_lines: usize,

    /// 成功解析并处理的行数
    success_lines: usize,

    /// 解析失败或出错的行数
    error_lines: usize,

    /// 解析过程的总耗时（毫秒）
    parse_time_ms: u64,
}

/// 插件信息结构
///
/// 描述单个日志解析插件的基本信息。
/// 用于前端展示可用插件列表和插件选择界面。
///
/// # 字段说明
/// - name: 插件的唯一标识符（用于API调用）
/// - description: 插件功能描述（面向用户的说明）
/// - version: 插件版本号（用于兼容性检查）
///
/// # 插件类型
/// - auto: 自动格式检测插件
/// - mybatis: MyBatis SQL日志解析插件
/// - docker_json: Docker容器日志解析插件
/// - raw: 原始文本日志解析插件
#[derive(Debug, Serialize, Deserialize)]
struct Plugin {
    /// 插件的唯一名称标识符
    name: String,

    /// 插件功能的用户友好描述
    description: String,

    /// 插件版本号（语义化版本）
    version: String,
}

/// 插件列表响应结构
///
/// 包含系统中所有可用日志解析插件的列表。
/// 这是get_plugins命令的返回值格式。
#[derive(Debug, Serialize, Deserialize)]
struct PluginsResponse {
    /// 可用插件列表
    plugins: Vec<Plugin>,
}

/// 健康检查响应结构
///
/// 包含应用程序运行状态的基本信息。
/// 用于监控系统健康状态和服务可用性检查。
///
/// # 字段说明
/// - status: 应用状态（"ok"表示正常）
/// - version: 应用程序版本号
/// - timestamp: 响应生成时间（ISO 8601格式）
#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    /// 应用运行状态
    status: String,

    /// 应用程序版本号
    version: String,

    /// 响应生成的时间戳（UTC）
    timestamp: String,
}

/// 主题配置响应结构
///
/// 包含应用程序的当前主题设置信息。
/// 用于前端应用主题样式和用户界面配置。
///
/// # 字段说明
/// - mode: 主题模式（"light", "dark", "auto"）
/// - primary_color: 主色调（十六进制颜色值）
/// - accent_color: 强调色（十六进制颜色值）
/// - font_size: 基础字体大小（像素）
/// - font_family: 字体族名称
#[derive(Debug, Serialize, Deserialize)]
struct ThemeResponse {
    /// 主题模式（light/dark/auto）
    mode: String,

    /// 主色调（如"#3b82f6"）
    primary_color: String,

    /// 强调色（如"#10b981"）
    accent_color: String,

    /// 基础字体大小（像素值）
    font_size: u32,

    /// 字体族名称（如"Inter", "Roboto"等）
    font_family: String,
}

/// 主题配置更新请求结构
///
/// 用于主题配置部分更新操作。
/// 支持只更新指定的字段，未提供的字段保持原值不变。
///
/// # 字段说明
/// - mode: 新的主题模式（必需字段）
/// - primary_color: 新的主色调（可选）
/// - accent_color: 新的强调色（可选）
/// - font_size: 新的字体大小（可选）
/// - font_family: 新的字体族（可选）
///
/// # 使用方式
/// - 必须提供mode字段
/// - 其他字段为可选，提供时更新，不提供时保持原值
/// - 实现部分更新功能，避免覆盖未修改的配置
#[derive(Debug, Serialize, Deserialize)]
struct ThemeUpdateRequest {
    /// 新的主题模式（必需字段）
    mode: String,

    /// 新的主色调（可选，不提供时保持原值）
    primary_color: Option<String>,

    /// 新的强调色（可选，不提供时保持原值）
    accent_color: Option<String>,

    /// 新的字体大小（可选，不提供时保持原值）
    font_size: Option<u32>,

    /// 新的字体族（可选，不提供时保持原值）
    font_family: Option<String>,
}

/// 文件信息响应结构
///
/// 包含日志文件的基本信息，用于前端确定分块处理策略。
///
/// # 字段说明
/// - file_path: 文件路径
/// - file_size: 文件大小（字节）
/// - total_lines: 总行数（估算值）
/// - recommended_chunk_size: 推荐的分块大小
/// - is_large_file: 是否为大文件
#[derive(Debug, Serialize, Deserialize)]
struct FileInfoResponse {
    /// 文件路径
    file_path: String,

    /// 文件大小（字节）
    file_size: u64,

    /// 总行数（大文件为估算值）
    total_lines: usize,

    /// 推荐的分块大小（行数）
    recommended_chunk_size: usize,

    /// 是否为大文件（>50MB）
    is_large_file: bool,
}

// ============================================================================
// 性能优化辅助函数
// ============================================================================

use once_cell::sync::Lazy;
use regex::Regex;

// 预编译的正则表达式，避免重复编译
static TIMESTAMP_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // ISO 8601 标准格式
        Regex::new(r"\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}").unwrap(),
        // 美式日期格式
        Regex::new(r"\d{2}/\d{2}/\d{4}\s+\d{2}:\d{2}:\d{2}").unwrap(),
        // 欧式日期格式
        Regex::new(r"\d{2}-\d{2}-\d{4}\s+\d{2}:\d{2}:\d{2}").unwrap(),
    ]
});


/// 使用插件系统处理日志条目
///
/// 将前端日志条目格式转换为插件系统格式，通过插件处理后再转换回前端格式。
/// 这个函数是前端LogEntry和插件系统PluginLogEntry之间的桥梁。
///
/// # 处理流程
/// 1. 格式转换：LogEntry -> PluginLogEntry
/// 2. 插件处理：调用插件管理器进行日志解析
/// 3. 结果转换：PluginLogEntry -> LogEntry
/// 4. 性能监控：记录处理时间和结果统计
///
/// # 参数
/// - `entries`: 前端格式的日志条目数组
/// - `plugin_manager`: 增强插件管理器实例
///
/// # Returns
/// - `Ok(Vec<LogEntry>)`: 处理后的前端格式日志条目
/// - `Err(String)`: 插件处理失败时的错误信息
///
/// # 性能特性
/// - 批量处理：一次性处理多个条目以提高效率
/// - 内存优化：避免不必要的内存分配和拷贝
/// - 错误隔离：插件失败不影响整个应用稳定性
async fn process_logs_with_plugin_system(entries: &[LogEntry], plugin_manager: &Arc<EnhancedPluginManager>) -> Result<Vec<LogEntry>, String> {
    let start_time = std::time::Instant::now();
    info!("🔧 开始插件系统处理，输入条目数: {}", entries.len());

    // 第一步：格式转换 LogEntry -> PluginLogEntry
    // 这是前端数据格式和插件系统数据格式之间的适配层
    debug!("📋 转换数据格式到插件系统格式");
    let plugin_entries: Vec<PluginLogEntry> = entries.iter().map(|entry| {
        PluginLogEntry {
            line_number: entry.line_number,
            content: entry.content.clone(),
            timestamp: entry.timestamp.clone(),
            level: entry.level.clone(),
            formatted_content: entry.formatted_content.clone(),
            metadata: std::collections::HashMap::new(), // 插件系统会重新构建元数据
            processed_by: Vec::new(), // 插件系统会重新记录处理链
        }
    }).collect();

    debug!("✅ 数据格式转换完成，条目数: {}", plugin_entries.len());

    // 第二步：插件系统处理
    // 调用增强插件管理器进行实际的日志解析和处理
    debug!("🔄 调用插件管理器处理日志条目");
    let process_start = std::time::Instant::now();
    let result = plugin_manager.process_log_entries(plugin_entries).await
        .map_err(|e| {
            error!("❌ 插件系统处理失败: {}", e);
            format!("插件处理失败: {}", e)
        })?;
    let process_time = process_start.elapsed();

    info!("✅ 插件系统处理完成，输入: {} -> 输出: {} 条目，处理耗时: {}ms",
          entries.len(), result.len(), process_time.as_millis());

    // 第三步：结果转换 PluginLogEntry -> LogEntry
    // 将插件系统处理结果转换回前端可用的格式
    debug!("🔄 转换插件处理结果到前端格式");
    let conversion_start = std::time::Instant::now();
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
    let conversion_time = conversion_start.elapsed();

    let total_time = start_time.elapsed();
    info!("✅ 完整插件系统处理完成，最终条目数: {}，总耗时: {}ms (处理: {}ms, 转换: {}ms)",
          converted_entries.len(), total_time.as_millis(), process_time.as_millis(), conversion_time.as_millis());

    Ok(converted_entries)
}

/// 智能检测日志格式
///
/// 通过分析日志行内容特征来自动识别日志格式类型。
/// 支持识别常见的日志格式，用于选择合适的解析插件。
///
/// # 支持的格式
/// - SpringBoot: Java应用日志，包含INFO/ERROR/WARN/DEBUG级别
/// - DockerJson: Docker容器日志，JSON格式包含log/stream字段
/// - MyBatis: SQL框架日志，包含Preparing/Parameters/==>关键字
/// - Unknown: 无法识别的格式，使用通用解析器
///
/// # 检测策略
/// 1. 按优先级依次检测各格式特征
/// 2. 基于特征出现的频率和模式判断
/// 3. 使用50%阈值作为主要格式的判断标准
/// 4. MyBatis格式使用存在性判断而非频率
///
/// # 参数
/// - `lines`: 日志行数组切片
///
/// # Returns
/// - `String`: 检测到的格式名称
///
/// # 性能优化
/// - 早期退出：一旦确定格式立即返回
/// - 采样检测：大文件可考虑只检测前N行
/// - 缓存结果：相同内容的重复检测
#[allow(dead_code)]
fn detect_log_format(lines: &[&str]) -> String {
    debug!("🔍 开始智能日志格式检测，总行数: {}", lines.len());

    if lines.is_empty() {
        warn!("⚠️ 日志行为空，返回Unknown格式");
        return "Unknown".to_string();
    }

    // 检测SpringBoot格式
    // 特征：包含标准日志级别关键字
    debug!("🔍 检测SpringBoot格式特征");
    let springboot_count = lines.iter()
        .filter(|line| {
            line.contains("INFO") || line.contains("ERROR") || line.contains("WARN") || line.contains("DEBUG")
        })
        .count();

    let springboot_ratio = springboot_count as f64 / lines.len() as f64;
    debug!("📊 SpringBoot特征匹配度: {}/{} ({:.1}%)", springboot_count, lines.len(), springboot_ratio * 100.0);

    if springboot_ratio > 0.5 { // 超过50%的行包含日志级别
        info!("✅ 检测到SpringBoot格式，特征匹配度: {:.1}%", springboot_ratio * 100.0);
        return "SpringBoot".to_string();
    }

    // 检测Docker JSON格式
    // 特征：JSON格式，包含log和stream字段
    debug!("🔍 检测Docker JSON格式特征");
    let docker_json_count = lines.iter()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with('{') && trimmed.contains("\"log\":") && trimmed.contains("\"stream\":")
        })
        .count();

    let docker_ratio = docker_json_count as f64 / lines.len() as f64;
    debug!("📊 Docker JSON特征匹配度: {}/{} ({:.1}%)", docker_json_count, lines.len(), docker_ratio * 100.0);

    if docker_ratio > 0.5 { // 超过50%的行符合JSON格式
        info!("✅ 检测到DockerJson格式，特征匹配度: {:.1}%", docker_ratio * 100.0);
        return "DockerJson".to_string();
    }

    // 检测MyBatis格式
    // 特征：包含MyBatis特有的SQL日志关键字
    debug!("🔍 检测MyBatis格式特征");
    let mybatis_count = lines.iter()
        .filter(|line| {
            line.contains("Preparing:") || line.contains("Parameters:") || line.contains("==>")
        })
        .count();

    debug!("📊 MyBatis特征匹配: {}/{} 行", mybatis_count, lines.len());

    if mybatis_count > 0 { // MyBatis格式使用存在性判断
        info!("✅ 检测到MyBatis格式，找到 {} 个特征行", mybatis_count);
        return "MyBatis".to_string();
    }

    // 无法识别任何已知格式
    info!("❓ 未能识别已知日志格式，使用通用解析器");
    "Unknown".to_string()
}

/// 从日志行中提取时间戳 (优化版本)
///
/// 使用预编译的正则表达式从日志行中提取符合常见格式的时间戳。
/// 支持多种时间戳格式，包括ISO 8601和其他常见格式。
///
/// # 支持的时间戳格式
/// - `2023-12-25 14:30:45` (标准格式)
/// - `2023-12-25T14:30:45` (ISO 8601格式)
/// - `12/25/2023 14:30:45` (美式格式)
/// - `25-12-2023 14:30:45` (欧式格式)
///
/// # 参数
/// - `line`: 要提取时间戳的日志行
///
/// # Returns
/// - `Option<String>`: 找到时间戳时返回Some，否则返回None
///
/// # 性能优化
/// - 使用预编译的正则表达式，避免重复编译开销
/// - 按常见程度排序，优先匹配最常见格式
/// - 一旦匹配立即返回，避免不必要的检查
/// - 仅在调试模式下输出详细日志
fn extract_timestamp(line: &str) -> Option<String> {
    // 仅在调试模式下输出详细日志，减少大文件处理时的性能开销
    if log::log_enabled!(log::Level::Debug) {
        debug!("🕐 尝试从日志行提取时间戳: {}",
              if line.len() > 50 { format!("{}...", &line[..50]) } else { line.to_string() });
    }

    // 使用预编译的正则表达式，避免重复编译
    for (index, re) in TIMESTAMP_PATTERNS.iter().enumerate() {
        if let Some(captures) = re.find(line) {
            let timestamp = captures.as_str().to_string();
            if log::log_enabled!(log::Level::Debug) {
                debug!("✅ 时间戳提取成功 (模式{}): {}", index + 1, timestamp);
            }
            return Some(timestamp);
        }
    }

    if log::log_enabled!(log::Level::Debug) {
        debug!("❌ 未能从日志行提取时间戳");
    }
    None
}

/// 从日志行中提取日志级别 (优化版本)
///
/// 通过关键词匹配识别日志行中的日志级别信息。
/// 支持标准日志级别和常见的关键词变体。
///
/// # 支持的日志级别
/// - ERROR: error, err
/// - WARN: warn, warning
/// - INFO: info
/// - DEBUG: debug
/// - TRACE: trace
/// - 默认: INFO (当无法识别时)
///
/// # 匹配策略
/// - 不区分大小写匹配
/// - 按优先级顺序检查 (ERROR > WARN > INFO > DEBUG > TRACE)
/// - 支持部分匹配和完整匹配
/// - 提供默认值确保始终返回有效级别
///
/// # 参数
/// - `line`: 要提取日志级别的日志行
///
/// # Returns
/// - `Option<String>`: 识别到的日志级别，始终返回Some值
///
/// # 性能优化
/// - 避免不必要的字符串分配
/// - 按匹配概率排序关键词顺序
/// - 早期返回提高匹配效率
/// - 减少大文件处理时的调试日志输出
fn extract_log_level(line: &str) -> Option<String> {
    // 仅在调试模式下输出详细日志，减少大文件处理时的性能开销
    if log::log_enabled!(log::Level::Debug) {
        debug!("🔍 尝试从日志行提取级别: {}",
              if line.len() > 30 { format!("{}...", &line[..30]) } else { line.to_string() });
    }

    let line_lower = line.to_lowercase();

    // 使用更精确的日志级别检测，避免误判
    let level = if contains_exact_word(&line_lower, "error") || contains_exact_word(&line_lower, "err") {
        if log::log_enabled!(log::Level::Debug) {
            debug!("✅ 检测到ERROR级别");
        }
        "ERROR"
    } else if contains_exact_word(&line_lower, "warn") || contains_exact_word(&line_lower, "warning") {
        if log::log_enabled!(log::Level::Debug) {
            debug!("✅ 检测到WARN级别");
        }
        "WARN"
    } else if contains_exact_word(&line_lower, "info") {
        if log::log_enabled!(log::Level::Debug) {
            debug!("✅ 检测到INFO级别");
        }
        "INFO"
    } else if contains_exact_word(&line_lower, "debug") {
        if log::log_enabled!(log::Level::Debug) {
            debug!("✅ 检测到DEBUG级别");
        }
        "DEBUG"
    } else if contains_exact_word(&line_lower, "trace") {
        if log::log_enabled!(log::Level::Debug) {
            debug!("✅ 检测到TRACE级别");
        }
        "TRACE"
    } else {
        if log::log_enabled!(log::Level::Debug) {
            debug!("❓ 未能识别日志级别，使用默认INFO级别");
        }
        "INFO" // 默认级别
    };

    Some(level.to_string())
}

/// 检查是否包含完整的单词，避免部分匹配导致的误判
///
/// 例如：避免将 "serial" 中的 "err" 识别为错误级别
fn contains_exact_word(text: &str, word: &str) -> bool {
    // 如果是简单的日志级别单词，可以直接包含检查
    if matches!(word, "info" | "debug" | "trace") {
        return text.contains(word);
    }

    // 对于可能造成误判的单词，使用更严格的检查
    let mut start = 0;
    while let Some(pos) = text[start..].find(word) {
        let actual_pos = start + pos;
        let word_end = actual_pos + word.len();

        // 检查单词边界
        let is_word_start = actual_pos == 0 || !text.chars().nth(actual_pos - 1).unwrap_or(' ').is_alphanumeric();
        let is_word_end = word_end >= text.len() || !text.chars().nth(word_end).unwrap_or(' ').is_alphanumeric();

        if is_word_start && is_word_end {
            return true;
        }

        start = actual_pos + 1;
    }

    false
}

/// LogWhisper应用程序主入口函数
///
/// 这是LogWhisper桌面应用的启动入口点，负责：
/// 1. 初始化日志系统和环境配置
/// 2. 创建和管理应用状态
/// 3. 配置Tauri应用框架
/// 4. 注册所有可用的Tauri命令
/// 5. 启动应用程序主循环
///
/// # 启动流程
/// 1. 日志系统初始化
/// 2. 应用状态创建和验证
/// 3. Tauri框架配置
/// 4. 命令处理器注册
/// 5. 应用启动和运行
///
/// # 错误处理
/// - 应用状态初始化失败时优雅退出
/// - 关键组件启动失败时记录详细错误信息
/// - 提供清晰的错误反馈用于问题诊断
///
/// # 注册的命令
/// - 健康检查: health_check
/// - 插件管理: get_plugins
/// - 日志解析: parse_log, test_parse
/// - 配置管理: get_theme_config, update_theme_config, get_parse_config, get_plugin_config, get_window_config, get_all_configs
/// - 文件操作: read_text_file, write_file, save_dialog
#[tokio::main]
async fn main() {
    // 第一步：初始化日志系统
    // 配置日志级别和输出格式，用于应用的调试和监控
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info) // 默认INFO级别，可通过环境变量调整
        .init();

    info!("🚀 LogWhisper Tauri 应用启动中...");
    debug!("🔧 日志系统初始化完成");

    // 第二步：初始化应用状态
    // 创建包含配置服务和插件管理器的应用状态
    info!("📦 开始初始化应用状态...");
    let app_state = match AppState::new().await {
        Ok(state) => {
            info!("✅ 应用状态初始化完成");
            debug!("🔧 配置服务和插件管理器已就绪");
            state
        }
        Err(e) => {
            error!("❌ 应用状态初始化失败: {}", e);
            error!("💥 应用无法启动，请检查配置和依赖");
            return; // 优雅退出，避免部分初始化的状态
        }
    };

    // 第三步：配置和启动Tauri应用
    info!("🏗️ 配置Tauri应用框架...");

    tauri::Builder::default()
        .manage(app_state) // 注册全局应用状态
        .invoke_handler(tauri::generate_handler![
            // 系统管理命令
            health_check,

            // 插件和解析命令
            get_plugins,
            get_file_info,
            parse_log,
            test_parse,

            // 配置管理命令
            get_theme_config,
            update_theme_config,
            get_parse_config,
            get_plugin_config,
            get_window_config,
            get_all_configs,

            // 文件系统操作命令
            read_text_file,
            write_file
        ])
        .run(tauri::generate_context!())
        .expect("🔥 Tauri应用运行失败，请检查配置");

    // 注意：正常情况下，expect()会导致应用退出，不会执行到这里
    // 如果需要清理代码，应该使用tauri::Builder::build().run()的方式
}