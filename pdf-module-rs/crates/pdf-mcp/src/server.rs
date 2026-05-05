use crate::protocol::{Content, JsonRpcError, JsonRpcRequest, JsonRpcResponse, ToolDefinition};
use crate::sampling::{
    create_sampling_jsonrpc_request, parse_sampling_response, OutgoingRequest, SamplingClient,
    SamplingClientConfig,
};
use pdf_core::{
    dto::*,
    management::{ConfigManager, HealthReporter},
    wiki::{AgentPayload, WikiStorage},
    FulltextIndex, GraphIndex, KnowledgeEngine, McpPdfPipeline, PathValidationConfig,
};
use rust_embed::RustEmbed;
use std::borrow::Cow;
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

#[derive(RustEmbed)]
#[folder = "src/ui/"]
struct UiAssets;

#[allow(dead_code)]
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

#[allow(dead_code)]
impl ToolStats {
    pub fn new() -> Self {
        Self {
            total_calls: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            files_processed: AtomicU64::new(0),
            start_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time is before UNIX epoch")
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
        self.total_latency_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
        self.files_processed.fetch_add(1, Ordering::Relaxed);

        match tool {
            "extract_text" => {
                self.extract_text_calls.fetch_add(1, Ordering::Relaxed);
                self.extract_text_latency
                    .fetch_add(latency_ms, Ordering::Relaxed);
            }
            "extract_structured" => {
                self.extract_structured_calls
                    .fetch_add(1, Ordering::Relaxed);
                self.extract_structured_latency
                    .fetch_add(latency_ms, Ordering::Relaxed);
            }
            "get_page_count" => {
                self.get_page_count_calls.fetch_add(1, Ordering::Relaxed);
                self.get_page_count_latency
                    .fetch_add(latency_ms, Ordering::Relaxed);
            }
            "search_keywords" => {
                self.search_keywords_calls.fetch_add(1, Ordering::Relaxed);
                self.search_keywords_latency
                    .fetch_add(latency_ms, Ordering::Relaxed);
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
                self.extract_structured_calls
                    .fetch_add(1, Ordering::Relaxed);
                self.extract_structured_errors
                    .fetch_add(1, Ordering::Relaxed);
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

#[tracing::instrument(skip(pipeline))]
pub async fn run_stdio(pipeline: Arc<McpPdfPipeline>) -> anyhow::Result<()> {
    info!("MCP server listening on stdio");

    let shutdown_token = CancellationToken::new();
    let shutdown_token_clone = shutdown_token.clone();

    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                shutdown_token_clone.cancel();
                info!("Received shutdown signal, finishing current request...");
            }
            Err(err) => {
                error!("Unable to listen for shutdown signal: {}", err);
            }
        }
    });

    let sampling_config = SamplingClientConfig::default();
    let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<OutgoingRequest>(100);
    let sampling_client = Arc::new(SamplingClient::with_sender(
        sampling_config.timeout_secs,
        outgoing_tx.clone(),
    ));
    let pending_requests = sampling_client.pending_requests();

    let stdout = std::io::stdout();
    let mut stdout_lock = stdout.lock();

    let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(100);

    tokio::task::spawn_blocking(move || {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(l) => {
                    if stdin_tx.blocking_send(l).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let _sampling_client_ref = Arc::clone(&sampling_client);

    loop {
        tokio::select! {
            Some(line) = stdin_rx.recv() => {
                if shutdown_token.is_cancelled() {
                    info!("Shutting down gracefully...");
                    break;
                }

                info!(
                    "Received: {}",
                    if line.len() > 100 { &line[..100] } else { &line }
                );

                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                    if value.get("method").is_none() && (value.get("result").is_some() || value.get("error").is_some()) {
                        match parse_sampling_response(&value) {
                            Ok((id, result)) => {
                                info!("Received sampling response for id={}", id);
                                let pending = pending_requests.clone();
                                tokio::spawn(async move {
                                    let response_tx = {
                                        let mut pending = pending.write().await;
                                        pending.remove(&id)
                                    };
                                    if let Some(tx) = response_tx {
                                        let _ = tx.send(result);
                                    }
                                });
                                continue;
                            }
                            Err(e) => {
                                error!("Failed to parse sampling response: {}", e);
                                continue;
                            }
                        }
                    }
                }

                let request: JsonRpcRequest = match serde_json::from_str::<JsonRpcRequest>(&line) {
                    Ok(req) => {
                        info!("Parsed request: method={}", req.method);
                        req
                    }
                    Err(e) => {
                        error!("Failed to parse request: {}", e);
                        let response = JsonRpcResponse::error(None, JsonRpcError::parse_error());
                        write_response(&mut stdout_lock, &response)?;
                        continue;
                    }
                };

                let response = handle_request(&pipeline, request).await;
                if let Some(resp) = response {
                    info!("Sending response for id={:?}", resp.id);
                    write_response(&mut stdout_lock, &resp)?;
                }
            }

            Some(outgoing) = outgoing_rx.recv() => {
                let json_request = create_sampling_jsonrpc_request(outgoing.id, outgoing.request);
                let json_str = serde_json::to_string(&json_request)?;
                info!("Sending sampling request: id={}", outgoing.id);
                writeln!(stdout_lock, "{}", json_str)?;
                stdout_lock.flush()?;
            }

            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                if shutdown_token.is_cancelled() {
                    break;
                }
            }
        }
    }

    drop(stdin_rx);
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

#[tracing::instrument(skip(pipeline, request), fields(method = %request.method))]
pub async fn handle_request(
    pipeline: &Arc<McpPdfPipeline>,
    request: JsonRpcRequest,
) -> Option<JsonRpcResponse> {
    if request.method.starts_with("notifications/") {
        return None;
    }

    let response = match request.method.as_str() {
        "initialize" => handle_initialize(&request),
        "tools/list" => handle_tools_list(&request),
        "tools/call" => handle_tools_call(pipeline, &request).await,
        "resources/list" => handle_resources_list(&request),
        "resources/read" => handle_resources_read(&request),
        _ => JsonRpcResponse::error(request.id, JsonRpcError::method_not_found(&request.method)),
    };
    Some(response)
}

fn handle_initialize(request: &JsonRpcRequest) -> JsonRpcResponse {
    let result = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name": "rsut-pdf-mcp",
            "version": "0.6.0",
            "description": "AI-native knowledge compilation engine — PDF extraction, Karpathy compiler pattern, Tantivy fulltext search (CJK-aware), petgraph knowledge graph, hierarchical compilation, dynamic reasoning. Pure Rust, single binary."
        },
        "capabilities": {
            "tools": { "listChanged": false },
            "resources": { "listChanged": false },
            "sampling": {
                "supported": true,
                "messageTypes": ["text", "image"]
            }
        },
        "instructions": "Knowledge engine with 25 tools. PDF extraction: extract_text, extract_structured, get_page_count, search_keywords, extrude_to_server_wiki, extrude_to_agent_payload. Compilation: compile_to_wiki, incremental_compile, recompile_entry, aggregate_entries, check_quality. Indexing: search_knowledge, rebuild_index, get_entry_context, find_orphans, suggest_links, export_concept_map. Reasoning: micro_compile, hypothesis_test. Management: get_config, set_config, get_health_report, trigger_incremental_compile, get_compile_status."
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
            description: "Extract PDF to server-side wiki (Karpathy paradigm). Rust engine only saves to raw/, AI Agent should read and create atomic wiki entries.".to_string(),
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
            description: "Extract PDF and return markdown payload with knowledge compilation instructions for AI Agent to create local wiki entries".to_string(),
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
        // === Knowledge Engine Tools ===
        ToolDefinition {
            name: "compile_to_wiki".to_string(),
            description: "Compile a PDF into the knowledge base: extract text, save to raw/, generate compilation prompt for AI. This is the primary entry point for the Karpathy compiler pattern.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pdf_path": {
                        "type": "string",
                        "description": "Absolute path to the PDF file"
                    },
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    },
                    "domain": {
                        "type": "string",
                        "description": "Domain classification (e.g. 'IT', 'Math'). Default: '未分类'"
                    }
                },
                "required": ["pdf_path", "knowledge_base"]
            }),
        },
        ToolDefinition {
            name: "incremental_compile".to_string(),
            description: "Scan raw/ directory for new or changed PDFs and compile only those that need it. Uses SHA-256 hash comparison for change detection.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    }
                },
                "required": ["knowledge_base"]
            }),
        },
        ToolDefinition {
            name: "search_knowledge".to_string(),
            description: "Full-text search across all wiki entries using Tantivy. Returns ranked results with snippets.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query (supports keywords, phrases, boolean)"
                    },
                    "limit": {
                        "type": "number",
                        "description": "Maximum number of results (default: 10)"
                    }
                },
                "required": ["knowledge_base", "query"]
            }),
        },
        ToolDefinition {
            name: "rebuild_index".to_string(),
            description: "Rebuild all indexes (Tantivy fulltext + petgraph link graph) from wiki Markdown files. Use after bulk changes or for recovery.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    }
                },
                "required": ["knowledge_base"]
            }),
        },
        ToolDefinition {
            name: "get_entry_context".to_string(),
            description: "Get N-hop neighbors of a knowledge entry (by link relationships, tag co-occurrence). Returns connected entries for context expansion.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    },
                    "entry_path": {
                        "type": "string",
                        "description": "Relative path of the entry within wiki/ (e.g. 'it/http2_multiplex.md')"
                    },
                    "hops": {
                        "type": "number",
                        "description": "Maximum number of hops to traverse (default: 2)"
                    }
                },
                "required": ["knowledge_base", "entry_path"]
            }),
        },
        ToolDefinition {
            name: "find_orphans".to_string(),
            description: "Find knowledge entries with no incoming or outgoing related/contradiction links. These are candidates for integration.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    }
                },
                "required": ["knowledge_base"]
            }),
        },
        ToolDefinition {
            name: "suggest_links".to_string(),
            description: "Suggest potential links for a knowledge entry based on tag similarity (Jaccard index). Helps discover hidden connections.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    },
                    "entry_path": {
                        "type": "string",
                        "description": "Relative path of the entry within wiki/"
                    },
                    "top_k": {
                        "type": "number",
                        "description": "Maximum number of suggestions (default: 10)"
                    }
                },
                "required": ["knowledge_base", "entry_path"]
            }),
        },
        ToolDefinition {
            name: "export_concept_map".to_string(),
            description: "Export a local concept map around an entry as Mermaid.js text. Shows relationships within N hops for visualization.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    },
                    "entry_path": {
                        "type": "string",
                        "description": "Relative path of the center entry within wiki/"
                    },
                    "depth": {
                        "type": "number",
                        "description": "Number of hops to include (default: 2)"
                    }
                },
                "required": ["knowledge_base", "entry_path"]
            }),
        },
        ToolDefinition {
            name: "check_quality".to_string(),
            description: "Analyze wiki quality: detect missing tags, orphan entries, broken links, style issues. Returns a comprehensive report.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    }
                },
                "required": ["knowledge_base"]
            }),
        },
        ToolDefinition {
            name: "micro_compile".to_string(),
            description: "On-demand extraction from a PDF for the current conversation context. Results are NOT saved to wiki — they are injected directly into the AI session for immediate use.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pdf_path": {
                        "type": "string",
                        "description": "Absolute path to the PDF file"
                    },
                    "page_range": {
                        "type": "string",
                        "description": "Page range to extract (e.g. '1-5', '3,7,12'). Default: all pages"
                    }
                },
                "required": ["pdf_path"]
            }),
        },
        ToolDefinition {
            name: "aggregate_entries".to_string(),
            description: "Identify clusters of related L1 wiki entries that can be aggregated into L2 summary entries. Returns clusters with shared tags for AI to synthesize. (Phase 3: Hierarchical compilation)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    }
                },
                "required": ["knowledge_base"]
            }),
        },
        ToolDefinition {
            name: "hypothesis_test".to_string(),
            description: "Find pairs of entries that explicitly contradict each other, and generate a debate framework for AI to resolve the contradictions. Returns contradiction pairs with entry context for AI-driven analysis. (Phase 4: Dynamic reasoning)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    }
                },
                "required": ["knowledge_base"]
            }),
        },
        ToolDefinition {
            name: "recompile_entry".to_string(),
            description: "Recompile a single wiki entry: bumps version, creates backup, checks if source PDF changed, and generates a recompile prompt for AI. Use for quality drift correction.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    },
                    "entry_path": {
                        "type": "string",
                        "description": "Relative path of the entry within wiki/ (e.g. 'it/concept.md')"
                    }
                },
                "required": ["knowledge_base", "entry_path"]
            }),
        },
        // === Management Tools (Phase 1) ===
        ToolDefinition {
            name: "get_config".to_string(),
            description: "Get current runtime configuration for a knowledge base. Returns all key-value pairs from the managed config file.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    }
                },
                "required": ["knowledge_base"]
            }),
        },
        ToolDefinition {
            name: "set_config".to_string(),
            description: "Set a runtime configuration value for a knowledge base. Persists atomically via write-tmp + rename.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    },
                    "key": {
                        "type": "string",
                        "description": "Configuration key (e.g. 'vlm_api_key', 'extract_mode')"
                    },
                    "value": {
                        "type": "string",
                        "description": "Configuration value"
                    }
                },
                "required": ["knowledge_base", "key", "value"]
            }),
        },
        ToolDefinition {
            name: "get_health_report".to_string(),
            description: "Get a comprehensive health report for the knowledge base: entry count, orphan count, contradiction count, index size, graph topology, quality score, and last compile time.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    }
                },
                "required": ["knowledge_base"]
            }),
        },
        ToolDefinition {
            name: "trigger_incremental_compile".to_string(),
            description: "Manually trigger an incremental compilation of the knowledge base. Scans raw/ for changed PDFs and recompiles only those that need it.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    }
                },
                "required": ["knowledge_base"]
            }),
        },
        ToolDefinition {
            name: "get_compile_status".to_string(),
            description: "Get the current compile status: whether a compile is running, last start/finish times, duration, outcome, and recent compile history.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "knowledge_base": {
                        "type": "string",
                        "description": "Absolute path to the knowledge base directory"
                    }
                },
                "required": ["knowledge_base"]
            }),
        },
    ];

    JsonRpcResponse::success(request.id.clone(), serde_json::json!({ "tools": tools }))
}

#[tracing::instrument(skip(pipeline, request), fields(tool = ?request.params.get("name")))]
async fn handle_tools_call(
    pipeline: &Arc<McpPdfPipeline>,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    let params = &request.params;

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
        // Knowledge Engine tools
        "compile_to_wiki" => handle_compile_to_wiki(pipeline, &arguments).await,
        "incremental_compile" => handle_incremental_compile(pipeline, &arguments).await,
        "search_knowledge" => handle_search_knowledge(&arguments).await,
        "rebuild_index" => handle_rebuild_index(&arguments).await,
        "get_entry_context" => handle_get_entry_context(&arguments).await,
        "find_orphans" => handle_find_orphans(&arguments).await,
        "suggest_links" => handle_suggest_links(&arguments).await,
        "export_concept_map" => handle_export_concept_map(&arguments).await,
        "check_quality" => handle_check_quality(&arguments).await,
        "micro_compile" => handle_micro_compile(pipeline, &arguments).await,
        "aggregate_entries" => handle_aggregate_entries(pipeline, &arguments).await,
        "hypothesis_test" => handle_hypothesis_test(pipeline, &arguments).await,
        "recompile_entry" => handle_recompile_entry(pipeline, &arguments).await,
        // Management tools
        "get_config" => handle_get_config(&arguments).await,
        "set_config" => handle_set_config(&arguments).await,
        "get_health_report" => handle_get_health_report(&arguments).await,
        "trigger_incremental_compile" => handle_trigger_incremental_compile(pipeline, &arguments).await,
        "get_compile_status" => handle_get_compile_status(&arguments).await,
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

#[tracing::instrument(skip(pipeline, args))]
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
            regex::Regex::new(&format!("{}{}", flags, pattern))
                .expect("Regex pattern should be valid after escaping")
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
        .map(std::path::Path::new)
        .unwrap_or_else(|| std::path::Path::new("./wiki"));

    let storage = WikiStorage::new(wiki_base_path)
        .map_err(|e| anyhow::anyhow!("Failed to create wiki storage: {}", e))?;

    let result = pipeline
        .extract_structured(file_path, &ExtractOptions::default())
        .await
        .map_err(|e| anyhow::anyhow!("Extraction failed: {}", e))?;

    let wiki_result = storage
        .save_raw(&result, file_path, 0.85)
        .map_err(|e| anyhow::anyhow!("Failed to save: {}", e))?;

    let response = serde_json::json!({
        "status": "success",
        "raw_path": wiki_result.raw_path.to_string_lossy().to_string(),
        "index_path": wiki_result.index_path.to_string_lossy().to_string(),
        "log_path": wiki_result.log_path.to_string_lossy().to_string(),
        "page_count": wiki_result.page_count,
        "message": "PDF extracted to raw/. AI Agent should process and create wiki entries.",
        "next_step": "Use extrude_to_agent_payload to get the prompt for AI Agent, or manually process raw/ content."
    });

    Ok(vec![Content::text(serde_json::to_string_pretty(
        &response,
    )?)])
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

    let payload = AgentPayload::from_extraction(&result, file_path, 0.85);
    let markdown = payload.to_markdown();

    Ok(vec![Content::text(markdown)])
}

// === Knowledge Engine Tool Handlers ===

fn parse_kb_path(args: &serde_json::Value) -> anyhow::Result<std::path::PathBuf> {
    let kb = args["knowledge_base"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing knowledge_base"))?;
    Ok(std::path::PathBuf::from(kb))
}

#[tracing::instrument(skip(pipeline, args))]
async fn handle_compile_to_wiki(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let pdf_path_str = args["pdf_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing pdf_path"))?;
    let pdf_path = std::path::Path::new(pdf_path_str);
    let kb_path = parse_kb_path(args)?;
    let domain = args["domain"].as_str();

    pdf_core::FileValidator::validate_path_safety(pdf_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

    let engine = KnowledgeEngine::new(Arc::clone(pipeline), &kb_path)?;
    let result = engine.compile_to_wiki(pdf_path, domain).await?;

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_incremental_compile(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;
    let engine = KnowledgeEngine::new(Arc::clone(pipeline), &kb_path)?;
    let raw_dir = engine.raw_dir();
    let result = engine.incremental_compile(&raw_dir).await?;

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_search_knowledge(args: &serde_json::Value) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;
    let query = args["query"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing query"))?;
    let limit = args["limit"].as_u64().unwrap_or(10) as usize;

    let idx = FulltextIndex::open_or_create(&kb_path)?;

    // Auto-rebuild if index is empty
    let wiki_dir = kb_path.join("wiki");
    if wiki_dir.exists() {
        let sample = idx.search("*", 1);
        let needs_rebuild = match sample {
            Ok(results) => results.is_empty(),
            Err(_) => true,
        };
        if needs_rebuild {
            idx.rebuild(&wiki_dir)?;
        }
    }

    let hits = idx.search(query, limit)?;
    Ok(vec![Content::text(serde_json::to_string_pretty(&hits)?)])
}

async fn handle_rebuild_index(args: &serde_json::Value) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;
    let wiki_dir = kb_path.join("wiki");

    // Rebuild Tantivy
    let ft_idx = FulltextIndex::open_or_create(&kb_path)?;
    let ft_count = ft_idx.rebuild(&wiki_dir)?;

    // Rebuild petgraph
    let mut g_idx = GraphIndex::new();
    let g_count = g_idx.rebuild(&wiki_dir)?;

    let result = serde_json::json!({
        "status": "success",
        "fulltext_entries_indexed": ft_count,
        "graph_nodes": g_count,
        "graph_edges": g_idx.edge_count(),
        "message": "All indexes rebuilt from wiki/ files."
    });
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_get_entry_context(args: &serde_json::Value) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;
    let entry_path = args["entry_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing entry_path"))?;
    let hops = args["hops"].as_u64().unwrap_or(2) as u32;

    let mut graph = GraphIndex::new();
    let wiki_dir = kb_path.join("wiki");
    graph.rebuild(&wiki_dir)?;

    let neighbors = graph.get_neighbors(entry_path, hops);

    let result = serde_json::json!({
        "entry": entry_path,
        "hops": hops,
        "neighbors": neighbors,
        "total": neighbors.len()
    });
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_find_orphans(args: &serde_json::Value) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;

    let mut graph = GraphIndex::new();
    let wiki_dir = kb_path.join("wiki");
    graph.rebuild(&wiki_dir)?;

    let orphans = graph.find_orphans();

    let result = serde_json::json!({
        "orphan_count": orphans.len(),
        "entries": orphans,
        "message": if orphans.is_empty() {
            "No orphan entries found. All entries have at least one link.".to_string()
        } else {
            format!("{} entries have no links. Consider integrating them.", orphans.len())
        }
    });
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_suggest_links(args: &serde_json::Value) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;
    let entry_path = args["entry_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing entry_path"))?;
    let top_k = args["top_k"].as_u64().unwrap_or(10) as usize;

    let mut graph = GraphIndex::new();
    let wiki_dir = kb_path.join("wiki");
    graph.rebuild(&wiki_dir)?;

    let suggestions = graph.suggest_links(entry_path, top_k);

    let result = serde_json::json!({
        "entry": entry_path,
        "suggestions": suggestions,
        "total": suggestions.len()
    });
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_export_concept_map(args: &serde_json::Value) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;
    let entry_path = args["entry_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing entry_path"))?;
    let depth = args["depth"].as_u64().unwrap_or(2) as u32;

    let mut graph = GraphIndex::new();
    let wiki_dir = kb_path.join("wiki");
    graph.rebuild(&wiki_dir)?;

    let mermaid = graph.export_concept_map(entry_path, depth);

    let result = serde_json::json!({
        "entry": entry_path,
        "depth": depth,
        "mermaid": mermaid,
        "usage": "Paste the mermaid field into any Mermaid.js renderer (e.g. Obsidian, GitHub, mermaid.live)"
    });
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_check_quality(args: &serde_json::Value) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;
    let wiki_dir = kb_path.join("wiki");

    let report = pdf_core::knowledge::quality::analyze_wiki(&wiki_dir)?;

    let result = serde_json::json!({
        "total_entries": report.total_entries,
        "avg_quality_score": format!("{:.1}%", report.avg_quality_score * 100.0),
        "domains": report.domains.iter().collect::<Vec<_>>(),
        "issues_count": report.issues.len(),
        "orphan_count": report.orphan_entries.len(),
        "broken_links_count": report.broken_links.len(),
        "report_markdown": report.to_markdown(),
        "has_errors": report.has_errors(),
        "has_warnings": report.has_warnings()
    });
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_micro_compile(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let pdf_path_str = args["pdf_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing pdf_path"))?;
    let pdf_path = std::path::Path::new(pdf_path_str);

    pdf_core::FileValidator::validate_path_safety(pdf_path, &default_path_config())
        .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

    let page_range = args["page_range"].as_str();

    let result = pipeline
        .extract_structured(pdf_path, &ExtractOptions::default())
        .await
        .map_err(|e| anyhow::anyhow!("Extraction failed: {}", e))?;

    let text = if let Some(range) = page_range {
        // Parse page range like "1-5" or "3,7,12"
        let pages_to_include = parse_page_range(range, result.page_count);
        let filtered: Vec<String> = result
            .pages
            .iter()
            .filter(|p| pages_to_include.contains(&p.page_number))
            .map(|p| format!("## Page {}\n\n{}", p.page_number, p.text))
            .collect();
        filtered.join("\n\n")
    } else {
        result.extracted_text.clone()
    };

    let source_name = pdf_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let output = format!(
        r#"# 微编译结果: {}

> 注意: 此内容仅用于当前对话上下文，不会保存到 wiki。
> 如需持久化，请使用 `compile_to_wiki` 工具。

- 页数: {}{}

---

{}
"#,
        source_name,
        result.page_count,
        if let Some(r) = page_range {
            format!("\n- 提取范围: {}", r)
        } else {
            String::new()
        },
        text
    );

    Ok(vec![Content::text(output)])
}

fn parse_page_range(range: &str, max_page: u32) -> Vec<u32> {
    let mut pages = Vec::new();
    for part in range.split(',') {
        let part = part.trim();
        if let Some(dash_pos) = part.find('-') {
            if let (Ok(start), Ok(end)) = (
                part[..dash_pos].trim().parse::<u32>(),
                part[dash_pos + 1..].trim().parse::<u32>(),
            ) {
                for p in start..=end.min(max_page) {
                    pages.push(p);
                }
            }
        } else if let Ok(p) = part.parse::<u32>() {
            if p <= max_page {
                pages.push(p);
            }
        }
    }
    pages.sort();
    pages.dedup();
    pages
}

async fn handle_aggregate_entries(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;

    let engine = pdf_core::KnowledgeEngine::new(Arc::clone(pipeline), &kb_path)?;

    let candidates = engine.identify_aggregation_candidates()?;

    let result = serde_json::json!({
        "candidates": candidates,
        "total_clusters": candidates.len(),
        "instructions": if candidates.is_empty() {
            "No aggregation candidates found. Entries may not have enough shared tags to form clusters.".to_string()
        } else {
            "For each cluster, create an L2 summary entry that synthesizes the key ideas. Use 'aggregated_from' field in front matter to record source entries.".to_string()
        }
    });
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_hypothesis_test(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;

    let engine = pdf_core::KnowledgeEngine::new(Arc::clone(pipeline), &kb_path)?;

    let contradictions = engine.find_contradictions()?;

    // Read entry content for each contradiction pair
    let wiki_dir = kb_path.join("wiki");
    let mut enriched = Vec::new();
    for mut pair in contradictions {
        // Try to read entry B's title
        let path_b = wiki_dir.join(&pair.entry_b);
        if let Ok(content) = std::fs::read_to_string(&path_b) {
            if let Some(entry) = pdf_core::knowledge::KnowledgeEntry::from_markdown(&content) {
                pair.title_b = entry.title;
            }
        }
        enriched.push(pair);
    }

    let result = serde_json::json!({
        "contradiction_pairs": enriched,
        "total": enriched.len(),
        "instructions": if enriched.is_empty() {
            "No explicit contradictions found. Use 'suggest_links' to discover implicit tensions between entries.".to_string()
        } else {
            "For each pair, read both entries and conduct a structured debate: 1) State the core claim of each entry, 2) Identify the precise point of disagreement, 3) Evaluate supporting evidence, 4) Propose a resolution or mark as 'open question'. Write the resolution into both entries' 'contradictions' field with a note.".to_string()
        }
    });
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_recompile_entry(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;
    let entry_path = args["entry_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing entry_path"))?;

    let engine = pdf_core::KnowledgeEngine::new(Arc::clone(pipeline), &kb_path)?;

    let result = engine.recompile_entry(std::path::Path::new(entry_path))?;

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

// === Management Tool Handlers ===

async fn handle_get_config(args: &serde_json::Value) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;
    let mut cm = ConfigManager::new(&kb_path);
    cm.load().map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;

    let data: std::collections::HashMap<String, String> = cm.all().clone();
    let result = serde_json::json!({
        "config": data,
        "total_keys": data.len(),
        "config_path": kb_path.join(".rsut_index").join("config.json").to_string_lossy(),
    });
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_set_config(args: &serde_json::Value) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;
    let key = args["key"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing key"))?;
    let value = args["value"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing value"))?;

    let mut cm = ConfigManager::new(&kb_path);
    cm.load().map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
    cm.set(key, value).map_err(|e| anyhow::anyhow!("Failed to set config: {}", e))?;

    let result = serde_json::json!({
        "status": "success",
        "key": key,
        "value": value,
        "message": format!("Configuration '{}' updated successfully.", key),
    });
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_get_health_report(args: &serde_json::Value) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;
    let reporter = HealthReporter::new(&kb_path);
    let report = reporter.report().map_err(|e| anyhow::anyhow!("Failed to generate report: {}", e))?;

    let result = serde_json::json!({
        "total_entries": report.total_entries,
        "orphan_count": report.orphan_count,
        "contradiction_count": report.contradiction_count,
        "broken_link_count": report.broken_link_count,
        "index_size_mb": report.index_size_bytes / 1024 / 1024,
        "graph_nodes": report.graph_node_count,
        "graph_edges": report.graph_edge_count,
        "avg_quality_score": format!("{:.1}%", report.avg_quality_score * 100.0),
        "domains": report.domains,
        "last_compile": report.last_compile.map(|t| t.to_rfc3339()),
        "generated_at": report.generated_at.to_rfc3339(),
        "report_text": report.to_string(),
    });
    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_trigger_incremental_compile(
    pipeline: &Arc<McpPdfPipeline>,
    args: &serde_json::Value,
) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;
    let engine = KnowledgeEngine::new(Arc::clone(pipeline), &kb_path)?;
    let raw_dir = engine.raw_dir();
    let result = engine.incremental_compile(&raw_dir).await?;

    // Persist compile status
    let status_path = kb_path.join(".rsut_index").join("compile_status.json");
    if let Some(parent) = status_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let status = serde_json::json!({
        "running": false,
        "last_finished": chrono::Utc::now().to_rfc3339(),
        "last_outcome": "success",
        "last_duration_ms": 0,
        "entries_compiled": result.compiled,
        "entries_skipped": result.skipped,
        "message": format!("Incremental compile: {} compiled, {} skipped", result.compiled, result.skipped),
    });
    let _ = std::fs::write(&status_path, serde_json::to_string_pretty(&status).unwrap_or_default());

    Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)])
}

async fn handle_get_compile_status(args: &serde_json::Value) -> anyhow::Result<Vec<Content>> {
    let kb_path = parse_kb_path(args)?;
    let status_path = kb_path.join(".rsut_index").join("compile_status.json");

    if !status_path.exists() {
        let result = serde_json::json!({
            "running": false,
            "last_started": null,
            "last_finished": null,
            "last_duration_ms": null,
            "last_outcome": null,
            "message": "No compile has been performed yet.",
            "history": [],
        });
        return Ok(vec![Content::text(serde_json::to_string_pretty(&result)?)]);
    }

    let content = std::fs::read_to_string(&status_path)
        .map_err(|e| anyhow::anyhow!("Failed to read compile status: {}", e))?;
    let status: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse compile status: {}", e))?;

    Ok(vec![Content::text(serde_json::to_string_pretty(&status)?)])
}

// === MCP Resources Handlers ===

fn handle_resources_list(request: &JsonRpcRequest) -> JsonRpcResponse {
    let resources = serde_json::json!({
        "resources": [
            {
                "uri": "ui://dashboard/health",
                "name": "Knowledge Health Dashboard",
                "description": "Interactive dashboard showing knowledge base health metrics, domain distribution, and index statistics.",
                "mimeType": "text/html;profile=mcp-app"
            }
        ]
    });
    JsonRpcResponse::success(request.id.clone(), resources)
}

fn handle_resources_read(request: &JsonRpcRequest) -> JsonRpcResponse {
    let uri = request
        .params
        .get("uri")
        .and_then(|u| u.as_str())
        .unwrap_or("");

    match uri {
        "ui://dashboard/health" => {
            let html = UiAssets::get("dashboard.html")
                .map(|f| String::from_utf8_lossy(&f.data).into_owned())
                .unwrap_or_else(|| "<html><body>Dashboard not available</body></html>".to_string());

            let result = serde_json::json!({
                "contents": [
                    {
                        "uri": uri,
                        "mimeType": "text/html;profile=mcp-app",
                        "text": html
                    }
                ]
            });
            JsonRpcResponse::success(request.id.clone(), result)
        }
        _ => JsonRpcResponse::error(
            request.id.clone(),
            JsonRpcError::invalid_params(&format!("Unknown resource URI: {}", uri)),
        ),
    }
}
