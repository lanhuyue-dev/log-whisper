//! 插件系统演示示例
//! 
//! 展示如何使用增强的插件系统处理各种日志格式

use log_whisper_api::plugins::{
    EnhancedPluginManager, 
    LogEntry, 
    PluginLogEntry,
    builtin::*,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    env_logger::init();
    
    println!("🚀 LogWhisper 插件系统演示");
    println!("================================");
    
    // 创建插件管理器
    let plugin_manager = EnhancedPluginManager::new();
    
    // 初始化插件系统
    plugin_manager.initialize().await?;
    println!("✅ 插件系统初始化完成");
    
    // 获取插件统计信息
    let stats = plugin_manager.get_plugin_stats().await;
    println!("📊 插件统计信息:");
    println!("  - 总插件数: {}", stats.total_plugins);
    println!("  - 启用插件数: {}", stats.enabled_plugins);
    println!("  - 插件类型分布: {:?}", stats.plugin_types);
    
    // 准备测试日志数据
    let test_logs = vec![
        // Docker JSON 格式日志
        r#"{"log":"[0.008s][info   ][gc,init] Memory: 15193M\n","stream":"stdout","time":"2025-09-16T08:17:32.180873992Z"}"#,
        r#"{"log":"ERROR: Failed to connect to database\n","stream":"stderr","time":"2025-09-16T08:17:34.180880085Z"}"#,
        
        // SpringBoot 标准日志
        "2025-10-04 09:15:01.123 INFO  [http-nio-8080-exec-1] com.example.controller.UserController : 用户登录请求开始",
        "2025-10-04 09:15:01.125 DEBUG [http-nio-8080-exec-1] com.example.service.UserService : 验证用户凭据: {\"username\": \"admin\", \"loginTime\": \"2025-10-04T09:15:01Z\"}",
        
        // 异常堆栈跟踪
        "2025-10-04 09:15:05.123 ERROR [http-nio-8080-exec-2] com.example.controller.ApiController : 处理请求时发生错误",
        "java.lang.NullPointerException: Cannot invoke \"String.trim()\" because \"input\" is null",
        "    at com.example.service.ValidationService.validateInput(ValidationService.java:45)",
        "    at com.example.controller.ApiController.processRequest(ApiController.java:78)",
        "    at java.base/java.lang.reflect.Method.invoke(Method.java:566)",
        
        // MyBatis SQL 日志
        "Preparing: SELECT * FROM users WHERE username = ? AND password = ?",
        "Parameters: admin(String), password123(String)",
        "Total: 1",
        
        // JSON 数据日志
        "2025-10-04 09:15:01.156 INFO  [http-nio-8080-exec-1] com.example.service.UserService : 用户认证成功: {\"userId\": 1001, \"username\": \"admin\", \"roles\": [\"ADMIN\", \"USER\"]}",
    ];
    
    println!("\n📝 开始处理测试日志...");
    println!("日志条目数: {}", test_logs.len());
    
    // 转换为插件日志条目
    let mut plugin_entries = Vec::new();
    for (index, log) in test_logs.iter().enumerate() {
        plugin_entries.push(PluginLogEntry::new(index + 1, log.to_string()));
    }
    
    // 使用插件系统处理日志
    let start_time = std::time::Instant::now();
    let processed_entries = plugin_manager.process_log_entries(plugin_entries).await?;
    let processing_time = start_time.elapsed();
    
    println!("✅ 日志处理完成，耗时: {:?}", processing_time);
    
    // 显示处理结果
    println!("\n📋 处理结果详情:");
    println!("==================");
    
    for (i, entry) in processed_entries.iter().enumerate() {
        println!("\n条目 {}: 行号 {}", i + 1, entry.line_number);
        println!("原始内容: {}", entry.content.chars().take(100).collect::<String>());
        
        if let Some(timestamp) = &entry.timestamp {
            println!("时间戳: {}", timestamp);
        }
        
        if let Some(level) = &entry.level {
            println!("日志级别: {}", level);
        }
        
        if !entry.metadata.is_empty() {
            println!("元数据: {:?}", entry.metadata);
        }
        
        if let Some(formatted) = &entry.formatted_content {
            println!("格式化内容: {}", formatted.chars().take(100).collect::<String>());
        }
        
        if !entry.processed_by.is_empty() {
            println!("处理插件: {:?}", entry.processed_by);
        }
        
        println!("---");
    }
    
    // 统计处理结果
    let mut format_stats = std::collections::HashMap::new();
    let mut plugin_stats = std::collections::HashMap::new();
    
    for entry in &processed_entries {
        // 统计格式类型
        if let Some(format_type) = entry.metadata.get("format") {
            *format_stats.entry(format_type.clone()).or_insert(0) += 1;
        }
        
        // 统计插件使用
        for plugin in &entry.processed_by {
            *plugin_stats.entry(plugin.clone()).or_insert(0) += 1;
        }
    }
    
    println!("\n📊 处理统计:");
    println!("格式类型分布: {:?}", format_stats);
    println!("插件使用统计: {:?}", plugin_stats);
    
    // 清理插件系统
    plugin_manager.cleanup().await?;
    println!("\n✅ 插件系统清理完成");
    
    Ok(())
}
