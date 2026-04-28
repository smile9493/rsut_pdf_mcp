use pdf_core::{dto::*, McpPdfPipeline, PathValidationConfig};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::sync::Arc;
use tracing::{debug, error, info};

fn default_path_config() -> PathValidationConfig {
    PathValidationConfig {
        require_absolute: true,
        allow_traversal: false,
        base_dir: None,
    }
}

pub async fn run_stdio(pipeline: Arc<McpPdfPipeline>) -> anyhow::Result<()> {
    info!("MCP server listening on stdio");

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
                let response = JsonRpcResponse::error(None, JsonRpcError::parse_error());
                write_response(&mut stdout_lock, &response)?;
                continue;
            }
        };

        let response = handle_request(&pipeline, request).await;
        write_response(&mut stdout_lock, &response)?;
    }

    Ok(())
}

fn write_response(
    stdout: &mut std::io::StdoutLock,
    response: &JsonRpcResponse,
) -> anyhow::Result<()> {
    let json = serde_json::to_string(response)?;
    debug!("Sending: {}", json);
    writeln!(stdout, "{}", json)?;
    stdout.flush()?;
    Ok(())
}

pub async fn handle_request(
    pipeline: &Arc<McpPdfPipeline>,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => handle_initialize(&request),
        "tools/list" => handle_tools_list(&request),
        "tools/call" => handle_tools_call(pipeline, &request).await,
        _ => JsonRpcResponse::error(request.id, JsonRpcError::method_not_found(&request.method)),
    }
}

fn handle_initialize(request: &JsonRpcRequest) -> JsonRpcResponse {
    let result = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name": "pdf-mcp",
            "version": "0.3.0",
            "description": "Pure PDF extraction MCP pipe — pdfium engine, stdio only, zero state"
        },
        "capabilities": {
            "tools": { "listChanged": false }
        },
        "instructions": "PDF extraction pipe. Tools: extract_text, extract_structured, get_page_count"
    });
    JsonRpcResponse::success(request.id.clone(), result)
}

fn handle_tools_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    let tools = vec![
        ToolDefinition {
            name: "extract_text".to_string(),
            description: "Extract plain text from a PDF file using pdfium engine".to_string(),
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
            name: "extract_structured".to_string(),
            description: "Extract structured data (per-page text + bbox) from PDF".to_string(),
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
    ];

    JsonRpcResponse::success(request.id.clone(), serde_json::json!({ "tools": tools }))
}

async fn handle_tools_call(
    pipeline: &Arc<McpPdfPipeline>,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    let params = match &request.params {
        Some(p) => p,
        None => return JsonRpcResponse::error(request.id.clone(), JsonRpcError::invalid_params("Missing params")),
    };

    let tool_name = match params.get("name").and_then(|n| n.as_str()) {
        Some(name) => name,
        None => return JsonRpcResponse::error(request.id.clone(), JsonRpcError::invalid_params("Missing tool name")),
    };

    let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

    let result = match tool_name {
        "extract_text" => handle_extract_text(pipeline, &arguments).await,
        "extract_structured" => handle_extract_structured(pipeline, &arguments).await,
        "get_page_count" => handle_get_page_count(pipeline, &arguments).await,
        _ => return JsonRpcResponse::error(request.id.clone(), JsonRpcError::invalid_params(&format!("Unknown tool: {}", tool_name))),
    };

    match result {
        Ok(content) => JsonRpcResponse::success(request.id.clone(), serde_json::json!({ "content": content })),
        Err(e) => JsonRpcResponse::error(request.id.clone(), JsonRpcError::internal_error(&e.to_string())),
    }
}

async fn handle_extract_text(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let file_path_str = args["file_path"].as_str().ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
    let file_path = std::path::Path::new(file_path_str);

    pdf_core::FileValidator::validate_path_safety(file_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

    let result = pipeline.extract_text(file_path).await?;
    Ok(vec![Content::text(result.extracted_text)])
}

async fn handle_extract_structured(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let file_path_str = args["file_path"].as_str().ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
    let file_path = std::path::Path::new(file_path_str);

    pdf_core::FileValidator::validate_path_safety(file_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

    let result = pipeline.extract_structured(file_path, &ExtractOptions::default()).await?;
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_get_page_count(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let file_path_str = args["file_path"].as_str().ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
    let file_path = std::path::Path::new(file_path_str);

    pdf_core::FileValidator::validate_path_safety(file_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

    let count = pipeline.get_page_count(file_path).await?;
    Ok(vec![Content::text(format!("{}", count))])
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

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
        Self { jsonrpc: "2.0".to_string(), id, result: Some(result), error: None }
    }

    fn error(id: Option<serde_json::Value>, error: JsonRpcError) -> Self {
        Self { jsonrpc: "2.0".to_string(), id, result: None, error: Some(error) }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcError {
    fn parse_error() -> Self { Self { code: -32700, message: "Parse error".to_string() } }
    fn method_not_found(method: &str) -> Self { Self { code: -32601, message: format!("Method not found: {}", method) } }
    fn invalid_params(msg: &str) -> Self { Self { code: -32602, message: msg.to_string() } }
    fn internal_error(msg: &str) -> Self { Self { code: -32603, message: msg.to_string() } }
}

#[derive(Debug, Serialize)]
struct ToolDefinition {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct Content {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

impl Content {
    fn text(text: String) -> Self { Self { content_type: "text".to_string(), text } }
}
