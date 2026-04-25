//! Transport layer abstraction
//! Provides stdio and SSE transport implementations

use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use async_trait::async_trait;
use serde_json::Value;
use std::io::{BufRead, Write};
use tracing::{debug, error};

/// Transport trait
/// Defines the interface for MCP message transport
#[async_trait]
pub trait Transport: Send + Sync {
    /// Receive a JSON-RPC request
    async fn receive(&self) -> anyhow::Result<Option<JsonRpcRequest>>;

    /// Send a JSON-RPC response
    async fn send(&self, response: &JsonRpcResponse) -> anyhow::Result<()>;

    /// Close the transport
    async fn close(&self) -> anyhow::Result<()>;
}

/// Stdio transport
/// Uses stdin/stdout for message transport
pub struct StdioTransport;

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn receive(&self) -> anyhow::Result<Option<JsonRpcRequest>> {
        let stdin = std::io::stdin();
        let mut line = String::new();

        match stdin.lock().read_line(&mut line) {
            Ok(0) => Ok(None), // EOF
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    return Ok(None);
                }
                debug!("Received: {}", line);
                match serde_json::from_str(line) {
                    Ok(request) => Ok(Some(request)),
                    Err(e) => {
                        error!("Failed to parse request: {}", e);
                        Err(anyhow::anyhow!("Parse error: {}", e))
                    }
                }
            }
            Err(e) => Err(anyhow::anyhow!("IO error: {}", e)),
        }
    }

    async fn send(&self, response: &JsonRpcResponse) -> anyhow::Result<()> {
        let json = serde_json::to_string(response)?;
        debug!("Sending: {}", json);

        let stdout = std::io::stdout();
        let mut stdout_lock = stdout.lock();
        writeln!(stdout_lock, "{}", json)?;
        stdout_lock.flush()?;

        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// SSE transport configuration
#[derive(Debug, Clone)]
pub struct SseTransportConfig {
    /// Host to bind
    pub host: String,
    /// Port to listen
    pub port: u16,
}

impl Default for SseTransportConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
        }
    }
}

/// HTTP transport configuration
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    /// Host to bind
    pub host: String,
    /// Port to listen
    pub port: u16,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3001,
        }
    }
}

/// Transport type enumeration
#[derive(Debug, Clone)]
pub enum TransportType {
    /// Standard I/O transport
    Stdio,
    /// Server-Sent Events transport
    Sse(SseTransportConfig),
    /// HTTP transport
    Http(HttpTransportConfig),
}

impl Default for TransportType {
    fn default() -> Self {
        Self::Stdio
    }
}

impl TransportType {
    /// Create stdio transport
    pub fn stdio() -> Self {
        Self::Stdio
    }

    /// Create SSE transport with default config
    pub fn sse() -> Self {
        Self::Sse(SseTransportConfig::default())
    }

    /// Create SSE transport with custom config
    pub fn sse_with(host: String, port: u16) -> Self {
        Self::Sse(SseTransportConfig { host, port })
    }

    /// Create HTTP transport with default config
    pub fn http() -> Self {
        Self::Http(HttpTransportConfig::default())
    }

    /// Create HTTP transport with custom config
    pub fn http_with(host: String, port: u16) -> Self {
        Self::Http(HttpTransportConfig { host, port })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_type_default() {
        let transport = TransportType::default();
        assert!(matches!(transport, TransportType::Stdio));
    }

    #[test]
    fn test_transport_type_sse() {
        let transport = TransportType::sse();
        assert!(matches!(transport, TransportType::Sse(_)));
    }

    #[test]
    fn test_transport_type_http() {
        let transport = TransportType::http();
        assert!(matches!(transport, TransportType::Http(_)));
    }

    #[test]
    fn test_sse_config_default() {
        let config = SseTransportConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
    }
}
