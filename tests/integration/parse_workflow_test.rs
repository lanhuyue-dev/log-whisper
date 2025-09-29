use log_whisper::parser::LogParser;
use log_whisper::models::ParseConfig;

#[tokio::test]
async fn test_full_parse_workflow() {
    // 创建测试日志文件
    let test_log_content = r#"2024-01-01 10:00:00 INFO Application started
2024-01-01 10:00:01 DEBUG Loading configuration
2024-01-01 10:00:02 INFO Database connection established
2024-01-01 10:00:03 DEBUG Preparing: SELECT * FROM users WHERE id = ?
2024-01-01 10:00:03 DEBUG Parameters: 123(Integer), "admin"(String)
2024-01-01 10:00:04 INFO User login successful
2024-01-01 10:00:05 ERROR Database connection failed
2024-01-01 10:00:06 WARN Retrying connection...
2024-01-01 10:00:07 INFO Connection restored
{"user": "admin", "action": "login", "timestamp": "2024-01-01T10:00:04Z"}
2024-01-01 10:00:08 INFO Application shutdown"#;
    
    // 写入测试文件
    let test_file_path = "test_parse_workflow.log";
    std::fs::write(test_file_path, test_log_content).unwrap();
    
    // 创建解析器
    let parser = LogParser::new();
    
    // 解析文件
    let result = parser.parse_file(test_file_path).await;
    assert!(result.is_ok());
    
    let result_set = result.unwrap();
    assert!(!result_set.results.is_empty());
    
    // 验证解析结果
    let results = &result_set.results;
    assert!(results.len() >= 8); // 至少应该有8行日志
    
    // 验证错误日志被正确识别
    let error_results: Vec<_> = results.iter().filter(|r| r.is_error).collect();
    assert!(!error_results.is_empty());
    
    // 验证警告日志被正确识别
    let warning_results: Vec<_> = results.iter().filter(|r| r.is_warning).collect();
    assert!(!warning_results.is_empty());
    
    // 验证统计信息
    assert!(result_set.total_stats.total_lines > 0);
    assert!(result_set.total_stats.total_parse_time_ms > 0);
    
    // 清理测试文件
    std::fs::remove_file(test_file_path).unwrap();
}

#[tokio::test]
async fn test_mybatis_plugin_parsing() {
    let test_log_content = r#"2024-01-01 10:00:00 DEBUG Preparing: SELECT * FROM users WHERE id = ? AND name = ?
2024-01-01 10:00:00 DEBUG Parameters: 123(Integer), "admin"(String)"#;
    
    let test_file_path = "test_mybatis.log";
    std::fs::write(test_file_path, test_log_content).unwrap();
    
    let config = ParseConfig {
        plugin_name: "MyBatis".to_string(),
        enable_cache: false,
        max_file_size: 50 * 1024 * 1024,
        timeout_ms: 30000,
    };
    
    let parser = LogParser::with_config(config);
    let result = parser.parse_file(test_file_path).await;
    
    assert!(result.is_ok());
    
    let result_set = result.unwrap();
    assert!(!result_set.results.is_empty());
    
    // 验证MyBatis SQL被正确解析
    let has_sql_blocks = result_set.results.iter().any(|r| {
        r.rendered_blocks.iter().any(|b| b.block_type.to_string() == "Sql")
    });
    assert!(has_sql_blocks);
    
    std::fs::remove_file(test_file_path).unwrap();
}

#[tokio::test]
async fn test_json_plugin_parsing() {
    let test_log_content = r#"2024-01-01 10:00:00 INFO {"user": "admin", "action": "login", "timestamp": "2024-01-01T10:00:04Z"}
2024-01-01 10:00:01 INFO {"error": "connection failed", "retry": true}"#;
    
    let test_file_path = "test_json.log";
    std::fs::write(test_file_path, test_log_content).unwrap();
    
    let config = ParseConfig {
        plugin_name: "JSON".to_string(),
        enable_cache: false,
        max_file_size: 50 * 1024 * 1024,
        timeout_ms: 30000,
    };
    
    let parser = LogParser::with_config(config);
    let result = parser.parse_file(test_file_path).await;
    
    assert!(result.is_ok());
    
    let result_set = result.unwrap();
    assert!(!result_set.results.is_empty());
    
    // 验证JSON被正确解析
    let has_json_blocks = result_set.results.iter().any(|r| {
        r.rendered_blocks.iter().any(|b| b.block_type.to_string() == "Json")
    });
    assert!(has_json_blocks);
    
    std::fs::remove_file(test_file_path).unwrap();
}

#[tokio::test]
async fn test_error_highlighting() {
    let test_log_content = r#"2024-01-01 10:00:00 INFO Application started
2024-01-01 10:00:01 ERROR Database connection failed
2024-01-01 10:00:02 WARN Retrying connection...
2024-01-01 10:00:03 INFO Connection restored"#;
    
    let test_file_path = "test_errors.log";
    std::fs::write(test_file_path, test_log_content).unwrap();
    
    let parser = LogParser::new();
    let result = parser.parse_file(test_file_path).await;
    
    assert!(result.is_ok());
    
    let result_set = result.unwrap();
    assert!(!result_set.results.is_empty());
    
    // 验证错误和警告被正确识别
    let error_count = result_set.results.iter().filter(|r| r.is_error).count();
    let warning_count = result_set.results.iter().filter(|r| r.is_warning).count();
    
    assert!(error_count > 0);
    assert!(warning_count > 0);
    
    std::fs::remove_file(test_file_path).unwrap();
}

#[tokio::test]
async fn test_large_file_handling() {
    // 创建一个大文件（模拟）
    let mut large_content = String::new();
    for i in 1..=1000 {
        large_content.push_str(&format!("2024-01-01 10:00:{:02} INFO Log entry {}\n", i % 60, i));
    }
    
    let test_file_path = "test_large.log";
    std::fs::write(test_file_path, large_content).unwrap();
    
    let parser = LogParser::new();
    let result = parser.parse_file(test_file_path).await;
    
    assert!(result.is_ok());
    
    let result_set = result.unwrap();
    assert_eq!(result_set.results.len(), 1000);
    
    std::fs::remove_file(test_file_path).unwrap();
}

#[tokio::test]
async fn test_file_validation() {
    let parser = LogParser::new();
    
    // 测试不存在的文件
    let result = parser.parse_file("nonexistent.log").await;
    assert!(result.is_err());
    
    // 测试不支持的文件类型
    let test_file_path = "test.txt";
    std::fs::write(test_file_path, "test content").unwrap();
    
    // 这个应该成功，因为.txt是支持的格式
    let result = parser.parse_file(test_file_path).await;
    assert!(result.is_ok());
    
    std::fs::remove_file(test_file_path).unwrap();
}
