use pdf_core::{McpPdfPipeline, ServerConfig};
use std::sync::Arc;
use tracing::info;

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
