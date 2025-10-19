use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub config_type: ConfigType,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigType {
    Theme,
    Parse,
    Plugin,
    Window,
    General,
}

impl ConfigType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigType::Theme => "theme",
            ConfigType::Parse => "parse",
            ConfigType::Plugin => "plugin",
            ConfigType::Window => "window",
            ConfigType::General => "general",
        }
    }
}

impl std::fmt::Display for ConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ConfigType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "theme" => Ok(ConfigType::Theme),
            "parse" => Ok(ConfigType::Parse),
            "plugin" => Ok(ConfigType::Plugin),
            "window" => Ok(ConfigType::Window),
            "general" => Ok(ConfigType::General),
            _ => Err(format!("Invalid config type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSchema {
    pub version: i32,
    pub created_at: DateTime<Utc>,
}

impl Default for ConfigSchema {
    fn default() -> Self {
        Self {
            version: 1,
            created_at: Utc::now(),
        }
    }
}