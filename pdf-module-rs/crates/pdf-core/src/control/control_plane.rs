//! Control plane trait
//! Defines the interface for system control and monitoring

use crate::audit::{AuditFilter, AuditLog};
use crate::dto::ExecutionMetric;
use crate::error::PdfResult;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Health status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Schema definition
#[derive(Debug, Clone)]
pub struct SchemaDefinition {
    /// Schema name
    pub name: String,
    /// Schema version
    pub version: String,
    /// JSON Schema
    pub schema: Value,
}

impl SchemaDefinition {
    /// Create a new schema definition
    pub fn new(name: String, version: String, schema: Value) -> Self {
        Self {
            name,
            version,
            schema,
        }
    }
}

/// Control plane trait
/// Defines the interface for system control and monitoring
#[async_trait]
pub trait ControlPlane: Send + Sync {
    /// Log an audit entry
    async fn log_audit(&self, log: AuditLog) -> PdfResult<()>;

    /// Query audit logs
    async fn query_audit_logs(
        &self,
        filter: AuditFilter,
        page: u32,
        page_size: u32,
    ) -> PdfResult<Vec<AuditLog>>;

    /// Register a schema
    async fn register_schema(&self, schema: SchemaDefinition) -> PdfResult<()>;

    /// Get a schema by name
    async fn get_schema(&self, name: &str) -> PdfResult<Option<SchemaDefinition>>;

    /// Check rate limit
    async fn check_rate_limit(&self, tool_name: &str, caller: &str) -> PdfResult<bool>;

    /// Record execution metric
    async fn record_metrics(&self, metric: ExecutionMetric) -> PdfResult<()>;

    /// Get health status of all components
    async fn get_health_status(&self) -> HashMap<String, HealthStatus>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_definition_creation() {
        let schema = SchemaDefinition::new(
            "test_schema".to_string(),
            "1.0.0".to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        );

        assert_eq!(schema.name, "test_schema");
        assert_eq!(schema.version, "1.0.0");
    }
}
