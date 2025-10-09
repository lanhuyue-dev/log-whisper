use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled_plugins: Vec<String>,
    pub plugin_settings: HashMap<String, serde_json::Value>,
    pub auto_update: bool,
    pub enable_notifications: bool,
    pub plugin_directory: String,
    pub max_plugins: usize,
}

impl Default for PluginConfig {
    fn default() -> Self {
        let mut settings = HashMap::new();
        settings.insert("mybatis".to_string(), serde_json::json!({
            "extract_params": true,
            "format_sql": true
        }));

        Self {
            enabled_plugins: vec!["auto".to_string(), "mybatis".to_string(), "docker_json".to_string(), "raw".to_string()],
            plugin_settings: settings,
            auto_update: false,
            enable_notifications: true,
            plugin_directory: "plugins".to_string(),
            max_plugins: 50,
        }
    }
}