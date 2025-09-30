use crate::plugins::trait_def::{LogFilter, FilterResult, LogEntry, PluginInfo, PluginType, Priority};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use regex::Regex;
use std::sync::Arc;

/// 日志去重器插件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogDeduplicatorConfig {
    /// 是否启用完全重复日志去重
    pub enable_exact_deduplication: bool,
    /// 是否启用相似日志折叠
    pub enable_similarity_folding: bool,
    /// 相似度阈值 (0.0-1.0)
    pub similarity_threshold: f64,
    /// 时间窗口内的去重（秒）
    pub time_window_seconds: u64,
    /// 最大保留重复日志数量
    pub max_duplicate_count: u32,
    /// 是否保留首次和最后一次出现
    pub keep_first_and_last: bool,
    /// 相似日志折叠的最小出现次数
    pub min_occurrences_for_folding: u32,
}

impl Default for LogDeduplicatorConfig {
    fn default() -> Self {
        Self {
            enable_exact_deduplication: true,
            enable_similarity_folding: true,
            similarity_threshold: 0.8,
            time_window_seconds: 300, // 5分钟
            max_duplicate_count: 3,
            keep_first_and_last: true,
            min_occurrences_for_folding: 3,
        }
    }
}

/// 重复日志统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateStats {
    /// 原始日志条目数
    pub original_count: u32,
    /// 去重后条目数
    pub deduplicated_count: u32,
    /// 完全重复的组数
    pub exact_duplicate_groups: u32,
    /// 相似日志折叠的组数
    pub similarity_groups: u32,
    /// 总节省的日志条目数
    pub saved_entries: u32,
}

/// 日志指纹
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct LogFingerprint {
    /// 标准化后的消息模式
    normalized_pattern: String,
    /// 日志级别
    log_level: String,
    /// 源文件或模块
    source: String,
}

/// 重复日志组
#[derive(Debug, Clone)]
struct DuplicateGroup {
    /// 代表性日志条目
    representative: LogEntry,
    /// 所有重复条目
    duplicates: Vec<LogEntry>,
    /// 首次出现时间
    first_occurrence: Option<chrono::DateTime<chrono::Utc>>,
    /// 最后出现时间
    last_occurrence: Option<chrono::DateTime<chrono::Utc>>,
    /// 重复类型
    duplicate_type: DuplicateType,
}

/// 重复类型
#[derive(Debug, Clone, PartialEq)]
enum DuplicateType {
    /// 完全重复
    Exact,
    /// 相似日志
    Similar,
}

/// 日志去重器插件
pub struct LogDeduplicator {
    enabled: bool,
    config: LogDeduplicatorConfig,
    /// 用于提取动态值的正则表达式
    dynamic_patterns: Vec<Arc<Regex>>,
    /// 用于提取时间戳的正则表达式
    timestamp_regex: Arc<Regex>,
}

impl LogDeduplicator {
    /// 创建新的日志去重器实例
    pub fn new() -> Self {
        let dynamic_patterns = vec![
            // 数字
            Arc::new(Regex::new(r"\b\d+\b").expect("Failed to compile number regex")),
            // UUID
            Arc::new(Regex::new(r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b").expect("Failed to compile UUID regex")),
            // IP地址
            Arc::new(Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").expect("Failed to compile IP regex")),
            // 时间戳
            Arc::new(Regex::new(r"\b\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}").expect("Failed to compile timestamp regex")),
            // 内存地址
            Arc::new(Regex::new(r"\b0x[0-9a-fA-F]+\b").expect("Failed to compile memory address regex")),
            // 文件路径
            Arc::new(Regex::new(r"\b[A-Za-z]:[\\\/][^\s]+|\/[^\s]+").expect("Failed to compile path regex")),
        ];

        let timestamp_regex = Arc::new(
            Regex::new(r"(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2})").expect("Failed to compile timestamp extraction regex")
        );

        Self {
            enabled: true,
            config: LogDeduplicatorConfig::default(),
            dynamic_patterns,
            timestamp_regex,
        }
    }

    /// 使用自定义配置创建实例
    pub fn with_config(config: LogDeduplicatorConfig) -> Self {
        let mut deduplicator = Self::new();
        deduplicator.config = config;
        deduplicator
    }

    /// 提取日志时间戳
    fn extract_timestamp(&self, entry: &LogEntry) -> Option<chrono::DateTime<chrono::Utc>> {
        // 优先使用结构化时间戳
        if let Some(ref structured) = entry.structured_data {
            if let Some(timestamp_value) = structured.get("timestamp") {
                if let Some(timestamp_str) = timestamp_value.as_str() {
                    if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
                        return Some(timestamp.with_timezone(&chrono::Utc));
                    }
                }
            }
        }

        // 从原始消息中解析时间戳
        if let Some(caps) = self.timestamp_regex.captures(&entry.raw_message) {
            if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(&caps[1], "%Y-%m-%d %H:%M:%S") {
                return Some(chrono::DateTime::from_naive_utc_and_offset(naive_dt, chrono::Utc));
            }
        }

        None
    }

    /// 生成日志指纹
    fn generate_fingerprint(&self, entry: &LogEntry) -> LogFingerprint {
        let message = entry.rendered_message.as_ref().unwrap_or(&entry.raw_message);
        
        // 标准化消息：移除动态值
        let mut normalized = message.clone();
        for pattern in &self.dynamic_patterns {
            normalized = pattern.replace_all(&normalized, "{VAR}").to_string();
        }

        // 简化空白字符
        normalized = Regex::new(r"\s+").unwrap().replace_all(&normalized, " ").to_string();
        normalized = normalized.trim().to_lowercase();

        // 提取日志级别
        let log_level = self.extract_log_level(message);

        // 提取源信息
        let source = self.extract_source(entry);

        LogFingerprint {
            normalized_pattern: normalized,
            log_level,
            source,
        }
    }

    /// 提取日志级别
    fn extract_log_level(&self, message: &str) -> String {
        let message_lower = message.to_lowercase();
        
        if message_lower.contains("error") || message_lower.contains("exception") {
            "ERROR".to_string()
        } else if message_lower.contains("warn") || message_lower.contains("warning") {
            "WARN".to_string()
        } else if message_lower.contains("info") {
            "INFO".to_string()
        } else if message_lower.contains("debug") {
            "DEBUG".to_string()
        } else {
            "UNKNOWN".to_string()
        }
    }

    /// 提取源信息
    fn extract_source(&self, entry: &LogEntry) -> String {
        // 从文件路径提取
        if let Some(filename) = std::path::Path::new(&entry.file_path).file_name() {
            return filename.to_string_lossy().to_string();
        }

        // 从结构化数据中提取
        if let Some(ref structured) = entry.structured_data {
            if let Some(source) = structured.get("source") {
                if let Some(source_str) = source.as_str() {
                    return source_str.to_string();
                }
            }
        }

        "unknown".to_string()
    }

    /// 计算两个日志条目的相似度
    fn calculate_similarity(&self, entry1: &LogEntry, entry2: &LogEntry) -> f64 {
        let fp1 = self.generate_fingerprint(entry1);
        let fp2 = self.generate_fingerprint(entry2);

        // 如果指纹完全相同，相似度为1.0
        if fp1 == fp2 {
            return 1.0;
        }

        // 如果日志级别或源不同，相似度降低
        if fp1.log_level != fp2.log_level || fp1.source != fp2.source {
            return 0.0;
        }

        // 计算文本相似度（Jaccard相似度）
        self.calculate_jaccard_similarity(&fp1.normalized_pattern, &fp2.normalized_pattern)
    }

    /// 计算Jaccard相似度
    fn calculate_jaccard_similarity(&self, text1: &str, text2: &str) -> f64 {
        let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();

        let intersection_size = words1.intersection(&words2).count();
        let union_size = words1.union(&words2).count();

        if union_size == 0 {
            1.0
        } else {
            intersection_size as f64 / union_size as f64
        }
    }

    /// 执行完全重复日志去重
    fn deduplicate_exact(&self, entries: &[LogEntry]) -> Vec<DuplicateGroup> {
        let mut fingerprint_groups: HashMap<LogFingerprint, DuplicateGroup> = HashMap::new();

        for entry in entries {
            let fingerprint = self.generate_fingerprint(entry);
            let timestamp = self.extract_timestamp(entry);

            match fingerprint_groups.get_mut(&fingerprint) {
                Some(group) => {
                    group.duplicates.push(entry.clone());
                    if let Some(ts) = timestamp {
                        if group.first_occurrence.is_none() || ts < group.first_occurrence.unwrap() {
                            group.first_occurrence = Some(ts);
                        }
                        if group.last_occurrence.is_none() || ts > group.last_occurrence.unwrap() {
                            group.last_occurrence = Some(ts);
                        }
                    }
                }
                None => {
                    let group = DuplicateGroup {
                        representative: entry.clone(),
                        duplicates: vec![entry.clone()],
                        first_occurrence: timestamp,
                        last_occurrence: timestamp,
                        duplicate_type: DuplicateType::Exact,
                    };
                    fingerprint_groups.insert(fingerprint, group);
                }
            }
        }

        fingerprint_groups.into_values().collect()
    }

    /// 执行相似日志折叠
    fn fold_similar_logs(&self, groups: Vec<DuplicateGroup>) -> Vec<DuplicateGroup> {
        let mut result = Vec::new();
        let mut processed = vec![false; groups.len()];

        for i in 0..groups.len() {
            if processed[i] {
                continue;
            }

            let mut similar_group = groups[i].clone();
            processed[i] = true;

            // 寻找相似的组
            for j in (i + 1)..groups.len() {
                if processed[j] {
                    continue;
                }

                let similarity = self.calculate_similarity(
                    &groups[i].representative,
                    &groups[j].representative,
                );

                if similarity >= self.config.similarity_threshold {
                    // 合并相似组
                    similar_group.duplicates.extend(groups[j].duplicates.clone());
                    similar_group.duplicate_type = DuplicateType::Similar;
                    
                    // 更新时间范围
                    if let (Some(other_first), Some(current_first)) = (groups[j].first_occurrence, similar_group.first_occurrence) {
                        if other_first < current_first {
                            similar_group.first_occurrence = groups[j].first_occurrence;
                        }
                    }
                    if let (Some(other_last), Some(current_last)) = (groups[j].last_occurrence, similar_group.last_occurrence) {
                        if other_last > current_last {
                            similar_group.last_occurrence = groups[j].last_occurrence;
                        }
                    }

                    processed[j] = true;
                }
            }

            result.push(similar_group);
        }

        result
    }

    /// 应用折叠策略
    fn apply_folding_strategy(&self, groups: Vec<DuplicateGroup>) -> (Vec<LogEntry>, DuplicateStats) {
        let mut filtered_entries = Vec::new();
        let mut stats = DuplicateStats {
            original_count: 0,
            deduplicated_count: 0,
            exact_duplicate_groups: 0,
            similarity_groups: 0,
            saved_entries: 0,
        };

        for group in groups {
            stats.original_count += group.duplicates.len() as u32;

            if group.duplicates.len() < self.config.min_occurrences_for_folding as usize {
                // 如果重复次数不够，保留所有日志
                filtered_entries.extend(group.duplicates);
                stats.deduplicated_count += group.duplicates.len() as u32;
            } else {
                // 应用折叠策略
                match group.duplicate_type {
                    DuplicateType::Exact => stats.exact_duplicate_groups += 1,
                    DuplicateType::Similar => stats.similarity_groups += 1,
                }

                if self.config.keep_first_and_last && group.duplicates.len() > 2 {
                    // 保留首次和最后一次，以及中间的几个代表
                    filtered_entries.push(group.duplicates[0].clone());
                    
                    let keep_count = (self.config.max_duplicate_count as usize).min(group.duplicates.len());
                    let middle_count = keep_count.saturating_sub(2);
                    
                    if middle_count > 0 {
                        let step = group.duplicates.len() / (middle_count + 1);
                        for i in 1..=middle_count {
                            let idx = (i * step).min(group.duplicates.len() - 2);
                            filtered_entries.push(group.duplicates[idx].clone());
                        }
                    }
                    
                    if group.duplicates.len() > 1 {
                        filtered_entries.push(group.duplicates[group.duplicates.len() - 1].clone());
                    }
                    
                    stats.deduplicated_count += keep_count as u32;
                } else {
                    // 只保留前N个
                    let keep_count = (self.config.max_duplicate_count as usize).min(group.duplicates.len());
                    filtered_entries.extend(group.duplicates.into_iter().take(keep_count));
                    stats.deduplicated_count += keep_count as u32;
                }
            }
        }

        stats.saved_entries = stats.original_count - stats.deduplicated_count;
        (filtered_entries, stats)
    }
}

impl Default for LogDeduplicator {
    fn default() -> Self {
        Self::new()
    }
}

impl LogFilter for LogDeduplicator {
    fn can_filter(&self, entries: &[LogEntry]) -> bool {
        if !self.enabled {
            return false;
        }

        // 需要至少2个日志条目才能进行去重
        entries.len() >= 2
    }

    fn filter(&self, entries: &[LogEntry]) -> Vec<FilterResult> {
        let mut results = Vec::new();

        // 步骤1: 完全重复日志去重
        let exact_groups = if self.config.enable_exact_deduplication {
            self.deduplicate_exact(entries)
        } else {
            entries.iter().map(|entry| DuplicateGroup {
                representative: entry.clone(),
                duplicates: vec![entry.clone()],
                first_occurrence: self.extract_timestamp(entry),
                last_occurrence: self.extract_timestamp(entry),
                duplicate_type: DuplicateType::Exact,
            }).collect()
        };

        // 步骤2: 相似日志折叠
        let folded_groups = if self.config.enable_similarity_folding {
            self.fold_similar_logs(exact_groups)
        } else {
            exact_groups
        };

        // 步骤3: 应用折叠策略
        let (filtered_entries, stats) = self.apply_folding_strategy(folded_groups);

        // 创建过滤结果
        let mut metadata = HashMap::new();
        metadata.insert("original_count".to_string(), stats.original_count.to_string());
        metadata.insert("deduplicated_count".to_string(), stats.deduplicated_count.to_string());
        metadata.insert("exact_duplicate_groups".to_string(), stats.exact_duplicate_groups.to_string());
        metadata.insert("similarity_groups".to_string(), stats.similarity_groups.to_string());
        metadata.insert("saved_entries".to_string(), stats.saved_entries.to_string());
        metadata.insert("compression_ratio".to_string(), 
            format!("{:.2}%", (stats.saved_entries as f64 / stats.original_count as f64) * 100.0));

        results.push(FilterResult {
            filter_type: "deduplication".to_string(),
            title: "日志去重与折叠".to_string(),
            description: format!(
                "原始日志: {} 条，去重后: {} 条，节省: {} 条 ({:.1}%)",
                stats.original_count,
                stats.deduplicated_count,
                stats.saved_entries,
                (stats.saved_entries as f64 / stats.original_count as f64) * 100.0
            ),
            filtered_entries,
            metadata,
            confidence: 0.95,
        });

        results
    }

    fn name(&self) -> &str {
        "LogDeduplicator"
    }

    fn description(&self) -> &str {
        "日志去重器：识别并折叠重复或相似的日志条目，减少日志噪音，提高可读性"
    }

    fn priority(&self) -> u32 {
        Priority::MEDIUM.value() + 10 // 40
    }

    fn plugin_info(&self) -> PluginInfo {
        PluginInfo::new(
            "LogDeduplicator".to_string(),
            "日志去重器：智能识别和折叠重复/相似日志".to_string(),
            "1.0.0".to_string(),
            "LogWhisper Team".to_string(),
            PluginType::Filter,
        )
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::trait_def::LogEntry;

    fn create_test_log_entry(message: &str, line_number: usize) -> LogEntry {
        LogEntry {
            raw_message: message.to_string(),
            rendered_message: None,
            line_number,
            file_path: "test.log".to_string(),
            structured_data: None,
        }
    }

    #[test]
    fn test_exact_deduplication() {
        let deduplicator = LogDeduplicator::new();
        let entries = vec![
            create_test_log_entry("Error: Connection failed", 1),
            create_test_log_entry("Error: Connection failed", 2),
            create_test_log_entry("Info: Process started", 3),
            create_test_log_entry("Error: Connection failed", 4),
        ];

        assert!(deduplicator.can_filter(&entries));
        let results = deduplicator.filter(&entries);
        assert!(!results.is_empty());
        
        let filter_result = &results[0];
        assert!(filter_result.filtered_entries.len() < entries.len());
    }

    #[test]
    fn test_similarity_detection() {
        let deduplicator = LogDeduplicator::new();
        let entry1 = create_test_log_entry("Error: Connection to server 192.168.1.1 failed", 1);
        let entry2 = create_test_log_entry("Error: Connection to server 192.168.1.2 failed", 2);
        
        let similarity = deduplicator.calculate_similarity(&entry1, &entry2);
        assert!(similarity > 0.5);
    }

    #[test]
    fn test_fingerprint_generation() {
        let deduplicator = LogDeduplicator::new();
        let entry1 = create_test_log_entry("Error: User 12345 not found", 1);
        let entry2 = create_test_log_entry("Error: User 67890 not found", 2);
        
        let fp1 = deduplicator.generate_fingerprint(&entry1);
        let fp2 = deduplicator.generate_fingerprint(&entry2);
        
        // 指纹应该相同（数字被标准化）
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_insufficient_entries() {
        let deduplicator = LogDeduplicator::new();
        let entries = vec![create_test_log_entry("Single message", 1)];
        
        assert!(!deduplicator.can_filter(&entries));
    }
}