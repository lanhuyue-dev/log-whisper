//! 日志渲染器插件
//! 
//! 提供各种日志格式的美化渲染功能

use super::{Plugin, LogEntry, PluginError, PluginType};
use std::collections::HashMap;

/// 堆栈跟踪渲染器
/// 专门美化显示Java堆栈跟踪信息
pub struct StackTraceRenderer {
    name: String,
    version: String,
    description: String,
    priority: u32,
}

impl StackTraceRenderer {
    pub fn new() -> Self {
        Self {
            name: "stack_trace_renderer".to_string(),
            version: "1.0.0".to_string(),
            description: "Java堆栈跟踪美化渲染器".to_string(),
            priority: 15,
        }
    }
    
    /// 检查是否为堆栈跟踪内容
    fn is_stack_trace_content(&self, content: &str) -> bool {
        content.contains("at ") ||
        content.contains("Exception:") ||
        content.contains("Error:") ||
        content.contains("Caused by:") ||
        content.contains("Suppressed:")
    }
    
    /// 渲染堆栈跟踪
    fn render_stack_trace(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut rendered_lines = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            let rendered_line = self.render_stack_trace_line(line, i);
            rendered_lines.push(rendered_line);
        }
        
        rendered_lines.join("\n")
    }
    
    /// 渲染单行堆栈跟踪
    fn render_stack_trace_line(&self, line: &str, line_number: usize) -> String {
        let trimmed = line.trim();
        
        // 异常标题行
        if trimmed.contains("Exception:") || trimmed.contains("Error:") {
            return format!("🔴 {}", self.highlight_exception_name(trimmed));
        }
        
        // Caused by 行
        if trimmed.starts_with("Caused by:") {
            return format!("🔗 {}", self.highlight_caused_by(trimmed));
        }
        
        // Suppressed 行
        if trimmed.starts_with("Suppressed:") {
            return format!("📦 {}", self.highlight_suppressed(trimmed));
        }
        
        // 堆栈跟踪行
        if trimmed.starts_with("at ") {
            return format!("  {} {}", self.get_line_indicator(line_number), self.highlight_stack_trace(trimmed));
        }
        
        // 其他行
        format!("  {}", trimmed)
    }
    
    /// 高亮异常名称
    fn highlight_exception_name(&self, line: &str) -> String {
        // 提取异常类名
        if let Some(colon_pos) = line.find(':') {
            let exception_name = &line[..colon_pos];
            let message = &line[colon_pos + 1..];
            format!("<span class='exception-name'>{}</span>: <span class='exception-message'>{}</span>", 
                   exception_name, message.trim())
        } else {
            format!("<span class='exception-name'>{}</span>", line)
        }
    }
    
    /// 高亮 Caused by
    fn highlight_caused_by(&self, line: &str) -> String {
        if let Some(colon_pos) = line.find(':') {
            let caused_by = &line[..colon_pos];
            let message = &line[colon_pos + 1..];
            format!("<span class='caused-by'>{}</span>: <span class='caused-message'>{}</span>", 
                   caused_by, message.trim())
        } else {
            format!("<span class='caused-by'>{}</span>", line)
        }
    }
    
    /// 高亮 Suppressed
    fn highlight_suppressed(&self, line: &str) -> String {
        if let Some(colon_pos) = line.find(':') {
            let suppressed = &line[..colon_pos];
            let message = &line[colon_pos + 1..];
            format!("<span class='suppressed'>{}</span>: <span class='suppressed-message'>{}</span>", 
                   suppressed, message.trim())
        } else {
            format!("<span class='suppressed'>{}</span>", line)
        }
    }
    
    /// 高亮堆栈跟踪
    fn highlight_stack_trace(&self, line: &str) -> String {
        // 解析堆栈跟踪行: at package.Class.method(Class.java:line)
        if let Some(at_pos) = line.find("at ") {
            let after_at = &line[at_pos + 3..];
            
            if let Some(paren_pos) = after_at.find('(') {
                let method_part = &after_at[..paren_pos];
                let file_part = &after_at[paren_pos..];
                
                // 高亮方法名
                let highlighted_method = self.highlight_method_name(method_part);
                
                // 高亮文件名和行号
                let highlighted_file = self.highlight_file_info(file_part);
                
                format!("at {} {}", highlighted_method, highlighted_file)
            } else {
                format!("at <span class='method'>{}</span>", after_at)
            }
        } else {
            line.to_string()
        }
    }
    
    /// 高亮方法名
    fn highlight_method_name(&self, method: &str) -> String {
        // 分离类名和方法名
        if let Some(dot_pos) = method.rfind('.') {
            let class_name = &method[..dot_pos];
            let method_name = &method[dot_pos + 1..];
            format!("<span class='class-name'>{}</span>.<span class='method-name'>{}</span>", 
                   class_name, method_name)
        } else {
            format!("<span class='method'>{}</span>", method)
        }
    }
    
    /// 高亮文件信息
    fn highlight_file_info(&self, file_info: &str) -> String {
        // 解析文件信息: (Class.java:line)
        if file_info.starts_with('(') && file_info.ends_with(')') {
            let content = &file_info[1..file_info.len()-1];
            
            if let Some(colon_pos) = content.find(':') {
                let file_name = &content[..colon_pos];
                let line_number = &content[colon_pos + 1..];
                
                format!("(<span class='file-name'>{}</span>:<span class='line-number'>{}</span>)", 
                       file_name, line_number)
            } else {
                format!("(<span class='file-name'>{}</span>)", content)
            }
        } else {
            file_info.to_string()
        }
    }
    
    /// 获取行指示器
    fn get_line_indicator(&self, line_number: usize) -> String {
        match line_number {
            0 => "┌─".to_string(),
            _ => "├─".to_string(),
        }
    }
}

impl Plugin for StackTraceRenderer {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn priority(&self) -> u32 {
        self.priority
    }
    
    fn plugin_type(&self) -> PluginType {
        PluginType::Renderer
    }
    
    fn can_handle(&self, log_line: &str) -> bool {
        self.is_stack_trace_content(log_line)
    }
    
    fn process(&self, log_entry: &mut LogEntry) -> Result<(), PluginError> {
        if !self.can_handle(&log_entry.content) {
            return Ok(());
        }
        
        // 渲染堆栈跟踪
        let rendered_content = self.render_stack_trace(&log_entry.content);
        log_entry.formatted_content = Some(rendered_content);
        
        // 添加渲染元数据
        log_entry.add_metadata("renderer".to_string(), "stack_trace_renderer".to_string());
        log_entry.add_metadata("rendered".to_string(), "true".to_string());
        
        log_entry.mark_processed(self.name.clone());
        Ok(())
    }
    
    fn initialize(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn cleanup(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// SQL 语句渲染器
/// 美化显示SQL语句
pub struct SqlRenderer {
    name: String,
    version: String,
    description: String,
    priority: u32,
}

impl SqlRenderer {
    pub fn new() -> Self {
        Self {
            name: "sql_renderer".to_string(),
            version: "1.0.0".to_string(),
            description: "SQL语句美化渲染器".to_string(),
            priority: 30,
        }
    }
    
    /// 检查是否为SQL内容
    fn is_sql_content(&self, content: &str) -> bool {
        content.to_uppercase().contains("SELECT") ||
        content.to_uppercase().contains("INSERT") ||
        content.to_uppercase().contains("UPDATE") ||
        content.to_uppercase().contains("DELETE") ||
        content.to_uppercase().contains("CREATE") ||
        content.to_uppercase().contains("ALTER") ||
        content.to_uppercase().contains("DROP")
    }
    
    /// 渲染SQL语句
    fn render_sql(&self, content: &str) -> String {
        let sql_keywords = [
            "SELECT", "FROM", "WHERE", "INSERT", "INTO", "VALUES", "UPDATE", "SET",
            "DELETE", "CREATE", "TABLE", "ALTER", "DROP", "INDEX", "PRIMARY", "KEY",
            "FOREIGN", "REFERENCES", "UNIQUE", "NOT", "NULL", "DEFAULT", "AUTO_INCREMENT",
            "ORDER", "BY", "GROUP", "HAVING", "LIMIT", "OFFSET", "JOIN", "INNER", "LEFT",
            "RIGHT", "OUTER", "ON", "AS", "AND", "OR", "IN", "EXISTS", "BETWEEN", "LIKE",
            "IS", "NULL", "DISTINCT", "COUNT", "SUM", "AVG", "MIN", "MAX", "CASE", "WHEN",
            "THEN", "ELSE", "END", "UNION", "ALL", "INTERSECT", "EXCEPT"
        ];
        
        let mut rendered = content.to_string();
        
        // 高亮SQL关键字
        for keyword in &sql_keywords {
            let pattern = format!(r"\b{}\b", keyword);
            let replacement = format!("<span class='sql-keyword'>{}</span>", keyword);
            rendered = rendered.replace(keyword, &replacement);
        }
        
        // 格式化SQL语句（简单的缩进）
        rendered = self.format_sql_indentation(&rendered);
        
        rendered
    }
    
    /// 格式化SQL缩进
    fn format_sql_indentation(&self, sql: &str) -> String {
        let lines: Vec<&str> = sql.lines().collect();
        let mut formatted_lines = Vec::new();
        let mut indent_level: i32 = 0;
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                formatted_lines.push("".to_string());
                continue;
            }
            
            // 减少缩进
            if trimmed.starts_with(")") || 
               trimmed.starts_with("END") ||
               trimmed.starts_with("ELSE") {
                indent_level = indent_level.saturating_sub(1);
            }
            
            // 添加缩进
            let indent = "  ".repeat(indent_level as usize);
            formatted_lines.push(format!("{}{}", indent, trimmed));
            
            // 增加缩进
            if trimmed.starts_with("SELECT") ||
               trimmed.starts_with("FROM") ||
               trimmed.starts_with("WHERE") ||
               trimmed.starts_with("INSERT") ||
               trimmed.starts_with("UPDATE") ||
               trimmed.starts_with("DELETE") ||
               trimmed.starts_with("CREATE") ||
               trimmed.starts_with("ALTER") ||
               trimmed.starts_with("DROP") ||
               trimmed.starts_with("CASE") ||
               trimmed.starts_with("WHEN") {
                indent_level += 1;
            }
        }
        
        formatted_lines.join("\n")
    }
}

impl Plugin for SqlRenderer {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn priority(&self) -> u32 {
        self.priority
    }
    
    fn plugin_type(&self) -> PluginType {
        PluginType::Renderer
    }
    
    fn can_handle(&self, log_line: &str) -> bool {
        self.is_sql_content(log_line)
    }
    
    fn process(&self, log_entry: &mut LogEntry) -> Result<(), PluginError> {
        if !self.can_handle(&log_entry.content) {
            return Ok(());
        }
        
        // 渲染SQL语句
        let rendered_content = self.render_sql(&log_entry.content);
        log_entry.formatted_content = Some(rendered_content);
        
        // 添加渲染元数据
        log_entry.add_metadata("renderer".to_string(), "sql_renderer".to_string());
        log_entry.add_metadata("rendered".to_string(), "true".to_string());
        
        log_entry.mark_processed(self.name.clone());
        Ok(())
    }
    
    fn initialize(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn cleanup(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// 错误高亮渲染器
/// 根据日志级别进行颜色高亮
pub struct ErrorHighlighter {
    name: String,
    version: String,
    description: String,
    priority: u32,
}

impl ErrorHighlighter {
    pub fn new() -> Self {
        Self {
            name: "error_highlighter".to_string(),
            version: "1.0.0".to_string(),
            description: "错误信息高亮渲染器".to_string(),
            priority: 5,
        }
    }
    
    /// 获取日志级别对应的CSS类
    fn get_level_css_class(&self, level: &str) -> &'static str {
        match level.to_uppercase().as_str() {
            "ERROR" | "FATAL" => "log-error",
            "WARN" | "WARNING" => "log-warn",
            "INFO" => "log-info",
            "DEBUG" => "log-debug",
            "TRACE" => "log-trace",
            _ => "log-default",
        }
    }
    
    /// 渲染日志级别高亮
    fn render_level_highlight(&self, content: &str, level: Option<&str>) -> String {
        if let Some(log_level) = level {
            let css_class = self.get_level_css_class(log_level);
            format!("<span class='{}'>{}</span>", css_class, content)
        } else {
            content.to_string()
        }
    }
}

impl Plugin for ErrorHighlighter {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn priority(&self) -> u32 {
        self.priority
    }
    
    fn plugin_type(&self) -> PluginType {
        PluginType::Renderer
    }
    
    fn can_handle(&self, log_line: &str) -> bool {
        // 所有日志都可以进行级别高亮
        true
    }
    
    fn process(&self, log_entry: &mut LogEntry) -> Result<(), PluginError> {
        // 渲染级别高亮
        let highlighted_content = self.render_level_highlight(
            &log_entry.content, 
            log_entry.level.as_deref()
        );
        
        // 如果还没有格式化内容，使用高亮内容
        if log_entry.formatted_content.is_none() {
            log_entry.formatted_content = Some(highlighted_content);
        }
        
        // 添加渲染元数据
        log_entry.add_metadata("renderer".to_string(), "error_highlighter".to_string());
        log_entry.add_metadata("highlighted".to_string(), "true".to_string());
        
        log_entry.mark_processed(self.name.clone());
        Ok(())
    }
    
    fn initialize(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn cleanup(&self) -> Result<(), PluginError> {
        Ok(())
    }
}
