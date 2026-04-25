//! Database Plugin
//! Wraps database operations as a tool plugin

use crate::dto::{
    ExecutionMetadata, InputType, OutputType, Parameter, ParameterType, ToolExecutionResult,
};
use crate::error::{PdfModuleError, PdfResult};
use crate::plugin::ToolHandler;
use crate::protocol::{RuntimeVariables, ToolDefinition, ToolSpec};
use crate::streamer::MessageStreamer;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Database Plugin
/// Provides database query and save operations as a tool plugin
pub struct DatabasePlugin {
    definition: ToolDefinition,
    spec: ToolSpec,
    variables: RuntimeVariables,
}

impl DatabasePlugin {
    /// Create a new database plugin
    pub fn new() -> Self {
        let definition = ToolDefinition::new(
            "Database Operations".to_string(),
            "db_operation".to_string(),
            "Execute database query or save operations".to_string(),
            vec![
                Parameter {
                    name: "operation".to_string(),
                    param_type: ParameterType::String,
                    description: "Operation type (query/save)".to_string(),
                    required: true,
                    default: None,
                    enum_values: Some(vec!["query".to_string(), "save".to_string()]),
                },
                Parameter {
                    name: "table".to_string(),
                    param_type: ParameterType::String,
                    description: "Database table name".to_string(),
                    required: true,
                    default: None,
                    enum_values: None,
                },
                Parameter {
                    name: "data".to_string(),
                    param_type: ParameterType::Object,
                    description: "Data to save (required for save operation)".to_string(),
                    required: false,
                    default: None,
                    enum_values: None,
                },
                Parameter {
                    name: "filters".to_string(),
                    param_type: ParameterType::Object,
                    description: "Query filters (optional for query operation)".to_string(),
                    required: false,
                    default: None,
                    enum_values: None,
                },
                Parameter {
                    name: "transaction_id".to_string(),
                    param_type: ParameterType::String,
                    description: "Transaction ID for atomic operations".to_string(),
                    required: false,
                    default: None,
                    enum_values: None,
                },
            ],
            InputType::Database,
            OutputType::Database,
        );

        let spec = ToolSpec::new(
            "Database Config".to_string(),
            "Configuration for database operations".to_string(),
        );

        let variables = RuntimeVariables::new(
            "Database Variables".to_string(),
            "Runtime variables for database operations".to_string(),
        );

        Self {
            definition,
            spec,
            variables,
        }
    }
}

impl Default for DatabasePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for DatabasePlugin {
    fn definition(&self) -> &ToolDefinition {
        &self.definition
    }

    fn spec(&self) -> &ToolSpec {
        &self.spec
    }

    fn variables(&self) -> &RuntimeVariables {
        &self.variables
    }

    async fn execute(
        &self,
        params: HashMap<String, Value>,
        _streamer: Option<&mut dyn MessageStreamer>,
    ) -> PdfResult<ToolExecutionResult> {
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PdfModuleError::ValidationFailed("operation parameter is required".to_string())
            })?;

        let table = params
            .get("table")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PdfModuleError::ValidationFailed("table parameter is required".to_string())
            })?;

        let start = std::time::Instant::now();

        let result = match operation {
            "query" => {
                // TODO: Integrate with actual database adapter
                serde_json::json!({
                    "operation": "query",
                    "table": table,
                    "rows": [],
                    "message": "Query executed (placeholder - integrate with sqlx)"
                })
            }
            "save" => {
                let data = params.get("data").ok_or_else(|| {
                    PdfModuleError::ValidationFailed(
                        "data parameter is required for save operation".to_string(),
                    )
                })?;

                // TODO: Integrate with actual database adapter
                serde_json::json!({
                    "operation": "save",
                    "table": table,
                    "saved_count": 1,
                    "data": data,
                    "message": "Save executed (placeholder - integrate with sqlx)"
                })
            }
            _ => {
                return Err(PdfModuleError::ValidationFailed(format!(
                    "Invalid operation '{}'. Valid operations: query, save",
                    operation
                )));
            }
        };

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(ToolExecutionResult {
            workflow_id: uuid::Uuid::new_v4().to_string(),
            elapsed_time: elapsed,
            output: result,
            metadata: Some(ExecutionMetadata {
                file_name: String::new(),
                file_size: 0,
                processing_time: elapsed,
                cache_hit: false,
                adapter_used: "database".to_string(),
            }),
        })
    }

    fn validate_params(&self, params: &HashMap<String, Value>) -> PdfResult<()> {
        if !params.contains_key("operation") {
            return Err(PdfModuleError::ValidationFailed(
                "operation parameter is required".to_string(),
            ));
        }

        if let Some(op) = params.get("operation").and_then(|v| v.as_str()) {
            if op != "query" && op != "save" {
                return Err(PdfModuleError::ValidationFailed(format!(
                    "Invalid operation '{}'. Valid: query, save",
                    op
                )));
            }
        }

        if !params.contains_key("table") {
            return Err(PdfModuleError::ValidationFailed(
                "table parameter is required".to_string(),
            ));
        }

        Ok(())
    }

    fn metadata(&self) -> ExecutionMetadata {
        ExecutionMetadata {
            file_name: String::new(),
            file_size: 0,
            processing_time: 0,
            cache_hit: false,
            adapter_used: "database".to_string(),
        }
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["database".to_string(), "transaction".to_string()]
    }

    fn category(&self) -> String {
        "storage".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_plugin_creation() {
        let plugin = DatabasePlugin::new();
        assert_eq!(plugin.name(), "db_operation");
        assert_eq!(plugin.category(), "storage");
        assert!(plugin.capabilities().contains(&"database".to_string()));
    }
}
