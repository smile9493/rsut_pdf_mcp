//! ETL Workflow Plugin
//! Wraps ETL pipeline as a tool plugin
//! Note: This file is placed in pdf-etl crate, but we define the interface here

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

/// ETL Workflow Plugin
/// Provides complete ETL pipeline execution as a tool plugin
pub struct EtlWorkflowPlugin {
    definition: ToolDefinition,
    spec: ToolSpec,
    variables: RuntimeVariables,
}

impl EtlWorkflowPlugin {
    /// Create a new ETL workflow plugin
    pub fn new() -> Self {
        let definition = ToolDefinition::new(
            "ETL Workflow".to_string(),
            "etl_workflow".to_string(),
            "Execute complete ETL pipeline: extract text from PDF, transform via LLM, load to database"
                .to_string(),
            vec![
                Parameter {
                    name: "pdf_path".to_string(),
                    param_type: ParameterType::String,
                    description: "Path to the PDF file".to_string(),
                    required: true,
                    default: None,
                    enum_values: None,
                },
                Parameter {
                    name: "schema".to_string(),
                    param_type: ParameterType::Object,
                    description: "Target JSON Schema for structured extraction".to_string(),
                    required: true,
                    default: None,
                    enum_values: None,
                },
                Parameter {
                    name: "llm_config".to_string(),
                    param_type: ParameterType::Object,
                    description: "LLM configuration (model, api_key, etc.)".to_string(),
                    required: false,
                    default: None,
                    enum_values: None,
                },
                Parameter {
                    name: "db_config".to_string(),
                    param_type: ParameterType::Object,
                    description: "Database configuration".to_string(),
                    required: false,
                    default: None,
                    enum_values: None,
                },
                Parameter {
                    name: "save_to_db".to_string(),
                    param_type: ParameterType::Boolean,
                    description: "Whether to save results to database".to_string(),
                    required: false,
                    default: Some(Value::Bool(false)),
                    enum_values: None,
                },
            ],
            InputType::File,
            OutputType::Json,
        );

        let spec = ToolSpec::new(
            "ETL Workflow Config".to_string(),
            "Configuration for ETL workflow execution".to_string(),
        );

        let variables = RuntimeVariables::new(
            "ETL Workflow Variables".to_string(),
            "Runtime variables for ETL workflow".to_string(),
        );

        Self {
            definition,
            spec,
            variables,
        }
    }
}

impl Default for EtlWorkflowPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolHandler for EtlWorkflowPlugin {
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
        mut streamer: Option<&mut dyn MessageStreamer>,
    ) -> PdfResult<ToolExecutionResult> {
        let pdf_path = params
            .get("pdf_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PdfModuleError::ValidationFailed("pdf_path parameter is required".to_string())
            })?;

        let _schema = params
            .get("schema")
            .ok_or_else(|| {
                PdfModuleError::ValidationFailed("schema parameter is required".to_string())
            })?;

        let save_to_db = params
            .get("save_to_db")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let start = std::time::Instant::now();

        // Send progress message
        if let Some(ref mut s) = streamer {
            let _ = s
                .send_log(crate::dto::LogLevel::Info, "Starting ETL workflow".to_string())
                .await;
        }

        // TODO: Integrate with actual ETL pipeline from pdf-etl crate
        // The actual implementation will call:
        // self.etl_pipeline.execute(pdf_path, schema, llm_config, save_to_db).await

        if let Some(ref mut s) = streamer {
            let _ = s
                .send_log(
                    crate::dto::LogLevel::Info,
                    "ETL workflow completed".to_string(),
                )
                .await;
        }

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(ToolExecutionResult {
            workflow_id: uuid::Uuid::new_v4().to_string(),
            elapsed_time: elapsed,
            output: serde_json::json!({
                "pdf_path": pdf_path,
                "save_to_db": save_to_db,
                "status": "completed",
                "message": "ETL workflow executed (placeholder - integrate with pdf-etl)"
            }),
            metadata: Some(ExecutionMetadata {
                file_name: pdf_path.to_string(),
                file_size: 0,
                processing_time: elapsed,
                cache_hit: false,
                adapter_used: "etl_workflow".to_string(),
            }),
        })
    }

    fn validate_params(&self, params: &HashMap<String, Value>) -> PdfResult<()> {
        if !params.contains_key("pdf_path") {
            return Err(PdfModuleError::ValidationFailed(
                "pdf_path parameter is required".to_string(),
            ));
        }

        if !params.contains_key("schema") {
            return Err(PdfModuleError::ValidationFailed(
                "schema parameter is required".to_string(),
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
            adapter_used: "etl_workflow".to_string(),
        }
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "file_input".to_string(),
            "json_output".to_string(),
            "llm_required".to_string(),
            "db_optional".to_string(),
        ]
    }

    fn category(&self) -> String {
        "etl".to_string()
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["pdf_extract".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etl_workflow_plugin_creation() {
        let plugin = EtlWorkflowPlugin::new();
        assert_eq!(plugin.name(), "etl_workflow");
        assert_eq!(plugin.category(), "etl");
        assert!(plugin.capabilities().contains(&"file_input".to_string()));
        assert!(plugin.dependencies().contains(&"pdf_extract".to_string()));
    }
}
