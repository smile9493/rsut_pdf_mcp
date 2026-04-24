//! SSE message streamer implementation
//! Implements MessageStreamer for Server-Sent Events communication

use crate::error::PdfResult;
use crate::streamer::streamer::MessageStreamer;
use crate::streamer::tool_message::ToolMessage;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

/// SSE message streamer implementation
/// Sends messages through a channel for SSE broadcasting
#[derive(Clone)]
pub struct SseMessageStreamer {
    sender: Arc<mpsc::UnboundedSender<String>>,
}

impl SseMessageStreamer {
    /// Create a new SSE message streamer
    pub fn new(sender: mpsc::UnboundedSender<String>) -> Self {
        Self {
            sender: Arc::new(sender),
        }
    }

    /// Create a new SSE message streamer with a new channel
    /// Returns the streamer and the receiver for the channel
    pub fn new_channel() -> (Self, mpsc::UnboundedReceiver<String>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (Self::new(sender), receiver)
    }

    /// Check if the channel is still open
    pub fn is_connected(&self) -> bool {
        !self.sender.is_closed()
    }
}

#[async_trait]
impl MessageStreamer for SseMessageStreamer {
    async fn send(&self, message: ToolMessage) -> PdfResult<()> {
        let json = serde_json::to_string(&message).map_err(|e| {
            crate::error::PdfModuleError::MessageSendError(format!(
                "Failed to serialize message: {}",
                e
            ))
        })?;

        // Format as SSE event
        let sse_message = format!("data: {}\n\n", json);

        self.sender.send(sse_message).map_err(|e| {
            crate::error::PdfModuleError::MessageSendError(format!(
                "Failed to send message through channel: {}",
                e
            ))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::LogLevel;
    use crate::{ToolExecutionResult, ToolSpec, ToolDefinition, RuntimeVariables};
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_sse_streamer_creation() {
        let (streamer, mut receiver) = SseMessageStreamer::new_channel();

        assert!(streamer.is_connected());

        // Test sending a message
        let result = streamer
            .send_log(LogLevel::Info, "Test message".to_string())
            .await;

        assert!(result.is_ok());

        // Verify message was received
        let received = timeout(Duration::from_millis(100), receiver.recv())
            .await
            .expect("Timeout waiting for message")
            .expect("Channel closed");

        assert!(received.contains("data:"));
        assert!(received.contains("Test message"));
    }

    #[tokio::test]
    async fn test_sse_streamer_send_progress() {
        let (streamer, mut receiver) = SseMessageStreamer::new_channel();

        let result = streamer.send_progress(0.5, "Processing".to_string()).await;

        assert!(result.is_ok());

        let received = timeout(Duration::from_millis(100), receiver.recv())
            .await
            .expect("Timeout waiting for message")
            .expect("Channel closed");

        assert!(received.contains("data:"));
        assert!(received.contains("[50.0%]"));
    }

    #[tokio::test]
    async fn test_sse_streamer_send_result() {
        let (streamer, mut receiver) = SseMessageStreamer::new_channel();

        let result = ToolExecutionResult {
            workflow_id: "test-workflow".to_string(),
            elapsed_time: 1000,
            output: serde_json::json!({"status": "success"}),
            metadata: None,
        };

        let send_result = streamer.send_result(result).await;

        assert!(send_result.is_ok());

        let received = timeout(Duration::from_millis(100), receiver.recv())
            .await
            .expect("Timeout waiting for message")
            .expect("Channel closed");

        assert!(received.contains("data:"));
        assert!(received.contains("test-workflow"));
    }

    #[tokio::test]
    async fn test_sse_streamer_send_cost() {
        let (streamer, mut receiver) = SseMessageStreamer::new_channel();

        let result = streamer.send_cost(0.01, "USD".to_string()).await;

        assert!(result.is_ok());

        let received = timeout(Duration::from_millis(100), receiver.recv())
            .await
            .expect("Timeout waiting for message")
            .expect("Channel closed");

        assert!(received.contains("data:"));
        assert!(received.contains("0.01"));
    }

    #[tokio::test]
    async fn test_sse_streamer_send_spec() {
        let (streamer, mut receiver) = SseMessageStreamer::new_channel();

        let spec = ToolSpec::new("Test Config".to_string(), "Test configuration".to_string());

        let result = streamer.send_spec(spec).await;

        assert!(result.is_ok());

        let received = timeout(Duration::from_millis(100), receiver.recv())
            .await
            .expect("Timeout waiting for message")
            .expect("Channel closed");

        assert!(received.contains("data:"));
        assert!(received.contains("Test Config"));
    }

    #[tokio::test]
    async fn test_sse_streamer_send_properties() {
        let (streamer, mut receiver) = SseMessageStreamer::new_channel();

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

        let received = timeout(Duration::from_millis(100), receiver.recv())
            .await
            .expect("Timeout waiting for message")
            .expect("Channel closed");

        assert!(received.contains("data:"));
        assert!(received.contains("Test Tool"));
    }

    #[tokio::test]
    async fn test_sse_streamer_send_variables() {
        let (streamer, mut receiver) = SseMessageStreamer::new_channel();

        let variables = RuntimeVariables::new(
            "Test Variables".to_string(),
            "Test runtime variables".to_string(),
        );

        let result = streamer.send_variables(variables).await;

        assert!(result.is_ok());

        let received = timeout(Duration::from_millis(100), receiver.recv())
            .await
            .expect("Timeout waiting for message")
            .expect("Channel closed");

        assert!(received.contains("data:"));
    }

    #[tokio::test]
    async fn test_sse_streamer_send_icon() {
        let (streamer, mut receiver) = SseMessageStreamer::new_channel();

        let icon = "<svg>test</svg>".to_string();
        let result = streamer.send_icon(icon).await;

        assert!(result.is_ok());

        let received = timeout(Duration::from_millis(100), receiver.recv())
            .await
            .expect("Timeout waiting for message")
            .expect("Channel closed");

        assert!(received.contains("data:"));
        assert!(received.contains("<svg>"));
    }

    #[tokio::test]
    async fn test_sse_streamer_send_single_step() {
        let (streamer, mut receiver) = SseMessageStreamer::new_channel();

        let result = streamer
            .send_single_step("Step 1 completed".to_string())
            .await;

        assert!(result.is_ok());

        let received = timeout(Duration::from_millis(100), receiver.recv())
            .await
            .expect("Timeout waiting for message")
            .expect("Channel closed");

        assert!(received.contains("data:"));
    }

    #[tokio::test]
    async fn test_sse_streamer_multiple_messages() {
        let (streamer, mut receiver) = SseMessageStreamer::new_channel();

        // Send multiple messages
        for i in 0..5 {
            let result = streamer.send_info(format!("Message {}", i)).await;
            assert!(result.is_ok());
        }

        // Receive all messages
        let mut count = 0;
        while count < 5 {
            let received = timeout(Duration::from_millis(100), receiver.recv()).await;
            if received.is_ok() {
                count += 1;
            } else {
                break;
            }
        }

        assert_eq!(count, 5);
    }

    #[test]
    fn test_sse_streamer_clone() {
        let (streamer, _receiver) = SseMessageStreamer::new_channel();
        let _cloned = streamer.clone(); // Should compile
    }

    #[tokio::test]
    async fn test_sse_streamer_disconnection() {
        let (streamer, receiver) = SseMessageStreamer::new_channel();

        // Drop receiver to simulate disconnection
        drop(receiver);

        // Give time for the channel to close
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert!(!streamer.is_connected());

        // Sending should fail
        let result = streamer.send_info("Test".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_sse_streamer_sse_format() {
        let (streamer, mut receiver) = SseMessageStreamer::new_channel();

        let result = streamer.send_info("Test message".to_string()).await;

        assert!(result.is_ok());

        let received = timeout(Duration::from_millis(100), receiver.recv())
            .await
            .expect("Timeout waiting for message")
            .expect("Channel closed");

        // Verify SSE format
        assert!(received.starts_with("data:"));
        assert!(received.ends_with("\n\n"));
    }
}
