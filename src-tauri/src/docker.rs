use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;
use log::{info, error, debug};
use serde::{Deserialize, Serialize};
use bollard::Docker;
use bollard::container::{ListContainersOptions, LogsOptions};
use futures_util::StreamExt;

use crate::AppState;
use crate::plugins::LogEntry;

// 停止信号发送器
pub type DockerStopper = tokio::sync::oneshot::Sender<()>;

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
    
    // 连接 Docker (自动检测 socket 路径)
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
            // Docker API 返回的 name 通常带有 '/' 前缀，如 "/my-container"
            name: c.names.unwrap_or_default().first().map(|n| n.trim_start_matches('/').to_string()).unwrap_or_default(),
            image: c.image.unwrap_or_default(),
            state: c.state.unwrap_or_default(),
            status: c.status.unwrap_or_default(),
        }
    }).collect();

    Ok(result)
}

/// 开始监听 Docker 容器日志
#[tauri::command]
pub async fn start_docker_tail(
    container_id: String,
    window: tauri::Window,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("🐳 开始监听容器日志: {}", container_id);

    // 1. 如果已有监听任务，先停止
    stop_docker_tail(state.clone()).await?;

    // 2. 创建停止信号通道
    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

    // 3. 保存停止信号发送器
    {
        let mut stopper = state.docker_stopper.lock().await;
        *stopper = Some(tx);
    }

    // 4. 启动异步任务
    let container_id_clone = container_id.clone();
    
    tokio::spawn(async move {
        let docker = match Docker::connect_with_local_defaults() {
            Ok(d) => d,
            Err(e) => {
                error!("❌ 无法连接 Docker: {}", e);
                let _ = window.emit("docker-error", format!("无法连接 Docker: {}", e));
                return;
            }
        };

        let options = LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            tail: "100".to_string(), // 获取最后100行
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
                            // Bollard 的 LogOutput 包含 Stdout/Stderr/Stdin 等变体
                            // 它们都实现了 Display，可以直接转为 string
                            let content = log_output.to_string();
                            
                            // 构造 LogEntry
                            // TODO: 尝试从 content 中提取时间戳 (Docker 默认可能不带，除非加 timestamps option)
                            // 这里我们简单处理
                            let entry = LogEntry {
                                line_number: 0,
                                content: content.clone(),
                                timestamp: None,
                                level: None, // 可以根据 Stdout/Stderr 判断 INFO/ERROR
                                formatted_content: Some(content),
                                metadata: std::collections::HashMap::new(),
                                processed_by: vec!["docker".to_string()],
                            };

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
pub async fn stop_docker_tail(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut stopper = state.docker_stopper.lock().await;
    if let Some(tx) = stopper.take() {
        let _ = tx.send(());
        info!("🛑 发送 Docker 停止信号");
    }
    Ok(())
}
