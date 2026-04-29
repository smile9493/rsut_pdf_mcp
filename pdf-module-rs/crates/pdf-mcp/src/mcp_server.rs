//! MCP Server launcher
//! Manages MCP server lifecycle with plugin architecture

use crate::protocol::McpProtocolHandler;
use crate::transport::{StdioTransport, Transport};
use pdf_core::plugin::{PluginRegistry, ToolRegistry, UnifiedDiscovery, UnifiedDiscoveryConfig};
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, warn};

/// MCP Server configuration
#[derive(Debug, Clone, Default)]
pub struct McpServerConfig {
    /// Enable compile-time discovery
    pub enable_compile_time_discovery: bool,
    /// Enable runtime discovery
    pub enable_runtime_discovery: bool,
    /// Plugin directories
    pub plugin_dirs: Vec<std::path::PathBuf>,
}

impl McpServerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_compile_time_discovery(mut self) -> Self {
        self.enable_compile_time_discovery = true;
        self
    }

    pub fn with_runtime_discovery(mut self) -> Self {
        self.enable_runtime_discovery = true;
        self
    }

    pub fn with_plugin_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.plugin_dirs.push(dir);
        self
    }
}

/// MCP Server
/// Main server struct that manages the MCP protocol and plugin lifecycle
pub struct McpServer {
    /// Server configuration
    config: McpServerConfig,
    /// Plugin registry
    registry: Arc<ToolRegistry>,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(config: McpServerConfig) -> Self {
        let registry = Arc::new(ToolRegistry::new());
        Self { config, registry }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(McpServerConfig::default())
    }

    /// Get a reference to the plugin registry
    pub fn registry(&self) -> Arc<ToolRegistry> {
        self.registry.clone()
    }

    /// Register a tool directly
    pub async fn register_tool(
        &self,
        tool: Arc<dyn pdf_core::plugin::ToolHandler>,
    ) -> pdf_core::error::PdfResult<()> {
        self.registry.register(tool).await
    }

    /// Discover and register tools
    pub async fn discover_and_register(&self) -> anyhow::Result<()> {
        let discovery_config = UnifiedDiscoveryConfig::new()
            .with_compile_time()
            .with_plugin_dir(std::path::PathBuf::from("./plugins"));

        let discovery = UnifiedDiscovery::new(discovery_config);
        let tools = discovery.discover().await?;

        info!("Discovered {} tools", tools.len());

        for tool in tools {
            let name = tool.name().to_string();
            match self.registry.register(tool).await {
                Ok(()) => info!("Registered tool: {}", name),
                Err(e) => warn!("Failed to register tool '{}': {}", name, e),
            }
        }

        Ok(())
    }

    /// Start the MCP server
    pub async fn start(&self) -> anyhow::Result<()> {
        info!("Starting MCP server");

        // Discover and register tools
        if let Err(e) = self.discover_and_register().await {
            warn!("Tool discovery failed: {}", e);
        }

        let tool_count = self.registry.count().await;
        info!("Registered {} tools", tool_count);

        // Create protocol handler
        let handler = McpProtocolHandler::new(self.registry.clone());

        // Start stdio transport
        info!("Starting stdio transport");
        self.run_stdio(&handler).await
    }

    /// Run with stdio transport
    async fn run_stdio(&self, handler: &McpProtocolHandler) -> anyhow::Result<()> {
        let transport = StdioTransport::new();

        info!("MCP server ready (stdio), waiting for requests...");

        loop {
            tokio::select! {
                // Handle incoming requests
                request = transport.receive() => {
                    match request {
                        Ok(Some(req)) => {
                            let response = handler.handle_request(req).await;
                            transport.send(&response).await?;
                        }
                        Ok(None) => {
                            // EOF, client disconnected
                            info!("Client disconnected");
                            break;
                        }
                        Err(e) => {
                            error!("Receive error: {}", e);
                            let response = crate::protocol::JsonRpcResponse::error(
                                None,
                                crate::protocol::JsonRpcError::parse_error(),
                            );
                            transport.send(&response).await?;
                        }
                    }
                }
                // Handle shutdown signal
                _ = signal::ctrl_c() => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }

        transport.close().await?;
        info!("MCP server stopped");
        Ok(())
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) {
        info!("Shutting down MCP server");
        let tools = self.registry.list_tools().await;
        for name in tools {
            if let Err(e) = self.registry.unregister(&name).await {
                warn!("Failed to unregister tool '{}': {}", name, e);
            }
        }
        info!("All tools unregistered");
    }
}
