use serde::{Deserialize, Serialize};

/// 渲染块结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedBlock {
    /// 块唯一标识
    pub id: String,
    /// 块类型
    pub block_type: BlockType,
    /// 块标题
    pub title: String,
    /// 原始内容
    pub content: String,
    /// 格式化后的内容
    pub formatted_content: String,
    /// 是否可复制
    pub is_copyable: bool,
    /// 块元数据
    pub metadata: BlockMetadata,
}

/// 块类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlockType {
    Sql,
    Json,
    Error,
    Warning,
    Info,
    Raw,
}

impl BlockType {
    /// 获取块类型的图标
    pub fn icon(&self) -> &'static str {
        match self {
            BlockType::Sql => "🔍",
            BlockType::Json => "📄",
            BlockType::Error => "⚠️",
            BlockType::Warning => "⚠️",
            BlockType::Info => "ℹ️",
            BlockType::Raw => "📝",
        }
    }
    
    /// 获取块类型的CSS类名
    pub fn css_class(&self) -> &'static str {
        match self {
            BlockType::Sql => "border-green-200 bg-green-50",
            BlockType::Json => "border-blue-200 bg-blue-50",
            BlockType::Error => "border-red-200 bg-red-50",
            BlockType::Warning => "border-yellow-200 bg-yellow-50",
            BlockType::Info => "border-gray-200 bg-gray-50",
            BlockType::Raw => "border-gray-200 bg-gray-50",
        }
    }
    
    /// 获取块类型的标题颜色
    pub fn title_color(&self) -> &'static str {
        match self {
            BlockType::Sql => "text-green-700",
            BlockType::Json => "text-blue-700",
            BlockType::Error => "text-red-700",
            BlockType::Warning => "text-yellow-700",
            BlockType::Info => "text-gray-700",
            BlockType::Raw => "text-gray-700",
        }
    }
}

/// 块元数据结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    /// 起始行号
    pub line_start: usize,
    /// 结束行号
    pub line_end: usize,
    /// 起始字符位置
    pub char_start: usize,
    /// 结束字符位置
    pub char_end: usize,
    /// 置信度 (0.0 - 1.0)
    pub confidence: f32,
}

impl RenderedBlock {
    /// 创建新的渲染块
    pub fn new(
        id: String,
        block_type: BlockType,
        title: String,
        content: String,
        formatted_content: String,
    ) -> Self {
        Self {
            id,
            block_type,
            title,
            content: content.clone(),
            formatted_content,
            is_copyable: true,
            metadata: BlockMetadata {
                line_start: 0,
                line_end: 0,
                char_start: 0,
                char_end: content.len(),
                confidence: 1.0,
            },
        }
    }
    
    /// 创建SQL块
    pub fn sql(id: String, sql: String, formatted_sql: String) -> Self {
        Self::new(id, BlockType::Sql, "SQL 查询".to_string(), sql, formatted_sql)
    }
    
    /// 创建JSON块
    pub fn json(id: String, json: String, formatted_json: String) -> Self {
        Self::new(id, BlockType::Json, "JSON 数据".to_string(), json, formatted_json)
    }
    
    /// 创建错误块
    pub fn error(id: String, error: String) -> Self {
        Self::new(id, BlockType::Error, "错误信息".to_string(), error.clone(), error)
    }
    
    /// 创建警告块
    pub fn warning(id: String, warning: String) -> Self {
        Self::new(id, BlockType::Warning, "警告信息".to_string(), warning.clone(), warning)
    }
    
    /// 创建信息块
    pub fn info(id: String, info: String) -> Self {
        Self::new(id, BlockType::Info, "信息".to_string(), info.clone(), info)
    }
    
    /// 创建原始文本块
    pub fn raw(id: String, content: String) -> Self {
        Self::new(id, BlockType::Raw, "原始文本".to_string(), content.clone(), content)
    }
    
    /// 设置元数据
    pub fn with_metadata(mut self, metadata: BlockMetadata) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// 设置置信度
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.metadata.confidence = confidence.clamp(0.0, 1.0);
        self
    }
    
    /// 设置行号范围
    pub fn with_line_range(mut self, line_start: usize, line_end: usize) -> Self {
        self.metadata.line_start = line_start;
        self.metadata.line_end = line_end;
        self
    }
    
    /// 设置字符位置范围
    pub fn with_char_range(mut self, char_start: usize, char_end: usize) -> Self {
        self.metadata.char_start = char_start;
        self.metadata.char_end = char_end;
        self
    }
    
    /// 检查块是否为空
    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }
    
    /// 获取块的长度
    pub fn len(&self) -> usize {
        self.content.len()
    }
    
    /// 获取格式化的内容长度
    pub fn formatted_len(&self) -> usize {
        self.formatted_content.len()
    }
}
