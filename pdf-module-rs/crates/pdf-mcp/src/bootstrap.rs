//! Bootstrap module
//! Initializes the MCP server with tool discovery and registration

use crate::mcp_server::{McpServer, McpServerConfig};
use pdf_core::plugin::{
    DatabasePlugin, EtlWorkflowPlugin, MiniMaxAdapterPlugin, MiniMaxConfig, PdfExtractorPlugin,
    ToolHandler,
};
use pdf_core::PdfExtractorService;
use std::sync::Arc;
use tracing::{info, warn};

/// Bootstrap configuration
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    /// Enable compile-time discovery
    pub enable_compile_time_discovery: bool,
    /// Enable runtime discovery
    pub enable_runtime_discovery: bool,
    /// Plugin directories
    pub plugin_dirs: Vec<std::path::PathBuf>,
    /// Register built-in adapters
    pub register_builtin_adapters: bool,
    /// MiniMax configuration (if enabled)
    pub minimax_config: Option<MiniMaxConfig>,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            enable_compile_time_discovery: true,
            enable_runtime_discovery: false,
            plugin_dirs: vec![],
            register_builtin_adapters: true,
            minimax_config: None,
        }
    }
}

impl BootstrapConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_runtime_discovery(mut self) -> Self {
        self.enable_runtime_discovery = true;
        self
    }

    pub fn with_plugin_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.plugin_dirs.push(dir);
        self
    }

    pub fn without_builtin_adapters(mut self) -> Self {
        self.register_builtin_adapters = false;
        self
    }

    pub fn with_minimax(mut self, config: MiniMaxConfig) -> Self {
        self.minimax_config = Some(config);
        self
    }
}

/// Bootstrap the MCP server
/// Creates and configures the server with all components
pub async fn bootstrap(
    extractor_service: Arc<PdfExtractorService>,
    config: BootstrapConfig,
) -> anyhow::Result<McpServer> {
    info!("Bootstrapping MCP server");

    let server_config = McpServerConfig {
        enable_compile_time_discovery: config.enable_compile_time_discovery,
        enable_runtime_discovery: config.enable_runtime_discovery,
        plugin_dirs: config.plugin_dirs,
    };

    let server = McpServer::new(server_config);

    // Register built-in adapters
    if config.register_builtin_adapters {
        register_builtin_adapters(&server, &extractor_service, &config).await?;
    }

    // Discover and register additional tools
    if let Err(e) = server.discover_and_register().await {
        warn!("Tool discovery failed: {}", e);
    }

    let tool_count = server.registry().count().await;
    info!("Bootstrap complete: {} tools registered", tool_count);

    Ok(server)
}

/// Register built-in adapter plugins
async fn register_builtin_adapters(
    server: &McpServer,
    extractor_service: &Arc<PdfExtractorService>,
    config: &BootstrapConfig,
) -> anyhow::Result<()> {
    // PDF Extractor Plugin
    let pdf_plugin = PdfExtractorPlugin::new(extractor_service.clone());
    match server.register_tool(Arc::new(pdf_plugin)).await {
        Ok(()) => info!("Registered PDF Extractor plugin"),
        Err(e) => warn!("Failed to register PDF Extractor: {}", e),
    }

    // ETL Workflow Plugin
    let etl_plugin = EtlWorkflowPlugin::new();
    match server.register_tool(Arc::new(etl_plugin)).await {
        Ok(()) => info!("Registered ETL Workflow plugin"),
        Err(e) => warn!("Failed to register ETL Workflow: {}", e),
    }

    // Database Plugin
    let db_plugin = DatabasePlugin::new();
    match server.register_tool(Arc::new(db_plugin)).await {
        Ok(()) => info!("Registered Database plugin"),
        Err(e) => warn!("Failed to register Database: {}", e),
    }

    // MiniMax Plugin (if configured)
    if let Some(ref minimax_config) = config.minimax_config {
        let minimax_plugin = MiniMaxAdapterPlugin::new(minimax_config.clone());
        match server.register_tool(Arc::new(minimax_plugin)).await {
            Ok(()) => info!("Registered MiniMax adapter plugin"),
            Err(e) => warn!("Failed to register MiniMax adapter: {}", e),
        }
    }

    Ok(())
}
