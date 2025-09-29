use regex::Regex;

/// 字符串工具函数
pub struct StringUtils;

impl StringUtils {
    /// 清理字符串
    pub fn clean_string(s: &str) -> String {
        s.trim().to_string()
    }
    
    /// 移除多余的空白字符
    pub fn remove_extra_whitespace(s: &str) -> String {
        let re = Regex::new(r"\s+").unwrap();
        re.replace_all(s, " ").trim().to_string()
    }
    
    /// 转义HTML字符
    pub fn escape_html(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
    
    /// 截断字符串
    pub fn truncate_string(s: &str, max_length: usize) -> String {
        if s.len() <= max_length {
            s.to_string()
        } else {
            format!("{}...", &s[..max_length.saturating_sub(3)])
        }
    }
    
    /// 高亮关键词
    pub fn highlight_keywords(text: &str, keywords: &[String]) -> String {
        let mut result = text.to_string();
        
        for keyword in keywords {
            let pattern = format!(r"\b{}\b", regex::escape(keyword));
            let replacement = format!("<mark>{}</mark>", keyword);
            if let Ok(re) = Regex::new(&pattern) {
                result = re.replace_all(&result, replacement.as_str()).to_string();
            }
        }
        
        result
    }
    
    /// 提取JSON字符串
    pub fn extract_json_strings(text: &str) -> Vec<String> {
        let json_pattern = Regex::new(r#"\{(?:[^{}]|"[^"]*")*\}"#).unwrap();
        json_pattern.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }
    
    /// 提取SQL语句
    pub fn extract_sql_statements(text: &str) -> Vec<String> {
        let sql_pattern = Regex::new(r"(?i)(SELECT|INSERT|UPDATE|DELETE|CREATE|DROP|ALTER)\s+.*?;?").unwrap();
        sql_pattern.find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect()
    }
    
    /// 格式化文件大小
    pub fn format_file_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
    
    /// 格式化时间
    pub fn format_duration_ms(millis: u64) -> String {
        if millis < 1000 {
            format!("{}ms", millis)
        } else if millis < 60000 {
            format!("{:.1}s", millis as f64 / 1000.0)
        } else {
            let minutes = millis / 60000;
            let seconds = (millis % 60000) / 1000;
            format!("{}m {}s", minutes, seconds)
        }
    }
}
