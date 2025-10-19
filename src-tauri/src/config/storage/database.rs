use rusqlite::{Connection, Result as SqliteResult, params, OptionalExtension};
use std::path::Path;
use crate::config::storage::{ConfigEntry, ConfigType};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use log::warn;

#[derive(Debug, Clone)]
pub struct ConfigDatabase {
    connection: Arc<Mutex<Connection>>,
}

impl ConfigDatabase {
    pub fn new<P: AsRef<Path>>(db_path: P) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self {
            connection: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    pub fn init_schema(&self) -> SqliteResult<()> {
        let conn = self.connection.lock().unwrap();

        // 创建配置表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS config_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,
                value TEXT NOT NULL,
                config_type TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // 创建模式表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                version INTEGER NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // 插入或更新模式版本
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO schema_version (id, version, created_at) VALUES (1, ?, ?)",
            params![1, now],
        )?;

        // 创建索引
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_config_key ON config_entries(key)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_config_type ON config_entries(config_type)",
            [],
        )?;

        Ok(())
    }

    pub fn set_config(&self, key: &str, value: &str, config_type: ConfigType) -> SqliteResult<()> {
        let conn = self.connection.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        let type_str = config_type.to_string();

        conn.execute(
            "INSERT OR REPLACE INTO config_entries (key, value, config_type, updated_at)
             VALUES (?, ?, ?, ?)",
            params![key, value, type_str, now],
        )?;

        Ok(())
    }

    pub fn get_config(&self, key: &str) -> SqliteResult<Option<ConfigEntry>> {
        let conn = self.connection.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT key, value, config_type, updated_at
             FROM config_entries
             WHERE key = ?"
        )?;

        let mut rows = stmt.query_map(params![key], |row| {
            Ok(ConfigEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                config_type: row.get::<_, String>(2)?.parse().unwrap(),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        match rows.next() {
            Some(entry) => Ok(Some(entry?)),
            None => Ok(None),
        }
    }

    pub fn get_configs_by_type(&self, config_type: &ConfigType) -> SqliteResult<Vec<ConfigEntry>> {
        let conn = self.connection.lock().unwrap();
        let type_str = config_type.to_string();

        let mut stmt = conn.prepare(
            "SELECT key, value, config_type, updated_at
             FROM config_entries
             WHERE config_type = ?
             ORDER BY key"
        )?;

        let entries: SqliteResult<Vec<_>> = stmt.query_map(params![type_str], |row| {
            Ok(ConfigEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                config_type: row.get::<_, String>(2)?.parse().unwrap(),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?.collect();

        entries
    }

    pub fn get_all_configs(&self) -> SqliteResult<Vec<ConfigEntry>> {
        let conn = self.connection.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT key, value, config_type, updated_at
             FROM config_entries
             ORDER BY config_type, key"
        )?;

        let entries: SqliteResult<Vec<_>> = stmt.query_map([], |row| {
            Ok(ConfigEntry {
                key: row.get(0)?,
                value: row.get(1)?,
                config_type: row.get::<_, String>(2)?.parse().unwrap(),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?.collect();

        entries
    }

    pub fn delete_config(&self, key: &str) -> SqliteResult<bool> {
        let conn = self.connection.lock().unwrap();

        let rows_affected = conn.execute(
            "DELETE FROM config_entries WHERE key = ?",
            params![key],
        )?;

        Ok(rows_affected > 0)
    }

    pub fn delete_configs_by_type(&self, config_type: &ConfigType) -> SqliteResult<usize> {
        let conn = self.connection.lock().unwrap();
        let type_str = config_type.to_string();

        conn.execute(
            "DELETE FROM config_entries WHERE config_type = ?",
            params![type_str],
        )
    }

    pub fn clear_all_configs(&self) -> SqliteResult<()> {
        let conn = self.connection.lock().unwrap();

        conn.execute("DELETE FROM config_entries", [])?;
        Ok(())
    }

    pub fn get_schema_version(&self) -> SqliteResult<i32> {
        let conn = self.connection.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT version FROM schema_version WHERE id = 1"
        )?;

        let version = stmt.query_row([], |row| row.get(0)).optional()?;

        Ok(version.unwrap_or(0))
    }

    pub fn backup_config(&self, _backup_path: &Path) -> SqliteResult<()> {
        // Note: SQLite backup is not available in the bundled version
        // This is a placeholder implementation
        warn!("SQLite backup not available in bundled version");
        Ok(())
    }

    pub fn get_stats(&self) -> SqliteResult<ConfigStats> {
        let conn = self.connection.lock().unwrap();

        let total_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM config_entries",
            [],
            |row| row.get(0),
        )?;

        let mut type_counts = conn.prepare(
            "SELECT config_type, COUNT(*)
             FROM config_entries
             GROUP BY config_type"
        )?;

        let mut stats = ConfigStats {
            total_entries: total_count as usize,
            config_types: std::collections::HashMap::new(),
        };

        let rows = type_counts.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
            ))
        })?;

        for row in rows {
            let (config_type, count) = row?;
            stats.config_types.insert(config_type, count as usize);
        }

        Ok(stats)
    }
}

#[derive(Debug, Clone)]
pub struct ConfigStats {
    pub total_entries: usize,
    pub config_types: std::collections::HashMap<String, usize>,
}