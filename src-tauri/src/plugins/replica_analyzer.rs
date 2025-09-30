use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::{LogEntry, LogLevel};
use crate::plugins::{LogAnalyzer, AnalysisResult, PluginCapabilities, PerformanceRating, MemoryUsageRating};

/// 副本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaInfo {
    /// 副本ID
    pub replica_id: String,
    /// 容器ID
    pub container_id: Option<String>,
    /// Pod名称
    pub pod_name: Option<String>,
    /// 服务名称
    pub service_name: Option<String>,
    /// 副本状态
    pub status: ReplicaStatus,
}

/// 副本状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReplicaStatus {
    /// 正常
    Normal,
    /// 异常
    Abnormal,
    /// 未知
    Unknown,
}

/// 副本行为模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaBehaviorPattern {
    /// 模式ID
    pub pattern_id: String,
    /// 模式名称
    pub pattern_name: String,
    /// 错误率
    pub error_rate: f32,
    /// 响应时间模式
    pub response_time_pattern: Vec<f32>,
    /// 日志频率模式
    pub log_frequency_pattern: Vec<u32>,
    /// 关键词频率
    pub keyword_frequencies: HashMap<String, u32>,
}

/// 异常检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionResult {
    /// 异常副本列表
    pub anomalous_replicas: Vec<String>,
    /// 异常类型
    pub anomaly_type: AnomalyType,
    /// 异常严重程度 (0.0-1.0)
    pub severity: f32,
    /// 异常描述
    pub description: String,
    /// 建议操作
    pub recommendations: Vec<String>,
}

/// 异常类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    /// 高错误率
    HighErrorRate,
    /// 性能异常
    PerformanceAnomaly,
    /// 日志模式异常
    LogPatternAnomaly,
    /// 服务不可用
    ServiceUnavailable,
    /// 资源异常
    ResourceAnomaly,
}

/// 副本分析器插件
pub struct ReplicaAnalyzer {
    enabled: bool,
    config: ReplicaAnalyzerConfig,
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaAnalyzerConfig {
    /// 异常阈值
    pub anomaly_threshold: f32,
    /// 最小样本数
    pub min_sample_size: usize,
    /// 分析时间窗口（秒）
    pub time_window_seconds: u64,
    /// 错误率阈值
    pub error_rate_threshold: f32,
    /// 启用的异常检测类型
    pub enabled_anomaly_types: Vec<AnomalyType>,
}

impl Default for ReplicaAnalyzerConfig {
    fn default() -> Self {
        Self {
            anomaly_threshold: 0.7,
            min_sample_size: 10,
            time_window_seconds: 300, // 5分钟
            error_rate_threshold: 0.1, // 10%
            enabled_anomaly_types: vec![
                AnomalyType::HighErrorRate,
                AnomalyType::PerformanceAnomaly,
                AnomalyType::LogPatternAnomaly,
            ],
        }
    }
}

impl ReplicaAnalyzer {
    /// 创建新的副本分析器
    pub fn new() -> Self {
        Self {
            enabled: true,
            config: ReplicaAnalyzerConfig::default(),
        }
    }

    /// 从日志条目中提取副本信息
    fn extract_replica_info(&self, entry: &LogEntry) -> Option<ReplicaInfo> {
        // 尝试从日志内容中提取副本相关信息
        let content = &entry.raw_line;
        
        // 检查是否包含容器相关信息
        if let Some(container_id) = self.extract_container_id(content) {
            let pod_name = self.extract_pod_name(content);
            let service_name = self.extract_service_name(content);
            
            Some(ReplicaInfo {
                replica_id: container_id.clone(),
                container_id: Some(container_id),
                pod_name,
                service_name,
                status: ReplicaStatus::Unknown,
            })
        } else {
            None
        }
    }

    /// 提取容器ID
    fn extract_container_id(&self, content: &str) -> Option<String> {
        // 匹配Docker容器ID模式
        use regex::Regex;
        let patterns = [
            r"container[_-]id[:\s=]+([a-f0-9]{12,64})",
            r"docker[_-]id[:\s=]+([a-f0-9]{12,64})",
            r"[Cc]ontainer:\s*([a-f0-9]{12,64})",
        ];
        
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(content) {
                    return captures.get(1).map(|m| m.as_str().to_string());
                }
            }
        }
        
        None
    }

    /// 提取Pod名称
    fn extract_pod_name(&self, content: &str) -> Option<String> {
        use regex::Regex;
        let patterns = [
            r"pod[_-]name[:\s=]+([a-zA-Z0-9\-]+)",
            r"[Pp]od:\s*([a-zA-Z0-9\-]+)",
            r"kubernetes\.pod\.name[:\s=]+([a-zA-Z0-9\-]+)",
        ];
        
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(content) {
                    return captures.get(1).map(|m| m.as_str().to_string());
                }
            }
        }
        
        None
    }

    /// 提取服务名称
    fn extract_service_name(&self, content: &str) -> Option<String> {
        use regex::Regex;
        let patterns = [
            r"service[_-]name[:\s=]+([a-zA-Z0-9\-]+)",
            r"[Ss]ervice:\s*([a-zA-Z0-9\-]+)",
            r"app[:\s=]+([a-zA-Z0-9\-]+)",
        ];
        
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(content) {
                    return captures.get(1).map(|m| m.as_str().to_string());
                }
            }
        }
        
        None
    }

    /// 分析副本行为模式
    fn analyze_replica_behavior(&self, entries: &[LogEntry], replica_info: &ReplicaInfo) -> ReplicaBehaviorPattern {
        let mut error_count = 0;
        let mut total_count = 0;
        let mut keyword_frequencies = HashMap::new();
        let mut log_frequency_pattern = Vec::new();
        
        // 计算错误率
        for entry in entries {
            if self.belongs_to_replica(entry, replica_info) {
                total_count += 1;
                if entry.is_error() {
                    error_count += 1;
                }
                
                // 统计关键词频率
                self.count_keywords(entry, &mut keyword_frequencies);
            }
        }
        
        let error_rate = if total_count > 0 {
            error_count as f32 / total_count as f32
        } else {
            0.0
        };
        
        // 生成时间窗口内的日志频率模式
        log_frequency_pattern = self.calculate_log_frequency_pattern(entries, replica_info);
        
        ReplicaBehaviorPattern {
            pattern_id: Uuid::new_v4().to_string(),
            pattern_name: format!("behavior_{}", replica_info.replica_id),
            error_rate,
            response_time_pattern: vec![], // 需要更复杂的响应时间提取逻辑
            log_frequency_pattern,
            keyword_frequencies,
        }
    }

    /// 判断日志条目是否属于指定副本
    fn belongs_to_replica(&self, entry: &LogEntry, replica_info: &ReplicaInfo) -> bool {
        if let Some(container_id) = &replica_info.container_id {
            return entry.raw_line.contains(container_id);
        }
        
        if let Some(pod_name) = &replica_info.pod_name {
            return entry.raw_line.contains(pod_name);
        }
        
        false
    }

    /// 统计关键词频率
    fn count_keywords(&self, entry: &LogEntry, keyword_frequencies: &mut HashMap<String, u32>) {
        let keywords = ["error", "exception", "warning", "timeout", "failed", "success", "completed"];
        
        for keyword in &keywords {
            if entry.raw_line.to_lowercase().contains(keyword) {
                *keyword_frequencies.entry(keyword.to_string()).or_insert(0) += 1;
            }
        }
    }

    /// 计算日志频率模式
    fn calculate_log_frequency_pattern(&self, entries: &[LogEntry], replica_info: &ReplicaInfo) -> Vec<u32> {
        // 简化实现：按时间窗口统计日志数量
        let mut pattern = vec![0u32; 10]; // 10个时间窗口
        
        for entry in entries {
            if self.belongs_to_replica(entry, replica_info) {
                // 这里需要根据时间戳计算应该放入哪个时间窗口
                // 简化实现：随机分布
                let index = entry.line_number % 10;
                pattern[index] += 1;
            }
        }
        
        pattern
    }

    /// 检测异常副本
    fn detect_anomalies(&self, patterns: &[ReplicaBehaviorPattern]) -> Vec<AnomalyDetectionResult> {
        let mut anomalies = Vec::new();
        
        if patterns.len() < 2 {
            return anomalies; // 需要至少2个副本进行对比
        }
        
        // 计算平均错误率
        let avg_error_rate: f32 = patterns.iter().map(|p| p.error_rate).sum::<f32>() / patterns.len() as f32;
        
        // 检测高错误率异常
        for pattern in patterns {
            if pattern.error_rate > self.config.error_rate_threshold {
                let severity = (pattern.error_rate - avg_error_rate).abs() / avg_error_rate;
                
                if severity > self.config.anomaly_threshold {
                    anomalies.push(AnomalyDetectionResult {
                        anomalous_replicas: vec![pattern.pattern_id.clone()],
                        anomaly_type: AnomalyType::HighErrorRate,
                        severity: severity.min(1.0),
                        description: format!(
                            "副本 {} 错误率异常高: {:.2}% (平均: {:.2}%)",
                            pattern.pattern_id,
                            pattern.error_rate * 100.0,
                            avg_error_rate * 100.0
                        ),
                        recommendations: vec![
                            "检查副本健康状态".to_string(),
                            "查看详细错误日志".to_string(),
                            "考虑重启异常副本".to_string(),
                        ],
                    });
                }
            }
        }
        
        // 检测日志模式异常
        anomalies.extend(self.detect_log_pattern_anomalies(patterns));
        
        anomalies
    }

    /// 检测日志模式异常
    fn detect_log_pattern_anomalies(&self, patterns: &[ReplicaBehaviorPattern]) -> Vec<AnomalyDetectionResult> {
        let mut anomalies = Vec::new();
        
        // 计算平均日志频率
        let avg_frequency: f32 = patterns.iter()
            .map(|p| p.log_frequency_pattern.iter().sum::<u32>() as f32)
            .sum::<f32>() / patterns.len() as f32;
        
        for pattern in patterns {
            let total_frequency: u32 = pattern.log_frequency_pattern.iter().sum();
            let deviation = ((total_frequency as f32 - avg_frequency) / avg_frequency).abs();
            
            if deviation > self.config.anomaly_threshold {
                anomalies.push(AnomalyDetectionResult {
                    anomalous_replicas: vec![pattern.pattern_id.clone()],
                    anomaly_type: AnomalyType::LogPatternAnomaly,
                    severity: deviation.min(1.0),
                    description: format!(
                        "副本 {} 日志模式异常: 频率偏差 {:.1}%",
                        pattern.pattern_id,
                        deviation * 100.0
                    ),
                    recommendations: vec![
                        "检查副本负载分布".to_string(),
                        "分析日志内容模式".to_string(),
                        "监控资源使用情况".to_string(),
                    ],
                });
            }
        }
        
        anomalies
    }
}

impl Default for ReplicaAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LogAnalyzer for ReplicaAnalyzer {
    fn can_analyze(&self, entries: &[LogEntry]) -> bool {
        if !self.enabled || entries.len() < self.config.min_sample_size {
            return false;
        }
        
        // 检查是否包含容器/副本相关的日志
        entries.iter().any(|entry| {
            entry.raw_line.to_lowercase().contains("container") ||
            entry.raw_line.to_lowercase().contains("pod") ||
            entry.raw_line.to_lowercase().contains("replica") ||
            entry.raw_line.to_lowercase().contains("docker")
        })
    }

    fn analyze(&self, entries: &[LogEntry]) -> Vec<AnalysisResult> {
        let start_time = std::time::Instant::now();
        let mut results = Vec::new();
        
        // 提取副本信息
        let mut replica_map: HashMap<String, ReplicaInfo> = HashMap::new();
        for entry in entries {
            if let Some(replica_info) = self.extract_replica_info(entry) {
                replica_map.insert(replica_info.replica_id.clone(), replica_info);
            }
        }
        
        if replica_map.len() < 2 {
            return results; // 需要至少2个副本进行对比分析
        }
        
        // 分析每个副本的行为模式
        let mut patterns = Vec::new();
        for replica_info in replica_map.values() {
            let pattern = self.analyze_replica_behavior(entries, replica_info);
            patterns.push(pattern);
        }
        
        // 检测异常
        let anomalies = self.detect_anomalies(&patterns);
        
        // 生成分析结果
        let analysis_data = serde_json::json!({
            "replica_count": replica_map.len(),
            "patterns": patterns,
            "anomalies": anomalies,
            "summary": {
                "total_replicas": replica_map.len(),
                "anomalous_replicas": anomalies.len(),
                "analysis_time_window": self.config.time_window_seconds
            }
        });
        
        let duration = start_time.elapsed().as_millis() as u64;
        
        results.push(AnalysisResult {
            analysis_type: "replica_analysis".to_string(),
            data: analysis_data,
            confidence: if anomalies.is_empty() { 0.9 } else { 0.95 },
            duration_ms: duration,
        });
        
        results
    }

    fn name(&self) -> &str {
        "ReplicaAnalyzer"
    }

    fn description(&self) -> &str {
        "多副本日志行为对比与异常副本识别分析器"
    }

    fn priority(&self) -> u32 {
        25
    }

    fn supported_analysis_types(&self) -> Vec<String> {
        vec![
            "replica_analysis".to_string(),
            "anomaly_detection".to_string(),
            "behavior_pattern".to_string(),
        ]
    }

    fn min_sample_size(&self) -> usize {
        self.config.min_sample_size
    }
}

impl PluginCapabilities for ReplicaAnalyzer {
    fn supported_file_types(&self) -> Vec<String> {
        vec![
            "*.log".to_string(),
            "*.json".to_string(),
            "docker-*.log".to_string(),
            "kubernetes-*.log".to_string(),
        ]
    }

    fn max_file_size(&self) -> Option<usize> {
        Some(200 * 1024 * 1024) // 200MB
    }

    fn performance_rating(&self) -> PerformanceRating {
        PerformanceRating::Medium // 需要一定计算资源进行模式分析
    }

    fn memory_usage_rating(&self) -> MemoryUsageRating {
        MemoryUsageRating::Medium
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LogLevel;

    #[test]
    fn test_can_analyze_with_container_logs() {
        let analyzer = ReplicaAnalyzer::new();
        let entries = vec![
            LogEntry::new(1, "container_id=abc123 Starting service".to_string()),
            LogEntry::new(2, "container_id=def456 Starting service".to_string()),
            LogEntry::new(3, "container_id=abc123 Service ready".to_string()),
        ];
        
        assert!(analyzer.can_analyze(&entries));
    }

    #[test]
    fn test_extract_container_id() {
        let analyzer = ReplicaAnalyzer::new();
        
        let test_cases = [
            ("container_id=abc123def456", Some("abc123def456")),
            ("Container: 1234567890abcdef", Some("1234567890abcdef")),
            ("docker-id: fedcba0987654321", Some("fedcba0987654321")),
            ("no container info here", None),
        ];
        
        for (input, expected) in &test_cases {
            let result = analyzer.extract_container_id(input);
            assert_eq!(result.as_deref(), *expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_replica_behavior_analysis() {
        let analyzer = ReplicaAnalyzer::new();
        let replica_info = ReplicaInfo {
            replica_id: "test_replica".to_string(),
            container_id: Some("abc123".to_string()),
            pod_name: None,
            service_name: None,
            status: ReplicaStatus::Unknown,
        };
        
        let entries = vec![
            LogEntry::new(1, "container_id=abc123 Starting service".to_string()),
            LogEntry::new(2, "container_id=abc123 Error: Failed to connect".to_string()),
            LogEntry::new(3, "container_id=abc123 Service ready".to_string()),
        ];
        
        let pattern = analyzer.analyze_replica_behavior(&entries, &replica_info);
        
        assert_eq!(pattern.pattern_name, "behavior_test_replica");
        assert!(pattern.error_rate > 0.0);
        assert!(pattern.keyword_frequencies.contains_key("error"));
    }

    #[test]
    fn test_anomaly_detection() {
        let analyzer = ReplicaAnalyzer::new();
        
        let patterns = vec![
            ReplicaBehaviorPattern {
                pattern_id: "replica1".to_string(),
                pattern_name: "behavior_replica1".to_string(),
                error_rate: 0.05, // 正常错误率
                response_time_pattern: vec![],
                log_frequency_pattern: vec![10, 12, 11, 9, 10, 11, 10, 12, 9, 11],
                keyword_frequencies: HashMap::new(),
            },
            ReplicaBehaviorPattern {
                pattern_id: "replica2".to_string(),
                pattern_name: "behavior_replica2".to_string(),
                error_rate: 0.25, // 异常高错误率
                response_time_pattern: vec![],
                log_frequency_pattern: vec![8, 9, 7, 8, 9, 8, 7, 9, 8, 7],
                keyword_frequencies: HashMap::new(),
            },
        ];
        
        let anomalies = analyzer.detect_anomalies(&patterns);
        
        assert!(!anomalies.is_empty());
        assert!(anomalies.iter().any(|a| matches!(a.anomaly_type, AnomalyType::HighErrorRate)));
    }
}