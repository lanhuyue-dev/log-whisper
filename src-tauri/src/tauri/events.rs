use serde::{Deserialize, Serialize};

/// 解析事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseEvent {
    pub event_type: ParseEventType,
    pub data: ParseEventData,
}

/// 解析事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParseEventType {
    Started,
    Progress,
    Completed,
    Error,
}

/// 解析事件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseEventData {
    pub file_path: Option<String>,
    pub progress: Option<f32>,
    pub result_count: Option<usize>,
    pub error_message: Option<String>,
}

impl ParseEvent {
    /// 创建解析开始事件
    pub fn started(file_path: String) -> Self {
        Self {
            event_type: ParseEventType::Started,
            data: ParseEventData {
                file_path: Some(file_path),
                progress: None,
                result_count: None,
                error_message: None,
            },
        }
    }
    
    /// 创建解析进度事件
    pub fn progress(progress: f32) -> Self {
        Self {
            event_type: ParseEventType::Progress,
            data: ParseEventData {
                file_path: None,
                progress: Some(progress),
                result_count: None,
                error_message: None,
            },
        }
    }
    
    /// 创建解析完成事件
    pub fn completed(result_count: usize) -> Self {
        Self {
            event_type: ParseEventType::Completed,
            data: ParseEventData {
                file_path: None,
                progress: Some(1.0),
                result_count: Some(result_count),
                error_message: None,
            },
        }
    }
    
    /// 创建解析错误事件
    pub fn error(error_message: String) -> Self {
        Self {
            event_type: ParseEventType::Error,
            data: ParseEventData {
                file_path: None,
                progress: None,
                result_count: None,
                error_message: Some(error_message),
            },
        }
    }
}

/// UI事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIEvent {
    pub event_type: UIEventType,
    pub data: UIEventData,
}

/// UI事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIEventType {
    FileDropped,
    PluginChanged,
    SearchPerformed,
    SearchCleared,
    ThemeChanged,
    WindowResized,
}

/// UI事件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIEventData {
    pub file_path: Option<String>,
    pub plugin_name: Option<String>,
    pub search_term: Option<String>,
    pub theme: Option<String>,
    pub window_size: Option<WindowSize>,
}

/// 窗口大小
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: f64,
    pub height: f64,
}

impl UIEvent {
    /// 创建文件拖拽事件
    pub fn file_dropped(file_path: String) -> Self {
        Self {
            event_type: UIEventType::FileDropped,
            data: UIEventData {
                file_path: Some(file_path),
                plugin_name: None,
                search_term: None,
                theme: None,
                window_size: None,
            },
        }
    }
    
    /// 创建插件切换事件
    pub fn plugin_changed(plugin_name: String) -> Self {
        Self {
            event_type: UIEventType::PluginChanged,
            data: UIEventData {
                file_path: None,
                plugin_name: Some(plugin_name),
                search_term: None,
                theme: None,
                window_size: None,
            },
        }
    }
    
    /// 创建搜索事件
    pub fn search_performed(search_term: String) -> Self {
        Self {
            event_type: UIEventType::SearchPerformed,
            data: UIEventData {
                file_path: None,
                plugin_name: None,
                search_term: Some(search_term),
                theme: None,
                window_size: None,
            },
        }
    }
    
    /// 创建清除搜索事件
    pub fn search_cleared() -> Self {
        Self {
            event_type: UIEventType::SearchCleared,
            data: UIEventData {
                file_path: None,
                plugin_name: None,
                search_term: None,
                theme: None,
                window_size: None,
            },
        }
    }
    
    /// 创建主题切换事件
    pub fn theme_changed(theme: String) -> Self {
        Self {
            event_type: UIEventType::ThemeChanged,
            data: UIEventData {
                file_path: None,
                plugin_name: None,
                search_term: None,
                theme: Some(theme),
                window_size: None,
            },
        }
    }
    
    /// 创建窗口大小改变事件
    pub fn window_resized(width: f64, height: f64) -> Self {
        Self {
            event_type: UIEventType::WindowResized,
            data: UIEventData {
                file_path: None,
                plugin_name: None,
                search_term: None,
                theme: None,
                window_size: Some(WindowSize { width, height }),
            },
        }
    }
}

/// 错误事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub event_type: ErrorEventType,
    pub message: String,
    pub details: Option<String>,
}

/// 错误事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorEventType {
    ParseError,
    FileError,
    PluginError,
    SystemError,
}

impl ErrorEvent {
    /// 创建解析错误事件
    pub fn parse_error(message: String, details: Option<String>) -> Self {
        Self {
            event_type: ErrorEventType::ParseError,
            message,
            details,
        }
    }
    
    /// 创建文件错误事件
    pub fn file_error(message: String, details: Option<String>) -> Self {
        Self {
            event_type: ErrorEventType::FileError,
            message,
            details,
        }
    }
    
    /// 创建插件错误事件
    pub fn plugin_error(message: String, details: Option<String>) -> Self {
        Self {
            event_type: ErrorEventType::PluginError,
            message,
            details,
        }
    }
    
    /// 创建系统错误事件
    pub fn system_error(message: String, details: Option<String>) -> Self {
        Self {
            event_type: ErrorEventType::SystemError,
            message,
            details,
        }
    }
}
