use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use tokio::fs;
use memmap2::MmapOptions;
use crate::models::ParseError;
use crate::utils::EncodingDetector;

/// 大文件处理模式
#[derive(Debug, Clone, PartialEq)]
pub enum LargeFileMode {
    /// 内存映射模式
    MemoryMapped,
    /// 流式读取模式
    Streaming,
    /// 分块读取模式
    Chunked,
    /// 传统模式（一次性加载）
    Traditional,
}

/// 文件块信息
#[derive(Debug, Clone)]
pub struct FileChunk {
    pub start_offset: u64,
    pub end_offset: u64,
    pub start_line: usize,
    pub end_line: usize,
    pub estimated_lines: usize,
}

/// 流式读取迭代器
pub struct StreamingLineIterator {
    file: File,
    buffer: Vec<u8>,
    position: u64,
    file_size: u64,
    chunk_size: usize,
    line_number: usize,
}

impl StreamingLineIterator {
    fn new(mut file: File, file_size: u64, chunk_size: usize) -> Result<Self, ParseError> {
        file.seek(SeekFrom::Start(0))
            .map_err(|e| ParseError::FileReadError(format!("Failed to seek file: {}", e)))?;
        
        Ok(Self {
            file,
            buffer: Vec::with_capacity(chunk_size),
            position: 0,
            file_size,
            chunk_size,
            line_number: 0,
        })
    }
}

impl Iterator for StreamingLineIterator {
    type Item = Result<String, ParseError>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.file_size {
            return None;
        }
        
        // 读取下一个块
        self.buffer.clear();
        self.buffer.resize(self.chunk_size, 0);
        
        match self.file.read(&mut self.buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    return None;
                }
                
                self.buffer.truncate(bytes_read);
                self.position += bytes_read as u64;
                
                // 解析为字符串并分割行
                match String::from_utf8(self.buffer.clone()) {
                    Ok(content) => {
                        for line in content.lines() {
                            self.line_number += 1;
                            if !line.trim().is_empty() {
                                return Some(Ok(line.to_string()));
                            }
                        }
                        self.next() // 递归查找下一个非空行
                    }
                    Err(e) => Some(Err(ParseError::FileReadError(format!("UTF-8 decode error: {}", e))))
                }
            }
            Err(e) => Some(Err(ParseError::FileReadError(format!("File read error: {}", e))))
        }
    }
}

/// 文件读取器
pub struct FileReader {
    max_file_size: usize,
    chunk_size: usize,
    encoding_detector: EncodingDetector,
    /// 大文件处理阈值（超过此大小使用大文件模式）
    large_file_threshold: usize,
    /// 默认大文件处理模式
    large_file_mode: LargeFileMode,
    /// 内存映射块大小
    mmap_chunk_size: usize,
}

impl FileReader {
    /// 创建新的文件读取器
    pub fn new() -> Self {
        Self {
            max_file_size: 2 * 1024 * 1024 * 1024, // 2GB
            chunk_size: 64 * 1024, // 64KB
            encoding_detector: EncodingDetector::new(),
            large_file_threshold: 100 * 1024 * 1024, // 100MB
            large_file_mode: LargeFileMode::MemoryMapped,
            mmap_chunk_size: 1024 * 1024, // 1MB
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
    
    /// 设置大文件处理模式
    pub fn with_large_file_mode(mut self, mode: LargeFileMode) -> Self {
        self.large_file_mode = mode;
        self
    }
    
    /// 设置大文件阈值
    pub fn with_large_file_threshold(mut self, threshold: usize) -> Self {
        self.large_file_threshold = threshold;
        self
    }
    /// 智能读取文件行（根据文件大小选择最优策略）
    pub async fn read_lines_smart(&self, file_path: &str) -> Result<Box<dyn Iterator<Item = Result<String, ParseError>>>, ParseError> {
        let path = Path::new(file_path);
        
        // 检查文件是否存在
        if !path.exists() {
            return Err(ParseError::FileReadError(format!("File not found: {}", file_path)));
        }
        
        // 检查文件大小
        let metadata = fs::metadata(path).await
            .map_err(|e| ParseError::FileReadError(format!("Failed to get file metadata: {}", e)))?;
        
        let file_size = metadata.len() as usize;
        
        if file_size > self.max_file_size {
            return Err(ParseError::FileReadError(format!(
                "File too large: {} bytes (max: {} bytes)", 
                file_size, 
                self.max_file_size
            )));
        }
        
        // 根据文件大小选择处理策略
        if file_size > self.large_file_threshold {
            match self.large_file_mode {
                LargeFileMode::MemoryMapped => self.read_lines_mmap(file_path),
                LargeFileMode::Streaming => self.read_lines_streaming(file_path),
                LargeFileMode::Chunked => self.read_lines_chunked(file_path),
                LargeFileMode::Traditional => {
                    // 传统模式，但先警告
                    eprintln!("Warning: Using traditional mode for large file ({} bytes). Consider using a more efficient mode.", file_size);
                    let lines = self.read_lines(file_path).await?;
                    Ok(Box::new(lines.into_iter().map(Ok)))
                }
            }
        } else {
            // 小文件使用传统模式
            let lines = self.read_lines(file_path).await?;
            Ok(Box::new(lines.into_iter().map(Ok)))
        }
    }
    
    /// 使用内存映射读取文件
    pub fn read_lines_mmap(&self, file_path: &str) -> Result<Box<dyn Iterator<Item = Result<String, ParseError>>>, ParseError> {
        let file = File::open(file_path)
            .map_err(|e| ParseError::FileReadError(format!("Failed to open file: {}", e)))?;
        
        // 安全地创建内存映射
        let mmap = unsafe {
            MmapOptions::new()
                .map(&file)
                .map_err(|e| ParseError::FileReadError(format!("Failed to memory map file: {}", e)))?
        };
        
        // 将内存映射的内容转换为字符串
        let content = std::str::from_utf8(&mmap)
            .map_err(|e| ParseError::FileReadError(format!("Invalid UTF-8 in file: {}", e)))?;
        
        // 分割为行并过滤空行
        let lines: Vec<String> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.to_string())
            .collect();
        
        Ok(Box::new(lines.into_iter().map(Ok)))
    }
    
    /// 使用流式读取文件
    pub fn read_lines_streaming(&self, file_path: &str) -> Result<Box<dyn Iterator<Item = Result<String, ParseError>>>, ParseError> {
        let file = File::open(file_path)
            .map_err(|e| ParseError::FileReadError(format!("Failed to open file: {}", e)))?;
        
        let metadata = file.metadata()
            .map_err(|e| ParseError::FileReadError(format!("Failed to get file metadata: {}", e)))?;
        
        let file_size = metadata.len();
        let iterator = StreamingLineIterator::new(file, file_size, self.chunk_size)?;
        
        Ok(Box::new(iterator))
    }
    
    /// 使用分块读取文件
    pub fn read_lines_chunked(&self, file_path: &str) -> Result<Box<dyn Iterator<Item = Result<String, ParseError>>>, ParseError> {
        let file = File::open(file_path)
            .map_err(|e| ParseError::FileReadError(format!("Failed to open file: {}", e)))?;
        
        let reader = BufReader::with_capacity(self.chunk_size, file);
        let lines = reader.lines().map(|line_result| {
            line_result
                .map_err(|e| ParseError::FileReadError(format!("Failed to read line: {}", e)))
                .and_then(|line| {
                    if line.trim().is_empty() {
                        Err(ParseError::FileReadError("Empty line".to_string()))
                    } else {
                        Ok(line)
                    }
                })
        }).filter(|result| result.is_ok());
        
        Ok(Box::new(lines))
    }
    
    /// 获取文件分块信息（用于虚拟滚动）
    pub fn get_file_chunks(&self, file_path: &str, chunk_count: usize) -> Result<Vec<FileChunk>, ParseError> {
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Err(ParseError::FileReadError(format!("File not found: {}", file_path)));
        }
        
        let metadata = std::fs::metadata(path)
            .map_err(|e| ParseError::FileReadError(format!("Failed to get file metadata: {}", e)))?;
        
        let file_size = metadata.len();
        let chunk_size = file_size / chunk_count as u64;
        
        let mut chunks = Vec::new();
        let mut current_offset = 0u64;
        let mut current_line = 0usize;
        
        for i in 0..chunk_count {
            let start_offset = current_offset;
            let end_offset = if i == chunk_count - 1 {
                file_size
            } else {
                current_offset + chunk_size
            };
            
            // 估算行数（基于平均行长度）
            let estimated_lines = ((end_offset - start_offset) / 80).max(1) as usize; // 假设平均行长80字符
            
            chunks.push(FileChunk {
                start_offset,
                end_offset,
                start_line: current_line,
                end_line: current_line + estimated_lines,
                estimated_lines,
            });
            
            current_offset = end_offset;
            current_line += estimated_lines;
        }
        
        Ok(chunks)
    }
    
    /// 读取指定范围的文件内容
    pub fn read_file_range(&self, file_path: &str, start_offset: u64, end_offset: u64) -> Result<Vec<String>, ParseError> {
        let mut file = File::open(file_path)
            .map_err(|e| ParseError::FileReadError(format!("Failed to open file: {}", e)))?;
        
        file.seek(SeekFrom::Start(start_offset))
            .map_err(|e| ParseError::FileReadError(format!("Failed to seek file: {}", e)))?;
        
        let mut buffer = vec![0u8; (end_offset - start_offset) as usize];
        file.read_exact(&mut buffer)
            .map_err(|e| ParseError::FileReadError(format!("Failed to read file range: {}", e)))?;
        
        let content = String::from_utf8(buffer)
            .map_err(|e| ParseError::FileReadError(format!("Invalid UTF-8 in file range: {}", e)))?;
        
        let lines: Vec<String> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.to_string())
            .collect();
        
        Ok(lines)
    }
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
