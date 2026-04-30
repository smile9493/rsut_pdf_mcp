//! # pdf-mcp
//!
//! A MCP (Model Context Protocol) server for PDF extraction.
//!
//! ## Architecture
//!
//! This binary provides a stdio-based MCP server that exposes PDF processing
//! capabilities to AI assistants. It uses the `pdf-core` crate for PDF parsing
//! via the Pdfium engine.
//!
//! ## Features
//!
//! - Extract plain text from PDF files
//! - Extract structured data (per-page text with bounding boxes)
//! - Get page count
//! - Search for keywords with context
//!
//! ## Usage
//!
//! ```bash
//! cargo run --release --bin pdf-mcp
//! ```
//!
//! ## Environment Variables
//!
//! - `VLM_ENDPOINT`: VLM API endpoint URL
//! - `VLM_API_KEY`: VLM API key
//! - `VLM_MODEL`: Target model (default: gpt-4o)
//! - `VLM_TIMEOUT_SECS`: Request timeout in seconds (default: 30)
//! - `VLM_MAX_CONCURRENCY`: Max concurrent VLM requests (default: 5)
//! - `VLM_MAX_RETRIES`: Max retry attempts (default: 3)
//! - `VLM_RETRY_DELAY_BASE_SECS`: Base retry delay (default: 1)
//! - `VLM_RETRY_DELAY_MAX_SECS`: Max retry delay (default: 30)

use pdf_core::{McpPdfPipeline, ServerConfig};
use std::sync::Arc;
use tracing::info;

mod sampling;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = ServerConfig::from_env()?;
    config.init_tracing();

    let pipeline = Arc::new(McpPdfPipeline::new(&config)?);
    info!("Starting MCP server (stdio only, pdfium engine)");

    server::run_stdio(pipeline).await
}
