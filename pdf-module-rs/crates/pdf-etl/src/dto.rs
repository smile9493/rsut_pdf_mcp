//! ETL 数据传输对象
//!
//! 定义提取、转换、保存等结果结构体

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// PDF 元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMetadata {
    /// 页数
    pub page_count: usize,
    /// 作者
    pub author: Option<String>,
    /// 标题
    pub title: Option<String>,
    /// 创建时间
    pub created_at: Option<String>,
    /// 文件大小（KB）
    pub file_size_kb: f64,
}

impl Default for PdfMetadata {
    fn default() -> Self {
        Self {
            page_count: 0,
            author: None,
            title: None,
            created_at: None,
            file_size_kb: 0.0,
        }
    }
}

/// 提取结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    /// 提取的文本
    pub text: String,
    /// PDF 元数据
    pub metadata: PdfMetadata,
    /// 文本分块
    pub chunks: Vec<String>,
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
}

impl ExtractionResult {
    /// 创建新的提取结果
    pub fn new(
        text: String,
        metadata: PdfMetadata,
        chunks: Vec<String>,
        processing_time_ms: u64,
    ) -> Self {
        Self {
            text,
            metadata,
            chunks,
            processing_time_ms,
        }
    }

    /// 创建空结果
    pub fn empty() -> Self {
        Self {
            text: String::new(),
            metadata: PdfMetadata::default(),
            chunks: Vec::new(),
            processing_time_ms: 0,
        }
    }
}

/// 转换结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    /// 结构化 JSON 数据
    pub data: Value,
    /// 是否通过验证
    pub is_valid: bool,
    /// 验证错误列表
    pub validation_errors: Vec<String>,
    /// 输入 token 数
    pub input_tokens: usize,
    /// 输出 token 数
    pub output_tokens: usize,
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
    /// 置信度（0.0-1.0）
    pub confidence: Option<f32>,
}

impl TransformResult {
    /// 创建有效的转换结果
    pub fn valid(
        data: Value,
        input_tokens: usize,
        output_tokens: usize,
        processing_time_ms: u64,
    ) -> Self {
        Self {
            data,
            is_valid: true,
            validation_errors: Vec::new(),
            input_tokens,
            output_tokens,
            processing_time_ms,
            confidence: None,
        }
    }

    /// 创建无效的转换结果
    pub fn invalid(
        data: Value,
        errors: Vec<String>,
        input_tokens: usize,
        output_tokens: usize,
        processing_time_ms: u64,
    ) -> Self {
        Self {
            data,
            is_valid: false,
            validation_errors: errors,
            input_tokens,
            output_tokens,
            processing_time_ms,
            confidence: None,
        }
    }

    /// 设置置信度
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = Some(confidence.clamp(0.0, 1.0));
        self
    }
}

/// 保存结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveResult {
    /// 记录 ID
    pub record_id: Value,
    /// 表名
    pub table_name: String,
    /// 保存时间
    pub timestamp: DateTime<Utc>,
    /// 影响行数
    pub affected_rows: u64,
}

impl SaveResult {
    /// 创建新的保存结果
    pub fn new(record_id: Value, table_name: String, affected_rows: u64) -> Self {
        Self {
            record_id,
            table_name,
            timestamp: Utc::now(),
            affected_rows,
        }
    }
}

/// ETL 完整结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ETLResult {
    /// 提取结果
    pub extraction: ExtractionResult,
    /// 转换结果
    pub transform: TransformResult,
    /// 保存结果（可选）
    pub save: Option<SaveResult>,
    /// 总耗时（毫秒）
    pub total_time_ms: u64,
}

impl ETLResult {
    /// 创建新的 ETL 结果
    pub fn new(
        extraction: ExtractionResult,
        transform: TransformResult,
        save: Option<SaveResult>,
        total_time_ms: u64,
    ) -> Self {
        Self {
            extraction,
            transform,
            save,
            total_time_ms,
        }
    }

    /// 获取总 token 消耗
    pub fn total_tokens(&self) -> usize {
        self.transform.input_tokens + self.transform.output_tokens
    }

    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        self.transform.is_valid
    }
}

/// Token 使用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// 输入 token
    pub prompt_tokens: usize,
    /// 输出 token
    pub completion_tokens: usize,
    /// 总 token
    pub total_tokens: usize,
}

impl TokenUsage {
    /// 创建新的 Token 使用量
    pub fn new(prompt_tokens: usize, completion_tokens: usize) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

/// LLM 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// 提供商
    pub provider: String,
    /// 模型名称
    pub model: String,
    /// 最大上下文 token
    pub max_context_tokens: usize,
}

impl ModelInfo {
    /// 创建新的模型信息
    pub fn new(provider: String, model: String, max_context_tokens: usize) -> Self {
        Self {
            provider,
            model,
            max_context_tokens,
        }
    }
}

/// 连接池状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatus {
    /// 总连接数
    pub total_connections: u32,
    /// 空闲连接数
    pub idle_connections: u32,
    /// 最大连接数
    pub max_connections: u32,
}

impl PoolStatus {
    /// 创建新的连接池状态
    pub fn new(total: u32, idle: u32, max: u32) -> Self {
        Self {
            total_connections: total,
            idle_connections: idle,
            max_connections: max,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extraction_result() {
        let result = ExtractionResult::new(
            "test text".to_string(),
            PdfMetadata::default(),
            vec!["chunk1".to_string()],
            100,
        );
        assert_eq!(result.text, "test text");
        assert_eq!(result.chunks.len(), 1);
    }

    #[test]
    fn test_transform_result() {
        let data = json!({"key": "value"});
        let result = TransformResult::valid(data.clone(), 10, 20, 50);
        assert!(result.is_valid);
        assert_eq!(result.input_tokens, 10);
        assert_eq!(result.output_tokens, 20);
    }

    #[test]
    fn test_transform_result_with_confidence() {
        let data = json!({"key": "value"});
        let result = TransformResult::valid(data, 10, 20, 50).with_confidence(0.95);
        assert_eq!(result.confidence, Some(0.95));
    }

    #[test]
    fn test_etl_result() {
        let extraction = ExtractionResult::empty();
        let transform = TransformResult::valid(json!({}), 0, 0, 0);
        let etl_result = ETLResult::new(extraction, transform, None, 100);
        assert!(etl_result.is_success());
        assert_eq!(etl_result.total_tokens(), 0);
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_serialization() {
        let result = ExtractionResult::new(
            "test".to_string(),
            PdfMetadata {
                page_count: 5,
                author: Some("test author".to_string()),
                title: None,
                created_at: None,
                file_size_kb: 100.0,
            },
            vec![],
            50,
        );
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ExtractionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.text, deserialized.text);
        assert_eq!(result.metadata.page_count, deserialized.metadata.page_count);
    }
}
