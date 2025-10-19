/// 具体过滤器实现
///
/// 实现各种具体的日志处理过滤器，用于插件链中。
/// 每个过滤器都实现了PluginFilter trait，可以独立处理特定的日志格式，
/// 并将处理结果传递给链中的下一个过滤器。
///
/// # 支持的过滤器类型
/// - **DockerJsonFilter**: 解析Docker JSON格式日志
/// - **SpringBootFilter**: 解析SpringBoot应用日志
/// - **MyBatisFilter**: 识别和格式化MyBatis SQL日志
/// - **JsonStructureFilter**: 结构化JSON输出格式化
/// - **ContentEnhancerFilter**: 内容增强和格式化
/// - **LevelClassifierFilter**: 日志级别分类和标准化
///
/// # 设计特点
/// - **模块化设计**: 每个过滤器专注于特定功能
/// - **链式处理**: 支持顺序处理和数据传递
/// - **智能判断**: 根据内容特征决定是否处理
/// - **错误恢复**: 提供良好的错误处理机制
/// - **性能优化**: 避免不必要的处理和内存分配

use crate::plugins::chain::{PluginFilter, PluginChainContext};
use crate::plugins::{ParseRequest, LogLine};
use std::collections::HashMap;
use serde_json;
use log::{debug, info, warn};

/// Docker JSON过滤器
///
/// 专门处理Docker容器输出的JSON格式日志。
/// 解析JSON结构，提取log、stream、time等字段，并将处理结果传递给后续过滤器。
///
/// # 处理逻辑
/// 1. 解析JSON格式的日志行
/// 2. 提取log字段作为主要内容
/// 3. 保留stream和time信息到元数据
/// 4. 将提取的内容传递给后续过滤器
///
/// # 链中位置
/// 通常作为链的第一个过滤器，负责将Docker JSON格式转换为纯文本格式。
pub struct DockerJsonFilter;

impl PluginFilter for DockerJsonFilter {
    fn name(&self) -> &str {
        "docker_json"
    }

    fn description(&self) -> &str {
        "Docker JSON日志解析过滤器，解析容器JSON格式日志并提取内容"
    }

    fn priority(&self) -> i32 {
        10 // 高优先级，通常首先执行
    }

    fn should_process(&self, context: &PluginChainContext) -> bool {
        // 如果当前行列表为空，说明这是第一次处理，需要解析原始内容
        if context.current_lines.is_empty() {
            return true;
        }

        // 检查是否还有JSON格式的行需要处理
        context.current_lines.iter().any(|line| {
            line.content.trim_start().starts_with('{') &&
            (line.content.contains("\"log\"") || line.content.contains("\"stream\""))
        })
    }

    fn process(&self, context: &mut PluginChainContext, _request: &ParseRequest) -> Result<(), String> {
        info!("🐳 Docker JSON过滤器开始处理");

        let lines_to_process = if context.current_lines.is_empty() {
            // 第一次处理，从原始内容创建行列表，过滤空行
            context.original_content.lines().enumerate().filter(|(_, line)| !line.trim().is_empty()).map(|(i, line)| {
                LogLine {
                    line_number: i + 1,
                    content: line.to_string(),
                    level: None,
                    timestamp: None,
                    formatted_content: None,
                    metadata: HashMap::new(),
                    processed_by: vec![],
                }
            }).collect()
        } else {
            // 后续处理，使用现有的行列表
            context.current_lines.clone()
        };

        let mut processed_lines = Vec::with_capacity(lines_to_process.len());
        let mut processed_count = 0;

        for mut line in lines_to_process {
            let trimmed = line.content.trim_start();

            if trimmed.starts_with('{') && (trimmed.contains("\"log\"") || trimmed.contains("\"stream\"")) {
                // 尝试解析JSON格式
                match serde_json::from_str::<serde_json::Value>(&line.content) {
                    Ok(json) => {
                        // 提取stream信息
                        if let Some(stream) = json.get("stream").and_then(|v| v.as_str()) {
                            line.metadata.insert("stream".to_string(), stream.to_string());
                        }

                        // 提取时间戳
                        if let Some(time) = json.get("time").and_then(|v| v.as_str()) {
                            line.timestamp = Some(time.to_string());
                        }

                        // 提取log内容作为主要内容，并解析Java GC日志格式
                        if let Some(log_content) = json.get("log").and_then(|v| v.as_str()) {
                            let clean_content = log_content.trim_end_matches('\n');
                            line.content = clean_content.to_string();

                            // 解析Java GC日志格式中的日志级别
                            // 格式: [0.000s][warning][gc] -XX:+PrintGCDetails is deprecated...
                            // 或: [0.002s][info   ][gc,init] CardTable entry size: 512
                            let gc_log_pattern = regex::Regex::new(r"^\[[^\]]+\]\[([^\]]+)\]").unwrap();
                            if let Some(caps) = gc_log_pattern.captures(clean_content) {
                                if let Some(level_str) = caps.get(1) {
                                    let normalized_level = match level_str.as_str().trim() {
                                        "warning" => "WARN".to_string(),
                                        "info" => "INFO".to_string(),
                                        "error" => "ERROR".to_string(),
                                        "debug" => "DEBUG".to_string(),
                                        "trace" => "DEBUG".to_string(),
                                        other => other.to_uppercase(),
                                    };
                                    line.level = Some(normalized_level);
                                }
                            }

                            // 设置格式化内容为清洁的消息内容（去除GC日志前缀）
                            let clean_content_pattern = regex::Regex::new(r"^\[[^\]]+\]\[[^\]]+\]\s*").unwrap();
                            let formatted = clean_content_pattern.replace(clean_content, "").to_string();
                            line.formatted_content = Some(formatted);
                        }

                        // 添加处理标记
                        line.processed_by.push("docker_json_filter".to_string());
                        processed_count += 1;

                        debug!("✅ Docker JSON解析成功: 行{} -> {}", line.line_number, line.content);
                    }
                    Err(e) => {
                        warn!("⚠️ Docker JSON解析失败: 行{} - {}", line.line_number, e);
                        // 解析失败时保留原始内容，但添加错误信息
                        line.metadata.insert("parse_error".to_string(), format!("JSON解析失败: {}", e));
                    }
                }
            }

            processed_lines.push(line);
        }

        context.current_lines = processed_lines;

        // 设置链级别的元数据
        context.set_chain_metadata("docker_json_processed".to_string(), processed_count.to_string());

        info!("🐳 Docker JSON过滤器处理完成，处理了 {} 行", processed_count);
        Ok(())
    }

    fn can_handle(&self, content: &str, _file_path: Option<&str>) -> bool {
        content.lines().any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with('{') && (trimmed.contains("\"log\"") || trimmed.contains("\"stream\""))
        })
    }
}

/// SpringBoot过滤器
///
/// 处理SpringBoot应用日志格式，提取时间戳、日志级别、线程名、类名等结构化信息。
///
/// # 处理逻辑
/// 1. 匹配SpringBoot标准日志格式
/// 2. 提取时间戳、级别、线程、类名等信息
/// 3. 标准化日志级别
/// 4. 确定输出流类型（stdout/stderr）
/// 注意：异常堆栈跟踪处理功能已移除
pub struct SpringBootFilter;

impl SpringBootFilter {
    /// SpringBoot日志格式的正则表达式
    /// 支持多种格式:
    /// 1. 2024-01-15 14:30:25.123 [main] INFO com.example.App - Message (传统格式)
    /// 2. 2025-10-15T07:40:55.169Z  INFO 1 --- [  EventHandler1] s.i.ProjectAttributeTemplateEventSpiImpl : Message (新格式)
    /// 3. 2025-01-15 10:30:45.123 INFO  [main] Starting application... (简化格式，无类名)
    const LOG_PATTERN: &'static str = r"^(\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}[.,]\d{3}(?:Z)?)\s+([A-Z]+)\s+(?:\d+\s+---\s+)?\[\s*([^\]]+)\s*\](?:\s+([^\s:]+)\s*:\s*)?(.*)$";
}

impl PluginFilter for SpringBootFilter {
    fn name(&self) -> &str {
        "springboot"
    }

    fn description(&self) -> &str {
        "SpringBoot应用日志解析过滤器，提取时间戳、级别、线程等结构化信息"
    }

    fn priority(&self) -> i32 {
        20 // 中等优先级，在Docker JSON之后
    }

    fn should_process(&self, context: &PluginChainContext) -> bool {
        // 如果当前行列表为空，说明这是第一次处理，需要解析原始内容
        if context.current_lines.is_empty() {
            return true;
        }

        // 检查是否有SpringBoot格式的日志行
        context.current_lines.iter().any(|line| {
            let content_lower = line.content.to_lowercase();
            // 检查新的日志格式特征: 2025-10-15T07:40:55.169Z  INFO 1 --- [thread] Class : Message
            let has_new_format = line.content.starts_with(|c: char| c.is_ascii_digit()) &&
                line.content.len() >= 20 &&
                (line.content.contains("---") && line.content.contains('['));

            // 检查传统格式特征
            let has_traditional_format = line.content.starts_with(|c: char| c.is_ascii_digit()) &&
                line.content.len() >= 10 &&
                (line.content.contains('[') || line.content.contains(" INFO ") || line.content.contains(" ERROR "));

            // 检查Spring相关关键字
            let has_spring_keywords = content_lower.contains("spring") ||
                content_lower.contains("application.start") ||
                content_lower.contains("springframework");

            // 检查标准日志级别
            let has_log_levels = content_lower.contains("info") ||
                content_lower.contains("error") ||
                content_lower.contains("warn") ||
                content_lower.contains("debug");

            has_new_format || has_traditional_format || has_spring_keywords || has_log_levels
        })
    }

    fn process(&self, context: &mut PluginChainContext, _request: &ParseRequest) -> Result<(), String> {
        info!("🌱 SpringBoot过滤器开始处理");

        let regex = regex::Regex::new(Self::LOG_PATTERN)
            .map_err(|e| format!("SpringBoot正则表达式编译失败: {}", e))?;

        info!("🔍 SpringBoot正则表达式: {}", Self::LOG_PATTERN);

        let lines_to_process = if context.current_lines.is_empty() {
            // 第一次处理，从原始内容创建行列表，过滤空行
            context.original_content.lines().enumerate().filter(|(_, line)| !line.trim().is_empty()).map(|(i, line)| {
                LogLine {
                    line_number: i + 1,
                    content: line.to_string(),
                    level: None,
                    timestamp: None,
                    formatted_content: None,
                    metadata: HashMap::new(),
                    processed_by: vec![],
                }
            }).collect()
        } else {
            // 后续处理，使用现有的行列表
            context.current_lines.clone()
        };

        let mut processed_lines = Vec::with_capacity(lines_to_process.len());
        let mut processed_count = 0;

        for mut line in lines_to_process {
            let trimmed = line.content.trim();

            // 跳过空白行 - 完全移除而不是标记为跳过
            if trimmed.is_empty() {
                continue;
            }

            let content_copy = line.content.clone();
            info!("🔍 尝试匹配行 {}: '{}'", line.line_number, content_copy);

            // 异常堆栈跟踪功能已移除 - 所有行都作为普通日志处理

            if let Some(captures) = regex.captures(&content_copy) {
                info!("✅ 匹配成功! 捕获组数量: {}", captures.len());
                for (i, cap) in captures.iter().enumerate() {
                    if let Some(group) = cap {
                        info!("  捕获组 {}: '{}'", i, group.as_str());
                    }
                }

                // 新格式的字段顺序: 时间戳、级别、线程名、类名、消息
                // 提取时间戳
                if let Some(timestamp) = captures.get(1) {
                    let normalized = self.normalize_timestamp(timestamp.as_str());
                    line.timestamp = Some(normalized.clone());
                    info!("  时间戳: {}", normalized);
                }

                // 提取并标准化日志级别 (捕获组2)
                if let Some(level) = captures.get(2) {
                    let normalized_level = self.normalize_level(level.as_str());
                    line.level = Some(normalized_level.clone());
                    info!("  日志级别: {} -> {}", level.as_str(), normalized_level);

                    // 根据级别确定stream类型
                    let stream_type = self.determine_stream_type(&normalized_level);
                    line.metadata.insert("stream".to_string(), stream_type.to_string());
                }

                // 提取线程名 (捕获组3)
                if let Some(thread) = captures.get(3) {
                    line.metadata.insert("thread".to_string(), thread.as_str().to_string());
                    info!("  线程名: {}", thread.as_str());
                }

                // 提取类名 (捕获组4) - 现在是可选的
                if let Some(logger) = captures.get(4) {
                    // 检查这是否是类名（不包含空格）还是消息内容的一部分
                    let logger_str = logger.as_str().trim();
                    if logger_str.contains(' ') {
                        // 如果包含空格，说明这是消息内容而不是类名
                        line.content = logger_str.to_string();
                        info!("  消息: {}", logger_str);
                    } else {
                        // 这是类名
                        line.metadata.insert("logger".to_string(), logger_str.to_string());
                        info!("  类名: {}", logger_str);

                        // 消息内容在捕获组5
                        if let Some(message) = captures.get(5) {
                            line.content = message.as_str().to_string();
                            info!("  消息: {}", message.as_str());
                        }
                    }
                } else {
                    // 没有类名，消息内容在捕获组5
                    if let Some(message) = captures.get(5) {
                        line.content = message.as_str().to_string();
                        info!("  消息: {}", message.as_str());
                    }
                }

                line.processed_by.push("springboot_filter".to_string());
                processed_count += 1;

                // 设置格式化内容为纯净的消息内容，避免重复显示日志级别
                line.formatted_content = Some(line.content.clone());

                info!("✅ SpringBoot解析成功: 行{} -> {}", line.line_number, line.content);
            } else {
                info!("❌ 匹配失败，检查是否有其他特征...");

                // 不匹配标准格式的行，可能是堆栈跟踪或其他内容
                // 检查是否包含日志级别关键字，如果不包含，设为DEBUG
                let content_lower = line.content.to_lowercase();
                if content_lower.contains("error") || content_lower.contains("exception") {
                    line.level = Some("ERROR".to_string());
                    line.metadata.insert("stream".to_string(), "stderr".to_string());
                    info!("  检测到ERROR关键字");
                } else if content_lower.contains("warn") {
                    line.level = Some("WARN".to_string());
                    line.metadata.insert("stream".to_string(), "stdout".to_string());
                    info!("  检测到WARN关键字");
                } else if content_lower.contains("info") {
                    line.level = Some("INFO".to_string());
                    line.metadata.insert("stream".to_string(), "stdout".to_string());
                    info!("  检测到INFO关键字");
                } else {
                    line.level = Some("DEBUG".to_string());
                    line.metadata.insert("stream".to_string(), "stdout".to_string());
                    info!("  设为DEBUG级别");
                }
                line.metadata.insert("type".to_string(), "unparsed".to_string());
            }

            processed_lines.push(line);
        }

        context.current_lines = processed_lines;
        context.set_chain_metadata("springboot_processed".to_string(), processed_count.to_string());

        info!("🌱 SpringBoot过滤器处理完成，处理了 {} 行", processed_count);
        Ok(())
    }

    fn can_handle(&self, content: &str, _file_path: Option<&str>) -> bool {
        let content_lower = content.to_lowercase();

        // 首先排除Docker JSON格式
        if content_lower.contains("{") &&
           content_lower.contains("\"log\"") &&
           content_lower.contains("\"stream\"") {
            return false;
        }

        // 检测SpringBoot特征
        content_lower.contains("spring") ||
        content_lower.contains("application.start") ||
        content_lower.contains("springframework") ||
        content_lower.contains("com.example.") ||  // 常见的SpringBoot包名
        content_lower.contains("http-nio-") ||     // Tomcat线程名
        content.lines().any(|line| {
            line.starts_with(|c: char| c.is_ascii_digit()) &&
            line.len() >= 10 &&
            (line.contains('[') || line.contains(" INFO ") || line.contains(" ERROR "))
        })
    }
}

impl SpringBootFilter {
    /// 标准化时间戳格式
    fn normalize_timestamp(&self, timestamp: &str) -> String {
        // 转换 "2024-01-15 14:30:25.123" 为 "2024-01-15T14:30:25"
        let trimmed = timestamp.trim();

        if let Some(dot_pos) = trimmed.find('.') {
            let base = &trimmed[..dot_pos];
            base.replace(' ', "T")
        } else if let Some(comma_pos) = trimmed.find(',') {
            let base = &trimmed[..comma_pos];
            base.replace(' ', "T")
        } else if trimmed.contains(' ') {
            trimmed.replace(' ', "T")
        } else {
            trimmed.to_string()
        }
    }

    /// 标准化日志级别
    fn normalize_level(&self, level: &str) -> String {
        match level.to_uppercase().as_str() {
            "ERROR" | "ERR" | "FATAL" | "SEVERE" => "ERROR".to_string(),
            "WARN" | "WARNING" | "ALERT" => "WARN".to_string(),
            "INFO" | "INFORMATION" | "NOTE" => "INFO".to_string(),
            "DEBUG" | "TRACE" | "VERBOSE" => "DEBUG".to_string(),
            _ => level.to_uppercase(),
        }
    }

    /// 根据日志级别确定输出流类型
    fn determine_stream_type(&self, level: &str) -> &'static str {
        match level {
            "ERROR" | "FATAL" | "SEVERE" => "stderr",
            _ => "stdout",
        }
    }

}

/// MyBatis过滤器
///
/// 识别和格式化MyBatis SQL日志，将分散的SQL相关行组合成完整的SQL语句。
///
/// # 处理逻辑
/// 1. 识别MyBatis特征关键词（Preparing, Parameters, ==>）
/// 2. 组合相关的SQL语句行
/// 3. 格式化SQL参数
/// 4. 提供SQL语句的统一格式化输出
pub struct MyBatisFilter;

impl PluginFilter for MyBatisFilter {
    fn name(&self) -> &str {
        "mybatis"
    }

    fn description(&self) -> &str {
        "MyBatis SQL日志过滤器，识别和格式化SQL语句及相关参数"
    }

    fn priority(&self) -> i32 {
        30 // 较低优先级，在基础格式解析之后
    }

    fn should_process(&self, context: &PluginChainContext) -> bool {
        // 检查是否有MyBatis相关的日志内容
        context.current_lines.iter().any(|line| {
            let content_lower = line.content.to_lowercase();
            content_lower.contains("preparing:") ||
            content_lower.contains("parameters:") ||
            content_lower.contains("==>")
        })
    }

    fn process(&self, context: &mut PluginChainContext, _request: &ParseRequest) -> Result<(), String> {
        info!("🗃️ MyBatis过滤器开始处理");

        let mut processed_lines = Vec::with_capacity(context.current_lines.len());
        let mut sql_group = Vec::new();
        let mut processed_count = 0;

        for line in context.current_lines.drain(..) {
            let content_lower = line.content.to_lowercase();

            if content_lower.contains("preparing:") ||
               content_lower.contains("parameters:") ||
               content_lower.contains("==>") {
                // 这是MyBatis相关的行，加入临时组
                sql_group.push(line);
            } else {
                // 不是MyBatis行，处理之前积累的SQL组
                if !sql_group.is_empty() {
                    let formatted_sql_lines = self.format_sql_group(sql_group.clone());
                    processed_lines.extend(formatted_sql_lines);
                    processed_count += sql_group.len();
                    sql_group = Vec::new();
                }
                processed_lines.push(line);
            }
        }

        // 处理最后的SQL组
        if !sql_group.is_empty() {
            let formatted_sql_lines = self.format_sql_group(sql_group.clone());
            processed_lines.extend(formatted_sql_lines);
            processed_count += sql_group.len();
        }

        context.current_lines = processed_lines;
        context.set_chain_metadata("mybatis_processed".to_string(), processed_count.to_string());

        info!("🗃️ MyBatis过滤器处理完成，处理了 {} 行", processed_count);
        Ok(())
    }

    fn can_handle(&self, content: &str, _file_path: Option<&str>) -> bool {
        let content_lower = content.to_lowercase();
        content_lower.contains("preparing:") ||
        content_lower.contains("parameters:") ||
        content_lower.contains("==>")
    }
}

impl MyBatisFilter {
    /// 格式化SQL行组
    fn format_sql_group(&self, sql_lines: Vec<LogLine>) -> Vec<LogLine> {
        let mut formatted_lines = Vec::with_capacity(sql_lines.len());

        for mut line in sql_lines {
            let content_lower = line.content.to_lowercase();

            if content_lower.contains("preparing:") {
                // SQL准备语句
                line.metadata.insert("sql_type".to_string(), "preparing".to_string());
                line.level = Some("DEBUG".to_string());

                // 提取SQL语句
                if let Some(sql_start) = line.content.to_lowercase().find("preparing:") {
                    let sql_statement = line.content[sql_start + 11..].trim();
                    line.metadata.insert("sql_statement".to_string(), sql_statement.to_string());
                }
            } else if content_lower.contains("parameters:") {
                // SQL参数
                line.metadata.insert("sql_type".to_string(), "parameters".to_string());
                line.level = Some("DEBUG".to_string());

                // 提取参数
                if let Some(param_start) = line.content.to_lowercase().find("parameters:") {
                    let parameters = line.content[param_start + 12..].trim();
                    line.metadata.insert("sql_parameters".to_string(), parameters.to_string());
                }
            } else if content_lower.contains("==>") {
                // SQL执行结果
                line.metadata.insert("sql_type".to_string(), "result".to_string());
                line.level = Some("INFO".to_string());
            }

            line.processed_by.push("mybatis_filter".to_string());
            formatted_lines.push(line);
        }

        formatted_lines
    }
}

/// JSON结构化过滤器
///
/// 将处理后的日志行统一格式化为JSON结构，便于前端处理和显示。
/// 通常作为链中的最后一个过滤器执行。
pub struct JsonStructureFilter;

impl PluginFilter for JsonStructureFilter {
    fn name(&self) -> &str {
        "json_structure"
    }

    fn description(&self) -> &str {
        "JSON结构化过滤器，将日志行统一格式化为JSON结构"
    }

    fn priority(&self) -> i32 {
        90 // 很高优先级，通常最后执行
    }

    fn should_process(&self, _context: &PluginChainContext) -> bool {
        true // 总是需要结构化输出
    }

    fn process(&self, context: &mut PluginChainContext, _request: &ParseRequest) -> Result<(), String> {
        info!("📋 JSON结构化过滤器开始处理");

        for line in &mut context.current_lines {
            // 如果已经有formatted_content，不要覆盖（保持SpringBootFilter等过滤器设置的纯净内容）
            if line.formatted_content.is_none() {
                // 构建格式化内容
                let formatted = self.build_formatted_content(&line);
                line.formatted_content = Some(formatted);
            }

            // 添加处理标记
            line.processed_by.push("json_structure_filter".to_string());
        }

        context.set_chain_metadata("json_structured".to_string(), context.current_lines.len().to_string());

        info!("📋 JSON结构化过滤器处理完成，格式化了 {} 行", context.current_lines.len());
        Ok(())
    }

    fn can_handle(&self, _content: &str, _file_path: Option<&str>) -> bool {
        true // 可以处理任何内容
    }
}

impl JsonStructureFilter {
    /// 构建格式化的内容字符串
    /// 格式: 级别 时间 前缀(10字符内，默认收起) 日志正文
    fn build_formatted_content(&self, line: &LogLine) -> String {
        let mut parts = Vec::new();

        // 1. 日志级别 (简化显示，不带括号)
        if let Some(level) = &line.level {
            parts.push(level.clone());
        }

        // 2. 时间戳 (简化格式，只显示时间部分)
        if let Some(timestamp) = &line.timestamp {
            let simplified_time = self.simplify_timestamp(timestamp);
            parts.push(simplified_time);
        }

        // 3. 前缀信息 (线程、类名等，限制在10字符以内，默认收起)
        let prefix = self.build_prefix(&line);
        if !prefix.is_empty() {
            parts.push(prefix);
        }

        // 4. 日志正文 (支持JSON收起和SQL格式化)
        let formatted_content = self.format_log_content(&line);
        parts.push(formatted_content);

        parts.join(" ")
    }

    /// 简化时间戳显示
    fn simplify_timestamp(&self, timestamp: &str) -> String {
        // 处理各种时间戳格式
        if timestamp.contains('T') {
            // ISO格式: 2025-01-15T10:30:25.123Z -> 10:30:25
            if let Some(time_part) = timestamp.split('T').nth(1) {
                return time_part.split('.').next().unwrap_or(time_part).to_string();
            }
        } else if timestamp.contains(':') {
            // 标准格式: 2025-01-15 10:30:25 -> 10:30:25
            if let Some(time_part) = timestamp.split_whitespace().nth(1) {
                return time_part.split('.').next().unwrap_or(time_part).to_string();
            }
        } else if timestamp.starts_with('[') && timestamp.contains(']') {
            // Java GC格式: [0.000s] -> 0.000s
            return timestamp.trim_matches(&['[', ']']).to_string();
        }

        // 如果无法解析，返回原时间戳的后8个字符
        if timestamp.len() > 8 {
            timestamp[timestamp.len() - 8..].to_string()
        } else {
            timestamp.to_string()
        }
    }

    /// 构建前缀信息 (限制10字符，默认收起)
    fn build_prefix(&self, line: &LogLine) -> String {
        let mut prefix_parts = Vec::new();

        // 线程名 (如果不是main)
        if let Some(thread) = line.metadata.get("thread") {
            if thread != "main" && thread.len() <= 8 {
                prefix_parts.push(thread.clone());
            }
        }

        // 类名/Logger名 (取前8个字符)
        if let Some(logger) = line.metadata.get("logger") {
            let short_logger = if logger.len() > 8 {
                format!("{}...", &logger[..5])
            } else {
                logger.clone()
            };
            prefix_parts.push(short_logger);
        }

        // SQL类型标记
        if let Some(sql_type) = line.metadata.get("sql_type") {
            let sql_icon = match sql_type.as_str() {
                "preparing" => "SQL",
                "parameters" => "PARAM",
                "result" => "RESULT",
                _ => "SQL"
            };
            prefix_parts.push(sql_icon.to_string());
        }

        // Java日志类型标记
        if let Some(log_type) = line.metadata.get("log_type") {
            if log_type.contains("gc") {
                prefix_parts.push("GC".to_string());
            }
        }

        // 组合前缀，限制总长度
        let combined = prefix_parts.join("|");
        if combined.len() > 10 {
            format!("{}...", &combined[..7])
        } else {
            combined
        }
    }

    /// 格式化日志正文 (支持JSON收起和SQL格式化)
    fn format_log_content(&self, line: &LogLine) -> String {
        let content = &line.content;

        // 如果是已格式化的异常，直接返回
        if let Some(log_type) = line.metadata.get("log_type") {
            if log_type == "exception_formatted" ||
               log_type == "exception_main" ||
               log_type == "exception_business_header" ||
               log_type == "exception_business" ||
               log_type == "exception_framework_header" ||
               log_type == "exception_framework" {
                return content.clone();
            }
        }

        // SQL格式化
        if let Some(sql_type) = line.metadata.get("sql_type") {
            return self.format_sql_content(content, sql_type);
        }

        // JSON内容收起
        if self.is_json_content(content) {
            return self.format_json_content(content);
        }

        // 普通内容直接返回
        content.clone()
    }

    /// 格式化SQL内容
    fn format_sql_content(&self, content: &str, sql_type: &str) -> String {
        let content = content.trim();

        match sql_type {
            "preparing" => {
                if let Some(start) = content.to_lowercase().find("preparing:") {
                    let sql = content[start + 11..].trim();
                    format!("📝 SQL: {}", self.format_sql_statement(sql))
                } else {
                    format!("📝 SQL: {}", content)
                }
            }
            "parameters" => {
                if let Some(start) = content.to_lowercase().find("parameters:") {
                    let params = content[start + 12..].trim();
                    format!("🔧 PARAMS: {}", self.format_sql_parameters(params))
                } else {
                    format!("🔧 PARAMS: {}", content)
                }
            }
            "result" => {
                if let Some(start) = content.to_lowercase().find("==>") {
                    let result = content[start + 3..].trim();
                    format!("✅ RESULT: {}", result)
                } else {
                    format!("✅ RESULT: {}", content)
                }
            }
            _ => content.to_string()
        }
    }

    /// 格式化SQL语句
    fn format_sql_statement(&self, sql: &str) -> String {
        // 简单的SQL格式化：关键字大写，添加换行
        let formatted = sql
            .replace("select", "SELECT")
            .replace("from", "FROM")
            .replace("where", "WHERE")
            .replace("insert", "INSERT")
            .replace("into", "INTO")
            .replace("values", "VALUES")
            .replace("update", "UPDATE")
            .replace("set", "SET")
            .replace("delete", "DELETE")
            .replace("join", "JOIN")
            .replace("on", "ON")
            .replace("order by", "ORDER BY")
            .replace("group by", "GROUP BY");

        // 如果SQL太长，进行收起处理
        if formatted.len() > 100 {
            format!("{}...", &formatted[..97])
        } else {
            formatted
        }
    }

    /// 格式化SQL参数
    fn format_sql_parameters(&self, params: &str) -> String {
        // 简化参数显示
        if params.len() > 50 {
            format!("{}...", &params[..47])
        } else {
            params.to_string()
        }
    }

    /// 检查是否为JSON内容
    fn is_json_content(&self, content: &str) -> bool {
        let trimmed = content.trim();
        (trimmed.starts_with('{') && trimmed.ends_with('}')) ||
        (trimmed.starts_with('[') && trimmed.ends_with(']'))
    }

    /// 格式化JSON内容
    fn format_json_content(&self, content: &str) -> String {
        let trimmed = content.trim();

        if trimmed.len() > 80 {
            // JSON内容太长，进行收起
            format!("📄 JSON: {}...", &trimmed[..77])
        } else {
            format!("📄 JSON: {}", trimmed)
        }
    }

    /// 格式化异常内容
    #[allow(dead_code)]
    fn format_exception_content(&self, content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();

        if lines.len() > 3 {
            // 异常内容太长，只显示前3行
            let preview: Vec<String> = lines.iter().take(3).map(|&s| s.to_string()).collect();
            format!("💥 EXCEPTION: {}... (+{} lines)", preview.join(" "), lines.len() - 3)
        } else {
            format!("💥 EXCEPTION: {}", content)
        }
    }
}

/// Java应用日志过滤器
///
/// 处理Java应用的通用日志格式，包括GC日志、应用日志等。
/// 能够识别和处理Java特有的日志格式，如GC日志、JVM日志等。
///
/// # 处理逻辑
/// 1. 识别Java日志特征（如GC日志、JVM启动日志等）
/// 2. 提取时间戳和日志级别
/// 3. 标准化日志级别（GC日志通常是INFO级别）
/// 4. 确定输出流类型
pub struct JavaLogFilter;

impl PluginFilter for JavaLogFilter {
    fn name(&self) -> &str {
        "java_log"
    }

    fn description(&self) -> &str {
        "Java应用日志过滤器，处理GC日志、JVM日志等Java应用日志"
    }

    fn priority(&self) -> i32 {
        25 // 中等优先级，在Docker JSON之后
    }

    fn should_process(&self, context: &PluginChainContext) -> bool {
        // 检查是否有Java相关的日志内容
        context.current_lines.iter().any(|line| {
            let content_lower = line.content.to_lowercase();
            content_lower.contains("[warning][gc]") ||
            content_lower.contains("[info][gc]") ||
            content_lower.contains("[debug][gc]") ||
            content_lower.contains("gc,") ||
            content_lower.contains("heap") ||
            content_lower.contains("g1") ||
            (line.content.starts_with('[') && line.content.contains("]"))
        })
    }

    fn process(&self, context: &mut PluginChainContext, _request: &ParseRequest) -> Result<(), String> {
        info!("☕ Java日志过滤器开始处理");

        let mut processed_lines = Vec::with_capacity(context.current_lines.len());
        let mut processed_count = 0;

        for mut line in context.current_lines.drain(..) {
            let content_lower = line.content.to_lowercase();

            // 识别Java日志格式，如: [0.000s][warning][gc] -XX:+PrintGCDetails is deprecated
            if content_lower.contains("[warning][gc]") {
                line.level = Some("WARN".to_string());
                line.metadata.insert("log_type".to_string(), "gc_warning".to_string());
                line.metadata.insert("stream".to_string(), "stdout".to_string());
                processed_count += 1;
            } else if content_lower.contains("[info][gc]") {
                line.level = Some("INFO".to_string());
                line.metadata.insert("log_type".to_string(), "gc_info".to_string());
                line.metadata.insert("stream".to_string(), "stdout".to_string());
                processed_count += 1;
            } else if content_lower.contains("[debug][gc]") {
                line.level = Some("DEBUG".to_string());
                line.metadata.insert("log_type".to_string(), "gc_debug".to_string());
                line.metadata.insert("stream".to_string(), "stdout".to_string());
                processed_count += 1;
            } else if line.content.starts_with('[') && line.content.contains("][") {
                // 通用Java日志格式
                if let Some(start_bracket) = line.content.find('[') {
                    if let Some(end_bracket) = line.content[start_bracket + 1..].find(']') {
                        let time_part = &line.content[start_bracket..end_bracket + 1];
                        let remaining = &line.content[end_bracket + 1..];

                        if let Some(level_start) = remaining.find('[') {
                            if let Some(level_end) = remaining[level_start + 1..].find(']') {
                                let level_part = &remaining[level_start..level_end + 1];
                                let message = &remaining[level_end + 1..].trim_start_matches(' ');

                                // 提取时间戳
                                if time_part.len() > 2 {
                                    line.timestamp = Some(time_part.trim_matches(&['[', ']']).to_string());
                                }

                                // 提取并标准化级别
                                let level_upper = level_part.to_uppercase();
                                let normalized_level = if level_upper.contains("WARNING") {
                                    "WARN".to_string()
                                } else if level_upper.contains("ERROR") {
                                    "ERROR".to_string()
                                } else if level_upper.contains("INFO") {
                                    "INFO".to_string()
                                } else if level_upper.contains("DEBUG") {
                                    "DEBUG".to_string()
                                } else if level_upper.contains("TRACE") {
                                    "TRACE".to_string()
                                } else {
                                    "INFO".to_string() // 默认级别
                                };

                                line.level = Some(normalized_level.clone());
                                line.content = message.to_string();

                                // 根据级别确定stream类型
                                let stream_type = if normalized_level == "ERROR" { "stderr" } else { "stdout" };
                                line.metadata.insert("stream".to_string(), stream_type.to_string());
                                line.metadata.insert("log_type".to_string(), "java".to_string());

                                processed_count += 1;
                            }
                        }
                    }
                }
            }

            line.processed_by.push("java_log_filter".to_string());
            processed_lines.push(line);
        }

        context.current_lines = processed_lines;
        context.set_chain_metadata("java_log_processed".to_string(), processed_count.to_string());

        info!("☕ Java日志过滤器处理完成，处理了 {} 行", processed_count);
        Ok(())
    }

    fn can_handle(&self, content: &str, _file_path: Option<&str>) -> bool {
        let content_lower = content.to_lowercase();
        content_lower.contains("[warning][gc]") ||
        content_lower.contains("[info][gc]") ||
        content_lower.contains("[debug][gc]") ||
        content_lower.contains("heap") ||
        content_lower.contains("g1") ||
        content.lines().any(|line| {
            line.starts_with('[') && line.contains("][")
        })
    }
}

/// 内容增强过滤器
///
/// 对日志内容进行额外的增强处理，如高亮、链接识别、数据提取等。
pub struct ContentEnhancerFilter;

impl PluginFilter for ContentEnhancerFilter {
    fn name(&self) -> &str {
        "content_enhancer"
    }

    fn description(&self) -> &str {
        "内容增强过滤器，提供高亮、链接识别等增强功能"
    }

    fn priority(&self) -> i32 {
        80 // 高优先级，在主要处理之后
    }

    fn should_process(&self, context: &PluginChainContext) -> bool {
        // 检查是否有需要增强的内容
        context.current_lines.iter().any(|line| {
            line.content.contains("http://") ||
            line.content.contains("https://") ||
            line.content.contains("@") ||
            line.level.as_ref().map_or(false, |l| l == "ERROR")
        })
    }

    fn process(&self, context: &mut PluginChainContext, _request: &ParseRequest) -> Result<(), String> {
        info!("✨ 内容增强过滤器开始处理");

        let mut enhanced_count = 0;

        for line in &mut context.current_lines {
            let mut enhanced = false;

            // 检测URL
            if line.content.contains("http://") || line.content.contains("https://") {
                line.metadata.insert("has_url".to_string(), "true".to_string());
                enhanced = true;
            }

            // 检测邮箱地址
            if line.content.contains("@") {
                line.metadata.insert("has_email".to_string(), "true".to_string());
                enhanced = true;
            }

            // 检测错误级别，添加特殊标记
            if line.level.as_ref().map_or(false, |l| l == "ERROR") {
                line.metadata.insert("is_error".to_string(), "true".to_string());
                enhanced = true;
            }

            if enhanced {
                line.processed_by.push("content_enhancer_filter".to_string());
                enhanced_count += 1;
            }
        }

        context.set_chain_metadata("content_enhanced".to_string(), enhanced_count.to_string());

        info!("✨ 内容增强过滤器处理完成，增强了 {} 行", enhanced_count);
        Ok(())
    }

    fn can_handle(&self, _content: &str, _file_path: Option<&str>) -> bool {
        true // 可以处理任何内容，但会选择性增强
    }
}

#[cfg(test)]
mod springboot_tests {
    use crate::plugins::chain::{PluginFilter, PluginChainContext};
    use crate::plugins::{LogLine, ParseRequest};
    use std::collections::HashMap;

    #[test]
    fn test_regex_pattern_directly() {
        use regex::Regex;

        let pattern = r"^(\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}[.,]\d{3}(?:Z)?)\s+([A-Z]+)\s+(?:\d+\s+---\s+)?\[\s*([^\]]+)\s*\]\s+([^\s:]+)\s*:\s*(.*)$";
        let regex = Regex::new(pattern).unwrap();

        let test_line = "2025-10-15T07:40:55.169Z  INFO 1 --- [  EventHandler1] s.i.ProjectAttributeTemplateEventSpiImpl : ProjectAttributeTemplateEventSpiImpl 收到事件，objectName:Document number:136";

        if let Some(captures) = regex.captures(test_line) {
            println!("✅ Regex匹配成功!");
            println!("  捕获组数量: {}", captures.len());
            for (i, cap) in captures.iter().enumerate() {
                if let Some(group) = cap {
                    println!("  捕获组 {}: '{}'", i, group.as_str());
                }
            }

            assert_eq!(captures.len(), 6); // 0 + 5 capture groups
            assert_eq!(captures.get(1).unwrap().as_str(), "2025-10-15T07:40:55.169Z");
            assert_eq!(captures.get(2).unwrap().as_str(), "INFO");
            assert_eq!(captures.get(3).unwrap().as_str(), "EventHandler1");
            assert_eq!(captures.get(4).unwrap().as_str(), "s.i.ProjectAttributeTemplateEventSpiImpl");
            assert_eq!(captures.get(5).unwrap().as_str(), "ProjectAttributeTemplateEventSpiImpl 收到事件，objectName:Document number:136");
        } else {
            panic!("❌ Regex匹配失败");
        }
    }

  }