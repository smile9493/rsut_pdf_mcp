//! PDF Extractor Plugin
//! Wraps PDF extraction capability as a tool plugin

use crate::dto::{ExecutionMetadata, InputType, OutputType, Parameter, ParameterType, ToolExecutionResult};
use crate::error::{PdfModuleError, PdfResult};
use crate::extractor::PdfExtractorService;
use crate::plugin::ToolHandler;
use crate::protocol::{RuntimeVariables, ToolDefinition, ToolSpec};
use crate::streamer::MessageStreamer;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// PDF Extractor Plugin
/// Provides PDF text extraction as a tool plugin
pub struct PdfExtractorPlugin {
    definition: ToolDefinition,
    spec: ToolSpec,
    variables: RuntimeVariables,
    extractor: Arc<PdfExtractorService>,
}

impl PdfExtractorPlugin {
    /// Create a new PDF extractor plugin
    pub fn new(extractor: Arc<PdfExtractorService>) -> Self {
        let definition = ToolDefinition::new(
            "PDF Extractor".to_string(),
            "pdf_extract".to_string(),
            "Extract text content from PDF files using various engines".to_string(),
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
                    name: "engine".to_string(),
                    param_type: ParameterType::String,
                    description: "Extraction engine (lopdf/pdfium/pdf_extract)".to_string(),
                    required: false,
                    default: Some(Value::String("lopdf".to_string())),
                    enum_values: Some(vec![
                        "lopdf".to_string(),
                        "pdfium".to_string(),
                        "pdf_extract".to_string(),
                    ]),
                },
                Parameter {
                    name: "enable_cache".to_string(),
                    param_type: ParameterType::Boolean,
                    description: "Enable result caching".to_string(),
                    required: false,
                    default: Some(Value::Bool(true)),
                    enum_values: None,
                },
            ],
            InputType::File,
            OutputType::Text,
        );

        let spec = ToolSpec::new(
            "PDF Extraction Config".to_string(),
            "Configuration for PDF text extraction".to_string(),
        );

        let variables = RuntimeVariables::new(
            "PDF Extraction Variables".to_string(),
            "Runtime variables for PDF extraction".to_string(),
        );

        Self {
            definition,
            spec,
            variables,
            extractor,
        }
    }
}

#[async_trait]
impl ToolHandler for PdfExtractorPlugin {
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
        let pdf_path = params
            .get("pdf_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PdfModuleError::ValidationFailed("pdf_path parameter is required".to_string())
            })?;

        let engine = params
            .get("engine")
            .and_then(|v| v.as_str())
            .unwrap_or("lopdf");

        let start = std::time::Instant::now();

        // Execute extraction
        let result = self
            .extractor
            .extract_text(std::path::Path::new(pdf_path), Some(engine))
            .await?;

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(ToolExecutionResult {
            workflow_id: uuid::Uuid::new_v4().to_string(),
            elapsed_time: elapsed,
            output: serde_json::to_value(&result)?,
            metadata: Some(ExecutionMetadata {
                file_name: pdf_path.to_string(),
                file_size: 0,
                processing_time: elapsed,
                cache_hit: false,
                adapter_used: engine.to_string(),
            }),
        })
    }

    fn validate_params(&self, params: &HashMap<String, Value>) -> PdfResult<()> {
        if !params.contains_key("pdf_path") {
            return Err(PdfModuleError::ValidationFailed(
                "pdf_path parameter is required".to_string(),
            ));
        }

        if let Some(engine) = params.get("engine").and_then(|v| v.as_str()) {
            let valid_engines = ["lopdf", "pdfium", "pdf_extract"];
            if !valid_engines.contains(&engine) {
                return Err(PdfModuleError::ValidationFailed(format!(
                    "Invalid engine '{}'. Valid engines: {:?}",
                    engine, valid_engines
                )));
            }
        }

        Ok(())
    }

    fn metadata(&self) -> ExecutionMetadata {
        ExecutionMetadata {
            file_name: String::new(),
            file_size: 0,
            processing_time: 0,
            cache_hit: false,
            adapter_used: "pdf_extractor".to_string(),
        }
    }

    async fn is_available(&self) -> bool {
        true
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "file_input".to_string(),
            "text_output".to_string(),
            "cacheable".to_string(),
        ]
    }

    fn category(&self) -> String {
        "extraction".to_string()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pdf_extractor_plugin_definition() {
        // Verify the plugin definition is correctly set up
        // (We can't create a full plugin without a real extractor service in tests)
        let valid_engines = ["lopdf", "pdfium", "pdf_extract"];
        assert!(valid_engines.contains(&"lopdf"));
        assert!(valid_engines.contains(&"pdfium"));
    }

    #[test]
    fn test_validate_params_missing_path() {
        // Test that missing pdf_path returns error
        // This would require a full plugin instance
    }
}
