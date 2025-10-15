#[cfg(test)]
mod tests {
    use crate::plugins::filters::SpringBootFilter;
    use crate::plugins::chain::{PluginFilter, PluginChainContext};
    use crate::plugins::ParseRequest;
    use std::collections::HashMap;

    #[test]
    fn test_application_log_format() {
        let filter = SpringBootFilter;
        let log_content = r#"2025-10-15T07:40:55.169Z  INFO 1 --- [  EventHandler1] s.i.ProjectAttributeTemplateEventSpiImpl : ProjectAttributeTemplateEventSpiImpl 收到事件，objectName:Document number:136"#;

        let mut context = PluginChainContext {
            original_content: log_content.to_string(),
            current_lines: vec![],
            chain_id: "test".to_string(),
            metadata: HashMap::new(),
        };

        let request = ParseRequest {
            content: log_content.to_string(),
            file_path: None,
            plugin_name: "springboot".to_string(),
            chain_name: Some("springboot".to_string()),
        };

        // Test should_process
        assert!(filter.should_process(&context));

        // Test process
        let result = filter.process(&mut context, &request);
        assert!(result.is_ok());

        // Verify the processed line
        assert_eq!(context.current_lines.len(), 1);
        let line = &context.current_lines[0];

        // Check that regex matched and extracted fields correctly
        assert!(line.timestamp.is_some());
        assert!(line.level.is_some());
        assert!(line.metadata.contains_key("thread"));
        assert!(line.metadata.contains_key("logger"));

        println!("✅ Test passed!");
        println!("  Timestamp: {:?}", line.timestamp);
        println!("  Level: {:?}", line.level);
        println!("  Thread: {:?}", line.metadata.get("thread"));
        println!("  Logger: {:?}", line.metadata.get("logger"));
        println!("  Content: {}", line.content);
    }

    #[test]
    fn test_regex_pattern_directly() {
        use regex::Regex;

        let pattern = r"^(\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}[.,]\d{3}(?:Z)?)\s+([A-Z]+)\s+(?:\d+\s+---\s+)?\[\s*([^\]]+)\s*\]\s+([^\s:]+)\s*:\s*(.*)$";
        let regex = Regex::new(pattern).unwrap();

        let test_line = "2025-10-15T07:40:55.169Z  INFO 1 --- [  EventHandler1] s.i.ProjectAttributeTemplateEventSpiImpl : ProjectAttributeTemplateEventSpiImpl 收到事件，objectName:Document number:136";

        if let Some(captures) = regex.captures(test_line) {
            println!("✅ Regex匹配成功!");
            println!("  捕获组数量: {}", captures.len());
            for (i, cap) in captures.iter().enumerate() {
                if let Some(group) = cap {
                    println!("  捕获组 {}: '{}'", i, group.as_str());
                }
            }

            assert_eq!(captures.len(), 6); // 0 + 5 capture groups
            assert_eq!(captures.get(1).unwrap().as_str(), "2025-10-15T07:40:55.169Z");
            assert_eq!(captures.get(2).unwrap().as_str(), "INFO");
            assert_eq!(captures.get(3).unwrap().as_str(), "EventHandler1");
            assert_eq!(captures.get(4).unwrap().as_str(), "s.i.ProjectAttributeTemplateEventSpiImpl");
            assert_eq!(captures.get(5).unwrap().as_str(), "ProjectAttributeTemplateEventSpiImpl 收到事件，objectName:Document number:136");
        } else {
            panic!("❌ Regex匹配失败");
        }
    }
}