//! SpringBoot 日志处理演示
//! 
//! 展示增强插件系统如何处理SpringBoot应用的各种日志格式

use log_whisper_api::plugins::{
    EnhancedPluginManager, 
    LogEntry, 
    PluginLogEntry,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    env_logger::init();
    
    println!("🚀 SpringBoot 日志处理演示");
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
    
    // 准备SpringBoot测试日志数据
    let springboot_logs = vec![
        // Docker JSON 格式的SpringBoot日志
        r#"{"log":"2025-10-04 09:15:01.123 INFO  [http-nio-8080-exec-1] com.example.controller.UserController : 用户登录请求开始\n","stream":"stdout","time":"2025-09-16T08:17:32.180873992Z"}"#,
        r#"{"log":"2025-10-04 09:15:01.125 DEBUG [http-nio-8080-exec-1] com.example.service.UserService : 验证用户凭据: {\"username\": \"admin\", \"loginTime\": \"2025-10-04T09:15:01Z\"}\n","stream":"stdout","time":"2025-09-16T08:17:33.180873992Z"}"#,
        
        // 标准SpringBoot日志
        "2025-10-04 09:15:01.140 INFO  [http-nio-8080-exec-1] org.springframework.jdbc.core.JdbcTemplate : Executing prepared SQL statement",
        "2025-10-04 09:15:01.142 DEBUG [http-nio-8080-exec-1] org.mybatis.spring.SqlSessionUtils : Creating a new SqlSession",
        
        // MyBatis SQL 日志
        "Preparing: SELECT * FROM users WHERE username = ? AND password = ?",
        "Parameters: admin(String), password123(String)",
        "Total: 1",
        
        // 异常堆栈跟踪
        "2025-10-04 09:15:05.123 ERROR [http-nio-8080-exec-2] com.example.controller.ApiController : 处理请求时发生错误",
        "java.lang.NullPointerException: Cannot invoke \"String.trim()\" because \"input\" is null",
        "    at com.example.service.ValidationService.validateInput(ValidationService.java:45)",
        "    at com.example.controller.ApiController.processRequest(ApiController.java:78)",
        "    at java.base/java.lang.reflect.Method.invoke(Method.java:566)",
        "    at org.springframework.web.method.support.InvocableHandlerMethod.doInvoke(InvocableHandlerMethod.java:205)",
        "    at org.springframework.web.method.support.InvocableHandlerMethod.invokeForRequest(InvocableHandlerMethod.java:150)",
        "    at org.springframework.web.servlet.mvc.method.annotation.ServletInvocableHandlerMethod.invokeAndHandle(ServletInvocableHandlerMethod.java:117)",
        
        // JSON 数据日志
        "2025-10-04 09:15:01.156 INFO  [http-nio-8080-exec-1] com.example.service.UserService : 用户认证成功: {\"userId\": 1001, \"username\": \"admin\", \"roles\": [\"ADMIN\", \"USER\"]}",
        
        // 警告日志
        "2025-10-04 09:15:02.001 WARN  [security-thread-1] com.example.security.SecurityMonitor : 检测到可疑登录行为: {\"ip\": \"192.168.1.100\", \"attempts\": 3, \"status\": \"SUSPICIOUS\"}",
    ];
    
    println!("\n📝 开始处理SpringBoot日志...");
    println!("日志条目数: {}", springboot_logs.len());
    
    // 转换为插件日志条目
    let mut plugin_entries = Vec::new();
    for (index, log) in springboot_logs.iter().enumerate() {
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
            println!("格式化内容: {}", formatted.chars().take(200).collect::<String>());
        }
        
        if !entry.processed_by.is_empty() {
            println!("处理插件: {:?}", entry.processed_by);
        }
        
        println!("---");
    }
    
    // 统计处理结果
    let mut format_stats = std::collections::HashMap::new();
    let mut plugin_stats = std::collections::HashMap::new();
    let mut renderer_stats = std::collections::HashMap::new();
    
    for entry in &processed_entries {
        // 统计格式类型
        if let Some(format_type) = entry.metadata.get("format") {
            *format_stats.entry(format_type.clone()).or_insert(0) += 1;
        }
        
        // 统计插件使用
        for plugin in &entry.processed_by {
            *plugin_stats.entry(plugin.clone()).or_insert(0) += 1;
        }
        
        // 统计渲染器使用
        if let Some(renderer) = entry.metadata.get("renderer") {
            *renderer_stats.entry(renderer.clone()).or_insert(0) += 1;
        }
    }
    
    println!("\n📊 处理统计:");
    println!("格式类型分布: {:?}", format_stats);
    println!("插件使用统计: {:?}", plugin_stats);
    println!("渲染器使用统计: {:?}", renderer_stats);
    
    // 演示特定功能
    println!("\n🔍 功能演示:");
    
    // 演示JSON格式处理
    let json_entries: Vec<_> = processed_entries.iter()
        .filter(|e| e.metadata.get("format") == Some(&"springboot_json".to_string()))
        .collect();
    println!("JSON格式日志处理: {} 条", json_entries.len());
    
    // 演示异常聚合
    let exception_entries: Vec<_> = processed_entries.iter()
        .filter(|e| e.metadata.get("exception_type").is_some())
        .collect();
    println!("异常日志聚合: {} 条", exception_entries.len());
    
    // 演示SQL日志处理
    let sql_entries: Vec<_> = processed_entries.iter()
        .filter(|e| e.metadata.get("format") == Some(&"mybatis".to_string()))
        .collect();
    println!("SQL日志处理: {} 条", sql_entries.len());
    
    // 演示渲染功能
    let rendered_entries: Vec<_> = processed_entries.iter()
        .filter(|e| e.formatted_content.is_some())
        .collect();
    println!("渲染处理: {} 条", rendered_entries.len());
    
    // 清理插件系统
    plugin_manager.cleanup().await?;
    println!("\n✅ 插件系统清理完成");
    
    Ok(())
}
