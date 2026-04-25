//! Audit logger implementation
//! Provides audit logging with storage, querying, and sensitive field masking

use crate::audit::{AuditFilter, AuditLog};
use crate::dto::ExecutionStatus;
use crate::error::{PdfModuleError, PdfResult};
use crate::storage::FileStorage;
use serde_json::Value;
use std::sync::Arc;

/// Audit logger
/// Records and queries audit logs with sensitive field masking
pub struct AuditLogger {
    /// File storage backend
    storage: Arc<dyn FileStorage>,
    /// Sensitive field names to mask
    sensitive_fields: Vec<String>,
    /// Log retention period in days
    retention_days: u32,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(storage: Arc<dyn FileStorage>, sensitive_fields: Vec<String>, retention_days: u32) -> Self {
        Self {
            storage,
            sensitive_fields,
            retention_days,
        }
    }

    /// Create with default sensitive fields
    pub fn with_defaults(storage: Arc<dyn FileStorage>) -> Self {
        let sensitive_fields = vec![
            "password".to_string(),
            "api_key".to_string(),
            "token".to_string(),
            "secret".to_string(),
            "credential".to_string(),
            "authorization".to_string(),
        ];
        Self::new(storage, sensitive_fields, 90)
    }

    /// Log an audit entry
    pub async fn log(&self, mut log: AuditLog) -> PdfResult<()> {
        // Mask sensitive fields
        log.mask_sensitive_fields(&self.sensitive_fields);

        let log_json = serde_json::to_string_pretty(&log)
            .map_err(|e| PdfModuleError::AuditError(format!("Failed to serialize audit log: {}", e)))?;

        let path = format!(
            "audit/{}/{}/{}.json",
            log.timestamp.format("%Y-%m-%d"),
            log.tool_name,
            log.id
        );

        self.storage.write(&path, log_json.as_bytes()).await?;
        Ok(())
    }

    /// Query audit logs with filter
    pub async fn query(
        &self,
        filter: &AuditFilter,
        page: u32,
        page_size: u32,
    ) -> PdfResult<Vec<AuditLog>> {
        let prefix = if let Some(ref tool_name) = filter.tool_name {
            format!("audit/{}/", tool_name)
        } else {
            "audit/".to_string()
        };

        let files = self.storage.list(&prefix, true).await?;
        let mut logs: Vec<AuditLog> = Vec::new();

        for file in files.iter().skip((page as usize) * (page_size as usize)).take(page_size as usize) {
            if let Ok(data) = self.storage.read(&file.path).await {
                if let Ok(log) = serde_json::from_slice::<AuditLog>(&data) {
                    if filter.matches(&log) {
                        logs.push(log);
                    }
                }
            }
        }

        Ok(logs)
    }

    /// Get sensitive fields list
    pub fn sensitive_fields(&self) -> &[String] {
        &self.sensitive_fields
    }

    /// Get retention days
    pub fn retention_days(&self) -> u32 {
        self.retention_days
    }

    /// Create a success audit log entry
    pub fn create_success_log(
        tool_name: &str,
        execution_id: &str,
        elapsed_time_ms: u64,
        params: Option<Value>,
        caller: Option<String>,
    ) -> AuditLog {
        let mut log = AuditLog::new(
            tool_name.to_string(),
            uuid::Uuid::parse_str(execution_id).unwrap_or(uuid::Uuid::new_v4()),
            ExecutionStatus::Success,
            elapsed_time_ms,
        );
        if let Some(p) = params {
            log = log.with_params(p);
        }
        if let Some(c) = caller {
            log = log.with_caller(c);
        }
        log
    }

    /// Create a failure audit log entry
    pub fn create_failure_log(
        tool_name: &str,
        execution_id: &str,
        elapsed_time_ms: u64,
        error_message: &str,
        params: Option<Value>,
        caller: Option<String>,
    ) -> AuditLog {
        let mut log = AuditLog::new(
            tool_name.to_string(),
            uuid::Uuid::parse_str(execution_id).unwrap_or(uuid::Uuid::new_v4()),
            ExecutionStatus::Failed,
            elapsed_time_ms,
        )
        .with_error(error_message.to_string());
        if let Some(p) = params {
            log = log.with_params(p);
        }
        if let Some(c) = caller {
            log = log.with_caller(c);
        }
        log
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_success_log() {
        let log = AuditLogger::create_success_log(
            "test_tool",
            "00000000-0000-0000-0000-000000000001",
            100,
            Some(serde_json::json!({"key": "value"})),
            Some("user1".to_string()),
        );

        assert_eq!(log.tool_name, "test_tool");
        assert_eq!(log.status, ExecutionStatus::Success);
        assert_eq!(log.elapsed_time_ms, 100);
        assert!(log.params.is_some());
        assert!(log.caller.is_some());
    }

    #[test]
    fn test_create_failure_log() {
        let log = AuditLogger::create_failure_log(
            "test_tool",
            "00000000-0000-0000-0000-000000000001",
            50,
            "Something went wrong",
            None,
            None,
        );

        assert_eq!(log.tool_name, "test_tool");
        assert_eq!(log.status, ExecutionStatus::Failed);
        assert_eq!(log.elapsed_time_ms, 50);
        assert!(log.error_message.is_some());
    }
}
