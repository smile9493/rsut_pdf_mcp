//! Remote Plugin Adapter
//! Provides generic remote service proxy as a tool plugin

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

/// Communication protocol
#[derive(Debug, Clone)]
pub enum Protocol {
    Http,
    Grpc,
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub enum AuthConfig {
    /// No authentication
    None,
    /// Bearer token authentication
    BearerToken(String),
    /// API key authentication (header name, key value)
    ApiKey(String, String),
    /// Basic authentication (username, password)
    Basic(String, String),
}

/// Remote plugin configuration
#[derive(Debug, Clone)]
pub struct RemotePluginConfig {
    /// Service endpoint URL
    pub endpoint: String,
    /// Communication protocol
    pub protocol: Protocol,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for RemotePluginConfig {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            protocol: Protocol::Http,
            auth: AuthConfig::None,
            timeout_ms: 30000,
        }
    }
}

/// Remote Plugin Adapter
/// Provides generic remote service proxy as a tool plugin
pub struct RemotePluginAdapter {
    definition: ToolDefinition,
    spec: ToolSpec,
    variables: RuntimeVariables,
    config: RemotePluginConfig,
}

impl RemotePluginAdapter {
    /// Create a new remote plugin adapter
    pub fn new(name: &str, description: &str, config: RemotePluginConfig) -> Self {
        let definition = ToolDefinition::new(
            format!("{} (Remote)", name),
            name.to_string(),
            description.to_string(),
            vec![
                Parameter {
                    name: "method".to_string(),
                    param_type: ParameterType::String,
                    description: "Remote method to call".to_string(),
                    required: true,
                    default: None,
                    enum_values: None,
                },
                Parameter {
                    name: "params".to_string(),
                    param_type: ParameterType::Object,
                    description: "Method parameters".to_string(),
                    required: false,
                    default: Some(Value::Object(serde_json::Map::new())),
                    enum_values: None,
                },
            ],
            InputType::Text,
            OutputType::Json,
        );

        let spec = ToolSpec::new(
            format!("{} Config", name),
            format!("Configuration for remote {} service", name),
        );

        let variables = RuntimeVariables::new(
            format!("{} Variables", name),
            format!("Runtime variables for remote {} service", name),
        );

        Self {
            definition,
            spec,
            variables,
            config,
        }
    }

    /// Build authentication headers
    pub fn build_auth_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        match &self.config.auth {
            AuthConfig::None => {}
            AuthConfig::BearerToken(token) => {
                headers.insert(
                    "Authorization".to_string(),
                    format!("Bearer {}", token),
                );
            }
            AuthConfig::ApiKey(header_name, key) => {
                headers.insert(header_name.clone(), key.clone());
            }
            AuthConfig::Basic(username, password) => {
                // Basic auth: base64 encode "username:password"
                let credentials = format!("{}:{}", username, password);
                // Simple base64 encoding without external dependency
                let encoded = simple_base64_encode(credentials.as_bytes());
                headers.insert(
                    "Authorization".to_string(),
                    format!("Basic {}", encoded),
                );
            }
        }
        headers
    }
}

#[async_trait]
impl ToolHandler for RemotePluginAdapter {
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
        let method = params
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                PdfModuleError::ValidationFailed("method parameter is required".to_string())
            })?;

        let call_params = params
            .get("params")
            .cloned()
            .unwrap_or(Value::Object(serde_json::Map::new()));

        let start = std::time::Instant::now();

        // TODO: Integrate with actual HTTP/gRPC client
        // The actual implementation will:
        // 1. Build request with auth headers
        // 2. Send request to self.config.endpoint
        // 3. Parse response

        let result = serde_json::json!({
            "endpoint": self.config.endpoint,
            "method": method,
            "params": call_params,
            "status": "completed",
            "message": "Remote call executed (placeholder - integrate with reqwest/tonic)"
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
                adapter_used: "remote".to_string(),
            }),
        })
    }

    fn validate_params(&self, params: &HashMap<String, Value>) -> PdfResult<()> {
        if !params.contains_key("method") {
            return Err(PdfModuleError::ValidationFailed(
                "method parameter is required".to_string(),
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
            adapter_used: "remote".to_string(),
        }
    }

    async fn is_available(&self) -> bool {
        // Check if remote service is available
        // TODO: Implement health check
        !self.config.endpoint.is_empty()
    }

    fn capabilities(&self) -> Vec<String> {
        vec!["remote".to_string()]
    }

    fn category(&self) -> String {
        "remote".to_string()
    }
}

/// Simple base64 encoding (minimal implementation to avoid external dependency)
fn simple_base64_encode(input: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::new();
    let chunks = input.chunks(3);
    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        output.push(CHARSET[((triple >> 18) & 0x3F) as usize] as char);
        output.push(CHARSET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            output.push(CHARSET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(CHARSET[(triple & 0x3F) as usize] as char);
        } else {
            output.push('=');
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_plugin_creation() {
        let config = RemotePluginConfig {
            endpoint: "https://api.example.com".to_string(),
            protocol: Protocol::Http,
            auth: AuthConfig::BearerToken("test-token".to_string()),
            timeout_ms: 30000,
        };

        let plugin = RemotePluginAdapter::new("test_service", "Test remote service", config);
        assert_eq!(plugin.name(), "test_service");
        assert_eq!(plugin.category(), "remote");
    }

    #[test]
    fn test_auth_headers_bearer() {
        let config = RemotePluginConfig {
            endpoint: "https://api.example.com".to_string(),
            protocol: Protocol::Http,
            auth: AuthConfig::BearerToken("my-token".to_string()),
            timeout_ms: 30000,
        };

        let plugin = RemotePluginAdapter::new("test", "test", config);
        let headers = plugin.build_auth_headers();
        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer my-token".to_string())
        );
    }

    #[test]
    fn test_auth_headers_api_key() {
        let config = RemotePluginConfig {
            endpoint: "https://api.example.com".to_string(),
            protocol: Protocol::Http,
            auth: AuthConfig::ApiKey("X-API-Key".to_string(), "my-key".to_string()),
            timeout_ms: 30000,
        };

        let plugin = RemotePluginAdapter::new("test", "test", config);
        let headers = plugin.build_auth_headers();
        assert_eq!(headers.get("X-API-Key"), Some(&"my-key".to_string()));
    }
}
