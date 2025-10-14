#[cfg(test)]
mod springboot_format_tests {
    use crate::plugins::{LogParser, ParseRequest};
    use crate::plugins::springboot::SpringBootParser;

    #[test]
    fn test_springboot_dockerjson_format_consistency() {
        let parser = SpringBootParser;
        let request = ParseRequest::default();

        // 测试SpringBoot日志格式
        let springboot_content = r#"2024-09-30 08:00:03.456 [main] WARN com.example.DeprecatedApi - Deprecated API endpoint /old-api detected
2024-09-30 08:00:04.789 [worker-thread-1] INFO com.example.Server - Server listening on port 8080
2024-09-30 08:00:05.123 [main] INFO com.example.Application - OK
2024-09-30 08:00:06.456 [http-nio-8080-exec-1] INFO com.example.Controller - POST /api/login - 201 Created
2024-09-30 08:00:07.789 [redis-thread-1] ERROR com.example.RedisService - Failed to connect to Redis Connection timeout after 30 seconds Retrying in 5 seconds...
    at com.example.RedisService.connect(RedisService.java:156)
    at com.example.RedisService.<init>(RedisService.java:89)
2024-09-30 08:00:13.456 [redis-thread-1] INFO com.example.RedisService - Redis connection re-established"#;

        let result = parser.parse(springboot_content, &request).unwrap();

        println!("=== SpringBoot格式化测试结果 ===");
        for (i, line) in result.lines.iter().take(8).enumerate() {
            println!("{}. {}", i + 1, line.formatted_content.as_ref().unwrap_or(&line.content));
        }

        // 验证格式一致性
        assert_eq!(result.lines.len(), 9); // 5个正常行 + 4个堆栈跟踪行

        // 验证第一行：WARNING日志应该显示为STDOUT
        let first_line = &result.lines[0];
        assert!(first_line.formatted_content.as_ref().unwrap().contains("2024-09-30T08:00:03"));
        assert!(first_line.formatted_content.as_ref().unwrap().contains("[WARN]"));
        assert!(first_line.formatted_content.as_ref().unwrap().contains("[STDOUT]"));
        assert!(first_line.metadata.get("stream").unwrap() == "stdout");

        // 验证第5行：ERROR日志应该显示为STDERR
        let fifth_line = &result.lines[4];
        assert!(fifth_line.formatted_content.as_ref().unwrap().contains("2024-09-30T08:00:07"));
        assert!(fifth_line.formatted_content.as_ref().unwrap().contains("[ERROR]"));
        assert!(fifth_line.formatted_content.as_ref().unwrap().contains("[STDERR]"));
        assert!(fifth_line.metadata.get("stream").unwrap() == "stderr");

        // 验证堆栈跟踪行
        let stacktrace_line = &result.lines[5];
        assert!(stacktrace_line.formatted_content.as_ref().unwrap().contains("[ERROR]"));
        assert!(stacktrace_line.formatted_content.as_ref().unwrap().contains("[STDERR]"));
        assert!(stacktrace_line.level.as_ref().unwrap() == "ERROR");
        assert!(stacktrace_line.metadata.get("stream").unwrap() == "stderr");
        assert!(stacktrace_line.metadata.get("type").unwrap() == "stacktrace");

        println!("✅ SpringBoot格式化测试通过！");
    }

    #[test]
    fn test_iso_timestamp_conversion() {
        let parser = SpringBootParser;
        let request = ParseRequest::default();

        let test_content = r#"2024-01-15 14:30:25.123 [main] INFO Test message"#;

        let result = parser.parse(test_content, &request).unwrap();
        let line = &result.lines[0];

        // 验证时间戳转换为ISO 8601格式
        assert!(line.timestamp.as_ref().unwrap() == "2024-01-15T14:30:25");

        // 验证格式化内容包含正确的ISO格式时间戳
        assert!(line.formatted_content.as_ref().unwrap().contains("2024-01-15T14:30:25"));

        println!("✅ ISO时间戳转换测试通过！");
        println!("   原始: 2024-01-15 14:30:25.123");
        println!("   转换: {}", line.timestamp.as_ref().unwrap());
    }

    #[test]
    fn test_stream_determination() {
        let parser = SpringBootParser;
        let request = ParseRequest::default();

        let test_content = r#"2024-01-15 14:30:25.123 [main] ERROR Error message
2024-01-15 14:30:26.456 [main] WARN Warning message
2024-01-15 14:30:27.789 [main] INFO Info message
2024-01-15 14:30:28.012 [main] DEBUG Debug message"#;

        let result = parser.parse(test_content, &request).unwrap();

        // ERROR -> STDERR
        assert_eq!(result.lines[0].metadata.get("stream").unwrap(), "stderr");
        assert!(result.lines[0].formatted_content.as_ref().unwrap().contains("[STDERR]"));

        // WARN -> STDOUT (不是ERROR级别)
        assert_eq!(result.lines[1].metadata.get("stream").unwrap(), "stdout");
        assert!(result.lines[1].formatted_content.as_ref().unwrap().contains("[STDOUT]"));

        // INFO -> STDOUT
        assert_eq!(result.lines[2].metadata.get("stream").unwrap(), "stdout");
        assert!(result.lines[2].formatted_content.as_ref().unwrap().contains("[STDOUT]"));

        // DEBUG -> STDOUT
        assert_eq!(result.lines[3].metadata.get("stream").unwrap(), "stdout");
        assert!(result.lines[3].formatted_content.as_ref().unwrap().contains("[STDOUT]"));

        println!("✅ Stream类型确定测试通过！");
    }
}