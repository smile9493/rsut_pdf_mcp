//! MiniMax Adapter Plugin
//! Wraps MiniMax remote AI service as a tool plugin

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

/// MiniMax configuration
#[derive(Debug, Clone)]
pub struct MiniMaxConfig {
    /// Base URL for MiniMax API
    pub base_url: String,
    /// API key for authentication
    pub api_key: String,
    /// Default model name
    pub default_model: String,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for MiniMaxConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.minimax.chat".to_string(),
            api_key: String::new(),
            default_model: "abab6.5-chat".to_string(),
            timeout_ms: 30000,
        }
    }
}

/// MiniMax Adapter Plugin
/// Provides MiniMax AI chat completion as a tool plugin
pub struct MiniMaxAdapterPlugin {
    definition: ToolDefinition,
    spec: ToolSpec,
    variables: RuntimeVariables,
    config: MiniMaxConfig,
}

impl MiniMaxAdapterPlugin {
    /// Create a new MiniMax adapter plugin
    pub fn new(config: MiniMaxConfig) -> Self {
        let definition = ToolDefinition::new(
            "MiniMax Chat".to_string(),
            "minimax_chat".to_string(),
            "MiniMax AI chat completion for text generation and analysis".to_string(),
            vec![
                Parameter {
                    name: "prompt".to_string(),
                    param_type: ParameterType::String,
                    description: "User prompt for chat completion".to_string(),
                    required: true,
                    default: None,
                    enum_values: None,
                },
                Parameter {
                    name: "model".to_string(),
                    param_type: ParameterType::String,
                    description: "Model name".to_string(),
                    required: false,
                    default: Some(Value::String(config.default_model.clone())),
                    enum_values: None,
                },
                Parameter {
                    name: "temperature".to_string(),
                    param_type: ParameterType::Number,
                    description: "Sampling temperature (0.0-1.0)".to_string(),
                    required: false,
                    default: Some(Value::Number(
                        serde_json::Number::from_f64(0.7).unwrap(),
                    )),
                    enum_values: None,
                },
                Parameter {
                    name: "max_tokens".to_string(),
                    param_type: ParameterType::Number,
                    description: "Maximum tokens to generate".to_string(),
                    required: false,
                    default: Some(Value::Number(serde_json::Number::from(4096))),
                    enum_values: None,
                },
            ],
            InputType::Text,
            OutputType::Json,
        );

        let spec = ToolSpec::new(
            "MiniMax Config".to_string(),
            "Configuration for MiniMax chat completion".to_string(),
        );

        let variables = RuntimeVariables::new(
            "MiniMax Variables".to_string(),
            "Runtime variables for MiniMax operations".to_string(),
        );

        Self {
            definition,
            spec,
            variables,
            config,
        }
    }
}

#[async_trait]
impl ToolHandler for MiniMaxAdapterPlugin {
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
        let prompt = params
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PdfModuleError::ValidationFailed("prompt parameter is required".to_string())
            })?;

        let model = params
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.config.default_model);

        let temperature = params
            .get("temperature")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7);

        let start = std::time::Instant::now();

        // TODO: Integrate with actual HTTP client (reqwest)
        // The actual implementation will call:
        // self.client.post(&format!("{}/chat/completions", self.config.base_url))
        //     .header("Authorization", format!("Bearer {}", self.config.api_key))
        //     .json(&request_body)
        //     .send().await?

        let result = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "temperature": temperature,
            "status": "completed",
            "message": "MiniMax chat executed (placeholder - integrate with reqwest)"
        });

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
                adapter_used: "minimax".to_string(),
            }),
        })
    }

    fn validate_params(&self, params: &HashMap<String, Value>) -> PdfResult<()> {
        if !params.contains_key("prompt") {
            return Err(PdfModuleError::ValidationFailed(
                "prompt parameter is required".to_string(),
            ));
        }

        if let Some(temp) = params.get("temperature").and_then(|v| v.as_f64()) {
            if !(0.0..=1.0).contains(&temp) {
                return Err(PdfModuleError::ValidationFailed(
                    "temperature must be between 0.0 and 1.0".to_string(),
                ));
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
            adapter_used: "minimax".to_string(),
        }
    }

    async fn is_available(&self) -> bool {
        // Check if MiniMax service is available
        // TODO: Implement health check via HTTP
        !self.config.api_key.is_empty()
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "remote".to_string(),
            "llm".to_string(),
            "chat".to_string(),
        ]
    }

    fn category(&self) -> String {
        "ai".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimax_plugin_creation() {
        let config = MiniMaxConfig::default();
        let plugin = MiniMaxAdapterPlugin::new(config);
        assert_eq!(plugin.name(), "minimax_chat");
        assert_eq!(plugin.category(), "ai");
        assert!(plugin.capabilities().contains(&"remote".to_string()));
    }

    #[tokio::test]
    async fn test_minimax_availability() {
        let config = MiniMaxConfig::default();
        let plugin = MiniMaxAdapterPlugin::new(config);
        // Default config has empty api_key, so not available
        assert!(!plugin.is_available().await);
    }
}
