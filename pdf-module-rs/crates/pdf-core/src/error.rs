//! Error types for PDF module
//! Corresponds to Python: exceptions.py

use thiserror::Error;

/// PDF module error enumeration
/// Corresponds to Python: PdfModuleError
#[derive(Debug, Error)]
#[non_exhaustive]
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
    Io(#[from] std::io::Error),

    #[error("Tool registration failed: {0}")]
    ToolRegistration(String),

    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Audit error: {0}")]
    Audit(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Invalid tool definition: {0}")]
    InvalidToolDefinition(String),

    #[error("Plugin load error: {0}")]
    PluginLoad(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Tool '{0}' is already registered")]
    ToolAlreadyRegistered(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("Tool execution timeout after {0}ms")]
    Timeout(u64),

    #[error("Tool unavailable: {0}")]
    ToolUnavailable(String),

    #[error("Discovery failed: {0}")]
    Discovery(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("LLM error: {0}")]
    LLM(String),

    #[error("Missing parameter: {0}")]
    ParameterMissing(String),

    #[error("Invalid parameter type: {0}")]
    ParameterType(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
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
            Self::Io(_) => 500,
            Self::ToolRegistration(_) => 500,
            Self::ToolExecution(_) => 500,
            Self::Validation(_) => 400,
            Self::Storage(_) => 500,
            Self::Audit(_) => 500,
            Self::Config(_) => 500,
            Self::ToolNotFound(_) => 404,
            Self::InvalidToolDefinition(_) => 400,
            Self::PluginLoad(_) => 500,
            Self::Json(_) => 500,
            Self::ToolAlreadyRegistered(_) => 409,
            Self::SchemaValidation(_) => 400,
            Self::Timeout(_) => 408,
            Self::ToolUnavailable(_) => 503,
            Self::Discovery(_) => 500,
            Self::Http(_) => 500,
            Self::Database(_) => 500,
            Self::LLM(_) => 500,
            Self::ParameterMissing(_) => 400,
            Self::ParameterType(_) => 400,
            Self::Unknown(_) => 500,
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
            Self::Io(_) => "IoError",
            Self::ToolRegistration(_) => "ToolRegistrationError",
            Self::ToolExecution(_) => "ToolExecutionError",
            Self::Validation(_) => "ValidationError",
            Self::Storage(_) => "StorageError",
            Self::Audit(_) => "AuditError",
            Self::Config(_) => "ConfigError",
            Self::ToolNotFound(_) => "ToolNotFoundError",
            Self::InvalidToolDefinition(_) => "InvalidToolDefinitionError",
            Self::PluginLoad(_) => "PluginLoadError",
            Self::Json(_) => "JsonError",
            Self::ToolAlreadyRegistered(_) => "ToolAlreadyRegisteredError",
            Self::SchemaValidation(_) => "SchemaValidationError",
            Self::Timeout(_) => "TimeoutError",
            Self::ToolUnavailable(_) => "ToolUnavailableError",
            Self::Discovery(_) => "DiscoveryError",
            Self::Http(_) => "HttpError",
            Self::Database(_) => "DatabaseError",
            Self::LLM(_) => "LLMError",
            Self::ParameterMissing(_) => "ParameterMissingError",
            Self::ParameterType(_) => "ParameterTypeError",
            Self::Unknown(_) => "UnknownError",
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
            Self::Io(e) => pdf_common::PdfError::Io(e),
            Self::ToolRegistration(s) => pdf_common::PdfError::ToolRegistration(s),
            Self::ToolExecution(s) => pdf_common::PdfError::ToolExecution(s),
            Self::Validation(s) => pdf_common::PdfError::Validation(s),
            Self::Storage(s) => pdf_common::PdfError::Storage(s),
            Self::Audit(s) => pdf_common::PdfError::Audit(s),
            Self::Config(s) => pdf_common::PdfError::Config(s),
            Self::ToolNotFound(s) => pdf_common::PdfError::ToolNotFound(s),
            Self::InvalidToolDefinition(s) => pdf_common::PdfError::InvalidToolDefinition(s),
            Self::PluginLoad(s) => pdf_common::PdfError::PluginLoad(s),
            Self::Json(e) => pdf_common::PdfError::Json(e),
            Self::ToolAlreadyRegistered(s) => pdf_common::PdfError::ToolAlreadyRegistered(s),
            Self::SchemaValidation(s) => pdf_common::PdfError::SchemaValidation(s),
            Self::Timeout(ms) => pdf_common::PdfError::Timeout(ms),
            Self::ToolUnavailable(s) => pdf_common::PdfError::ToolUnavailable(s),
            Self::Discovery(s) => pdf_common::PdfError::Discovery(s),
            Self::Http(s) => pdf_common::PdfError::Http(s),
            Self::Database(s) => pdf_common::PdfError::Database(s),
            Self::LLM(s) => pdf_common::PdfError::LLM(s),
            Self::ParameterMissing(s) => pdf_common::PdfError::ParameterMissing(s),
            Self::ParameterType(s) => pdf_common::PdfError::ParameterType(s),
            Self::Unknown(s) => pdf_common::PdfError::Extraction(s),
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
            pdf_common::PdfError::ToolRegistration(s) => Self::ToolRegistration(s),
            pdf_common::PdfError::ToolExecution(s) => Self::ToolExecution(s),
            pdf_common::PdfError::ToolNotFound(s) => Self::ToolNotFound(s),
            pdf_common::PdfError::ToolAlreadyRegistered(s) => Self::ToolAlreadyRegistered(s),
            pdf_common::PdfError::InvalidToolDefinition(s) => Self::InvalidToolDefinition(s),
            pdf_common::PdfError::PluginLoad(s) => Self::PluginLoad(s),
            pdf_common::PdfError::ToolUnavailable(s) => Self::ToolUnavailable(s),
            pdf_common::PdfError::Discovery(s) => Self::Discovery(s),
            pdf_common::PdfError::Timeout(ms) => Self::Timeout(ms),
            pdf_common::PdfError::Validation(s) => Self::Validation(s),
            pdf_common::PdfError::SchemaValidation(s) => Self::SchemaValidation(s),
            pdf_common::PdfError::Config(s) => Self::Config(s),
            pdf_common::PdfError::Storage(s) => Self::Storage(s),
            pdf_common::PdfError::Audit(s) => Self::Audit(s),
            pdf_common::PdfError::Http(s) => Self::Http(s),
            pdf_common::PdfError::Database(s) => Self::Database(s),
            pdf_common::PdfError::LLM(s) => Self::LLM(s),
            pdf_common::PdfError::ParameterMissing(s) => Self::Validation(s),
            pdf_common::PdfError::ParameterType(s) => Self::Validation(s),
            pdf_common::PdfError::Io(e) => Self::Io(e),
            pdf_common::PdfError::Json(e) => Self::Json(e),
            _ => Self::Unknown(err.to_string()),
        }
    }
}

impl From<serde_yaml::Error> for PdfModuleError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Storage(format!("YAML error: {}", err))
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

    #[test]
    fn test_error_type_naming_consistency() {
        assert_eq!(
            PdfModuleError::Io(std::io::Error::other("test")).error_type(),
            "IoError"
        );
        assert_eq!(
            PdfModuleError::ToolRegistration("test".into()).error_type(),
            "ToolRegistrationError"
        );
        assert_eq!(
            PdfModuleError::ToolExecution("test".into()).error_type(),
            "ToolExecutionError"
        );
        assert_eq!(
            PdfModuleError::Storage("test".into()).error_type(),
            "StorageError"
        );
        assert_eq!(
            PdfModuleError::Audit("test".into()).error_type(),
            "AuditError"
        );
        assert_eq!(
            PdfModuleError::Config("test".into()).error_type(),
            "ConfigError"
        );
        assert_eq!(
            PdfModuleError::PluginLoad("test".into()).error_type(),
            "PluginLoadError"
        );
        assert_eq!(
            PdfModuleError::Json(serde_json::from_str::<serde_json::Value>("invalid").unwrap_err())
                .error_type(),
            "JsonError"
        );
        assert_eq!(
            PdfModuleError::SchemaValidation("test".into()).error_type(),
            "SchemaValidationError"
        );
        assert_eq!(
            PdfModuleError::Discovery("test".into()).error_type(),
            "DiscoveryError"
        );
        assert_eq!(
            PdfModuleError::Http("test".into()).error_type(),
            "HttpError"
        );
        assert_eq!(
            PdfModuleError::Database("test".into()).error_type(),
            "DatabaseError"
        );
        assert_eq!(PdfModuleError::LLM("test".into()).error_type(), "LLMError");
        assert_eq!(
            PdfModuleError::Unknown("test".into()).error_type(),
            "UnknownError"
        );
    }

    #[test]
    fn test_into_unified_conversion() {
        let err = PdfModuleError::FileNotFound("test".into());
        let unified = err.into_unified();
        assert!(matches!(unified, pdf_common::PdfError::FileNotFound(_)));

        let err = PdfModuleError::Http("test".into());
        let unified = err.into_unified();
        assert!(matches!(unified, pdf_common::PdfError::Http(_)));

        let err = PdfModuleError::Database("test".into());
        let unified = err.into_unified();
        assert!(matches!(unified, pdf_common::PdfError::Database(_)));

        let err = PdfModuleError::LLM("test".into());
        let unified = err.into_unified();
        assert!(matches!(unified, pdf_common::PdfError::LLM(_)));

        let err = PdfModuleError::ToolUnavailable("test".into());
        let unified = err.into_unified();
        assert!(matches!(unified, pdf_common::PdfError::ToolUnavailable(_)));
    }

    #[test]
    fn test_from_pdf_error_conversion() {
        let err = pdf_common::PdfError::Http("test".into());
        let module_err: PdfModuleError = err.into();
        assert!(matches!(module_err, PdfModuleError::Http(_)));

        let err = pdf_common::PdfError::Database("test".into());
        let module_err: PdfModuleError = err.into();
        assert!(matches!(module_err, PdfModuleError::Database(_)));

        let err = pdf_common::PdfError::LLM("test".into());
        let module_err: PdfModuleError = err.into();
        assert!(matches!(module_err, PdfModuleError::LLM(_)));

        let err = pdf_common::PdfError::ToolUnavailable("test".into());
        let module_err: PdfModuleError = err.into();
        assert!(matches!(module_err, PdfModuleError::ToolUnavailable(_)));
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original = PdfModuleError::Http("HTTP error".into());
        let unified = original.into_unified();
        let back: PdfModuleError = unified.into();
        assert!(matches!(back, PdfModuleError::Http(_)));
        assert_eq!(back.to_string(), "HTTP error: HTTP error");

        let original = PdfModuleError::ToolUnavailable("tool unavailable".into());
        let unified = original.into_unified();
        let back: PdfModuleError = unified.into();
        assert!(matches!(back, PdfModuleError::ToolUnavailable(_)));
        assert_eq!(back.to_string(), "Tool unavailable: tool unavailable");
    }
}
