pub mod theme;
pub mod parse;
pub mod plugin;
pub mod window;
pub mod storage;

use serde::{Deserialize, Serialize};

// Re-export commonly used types
pub use theme::{ThemeConfig, ThemeMode};
pub use parse::ParseConfig;
pub use plugin::PluginConfig;
pub use window::WindowConfig;
pub use storage::{ConfigType};

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
    storage: storage::simple::SimpleConfigStorage,
    config: AppConfig,
    db_path: std::path::PathBuf,
}

impl ConfigService {
    pub fn new<P: AsRef<std::path::Path>>(db_path: P) -> Result<Self, String> {
        let storage = storage::simple::SimpleConfigStorage::new(&db_path)
            .map_err(|e| format!("Failed to initialize config storage: {}", e))?;

        let mut service = Self {
            storage,
            config: AppConfig::default(),
            db_path: db_path.as_ref().to_path_buf(),
        };

        // Load existing configs from database
        service.load_all_configs().map_err(|e| {
            log::warn!("Failed to load configs from database: {}. Using defaults.", e);
            e
        }).ok();

        Ok(service)
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    pub fn update_config(&mut self, request: ConfigUpdateRequest) -> Result<(), String> {
        if let Some(theme) = request.theme {
            self.config.theme = theme.clone();
            self.save_theme_config(&theme)?;
        }
        if let Some(parse) = request.parse {
            self.config.parse = parse.clone();
            self.save_parse_config(&parse)?;
        }
        if let Some(plugin) = request.plugin {
            self.config.plugin = plugin.clone();
            self.save_plugin_config(&plugin)?;
        }
        if let Some(window) = request.window {
            self.config.window = window.clone();
            self.save_window_config(&window)?;
        }
        Ok(())
    }

    // Load all configs from database
    fn load_all_configs(&mut self) -> Result<(), String> {
        // Load theme config
        if let Some(value) = self.storage.get_config("theme.main")
            .map_err(|e| format!("Failed to load theme config: {}", e))?
        {
            self.config.theme = serde_json::from_str(&value)
                .map_err(|e| format!("Failed to parse theme config: {}", e))?;
        }

        // Load parse config
        if let Some(value) = self.storage.get_config("parse.main")
            .map_err(|e| format!("Failed to load parse config: {}", e))?
        {
            self.config.parse = serde_json::from_str(&value)
                .map_err(|e| format!("Failed to parse parse config: {}", e))?;
        }

        // Load plugin config
        if let Some(value) = self.storage.get_config("plugin.main")
            .map_err(|e| format!("Failed to load plugin config: {}", e))?
        {
            self.config.plugin = serde_json::from_str(&value)
                .map_err(|e| format!("Failed to parse plugin config: {}", e))?;
        }

        // Load window config
        if let Some(value) = self.storage.get_config("window.main")
            .map_err(|e| format!("Failed to load window config: {}", e))?
        {
            self.config.window = serde_json::from_str(&value)
                .map_err(|e| format!("Failed to parse window config: {}", e))?;
        }

        log::info!("✅ Loaded all configurations from database");
        Ok(())
    }

    // Save methods
    fn save_theme_config(&mut self, theme: &ThemeConfig) -> Result<(), String> {
        let value = serde_json::to_string(theme)
            .map_err(|e| format!("Failed to serialize theme config: {}", e))?;

        self.storage.set_config("theme.main", &value, ConfigType::Theme)
            .map_err(|e| format!("Failed to save theme config: {}", e))?;

        log::info!("✅ Theme config saved to database");
        Ok(())
    }

    fn save_parse_config(&mut self, parse: &ParseConfig) -> Result<(), String> {
        let value = serde_json::to_string(parse)
            .map_err(|e| format!("Failed to serialize parse config: {}", e))?;

        self.storage.set_config("parse.main", &value, ConfigType::Parse)
            .map_err(|e| format!("Failed to save parse config: {}", e))?;

        log::info!("✅ Parse config saved to database");
        Ok(())
    }

    fn save_plugin_config(&mut self, plugin: &PluginConfig) -> Result<(), String> {
        let value = serde_json::to_string(plugin)
            .map_err(|e| format!("Failed to serialize plugin config: {}", e))?;

        self.storage.set_config("plugin.main", &value, ConfigType::Plugin)
            .map_err(|e| format!("Failed to save plugin config: {}", e))?;

        log::info!("✅ Plugin config saved to database");
        Ok(())
    }

    fn save_window_config(&mut self, window: &WindowConfig) -> Result<(), String> {
        let value = serde_json::to_string(window)
            .map_err(|e| format!("Failed to serialize window config: {}", e))?;

        self.storage.set_config("window.main", &value, ConfigType::Window)
            .map_err(|e| format!("Failed to save window config: {}", e))?;

        log::info!("✅ Window config saved to database");
        Ok(())
    }

    // Public methods for Tauri commands
    pub fn get_theme_config(&self) -> Result<ThemeConfig, String> {
        Ok(self.config.theme.clone())
    }

    pub fn set_theme_config(&mut self, theme: &ThemeConfig) -> Result<(), String> {
        self.config.theme = theme.clone();
        self.save_theme_config(theme)?;
        Ok(())
    }

    pub fn get_parse_config(&self) -> Result<ParseConfig, String> {
        Ok(self.config.parse.clone())
    }

    pub fn set_parse_config(&mut self, parse: &ParseConfig) -> Result<(), String> {
        self.config.parse = parse.clone();
        self.save_parse_config(parse)?;
        Ok(())
    }

    pub fn get_plugin_config(&self) -> Result<PluginConfig, String> {
        Ok(self.config.plugin.clone())
    }

    pub fn set_plugin_config(&mut self, plugin: &PluginConfig) -> Result<(), String> {
        self.config.plugin = plugin.clone();
        self.save_plugin_config(plugin)?;
        Ok(())
    }

    pub fn get_window_config(&self) -> Result<WindowConfig, String> {
        Ok(self.config.window.clone())
    }

    pub fn set_window_config(&mut self, window: &WindowConfig) -> Result<(), String> {
        self.config.window = window.clone();
        self.save_window_config(window)?;
        Ok(())
    }

    pub fn get_all_configs(&self) -> Result<AppConfig, String> {
        Ok(self.config.clone())
    }

    pub fn update_configs(&mut self, request: ConfigUpdateRequest) -> Result<(), String> {
        self.update_config(request)
    }

    // Reopen database connection
    pub fn reload_config(&mut self) -> Result<(), String> {
        let storage = storage::simple::SimpleConfigStorage::new(&self.db_path)
            .map_err(|e| format!("Failed to reopen config storage: {}", e))?;

        self.storage = storage;
        self.config = AppConfig::default();
        self.load_all_configs()?;
        Ok(())
    }

    // Database management methods
    pub fn backup_configs(&self) -> Result<Vec<u8>, String> {
        // Simple backup: export configs as JSON
        let configs = serde_json::to_string(&self.config)
            .map_err(|e| format!("Failed to serialize configs for backup: {}", e))?;

        Ok(configs.into_bytes())
    }

    pub fn reset_configs(&mut self) -> Result<(), String> {
        self.storage.clear_all()
            .map_err(|e| format!("Failed to clear configs: {}", e))?;

        self.config = AppConfig::default();
        log::info!("✅ All configurations reset to defaults");
        Ok(())
    }
}

impl Default for ConfigService {
    fn default() -> Self {
        Self::new("config.db").expect("Failed to create default config service")
    }
}