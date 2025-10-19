use rusqlite::{Connection, Result as SqliteResult, params, OptionalExtension};
use std::path::Path;
use crate::config::storage::{ConfigType};
use std::collections::HashMap;

/// 简化的配置存储服务
pub struct SimpleConfigStorage {
    connection: Connection,
}

impl SimpleConfigStorage {
    pub fn new<P: AsRef<Path>>(db_path: P) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        let storage = Self { connection: conn };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> SqliteResult<()> {
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS configs (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                config_type TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn set_config(&mut self, key: &str, value: &str, config_type: ConfigType) -> SqliteResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let type_str = match config_type {
            ConfigType::Theme => "theme",
            ConfigType::Parse => "parse",
            ConfigType::Plugin => "plugin",
            ConfigType::Window => "window",
            ConfigType::General => "general",
        };

        self.connection.execute(
            "INSERT OR REPLACE INTO configs (key, value, config_type, updated_at)
             VALUES (?, ?, ?, ?)",
            params![key, value, type_str, now],
        )?;
        Ok(())
    }

    pub fn get_config(&self, key: &str) -> SqliteResult<Option<String>> {
        let mut stmt = self.connection.prepare(
            "SELECT value FROM configs WHERE key = ?"
        )?;

        let value = stmt.query_row(params![key], |row| row.get::<_, String>(0)).optional()?;
        Ok(value)
    }

    pub fn get_configs_by_type(&self, config_type: &ConfigType) -> SqliteResult<HashMap<String, String>> {
        let type_str = match config_type {
            ConfigType::Theme => "theme",
            ConfigType::Parse => "parse",
            ConfigType::Plugin => "plugin",
            ConfigType::Window => "window",
            ConfigType::General => "general",
        };

        let mut stmt = self.connection.prepare(
            "SELECT key, value FROM configs WHERE config_type = ?"
        )?;

        let mut configs = HashMap::new();
        let rows = stmt.query_map(params![type_str], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (key, value) = row?;
            configs.insert(key, value);
        }

        Ok(configs)
    }

    pub fn delete_config(&mut self, key: &str) -> SqliteResult<bool> {
        let rows_affected = self.connection.execute(
            "DELETE FROM configs WHERE key = ?",
            params![key],
        )?;
        Ok(rows_affected > 0)
    }

    pub fn clear_all(&mut self) -> SqliteResult<()> {
        self.connection.execute("DELETE FROM configs", [])?;
        Ok(())
    }
}