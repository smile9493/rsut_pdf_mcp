//! Error types for PDF module
//! Corresponds to Python: exceptions.py

use thiserror::Error;

/// PDF module error enumeration
/// Corresponds to Python: PdfModuleError
#[derive(Debug, Error)]
pub enum PdfModuleError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid file type: {0}")]
    InvalidFileType(String),

    #[error("File too large: {0}")]
    FileTooLarge(String),

    #[error("Extraction failed: {0}")]
    Extraction(String),

    #[error("Adapter not found: {0}")]
    AdapterNotFound(String),

    #[error("Corrupted file: {0}")]
    CorruptedFile(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    // === MCP Optimization Errors ===

    #[error("Tool registration failed: {0}")]
    ToolRegistrationError(String),

    #[error("Tool execution failed: {0}")]
    ToolExecutionError(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Audit error: {0}")]
    AuditError(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Message send error: {0}")]
    MessageSendError(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid tool definition: {0}")]
    InvalidToolDefinition(String),

    #[error("Plugin load error: {0}")]
    PluginLoadError(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl PdfModuleError {
    /// Returns HTTP status code for this error
    /// Corresponds to Python: PdfModuleError.status_code
    pub fn status_code(&self) -> u16 {
        match self {
            Self::FileNotFound(_) => 404,
            Self::InvalidFileType(_) | Self::AdapterNotFound(_) => 400,
            Self::FileTooLarge(_) => 413,
            Self::CorruptedFile(_) => 422,
            Self::Extraction(_) => 500,
            Self::IoError(_) => 500,
            Self::ToolRegistrationError(_) => 500,
            Self::ToolExecutionError(_) => 500,
            Self::ValidationFailed(_) => 400,
            Self::StorageError(_) => 500,
            Self::AuditError(_) => 500,
            Self::ConfigError(_) => 500,
            Self::MessageSendError(_) => 500,
            Self::ToolNotFound(_) => 404,
            Self::InvalidToolDefinition(_) => 400,
            Self::PluginLoadError(_) => 500,
            Self::JsonError(_) => 500,
        }
    }

    /// Returns error type name for JSON output
    pub fn error_type(&self) -> &'static str {
        match self {
            Self::FileNotFound(_) => "FileNotFoundError",
            Self::InvalidFileType(_) => "InvalidFileTypeError",
            Self::FileTooLarge(_) => "FileTooLargeError",
            Self::Extraction(_) => "ExtractionError",
            Self::AdapterNotFound(_) => "AdapterNotFoundError",
            Self::CorruptedFile(_) => "CorruptedFileError",
            Self::IoError(_) => "IOError",
            Self::ToolRegistrationError(_) => "ToolRegistrationError",
            Self::ToolExecutionError(_) => "ToolExecutionError",
            Self::ValidationFailed(_) => "ValidationError",
            Self::StorageError(_) => "StorageError",
            Self::AuditError(_) => "AuditError",
            Self::ConfigError(_) => "ConfigError",
            Self::MessageSendError(_) => "MessageSendError",
            Self::ToolNotFound(_) => "ToolNotFoundError",
            Self::InvalidToolDefinition(_) => "InvalidToolDefinitionError",
            Self::PluginLoadError(_) => "PluginLoadError",
            Self::JsonError(_) => "JsonError",
        }
    }

    /// Converts error to JSON format
    /// Corresponds to Python: PdfModuleError.to_dict()
    pub fn to_dict(&self) -> serde_json::Value {
        serde_json::json!({
            "error": self.error_type(),
            "message": self.to_string(),
            "status_code": self.status_code(),
        })
    }
}

/// Result type alias for PDF module operations
pub type PdfResult<T> = Result<T, PdfModuleError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(PdfModuleError::FileNotFound("test".into()).status_code(), 404);
        assert_eq!(PdfModuleError::InvalidFileType("test".into()).status_code(), 400);
        assert_eq!(PdfModuleError::FileTooLarge("test".into()).status_code(), 413);
        assert_eq!(PdfModuleError::Extraction("test".into()).status_code(), 500);
        assert_eq!(PdfModuleError::AdapterNotFound("test".into()).status_code(), 400);
        assert_eq!(PdfModuleError::CorruptedFile("test".into()).status_code(), 422);
    }

    #[test]
    fn test_error_to_dict() {
        let err = PdfModuleError::FileNotFound("/path/to/file.pdf".into());
        let json = err.to_dict();
        assert_eq!(json["error"], "FileNotFoundError");
        assert_eq!(json["status_code"], 404);
        assert!(json["message"].as_str().unwrap().contains("/path/to/file.pdf"));
    }
}
