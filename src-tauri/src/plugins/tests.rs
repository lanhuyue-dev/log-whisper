// Integration test for Docker chain processing with GC logs
// This test verifies that the plugin chain system correctly handles Docker JSON logs containing Java GC logs

#[cfg(test)]
mod tests {
    use crate::plugins::core::EnhancedPluginManager;
    use crate::plugins::presets::register_preset_chains;
    use crate::plugins::chain::{PluginChainManager, PluginFilter};
    use crate::plugins::filters::{DockerJsonFilter, SpringBootFilter, JavaLogFilter, MyBatisFilter};
    use crate::plugins::{ParseRequest, LogLine};

    #[tokio::test]
    async fn test_docker_chain_gc_log_processing() {
        // Initialize logging for test
        let _ = env_logger::builder().is_test(true).try_init();

        println!("🧪 Testing Docker Chain Processing with GC Logs");

        // Test data: Docker JSON logs containing Java GC logs (from user's sample)
        let docker_logs = vec![
            r#"{"log":"[0.000s][warning][gc] -XX:+PrintGCDetails is deprecated. Will use -Xlog:gc* instead.\n","stream":"stdout","time":"2025-09-16T08:17:32.172897326Z"}"#,
            r#"{"log":"[0.002s][info ][gc,init] CardTable entry size: 512\n","stream":"stdout","time":"2025-09-16T08:17:32.174908043Z"}"#,
            r#"{"log":"[0.002s][info ][gc ] Using G1\n","stream":"stdout","time":"2025-09-16T08:17:32.174994988Z"}"#,
            r#"{"log":"[0.008s][info ][gc,init] Version: 21.0.7+9-LTS (release)\n","stream":"stdout","time":"2025-09-16T08:17:32.175102Z"}"#,
            r#"{"log":"2025-01-15 10:30:25.123 [main] INFO com.example.Application - Application started successfully\n","stream":"stdout","time":"2025-01-15T10:30:25.123Z"}"#
        ];

        let combined_content = docker_logs.join("\n");

        // Test 1: Enhanced Plugin Manager Auto-Detection
        println!("🔧 Test 1: Enhanced Plugin Manager Auto-Detection");

        let enhanced_manager = EnhancedPluginManager::new();

        // Initialize the enhanced plugin manager
        enhanced_manager.initialize().await.expect("Failed to initialize enhanced plugin manager");

        let request = ParseRequest {
            content: combined_content.clone(),
            plugin: None, // Auto-detect
            file_path: Some("container.log".to_string()),
            chunk_size: None,
        };

        let result = enhanced_manager.auto_detect_and_parse(&request);

        assert!(result.is_ok(), "Auto-detection should succeed: {:?}", result);

        let parse_result = result.unwrap();
        println!("✅ Auto-detection successful!");
        println!("📊 Processing Results:");
        println!("  - Total input lines: {}", parse_result.total_lines);
        println!("  - Parsed log lines: {}", parse_result.lines.len());
        println!("  - Detected format: {:?}", parse_result.detected_format);

        // Verify that logs were processed
        assert!(!parse_result.lines.is_empty(), "Should have parsed log lines");

        // Test 2: Docker Chain Direct Processing
        println!("🔗 Test 2: Docker Chain Direct Processing");

        let mut chain_manager = PluginChainManager::new();
        register_preset_chains(&mut chain_manager);

        let request = ParseRequest {
            content: combined_content.clone(),
            plugin: None,
            file_path: Some("docker-container.log".to_string()),
            chunk_size: None,
        };

        let chain_result = chain_manager.process(&combined_content, &request);

        assert!(chain_result.is_ok(), "Docker chain processing should succeed: {:?}", chain_result);

        let chain_parse_result = chain_result.unwrap();
        println!("✅ Docker chain processing successful!");

        // Analyze GC log processing specifically
        let mut gc_logs_processed = 0;
        let mut springboot_logs_processed = 0;
        let mut mybatis_logs_processed = 0;

        println!("🔍 Analyzing {} parsed lines:", chain_parse_result.lines.len());
        for (i, line) in chain_parse_result.lines.iter().enumerate() {
            println!("  Line {}: {}", i+1, &line.content[..std::cmp::min(100, line.content.len())]);
            println!("    Level: {:?}", line.level);
            println!("    Timestamp: {:?}", line.timestamp);
            println!("    Processed by: {:?}", line.processed_by);

            if let Some(formatted) = &line.formatted_content {
                println!("    Formatted: {}", &formatted[..std::cmp::min(150, formatted.len())]);
            }

            if !line.metadata.is_empty() {
                println!("    Metadata: {:?}", line.metadata);
            }
            println!();

            if let Some(level) = &line.level {
                if line.content.contains("[gc]") || line.content.contains("[warning][gc]") {
                    gc_logs_processed += 1;
                    println!("🔍 GC Log Found - Level: {}, Original: {}",
                            level, &line.content[..std::cmp::min(80, line.content.len())]);

                    // Verify that GC logs have proper log levels
                    assert!(!level.is_empty(), "GC log should have a log level");
                }
            }

            if line.content.contains("Application") || line.content.contains("com.example") {
                springboot_logs_processed += 1;
                println!("🌱 SpringBoot Log Found - Level: {:?}", line.level);
            }

            if line.content.contains("Preparing:") || line.content.contains("Parameters:") {
                mybatis_logs_processed += 1;
                println!("💾 MyBatis SQL Log Found - Level: {:?}", line.level);
            }
        }

        println!("📈 Log Type Analysis:");
        println!("  - GC logs processed: {}", gc_logs_processed);
        println!("  - SpringBoot logs processed: {}", springboot_logs_processed);
        println!("  - MyBatis logs processed: {}", mybatis_logs_processed);

        // Verify that GC logs were properly processed
        assert!(gc_logs_processed > 0, "GC logs should be processed by JavaLogFilter");

        // Test 3: Individual Filter Behavior
        println!("🔍 Test 3: Individual Filter Behavior");

        // Test DockerJsonFilter
        let docker_filter = DockerJsonFilter;
        let docker_content = r#"{"log":"[0.000s][warning][gc] Test message\n","stream":"stdout","time":"2025-09-16T08:17:32.172897326Z"}"#;
        assert!(docker_filter.can_handle(docker_content, None), "DockerJsonFilter should handle Docker JSON content");

        // Test JavaLogFilter
        let java_filter = JavaLogFilter;
        let gc_content = "[0.000s][warning][gc] -XX:+PrintGCDetails is deprecated. Will use -Xlog:gc* instead.";
        assert!(java_filter.can_handle(gc_content, None), "JavaLogFilter should handle GC logs");

        let java_info_content = "[0.002s][info ][gc,init] CardTable entry size: 512";
        assert!(java_filter.can_handle(java_info_content, None), "JavaLogFilter should handle Java info logs");

        // Test SpringBootFilter
        let springboot_filter = SpringBootFilter;
        let springboot_content = "2025-01-15 10:30:25.123 [main] INFO com.example.Application - Application started";
        assert!(springboot_filter.can_handle(springboot_content, None), "SpringBootFilter should handle SpringBoot logs");

        // Test MyBatisFilter
        let mybatis_filter = MyBatisFilter;
        let mybatis_content = "DEBUG - ==>  Preparing: SELECT * FROM users WHERE id = ?";
        assert!(mybatis_filter.can_handle(mybatis_content, None), "MyBatisFilter should handle MyBatis logs");

        println!("✅ All individual filter tests passed");
        println!("🎯 Integration test completed successfully!");
    }

    #[tokio::test]
    async fn test_chain_recommendation_for_docker_logs() {
        let enhanced_manager = EnhancedPluginManager::new();
        enhanced_manager.initialize().await.unwrap();

        let docker_content = r#"{"log":"[0.000s][warning][gc] Test message\n","stream":"stdout","time":"2025-09-16T08:17:32.172897326Z"}"#;

        let recommended = enhanced_manager.recommend_chain(docker_content, Some("container.log"));

        assert!(recommended.is_some(), "Should recommend a chain for Docker logs");
        assert_eq!(recommended.unwrap(), "docker", "Should recommend docker chain for Docker JSON logs");
    }

    #[tokio::test]
    async fn test_gc_log_level_detection() {
        let enhanced_manager = EnhancedPluginManager::new();
        enhanced_manager.initialize().await.unwrap();

        // Test GC warning log
        let gc_warning_content = "[0.000s][warning][gc] -XX:+PrintGCDetails is deprecated.";
        let request = ParseRequest {
            content: gc_warning_content.to_string(),
            plugin: None,
            file_path: Some("gc.log".to_string()),
            chunk_size: None,
        };

        let result = enhanced_manager.auto_detect_and_parse(&request);
        assert!(result.is_ok(), "Should process GC warning log");

        let parse_result = result.unwrap();
        if let Some(first_line) = parse_result.lines.first() {
            if let Some(level) = &first_line.level {
                assert!(level.contains("WARN") || level.contains("WARNING"),
                        "GC warning should be detected as WARN level, got: {}", level);
            }
        }

        // Test GC info log
        let gc_info_content = "[0.002s][info ][gc,init] CardTable entry size: 512";
        let request = ParseRequest {
            content: gc_info_content.to_string(),
            plugin: None,
            file_path: Some("gc.log".to_string()),
            chunk_size: None,
        };

        let result = enhanced_manager.auto_detect_and_parse(&request);
        assert!(result.is_ok(), "Should process GC info log");

        let parse_result = result.unwrap();
        if let Some(first_line) = parse_result.lines.first() {
            if let Some(level) = &first_line.level {
                assert!(level.contains("INFO"),
                        "GC info should be detected as INFO level, got: {}", level);
            }
        }
    }
}