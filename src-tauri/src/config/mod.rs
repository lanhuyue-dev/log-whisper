pub mod theme;
pub mod parse;
pub mod plugin;
pub mod window;

use serde::{Deserialize, Serialize};

// Re-export commonly used types
pub use theme::{ThemeConfig, ThemeMode};
pub use parse::ParseConfig;
pub use plugin::PluginConfig;
pub use window::WindowConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemeConfig,
    pub parse: ParseConfig,
    pub plugin: PluginConfig,
    pub window: WindowConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeConfig::default(),
            parse: ParseConfig::default(),
            plugin: PluginConfig::default(),
            window: WindowConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigUpdateRequest {
    pub theme: Option<ThemeConfig>,
    pub parse: Option<ParseConfig>,
    pub plugin: Option<PluginConfig>,
    pub window: Option<WindowConfig>,
}

pub struct ConfigService {
    config: AppConfig,
}

impl ConfigService {
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    pub fn update_config(&mut self, request: ConfigUpdateRequest) -> Result<(), String> {
        if let Some(theme) = request.theme {
            self.config.theme = theme;
        }
        if let Some(parse) = request.parse {
            self.config.parse = parse;
        }
        if let Some(plugin) = request.plugin {
            self.config.plugin = plugin;
        }
        if let Some(window) = request.window {
            self.config.window = window;
        }
        Ok(())
    }

    // TODO: Implement async config methods for database persistence
    pub async fn get_theme_config(&self) -> Result<ThemeConfig, String> {
        Ok(self.config.theme.clone())
    }

    pub async fn set_theme_config(&self, theme: &ThemeConfig) -> Result<(), String> {
        // TODO: Implement persistent storage
        log::info!("Theme config updated (not persisted): {:?}", theme);
        Ok(())
    }

    pub async fn get_parse_config(&self) -> Result<ParseConfig, String> {
        Ok(self.config.parse.clone())
    }

    pub async fn get_plugin_config(&self) -> Result<PluginConfig, String> {
        Ok(self.config.plugin.clone())
    }

    pub async fn get_window_config(&self) -> Result<WindowConfig, String> {
        Ok(self.config.window.clone())
    }

    pub async fn get_all_configs(&self) -> Result<AppConfig, String> {
        Ok(self.config.clone())
    }
}

impl Default for ConfigService {
    fn default() -> Self {
        Self::new()
    }
}