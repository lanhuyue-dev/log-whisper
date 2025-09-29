use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tokio::fs;
use crate::models::ParseError;
use crate::utils::EncodingDetector;

/// 文件读取器
pub struct FileReader {
    max_file_size: usize,
    chunk_size: usize,
    encoding_detector: EncodingDetector,
}

impl FileReader {
    /// 创建新的文件读取器
    pub fn new() -> Self {
        Self {
            max_file_size: 50 * 1024 * 1024, // 50MB
            chunk_size: 8192, // 8KB
            encoding_detector: EncodingDetector::new(),
        }
    }
    
    /// 设置最大文件大小
    pub fn with_max_file_size(mut self, max_size: usize) -> Self {
        self.max_file_size = max_size;
        self
    }
    
    /// 设置块大小
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }
    
    /// 异步读取文件行
    pub async fn read_lines(&self, file_path: &str) -> Result<Vec<String>, ParseError> {
        let path = Path::new(file_path);
        
        // 检查文件是否存在
        if !path.exists() {
            return Err(ParseError::FileReadError(format!("File not found: {}", file_path)));
        }
        
        // 检查文件大小
        let metadata = fs::metadata(path).await
            .map_err(|e| ParseError::FileReadError(format!("Failed to get file metadata: {}", e)))?;
        
        if metadata.len() as usize > self.max_file_size {
            return Err(ParseError::FileReadError(format!(
                "File too large: {} bytes (max: {} bytes)", 
                metadata.len(), 
                self.max_file_size
            )));
        }
        
        // 使用编码检测读取文件
        let (content, encoding_result) = self.encoding_detector.read_file_with_encoding(file_path)
            .map_err(|e| ParseError::FileReadError(format!("Failed to read file with encoding detection: {}", e)))?;
        
        println!("Detected encoding: {} (confidence: {:.2})", 
                encoding_result.encoding_name, encoding_result.confidence);
        
        // 分割为行
        let lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
        
        Ok(lines)
    }
    
    /// 同步读取文件行
    pub fn read_lines_sync(&self, file_path: &str) -> Result<Vec<String>, ParseError> {
        let path = Path::new(file_path);
        
        // 检查文件是否存在
        if !path.exists() {
            return Err(ParseError::FileReadError(format!("File not found: {}", file_path)));
        }
        
        // 检查文件大小
        let metadata = std::fs::metadata(path)
            .map_err(|e| ParseError::FileReadError(format!("Failed to get file metadata: {}", e)))?;
        
        if metadata.len() as usize > self.max_file_size {
            return Err(ParseError::FileReadError(format!(
                "File too large: {} bytes (max: {} bytes)", 
                metadata.len(), 
                self.max_file_size
            )));
        }
        
        // 使用编码检测读取文件
        let (content, encoding_result) = self.encoding_detector.read_file_with_encoding(file_path)
            .map_err(|e| ParseError::FileReadError(format!("Failed to read file with encoding detection: {}", e)))?;
        
        println!("Detected encoding: {} (confidence: {:.2})", 
                encoding_result.encoding_name, encoding_result.confidence);
        
        // 分割为行
        let lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
        
        Ok(lines)
    }
    
    /// 流式读取文件
    pub fn read_file_stream(&self, file_path: &str) -> Result<impl Iterator<Item = String>, ParseError> {
        let path = Path::new(file_path);
        
        // 检查文件是否存在
        if !path.exists() {
            return Err(ParseError::FileReadError(format!("File not found: {}", file_path)));
        }
        
        // 检查文件大小
        let metadata = std::fs::metadata(path)
            .map_err(|e| ParseError::FileReadError(format!("Failed to get file metadata: {}", e)))?;
        
        if metadata.len() as usize > self.max_file_size {
            return Err(ParseError::FileReadError(format!(
                "File too large: {} bytes (max: {} bytes)", 
                metadata.len(), 
                self.max_file_size
            )));
        }
        
        // 创建文件读取器
        let file = File::open(path)
            .map_err(|e| ParseError::FileReadError(format!("Failed to open file: {}", e)))?;
        
        let reader = BufReader::new(file);
        let lines = reader.lines().map(|line| line.unwrap_or_default());
        
        Ok(lines)
    }
    
    /// 验证文件类型
    pub fn validate_file_type(&self, file_path: &str) -> Result<(), ParseError> {
        let path = Path::new(file_path);
        
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            match ext.as_str() {
                "log" | "txt" => Ok(()),
                _ => Err(ParseError::FileReadError(format!(
                    "Unsupported file type: {} (supported: .log, .txt)", 
                    ext
                ))),
            }
        } else {
            Err(ParseError::FileReadError("File has no extension".to_string()))
        }
    }
    
    /// 获取文件信息
    pub async fn get_file_info(&self, file_path: &str) -> Result<FileInfo, ParseError> {
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Err(ParseError::FileReadError(format!("File not found: {}", file_path)));
        }
        
        let metadata = fs::metadata(path).await
            .map_err(|e| ParseError::FileReadError(format!("Failed to get file metadata: {}", e)))?;
        
        let content = fs::read_to_string(path).await
            .map_err(|e| ParseError::FileReadError(format!("Failed to read file: {}", e)))?;
        
        let line_count = content.lines().count();
        let char_count = content.chars().count();
        let word_count = content.split_whitespace().count();
        
        Ok(FileInfo {
            path: file_path.to_string(),
            size: metadata.len() as usize,
            line_count,
            char_count,
            word_count,
            created: metadata.created().ok(),
            modified: metadata.modified().ok(),
        })
    }
}

/// 文件信息结构体
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub size: usize,
    pub line_count: usize,
    pub char_count: usize,
    pub word_count: usize,
    pub created: Option<std::time::SystemTime>,
    pub modified: Option<std::time::SystemTime>,
}

impl Default for FileReader {
    fn default() -> Self {
        Self::new()
    }
}
