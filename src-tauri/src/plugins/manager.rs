use crate::plugins::{LogParser, PluginInfo, ParseRequest, ParseResult};
use std::collections::HashMap;
use std::sync::Arc;

pub struct PluginManager {
    parsers: HashMap<String, Arc<dyn LogParser + Send + Sync>>,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut parsers: HashMap<String, Arc<dyn LogParser + Send + Sync>> = HashMap::new();

        // 注册内置解析器
        parsers.insert("auto".to_string(), Arc::new(crate::plugins::auto::AutoParser));
        parsers.insert("mybatis".to_string(), Arc::new(crate::plugins::mybatis::MyBatisParser));
        parsers.insert("docker_json".to_string(), Arc::new(crate::plugins::docker_json::DockerJsonParser));
        parsers.insert("raw".to_string(), Arc::new(crate::plugins::raw::RawParser));
        parsers.insert("springboot".to_string(), Arc::new(crate::plugins::springboot::SpringBootParser));

        Self { parsers }
    }

    pub fn get_available_plugins(&self) -> Vec<PluginInfo> {
        self.parsers.values().map(|parser| {
            PluginInfo {
                name: parser.name().to_string(),
                description: parser.description().to_string(),
                supported_extensions: parser.supported_extensions(),
                auto_detectable: true,
            }
        }).collect()
    }

    pub fn parse_with_plugin(&self, plugin_name: &str, request: &ParseRequest) -> Result<ParseResult, String> {
        let parser = self.parsers.get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;

        parser.parse(&request.content, request)
    }

    pub fn auto_detect_and_parse(&self, request: &ParseRequest) -> Result<ParseResult, String> {
        // 尝试自动检测最适合的解析器
        let content = &request.content;
        let file_path = request.file_path.as_deref();

        // 首先尝试特定的解析器
        for (name, parser) in &self.parsers {
            if name != "auto" && name != "raw" && parser.can_parse(content, file_path) {
                let mut result = parser.parse(content, request)?;
                result.detected_format = Some(name.clone());
                return Ok(result);
            }
        }

        // 如果没有特定的解析器匹配，使用 auto 解析器
        if let Some(auto_parser) = self.parsers.get("auto") {
            let mut result = auto_parser.parse(content, request)?;
            result.detected_format = Some("auto".to_string());
            return Ok(result);
        }

        // 最后使用 raw 解析器
        if let Some(raw_parser) = self.parsers.get("raw") {
            let mut result = raw_parser.parse(content, request)?;
            result.detected_format = Some("raw".to_string());
            return Ok(result);
        }

        Err("No suitable parser found".to_string())
    }

    pub fn get_plugin(&self, name: &str) -> Option<&Arc<dyn LogParser + Send + Sync>> {
        self.parsers.get(name)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}