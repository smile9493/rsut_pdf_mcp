//! MCP Protocol Handler
//! Handles MCP protocol messages and dispatches to plugin registry

use pdf_core::error::PdfResult;
use pdf_core::plugin::{PluginRegistry, ToolHandler};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    pub fn invalid_request() -> Self {
        Self {
            code: -32600,
            message: "Invalid request".to_string(),
            data: None,
        }
    }

    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method),
            data: None,
        }
    }

    pub fn invalid_params(message: &str) -> Self {
        Self {
            code: -32602,
            message: format!("Invalid params: {}", message),
            data: None,
        }
    }

    pub fn internal_error(message: &str) -> Self {
        Self {
            code: -32603,
            message: message.to_string(),
            data: None,
        }
    }
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// MCP Protocol Handler
/// Processes MCP protocol messages and dispatches to the plugin registry
pub struct McpProtocolHandler {
    /// Plugin registry
    registry: Arc<dyn PluginRegistry>,
}

impl McpProtocolHandler {
    /// Create a new protocol handler
    pub fn new(registry: Arc<dyn PluginRegistry>) -> Self {
        Self { registry }
    }

    /// Handle a JSON-RPC request
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        debug!("Handling request: method={}", request.method);

        match request.method.as_str() {
            "initialize" => self.handle_initialize(&request).await,
            "tools/list" => self.handle_tools_list(&request).await,
            "tools/call" => self.handle_tools_call(&request).await,
            "resources/list" => self.handle_resources_list(&request).await,
            "ping" => self.handle_ping(&request),
            _ => {
                JsonRpcResponse::error(request.id, JsonRpcError::method_not_found(&request.method))
            }
        }
    }

    /// Handle initialize request
    async fn handle_initialize(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        info!("MCP initialize request received");

        let tool_count = self.registry.count().await;
        let result = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": {
                "name": "pdf-module-mcp",
                "version": "0.2.0",
                "description": "PDF extraction MCP server with plugin architecture",
                "toolCount": tool_count,
            },
            "capabilities": {
                "tools": { "listChanged": true },
                "resources": { "subscribe": false, "listChanged": false },
            }
        });

        JsonRpcResponse::success(request.id.clone(), result)
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        debug!("tools/list request");

        let definitions = self.registry.list_definitions().await;
        let tools: Vec<Value> = definitions
            .iter()
            .map(|def| {
                serde_json::json!({
                    "name": def.function_name,
                    "description": def.description,
                    "inputSchema": {
                        "type": "object",
                        "properties": def.parameters.iter().map(|p| {
                            (p.name.clone(), serde_json::json!({
                                "type": format!("{:?}", p.param_type).to_lowercase(),
                                "description": p.description,
                            }))
                        }).collect::<HashMap<String, Value>>(),
                        "required": def.parameters.iter()
                            .filter(|p| p.required)
                            .map(|p| p.name.clone())
                            .collect::<Vec<String>>(),
                    }
                })
            })
            .collect();

        let result = serde_json::json!({ "tools": tools });
        JsonRpcResponse::success(request.id.clone(), result)
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let tool_name = request
            .params
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if tool_name.is_empty() {
            return JsonRpcResponse::error(
                request.id.clone(),
                JsonRpcError::invalid_params("tool name is required"),
            );
        }

        let arguments = request
            .params
            .get("arguments")
            .cloned()
            .unwrap_or(Value::Object(serde_json::Map::new()));

        let params: HashMap<String, Value> = match serde_json::from_value(arguments) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    request.id.clone(),
                    JsonRpcError::invalid_params(&format!("Invalid arguments: {}", e)),
                );
            }
        };

        debug!("tools/call: tool={}, params={:?}", tool_name, params);

        match self.registry.get(tool_name).await {
            Ok(tool) => {
                // Validate parameters
                if let Err(e) = tool.validate_params(&params) {
                    return JsonRpcResponse::error(
                        request.id.clone(),
                        JsonRpcError::invalid_params(&e.to_string()),
                    );
                }

                // Execute tool
                match tool.execute(params, None).await {
                    Ok(result) => {
                        let response = serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result.output)
                                    .unwrap_or_else(|_| result.output.to_string()),
                            }],
                            "isError": false,
                        });
                        JsonRpcResponse::success(request.id.clone(), response)
                    }
                    Err(e) => {
                        error!("Tool execution failed: {}", e);
                        let response = serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Error: {}", e),
                            }],
                            "isError": true,
                        });
                        JsonRpcResponse::success(request.id.clone(), response)
                    }
                }
            }
            Err(e) => {
                JsonRpcResponse::error(
                    request.id.clone(),
                    JsonRpcError::internal_error(&format!(
                        "Tool '{}' not found: {}",
                        tool_name, e
                    )),
                )
            }
        }
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let result = serde_json::json!({ "resources": [] });
        JsonRpcResponse::success(request.id.clone(), result)
    }

    /// Handle ping request
    fn handle_ping(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse::success(request.id.clone(), serde_json::json!({}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_error_codes() {
        let parse_err = JsonRpcError::parse_error();
        assert_eq!(parse_err.code, -32700);

        let method_err = JsonRpcError::method_not_found("test");
        assert_eq!(method_err.code, -32601);

        let params_err = JsonRpcError::invalid_params("bad");
        assert_eq!(params_err.code, -32602);

        let internal_err = JsonRpcError::internal_error("fail");
        assert_eq!(internal_err.code, -32603);
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(
            Some(Value::Number(serde_json::Number::from(1))),
            serde_json::json!({"result": "ok"}),
        );
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse::error(
            Some(Value::Number(serde_json::Number::from(1))),
            JsonRpcError::method_not_found("test"),
        );
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }
}
