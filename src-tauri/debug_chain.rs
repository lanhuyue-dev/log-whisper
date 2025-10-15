// Debug script to understand why plugin chain is not working properly
use log_whisper::plugins::core::EnhancedPluginManager;
use log_whisper::plugins::presets::register_preset_chains;
use log_whisper::plugins::chain::{PluginChainManager};
use log_whisper::plugins::{ParseRequest};

fn main() {
    // Initialize logging
    env_logger::init();

    println!("🔍 Debug Plugin Chain Processing");
    println!("=================================");

    // Test data: Simple Docker JSON log with GC content
    let docker_content = r#"{"log":"[0.000s][warning][gc] -XX:+PrintGCDetails is deprecated. Will use -Xlog:gc* instead.\n","stream":"stdout","time":"2025-09-16T08:17:32.172897326Z"}"#;

    println!("📋 Input:");
    println!("  {}", docker_content);
    println!();

    // Test 1: Try direct chain processing
    println!("🔗 Test 1: Direct Chain Processing");
    println!("-----------------------------------");

    let mut chain_manager = PluginChainManager::new();
    register_preset_chains(&mut chain_manager);

    let available_chains = chain_manager.get_available_chains();
    println!("Available chains: {:?}", available_chains);

    let request = ParseRequest {
        content: docker_content.to_string(),
        plugin: None,
        file_path: Some("docker-container.log".to_string()),
        chunk_size: None,
    };

    // Try to select best chain
    let selected_chain = chain_manager.select_best_chain(&docker_content, Some("docker-container.log"));
    match selected_chain {
        Some(chain) => println!("Selected chain: {}", chain.name),
        None => println!("No chain selected"),
    }

    // Process with chain
    let chain_result = chain_manager.process(&docker_content, &request);
    match &chain_result {
        Ok(result) => {
            println!("✅ Chain processing successful!");
            println!("  - Total lines: {}", result.total_lines);
            println!("  - Parsed lines: {}", result.lines.len());
            println!("  - Detected format: {:?}", result.detected_format);
            println!("  - Errors: {}", result.parsing_errors.len());

            for (i, line) in result.lines.iter().enumerate() {
                println!("  Line {}: {}", i+1, &line.content[..std::cmp::min(100, line.content.len())]);
                println!("    Level: {:?}", line.level);
                println!("    Processed by: {:?}", line.processed_by);
            }
        }
        Err(e) => {
            println!("❌ Chain processing failed: {}", e);
        }
    }

    println!();

    // Test 2: Try enhanced plugin manager
    println!("🔧 Test 2: Enhanced Plugin Manager");
    println!("---------------------------------");

    let rt = tokio::runtime::Runtime::new().unwrap();
    let enhanced_manager = EnhancedPluginManager::new();

    rt.block_on(async {
        if let Err(e) = enhanced_manager.initialize().await {
            println!("❌ Failed to initialize: {}", e);
            return;
        }
    });

    println!("Chain enabled: {}", enhanced_manager.is_chain_enabled());
    let available_chains = enhanced_manager.get_available_chains();
    println!("Available chains: {:?}", available_chains);

    let request = ParseRequest {
        content: docker_content.to_string(),
        plugin: None,
        file_path: Some("container.log".to_string()),
        chunk_size: None,
    };

    let enhanced_result = enhanced_manager.auto_detect_and_parse(&request);
    match &enhanced_result {
        Ok(result) => {
            println!("✅ Enhanced manager successful!");
            println!("  - Total lines: {}", result.total_lines);
            println!("  - Parsed lines: {}", result.lines.len());
            println!("  - Detected format: {:?}", result.detected_format);
            println!("  - Errors: {}", result.parsing_errors.len());

            for (i, line) in result.lines.iter().enumerate() {
                println!("  Line {}: {}", i+1, &line.content[..std::cmp::min(100, line.content.len())]);
                println!("    Level: {:?}", line.level);
                println!("    Processed by: {:?}", line.processed_by);
            }
        }
        Err(e) => {
            println!("❌ Enhanced manager failed: {}", e);
        }
    }

    // Test 3: Try chain recommendation
    println!();
    println!("💡 Test 3: Chain Recommendation");
    println!("------------------------------");

    let recommended = enhanced_manager.recommend_chain(&docker_content, Some("container.log"));
    println!("Recommended chain: {:?}", recommended);
}