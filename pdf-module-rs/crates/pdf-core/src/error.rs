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

    // === Plugin Architecture Errors ===
    #[error("Tool '{0}' is already registered")]
    ToolAlreadyRegistered(String),

    #[error("Rate limit exceeded for tool '{0}'")]
    RateLimitExceeded(String),

    #[error("Circuit breaker is open for tool '{0}'")]
    CircuitBreakerOpen(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidationError(String),

    #[error("Tool execution timeout after {0}ms")]
    ExecutionTimeout(u64),

    #[error("Tool '{0}' is unavailable: {1}")]
    ToolUnavailable(String, String),

    #[error("Discovery failed: {0}")]
    DiscoveryError(String),

    #[error("Control plane error: {0}")]
    ControlPlaneError(String),
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
            Self::ToolAlreadyRegistered(_) => 409,
            Self::RateLimitExceeded(_) => 429,
            Self::CircuitBreakerOpen(_) => 503,
            Self::SchemaValidationError(_) => 400,
            Self::ExecutionTimeout(_) => 408,
            Self::ToolUnavailable(_, _) => 503,
            Self::DiscoveryError(_) => 500,
            Self::ControlPlaneError(_) => 500,
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
            Self::ToolAlreadyRegistered(_) => "ToolAlreadyRegisteredError",
            Self::RateLimitExceeded(_) => "RateLimitExceededError",
            Self::CircuitBreakerOpen(_) => "CircuitBreakerOpenError",
            Self::SchemaValidationError(_) => "SchemaValidationError",
            Self::ExecutionTimeout(_) => "ExecutionTimeoutError",
            Self::ToolUnavailable(_, _) => "ToolUnavailableError",
            Self::DiscoveryError(_) => "DiscoveryError",
            Self::ControlPlaneError(_) => "ControlPlaneError",
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

impl PdfModuleError {
    /// Convert this legacy error into the unified `pdf_common::PdfError`.
    ///
    /// This enables gradual migration: existing code continues to use
    /// `PdfModuleError` while new code can accept `PdfError`.
    pub fn into_unified(self) -> pdf_common::PdfError {
        match self {
            Self::FileNotFound(s) => pdf_common::PdfError::FileNotFound(s),
            Self::InvalidFileType(s) => pdf_common::PdfError::InvalidFileType(s),
            Self::FileTooLarge(s) => pdf_common::PdfError::FileTooLarge(s),
            Self::Extraction(s) => pdf_common::PdfError::Extraction(s),
            Self::AdapterNotFound(s) => pdf_common::PdfError::AdapterNotFound(s),
            Self::CorruptedFile(s) => pdf_common::PdfError::CorruptedFile(s),
            Self::IoError(e) => pdf_common::PdfError::Io(e),
            Self::ToolRegistrationError(s) => pdf_common::PdfError::ToolRegistration(s),
            Self::ToolExecutionError(s) => pdf_common::PdfError::ToolExecution(s),
            Self::ValidationFailed(s) => pdf_common::PdfError::Validation(s),
            Self::StorageError(s) => pdf_common::PdfError::Storage(s),
            Self::AuditError(s) => pdf_common::PdfError::Audit(s),
            Self::ConfigError(s) => pdf_common::PdfError::Config(s),
            Self::MessageSendError(s) => pdf_common::PdfError::MessageSend(s),
            Self::ToolNotFound(s) => pdf_common::PdfError::ToolNotFound(s),
            Self::InvalidToolDefinition(s) => pdf_common::PdfError::InvalidToolDefinition(s),
            Self::PluginLoadError(s) => pdf_common::PdfError::PluginLoad(s),
            Self::JsonError(e) => pdf_common::PdfError::Json(e),
            Self::ToolAlreadyRegistered(s) => pdf_common::PdfError::ToolAlreadyRegistered(s),
            Self::RateLimitExceeded(s) => pdf_common::PdfError::RateLimitExceeded(s),
            Self::CircuitBreakerOpen(s) => pdf_common::PdfError::CircuitBreakerOpen(s),
            Self::SchemaValidationError(s) => pdf_common::PdfError::SchemaValidation(s),
            Self::ExecutionTimeout(ms) => pdf_common::PdfError::Timeout(ms),
            Self::ToolUnavailable(a, b) => {
                pdf_common::PdfError::ToolUnavailable(format!("{}: {}", a, b))
            }
            Self::DiscoveryError(s) => pdf_common::PdfError::Discovery(s),
            Self::ControlPlaneError(s) => pdf_common::PdfError::ControlPlane(s),
        }
    }
}

/// Implement `From<pdf_common::PdfError> for PdfModuleError` for backward compatibility.
/// This allows functions that return `PdfResult<T>` to accept `pdf_common::PdfError`
/// via the `?` operator.
impl From<pdf_common::PdfError> for PdfModuleError {
    fn from(err: pdf_common::PdfError) -> Self {
        match err {
            pdf_common::PdfError::FileNotFound(s) => Self::FileNotFound(s),
            pdf_common::PdfError::InvalidFileType(s) => Self::InvalidFileType(s),
            pdf_common::PdfError::FileTooLarge(s) => Self::FileTooLarge(s),
            pdf_common::PdfError::CorruptedFile(s) => Self::CorruptedFile(s),
            pdf_common::PdfError::Extraction(s) => Self::Extraction(s),
            pdf_common::PdfError::AdapterNotFound(s) => Self::AdapterNotFound(s),
            pdf_common::PdfError::ToolRegistration(s) => Self::ToolRegistrationError(s),
            pdf_common::PdfError::ToolExecution(s) => Self::ToolExecutionError(s),
            pdf_common::PdfError::ToolNotFound(s) => Self::ToolNotFound(s),
            pdf_common::PdfError::ToolAlreadyRegistered(s) => Self::ToolAlreadyRegistered(s),
            pdf_common::PdfError::InvalidToolDefinition(s) => Self::InvalidToolDefinition(s),
            pdf_common::PdfError::PluginLoad(s) => Self::PluginLoadError(s),
            pdf_common::PdfError::ToolUnavailable(s) => Self::ToolUnavailable(s.clone(), s),
            pdf_common::PdfError::Discovery(s) => Self::DiscoveryError(s),
            pdf_common::PdfError::RateLimitExceeded(s) => Self::RateLimitExceeded(s),
            pdf_common::PdfError::CircuitBreakerOpen(s) => Self::CircuitBreakerOpen(s),
            pdf_common::PdfError::Timeout(ms) => Self::ExecutionTimeout(ms),
            pdf_common::PdfError::MessageSend(s) => Self::MessageSendError(s),
            pdf_common::PdfError::ControlPlane(s) => Self::ControlPlaneError(s),
            pdf_common::PdfError::Validation(s) => Self::ValidationFailed(s),
            pdf_common::PdfError::SchemaValidation(s) => Self::SchemaValidationError(s),
            pdf_common::PdfError::Config(s) => Self::ConfigError(s),
            pdf_common::PdfError::Storage(s) => Self::StorageError(s),
            pdf_common::PdfError::Audit(s) => Self::AuditError(s),
            pdf_common::PdfError::Http(s) => Self::Extraction(s),
            pdf_common::PdfError::Database(s) => Self::StorageError(s),
            pdf_common::PdfError::LLM(s) => Self::Extraction(s),
            pdf_common::PdfError::ParameterMissing(s) => Self::ValidationFailed(s),
            pdf_common::PdfError::ParameterType(s) => Self::ValidationFailed(s),
            pdf_common::PdfError::Io(e) => Self::IoError(e),
            pdf_common::PdfError::Json(e) => Self::JsonError(e),
        }
    }
}

impl From<serde_yaml::Error> for PdfModuleError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::StorageError(format!("YAML error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            PdfModuleError::FileNotFound("test".into()).status_code(),
            404
        );
        assert_eq!(
            PdfModuleError::InvalidFileType("test".into()).status_code(),
            400
        );
        assert_eq!(
            PdfModuleError::FileTooLarge("test".into()).status_code(),
            413
        );
        assert_eq!(PdfModuleError::Extraction("test".into()).status_code(), 500);
        assert_eq!(
            PdfModuleError::AdapterNotFound("test".into()).status_code(),
            400
        );
        assert_eq!(
            PdfModuleError::CorruptedFile("test".into()).status_code(),
            422
        );
    }

    #[test]
    fn test_error_to_dict() {
        let err = PdfModuleError::FileNotFound("/path/to/file.pdf".into());
        let json = err.to_dict();
        assert_eq!(json["error"], "FileNotFoundError");
        assert_eq!(json["status_code"], 404);
        assert!(json["message"]
            .as_str()
            .unwrap()
            .contains("/path/to/file.pdf"));
    }
}
