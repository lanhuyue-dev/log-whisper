use crate::plugins::trait_def::{LogNavigator, NavigationResult, LogEntry, PluginInfo, PluginType, Priority};
use crate::plugins::trait_def::{LogNavigator, NavigationResult, PluginType, Priority};
use crate::models::LogEntry;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use chrono::{DateTime, Utc, Duration};
use regex::Regex;
use std::sync::Arc;

/// 时间漂移导航器插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeDriftNavigatorConfig {
    /// 是否启用时间轴缩略图
    pub enable_timeline_thumbnail: bool,
    /// 是否启用时间段对比摘要
    pub enable_period_comparison: bool,
    /// 时间轴缩略图分辨率（分钟为单位）
    pub timeline_resolution_minutes: u32,
    /// 对比摘要时间窗口（分钟为单位）
    pub comparison_window_minutes: u32,
    /// 最小日志密度阈值
    pub min_log_density_threshold: f64,
}

impl Default for TimeDriftNavigatorConfig {
    fn default() -> Self {
        Self {
            enable_timeline_thumbnail: true,
            enable_period_comparison: true,
            timeline_resolution_minutes: 1,
            comparison_window_minutes: 5,
            min_log_density_threshold: 0.1,
        }
    }
}

/// 时间轴数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 日志数量
    pub log_count: u32,
    /// 错误日志数量
    pub error_count: u32,
    /// 警告日志数量
    pub warn_count: u32,
    /// 日志密度
    pub density: f64,
    /// 主要关键词
    pub top_keywords: Vec<String>,
}

/// 时间段对比数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodComparison {
    /// 对比时间段开始
    pub period_start: DateTime<Utc>,
    /// 对比时间段结束
    pub period_end: DateTime<Utc>,
    /// 日志总数
    pub total_logs: u32,
    /// 错误率
    pub error_rate: f64,
    /// 平均日志间隔（秒）
    pub avg_log_interval_seconds: f64,
    /// 关键词频率统计
    pub keyword_frequency: HashMap<String, u32>,
    /// 异常指标
    pub anomaly_score: f64,
}

/// 时间漂移导航器插件
pub struct TimeDriftNavigator {
    enabled: bool,
    config: TimeDriftNavigatorConfig,
    keyword_extractor: Arc<Regex>,
}

impl TimeDriftNavigator {
    /// 创建新的时间漂移导航器实例
    pub fn new() -> Self {
        let keyword_extractor = Arc::new(
            Regex::new(r"\b(?:error|exception|fail|timeout|crash|panic|abort|denied|refused|invalid|unauthorized|forbidden|conflict|critical|fatal|warning|alert)\b")
                .expect("Failed to compile keyword regex")
        );

        Self {
            enabled: true,
            config: TimeDriftNavigatorConfig::default(),
            keyword_extractor,
        }
    }

    /// 使用自定义配置创建实例
    pub fn with_config(config: TimeDriftNavigatorConfig) -> Self {
        let mut navigator = Self::new();
        navigator.config = config;
        navigator
    }

    /// 提取日志时间戳
    fn extract_timestamp(&self, entry: &LogEntry) -> Option<DateTime<Utc>> {
        // 优先使用结构化时间戳
        if let Some(ref structured) = entry.structured_data {
            if let Some(timestamp_value) = structured.get("timestamp") {
                if let Some(timestamp_str) = timestamp_value.as_str() {
                    if let Ok(timestamp) = DateTime::parse_from_rfc3339(timestamp_str) {
                        return Some(timestamp.with_timezone(&Utc));
                    }
                }
            }
        }

        // 从原始消息中解析时间戳
        self.parse_timestamp_from_message(&entry.raw_message)
    }

    /// 从消息中解析时间戳
    fn parse_timestamp_from_message(&self, message: &str) -> Option<DateTime<Utc>> {
        // ISO 8601 格式
        let iso_regex = Regex::new(r"(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{3})?(?:Z|[+-]\d{2}:\d{2}))").ok()?;
        if let Some(caps) = iso_regex.captures(message) {
            if let Ok(timestamp) = DateTime::parse_from_rfc3339(&caps[1]) {
                return Some(timestamp.with_timezone(&Utc));
            }
        }

        // 标准日志格式
        let log_regex = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})").ok()?;
        if let Some(caps) = log_regex.captures(message) {
            if let Ok(timestamp) = chrono::NaiveDateTime::parse_from_str(&caps[1], "%Y-%m-%d %H:%M:%S") {
                return Some(DateTime::from_naive_utc_and_offset(timestamp, Utc));
            }
        }

        None
    }

    /// 确定日志级别
    fn determine_log_level(&self, entry: &LogEntry) -> String {
        let message = entry.rendered_message.as_ref().unwrap_or(&entry.raw_message).to_lowercase();
        
        if message.contains("error") || message.contains("exception") || message.contains("fail") {
            "ERROR".to_string()
        } else if message.contains("warn") || message.contains("warning") {
            "WARN".to_string()
        } else if message.contains("info") {
            "INFO".to_string()
        } else if message.contains("debug") {
            "DEBUG".to_string()
        } else {
            "UNKNOWN".to_string()
        }
    }

    /// 提取关键词
    fn extract_keywords(&self, entry: &LogEntry) -> Vec<String> {
        let message = entry.rendered_message.as_ref().unwrap_or(&entry.raw_message);
        
        self.keyword_extractor
            .find_iter(message)
            .map(|m| m.as_str().to_lowercase())
            .collect()
    }

    /// 生成时间轴缩略图
    fn generate_timeline_thumbnail(&self, entries: &[LogEntry]) -> Vec<TimelinePoint> {
        let mut timeline_map: BTreeMap<DateTime<Utc>, TimelinePoint> = BTreeMap::new();
        let resolution = Duration::minutes(self.config.timeline_resolution_minutes as i64);

        for entry in entries {
            if let Some(timestamp) = self.extract_timestamp(entry) {
                // 将时间戳对齐到分辨率网格
                let aligned_time = self.align_to_resolution(timestamp, resolution);
                
                let level = self.determine_log_level(entry);
                let keywords = self.extract_keywords(entry);

                let point = timeline_map.entry(aligned_time).or_insert_with(|| TimelinePoint {
                    timestamp: aligned_time,
                    log_count: 0,
                    error_count: 0,
                    warn_count: 0,
                    density: 0.0,
                    top_keywords: Vec::new(),
                });

                point.log_count += 1;
                match level.as_str() {
                    "ERROR" => point.error_count += 1,
                    "WARN" => point.warn_count += 1,
                    _ => {}
                }

                // 累积关键词
                for keyword in keywords {
                    if !point.top_keywords.contains(&keyword) && point.top_keywords.len() < 5 {
                        point.top_keywords.push(keyword);
                    }
                }
            }
        }

        // 计算密度
        let max_count = timeline_map.values().map(|p| p.log_count).max().unwrap_or(1);
        for point in timeline_map.values_mut() {
            point.density = point.log_count as f64 / max_count as f64;
        }

        timeline_map.into_values().collect()
    }

    /// 将时间戳对齐到分辨率网格
    fn align_to_resolution(&self, timestamp: DateTime<Utc>, resolution: Duration) -> DateTime<Utc> {
        let resolution_secs = resolution.num_seconds();
        let aligned_secs = (timestamp.timestamp() / resolution_secs) * resolution_secs;
        DateTime::from_timestamp(aligned_secs, 0).unwrap_or(timestamp)
    }

    /// 生成时间段对比摘要
    fn generate_period_comparison(&self, entries: &[LogEntry]) -> Vec<PeriodComparison> {
        let window = Duration::minutes(self.config.comparison_window_minutes as i64);
        let mut periods: BTreeMap<DateTime<Utc>, Vec<LogEntry>> = BTreeMap::new();

        // 按时间窗口分组日志
        for entry in entries {
            if let Some(timestamp) = self.extract_timestamp(entry) {
                let period_start = self.align_to_resolution(timestamp, window);
                periods.entry(period_start).or_insert_with(Vec::new).push(entry.clone());
            }
        }

        // 生成每个时间段的对比数据
        periods.into_iter().map(|(period_start, period_entries)| {
            let total_logs = period_entries.len() as u32;
            let error_count = period_entries.iter()
                .filter(|entry| self.determine_log_level(entry) == "ERROR")
                .count() as u32;
            
            let error_rate = if total_logs > 0 {
                error_count as f64 / total_logs as f64
            } else {
                0.0
            };

            // 计算平均日志间隔
            let avg_log_interval_seconds = if period_entries.len() > 1 {
                let timestamps: Vec<_> = period_entries.iter()
                    .filter_map(|entry| self.extract_timestamp(entry))
                    .collect();
                
                if timestamps.len() > 1 {
                    let total_duration = timestamps.last().unwrap().timestamp() - timestamps.first().unwrap().timestamp();
                    total_duration as f64 / (timestamps.len() - 1) as f64
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // 统计关键词频率
            let mut keyword_frequency = HashMap::new();
            for entry in &period_entries {
                for keyword in self.extract_keywords(entry) {
                    *keyword_frequency.entry(keyword).or_insert(0) += 1;
                }
            }

            // 计算异常分数
            let anomaly_score = self.calculate_anomaly_score(error_rate, avg_log_interval_seconds, &keyword_frequency);

            PeriodComparison {
                period_start,
                period_end: period_start + window,
                total_logs,
                error_rate,
                avg_log_interval_seconds,
                keyword_frequency,
                anomaly_score,
            }
        }).collect()
    }

    /// 计算异常分数
    fn calculate_anomaly_score(&self, error_rate: f64, avg_interval: f64, keywords: &HashMap<String, u32>) -> f64 {
        let mut score = 0.0;

        // 错误率异常
        if error_rate > 0.1 {
            score += error_rate * 100.0;
        }

        // 日志间隔异常（过于频繁或过于稀少）
        if avg_interval < 1.0 || avg_interval > 60.0 {
            score += 10.0;
        }

        // 关键词异常
        let critical_keywords = ["error", "exception", "fail", "timeout", "crash", "panic"];
        for keyword in critical_keywords {
            if let Some(&count) = keywords.get(keyword) {
                score += count as f64 * 5.0;
            }
        }

        score.min(100.0) // 最大分数为100
    }
}

impl Default for TimeDriftNavigator {
    fn default() -> Self {
        Self::new()
    }
}

impl LogNavigator for TimeDriftNavigator {
    fn can_navigate(&self, entries: &[LogEntry]) -> bool {
        if !self.enabled {
            return false;
        }

        // 检查是否有足够的日志条目
        if entries.len() < 2 {
            return false;
        }

        // 检查是否有时间戳信息
        entries.iter().any(|entry| self.extract_timestamp(entry).is_some())
    }

    fn navigate(&self, entries: &[LogEntry]) -> Vec<NavigationResult> {
        let mut results = Vec::new();

        if self.config.enable_timeline_thumbnail {
            let timeline = self.generate_timeline_thumbnail(entries);
            let mut timeline_metadata = HashMap::new();
            timeline_metadata.insert("type".to_string(), "timeline_thumbnail".to_string());
            timeline_metadata.insert("resolution_minutes".to_string(), self.config.timeline_resolution_minutes.to_string());
            timeline_metadata.insert("points_count".to_string(), timeline.len().to_string());

            results.push(NavigationResult {
                navigation_type: "timeline_thumbnail".to_string(),
                title: "时间轴缩略图".to_string(),
                description: format!("生成了 {} 个时间点的缩略图，分辨率: {}分钟", timeline.len(), self.config.timeline_resolution_minutes),
                time_range: if !timeline.is_empty() {
                    Some((timeline.first().unwrap().timestamp, timeline.last().unwrap().timestamp))
                } else {
                    None
                },
                navigation_data: serde_json::to_value(timeline).unwrap_or_default(),
                data: serde_json::to_value(timeline_metadata).unwrap_or_default(),
                metadata: timeline_metadata,
                confidence: 0.9,
            });
        }

        if self.config.enable_period_comparison {
            let comparisons = self.generate_period_comparison(entries);
            let mut comparison_metadata = HashMap::new();
            comparison_metadata.insert("type".to_string(), "period_comparison".to_string());
            comparison_metadata.insert("window_minutes".to_string(), self.config.comparison_window_minutes.to_string());
            comparison_metadata.insert("periods_count".to_string(), comparisons.len().to_string());

            // 计算总体统计
            let total_anomaly_score: f64 = comparisons.iter().map(|c| c.anomaly_score).sum();
            let avg_anomaly_score = if !comparisons.is_empty() {
                total_anomaly_score / comparisons.len() as f64
            } else {
                0.0
            };

            comparison_metadata.insert("avg_anomaly_score".to_string(), format!("{:.2}", avg_anomaly_score));

            results.push(NavigationResult {
                navigation_type: "period_comparison".to_string(),
                title: "时间段对比摘要".to_string(),
                description: format!("分析了 {} 个时间段，平均异常分数: {:.2}", comparisons.len(), avg_anomaly_score),
                data: serde_json::to_value(comparisons).unwrap_or_default(),
                metadata: comparison_metadata,
                confidence: 0.85,
            });
        }

        results
    }

    fn name(&self) -> &str {
        "TimeDriftNavigator"
    }

    fn description(&self) -> &str {
        "时间漂移导航器：提供时间轴缩略图和时间段对比摘要，帮助用户快速定位时间异常和日志模式变化"
    }

    fn priority(&self) -> u32 {
        30 // 中等优先级
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::trait_def::LogEntry;

    fn create_test_log_entry(message: &str, timestamp: Option<&str>) -> LogEntry {
        let mut entry = LogEntry {
            raw_message: message.to_string(),
            rendered_message: None,
            line_number: 1,
            file_path: "test.log".to_string(),
            structured_data: None,
        };

        if let Some(ts) = timestamp {
            let mut structured = HashMap::new();
            structured.insert("timestamp".to_string(), serde_json::Value::String(ts.to_string()));
            entry.structured_data = Some(structured);
        }

        entry
    }

    #[test]
    fn test_can_navigate_with_timestamps() {
        let navigator = TimeDriftNavigator::new();
        let entries = vec![
            create_test_log_entry("Info message", Some("2024-01-01T10:00:00Z")),
            create_test_log_entry("Error occurred", Some("2024-01-01T10:01:00Z")),
        ];

        assert!(navigator.can_navigate(&entries));
    }

    #[test]
    fn test_can_navigate_insufficient_entries() {
        let navigator = TimeDriftNavigator::new();
        let entries = vec![
            create_test_log_entry("Single message", Some("2024-01-01T10:00:00Z")),
        ];

        assert!(!navigator.can_navigate(&entries));
    }

    #[test]
    fn test_timeline_generation() {
        let navigator = TimeDriftNavigator::new();
        let entries = vec![
            create_test_log_entry("Info message 1", Some("2024-01-01T10:00:00Z")),
            create_test_log_entry("Error occurred", Some("2024-01-01T10:00:30Z")),
            create_test_log_entry("Warning message", Some("2024-01-01T10:01:00Z")),
        ];

        let results = navigator.navigate(&entries);
        assert!(!results.is_empty());
        
        let timeline_result = results.iter().find(|r| r.navigation_type == "timeline_thumbnail");
        assert!(timeline_result.is_some());
    }

    #[test]
    fn test_period_comparison() {
        let navigator = TimeDriftNavigator::new();
        let entries = vec![
            create_test_log_entry("Info message", Some("2024-01-01T10:00:00Z")),
            create_test_log_entry("Error occurred", Some("2024-01-01T10:02:00Z")),
            create_test_log_entry("Another error", Some("2024-01-01T10:04:00Z")),
        ];

        let results = navigator.navigate(&entries);
        let comparison_result = results.iter().find(|r| r.navigation_type == "period_comparison");
        assert!(comparison_result.is_some());
    }
}