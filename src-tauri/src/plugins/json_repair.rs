use regex::Regex;
use serde_json::Value;
use crate::models::{LogEntry, RenderedBlock, BlockMetadata};
use crate::plugins::{LogRenderer, PluginLifecycle, PluginCapabilities, PerformanceRating, MemoryUsageRating};

/// JSON修复插件
pub struct JsonRepairRenderer {
    json_regex: Regex,
    enabled: bool,
}

impl JsonRepairRenderer {
    /// 创建新的JSON修复渲染器
    pub fn new() -> Self {
        Self {
            json_regex: Regex::new(r#"\{(?:[^{}]|"[^"]*")*\}"#).unwrap(),
            enabled: true,
        }
    }
    
    /// 提取JSON字符串
    fn extract_json<'a>(&self, content: &'a str) -> Option<&'a str> {
        self.json_regex.find(content).map(|m| m.as_str())
    }
    
    /// 修复JSON字符串
    fn repair_json(&self, json_str: &str) -> Result<Value, serde_json::Error> {
        let repaired = self.fix_common_issues(json_str);
        serde_json::from_str(&repaired)
    }
    
    /// 修复常见JSON问题
    fn fix_common_issues(&self, json_str: &str) -> String {
        let mut result = json_str.to_string();
        
        // 修复缺少逗号
        result = result.replace(r#"}{"#, r#"},{""#);
        
        // 修复缺少引号
        result = self.fix_missing_quotes(&result);
        
        // 修复未闭合括号
        result = self.fix_unclosed_brackets(&result);
        
        // 修复转义字符
        result = self.fix_escape_characters(&result);
        
        result
    }
    
    /// 修复缺少引号
    fn fix_missing_quotes(&self, json_str: &str) -> String {
        let mut result = json_str.to_string();
        
        // 简单的引号修复逻辑
        // 查找 key: value 模式，如果key没有引号则添加
        let key_pattern = Regex::new(r#"(\w+):"#).unwrap();
        result = key_pattern.replace_all(&result, r#""$1":"#).to_string();
        
        result
    }
    
    /// 修复未闭合括号
    fn fix_unclosed_brackets(&self, json_str: &str) -> String {
        let mut result = json_str.to_string();
        let mut open_braces = 0;
        let mut open_brackets = 0;
        
        for ch in result.chars() {
            match ch {
                '{' => open_braces += 1,
                '}' => open_braces -= 1,
                '[' => open_brackets += 1,
                ']' => open_brackets -= 1,
                _ => {}
            }
        }
        
        // 添加缺失的闭合括号
        for _ in 0..open_braces {
            result.push('}');
        }
        for _ in 0..open_brackets {
            result.push(']');
        }
        
        result
    }
    
    /// 修复转义字符
    fn fix_escape_characters(&self, json_str: &str) -> String {
        let mut result = json_str.to_string();
        
        // 修复常见的转义问题
        result = result.replace(r#"\""#, r#"\""#);
        result = result.replace(r#"\\"#, r#"\"#);
        result = result.replace(r#"\n"#, "\n");
        result = result.replace(r#"\t"#, "\t");
        result = result.replace(r#"\r"#, "\r");
        
        result
    }
    
    /// 格式化JSON
    fn format_json(&self, value: &Value) -> String {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
    }
    
    /// 验证JSON是否有效
    fn is_valid_json(&self, json_str: &str) -> bool {
        serde_json::from_str::<Value>(json_str).is_ok()
    }
}

impl LogRenderer for JsonRepairRenderer {
    fn can_handle(&self, entry: &LogEntry) -> bool {
        if !self.enabled {
            return false;
        }
        
        self.json_regex.is_match(&entry.content)
    }
    
    fn render(&self, entry: &LogEntry) -> Vec<RenderedBlock> {
        let mut blocks = Vec::new();
        
        if let Some(json_str) = self.extract_json(&entry.content) {
            // 检查原始JSON是否有效
            let is_original_valid = self.is_valid_json(json_str);
            
            if is_original_valid {
                // 原始JSON有效，直接格式化
                if let Ok(value) = serde_json::from_str::<Value>(json_str) {
                    let formatted = self.format_json(&value);
                    let block = RenderedBlock::json(
                        format!("json_{}", entry.line_number),
                        json_str.to_string(),
                        formatted,
                    ).with_metadata(BlockMetadata {
                        line_start: entry.line_number,
                        line_end: entry.line_number,
                        char_start: 0,
                        char_end: entry.content.len(),
                        confidence: 1.0,
                    });
                    blocks.push(block);
                }
            } else {
                // 原始JSON无效，尝试修复
                match self.repair_json(json_str) {
                    Ok(value) => {
                        let formatted = self.format_json(&value);
                        let block = RenderedBlock::json(
                            format!("json_repaired_{}", entry.line_number),
                            json_str.to_string(),
                            formatted,
                        ).with_metadata(BlockMetadata {
                            line_start: entry.line_number,
                            line_end: entry.line_number,
                            char_start: 0,
                            char_end: entry.content.len(),
                            confidence: 0.8,
                        });
                        blocks.push(block);
                    }
                    Err(_) => {
                        // 修复失败，显示错误信息
                        let block = RenderedBlock::error(
                            format!("json_error_{}", entry.line_number),
                            format!("JSON解析失败: {}", json_str),
                        ).with_metadata(BlockMetadata {
                            line_start: entry.line_number,
                            line_end: entry.line_number,
                            char_start: 0,
                            char_end: entry.content.len(),
                            confidence: 0.0,
                        });
                        blocks.push(block);
                    }
                }
            }
        }
        
        blocks
    }
    
    fn name(&self) -> &str {
        "JSON Repair"
    }
    
    fn description(&self) -> &str {
        "修复和格式化JSON数据，支持常见JSON语法错误的自动修复"
    }
    
    fn priority(&self) -> u32 {
        20
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl PluginLifecycle for JsonRepairRenderer {
    fn initialize(&mut self) -> Result<(), String> {
        self.json_regex = Regex::new(r#"\{(?:[^{}]|"[^"]*")*\}"#)
            .map_err(|e| format!("Failed to compile JSON regex: {}", e))?;
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), String> {
        Ok(())
    }
}

impl PluginCapabilities for JsonRepairRenderer {
    fn supported_file_types(&self) -> Vec<String> {
        vec!["log".to_string(), "txt".to_string(), "json".to_string()]
    }
    
    fn max_file_size(&self) -> Option<usize> {
        Some(100 * 1024 * 1024) // 100MB
    }
    
    fn performance_rating(&self) -> PerformanceRating {
        PerformanceRating::High
    }
    
    fn memory_usage_rating(&self) -> MemoryUsageRating {
        MemoryUsageRating::Medium
    }
}
