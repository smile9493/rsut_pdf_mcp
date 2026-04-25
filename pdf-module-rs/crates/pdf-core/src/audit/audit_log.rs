//! Audit log model for plugin architecture
//! 
//! This module provides audit logging functionality for tool execution:
//! - `AuditLog`: Audit record structure for tool execution
//! - `AuditFilter`: Query filter for audit logs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::dto::ExecutionStatus;

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    /// Audit ID
    pub id: Uuid,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Tool name
    pub tool_name: String,
    /// Execution ID
    pub execution_id: Uuid,
    /// Caller information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<String>,
    /// Parameters (masked)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Result summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_summary: Option<String>,
    /// Elapsed time in milliseconds
    pub elapsed_time_ms: u64,
    /// Execution status
    pub status: ExecutionStatus,
    /// Error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl AuditLog {
    /// Create a new audit log entry
    pub fn new(
        tool_name: String,
        execution_id: Uuid,
        status: ExecutionStatus,
        elapsed_time_ms: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            tool_name,
            execution_id,
            caller: None,
            params: None,
            result_summary: None,
            elapsed_time_ms,
            status,
            error_message: None,
        }
    }

    /// Mask sensitive fields in parameters
    pub fn mask_sensitive_fields(&mut self, sensitive_fields: &[String]) {
        if let Some(ref mut params) = self.params {
            *params = Self::mask_value(params.clone(), sensitive_fields);
        }
    }

    /// Recursively mask sensitive fields in a JSON value
    fn mask_value(mut value: Value, sensitive_fields: &[String]) -> Value {
        if let Some(obj) = value.as_object_mut() {
            for field in sensitive_fields {
                if obj.contains_key(field) {
                    obj.insert(field.clone(), Value::String("***MASKED***".to_string()));
                }
            }
            // Recursively process nested objects
            for (_, v) in obj.iter_mut() {
                *v = Self::mask_value(v.clone(), sensitive_fields);
            }
        } else if let Some(arr) = value.as_array_mut() {
            for item in arr.iter_mut() {
                *item = Self::mask_value(item.clone(), sensitive_fields);
            }
        }
        value
    }

    /// Set caller information
    pub fn with_caller(mut self, caller: String) -> Self {
        self.caller = Some(caller);
        self
    }

    /// Set parameters
    pub fn with_params(mut self, params: Value) -> Self {
        self.params = Some(params);
        self
    }

    /// Set result summary
    pub fn with_result_summary(mut self, summary: String) -> Self {
        self.result_summary = Some(summary);
        self
    }

    /// Set error message
    pub fn with_error(mut self, error: String) -> Self {
        self.error_message = Some(error);
        self
    }
}

/// Audit log query filter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditFilter {
    /// Filter by tool name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Filter by start time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<DateTime<Utc>>,
    /// Filter by end time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,
    /// Filter by status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ExecutionStatus>,
    /// Filter by caller
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<String>,
}

impl AuditFilter {
    /// Create a new filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by tool name
    pub fn with_tool_name(mut self, tool_name: String) -> Self {
        self.tool_name = Some(tool_name);
        self
    }

    /// Filter by time range
    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Filter by status
    pub fn with_status(mut self, status: ExecutionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Filter by caller
    pub fn with_caller(mut self, caller: String) -> Self {
        self.caller = Some(caller);
        self
    }

    /// Check if an audit log matches this filter
    pub fn matches(&self, log: &AuditLog) -> bool {
        if let Some(ref tool_name) = self.tool_name {
            if &log.tool_name != tool_name {
                return false;
            }
        }

        if let Some(ref start_time) = self.start_time {
            if log.timestamp < *start_time {
                return false;
            }
        }

        if let Some(ref end_time) = self.end_time {
            if log.timestamp > *end_time {
                return false;
            }
        }

        if let Some(ref status) = self.status {
            if &log.status != status {
                return false;
            }
        }

        if let Some(ref caller) = self.caller {
            if log.caller.as_ref() != Some(caller) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_audit_log_creation() {
        let log = AuditLog::new(
            "test_tool".to_string(),
            Uuid::new_v4(),
            ExecutionStatus::Success,
            100,
        );

        assert_eq!(log.tool_name, "test_tool");
        assert_eq!(log.status, ExecutionStatus::Success);
        assert_eq!(log.elapsed_time_ms, 100);
    }

    #[test]
    fn test_mask_sensitive_fields() {
        let mut log = AuditLog::new(
            "test_tool".to_string(),
            Uuid::new_v4(),
            ExecutionStatus::Success,
            100,
        );

        log.params = Some(json!({
            "username": "test",
            "password": "secret123",
            "api_key": "key123",
            "nested": {
                "token": "token123"
            }
        }));

        let sensitive_fields = vec![
            "password".to_string(),
            "api_key".to_string(),
            "token".to_string(),
        ];

        log.mask_sensitive_fields(&sensitive_fields);

        let params = log.params.unwrap();
        assert_eq!(params["password"], "***MASKED***");
        assert_eq!(params["api_key"], "***MASKED***");
        assert_eq!(params["nested"]["token"], "***MASKED***");
        assert_eq!(params["username"], "test");
    }

    #[test]
    fn test_audit_filter_matching() {
        let log = AuditLog::new(
            "test_tool".to_string(),
            Uuid::new_v4(),
            ExecutionStatus::Success,
            100,
        );

        let filter = AuditFilter::new()
            .with_tool_name("test_tool".to_string())
            .with_status(ExecutionStatus::Success);

        assert!(filter.matches(&log));

        let filter2 = AuditFilter::new()
            .with_tool_name("other_tool".to_string());

        assert!(!filter2.matches(&log));
    }

    #[test]
    fn test_audit_log_serialization() {
        let log = AuditLog::new(
            "test_tool".to_string(),
            Uuid::new_v4(),
            ExecutionStatus::Success,
            100,
        )
        .with_caller("user123".to_string())
        .with_result_summary("Success".to_string());

        let json = serde_json::to_string(&log).unwrap();
        let parsed: AuditLog = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.tool_name, log.tool_name);
        assert_eq!(parsed.caller, log.caller);
        assert_eq!(parsed.result_summary, log.result_summary);
    }
}
