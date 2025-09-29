use chrono::{DateTime, Utc, Local, TimeZone};

/// 时间工具函数
pub struct TimeUtils;

impl TimeUtils {
    /// 解析时间戳字符串
    pub fn parse_timestamp(timestamp_str: &str) -> Option<DateTime<Utc>> {
        let formats = [
            "%Y-%m-%d %H:%M:%S%.3f",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%dT%H:%M:%S%.3fZ",
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y-%m-%d %H:%M:%S%.3f %z",
            "%Y-%m-%d %H:%M:%S %z",
        ];
        
        for format in &formats {
            if let Ok(dt) = DateTime::parse_from_str(timestamp_str, format) {
                return Some(dt.with_timezone(&Utc));
            }
        }
        
        None
    }
    
    /// 格式化时间戳
    pub fn format_timestamp(dt: &DateTime<Utc>) -> String {
        dt.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
    }
    
    /// 格式化相对时间
    pub fn format_relative_time(dt: &DateTime<Utc>) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(*dt);
        
        if duration.num_seconds() < 60 {
            format!("{}秒前", duration.num_seconds())
        } else if duration.num_minutes() < 60 {
            format!("{}分钟前", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{}小时前", duration.num_hours())
        } else if duration.num_days() < 30 {
            format!("{}天前", duration.num_days())
        } else {
            dt.format("%Y-%m-%d").to_string()
        }
    }
    
    /// 获取当前时间戳
    pub fn current_timestamp() -> DateTime<Utc> {
        Utc::now()
    }
    
    /// 获取本地时间
    pub fn local_time() -> DateTime<Local> {
        Local::now()
    }
    
    /// 计算时间差（毫秒）
    pub fn time_diff_ms(start: &DateTime<Utc>, end: &DateTime<Utc>) -> i64 {
        end.signed_duration_since(*start).num_milliseconds()
    }
    
    /// 格式化持续时间
    pub fn format_duration(duration_ms: i64) -> String {
        if duration_ms < 1000 {
            format!("{}ms", duration_ms)
        } else if duration_ms < 60000 {
            format!("{:.1}s", duration_ms as f64 / 1000.0)
        } else if duration_ms < 3600000 {
            let minutes = duration_ms / 60000;
            let seconds = (duration_ms % 60000) / 1000;
            format!("{}m {}s", minutes, seconds)
        } else {
            let hours = duration_ms / 3600000;
            let minutes = (duration_ms % 3600000) / 60000;
            format!("{}h {}m", hours, minutes)
        }
    }
}
