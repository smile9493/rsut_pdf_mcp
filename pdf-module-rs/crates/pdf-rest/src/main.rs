//! REST API server entry point

use clap::Parser;
use pdf_core::{PdfExtractorService, ServerConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

mod routes;

#[derive(Parser)]
#[command(name = "pdf-rest", version = "0.1.0")]
struct Cli {
    /// Listen host
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    /// Listen port
    #[arg(long, default_value_t = 8000)]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenvy::dotenv().ok();

    // Parse CLI
    let cli = Cli::parse();

    // Load config
    let config = ServerConfig::from_env()?;
    config.init_tracing();

    // Initialize metrics
    pdf_core::metrics::init_metrics();

    // Create service
    let service = Arc::new(PdfExtractorService::new(&config)?);

    // Build router
    let app = routes::build_router(service);

    // Start server
    let addr: SocketAddr = format!("{}:{}", cli.host, cli.port).parse()?;
    info!("Starting REST API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
