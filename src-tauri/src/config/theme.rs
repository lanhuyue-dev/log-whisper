use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
    Auto,
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub mode: ThemeMode,
    pub primary_color: String,
    pub accent_color: String,
    pub font_size: u32,
    pub font_family: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Auto,
            primary_color: "#3b82f6".to_string(),
            accent_color: "#10b981".to_string(),
            font_size: 14,
            font_family: "system-ui".to_string(),
        }
    }
}