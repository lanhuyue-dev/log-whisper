use regex::Regex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

// 预编译正则
static TRACE_ID_REGEX: Lazy<Regex> = Lazy::new(|| {
    // 匹配常见的 Trace ID 模式
    // 支持: traceId, trace_id, request_id, correlation_id
    // 格式: UUID, 32位 Hex
    Regex::new(r"(?i)(?:trace|request|correlation)[-_]?(?:id)?[^a-zA-Z0-9_-]+([a-f0-9-]{16,36})").unwrap()
});

static DURATION_REGEX: Lazy<Regex> = Lazy::new(|| {
    // 匹配耗时描述
    // 支持: taken 10ms, cost=2.5s, time: 100us
    Regex::new(r"(?i)(?:duration|taken|cost|time)[^a-zA-Z0-9]+(\d+(?:\.\d+)?)\s*(ms|s|us|μs|ns)").unwrap()
});

/// 性能和链路分析结果
pub struct LogAnalysis {
    pub trace_id: Option<String>,
    pub duration_ms: Option<f64>,
}

/// 对日志内容进行深度分析
pub fn analyze_log_content(content: &str) -> LogAnalysis {
    let mut trace_id = None;
    let mut duration_ms = None;

    // 1. 提取 Trace ID
    if let Some(caps) = TRACE_ID_REGEX.captures(content) {
        if let Some(id) = caps.get(1) {
            trace_id = Some(id.as_str().to_string());
        }
    }

    // 2. 提取耗时
    if let Some(caps) = DURATION_REGEX.captures(content) {
        if let Some(val_str) = caps.get(1) {
            if let Ok(val) = val_str.as_str().parse::<f64>() {
                if let Some(unit) = caps.get(2) {
                    let ms = match unit.as_str().to_lowercase().as_str() {
                        "s" => val * 1000.0,
                        "ms" => val,
                        "us" | "μs" => val / 1000.0,
                        "ns" => val / 1_000_000.0,
                        _ => val,
                    };
                    duration_ms = Some(ms);
                }
            }
        }
    }

    LogAnalysis {
        trace_id,
        duration_ms,
    }
}

/// 将分析结果注入到 Metadata
pub fn inject_analysis_metadata(metadata: &mut HashMap<String, String>, analysis: LogAnalysis) {
    if let Some(tid) = analysis.trace_id {
        metadata.insert("_trace_id".to_string(), tid);
    }
    if let Some(ms) = analysis.duration_ms {
        metadata.insert("_duration_ms".to_string(), format!("{:.2}", ms));
    }
}
