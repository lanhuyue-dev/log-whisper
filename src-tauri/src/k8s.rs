use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;
use log::{info, error, debug};
use serde::{Deserialize, Serialize};
use kube::{Client, Api, Config};
use kube::api::{ListParams, LogParams};
use k8s_openapi::api::core::v1::{Pod, Namespace};
use futures_util::{StreamExt, TryStreamExt};
use std::collections::HashMap;

use crate::AppState;
use crate::plugins::LogEntry;
use crate::plugins::analysis::{analyze_log_content, inject_analysis_metadata};

#[derive(Debug, Serialize, Deserialize)]
pub struct K8sPodInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub restart_count: i32,
    pub start_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct K8sNamespaceInfo {
    pub name: String,
    pub status: String,
}

/// 获取 Kubernetes Client
async fn get_client() -> Result<Client, String> {
    let config = Config::infer().await.map_err(|e| format!("无法加载 K8s 配置: {}", e))?;
    let client = Client::try_from(config).map_err(|e| format!("无法创建 K8s Client: {}", e))?;
    Ok(client)
}

/// 列出 Namespaces
#[tauri::command]
pub async fn get_k8s_namespaces() -> Result<Vec<K8sNamespaceInfo>, String> {
    debug!("☸️ 获取 K8s Namespaces");
    let client = get_client().await?;
    let namespaces: Api<Namespace> = Api::all(client);
    
    let ns_list = namespaces.list(&ListParams::default()).await
        .map_err(|e| format!("列出 Namespaces 失败: {}", e))?;

    let result = ns_list.items.into_iter().map(|ns| {
        K8sNamespaceInfo {
            name: ns.metadata.name.unwrap_or_default(),
            status: ns.status.and_then(|s| s.phase).unwrap_or_default(),
        }
    }).collect();

    Ok(result)
}

/// 列出 Pods
#[tauri::command]
pub async fn get_k8s_pods(namespace: String) -> Result<Vec<K8sPodInfo>, String> {
    debug!("☸️ 获取 K8s Pods, namespace: {}", namespace);
    let client = get_client().await?;
    let pods: Api<Pod> = Api::namespaced(client, &namespace);
    
    let pod_list = pods.list(&ListParams::default()).await
        .map_err(|e| format!("列出 Pods 失败: {}", e))?;

    let result = pod_list.items.into_iter().map(|pod| {
        let status = pod.status.as_ref();
        let container_statuses = status.and_then(|s| s.container_statuses.as_ref());
        let restart_count = container_statuses
            .and_then(|cs| cs.first())
            .map(|c| c.restart_count)
            .unwrap_or(0);

        K8sPodInfo {
            name: pod.metadata.name.unwrap_or_default(),
            namespace: pod.metadata.namespace.clone().unwrap_or_default(),
            status: status.and_then(|s| s.phase.clone()).unwrap_or_default(),
            restart_count,
            start_time: status.and_then(|s| s.start_time.as_ref()).map(|t| t.0.to_rfc3339()),
        }
    }).collect();

    Ok(result)
}

/// 开始监听 K8s Pod 日志
#[tauri::command]
pub async fn start_k8s_tail(
    session_id: String,
    namespace: String,
    pod_name: String,
    container_name: Option<String>, 
    window: tauri::Window,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("☸️ 开始监听 Pod 日志: {}/{} (Session: {})", namespace, pod_name, session_id);

    // 1. 创建停止信号通道
    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

    // 2. 添加到 SessionManager
    {
        let mut manager = state.session_manager.lock().await;
        manager.add_session(session_id.clone(), tx);
    }

    // 3. 启动异步任务
    let ns_clone = namespace.clone();
    let pod_clone = pod_name.clone();
    let container_clone = container_name.clone();
    let session_id_clone = session_id.clone();

    tokio::spawn(async move {
        let client = match get_client().await {
            Ok(c) => c,
            Err(e) => {
                error!("❌ 无法连接 K8s: {}", e);
                let _ = window.emit("k8s-error", format!("无法连接 K8s: {}", e));
                return;
            }
        };

        let pods: Api<Pod> = Api::namespaced(client, &ns_clone);
        let mut log_params = LogParams {
            container: container_clone,
            follow: true,
            tail_lines: Some(100),
            timestamps: true, 
            ..LogParams::default()
        };

        let mut stream = match pods.log_stream(&pod_clone, &log_params).await {
            Ok(s) => s,
            Err(e) => {
                error!("❌ 获取日志流失败: {}", e);
                let _ = window.emit("k8s-error", format!("获取日志流失败: {}", e));
                return;
            }
        };

        loop {
            tokio::select! {
                _ = &mut rx => {
                    info!("🛑 K8s Tail 停止: {}/{}", ns_clone, pod_clone);
                    break;
                }
                item = stream.next() => {
                    match item {
                        Some(Ok(bytes)) => {
                            let chunk = String::from_utf8_lossy(&bytes);
                            
                            for line in chunk.lines() {
                                if line.trim().is_empty() { continue; }
                                
                                let (ts, content) = if let Some(idx) = line.find(' ') {
                                    let (t, c) = line.split_at(idx);
                                    if t.len() > 10 && t.contains('T') {
                                        (Some(t.to_string()), c.trim_start().to_string())
                                    } else {
                                        (None, line.to_string())
                                    }
                                } else {
                                    (None, line.to_string())
                                };

                                let mut entry = LogEntry {
                                    line_number: 0,
                                    content: content.clone(),
                                    timestamp: ts,
                                    level: None,
                                    formatted_content: Some(content.clone()),
                                    metadata: HashMap::new(),
                                    processed_by: vec!["k8s".to_string()],
                                };

                                entry.metadata.insert("namespace".to_string(), ns_clone.clone());
                                entry.metadata.insert("pod".to_string(), pod_clone.clone());
                                entry.metadata.insert("_session_id".to_string(), session_id_clone.clone());

                                if let Some(l) = crate::plugins::docker_json::extract_level_from_log(&content) {
                                    entry.level = Some(l);
                                }
                                
                                let analysis = analyze_log_content(&content);
                                inject_analysis_metadata(&mut entry.metadata, analysis);

                                let _ = window.emit("tail-update", vec![entry]);
                            }
                        }
                        Some(Err(e)) => {
                            error!("❌ 读取 K8s 流失败: {}", e);
                            break;
                        }
                        None => {
                            info!("✅ K8s 流结束");
                            break;
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

/// 停止监听 K8s
#[tauri::command]
pub async fn stop_k8s_tail(
    session_id: String,
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    let mut manager = state.session_manager.lock().await;
    manager.stop_session(&session_id);
    Ok(())
}
