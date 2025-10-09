use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub resizable: bool,
    pub fullscreen: bool,
    pub title: String,
    pub always_on_top: bool,
    pub skip_taskbar: bool,
    pub theme: String,
    pub maximized: bool,
    pub remember_position: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            min_width: 800,
            min_height: 600,
            resizable: true,
            fullscreen: false,
            title: "LogWhisper - 日志分析工具".to_string(),
            always_on_top: false,
            skip_taskbar: false,
            theme: "system".to_string(),
            maximized: false,
            remember_position: true,
        }
    }
}