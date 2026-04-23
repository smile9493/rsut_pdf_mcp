//! Stdio message streamer implementation
//! Implements MessageStreamer for stdio communication

use crate::dto::{LogLevel, ToolExecutionResult};
use crate::error::PdfResult;
use crate::protocol::{ToolDefinition, ToolSpec, RuntimeVariables};
use crate::streamer::tool_message::ToolMessage;
use crate::streamer::streamer::MessageStreamer;
use async_trait::async_trait;
use std::io::{self, Write};

/// Stdio message streamer implementation
/// Sends messages through stdout in JSON format
#[derive(Clone)]
pub struct StdioMessageStreamer;

impl StdioMessageStreamer {
    /// Create a new stdio message streamer
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdioMessageStreamer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MessageStreamer for StdioMessageStreamer {
    async fn send(&self, message: ToolMessage) -> PdfResult<()> {
        let json = serde_json::to_string(&message)
            .map_err(|e| crate::error::PdfModuleError::MessageSendError(format!(
                "Failed to serialize message: {}",
                e
            )))?;

        writeln!(io::stdout(), "{}", json)
            .map_err(|e| crate::error::PdfModuleError::MessageSendError(format!(
                "Failed to write to stdout: {}",
                e
            )))?;

        io::stdout().flush()
            .map_err(|e| crate::error::PdfModuleError::MessageSendError(format!(
                "Failed to flush stdout: {}",
                e
            )))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::LogLevel;

    #[tokio::test]
    async fn test_stdio_streamer_send_log() {
        let streamer = StdioMessageStreamer::new();

        // This test will print to stdout, which is expected behavior
        let result = streamer
            .send_log(LogLevel::Info, "Test message".to_string())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_streamer_send_progress() {
        let streamer = StdioMessageStreamer::new();

        let result = streamer.send_progress(0.5, "Processing".to_string()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_streamer_send_result() {
        let streamer = StdioMessageStreamer::new();

        let result = ToolExecutionResult {
            workflow_id: "test-workflow".to_string(),
            elapsed_time: 1000,
            output: serde_json::json!({"status": "success"}),
            metadata: None,
        };

        let send_result = streamer.send_result(result).await;

        assert!(send_result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_streamer_send_cost() {
        let streamer = StdioMessageStreamer::new();

        let result = streamer.send_cost(0.01, "USD".to_string()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_streamer_send_spec() {
        let streamer = StdioMessageStreamer::new();

        let spec = ToolSpec::new(
            "Test Config".to_string(),
            "Test configuration".to_string(),
        );

        let result = streamer.send_spec(spec).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_streamer_send_properties() {
        let streamer = StdioMessageStreamer::new();

        let properties = ToolDefinition::new(
            "Test Tool".to_string(),
            "test_tool".to_string(),
            "Test tool description".to_string(),
            vec![],
            crate::dto::InputType::File,
            crate::dto::OutputType::File,
        );

        let result = streamer.send_properties(properties).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_streamer_send_variables() {
        let streamer = StdioMessageStreamer::new();

        let variables = RuntimeVariables::new(
            "Test Variables".to_string(),
            "Test runtime variables".to_string(),
        );

        let result = streamer.send_variables(variables).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_streamer_send_icon() {
        let streamer = StdioMessageStreamer::new();

        let icon = "<svg>test</svg>".to_string();
        let result = streamer.send_icon(icon).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_streamer_send_single_step() {
        let streamer = StdioMessageStreamer::new();

        let result = streamer.send_single_step("Step 1 completed".to_string()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stdio_streamer_default() {
        let streamer = StdioMessageStreamer::default();

        let result = streamer.send_info("Test message".to_string()).await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_stdio_streamer_clone() {
        let streamer = StdioMessageStreamer::new();
        let _cloned = streamer.clone(); // Should compile
    }
}
