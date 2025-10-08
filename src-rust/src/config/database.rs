use rusqlite::{Connection, Result as SqliteResult, params, Row, OptionalExtension};
use chrono::{DateTime, Utc};
use crate::config::models::*;

/// 配置数据库管理器
pub struct ConfigDatabase {
    conn: Connection,
}

impl ConfigDatabase {
    /// 创建新的配置数据库连接
    pub fn new(db_path: &str) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        let db = ConfigDatabase { conn };
        db.init_tables()?;
        Ok(db)
    }

    /// 初始化数据库表
    fn init_tables(&self) -> SqliteResult<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS app_configs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT UNIQUE NOT NULL,
                value TEXT NOT NULL,
                category TEXT NOT NULL,
                description TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // 创建索引
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_config_key ON app_configs(key)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_config_category ON app_configs(category)",
            [],
        )?;

        // 插入默认配置
        self.insert_default_configs()?;

        Ok(())
    }

    /// 插入默认配置
    fn insert_default_configs(&self) -> SqliteResult<()> {
        let default_configs = vec![
            // 主题配置
            ("theme.mode", "auto", "Theme", "主题模式: light, dark, auto"),
            ("theme.primary_color", "#0078d4", "Theme", "主色调"),
            ("theme.accent_color", "#106ebe", "Theme", "强调色"),
            ("theme.font_size", "14", "Theme", "字体大小"),
            ("theme.font_family", "Segoe UI", "Theme", "字体族"),
            
            // 解析配置
            ("parse.show_line_numbers", "true", "Parse", "显示行号"),
            ("parse.max_file_size", "100", "Parse", "最大文件大小(MB)"),
            ("parse.chunk_size", "1000", "Parse", "分块大小"),
            ("parse.timeout_seconds", "30", "Parse", "超时时间(秒)"),
            
            // 插件配置
            ("plugin.auto_update", "true", "Plugin", "自动更新插件"),
            ("plugin.enable_notifications", "true", "Plugin", "启用插件通知"),
            ("plugin.plugin_directory", "./plugins", "Plugin", "插件目录"),
            ("plugin.max_plugins", "50", "Plugin", "最大插件数量"),
            
            // 窗口配置
            ("window.width", "1400", "Window", "窗口宽度"),
            ("window.height", "900", "Window", "窗口高度"),
            ("window.maximized", "false", "Window", "最大化启动"),
            ("window.always_on_top", "false", "Window", "置顶显示"),
            ("window.remember_position", "true", "Window", "记住位置"),
            
            // 日志配置
            ("log.log_level", "info", "Log", "日志级别"),
            ("log.log_file", "./logs/app.log", "Log", "日志文件路径"),
            ("log.max_log_size", "10", "Log", "最大日志大小(MB)"),
            ("log.log_rotation", "true", "Log", "日志轮转"),
            
            // 性能配置
            ("performance.enable_gpu_acceleration", "true", "Performance", "启用GPU加速"),
            ("performance.max_memory_usage", "512", "Performance", "最大内存使用(MB)"),
            ("performance.cache_size", "64", "Performance", "缓存大小(MB)"),
            ("performance.enable_compression", "true", "Performance", "启用压缩"),
            
            // 通用配置
            ("general.language", "zh-CN", "General", "界面语言"),
            ("general.auto_save", "true", "General", "自动保存"),
            ("general.check_updates", "true", "General", "检查更新"),
        ];

        for (key, value, category, description) in default_configs {
            self.conn.execute(
                "INSERT OR IGNORE INTO app_configs (key, value, category, description) VALUES (?, ?, ?, ?)",
                params![key, value, category, description],
            )?;
        }

        Ok(())
    }

    /// 获取配置
    pub fn get_config(&self, key: &str) -> SqliteResult<Option<AppConfig>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, key, value, category, description, created_at, updated_at 
             FROM app_configs WHERE key = ?"
        )?;
        
        let config = stmt.query_row(params![key], |row| {
            Ok(AppConfig {
                id: row.get(0)?,
                key: row.get(1)?,
                value: row.get(2)?,
                category: row.get(3)?,
                description: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        }).optional()?;

        Ok(config)
    }

    /// 设置配置
    pub fn set_config(&self, key: &str, value: &str, category: &str, description: Option<&str>) -> SqliteResult<AppConfig> {
        let now = Utc::now().to_rfc3339();
        
        // 尝试更新现有配置
        let updated = self.conn.execute(
            "UPDATE app_configs SET value = ?, updated_at = ? WHERE key = ?",
            params![value, now, key],
        )?;

        if updated > 0 {
            // 更新成功，返回更新后的配置
            self.get_config(key)?.ok_or_else(|| {
                rusqlite::Error::QueryReturnedNoRows
            })
        } else {
            // 插入新配置
            self.conn.execute(
                "INSERT INTO app_configs (key, value, category, description, created_at, updated_at) 
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![key, value, category, description, now, now],
            )?;

            self.get_config(key)?.ok_or_else(|| {
                rusqlite::Error::QueryReturnedNoRows
            })
        }
    }

    /// 获取分类下的所有配置
    pub fn get_configs_by_category(&self, category: &str) -> SqliteResult<Vec<AppConfig>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, key, value, category, description, created_at, updated_at 
             FROM app_configs WHERE category = ? ORDER BY key"
        )?;
        
        let configs = stmt.query_map(params![category], |row| {
            Ok(AppConfig {
                id: row.get(0)?,
                key: row.get(1)?,
                value: row.get(2)?,
                category: row.get(3)?,
                description: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?.collect::<SqliteResult<Vec<_>>>()?;

        Ok(configs)
    }

    /// 获取所有配置
    pub fn get_all_configs(&self) -> SqliteResult<Vec<AppConfig>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, key, value, category, description, created_at, updated_at 
             FROM app_configs ORDER BY category, key"
        )?;
        
        let configs = stmt.query_map([], |row| {
            Ok(AppConfig {
                id: row.get(0)?,
                key: row.get(1)?,
                value: row.get(2)?,
                category: row.get(3)?,
                description: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?.collect::<SqliteResult<Vec<_>>>()?;

        Ok(configs)
    }

    /// 删除配置
    pub fn delete_config(&self, key: &str) -> SqliteResult<bool> {
        let rows_affected = self.conn.execute(
            "DELETE FROM app_configs WHERE key = ?",
            params![key],
        )?;

        Ok(rows_affected > 0)
    }

    /// 批量更新配置
    pub fn batch_update_configs(&self, configs: &[ConfigUpdateRequest]) -> SqliteResult<Vec<AppConfig>> {
        let mut updated_configs = Vec::new();
        
        for config in configs {
            let app_config = self.set_config(
                &config.key,
                &config.value,
                &config.category.to_string(),
                None,
            )?;
            updated_configs.push(app_config);
        }

        Ok(updated_configs)
    }

    /// 重置配置为默认值
    pub fn reset_to_defaults(&self) -> SqliteResult<()> {
        // 删除所有现有配置
        self.conn.execute("DELETE FROM app_configs", [])?;
        
        // 重新插入默认配置
        self.insert_default_configs()?;
        
        Ok(())
    }
}

impl Drop for ConfigDatabase {
    fn drop(&mut self) {
        // 数据库连接会在结构体销毁时自动关闭
    }
}
