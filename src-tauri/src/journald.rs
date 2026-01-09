use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;
use log::{info, error, debug};
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::plugins::LogEntry;

// 停止信号发送器
pub type JournalStopper = tokio::sync::oneshot::Sender<()>;

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
    // 允许其他字段
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

/// 开始监听 Systemd Journal
#[tauri::command]
pub async fn start_journal_tail(
    unit_filter: Option<String>,
    window: tauri::Window,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("🚀 开始监听 Journald, filter: {:?}", unit_filter);

    // 1. 如果已有监听任务，先停止
    stop_journal_tail(state.clone()).await?;

    // 2. 创建停止信号通道
    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

    // 3. 保存停止信号发送器
    {
        let mut stopper = state.journal_stopper.lock().await;
        *stopper = Some(tx);
    }

    // 4. 启动异步任务
    tokio::spawn(async move {
        debug!("👀 Journald Tail 任务启动");

        // 构建命令: journalctl -o json -f
        let mut cmd = Command::new("journalctl");
        cmd.arg("-o").arg("json").arg("-f");
        
        if let Some(unit) = unit_filter {
            cmd.arg("-u").arg(unit);
        }

        // 设置 stdout 为管道
        cmd.stdout(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                error!("❌ 无法启动 journalctl: {}", e);
                let _ = window.emit("journal-error", format!("无法启动 journalctl: {}", e));
                return;
            }
        };

        let stdout = child.stdout.take().expect("无法获取 stdout");
        let reader = BufReader::new(stdout);
        
        // 由于 reader.lines() 是阻塞的，我们需要在一个能被 kill 的循环中运行
        // 但标准库的 Command 没有异步接口，我们需要将其转换为异步流，或者使用 tokio::process::Command
        // 这里为了简单，我们使用 tokio::process::Command
        
        // 重新使用 tokio::process::Command
        drop(reader); // 释放同步 reader
        drop(child);  // 杀死同步 child (它应该还没开始读太多) - wait, spawn() started it.
        // 其实这里应该直接用 tokio::process::Command，上面的代码是 std::process::Command
    });
    
    // 重新实现使用 tokio::process
    let unit_filter_clone = unit_filter.clone();
    
    // 这里的逻辑有点乱，我将在下面重写完整的 tokio 实现
    start_async_journal_tail(unit_filter_clone, window, rx).await;

    Ok(())
}

async fn start_async_journal_tail(
    unit_filter: Option<String>,
    window: tauri::Window,
    mut rx: tokio::sync::oneshot::Receiver<()>,
) {
    tokio::spawn(async move {
        use tokio::process::Command;
        use tokio::io::{BufReader, AsyncBufReadExt};
        
        let mut cmd = Command::new("journalctl");
        cmd.arg("-o").arg("json").arg("-f"); // -f for follow, json for output
        
        // 添加 -n 100 以获取最近的日志，避免过多
        cmd.arg("-n").arg("100");

        if let Some(unit) = unit_filter {
            cmd.arg("-u").arg(unit);
        }

        cmd.stdout(Stdio::piped());
        // 忽略 stderr 或重定向到 null
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
                    info!("🛑 Journald Tail 收到停止信号");
                    let _ = child.kill().await;
                    break;
                }
                line = reader.next_line() => {
                    match line {
                        Ok(Some(content)) => {
                            // 解析 JSON
                            if let Ok(record) = serde_json::from_str::<JournalRecord>(&content) {
                                // 转换为 LogEntry
                                let timestamp = record.timestamp.and_then(|ts| {
                                    // systemd 时间戳是微秒，需要转换
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
                                    metadata: std::collections::HashMap::new(), // TODO: add unit/host
                                    processed_by: vec!["journald".to_string()],
                                };

                                let _ = window.emit("tail-update", vec![entry]);
                            }
                        }
                        Ok(None) => {
                            // EOF
                            break;
                        }
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
pub async fn stop_journal_tail(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut stopper = state.journal_stopper.lock().await;
    if let Some(tx) = stopper.take() {
        let _ = tx.send(());
        info!("🛑 发送 Journald 停止信号");
    }
    Ok(())
}
