// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::{info, debug, trace};
use env_logger;
use std::sync::Arc;
use log_whisper::tauri::AppState;
use log_whisper::parser::ChunkLoader;
use log_whisper::models::ChunkLoaderConfig;

fn main() {
    // 初始化日志系统
    init_logging();
    
    info!("LogWhisper 应用启动中...");
    
    // 创建应用状态
    let app_state = AppState::new();
    let chunk_loader = app_state.chunk_loader.clone();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .manage(chunk_loader)
        .invoke_handler(tauri::generate_handler![
            log_whisper::tauri::commands::parse_file,
            log_whisper::tauri::commands::get_supported_formats,
            log_whisper::tauri::commands::get_available_plugins,
            log_whisper::tauri::commands::get_file_info,
            log_whisper::tauri::commands::clear_cache,
            log_whisper::tauri::commands::get_cache_stats,
            log_whisper::tauri::commands::write_log,
            log_whisper::tauri::commands::initialize_file_chunks,
            log_whisper::tauri::commands::load_chunks,
            log_whisper::tauri::commands::get_memory_info,
            log_whisper::tauri::commands::cleanup_memory,
            log_whisper::tauri::commands::get_chunk_status,
            log_whisper::tauri::commands::preload_chunks,
            log_whisper::tauri::commands::clear_all_cache,
            log_whisper::tauri::commands::validate_file
        ])
        .setup(|app| {
            info!("LogWhisper 启动完成");
            debug!("应用配置: {:?}", app.config());
            trace!("Tauri 应用实例创建成功");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 初始化日志系统
fn init_logging() {
    // 设置日志级别，开发环境使用debug，生产环境使用info
    let log_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "info"
    };
    
    std::env::set_var("RUST_LOG", log_level);
    
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            use std::io::Write;
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            writeln!(
                buf,
                "{} {} [{}] {}:{} - {}",
                timestamp,
                record.level(),
                record.target(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();
    
    info!("日志系统初始化完成，级别: {}", log_level);
}