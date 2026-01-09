use std::io::{Seek, SeekFrom, BufReader, BufRead};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;
use log::{info, error, debug};
use std::fs::File;
use notify::{Watcher, RecursiveMode, RecommendedWatcher, Config};
use std::collections::HashMap;

use crate::AppState;
use crate::plugins::LogEntry;

/// 开始监听日志文件
#[tauri::command]
pub async fn start_tail(
    session_id: String,
    file_path: String,
    window: tauri::Window,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("🚀 开始监听文件: {} (Session: {})", file_path, session_id);

    // 1. 创建停止信号通道
    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

    // 2. 添加到 SessionManager
    {
        let mut manager = state.session_manager.lock().await;
        manager.add_session(session_id.clone(), tx);
    }

    // 3. 启动异步任务
    let file_path_clone = file_path.clone();
    let session_id_clone = session_id.clone();
    
    tokio::spawn(async move {
        debug!("👀 Tail 任务启动: {}", file_path_clone);
        
        // 打开文件
        let file = match File::open(&file_path_clone) {
            Ok(f) => f,
            Err(e) => {
                error!("❌ 无法打开文件: {}", e);
                let _ = window.emit("tail-error", format!("无法打开文件: {}", e));
                return;
            }
        };

        // 获取初始文件大小，从末尾开始监听
        let mut pos = match file.metadata() {
            Ok(m) => m.len(),
            Err(_) => 0,
        };
        
        debug!("📏 初始文件位置: {}", pos);

        // 创建文件监听器
        let (notify_tx, mut notify_rx) = tokio::sync::mpsc::channel(1);
        
        let mut watcher = match RecommendedWatcher::new(move |res| {
            let _ = notify_tx.blocking_send(res);
        }, Config::default()) {
            Ok(w) => w,
            Err(e) => {
                error!("❌ 创建监听器失败: {}", e);
                return;
            }
        };

        if let Err(e) = watcher.watch(std::path::Path::new(&file_path_clone), RecursiveMode::NonRecursive) {
             error!("❌ 监听文件失败: {}", e);
             return;
        }

        loop {
            tokio::select! {
                // 接收停止信号
                _ = &mut rx => {
                    info!("🛑 Tail 任务停止: {}", file_path_clone);
                    break;
                }
                
                // 接收文件变化事件
                Some(res) = notify_rx.recv() => {
                    match res {
                        Ok(event) => {
                            // 简单的防抖或过滤，这里只关心修改
                            debug!("📄 文件事件: {:?}", event.kind);
                            
                            // 读取新内容
                            match File::open(&file_path_clone) {
                                Ok(mut f) => {
                                    if let Ok(metadata) = f.metadata() {
                                        let current_len = metadata.len();
                                        
                                        // 如果文件变小了（被截断），重置位置
                                        if current_len < pos {
                                            pos = 0;
                                        }
                                        
                                        if current_len > pos {
                                            if let Ok(_) = f.seek(SeekFrom::Start(pos)) {
                                                let reader = BufReader::new(f);
                                                let mut new_lines = Vec::new();
                                                let mut bytes_read = 0;
                                                
                                                for line in reader.lines() {
                                                    if let Ok(l) = line {
                                                        bytes_read += l.len() as u64 + 1; // +1 for newline
                                                        new_lines.push(l);
                                                    }
                                                }
                                                
                                                pos += bytes_read; // 更新位置
                                                
                                                if !new_lines.is_empty() {
                                                    debug!("📥 读取到 {} 行新日志", new_lines.len());
                                                    
                                                    // 解析新行
                                                    let log_entries: Vec<LogEntry> = new_lines.into_iter().enumerate().map(|(i, line)| {
                                                        let mut metadata = HashMap::new();
                                                        metadata.insert("_session_id".to_string(), session_id_clone.clone());
                                                        
                                                        LogEntry {
                                                            line_number: 0,
                                                            content: line.clone(),
                                                            timestamp: None, // TODO: 提取
                                                            level: None, // TODO: 提取
                                                            formatted_content: Some(line),
                                                            metadata,
                                                            processed_by: vec!["tail".to_string()],
                                                        }
                                                    }).collect();
                                                    
                                                    // 发送给前端
                                                    if let Err(e) = window.emit("tail-update", log_entries) {
                                                        error!("❌ 发送日志更新失败: {}", e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => error!("❌ 重新打开文件失败: {}", e),
                            }
                        }
                        Err(e) => error!("❌ 监听错误: {}", e),
                    }
                }
            }
        }
    });

    Ok(())
}

/// 停止监听
#[tauri::command]
pub async fn stop_tail(
    session_id: String,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    let mut manager = state.session_manager.lock().await;
    manager.stop_session(&session_id);
    info!("🛑 发送停止信号 (Session: {})", session_id);
    Ok(())
}
