//! MCP Server entry point

use clap::{Parser, Subcommand};
use pdf_core::{PdfExtractorService, ServerConfig};
use std::sync::Arc;
use tracing::info;

mod server;
mod sse;

#[derive(Parser)]
#[command(name = "pdf-mcp", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start MCP server
    Serve {
        /// Transport mode: stdio or sse
        #[arg(long, default_value = "stdio")]
        transport: String,
        /// SSE listen port (only for sse mode)
        #[arg(long, default_value_t = 8001)]
        port: u16,
    },
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

    // Create service
    let service = Arc::new(PdfExtractorService::new(&config)?);

    match cli.command {
        Commands::Serve { transport, port } => {
            info!("Starting MCP server with transport: {}", transport);
            match transport.as_str() {
                "stdio" => {
                    server::run_stdio(service).await?;
                }
                "sse" => {
                    sse::run_sse(service, port).await?;
                }
                _ => {
                    anyhow::bail!("Unknown transport mode: {}", transport);
                }
            }
        }
    }

    Ok(())
}
