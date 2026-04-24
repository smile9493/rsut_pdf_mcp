//! Message streaming trait and implementations
//! Defines the interface for sending streaming messages in MCP tools

use crate::dto::{LogLevel, ToolExecutionResult};
use crate::error::PdfResult;
use crate::protocol::{RuntimeVariables, ToolDefinition, ToolSpec};
use crate::streamer::tool_message::ToolMessage;
use async_trait::async_trait;

/// Streaming message sender interface
#[async_trait]
pub trait MessageStreamer: Send + Sync {
    /// Send a tool message
    async fn send(&self, message: ToolMessage) -> PdfResult<()>;

    /// Send a log message
    async fn send_log(&self, level: LogLevel, message: String) -> PdfResult<()> {
        self.send(ToolMessage::log(level, message)).await
    }

    /// Send a progress update (as a log message)
    async fn send_progress(&self, progress: f64, message: String) -> PdfResult<()> {
        self.send_log(
            LogLevel::Info,
            format!("[{:.1}%] {}", progress * 100.0, message),
        )
        .await
    }

    /// Send a debug log message
    async fn send_debug(&self, message: String) -> PdfResult<()> {
        self.send_log(LogLevel::Debug, message).await
    }

    /// Send an info log message
    async fn send_info(&self, message: String) -> PdfResult<()> {
        self.send_log(LogLevel::Info, message).await
    }

    /// Send a warning log message
    async fn send_warn(&self, message: String) -> PdfResult<()> {
        self.send_log(LogLevel::Warn, message).await
    }

    /// Send an error log message
    async fn send_error(&self, message: String) -> PdfResult<()> {
        self.send_log(LogLevel::Error, message).await
    }

    /// Send a fatal log message
    async fn send_fatal(&self, message: String) -> PdfResult<()> {
        self.send_log(LogLevel::Fatal, message).await
    }

    /// Send a result message
    async fn send_result(&self, result: ToolExecutionResult) -> PdfResult<()> {
        self.send(ToolMessage::result(result)).await
    }

    /// Send a cost message
    async fn send_cost(&self, cost: f64, cost_units: String) -> PdfResult<()> {
        self.send(ToolMessage::cost(cost, cost_units)).await
    }

    /// Send a tool specification message
    async fn send_spec(&self, spec: ToolSpec) -> PdfResult<()> {
        self.send(ToolMessage::spec(spec)).await
    }

    /// Send a tool properties message
    async fn send_properties(&self, properties: ToolDefinition) -> PdfResult<()> {
        self.send(ToolMessage::properties(properties)).await
    }

    /// Send a tool icon message
    async fn send_icon(&self, icon: String) -> PdfResult<()> {
        self.send(ToolMessage::icon(icon)).await
    }

    /// Send a runtime variables message
    async fn send_variables(&self, variables: RuntimeVariables) -> PdfResult<()> {
        self.send(ToolMessage::variables(variables)).await
    }

    /// Send a single step debug message
    async fn send_single_step(&self, message: String) -> PdfResult<()> {
        self.send(ToolMessage::single_step(message)).await
    }
}

/// No-op message streamer for testing
#[derive(Debug, Clone)]
pub struct NoOpMessageStreamer;

#[async_trait]
impl MessageStreamer for NoOpMessageStreamer {
    async fn send(&self, _message: ToolMessage) -> PdfResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_streamer() {
        let streamer = NoOpMessageStreamer;
        // NoOp streamer should compile and be usable
        // Actual async tests would need a runtime
        let _ = streamer;
    }
}
