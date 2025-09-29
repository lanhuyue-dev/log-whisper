use std::path::Path;
use std::fs;

/// 文件工具函数
pub struct FileUtils;

impl FileUtils {
    /// 检查文件是否存在
    pub fn file_exists(file_path: &str) -> bool {
        Path::new(file_path).exists()
    }
    
    /// 获取文件大小
    pub fn get_file_size(file_path: &str) -> Result<u64, String> {
        let metadata = fs::metadata(file_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;
        Ok(metadata.len())
    }
    
    /// 获取文件扩展名
    pub fn get_file_extension(file_path: &str) -> Option<String> {
        Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }
    
    /// 验证文件类型
    pub fn validate_file_type(file_path: &str) -> Result<(), String> {
        if let Some(extension) = Self::get_file_extension(file_path) {
            match extension.as_str() {
                "log" | "txt" => Ok(()),
                _ => Err(format!("不支持的文件类型: {}", extension)),
            }
        } else {
            Err("文件没有扩展名".to_string())
        }
    }
    
    /// 获取文件修改时间
    pub fn get_file_modified_time(file_path: &str) -> Result<std::time::SystemTime, String> {
        let metadata = fs::metadata(file_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;
        metadata.modified()
            .map_err(|e| format!("Failed to get modified time: {}", e))
    }
    
    /// 读取文件前几行
    pub fn read_file_preview(file_path: &str, lines: usize) -> Result<Vec<String>, String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let preview_lines: Vec<String> = content
            .lines()
            .take(lines)
            .map(|line| line.to_string())
            .collect();
        
        Ok(preview_lines)
    }
}
