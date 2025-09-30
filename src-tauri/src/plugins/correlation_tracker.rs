use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

use crate::models::{LogEntry, LogLevel};
use crate::plugins::{LogCorrelator, CorrelationResult, PluginCapabilities, PerformanceRating, MemoryUsageRating};

/// 业务关键词配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessKeyword {
    /// 关键词
    pub keyword: String,
    /// 关键词类型
    pub keyword_type: KeywordType,
    /// 权重 (0.0-1.0)
    pub weight: f32,
    /// 是否大小写敏感
    pub case_sensitive: bool,
    /// 关联范围（秒）
    pub correlation_scope_seconds: u64,
}

/// 关键词类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeywordType {
    /// 事务ID
    TransactionId,
    /// 会话ID
    SessionId,
    /// 用户ID
    UserId,
    /// 请求ID
    RequestId,
    /// 业务流程
    BusinessProcess,
    /// 错误代码
    ErrorCode,
    /// 自定义
    Custom(String),
}

/// 关联上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationContext {
    /// 上下文ID
    pub context_id: String,
    /// 业务关键词
    pub business_keyword: String,
    /// 关键词类型
    pub keyword_type: KeywordType,
    /// 相关容器列表
    pub involved_containers: HashSet<String>,
    /// 时间范围
    pub time_range: (DateTime<Utc>, DateTime<Utc>),
    /// 关联强度
    pub correlation_strength: f32,
}

/// 跨容器事件链
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossContainerEventChain {
    /// 链ID
    pub chain_id: String,
    /// 事件序列
    pub event_sequence: Vec<ContainerEvent>,
    /// 总耗时
    pub total_duration_ms: u64,
    /// 成功状态
    pub success: bool,
    /// 错误信息
    pub error_messages: Vec<String>,
}

/// 容器事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerEvent {
    /// 容器ID
    pub container_id: String,
    /// 事件时间
    pub timestamp: DateTime<Utc>,
    /// 事件类型
    pub event_type: EventType,
    /// 事件内容
    pub content: String,
    /// 日志条目引用
    pub log_entry_line: usize,
}

/// 事件类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// 请求开始
    RequestStart,
    /// 请求结束
    RequestEnd,
    /// 服务调用
    ServiceCall,
    /// 数据库操作
    DatabaseOperation,
    /// 错误发生
    ErrorOccurred,
    /// 其他
    Other(String),
}

/// 关联跟踪器插件
pub struct CorrelationTracker {
    enabled: bool,
    config: CorrelationTrackerConfig,
    business_keywords: Vec<BusinessKeyword>,
}

/// 插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationTrackerConfig {
    /// 最大关联距离（秒）
    pub max_correlation_distance_seconds: u64,
    /// 最小关联强度
    pub min_correlation_strength: f32,
    /// 最大并发上下文数
    pub max_concurrent_contexts: usize,
    /// 启用的关键词类型
    pub enabled_keyword_types: Vec<KeywordType>,
    /// 容器标识模式
    pub container_id_patterns: Vec<String>,
}

impl Default for CorrelationTrackerConfig {
    fn default() -> Self {
        Self {
            max_correlation_distance_seconds: 300, // 5分钟
            min_correlation_strength: 0.3,
            max_concurrent_contexts: 1000,
            enabled_keyword_types: vec![
                KeywordType::TransactionId,
                KeywordType::RequestId,
                KeywordType::SessionId,
                KeywordType::UserId,
            ],
            container_id_patterns: vec![
                r"container[_-]id[:\s=]+([a-f0-9]+)".to_string(),
                r"pod[_-]name[:\s=]+([a-zA-Z0-9\-]+)".to_string(),
                r"service[:\s=]+([a-zA-Z0-9\-]+)".to_string(),
            ],
        }
    }
}

impl CorrelationTracker {
    /// 创建新的关联跟踪器
    pub fn new() -> Self {
        let business_keywords = vec![
            BusinessKeyword {
                keyword: "transaction_id".to_string(),
                keyword_type: KeywordType::TransactionId,
                weight: 1.0,
                case_sensitive: false,
                correlation_scope_seconds: 300,
            },
            BusinessKeyword {
                keyword: "request_id".to_string(),
                keyword_type: KeywordType::RequestId,
                weight: 0.9,
                case_sensitive: false,
                correlation_scope_seconds: 120,
            },
            BusinessKeyword {
                keyword: "session_id".to_string(),
                keyword_type: KeywordType::SessionId,
                weight: 0.8,
                case_sensitive: false,
                correlation_scope_seconds: 600,
            },
            BusinessKeyword {
                keyword: "user_id".to_string(),
                keyword_type: KeywordType::UserId,
                weight: 0.7,
                case_sensitive: false,
                correlation_scope_seconds: 1800,
            },
        ];

        Self {
            enabled: true,
            config: CorrelationTrackerConfig::default(),
            business_keywords,
        }
    }

    /// 从日志条目中提取容器ID
    fn extract_container_id(&self, entry: &LogEntry) -> Option<String> {
        use regex::Regex;
        
        for pattern in &self.config.container_id_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(&entry.raw_line) {
                    return captures.get(1).map(|m| m.as_str().to_string());
                }
            }
        }
        
        None
    }

    /// 提取业务关键词值
    fn extract_business_keyword_values(&self, entry: &LogEntry) -> Vec<(BusinessKeyword, String)> {
        let mut extracted = Vec::new();
        
        for keyword_config in &self.business_keywords {
            if let Some(value) = self.extract_keyword_value(entry, keyword_config) {
                extracted.push((keyword_config.clone(), value));
            }
        }
        
        extracted
    }

    /// 提取具体关键词值
    fn extract_keyword_value(&self, entry: &LogEntry, keyword_config: &BusinessKeyword) -> Option<String> {
        use regex::Regex;
        
        let content = if keyword_config.case_sensitive {
            &entry.raw_line
        } else {
            &entry.raw_line.to_lowercase()
        };
        
        let keyword = if keyword_config.case_sensitive {
            &keyword_config.keyword
        } else {
            &keyword_config.keyword.to_lowercase()
        };
        
        // 构建提取模式 - 简化版本
        let simple_pattern = format!("{}.*([a-zA-Z0-9_]+)", regex::escape(keyword));
        
        if let Ok(re) = Regex::new(&simple_pattern) {
            if let Some(captures) = re.captures(content) {
                return captures.get(1).map(|m| m.as_str().to_string());
            }
        }
        
        None
    }

    /// 构建关联上下文
    fn build_correlation_contexts(&self, entries: &[LogEntry]) -> Vec<CorrelationContext> {
        let mut contexts_map: HashMap<String, CorrelationContext> = HashMap::new();
        
        for entry in entries {
            let container_id = self.extract_container_id(entry);
            let keyword_values = self.extract_business_keyword_values(entry);
            
            for (keyword_config, value) in keyword_values {
                let context_key = format!("{}:{}", keyword_config.keyword_type.type_name(), value);
                
                let context = contexts_map.entry(context_key.clone()).or_insert_with(|| {
                    CorrelationContext {
                        context_id: Uuid::new_v4().to_string(),
                        business_keyword: value.clone(),
                        keyword_type: keyword_config.keyword_type.clone(),
                        involved_containers: HashSet::new(),
                        time_range: (entry.timestamp.unwrap_or_else(|| Utc::now()), entry.timestamp.unwrap_or_else(|| Utc::now())),
                        correlation_strength: keyword_config.weight,
                    }
                });
                
                // 更新容器信息
                if let Some(container) = &container_id {
                    context.involved_containers.insert(container.clone());
                }
                
                // 更新时间范围
                if let Some(timestamp) = entry.timestamp {
                    if timestamp < context.time_range.0 {
                        context.time_range.0 = timestamp;
                    }
                    if timestamp > context.time_range.1 {
                        context.time_range.1 = timestamp;
                    }
                }
            }
        }
        
        // 过滤无效上下文
        contexts_map.into_values()
            .filter(|ctx| ctx.involved_containers.len() > 1) // 至少涉及2个容器
            .collect()
    }

    /// 构建跨容器事件链
    fn build_event_chains(&self, entries: &[LogEntry], context: &CorrelationContext) -> Vec<CrossContainerEventChain> {
        let mut chains = Vec::new();
        
        // 找到与该上下文相关的所有日志条目
        let mut related_entries: Vec<&LogEntry> = entries.iter()
            .filter(|entry| {
                // 检查是否包含业务关键词
                self.contains_business_keyword(entry, &context.business_keyword) &&
                // 检查是否在时间范围内
                entry.timestamp.map_or(true, |ts| ts >= context.time_range.0 && ts <= context.time_range.1)
            })
            .collect();
        
        // 按时间排序
        related_entries.sort_by_key(|entry| entry.timestamp);
        
        if related_entries.len() < 2 {
            return chains;
        }
        
        // 构建事件序列
        let mut events = Vec::new();
        for entry in related_entries {
            if let Some(container_id) = self.extract_container_id(entry) {
                let event_type = self.determine_event_type(entry);
                events.push(ContainerEvent {
                    container_id,
                    timestamp: entry.timestamp.unwrap_or_else(|| Utc::now()),
                    event_type,
                    content: entry.content.clone(),
                    log_entry_line: entry.line_number,
                });
            }
        }
        
        if !events.is_empty() {
            let total_duration = if events.len() > 1 {
                (events.last().unwrap().timestamp - events.first().unwrap().timestamp)
                    .num_milliseconds() as u64
            } else {
                0
            };
            
            let success = !events.iter().any(|e| matches!(e.event_type, EventType::ErrorOccurred));
            let error_messages: Vec<String> = events.iter()
                .filter(|e| matches!(e.event_type, EventType::ErrorOccurred))
                .map(|e| e.content.clone())
                .collect();
            
            chains.push(CrossContainerEventChain {
                chain_id: Uuid::new_v4().to_string(),
                event_sequence: events,
                total_duration_ms: total_duration,
                success,
                error_messages,
            });
        }
        
        chains
    }

    /// 检查日志条目是否包含业务关键词
    fn contains_business_keyword(&self, entry: &LogEntry, keyword_value: &str) -> bool {
        entry.raw_line.contains(keyword_value)
    }

    /// 确定事件类型
    fn determine_event_type(&self, entry: &LogEntry) -> EventType {
        let content_lower = entry.raw_line.to_lowercase();
        
        if content_lower.contains("request") && content_lower.contains("start") {
            EventType::RequestStart
        } else if content_lower.contains("request") && (content_lower.contains("end") || content_lower.contains("complete")) {
            EventType::RequestEnd
        } else if content_lower.contains("call") || content_lower.contains("invoke") {
            EventType::ServiceCall
        } else if content_lower.contains("sql") || content_lower.contains("database") || content_lower.contains("query") {
            EventType::DatabaseOperation
        } else if entry.is_error() {
            EventType::ErrorOccurred
        } else {
            EventType::Other("general".to_string())
        }
    }

    /// 计算关联强度
    fn calculate_correlation_strength(&self, context: &CorrelationContext, chains: &[CrossContainerEventChain]) -> f32 {
        let base_strength = context.correlation_strength;
        
        // 根据涉及的容器数量调整强度
        let container_factor = (context.involved_containers.len() as f32).ln() / 10.0;
        
        // 根据事件链的完整性调整强度
        let chain_factor = if chains.is_empty() {
            0.0
        } else {
            let avg_chain_length = chains.iter().map(|c| c.event_sequence.len()).sum::<usize>() as f32 / chains.len() as f32;
            (avg_chain_length / 10.0).min(1.0)
        };
        
        // 根据时间跨度调整强度
        let time_span = (context.time_range.1 - context.time_range.0).num_seconds() as f32;
        let time_factor = if time_span > 0.0 {
            (1.0 / (1.0 + time_span / 300.0)).max(0.1) // 5分钟内的事件关联性更强
        } else {
            1.0
        };
        
        (base_strength + container_factor + chain_factor) * time_factor
    }
}

impl Default for CorrelationTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl LogCorrelator for CorrelationTracker {
    fn can_correlate(&self, entries: &[LogEntry]) -> bool {
        if !self.enabled || entries.len() < 2 {
            return false;
        }
        
        // 检查是否包含业务关键词
        let has_business_keywords = entries.iter().any(|entry| {
            self.business_keywords.iter().any(|keyword| {
                let content = if keyword.case_sensitive {
                    &entry.raw_line
                } else {
                    &entry.raw_line.to_lowercase()
                };
                content.contains(&keyword.keyword.to_lowercase())
            })
        });
        
        // 检查是否包含容器标识
        let has_container_info = entries.iter().any(|entry| {
            self.extract_container_id(entry).is_some()
        });
        
        has_business_keywords && has_container_info
    }

    fn correlate(&self, entries: &[LogEntry]) -> Vec<CorrelationResult> {
        let mut results = Vec::new();
        
        // 构建关联上下文
        let contexts = self.build_correlation_contexts(entries);
        
        for context in contexts {
            // 为每个上下文构建事件链
            let event_chains = self.build_event_chains(entries, &context);
            
            // 计算关联强度
            let final_strength = self.calculate_correlation_strength(&context, &event_chains);
            
            if final_strength >= self.config.min_correlation_strength {
                // 收集相关的日志条目
                let related_entries: Vec<LogEntry> = entries.iter()
                    .filter(|entry| self.contains_business_keyword(entry, &context.business_keyword))
                    .cloned()
                    .collect();
                
                let correlation_type = format!("business_keyword_{}", context.keyword_type.type_name());
                
                results.push(CorrelationResult {
                    correlation_id: context.context_id,
                    related_entries,
                    correlation_strength: final_strength,
                    correlation_type,
                });
            }
        }
        
        results
    }

    fn name(&self) -> &str {
        "CorrelationTracker"
    }

    fn description(&self) -> &str {
        "基于业务关键词的跨容器日志聚合关联器"
    }

    fn priority(&self) -> u32 {
        30
    }

    fn supported_correlation_types(&self) -> Vec<String> {
        vec![
            "business_keyword_transaction_id".to_string(),
            "business_keyword_request_id".to_string(),
            "business_keyword_session_id".to_string(),
            "business_keyword_user_id".to_string(),
            "cross_container_event_chain".to_string(),
        ]
    }

    fn max_correlation_distance_seconds(&self) -> u64 {
        self.config.max_correlation_distance_seconds
    }
}

impl PluginCapabilities for CorrelationTracker {
    fn supported_file_types(&self) -> Vec<String> {
        vec![
            "*.log".to_string(),
            "*.json".to_string(),
            "docker-*.log".to_string(),
            "kubernetes-*.log".to_string(),
            "microservice-*.log".to_string(),
        ]
    }

    fn max_file_size(&self) -> Option<usize> {
        Some(500 * 1024 * 1024) // 500MB
    }

    fn performance_rating(&self) -> PerformanceRating {
        PerformanceRating::Medium
    }

    fn memory_usage_rating(&self) -> MemoryUsageRating {
        MemoryUsageRating::High // 需要维护关联上下文
    }
}

impl KeywordType {
    /// 获取类型名称
    pub fn type_name(&self) -> &str {
        match self {
            KeywordType::TransactionId => "transaction_id",
            KeywordType::SessionId => "session_id",
            KeywordType::UserId => "user_id",
            KeywordType::RequestId => "request_id",
            KeywordType::BusinessProcess => "business_process",
            KeywordType::ErrorCode => "error_code",
            KeywordType::Custom(name) => name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_can_correlate() {
        let tracker = CorrelationTracker::new();
        
        let entries = vec![
            LogEntry::new(1, "container_id=abc123 transaction_id=txn_001 Processing request".to_string()),
            LogEntry::new(2, "container_id=def456 transaction_id=txn_001 Service call completed".to_string()),
        ];
        
        assert!(tracker.can_correlate(&entries));
    }

    #[test]
    fn test_extract_container_id() {
        let tracker = CorrelationTracker::new();
        
        let test_cases = [
            ("container_id=abc123 Some log message", Some("abc123")),
            ("pod-name=my-service-123 Log message", Some("my-service-123")),
            ("service: user-service Log message", Some("user-service")),
            ("no container info", None),
        ];
        
        for (input, expected) in &test_cases {
            let entry = LogEntry::new(1, input.to_string());
            let result = tracker.extract_container_id(&entry);
            assert_eq!(result.as_deref(), *expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_extract_business_keyword_values() {
        let tracker = CorrelationTracker::new();
        
        let entry = LogEntry::new(1, "transaction_id=txn_001 request_id=req_123 Processing".to_string());
        let extracted = tracker.extract_business_keyword_values(&entry);
        
        assert!(!extracted.is_empty());
        assert!(extracted.iter().any(|(_, value)| value == "txn_001"));
        assert!(extracted.iter().any(|(_, value)| value == "req_123"));
    }

    #[test]
    fn test_build_correlation_contexts() {
        let tracker = CorrelationTracker::new();
        
        let entries = vec![
            LogEntry::new(1, "container_id=abc123 transaction_id=txn_001 Start".to_string()),
            LogEntry::new(2, "container_id=def456 transaction_id=txn_001 Process".to_string()),
            LogEntry::new(3, "container_id=ghi789 transaction_id=txn_001 End".to_string()),
        ];
        
        let contexts = tracker.build_correlation_contexts(&entries);
        
        assert!(!contexts.is_empty());
        let context = &contexts[0];
        assert_eq!(context.business_keyword, "txn_001");
        assert_eq!(context.involved_containers.len(), 3);
    }

    #[test]
    fn test_determine_event_type() {
        let tracker = CorrelationTracker::new();
        
        let test_cases = [
            ("Request start processing", EventType::RequestStart),
            ("Request completed successfully", EventType::RequestEnd),
            ("SQL query executed", EventType::DatabaseOperation),
            ("Service call to user-service", EventType::ServiceCall),
            ("Error occurred: connection failed", EventType::ErrorOccurred),
        ];
        
        for (input, expected_type) in &test_cases {
            let entry = LogEntry::new(1, input.to_string());
            let event_type = tracker.determine_event_type(&entry);
            
            match (event_type, expected_type) {
                (EventType::RequestStart, EventType::RequestStart) => {},
                (EventType::RequestEnd, EventType::RequestEnd) => {},
                (EventType::DatabaseOperation, EventType::DatabaseOperation) => {},
                (EventType::ServiceCall, EventType::ServiceCall) => {},
                (EventType::ErrorOccurred, EventType::ErrorOccurred) => {},
                _ => panic!("Event type mismatch for input: {}", input),
            }
        }
    }
}