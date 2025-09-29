use std::fs;
use std::path::Path;
use encoding_rs::{Encoding, UTF_8, GBK, GB18030};

/// 编码检测器
pub struct EncodingDetector {
    /// 用于检测的字节数量
    sample_size: usize,
}

/// 编码检测结果
#[derive(Debug, Clone)]
pub struct EncodingResult {
    pub encoding: &'static Encoding,
    pub confidence: f32,
    pub encoding_name: String,
}

impl EncodingDetector {
    /// 创建新的编码检测器
    pub fn new() -> Self {
        Self {
            sample_size: 8192, // 8KB 样本大小
        }
    }

    /// 设置样本大小
    pub fn with_sample_size(mut self, size: usize) -> Self {
        self.sample_size = size;
        self
    }

    /// 检测文件编码
    pub fn detect_file_encoding(&self, file_path: &str) -> Result<EncodingResult, String> {
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Err(format!("File not found: {}", file_path));
        }

        // 读取文件的前面部分用于编码检测
        let bytes = fs::read(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        self.detect_encoding(&bytes)
    }

    /// 检测字节数组的编码
    pub fn detect_encoding(&self, bytes: &[u8]) -> Result<EncodingResult, String> {
        if bytes.is_empty() {
            return Ok(EncodingResult {
                encoding: UTF_8,
                confidence: 1.0,
                encoding_name: "UTF-8".to_string(),
            });
        }

        // 取样本进行检测
        let sample = if bytes.len() > self.sample_size {
            &bytes[..self.sample_size]
        } else {
            bytes
        };

        // 检查BOM
        if let Some(result) = self.check_bom(sample) {
            return Ok(result);
        }

        // 尝试不同编码的解码成功率
        let encodings = vec![
            (UTF_8, "UTF-8"),
            (GBK, "GBK"),
            (GB18030, "GB18030"),
        ];

        let mut best_result = EncodingResult {
            encoding: UTF_8,
            confidence: 0.0,
            encoding_name: "UTF-8".to_string(),
        };

        for (encoding, name) in encodings {
            let confidence = self.calculate_encoding_confidence(sample, encoding);
            
            if confidence > best_result.confidence {
                best_result = EncodingResult {
                    encoding,
                    confidence,
                    encoding_name: name.to_string(),
                };
            }
        }

        // 如果所有编码的置信度都很低，默认使用UTF-8
        if best_result.confidence < 0.5 {
            best_result = EncodingResult {
                encoding: UTF_8,
                confidence: 0.6,
                encoding_name: "UTF-8".to_string(),
            };
        }

        Ok(best_result)
    }

    /// 检查字节顺序标记(BOM)
    fn check_bom(&self, bytes: &[u8]) -> Option<EncodingResult> {
        if bytes.len() >= 3 {
            // UTF-8 BOM
            if bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
                return Some(EncodingResult {
                    encoding: UTF_8,
                    confidence: 1.0,
                    encoding_name: "UTF-8".to_string(),
                });
            }
        }

        if bytes.len() >= 2 {
            // UTF-16 LE BOM
            if bytes[0] == 0xFF && bytes[1] == 0xFE {
                return Some(EncodingResult {
                    encoding: encoding_rs::UTF_16LE,
                    confidence: 1.0,
                    encoding_name: "UTF-16LE".to_string(),
                });
            }
            
            // UTF-16 BE BOM
            if bytes[0] == 0xFE && bytes[1] == 0xFF {
                return Some(EncodingResult {
                    encoding: encoding_rs::UTF_16BE,
                    confidence: 1.0,
                    encoding_name: "UTF-16BE".to_string(),
                });
            }
        }

        None
    }

    /// 计算编码置信度
    fn calculate_encoding_confidence(&self, bytes: &[u8], encoding: &'static Encoding) -> f32 {
        let (decoded, had_errors) = encoding.decode_without_bom_handling(bytes);
        
        if had_errors {
            return 0.0;
        }

        let text = decoded.as_ref();
        
        // 基本置信度（能解码且无错误）
        let mut confidence = 0.6;
        
        // 检查文本特征
        confidence += self.analyze_text_characteristics(text, encoding);
        
        // 限制置信度在0.0-1.0之间
        confidence.max(0.0).min(1.0)
    }

    /// 分析文本特征
    fn analyze_text_characteristics(&self, text: &str, encoding: &'static Encoding) -> f32 {
        let mut bonus = 0.0;
        
        // 检查是否包含常见的中文字符（对于中文编码）
        if encoding == GBK || encoding == GB18030 {
            let chinese_chars = text.chars().filter(|c| {
                let code = *c as u32;
                // 常见中文字符范围
                (code >= 0x4E00 && code <= 0x9FFF) || // CJK统一汉字
                (code >= 0x3400 && code <= 0x4DBF)    // CJK扩展A
            }).count();
            
            if chinese_chars > 0 {
                bonus += 0.3;
            }
        }

        // 检查是否包含ASCII字符
        let ascii_chars = text.chars().filter(|c| c.is_ascii()).count();
        let total_chars = text.chars().count();
        
        if total_chars > 0 {
            let ascii_ratio = ascii_chars as f32 / total_chars as f32;
            
            // UTF-8适合处理混合ASCII内容
            if encoding == UTF_8 && ascii_ratio > 0.5 {
                bonus += 0.2;
            }
        }

        // 检查常见的日志关键词
        let log_keywords = [
            "INFO", "DEBUG", "WARN", "ERROR", "TRACE", "FATAL",
            "Exception", "Caused by", "at ",
            "日志", "错误", "警告", "信息", "调试"
        ];

        let keyword_matches = log_keywords.iter()
            .filter(|&keyword| text.contains(keyword))
            .count();

        if keyword_matches > 0 {
            bonus += 0.1;
        }

        bonus
    }

    /// 使用检测到的编码读取文件
    pub fn read_file_with_encoding(&self, file_path: &str) -> Result<(String, EncodingResult), String> {
        let encoding_result = self.detect_file_encoding(file_path)?;
        
        let bytes = fs::read(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let (decoded, _had_errors) = encoding_result.encoding.decode_without_bom_handling(&bytes);
        
        Ok((decoded.into_owned(), encoding_result))
    }
}

impl Default for EncodingDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_utf8_detection() {
        let detector = EncodingDetector::new();
        let utf8_text = "Hello, 世界! This is a UTF-8 test.";
        let result = detector.detect_encoding(utf8_text.as_bytes()).unwrap();
        
        assert_eq!(result.encoding, UTF_8);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_empty_bytes() {
        let detector = EncodingDetector::new();
        let result = detector.detect_encoding(&[]).unwrap();
        
        assert_eq!(result.encoding, UTF_8);
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn test_bom_detection() {
        let detector = EncodingDetector::new();
        
        // UTF-8 BOM
        let utf8_bom = [0xEF, 0xBB, 0xBF, b'H', b'e', b'l', b'l', b'o'];
        let result = detector.detect_encoding(&utf8_bom).unwrap();
        
        assert_eq!(result.encoding, UTF_8);
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn test_log_keywords_bonus() {
        let detector = EncodingDetector::new();
        let log_text = "2024-01-01 10:00:00 INFO Starting application...";
        let result = detector.detect_encoding(log_text.as_bytes()).unwrap();
        
        assert_eq!(result.encoding, UTF_8);
        assert!(result.confidence > 0.6);
    }
}