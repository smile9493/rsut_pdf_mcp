//! MCP SSE (Server-Sent Events) transport implementation
//! Provides HTTP-based MCP server for web clients

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use pdf_core::PdfExtractorService;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::mpsc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info};

use crate::server::{handle_request, JsonRpcRequest, JsonRpcResponse};

/// Run MCP server with SSE transport
pub async fn run_sse(service: Arc<PdfExtractorService>, port: u16) -> anyhow::Result<()> {
    info!("Starting MCP SSE server on port {}", port);

    let app = Router::new()
        // SSE endpoint for streaming responses
        .route("/sse", get(sse_handler))
        // HTTP POST endpoint for simple requests
        .route("/message", post(message_handler))
        // Health check
        .route("/health", get(health))
        // CORS for web clients
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(service);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("MCP SSE server listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
async fn health() -> &'static str {
    "OK"
}

/// SSE endpoint - returns a stream of events
async fn sse_handler(
    #[allow(unused_variables)] State(service): State<Arc<PdfExtractorService>>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, anyhow::Error>>> {
    // Create a channel for sending events
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Send initial connection event
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let init_event = serde_json::json!({
            "type": "connection",
            "status": "connected",
            "server": "pdf-module-mcp",
            "version": "0.1.0"
        });
        let _ = tx_clone.send(init_event.to_string()).await;
    });

    // Create stream from channel
    let stream = async_stream::stream! {
        while let Some(msg) = rx.recv().await {
            yield Ok(Event::default().data(msg));
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// HTTP POST message handler - for request/response mode
async fn message_handler(
    State(service): State<Arc<PdfExtractorService>>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    debug!("Received HTTP request: {:?}", request.method);
    let response = handle_request(&service, request).await;
    Json(response)
}

/// Query parameters for SSE
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct SseQuery {
    #[serde(default)]
    session_id: Option<String>,
}
