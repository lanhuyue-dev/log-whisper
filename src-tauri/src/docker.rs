use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;
use log::{info, error, debug};
use serde::{Deserialize, Serialize};
use bollard::Docker;
use bollard::container::{ListContainersOptions, LogsOptions};
use futures_util::StreamExt;
use std::collections::HashMap;

use crate::AppState;
use crate::plugins::LogEntry;
use crate::plugins::analysis::{analyze_log_content, inject_analysis_metadata};

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
}

/// 获取运行中的容器列表
#[tauri::command]
pub async fn get_containers() -> Result<Vec<ContainerInfo>, String> {
    debug!("🐳 获取容器列表");
    
    let docker = Docker::connect_with_local_defaults()
        .map_err(|e| format!("无法连接 Docker Daemon: {}", e))?;

    let options = ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    };

    let containers = docker.list_containers(Some(options)).await
        .map_err(|e| format!("获取容器列表失败: {}", e))?;

    let result = containers.into_iter().map(|c| {
        ContainerInfo {
            id: c.id.unwrap_or_default(),
            name: c.names.unwrap_or_default().first().map(|n| n.trim_start_matches('/').to_string()).unwrap_or_default(),
            image: c.image.unwrap_or_default(),
            state: c.state.unwrap_or_default(),
            status: c.status.unwrap_or_default(),
        }
    }).collect();

    Ok(result)
}

/// 尝试解析 JSON 日志内容
fn try_parse_json_content(content: &str) -> Option<(String, Option<String>, Option<String>, HashMap<String, String>)> {
    if !content.trim_start().starts_with('{') {
        return None;
    }

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        if !json.is_object() {
            return None;
        }

        let mut metadata = HashMap::new();
        let mut level = None;
        let mut timestamp = None;
        let mut message = content.to_string();

        if let Some(l) = json.get("level").or(json.get("severity")).or(json.get("logLevel")) {
            if let Some(s) = l.as_str() {
                level = Some(s.to_uppercase());
            }
        }

        if let Some(t) = json.get("time").or(json.get("timestamp")).or(json.get("ts")).or(json.get("@timestamp")) {
            if let Some(s) = t.as_str() {
                timestamp = Some(s.to_string());
            }
        }

        if let Some(m) = json.get("message").or(json.get("msg")).or(json.get("log")).or(json.get("content")) {
            if let Some(s) = m.as_str() {
                message = s.to_string();
            } else {
                message = m.to_string();
            }
        } else if let Some(log) = json.get("log") {
             if let Some(s) = log.as_str() {
                 message = s.to_string();
             }
        }

        if let Some(obj) = json.as_object() {
            for (k, v) in obj {
                if !["level", "severity", "logLevel", "time", "timestamp", "ts", "@timestamp", "message", "msg", "log", "content"].contains(&k.as_str()) {
                    if let Some(s) = v.as_str() {
                        metadata.insert(k.clone(), s.to_string());
                    } else {
                        metadata.insert(k.clone(), v.to_string());
                    }
                }
            }
        }

        return Some((message, level, timestamp, metadata));
    }

    None
}

/// 简单的文本日志级别提取
fn extract_level_simple(content: &str) -> Option<String> {
    let content_lower = content.to_lowercase();
    if content_lower.contains("error") || content_lower.contains("err]") || content_lower.contains("[err") {
        Some("ERROR".to_string())
    } else if content_lower.contains("warn") {
        Some("WARN".to_string())
    } else if content_lower.contains("info") {
        Some("INFO".to_string())
    } else if content_lower.contains("debug") {
        Some("DEBUG".to_string())
    } else {
        None
    }
}

/// 开始监听 Docker 容器日志
#[tauri::command]
pub async fn start_docker_tail(
    session_id: String,
    container_id: String,
    window: tauri::Window,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("🐳 开始监听容器日志: {} (Session: {})", container_id, session_id);

    // 1. 创建停止信号通道
    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

    // 2. 添加到 SessionManager
    {
        let mut manager = state.session_manager.lock().await;
        manager.add_session(session_id.clone(), tx);
    }

    // 3. 启动异步任务
    let container_id_clone = container_id.clone();
    let session_id_clone = session_id.clone();
    
    tokio::spawn(async move {
        let docker = match Docker::connect_with_local_defaults() {
            Ok(d) => d,
            Err(e) => {
                error!("❌ 无法连接 Docker: {}", e);
                let _ = window.emit("docker-error", format!("无法连接 Docker: {}", e));
                return;
            }
        };

        let container_name = match docker.inspect_container(&container_id_clone, None).await {
            Ok(info) => info.name.unwrap_or_default().trim_start_matches('/').to_string(),
            Err(_) => container_id_clone.clone(),
        };

        let options = LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            tail: "100".to_string(),
            timestamps: true, 
            ..Default::default()
        };

        let mut stream = docker.logs(&container_id_clone, Some(options));

        loop {
            tokio::select! {
                _ = &mut rx => {
                    info!("🛑 Docker Tail 停止: {}", container_id_clone);
                    break;
                }
                item = stream.next() => {
                    match item {
                        Some(Ok(log_output)) => {
                            let raw_content = log_output.to_string();
                            
                            let (docker_ts, content_body) = if let Some(idx) = raw_content.find(' ') {
                                let (ts, body) = raw_content.split_at(idx);
                                if ts.len() > 10 && ts.contains('T') {
                                    (Some(ts.to_string()), body.trim_start().to_string())
                                } else {
                                    (None, raw_content.clone())
                                }
                            } else {
                                (None, raw_content.clone())
                            };

                            let mut entry = LogEntry {
                                line_number: 0,
                                content: content_body.clone(),
                                timestamp: docker_ts,
                                level: None,
                                formatted_content: None,
                                metadata: HashMap::new(),
                                processed_by: vec!["docker".to_string()],
                            };

                            if let Some((msg, lvl, ts, meta)) = try_parse_json_content(&content_body) {
                                entry.formatted_content = Some(msg.clone());
                                if let Some(l) = lvl { entry.level = Some(l); }
                                if let Some(t) = ts { entry.timestamp = Some(t); }
                                entry.metadata.extend(meta);
                                entry.processed_by.push("json_parser".to_string());
                            } else {
                                entry.level = extract_level_simple(&content_body);
                                entry.formatted_content = Some(content_body.clone());
                            }

                            let content_to_analyze = entry.formatted_content.as_deref().unwrap_or(&content_body);
                            let analysis = analyze_log_content(content_to_analyze);
                            inject_analysis_metadata(&mut entry.metadata, analysis);

                            entry.metadata.insert("container".to_string(), container_name.clone());
                            entry.metadata.insert("_session_id".to_string(), session_id_clone.clone());

                            let _ = window.emit("tail-update", vec![entry]);
                        }
                        Some(Err(e)) => {
                            error!("❌ 读取 Docker 流失败: {}", e);
                            break;
                        }
                        None => {
                            info!("✅ Docker 流结束");
                            break;
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

/// 停止监听 Docker
#[tauri::command]
pub async fn stop_docker_tail(
    session_id: String,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    let mut manager = state.session_manager.lock().await;
    manager.stop_session(&session_id);
    Ok(())
}
