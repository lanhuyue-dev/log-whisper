//! 插件系统测试模块
//! 
//! 提供插件系统的单元测试和集成测试

#[cfg(test)]
mod tests {
    use super::*;
    use super::builtin::*;
    use super::core::EnhancedPluginManager;

    #[tokio::test]
    async fn test_docker_json_parser() {
        let parser = DockerJsonParser::new();
        
        // 测试Docker JSON格式识别
        let docker_json_line = r#"{"log":"[0.008s][info   ][gc,init] Memory: 15193M\n","stream":"stdout","time":"2025-09-16T08:17:32.180873992Z"}"#;
        assert!(parser.can_handle(docker_json_line));
        
        // 测试处理
        let mut log_entry = LogEntry::new(1, docker_json_line.to_string());
        let result = parser.process(&mut log_entry);
        assert!(result.is_ok());
        
        // 验证处理结果
        assert_eq!(log_entry.level, Some("INFO".to_string()));
        assert!(log_entry.timestamp.is_some());
        assert!(log_entry.metadata.contains_key("stream"));
        assert!(log_entry.is_processed_by("docker_json"));
    }

    #[tokio::test]
    async fn test_springboot_parser() {
        let parser = SpringBootParser::new().unwrap();
        
        // 测试SpringBoot格式识别
        let springboot_line = "2025-10-04 09:15:01.123 INFO  [http-nio-8080-exec-1] com.example.controller.UserController : 用户登录请求开始";
        assert!(parser.can_handle(springboot_line));
        
        // 测试处理
        let mut log_entry = LogEntry::new(1, springboot_line.to_string());
        let result = parser.process(&mut log_entry);
        assert!(result.is_ok());
        
        // 验证处理结果
        assert_eq!(log_entry.level, Some("INFO".to_string()));
        assert!(log_entry.timestamp.is_some());
        assert!(log_entry.metadata.contains_key("thread"));
        assert!(log_entry.metadata.contains_key("class"));
        assert!(log_entry.is_processed_by("springboot"));
    }

    #[tokio::test]
    async fn test_stack_trace_aggregator() {
        let aggregator = StackTraceAggregator::new();
        
        // 测试堆栈跟踪行识别
        let stack_line = "    at com.example.service.ValidationService.validateInput(ValidationService.java:45)";
        assert!(aggregator.can_handle(stack_line));
        
        // 测试异常开始行
        let exception_line = "java.lang.NullPointerException: Cannot invoke \"String.trim()\" because \"input\" is null";
        assert!(aggregator.can_handle(exception_line));
        
        // 测试处理
        let mut log_entry = LogEntry::new(1, stack_line.to_string());
        let result = aggregator.process(&mut log_entry);
        assert!(result.is_ok());
        
        // 验证处理结果
        assert_eq!(log_entry.metadata.get("type"), Some(&"stack_trace".to_string()));
        assert!(log_entry.is_processed_by("stack_trace_aggregator"));
    }

    #[tokio::test]
    async fn test_mybatis_parser() {
        let parser = MyBatisParser::new();
        
        // 测试MyBatis SQL日志识别
        let sql_line = "Preparing: SELECT * FROM users WHERE username = ? AND password = ?";
        assert!(parser.can_handle(sql_line));
        
        let params_line = "Parameters: admin(String), password123(String)";
        assert!(parser.can_handle(params_line));
        
        // 测试处理
        let mut log_entry = LogEntry::new(1, sql_line.to_string());
        let result = parser.process(&mut log_entry);
        assert!(result.is_ok());
        
        // 验证处理结果
        assert_eq!(log_entry.metadata.get("sql_type"), Some(&"preparing".to_string()));
        assert!(log_entry.is_processed_by("mybatis"));
    }

    #[tokio::test]
    async fn test_json_formatter() {
        let formatter = JsonFormatter::new();
        
        // 测试JSON内容识别
        let json_line = r#"{"username": "admin", "loginTime": "2025-10-04T09:15:01Z"}"#;
        assert!(formatter.can_handle(json_line));
        
        // 测试处理
        let mut log_entry = LogEntry::new(1, json_line.to_string());
        let result = formatter.process(&mut log_entry);
        assert!(result.is_ok());
        
        // 验证处理结果
        assert!(log_entry.formatted_content.is_some());
        assert_eq!(log_entry.metadata.get("formatted"), Some(&"true".to_string()));
        assert!(log_entry.is_processed_by("json_formatter"));
    }

    #[tokio::test]
    async fn test_plugin_manager() {
        let manager = EnhancedPluginManager::new();
        
        // 初始化插件系统
        let result = manager.initialize().await;
        assert!(result.is_ok());
        
        // 获取插件统计
        let stats = manager.get_plugin_stats().await;
        assert!(stats.total_plugins > 0);
        
        // 测试日志处理
        let test_logs = vec![
            r#"{"log":"Application starting up...\n","stream":"stdout","time":"2025-09-16T08:17:33.180880085Z"}"#,
            "2025-10-04 09:15:01.123 INFO  [http-nio-8080-exec-1] com.example.controller.UserController : 用户登录请求开始",
            "    at com.example.service.ValidationService.validateInput(ValidationService.java:45)",
        ];
        
        let mut log_entries = Vec::new();
        for (i, log) in test_logs.iter().enumerate() {
            log_entries.push(LogEntry::new(i + 1, log.to_string()));
        }
        
        let result = manager.process_log_entries(log_entries).await;
        assert!(result.is_ok());
        
        let processed_entries = result.unwrap();
        assert_eq!(processed_entries.len(), 3);
        
        // 验证第一个条目（Docker JSON）
        let first_entry = &processed_entries[0];
        assert!(first_entry.is_processed_by("docker_json"));
        
        // 验证第二个条目（SpringBoot）
        let second_entry = &processed_entries[1];
        assert!(second_entry.is_processed_by("springboot"));
        
        // 验证第三个条目（堆栈跟踪）
        let third_entry = &processed_entries[2];
        assert!(third_entry.is_processed_by("stack_trace_aggregator"));
    }

    #[tokio::test]
    async fn test_plugin_priority() {
        let manager = EnhancedPluginManager::new();
        manager.initialize().await.unwrap();
        
        // 测试插件优先级处理
        let test_line = r#"{"log":"{\"username\": \"admin\", \"loginTime\": \"2025-10-04T09:15:01Z\"}\n","stream":"stdout","time":"2025-09-16T08:17:33.180880085Z"}"#;
        
        let mut log_entry = LogEntry::new(1, test_line.to_string());
        let result = manager.process_log_entry(log_entry).await;
        assert!(result.is_ok());
        
        let processed_entry = result.unwrap();
        
        // 应该被Docker JSON解析器处理
        assert!(processed_entry.is_processed_by("docker_json"));
        
        // 如果内容包含JSON，也应该被JSON格式化器处理
        if processed_entry.content.contains('{') && processed_entry.content.contains('}') {
            assert!(processed_entry.is_processed_by("json_formatter"));
        }
    }
}
