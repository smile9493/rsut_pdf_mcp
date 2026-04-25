
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
                let response = JsonRpcResponse::error(None, JsonRpcError::parse_error());
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

/// Handle a JSON-RPC request
pub async fn handle_request(
    service: &Arc<PdfExtractorService>,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => handle_initialize(&request),
        "tools/list" => handle_tools_list(&request),
        "tools/call" => handle_tools_call(service, &request).await,
        _ => JsonRpcResponse::error(request.id, JsonRpcError::method_not_found(&request.method)),
    }
}

/// Handle initialize request with detailed server info
fn handle_initialize(request: &JsonRpcRequest) -> JsonRpcResponse {
    let result = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name": "pdf-module-mcp",
            "version": "0.2.0",
            "description": "高性能 PDF 文本提取 MCP 服务器，支持多种提取引擎、智能路由、缓存优化",
            "author": "smile9493",
            "homepage": "https://github.com/smile9493/rsut_pdf_mcp",
            "features": [
                "multi-engine",
                "smart-routing",
                "circuit-breaker",
                "cache",
                "chinese-support"
            ]
        },
        "capabilities": {
            "tools": {
                "listChanged": true
            },
            "resources": {},
            "prompts": {}
        },
        "instructions": "欢迎使用 PDF Module MCP Server！\n\n这是一个高性能的 PDF 文本提取服务，提供以下核心功能：\n\n1. **文本提取**：从 PDF 中提取纯文本或结构化数据\n2. **关键词搜索**：在 PDF 中搜索关键词并返回上下文\n3. **关键词提取**：自动提取高频关键词（支持中英文）\n4. **智能路由**：根据文档特征自动选择最优提取引擎\n\n**快速开始**：\n- 使用 `list_adapters` 查看可用引擎\n- 使用 `extract_text` 提取文本（推荐不指定 adapter，让系统自动选择）\n- 使用 `search_keywords` 搜索关键词\n\n**性能提示**：\n- 结果会被缓存，重复查询更快\n- 大文件（>10MB）处理时间较长，建议异步处理"
    });
    JsonRpcResponse::success(request.id.clone(), result)
}

/// Handle tools/list request with detailed descriptions
fn handle_tools_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    let tools = vec![
        // Tool 1: extract_text
        ToolDefinition {
            name: "extract_text".to_string(),
            description: include_str!("../descriptions/extract_text.md").to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "PDF 文件的绝对路径。路径必须存在且可读。支持的安全检查：禁止路径遍历（../）、符号链接验证。"
                    },
                    "adapter": {
                        "type": "string",
                        "description": "指定提取引擎。不指定时使用智能路由自动选择。可选值：lopdf（布局感知）、pdf-extract（快速）、pdfium（高兼容）",
                        "enum": ["lopdf", "pdf-extract", "pdfium", "pymupdf", "pdfplumber"]
                    }
                },
                "required": ["file_path"]
            }),
        },
        // Tool 2: extract_structured
        ToolDefinition {
            name: "extract_structured".to_string(),
            description: include_str!("../descriptions/extract_structured.md").to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "PDF 文件的绝对路径"
                    },
                    "adapter": {
                        "type": "string",
                        "description": "提取引擎（推荐 lopdf 以获得最佳位置精度）"
                    },
                    "enable_highlight": {
                        "type": "boolean",
                        "description": "是否包含高亮元数据（用于前端渲染）。默认 false。"
                    }
                },
                "required": ["file_path"]
            }),
        },
        // Tool 3: search_keywords
        ToolDefinition {
            name: "search_keywords".to_string(),
            description: include_str!("../descriptions/search_keywords.md").to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "PDF 文件的绝对路径"
                    },
                    "keywords": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "要搜索的关键词列表。支持正则表达式特殊字符（会被自动转义）。"
                    },
                    "case_sensitive": {
                        "type": "boolean",
                        "description": "是否区分大小写。默认 false（不区分）。"
                    },
                    "context_length": {
                        "type": "integer",
                        "description": "匹配上下文的字符数（前后各取 N 个字符）。默认 50。建议范围：30-200。",
                        "default": 50,
                        "minimum": 10,
                        "maximum": 500
                    }
                },
                "required": ["file_path", "keywords"]
            }),
        },
        // Tool 4: extract_keywords
        ToolDefinition {
            name: "extract_keywords".to_string(),
            description: include_str!("../descriptions/extract_keywords.md").to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "PDF 文件的绝对路径"
                    },
                    "top_n": {
                        "type": "integer",
                        "description": "返回前 N 个高频关键词。默认 10。建议范围：5-50。",
                        "default": 10,
                        "minimum": 1,
                        "maximum": 100
                    }
                },
                "required": ["file_path"]
            }),
        },
        // Tool 5: get_page_count
        ToolDefinition {
            name: "get_page_count".to_string(),
            description: include_str!("../descriptions/get_page_count.md").to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "PDF 文件的绝对路径"
                    }
                },
                "required": ["file_path"]
            }),
        },
        // Tool 6: list_adapters
        ToolDefinition {
            name: "list_adapters".to_string(),
            description: include_str!("../descriptions/list_adapters.md").to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        // Tool 7: cache_stats
        ToolDefinition {
            name: "cache_stats".to_string(),
            description: include_str!("../descriptions/cache_stats.md").to_string(),
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

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

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

    let result = service.extract_text(file_path, adapter).await?;

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
        .extract_structured(file_path, adapter, &ExtractOptions::default())
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

    let result = service.extract_keywords(file_path, 2, 20, top_n).await?;

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

/// Handle list_adapters tool
fn handle_list_adapters(service: &Arc<PdfExtractorService>) -> anyhow::Result<Vec<Content>> {
    let adapters = service.list_engines();
    Ok(vec![Content::text(serde_json::to_string_pretty(
        &adapters,
    )?)])
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
