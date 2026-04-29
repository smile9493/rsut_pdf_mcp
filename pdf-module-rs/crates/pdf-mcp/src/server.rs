use pdf_core::{dto::*, wiki::{AgentPayload, WikiStorage}, McpPdfPipeline, PathValidationConfig};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::signal;
use tracing::{debug, error, info};

static SHUTDOWN_FLAG: AtomicBool = AtomicBool::new(false);

pub struct ToolStats {
    pub total_calls: AtomicU64,
    pub total_latency_ms: AtomicU64,
    pub total_errors: AtomicU64,
    pub files_processed: AtomicU64,
    pub start_time: u64,
    pub extract_text_calls: AtomicU64,
    pub extract_text_latency: AtomicU64,
    pub extract_text_errors: AtomicU64,
    pub extract_structured_calls: AtomicU64,
    pub extract_structured_latency: AtomicU64,
    pub extract_structured_errors: AtomicU64,
    pub get_page_count_calls: AtomicU64,
    pub get_page_count_latency: AtomicU64,
    pub get_page_count_errors: AtomicU64,
    pub search_keywords_calls: AtomicU64,
    pub search_keywords_latency: AtomicU64,
    pub search_keywords_errors: AtomicU64,
}

impl ToolStats {
    pub fn new() -> Self {
        Self {
            total_calls: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            files_processed: AtomicU64::new(0),
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            extract_text_calls: AtomicU64::new(0),
            extract_text_latency: AtomicU64::new(0),
            extract_text_errors: AtomicU64::new(0),
            extract_structured_calls: AtomicU64::new(0),
            extract_structured_latency: AtomicU64::new(0),
            extract_structured_errors: AtomicU64::new(0),
            get_page_count_calls: AtomicU64::new(0),
            get_page_count_latency: AtomicU64::new(0),
            get_page_count_errors: AtomicU64::new(0),
            search_keywords_calls: AtomicU64::new(0),
            search_keywords_latency: AtomicU64::new(0),
            search_keywords_errors: AtomicU64::new(0),
        }
    }

    pub fn record_success(&self, tool: &str, latency_ms: u64) {
        self.total_calls.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
        self.files_processed.fetch_add(1, Ordering::Relaxed);

        match tool {
            "extract_text" => {
                self.extract_text_calls.fetch_add(1, Ordering::Relaxed);
                self.extract_text_latency.fetch_add(latency_ms, Ordering::Relaxed);
            }
            "extract_structured" => {
                self.extract_structured_calls.fetch_add(1, Ordering::Relaxed);
                self.extract_structured_latency.fetch_add(latency_ms, Ordering::Relaxed);
            }
            "get_page_count" => {
                self.get_page_count_calls.fetch_add(1, Ordering::Relaxed);
                self.get_page_count_latency.fetch_add(latency_ms, Ordering::Relaxed);
            }
            "search_keywords" => {
                self.search_keywords_calls.fetch_add(1, Ordering::Relaxed);
                self.search_keywords_latency.fetch_add(latency_ms, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    pub fn record_error(&self, tool: &str) {
        self.total_calls.fetch_add(1, Ordering::Relaxed);
        self.total_errors.fetch_add(1, Ordering::Relaxed);

        match tool {
            "extract_text" => {
                self.extract_text_calls.fetch_add(1, Ordering::Relaxed);
                self.extract_text_errors.fetch_add(1, Ordering::Relaxed);
            }
            "extract_structured" => {
                self.extract_structured_calls.fetch_add(1, Ordering::Relaxed);
                self.extract_structured_errors.fetch_add(1, Ordering::Relaxed);
            }
            "get_page_count" => {
                self.get_page_count_calls.fetch_add(1, Ordering::Relaxed);
                self.get_page_count_errors.fetch_add(1, Ordering::Relaxed);
            }
            "search_keywords" => {
                self.search_keywords_calls.fetch_add(1, Ordering::Relaxed);
                self.search_keywords_errors.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    pub fn total_calls(&self) -> u64 {
        self.total_calls.load(Ordering::Relaxed)
    }

    pub fn avg_latency(&self) -> u64 {
        let total = self.total_calls.load(Ordering::Relaxed);
        if total == 0 {
            return 0;
        }
        self.total_latency_ms.load(Ordering::Relaxed) / total
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total_calls.load(Ordering::Relaxed);
        if total == 0 {
            return 100.0;
        }
        let errors = self.total_errors.load(Ordering::Relaxed);
        ((total - errors) as f64 / total as f64) * 100.0
    }
}

fn default_path_config() -> PathValidationConfig {
    PathValidationConfig {
        require_absolute: true,
        allow_traversal: false,
        base_dir: None,
    }
}

pub async fn run_stdio(pipeline: Arc<McpPdfPipeline>) -> anyhow::Result<()> {
    info!("MCP server listening on stdio");

    let shutdown_notifier = Arc::new(AtomicBool::new(false));
    let notifier_clone = Arc::clone(&shutdown_notifier);

    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                notifier_clone.store(true, Ordering::SeqCst);
                info!("Received shutdown signal, finishing current request...");
            }
            Err(err) => {
                error!("Unable to listen for shutdown signal: {}", err);
            }
        }
    });

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();

    for line in stdin.lock().lines() {
        if SHUTDOWN_FLAG.load(Ordering::SeqCst) || shutdown_notifier.load(Ordering::SeqCst) {
            info!("Shutting down gracefully...");
            break;
        }

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

    SHUTDOWN_FLAG.store(true, Ordering::SeqCst);
    info!("Server shut down gracefully");
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
        ToolDefinition {
            name: "search_keywords".to_string(),
            description:
                "Search for keywords in a PDF file and return matches with page numbers and context"
                    .to_string(),
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
                        "description": "Keywords to search for"
                    },
                    "case_sensitive": {
                        "type": "boolean",
                        "description": "Case sensitive search (default: false)"
                    },
                    "context_length": {
                        "type": "number",
                        "description": "Characters of context around match (default: 50)"
                    }
                },
                "required": ["file_path", "keywords"]
            }),
        },
        ToolDefinition {
            name: "extrude_to_server_wiki".to_string(),
            description: "Extract PDF and save to server-side wiki with automatic indexing".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute path to the PDF file"
                    },
                    "wiki_base_path": {
                        "type": "string",
                        "description": "Base directory for wiki storage (default: ./wiki)"
                    }
                },
                "required": ["file_path"]
            }),
        },
        ToolDefinition {
            name: "extrude_to_agent_payload".to_string(),
            description: "Extract PDF and return markdown payload with prompt for local wiki creation".to_string(),
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
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                JsonRpcError::invalid_params("Missing params"),
            )
        }
    };

    let tool_name = match params.get("name").and_then(|n| n.as_str()) {
        Some(name) => name,
        None => {
            return JsonRpcResponse::error(
                request.id.clone(),
                JsonRpcError::invalid_params("Missing tool name"),
            )
        }
    };

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    let result = match tool_name {
        "extract_text" => handle_extract_text(pipeline, &arguments).await,
        "extract_structured" => handle_extract_structured(pipeline, &arguments).await,
        "get_page_count" => handle_get_page_count(pipeline, &arguments).await,
        "search_keywords" => handle_search_keywords(pipeline, &arguments).await,
        "extrude_to_server_wiki" => handle_extrude_to_server_wiki(pipeline, &arguments).await,
        "extrude_to_agent_payload" => handle_extrude_to_agent_payload(pipeline, &arguments).await,
        _ => {
            return JsonRpcResponse::error(
                request.id.clone(),
                JsonRpcError::invalid_params(&format!("Unknown tool: {}", tool_name)),
            )
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

async fn handle_extract_text(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let file_path_str = args["file_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
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
    let file_path_str = args["file_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
    let file_path = std::path::Path::new(file_path_str);

    pdf_core::FileValidator::validate_path_safety(file_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

    let result = pipeline
        .extract_structured(file_path, &ExtractOptions::default())
        .await?;
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_get_page_count(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let file_path_str = args["file_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
    let file_path = std::path::Path::new(file_path_str);

    pdf_core::FileValidator::validate_path_safety(file_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

    let count = pipeline.get_page_count(file_path).await?;
    Ok(vec![Content::text(format!("{}", count))])
}

async fn handle_search_keywords(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let file_path_str = args["file_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
    let file_path = std::path::Path::new(file_path_str);

    pdf_core::FileValidator::validate_path_safety(file_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

    let keywords: Vec<String> = args["keywords"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Missing keywords array"))?
        .iter()
        .filter_map(|k| k.as_str().map(|s| s.to_string()))
        .collect();

    if keywords.is_empty() {
        return Err(anyhow::anyhow!("Keywords array is empty"));
    }

    let case_sensitive = args["case_sensitive"].as_bool().unwrap_or(false);
    let context_length = args["context_length"].as_u64().unwrap_or(50) as usize;

    let result = pipeline
        .extract_structured(file_path, &ExtractOptions::default())
        .await?;
    let text = &result.extracted_text;

    // OPTIMIZATION: Precompute page boundaries for O(log n) page lookup
    let mut page_boundaries: Vec<(usize, u32)> = Vec::with_capacity(result.pages.len());
    let mut offset = 0usize;
    for page in &result.pages {
        page_boundaries.push((offset, page.page_number));
        offset += page.text.len();
    }

    // Binary search for page number
    let find_page = |pos: usize| -> u32 {
        match page_boundaries.binary_search_by(|(start, _)| start.cmp(&pos)) {
            Ok(idx) => page_boundaries[idx].1,
            Err(idx) => {
                if idx == 0 {
                    1
                } else if idx >= page_boundaries.len() {
                    page_boundaries.last().map(|(_, p)| *p).unwrap_or(1)
                } else {
                    page_boundaries[idx - 1].1
                }
            }
        }
    };

    // OPTIMIZATION: Precompile all regex patterns
    let patterns: Vec<regex::Regex> = keywords
        .iter()
        .map(|kw| {
            let pattern = regex::escape(kw);
            let flags = if case_sensitive { "" } else { "(?i)" };
            regex::Regex::new(&format!("{}{}", flags, pattern)).unwrap()
        })
        .collect();

    // OPTIMIZATION: Estimate capacity
    let mut matches: Vec<serde_json::Value> = Vec::with_capacity(256);
    let mut pages_with_matches: std::collections::HashSet<u32> = std::collections::HashSet::new();

    for (keyword, re) in keywords.iter().zip(patterns.iter()) {
        for m in re.find_iter(text) {
            let start = m.start();
            let end = m.end();

            let page_number = find_page(start);
            pages_with_matches.insert(page_number);

            // UTF-8 safe slicing
            let ctx_start = text.floor_char_boundary(start.saturating_sub(context_length));
            let ctx_end = text.ceil_char_boundary((end + context_length).min(text.len()));

            matches.push(serde_json::json!({
                "keyword": keyword,
                "page": page_number,
                "position": start,
                "context": &text[ctx_start..ctx_end]
            }));
        }
    }

    let search_result = serde_json::json!({
        "total_matches": matches.len(),
        "pages_with_matches": pages_with_matches.len(),
        "matches": matches
    });

    Ok(vec![Content::text(serde_json::to_string(&search_result)?)])
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
    fn text(text: String) -> Self {
        Self {
            content_type: "text".to_string(),
            text,
        }
    }
}

async fn handle_extrude_to_server_wiki(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let file_path_str = args["file_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
    let file_path = std::path::Path::new(file_path_str);

    pdf_core::FileValidator::validate_path_safety(file_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

    let wiki_base_path = args["wiki_base_path"]
        .as_str()
        .map(|s| std::path::Path::new(s))
        .unwrap_or_else(|| std::path::Path::new("./wiki"));

    let storage = WikiStorage::new(wiki_base_path)
        .map_err(|e| anyhow::anyhow!("Failed to create wiki storage: {}", e))?;

    let result = pipeline
        .extract_structured(file_path, &ExtractOptions::default())
        .await
        .map_err(|e| anyhow::anyhow!("Extraction failed: {}", e))?;

    let raw_path = storage
        .save_raw(&result, file_path, 0.85, "pdfium")
        .map_err(|e| anyhow::anyhow!("Failed to save raw document: {}", e))?;

    let map_path = storage
        .generate_map()
        .map_err(|e| anyhow::anyhow!("Failed to generate MAP.md: {}", e))?;

    let response = serde_json::json!({
        "status": "success",
        "raw_path": raw_path.to_string_lossy().to_string(),
        "map_path": map_path.to_string_lossy().to_string(),
        "page_count": result.page_count,
        "message": format!("PDF extracted and saved to wiki at {:?}", wiki_base_path)
    });

    Ok(vec![Content::text(serde_json::to_string_pretty(&response)?)])
}

async fn handle_extrude_to_agent_payload(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let file_path_str = args["file_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing file_path"))?;
    let file_path = std::path::Path::new(file_path_str);

    pdf_core::FileValidator::validate_path_safety(file_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

    let result = pipeline
        .extract_structured(file_path, &ExtractOptions::default())
        .await
        .map_err(|e| anyhow::anyhow!("Extraction failed: {}", e))?;

    let payload = AgentPayload::from_extraction(&result, file_path, 0.85, "pdfium");
    let markdown = payload.to_markdown();

    Ok(vec![Content::text(markdown)])
}
