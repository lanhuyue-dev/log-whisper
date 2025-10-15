// Additional debug tests for chain selection issues

#[cfg(test)]
mod debug_tests2 {
    use crate::plugins::chain::PluginChainManager;
    use crate::plugins::presets::register_preset_chains;
    use crate::plugins::{ParseRequest};

    #[tokio::test]
    async fn debug_chain_conditions() {
        env_logger::init();

        println!("🔍 Testing chain conditions matching");

        // Create chain manager
        let mut chain_manager = PluginChainManager::new();
        register_preset_chains(&mut chain_manager);

        let docker_content = r#"{"log":"[0.000s][warning][gc] -XX:+PrintGCDetails is deprecated. Will use -Xlog:gc* instead.\n","stream":"stdout","time":"2025-09-16T08:17:32.172897326Z"}"#;

        // Test different file paths
        let test_paths = vec![
            Some("container.log"),
            Some("docker-container.log"),
            Some("docker.log"),
            Some("some-other-file.log"),
            None
        ];

        for path in test_paths {
            println!("\nTesting path: {:?}", path);
            let selected_chain = chain_manager.select_best_chain(docker_content, path);
            match selected_chain {
                Some(chain) => {
                    println!("  Selected chain: {}", chain.name);

                    // Test conditions if they exist
                    if let Some(conditions) = &chain.conditions {
                        println!("  Chain conditions:");
                        println!("    File patterns: {:?}", conditions.file_patterns);
                        println!("    Content patterns: {:?}", conditions.content_patterns);
                        println!("    Min confidence: {}", conditions.min_confidence);

                        let matches = conditions.matches(docker_content, path);
                        println!("    Conditions match: {}", matches);

                        // Test individual pattern matching
                        if let Some(p) = path {
                            let path_lower = p.to_lowercase();
                            for pattern in &conditions.file_patterns {
                                let pattern_matches = path_lower.contains(&pattern.to_lowercase());
                                println!("    File pattern '{}' matches: {}", pattern, pattern_matches);
                            }
                        }

                        let content_lower = docker_content.to_lowercase();
                        for pattern in &conditions.content_patterns {
                            let pattern_matches = content_lower.contains(&pattern.to_lowercase());
                            println!("    Content pattern '{}' matches: {}", pattern, pattern_matches);
                        }
                    }
                }
                None => {
                    println!("  No chain selected");
                }
            }
        }
    }

    #[tokio::test]
    async fn debug_docker_chain_direct() {
        env_logger::init();

        println!("🐳 Testing Docker chain direct execution");

        // Create chain manager
        let mut chain_manager = PluginChainManager::new();
        register_preset_chains(&mut chain_manager);

        let docker_content = r#"{"log":"[0.000s][warning][gc] -XX:+PrintGCDetails is deprecated. Will use -Xlog:gc* instead.\n","stream":"stdout","time":"2025-09-16T08:17:32.172897326Z"}"#;

        // Get docker chain directly by selecting it specifically
        let docker_chain = chain_manager.select_best_chain(docker_content, Some("docker-container.log"));
        if let Some(docker_chain) = docker_chain {
            // Only proceed if it's actually the docker chain
            if docker_chain.name != "docker" {
                println!("❌ Selected chain is not docker: {}", docker_chain.name);
                return;
            }
            println!("Docker chain found:");
            println!("  Name: {}", docker_chain.name);
            println!("  Filters: {}", docker_chain.filters.len());
            println!("  Enabled: {}", docker_chain.enabled);

            if let Some(conditions) = &docker_chain.conditions {
                println!("  Conditions:");
                println!("    File patterns: {:?}", conditions.file_patterns);
                println!("    Content patterns: {:?}", conditions.content_patterns);
                println!("    Min confidence: {}", conditions.min_confidence);

                let matches = conditions.matches(docker_content, Some("container.log"));
                println!("    Matches 'container.log': {}", matches);

                let matches_docker = conditions.matches(docker_content, Some("docker-container.log"));
                println!("    Matches 'docker-container.log': {}", matches_docker);
            }

            // Try direct processing
            let request = ParseRequest {
                content: docker_content.to_string(),
                plugin: None,
                file_path: Some("docker-container.log".to_string()), // Use path that should match
                chunk_size: None,
            };

            println!("\n🔄 Running direct docker chain processing...");
            let result = docker_chain.process(docker_content, &request);
            match result {
                Ok(parse_result) => {
                    println!("✅ Docker chain processing successful!");
                    println!("  - Total input lines: {}", parse_result.total_lines);
                    println!("  - Parsed lines: {}", parse_result.lines.len());
                    println!("  - Detected format: {:?}", parse_result.detected_format);

                    for (i, line) in parse_result.lines.iter().enumerate() {
                        println!("  Line {}: {}", i+1, line.content);
                        println!("    Level: {:?}", line.level);
                        println!("    Processed by: {:?}", line.processed_by);
                        println!("    Metadata: {:?}", line.metadata);
                    }
                }
                Err(e) => {
                    println!("❌ Docker chain processing failed: {}", e);
                }
            }
        } else {
            println!("❌ Docker chain not found!");
        }
    }

    #[tokio::test]
    async fn debug_enhanced_manager_with_docker_path() {
        env_logger::init();

        println!("🔧 Testing Enhanced Manager with Docker-specific path");

        let enhanced_manager = crate::plugins::core::EnhancedPluginManager::new();
        enhanced_manager.initialize().await.unwrap();

        let docker_content = r#"{"log":"[0.000s][warning][gc] -XX:+PrintGCDetails is deprecated. Will use -Xlog:gc* instead.\n","stream":"stdout","time":"2025-09-16T08:17:32.172897326Z"}"#;

        // Test with different paths
        let test_paths = vec![
            "container.log",
            "docker-container.log",
            "docker.log",
            "application.log"
        ];

        for path in test_paths {
            println!("\nTesting with path: {}", path);

            let request = ParseRequest {
                content: docker_content.to_string(),
                plugin: None,
                file_path: Some(path.to_string()),
                chunk_size: None,
            };

            let recommended = enhanced_manager.recommend_chain(docker_content, Some(path));
            println!("  Recommended chain: {:?}", recommended);

            let result = enhanced_manager.auto_detect_and_parse(&request);
            match result {
                Ok(parse_result) => {
                    println!("  ✅ Processing successful!");
                    println!("    - Detected format: {:?}", parse_result.detected_format);
                    println!("    - Parsed lines: {}", parse_result.lines.len());
                }
                Err(e) => {
                    println!("  ❌ Processing failed: {}", e);
                }
            }
        }
    }
}