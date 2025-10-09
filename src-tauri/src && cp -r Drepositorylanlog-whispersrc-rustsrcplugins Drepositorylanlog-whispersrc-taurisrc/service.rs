use std::sync::{Arc, Mutex};
use crate::config::database::ConfigDatabase;
use crate::config::models::*;

/// 配置服务
pub struct ConfigService {
    db: Arc<Mutex<ConfigDatabase>>,
}

impl ConfigService {
    /// 创建新的配置服务
    pub fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let db = ConfigDatabase::new(db_path)?;
        Ok(ConfigService {
            db: Arc::new(Mutex::new(db)),
        })
    }

    /// 获取配置值
    pub async fn get_config(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        match db.get_config(key)? {
            Some(config) => Ok(Some(config.value)),
            None => Ok(None),
        }
    }

    /// 设置配置值
    pub async fn set_config(&self, key: &str, value: &str, category: &str) -> Result<AppConfig, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        db.set_config(key, value, category, None)
            .map_err(|e| e.into())
    }

    /// 获取主题配置
    pub async fn get_theme_config(&self) -> Result<ThemeConfig, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        
        let mode = match db.get_config("theme.mode")? {
            Some(config) => match config.value.as_str() {
                "light" => ThemeMode::Light,
                "dark" => ThemeMode::Dark,
                "auto" => ThemeMode::Auto,
                _ => ThemeMode::Auto,
            },
            None => ThemeMode::Auto,
        };

        let primary_color = db.get_config("theme.primary_color")?
            .map(|c| c.value)
            .unwrap_or_else(|| "#0078d4".to_string());

        let accent_color = db.get_config("theme.accent_color")?
            .map(|c| c.value)
            .unwrap_or_else(|| "#106ebe".to_string());

        let font_size = db.get_config("theme.font_size")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(14);

        let font_family = db.get_config("theme.font_family")?
            .map(|c| c.value)
            .unwrap_or_else(|| "Segoe UI".to_string());

        Ok(ThemeConfig {
            mode,
            primary_color,
            accent_color,
            font_size,
            font_family,
        })
    }

    /// 设置主题配置
    pub async fn set_theme_config(&self, theme: &ThemeConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        
        let mode_str = match theme.mode {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
            ThemeMode::Auto => "auto",
        };

        db.set_config("theme.mode", mode_str, "Theme", Some("主题模式"))?;
        db.set_config("theme.primary_color", &theme.primary_color, "Theme", Some("主色调"))?;
        db.set_config("theme.accent_color", &theme.accent_color, "Theme", Some("强调色"))?;
        db.set_config("theme.font_size", &theme.font_size.to_string(), "Theme", Some("字体大小"))?;
        db.set_config("theme.font_family", &theme.font_family, "Theme", Some("字体族"))?;

        Ok(())
    }

    /// 获取解析配置
    pub async fn get_parse_config(&self) -> Result<ParseConfig, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        
        let auto_parse = db.get_config("parse.auto_parse")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(false);

        let show_line_numbers = db.get_config("parse.show_line_numbers")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(true);

        let max_file_size = db.get_config("parse.max_file_size")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(100);

        let chunk_size = db.get_config("parse.chunk_size")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(1000);

        let timeout_seconds = db.get_config("parse.timeout_seconds")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(30);

        Ok(ParseConfig {
            auto_parse,
            show_line_numbers,
            max_file_size,
            chunk_size,
            timeout_seconds,
        })
    }

    /// 设置解析配置
    pub async fn set_parse_config(&self, parse: &ParseConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        
        db.set_config("parse.show_line_numbers", &parse.show_line_numbers.to_string(), "Parse", Some("显示行号"))?;
        db.set_config("parse.max_file_size", &parse.max_file_size.to_string(), "Parse", Some("最大文件大小"))?;
        db.set_config("parse.chunk_size", &parse.chunk_size.to_string(), "Parse", Some("分块大小"))?;
        db.set_config("parse.timeout_seconds", &parse.timeout_seconds.to_string(), "Parse", Some("超时时间"))?;

        Ok(())
    }

    /// 获取插件配置
    pub async fn get_plugin_config(&self) -> Result<PluginConfig, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        
        let auto_update = db.get_config("plugin.auto_update")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(true);

        let enable_notifications = db.get_config("plugin.enable_notifications")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(true);

        let plugin_directory = db.get_config("plugin.plugin_directory")?
            .map(|c| c.value)
            .unwrap_or_else(|| "./plugins".to_string());

        let max_plugins = db.get_config("plugin.max_plugins")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(50);

        Ok(PluginConfig {
            auto_update,
            enable_notifications,
            plugin_directory,
            max_plugins,
        })
    }

    /// 设置插件配置
    pub async fn set_plugin_config(&self, plugin: &PluginConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        
        db.set_config("plugin.auto_update", &plugin.auto_update.to_string(), "Plugin", Some("自动更新插件"))?;
        db.set_config("plugin.enable_notifications", &plugin.enable_notifications.to_string(), "Plugin", Some("启用插件通知"))?;
        db.set_config("plugin.plugin_directory", &plugin.plugin_directory, "Plugin", Some("插件目录"))?;
        db.set_config("plugin.max_plugins", &plugin.max_plugins.to_string(), "Plugin", Some("最大插件数量"))?;

        Ok(())
    }

    /// 获取窗口配置
    pub async fn get_window_config(&self) -> Result<WindowConfig, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        
        let width = db.get_config("window.width")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(1400);

        let height = db.get_config("window.height")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(900);

        let maximized = db.get_config("window.maximized")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(false);

        let always_on_top = db.get_config("window.always_on_top")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(false);

        let remember_position = db.get_config("window.remember_position")?
            .and_then(|c| c.value.parse().ok())
            .unwrap_or(true);

        Ok(WindowConfig {
            width,
            height,
            maximized,
            always_on_top,
            remember_position,
        })
    }

    /// 设置窗口配置
    pub async fn set_window_config(&self, window: &WindowConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        
        db.set_config("window.width", &window.width.to_string(), "Window", Some("窗口宽度"))?;
        db.set_config("window.height", &window.height.to_string(), "Window", Some("窗口高度"))?;
        db.set_config("window.maximized", &window.maximized.to_string(), "Window", Some("最大化启动"))?;
        db.set_config("window.always_on_top", &window.always_on_top.to_string(), "Window", Some("置顶显示"))?;
        db.set_config("window.remember_position", &window.remember_position.to_string(), "Window", Some("记住位置"))?;

        Ok(())
    }

    /// 获取所有配置
    pub async fn get_all_configs(&self) -> Result<Vec<AppConfig>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        db.get_all_configs().map_err(|e| e.into())
    }

    /// 获取分类配置
    pub async fn get_configs_by_category(&self, category: &str) -> Result<Vec<AppConfig>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        db.get_configs_by_category(category).map_err(|e| e.into())
    }

    /// 批量更新配置
    pub async fn batch_update_configs(&self, configs: &[ConfigUpdateRequest]) -> Result<Vec<AppConfig>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        db.batch_update_configs(configs).map_err(|e| e.into())
    }

    /// 重置为默认配置
    pub async fn reset_to_defaults(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        db.reset_to_defaults().map_err(|e| e.into())
    }

    /// 删除配置
    pub async fn delete_config(&self, key: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().unwrap();
        db.delete_config(key).map_err(|e| e.into())
    }
}
