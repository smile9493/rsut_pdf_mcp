//! Transport layer abstraction
//! Provides stdio transport implementation

use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use async_trait::async_trait;
use std::io::{BufRead, Write};
use tracing::{debug, error};

/// Transport trait
#[async_trait]
pub trait Transport: Send + Sync {
    async fn receive(&self) -> anyhow::Result<Option<JsonRpcRequest>>;
    async fn send(&self, response: &JsonRpcResponse) -> anyhow::Result<()>;
    async fn close(&self) -> anyhow::Result<()>;
}

/// Stdio transport
pub struct StdioTransport;

impl StdioTransport {
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
            Ok(0) => Ok(None),
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

/// Transport type (stdio only, per design philosophy)
#[derive(Debug, Clone, Default)]
pub struct TransportType;

impl TransportType {
    pub fn stdio() -> Self {
        Self
    }
}
