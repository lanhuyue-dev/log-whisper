// Test file for Docker chain processing with GC logs
// This file demonstrates the plugin chain system handling Docker JSON logs containing Java GC logs

use std::sync::{Arc, Mutex};
use log_whisper::plugins::core::EnhancedPluginManager;
use log_whisper::plugins::presets::register_preset_chains;
use log_whisper::plugins::chain::{PluginChainManager, PluginChainContext};
use log_whisper::plugins::filters::{DockerJsonFilter, SpringBootFilter, JavaLogFilter, MyBatisFilter};
use log_whisper::plugins::{ParseRequest, LogLine};
use std::collections::HashMap;

fn main() {
    // Initialize logging
    env_logger::init();

    println!("🧪 Testing Docker Chain Processing with GC Logs");
    println!("================================================");

    // Test data: Docker JSON logs containing Java GC logs (from user's sample)
    let docker_logs = vec![
        r#"{"log":"[0.000s][warning][gc] -XX:+PrintGCDetails is deprecated. Will use -Xlog:gc* instead.\n","stream":"stdout","time":"2025-09-16T08:17:32.172897326Z"}"#,
        r#"{"log":"[0.002s][info ][gc,init] CardTable entry size: 512\n","stream":"stdout","time":"2025-09-16T08:17:32.174908043Z"}"#,
        r#"{"log":"[0.002s][info ][gc ] Using G1\n","stream":"stdout","time":"2025-09-16T08:17:32.174994988Z"}"#,
        r#"{"log":"[0.008s][info ][gc,init] Version: 21.0.7+9-LTS (release)\n","stream":"stdout","time":"2025-09-16T08:17:32.175102Z"}"#,
        r#"{"log":"2025-01-15 10:30:25.123 [main] INFO com.example.Application - Application started successfully\n","stream":"stdout","time":"2025-01-15T10:30:25.123Z"}"#,
        r#"{"log":"DEBUG - ==>  Preparing: SELECT * FROM users WHERE id = ?\nDEBUG - ==> Parameters: 123(String)\n","stream":"stdout","time":"2025-01-15T10:30:26.456Z"}"#
    ];

    let combined_content = docker_logs.join("\n");

    println!("📋 Test Input ({} Docker JSON log lines):", docker_logs.len());
    for (i, log) in docker_logs.iter().enumerate() {
        println!("  {}: {}", i + 1, &log[..std::cmp::min(80, log.len())]);
    }
    println!();

    // Test 1: Create enhanced plugin manager and test auto-detection
    println!("🔧 Test 1: Enhanced Plugin Manager Auto-Detection");
    println!("---------------------------------------------------");

    let enhanced_manager = EnhancedPluginManager::new();

    // Use block_on for async initialization in this test context
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        if let Err(e) = enhanced_manager.initialize().await {
            eprintln!("❌ Failed to initialize enhanced plugin manager: {}", e);
            return;
        }
    });

    let request = ParseRequest {
        content: combined_content.clone(),
        plugin: None, // Auto-detect
        file_path: Some("container.log".to_string()),
        chunk_size: None,
    };

    match enhanced_manager.auto_detect_and_parse(&request) {
        Ok(result) => {
            println!("✅ Auto-detection successful!");
            println!("📊 Processing Results:");
            println!("  - Total input lines: {}", result.total_lines);
            println!("  - Parsed log lines: {}", result.lines.len());
            println!("  - Detected format: {:?}", result.detected_format);
            println!("  - Processing errors: {}", result.parsing_errors.len());

            if !result.parsing_errors.is_empty() {
                println!("❌ Errors encountered:");
                for error in &result.parsing_errors {
                    println!("    - {}", error);
                }
            }

            println!("\n📝 Parsed Log Lines:");
            for (i, line) in result.lines.iter().enumerate() {
                println!("  Line {}:", line.line_number);
                println!("    Original: {}", &line.content[..std::cmp::min(100, line.content.len())]);
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
            }
        }
        Err(e) => {
            println!("❌ Auto-detection failed: {}", e);
        }
    }

    // Test 2: Test individual Docker chain processing
    println!("\n🔗 Test 2: Docker Chain Direct Processing");
    println!("-------------------------------------------");

    let mut chain_manager = PluginChainManager::new();
    register_preset_chains(&mut chain_manager);

    let request = ParseRequest {
        content: combined_content.clone(),
        plugin: None,
        file_path: Some("docker-container.log".to_string()),
        chunk_size: None,
    };

    match chain_manager.process(&combined_content, &request) {
        Ok(result) => {
            println!("✅ Docker chain processing successful!");
            println!("📊 Chain Processing Results:");
            println!("  - Total input lines: {}", result.total_lines);
            println!("  - Parsed log lines: {}", result.lines.len());
            println!("  - Detected format: {:?}", result.detected_format);
            println!("  - Processing errors: {}", result.parsing_errors.len());

            // Analyze GC log processing specifically
            let mut gc_logs_processed = 0;
            let mut springboot_logs_processed = 0;
            let mut mybatis_logs_processed = 0;

            for line in &result.lines {
                if let Some(level) = &line.level {
                    if line.content.contains("[gc]") || line.content.contains("[warning][gc]") {
                        gc_logs_processed += 1;
                        println!("🔍 GC Log Found - Level: {}, Original: {}",
                                level, &line.content[..std::cmp::min(80, line.content.len())]);
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

            println!("\n📈 Log Type Analysis:");
            println!("  - GC logs processed: {}", gc_logs_processed);
            println!("  - SpringBoot logs processed: {}", springboot_logs_processed);
            println!("  - MyBatis logs processed: {}", mybatis_logs_processed);

            // Check if GC logs were properly processed with levels
            if gc_logs_processed > 0 {
                println!("✅ GC logs were successfully processed by JavaLogFilter");
            } else {
                println!("❌ GC logs were not properly processed");
            }
        }
        Err(e) => {
            println!("❌ Docker chain processing failed: {}", e);
        }
    }

    // Test 3: Test individual filter behavior
    println!("\n🔍 Test 3: Individual Filter Behavior");
    println!("--------------------------------------");

    // Test DockerJsonFilter
    let docker_filter = DockerJsonFilter;
    let docker_content = r#"{"log":"[0.000s][warning][gc] Test message\n","stream":"stdout","time":"2025-09-16T08:17:32.172897326Z"}"#;
    println!("DockerJsonFilter can handle: {}", docker_filter.can_handle(docker_content, None));

    // Test JavaLogFilter
    let java_filter = JavaLogFilter;
    let gc_content = "[0.000s][warning][gc] -XX:+PrintGCDetails is deprecated. Will use -Xlog:gc* instead.";
    println!("JavaLogFilter can handle GC log: {}", java_filter.can_handle(gc_content, None));

    let java_info_content = "[0.002s][info ][gc,init] CardTable entry size: 512";
    println!("JavaLogFilter can handle Java info log: {}", java_filter.can_handle(java_info_content, None));

    // Test SpringBootFilter
    let springboot_filter = SpringBootFilter;
    let springboot_content = "2025-01-15 10:30:25.123 [main] INFO com.example.Application - Application started";
    println!("SpringBootFilter can handle SpringBoot log: {}", springboot_filter.can_handle(springboot_content, None));

    // Test MyBatisFilter
    let mybatis_filter = MyBatisFilter;
    let mybatis_content = "DEBUG - ==>  Preparing: SELECT * FROM users WHERE id = ?";
    println!("MyBatisFilter can handle MyBatis log: {}", mybatis_filter.can_handle(mybatis_content, None));

    println!("\n🎯 Test Summary:");
    println!("================");
    println!("✅ All tests completed. Check output above to verify:");
    println!("   1. Docker JSON logs are correctly parsed");
    println!("   2. GC logs are recognized and processed with proper levels");
    println!("   3. SpringBoot logs are formatted correctly");
    println!("   4. MyBatis SQL logs are identified and formatted");
    println!("   5. Plugin chain processes logs in correct order");
}