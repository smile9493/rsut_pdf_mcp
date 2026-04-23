 //! MCP Server implementation using stdio transport
//! Implements JSON-RPC 2.0 protocol for MCP (Model Context Protocol)

use pdf_core::{dto::*, PathValidationConfig, PdfExtractorService};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Default path validation config for MCP server
fn default_path_config() -> PathValidationConfig {
    PathValidationConfig {
        require_absolute: true,
        allow_traversal: false,
        base_dir: None,
    }
}

/// Run MCP server with stdio transport
pub async fn run_stdio(service: Arc<PdfExtractorService>) -> anyhow::Result<()> {
    info!("Starting MCP server with stdio transport");

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        debug!("Received: {}", line);

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse request: {}", e);
                let response = JsonRpcResponse::error(
                    None,
                    JsonRpcError::parse_error(),
                );
                write_response(&mut stdout_lock, &response)?;
                continue;
            }
        };

        let response = handle_request(&service, request).await;
        write_response(&mut stdout_lock, &response)?;
    }

    Ok(())
}

/// Write JSON-RPC response to stdout
fn write_response(stdout: &mut std::io::StdoutLock, response: &JsonRpcResponse) -> anyhow::Result<()> {
    let json = serde_json::to_string(response)?;
    debug!("Sending: {}", json);
    writeln!(stdout, "{}", json)?;
    stdout.flush()?;
    Ok(())
}

/// Handle a JSON-RPC request
pub async fn handle_request(
    service: &Arc<PdfExtractorService>,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => handle_initialize(&request),
        "tools/list" => handle_tools_list(&request),
        "tools/call" => handle_tools_call(service, &request).await,
        _ => JsonRpcResponse::error(
            request.id,
            JsonRpcError::method_not_found(&request.method),
        ),
    }
}

/// Handle initialize request
fn handle_initialize(request: &JsonRpcRequest) -> JsonRpcResponse {
    let result = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name": "pdf-module-mcp",
            "version": "0.1.0"
        },
        "capabilities": {
            "tools": {}
        }
    });
    JsonRpcResponse::success(request.id.clone(), result)
}

/// Handle tools/list request
fn handle_tools_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    let tools = vec![
        ToolDefinition {
            name: "extract_text".to_string(),
            description: "Extract text content from a PDF file".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute path to the PDF file"
                    },
                    "adapter": {
                        "type": "string",
                        "description": "Extraction engine: lopdf, pdf-extract, pdfium",
                        "enum": ["lopdf", "pdf-extract", "pdfium", "pymupdf", "pdfplumber"]
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolDefinition {
            name: "extract_structured".to_string(),
            description: "Extract structured data with page info and positions".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute path to the PDF file"
                    },
                    "adapter": {
                        "type": "string",
                        "description": "Extraction engine"
                    },
                    "enable_highlight": {
                        "type": "boolean",
                        "description": "Include highlight metadata"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolDefinition {
            name: "get_page_count".to_string(),
            description: "Get the number of pages in a PDF file".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute path to the PDF file"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolDefinition {
            name: "search_keywords".to_string(),
            description: "Search for keywords in a PDF file".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute path to the PDF file"
                    },
                    "keywords": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of keywords to search"
                    },
                    "case_sensitive": {
                        "type": "boolean",
                        "description": "Case sensitive search"
                    },
                    "context_length": {
                        "type": "integer",
                        "description": "Context characters around match (default: 50)"
                    }
                },
                "required": ["file_path", "keywords"]
            }),
        },
        ToolDefinition {
            name: "extract_keywords".to_string(),
            description: "Auto-extract top keywords by frequency".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute path to the PDF file"
                    },
                    "top_n": {
                        "type": "integer",
                        "description": "Number of top keywords"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolDefinition {
            name: "list_adapters".to_string(),
            description: "List available PDF extraction engines".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "cache_stats".to_string(),
            description: "Get cache statistics".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
    ];

    let result = serde_json::json!({
        "tools": tools
    });
    JsonRpcResponse::success(request.id.clone(), result)
}

/// Handle tools/call request
async fn handle_tools_call(
    service: &Arc<PdfExtractorService>,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    let params = match &request.params {
        Some(p) => p,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                JsonRpcError::invalid_params("Missing params"),
            );
        }
    };

    let tool_name = match params.get("name").and_then(|n| n.as_str()) {
        Some(name) => name,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                JsonRpcError::invalid_params("Missing tool name"),
            );
        }
    };

    let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

    let result = match tool_name {
        "extract_text" => handle_extract_text(service, &arguments).await,
        "extract_structured" => handle_extract_structured(service, &arguments).await,
        "get_page_count" => handle_get_page_count(service, &arguments).await,
        "search_keywords" => handle_search_keywords(service, &arguments).await,
        "extract_keywords" => handle_extract_keywords(service, &arguments).await,
        "list_adapters" => handle_list_adapters(service),
        "cache_stats" => handle_cache_stats(service),
        _ => {
            return JsonRpcResponse::error(
                request.id.clone(),
                JsonRpcError::invalid_params(&format!("Unknown tool: {}", tool_name)),
            );
        }
    };

    match result {
        Ok(content) => JsonRpcResponse::success(
            request.id.clone(),
            serde_json::json!({ "content": content }),
        ),
        Err(e) => JsonRpcResponse::error(
            request.id.clone(),
            JsonRpcError::internal_error(&e.to_string()),
        ),
    }
}

/// Handle extract_text tool
async fn handle_extract_text(
    service: &Arc<PdfExtractorService>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let file_path_str = args["file_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
    let file_path = std::path::Path::new(file_path_str);
    
    // Validate path safety
    pdf_core::FileValidator::validate_path_safety(file_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;
    
    let adapter = args["adapter"].as_str();

    let result = service
        .extract_text(file_path, adapter)
        .await?;

    Ok(vec![Content::text(result.extracted_text)])
}

/// Handle extract_structured tool
async fn handle_extract_structured(
    service: &Arc<PdfExtractorService>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let file_path_str = args["file_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
    let file_path = std::path::Path::new(file_path_str);
    
    // Validate path safety
    pdf_core::FileValidator::validate_path_safety(file_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;
    
    let adapter = args["adapter"].as_str();

    let result = service
        .extract_structured(
            file_path,
            adapter,
            &ExtractOptions::default(),
        )
        .await?;

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

/// Handle get_page_count tool
async fn handle_get_page_count(
    service: &Arc<PdfExtractorService>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let file_path_str = args["file_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
    let file_path = std::path::Path::new(file_path_str);
    
    // Validate path safety
    pdf_core::FileValidator::validate_path_safety(file_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

    let count = service.get_page_count(file_path).await?;

    Ok(vec![Content::text(format!("{}", count))])
}

/// Handle search_keywords tool
async fn handle_search_keywords(
    service: &Arc<PdfExtractorService>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let file_path_str = args["file_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
    let file_path = std::path::Path::new(file_path_str);
    
    // Validate path safety
    pdf_core::FileValidator::validate_path_safety(file_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;
    
    let keywords: Vec<String> = args["keywords"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Missing keywords"))?
        .iter()
        .filter_map(|k| k.as_str().map(|s| s.to_string()))
        .collect();

    let case_sensitive = args["case_sensitive"].as_bool().unwrap_or(false);
    let context_length = args["context_length"].as_u64().unwrap_or(50) as usize;

    let result = service
        .search_keywords_with_options(file_path, &keywords, context_length, case_sensitive)
        .await?;

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

/// Handle extract_keywords tool
async fn handle_extract_keywords(
    service: &Arc<PdfExtractorService>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let file_path_str = args["file_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
    let file_path = std::path::Path::new(file_path_str);
    
    // Validate path safety
    pdf_core::FileValidator::validate_path_safety(file_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;
    
    let top_n = args["top_n"].as_u64().unwrap_or(10) as usize;

    let result = service
        .extract_keywords(file_path, 2, 20, top_n)
        .await?;

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

/// Handle list_adapters tool
fn handle_list_adapters(service: &Arc<PdfExtractorService>) -> anyhow::Result<Vec<Content>> {
    let adapters = service.list_engines();
    Ok(vec![Content::text(serde_json::to_string_pretty(&adapters)?)])
}

/// Handle cache_stats tool
fn handle_cache_stats(service: &Arc<PdfExtractorService>) -> anyhow::Result<Vec<Content>> {
    let stats = service.cache_stats();
    Ok(vec![Content::text(serde_json::to_string_pretty(&stats)?)])
}

// ============ JSON-RPC Types ============

/// JSON-RPC Request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC Response
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    fn success(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Option<serde_json::Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC Error
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcError {
    fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
        }
    }

    fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method),
        }
    }

    fn invalid_params(msg: &str) -> Self {
        Self {
            code: -32602,
            message: msg.to_string(),
        }
    }

    fn internal_error(msg: &str) -> Self {
        Self {
            code: -32603,
            message: msg.to_string(),
        }
    }
}

/// Tool definition
#[derive(Debug, Serialize)]
struct ToolDefinition {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

/// Content for tool result
#[derive(Debug, Serialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

impl Content {
    fn text(text: String) -> Self {
        Self {
            content_type: "text".to_string(),
            text,
        }
    }
}
