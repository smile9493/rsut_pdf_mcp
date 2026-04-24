//! ETL 错误类型定义
//!
//! 提供统一的错误类型，支持 JSON 序列化输出

use serde::{Serialize, Serializer};
use thiserror::Error;

/// ETL 统一错误类型
#[derive(Debug, Error)]
pub enum EtlError {
    /// PDF 提取错误
    #[error("PDF extraction failed: {0}")]
    ExtractionError(String),

    /// Schema 定义或验证错误
    #[error("Schema error: {0}")]
    SchemaError(String),

    /// LLM 调用错误
    #[error("LLM error: {0}")]
    LLMError(String),

    /// 数据库操作错误
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// 数据验证错误
    #[error("Validation failed: {0}, details: {1:?}")]
    ValidationError(String, Vec<String>),

    /// 配置错误
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// 超时错误
    #[error("Timeout: {0}")]
    TimeoutError(String),

    /// IO 错误
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON 序列化/反序列化错误
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// HTTP 请求错误
    #[error("HTTP request error: {0}")]
    HttpError(String),

    /// 文件未找到错误
    #[error("File not found: {0}")]
    FileNotFoundError(String),

    /// PDF 文件损坏错误
    #[error("Corrupted PDF: {0}")]
    CorruptedPdfError(String),

    /// 参数缺失错误
    #[error("Missing parameter: {0}")]
    ParameterMissingError(String),

    /// 参数类型错误
    #[error("Invalid parameter type: {0}")]
    ParameterTypeError(String),
}

impl EtlError {
    /// 转换为 JSON 格式的错误信息
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            EtlError::ValidationError(msg, details) => {
                serde_json::json!({
                    "error": "validation_error",
                    "message": msg,
                    "details": details
                })
            }
            EtlError::IoError(e) => {
                serde_json::json!({
                    "error": "io_error",
                    "message": e.to_string()
                })
            }
            EtlError::JsonError(e) => {
                serde_json::json!({
                    "error": "json_error",
                    "message": e.to_string()
                })
            }
            _ => {
                let (error_type, message) = match self {
                    EtlError::ExtractionError(msg) => ("extraction_error", msg.as_str()),
                    EtlError::SchemaError(msg) => ("schema_error", msg.as_str()),
                    EtlError::LLMError(msg) => ("llm_error", msg.as_str()),
                    EtlError::DatabaseError(msg) => ("database_error", msg.as_str()),
                    EtlError::ConfigError(msg) => ("config_error", msg.as_str()),
                    EtlError::TimeoutError(msg) => ("timeout_error", msg.as_str()),
                    EtlError::HttpError(msg) => ("http_error", msg.as_str()),
                    EtlError::FileNotFoundError(msg) => ("file_not_found", msg.as_str()),
                    EtlError::CorruptedPdfError(msg) => ("corrupted_pdf", msg.as_str()),
                    EtlError::ParameterMissingError(msg) => ("parameter_missing", msg.as_str()),
                    EtlError::ParameterTypeError(msg) => ("parameter_type_error", msg.as_str()),
                    _ => unreachable!(),
                };
                serde_json::json!({
                    "error": error_type,
                    "message": message
                })
            }
        }
    }

    /// 获取错误类型名称
    pub fn error_type(&self) -> &'static str {
        match self {
            EtlError::ExtractionError(_) => "extraction_error",
            EtlError::SchemaError(_) => "schema_error",
            EtlError::LLMError(_) => "llm_error",
            EtlError::DatabaseError(_) => "database_error",
            EtlError::ValidationError(_, _) => "validation_error",
            EtlError::ConfigError(_) => "config_error",
            EtlError::TimeoutError(_) => "timeout_error",
            EtlError::IoError(_) => "io_error",
            EtlError::JsonError(_) => "json_error",
            EtlError::HttpError(_) => "http_error",
            EtlError::FileNotFoundError(_) => "file_not_found",
            EtlError::CorruptedPdfError(_) => "corrupted_pdf",
            EtlError::ParameterMissingError(_) => "parameter_missing",
            EtlError::ParameterTypeError(_) => "parameter_type_error",
        }
    }
}

impl Serialize for EtlError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_json().serialize(serializer)
    }
}

// 实现 From 转换，支持从其他错误类型自动转换

impl From<sqlx::Error> for EtlError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => EtlError::DatabaseError("Row not found".to_string()),
            sqlx::Error::Database(msg) => EtlError::DatabaseError(msg.to_string()),
            _ => EtlError::DatabaseError(err.to_string()),
        }
    }
}

impl From<reqwest::Error> for EtlError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            EtlError::TimeoutError(err.to_string())
        } else if err.is_connect() {
            EtlError::HttpError(format!("Connection failed: {}", err))
        } else {
            EtlError::HttpError(err.to_string())
        }
    }
}

impl From<jsonschema::ValidationError<'_>> for EtlError {
    fn from(err: jsonschema::ValidationError) -> Self {
        EtlError::ValidationError(
            err.to_string(),
            vec![format!("Path: {}", err.instance_path)],
        )
    }
}

/// ETL Result 类型别名
pub type Result<T> = std::result::Result<T, EtlError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_error() {
        let err = EtlError::ExtractionError("test error".to_string());
        assert_eq!(err.error_type(), "extraction_error");
        let json = err.to_json();
        assert_eq!(json["error"], "extraction_error");
        assert_eq!(json["message"], "test error");
    }

    #[test]
    fn test_validation_error() {
        let err = EtlError::ValidationError(
            "Invalid data".to_string(),
            vec![
                "field1 is required".to_string(),
                "field2 must be positive".to_string(),
            ],
        );
        let json = err.to_json();
        assert_eq!(json["error"], "validation_error");
        assert_eq!(json["message"], "Invalid data");
        assert!(json["details"].is_array());
    }

    #[test]
    fn test_from_sqlx_error() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let etl_err: EtlError = sqlx_err.into();
        assert!(matches!(etl_err, EtlError::DatabaseError(_)));
    }
}
