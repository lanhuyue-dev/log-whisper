use regex::Regex;
use crate::models::{LogEntry, RenderedBlock, BlockType, BlockMetadata};
use crate::plugins::{LogRenderer, PluginLifecycle, PluginCapabilities, PerformanceRating, MemoryUsageRating};

/// MyBatis SQL解析插件
pub struct MyBatisRenderer {
    preparing_regex: Regex,
    parameters_regex: Regex,
    state: MyBatisState,
    enabled: bool,
}

/// MyBatis解析状态
#[derive(Debug, Clone)]
struct MyBatisState {
    current_sql: Option<String>,
    current_params: Option<Vec<String>>,
    pending_blocks: Vec<RenderedBlock>,
}

impl MyBatisRenderer {
    /// 创建新的MyBatis渲染器
    pub fn new() -> Self {
        Self {
            preparing_regex: Regex::new(r"Preparing:\s*(.+)").unwrap(),
            parameters_regex: Regex::new(r"Parameters:\s*(.+)").unwrap(),
            state: MyBatisState {
                current_sql: None,
                current_params: None,
                pending_blocks: Vec::new(),
            },
            enabled: true,
        }
    }
    
    /// 处理Preparing语句
    fn handle_preparing(&mut self, entry: &LogEntry) -> Vec<RenderedBlock> {
        if let Some(captures) = self.preparing_regex.captures(&entry.content) {
            if let Some(sql) = captures.get(1) {
                let sql_str = sql.as_str().trim();
                self.state.current_sql = Some(sql_str.to_string());
                
                // 创建SQL块
                let block = RenderedBlock::sql(
                    format!("sql_{}", entry.line_number),
                    sql_str.to_string(),
                    self.format_sql(sql_str),
                ).with_metadata(BlockMetadata {
                    line_start: entry.line_number,
                    line_end: entry.line_number,
                    char_start: 0,
                    char_end: entry.content.len(),
                    confidence: 0.9,
                });
                
                return vec![block];
            }
        }
        vec![]
    }
    
    /// 处理Parameters语句
    fn handle_parameters(&mut self, entry: &LogEntry) -> Vec<RenderedBlock> {
        if let Some(captures) = self.parameters_regex.captures(&entry.content) {
            if let Some(params) = captures.get(1) {
                let params_str = params.as_str().trim();
                self.state.current_params = Some(self.parse_parameters(params_str));
                
                // 如果有待处理的SQL，合并生成完整的SQL
                if let Some(sql) = &self.state.current_sql {
                    let complete_sql = self.merge_sql_with_params(sql, &self.state.current_params.as_ref().unwrap());
                    
                    let block = RenderedBlock::sql(
                        format!("complete_sql_{}", entry.line_number),
                        complete_sql.clone(),
                        self.format_sql(&complete_sql),
                    ).with_metadata(BlockMetadata {
                        line_start: entry.line_number,
                        line_end: entry.line_number,
                        char_start: 0,
                        char_end: entry.content.len(),
                        confidence: 0.95,
                    });
                    
                    // 清理状态
                    self.state.current_sql = None;
                    self.state.current_params = None;
                    
                    return vec![block];
                }
            }
        }
        vec![]
    }
    
    /// 解析参数字符串
    fn parse_parameters(&self, params_str: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut current_param = String::new();
        let mut in_quotes = false;
        let mut quote_char = '"';
        
        for ch in params_str.chars() {
            match ch {
                '"' | '\'' => {
                    if !in_quotes {
                        in_quotes = true;
                        quote_char = ch;
                    } else if ch == quote_char {
                        in_quotes = false;
                        if !current_param.is_empty() {
                            params.push(current_param.clone());
                            current_param.clear();
                        }
                    } else {
                        current_param.push(ch);
                    }
                }
                ',' if !in_quotes => {
                    if !current_param.is_empty() {
                        params.push(current_param.trim().to_string());
                        current_param.clear();
                    }
                }
                _ => {
                    current_param.push(ch);
                }
            }
        }
        
        if !current_param.is_empty() {
            params.push(current_param.trim().to_string());
        }
        
        params
    }
    
    /// 合并SQL和参数
    fn merge_sql_with_params(&self, sql: &str, params: &[String]) -> String {
        let mut result = sql.to_string();
        let mut param_index = 0;
        
        // 替换SQL中的?占位符
        while let Some(pos) = result.find('?') {
            if param_index < params.len() {
                let param = &params[param_index];
                let formatted_param = self.format_parameter(param);
                result.replace_range(pos..pos + 1, &formatted_param);
                param_index += 1;
            } else {
                break;
            }
        }
        
        result
    }
    
    /// 格式化参数
    fn format_parameter(&self, param: &str) -> String {
        // 检查参数类型
        if param.ends_with("(Integer)") || param.ends_with("(Long)") || param.ends_with("(Double)") {
            // 数值类型，去掉类型后缀
            let value = param.split('(').next().unwrap_or(param);
            value.trim().to_string()
        } else if param.ends_with("(String)") {
            // 字符串类型，添加引号
            let value = param.split('(').next().unwrap_or(param);
            format!("'{}'", value.trim())
        } else {
            // 其他类型，尝试智能判断
            if param.parse::<i64>().is_ok() || param.parse::<f64>().is_ok() {
                param.trim().to_string()
            } else {
                format!("'{}'", param.trim())
            }
        }
    }
    
    /// 格式化SQL
    fn format_sql(&self, sql: &str) -> String {
        // 简单的SQL格式化
        let mut formatted = sql.to_string();
        
        // 添加关键字换行
        let keywords = ["SELECT", "FROM", "WHERE", "AND", "OR", "ORDER BY", "GROUP BY", "HAVING"];
        for keyword in &keywords {
            let pattern = format!(r"\b{}\b", keyword);
            let replacement = format!("\n{}", keyword);
            formatted = formatted.replace(&pattern, &replacement);
        }
        
        // 清理多余的空格和换行
        formatted = formatted.replace("  ", " ");
        formatted = formatted.replace("\n ", "\n");
        formatted.trim().to_string()
    }
}

impl LogRenderer for MyBatisRenderer {
    fn can_handle(&self, entry: &LogEntry) -> bool {
        if !self.enabled {
            return false;
        }
        
        self.preparing_regex.is_match(&entry.content) || 
        self.parameters_regex.is_match(&entry.content)
    }
    
    fn render(&self, entry: &LogEntry) -> Vec<RenderedBlock> {
        let mut blocks = Vec::new();
        
        if self.preparing_regex.is_match(&entry.content) {
            // 处理Preparing语句
            if let Some(captures) = self.preparing_regex.captures(&entry.content) {
                if let Some(sql) = captures.get(1) {
                    let sql_str = sql.as_str().trim();
                    let block = RenderedBlock::sql(
                        format!("sql_{}", entry.line_number),
                        sql_str.to_string(),
                        self.format_sql(sql_str),
                    ).with_metadata(BlockMetadata {
                        line_start: entry.line_number,
                        line_end: entry.line_number,
                        char_start: 0,
                        char_end: entry.content.len(),
                        confidence: 0.9,
                    });
                    blocks.push(block);
                }
            }
        } else if self.parameters_regex.is_match(&entry.content) {
            // 处理Parameters语句
            if let Some(captures) = self.parameters_regex.captures(&entry.content) {
                if let Some(params) = captures.get(1) {
                    let params_str = params.as_str().trim();
                    let parsed_params = self.parse_parameters(params_str);
                    
                    let block = RenderedBlock::new(
                        format!("params_{}", entry.line_number),
                        BlockType::Info,
                        "SQL 参数".to_string(),
                        params_str.to_string(),
                        self.format_parameters_display(&parsed_params),
                    ).with_metadata(BlockMetadata {
                        line_start: entry.line_number,
                        line_end: entry.line_number,
                        char_start: 0,
                        char_end: entry.content.len(),
                        confidence: 0.8,
                    });
                    blocks.push(block);
                }
            }
        }
        
        blocks
    }
    
    fn name(&self) -> &str {
        "MyBatis SQL Parser"
    }
    
    fn description(&self) -> &str {
        "解析MyBatis日志中的SQL语句和参数，支持参数还原和SQL格式化"
    }
    
    fn priority(&self) -> u32 {
        10
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl PluginLifecycle for MyBatisRenderer {
    fn initialize(&mut self) -> Result<(), String> {
        // 初始化正则表达式
        self.preparing_regex = Regex::new(r"Preparing:\s*(.+)")
            .map_err(|e| format!("Failed to compile preparing regex: {}", e))?;
        self.parameters_regex = Regex::new(r"Parameters:\s*(.+)")
            .map_err(|e| format!("Failed to compile parameters regex: {}", e))?;
        
        Ok(())
    }
    
    fn cleanup(&mut self) -> Result<(), String> {
        self.state.current_sql = None;
        self.state.current_params = None;
        self.state.pending_blocks.clear();
        Ok(())
    }
}

impl PluginCapabilities for MyBatisRenderer {
    fn supported_file_types(&self) -> Vec<String> {
        vec!["log".to_string(), "txt".to_string()]
    }
    
    fn max_file_size(&self) -> Option<usize> {
        Some(50 * 1024 * 1024) // 50MB
    }
    
    fn performance_rating(&self) -> PerformanceRating {
        PerformanceRating::High
    }
    
    fn memory_usage_rating(&self) -> MemoryUsageRating {
        MemoryUsageRating::Low
    }
}

impl MyBatisRenderer {
    /// 格式化参数显示
    fn format_parameters_display(&self, params: &[String]) -> String {
        if params.is_empty() {
            return "无参数".to_string();
        }
        
        let mut result = String::new();
        for (i, param) in params.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&format!("{}: {}", i + 1, param));
        }
        result
    }
}
