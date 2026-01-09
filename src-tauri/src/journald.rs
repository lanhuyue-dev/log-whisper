use std::process::{Command, Stdio};
use tokio::io::{BufReader, AsyncBufReadExt};
use tauri::Manager;
use log::{info, error, debug};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::plugins::LogEntry;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct JournalRecord {
    #[serde(rename = "MESSAGE")]
    message: String,
    #[serde(rename = "__REALTIME_TIMESTAMP")]
    timestamp: Option<String>,
    #[serde(rename = "PRIORITY")]
    priority: Option<String>,
    #[serde(rename = "_SYSTEMD_UNIT")]
    unit: Option<String>,
    #[serde(rename = "_HOSTNAME")]
    hostname: Option<String>,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

/// 开始监听 Systemd Journal
#[tauri::command]
pub async fn start_journal_tail(
    session_id: String,
    unit_filter: Option<String>,
    window: tauri::Window,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("🚀 开始监听 Journald (Session: {}), filter: {:?}", session_id, unit_filter);

    // 1. 创建停止信号通道
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    // 2. 添加到 SessionManager
    {
        let mut manager = state.session_manager.lock().await;
        manager.add_session(session_id.clone(), tx);
    }

    // 3. 启动异步任务
    let session_id_clone = session_id.clone();
    start_async_journal_tail(session_id_clone, unit_filter, window, rx).await;

    Ok(())
}

async fn start_async_journal_tail(
    session_id: String,
    unit_filter: Option<String>,
    window: tauri::Window,
    mut rx: tokio::sync::oneshot::Receiver<()>,
) {
    tokio::spawn(async move {
        use tokio::process::Command;
        
        let mut cmd = Command::new("journalctl");
        cmd.arg("-o").arg("json").arg("-f"); 
        cmd.arg("-n").arg("100");

        if let Some(unit) = unit_filter {
            cmd.arg("-u").arg(unit);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::null()); 

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                error!("❌ 无法启动 journalctl (async): {}", e);
                let _ = window.emit("journal-error", format!("无法启动 journalctl: {}", e));
                return;
            }
        };

        let stdout = child.stdout.take().expect("无法获取 stdout");
        let mut reader = BufReader::new(stdout).lines();

        loop {
            tokio::select! {
                _ = &mut rx => {
                    info!("🛑 Journald Tail 停止 (Session: {})", session_id);
                    let _ = child.kill().await;
                    break;
                }
                line = reader.next_line() => {
                    match line {
                        Ok(Some(content)) => {
                            if let Ok(record) = serde_json::from_str::<JournalRecord>(&content) {
                                let timestamp = record.timestamp.and_then(|ts| {
                                    ts.parse::<i64>().ok().map(|micros| {
                                        let seconds = micros / 1_000_000;
                                        let nanos = (micros % 1_000_000) * 1_000;
                                        if let Some(dt) = chrono::NaiveDateTime::from_timestamp_opt(seconds, nanos as u32) {
                                            let dt: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_utc(dt, chrono::Utc);
                                            dt.to_rfc3339()
                                        } else {
                                            ts
                                        }
                                    })
                                });

                                let level = record.priority.and_then(|p| {
                                    match p.as_str() {
                                        "0" => Some("EMERG"),
                                        "1" => Some("ALERT"),
                                        "2" => Some("CRIT"),
                                        "3" => Some("ERROR"),
                                        "4" => Some("WARN"),
                                        "5" => Some("NOTICE"),
                                        "6" => Some("INFO"),
                                        "7" => Some("DEBUG"),
                                        _ => Some("INFO"),
                                    }
                                }).map(|s| s.to_string());

                                let mut metadata = HashMap::new();
                                metadata.insert("_session_id".to_string(), session_id.clone());

                                let entry = LogEntry {
                                    line_number: 0,
                                    content: record.message.clone(),
                                    timestamp,
                                    level,
                                    formatted_content: Some(format!(
                                        "[{}] {}: {}", 
                                        record.unit.unwrap_or_else(|| "system".to_string()),
                                        record.hostname.unwrap_or_else(|| "localhost".to_string()),
                                        record.message
                                    )),
                                    metadata, 
                                    processed_by: vec!["journald".to_string()],
                                };

                                let _ = window.emit("tail-update", vec![entry]);
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            error!("❌ 读取 journalctl 输出失败: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    });
}

/// 停止监听 Journald
#[tauri::command]
pub async fn stop_journal_tail(
    session_id: String,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    let mut manager = state.session_manager.lock().await;
    manager.stop_session(&session_id);
    Ok(())
}
