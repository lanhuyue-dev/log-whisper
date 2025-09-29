use tauri::{AppHandle, Manager, Window, Emitter, EventTarget};
use crate::tauri::AppState;

/// 文件拖拽处理器
pub struct FileDropHandler;

impl FileDropHandler {
    /// 处理文件拖拽事件
    pub fn handle_file_drop(
        app: &AppHandle,
        window: &Window,
        file_path: String,
    ) -> Result<(), String> {
        // 验证文件类型
        if !Self::is_valid_file(&file_path) {
            return Err("不支持的文件格式".to_string());
        }
        
        // 更新应用状态
        if let Some(_state) = app.try_state::<AppState>() {
            // 这里需要可变引用，但AppState在Tauri中通常是不可变的
            // 实际应用中可能需要使用Mutex或其他同步机制
        }
        
        // 发送文件拖拽事件到前端
        window.emit("file-dropped", file_path)
            .map_err(|e| format!("Failed to emit file-dropped event: {}", e))?;
        
        Ok(())
    }
    
    /// 检查文件是否有效
    fn is_valid_file(file_path: &str) -> bool {
        let path = std::path::Path::new(file_path);
        
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            matches!(ext.as_str(), "log" | "txt")
        } else {
            false
        }
    }
}

/// 插件切换处理器
pub struct PluginChangeHandler;

impl PluginChangeHandler {
    /// 处理插件切换事件
    pub fn handle_plugin_change(
        app: &AppHandle,
        plugin_name: String,
    ) -> Result<(), String> {
        // 验证插件名称
        if !Self::is_valid_plugin(&plugin_name) {
            return Err(format!("无效的插件名称: {}", plugin_name));
        }
        
        // 发送插件切换事件到前端
        app.emit_to(EventTarget::Any, "plugin-changed", plugin_name)
            .map_err(|e| format!("Failed to emit plugin-changed event: {}", e))?;
        
        Ok(())
    }
    
    /// 检查插件是否有效
    fn is_valid_plugin(plugin_name: &str) -> bool {
        matches!(plugin_name, "Auto" | "MyBatis" | "JSON" | "Raw")
    }
}

/// 搜索处理器
pub struct SearchHandler;

impl SearchHandler {
    /// 处理搜索事件
    pub fn handle_search(
        app: &AppHandle,
        search_term: String,
    ) -> Result<(), String> {
        // 发送搜索事件到前端
        app.emit_to(EventTarget::Any, "search-performed", search_term)
            .map_err(|e| format!("Failed to emit search-performed event: {}", e))?;
        
        Ok(())
    }
    
    /// 处理清除搜索事件
    pub fn handle_clear_search(
        app: &AppHandle,
    ) -> Result<(), String> {
        // 发送清除搜索事件到前端
        app.emit_to(EventTarget::Any, "search-cleared", ())
            .map_err(|e| format!("Failed to emit search-cleared event: {}", e))?;
        
        Ok(())
    }
}

/// 错误处理器
pub struct ErrorHandler;

impl ErrorHandler {
    /// 处理解析错误
    pub fn handle_parse_error(
        app: &AppHandle,
        error: String,
    ) -> Result<(), String> {
        // 发送解析错误事件到前端
        app.emit_to(EventTarget::Any, "parse-error", error)
            .map_err(|e| format!("Failed to emit parse-error event: {}", e))?;
        
        Ok(())
    }
    
    /// 处理文件错误
    pub fn handle_file_error(
        app: &AppHandle,
        error: String,
    ) -> Result<(), String> {
        // 发送文件错误事件到前端
        app.emit_to(EventTarget::Any, "file-error", error)
            .map_err(|e| format!("Failed to emit file-error event: {}", e))?;
        
        Ok(())
    }
}

/// 进度处理器
pub struct ProgressHandler;

impl ProgressHandler {
    /// 处理解析进度
    pub fn handle_parse_progress(
        app: &AppHandle,
        progress: f32,
    ) -> Result<(), String> {
        // 发送解析进度事件到前端
        app.emit_to(EventTarget::Any, "parse-progress", progress)
            .map_err(|e| format!("Failed to emit parse-progress event: {}", e))?;
        
        Ok(())
    }
    
    /// 处理解析完成
    pub fn handle_parse_complete(
        app: &AppHandle,
        result_count: usize,
    ) -> Result<(), String> {
        // 发送解析完成事件到前端
        app.emit_to(EventTarget::Any, "parse-complete", result_count)
            .map_err(|e| format!("Failed to emit parse-complete event: {}", e))?;
        
        Ok(())
    }
}
