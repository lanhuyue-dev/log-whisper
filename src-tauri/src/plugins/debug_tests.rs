// Debug test for the plugin chain system

#[cfg(test)]
mod debug_tests {
    use crate::plugins::core::EnhancedPluginManager;
    use crate::plugins::presets::register_preset_chains;
    use crate::plugins::chain::{PluginChainManager};
    use crate::plugins::filters::{DockerJsonFilter, JavaLogFilter};
    use crate::plugins::chain::{PluginFilter, PluginChainContext};
    use crate::plugins::{ParseRequest};

    #[tokio::test]
    async fn debug_docker_json_filter() {
        env_logger::init();

        println!("🐳 Testing DockerJsonFilter directly");

        let docker_filter = DockerJsonFilter;
        let docker_content = r#"{"log":"[0.000s][warning][gc] -XX:+PrintGCDetails is deprecated. Will use -Xlog:gc* instead.\n","stream":"stdout","time":"2025-09-16T08:17:32.172897326Z"}"#;

        println!("Input: {}", docker_content);
        println!("Can handle: {}", docker_filter.can_handle(docker_content, None));

        // Test the filter directly
        let mut context = PluginChainContext::new(docker_content.to_string());
        let request = ParseRequest {
            content: docker_content.to_string(),
            plugin: None,
            file_path: Some("container.log".to_string()),
            chunk_size: None,
        };

        let should_process = docker_filter.should_process(&context);
        println!("Should process: {}", should_process);

        if should_process {
            let result = docker_filter.process(&mut context, &request);
            println!("Process result: {:?}", result);

            if result.is_ok() {
                println!("Current lines after processing: {}", context.current_lines.len());
                for (i, line) in context.current_lines.iter().enumerate() {
                    println!("  Line {}: {}", i+1, line.content);
                    println!("    Level: {:?}", line.level);
                    println!("    Processed by: {:?}", line.processed_by);
                    println!("    Metadata: {:?}", line.metadata);
                }
            }
        }
    }

    #[tokio::test]
    async fn debug_java_log_filter() {
        env_logger::init();

        println!("☕ Testing JavaLogFilter directly");

        let java_filter = JavaLogFilter;
        let gc_content = "[0.000s][warning][gc] -XX:+PrintGCDetails is deprecated. Will use -Xlog:gc* instead.";

        println!("Input: {}", gc_content);
        println!("Can handle: {}", java_filter.can_handle(gc_content, None));

        // Test the filter directly
        let mut context = PluginChainContext::new(gc_content.to_string());
        let request = ParseRequest {
            content: gc_content.to_string(),
            plugin: None,
            file_path: Some("gc.log".to_string()),
            chunk_size: None,
        };

        let should_process = java_filter.should_process(&context);
        println!("Should process: {}", should_process);

        if should_process {
            let result = java_filter.process(&mut context, &request);
            println!("Process result: {:?}", result);

            if result.is_ok() {
                println!("Current lines after processing: {}", context.current_lines.len());
                for (i, line) in context.current_lines.iter().enumerate() {
                    println!("  Line {}: {}", i+1, line.content);
                    println!("    Level: {:?}", line.level);
                    println!("    Processed by: {:?}", line.processed_by);
                    println!("    Metadata: {:?}", line.metadata);
                }
            }
        }
    }

    #[tokio::test]
    async fn debug_chain_execution() {
        env_logger::init();

        println!("🔗 Testing chain execution step by step");

        // Create chain
        let mut chain_manager = PluginChainManager::new();
        register_preset_chains(&mut chain_manager);

        let docker_content = r#"{"log":"[0.000s][warning][gc] -XX:+PrintGCDetails is deprecated. Will use -Xlog:gc* instead.\n","stream":"stdout","time":"2025-09-16T08:17:32.172897326Z"}"#;

        println!("Input content: {}", docker_content);

        // Test chain selection
        let available_chains = chain_manager.get_available_chains();
        println!("Available chains: {:?}", available_chains);

        let selected_chain = chain_manager.select_best_chain(docker_content, Some("container.log"));
        match selected_chain {
            Some(chain) => {
                println!("Selected chain: {}", chain.name);
                println!("Chain description: {}", chain.description);
                println!("Chain has {} filters", chain.filters.len());

                for (i, filter) in chain.filters.iter().enumerate() {
                    println!("  Filter {}: {} (priority: {})", i+1, filter.name(), filter.priority());
                    println!("    Can handle: {}", filter.can_handle(docker_content, Some("container.log")));
                }

                // Test actual processing
                let request = ParseRequest {
                    content: docker_content.to_string(),
                    plugin: None,
                    file_path: Some("container.log".to_string()),
                    chunk_size: None,
                };

                println!("\n🔄 Running chain processing...");
                let result = chain_manager.process(docker_content, &request);
                match result {
                    Ok(parse_result) => {
                        println!("✅ Chain processing successful!");
                        println!("  - Total input lines: {}", parse_result.total_lines);
                        println!("  - Parsed lines: {}", parse_result.lines.len());
                        println!("  - Detected format: {:?}", parse_result.detected_format);

                        for (i, line) in parse_result.lines.iter().enumerate() {
                            println!("  Line {}: {}", i+1, line.content);
                            println!("    Level: {:?}", line.level);
                            println!("    Timestamp: {:?}", line.timestamp);
                            println!("    Processed by: {:?}", line.processed_by);
                            println!("    Metadata: {:?}", line.metadata);
                        }
                    }
                    Err(e) => {
                        println!("❌ Chain processing failed: {}", e);
                    }
                }
            }
            None => {
                println!("❌ No chain selected");
            }
        }
    }

    #[tokio::test]
    async fn debug_enhanced_manager() {
        env_logger::init();

        println!("🔧 Testing Enhanced Plugin Manager");

        let enhanced_manager = EnhancedPluginManager::new();
        enhanced_manager.initialize().await.unwrap();

        let docker_content = r#"{"log":"[0.000s][warning][gc] -XX:+PrintGCDetails is deprecated. Will use -Xlog:gc* instead.\n","stream":"stdout","time":"2025-09-16T08:17:32.172897326Z"}"#;

        println!("Chain enabled: {}", enhanced_manager.is_chain_enabled());

        let available_chains = enhanced_manager.get_available_chains();
        println!("Available chains: {:?}", available_chains);

        let recommended = enhanced_manager.recommend_chain(docker_content, Some("container.log"));
        println!("Recommended chain: {:?}", recommended);

        let request = ParseRequest {
            content: docker_content.to_string(),
            plugin: None,
            file_path: Some("container.log".to_string()),
            chunk_size: None,
        };

        println!("🔄 Running enhanced manager processing...");
        let result = enhanced_manager.auto_detect_and_parse(&request);
        match result {
            Ok(parse_result) => {
                println!("✅ Enhanced manager successful!");
                println!("  - Total input lines: {}", parse_result.total_lines);
                println!("  - Parsed lines: {}", parse_result.lines.len());
                println!("  - Detected format: {:?}", parse_result.detected_format);

                for (i, line) in parse_result.lines.iter().enumerate() {
                    println!("  Line {}: {}", i+1, line.content);
                    println!("    Level: {:?}", line.level);
                    println!("    Timestamp: {:?}", line.timestamp);
                    println!("    Processed by: {:?}", line.processed_by);
                    println!("    Metadata: {:?}", line.metadata);
                }
            }
            Err(e) => {
                println!("❌ Enhanced manager failed: {}", e);
            }
        }
    }
}