//! Tool handler trait
//! Defines the interface for tool execution and management
//!
//! `ToolContext` and `ToolExecutionOptions` are re-exported from `pdf_common`
//! (unified source of truth). Only the `ToolHandler` trait remains defined here
//! as it depends on pdf-core-specific protocol types.

use crate::dto::{ExecutionMetadata, ToolExecutionResult};
use crate::error::PdfResult;
use crate::protocol::{RuntimeVariables, ToolDefinition, ToolSpec};
use crate::streamer::MessageStreamer;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

// Re-export unified types from pdf-common (single source of truth).
pub use pdf_common::dto::{ToolContext, ToolExecutionOptions};

/// Tool handler trait
/// Defines the interface for tool execution and lifecycle management
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Get the tool definition
    fn definition(&self) -> &ToolDefinition;

    /// Get the tool specification
    fn spec(&self) -> &ToolSpec;

    /// Get the runtime variables
    fn variables(&self) -> &RuntimeVariables;

    /// Execute the tool with given parameters
    async fn execute(
        &self,
        params: HashMap<String, Value>,
        streamer: Option<&mut dyn MessageStreamer>,
    ) -> PdfResult<ToolExecutionResult>;

    /// Validate the tool parameters
    fn validate_params(&self, params: &HashMap<String, Value>) -> PdfResult<()>;

    /// Get tool metadata
    fn metadata(&self) -> ExecutionMetadata;

    /// Check if the tool is available
    async fn is_available(&self) -> bool {
        true
    }

    /// Get tool version
    fn version(&self) -> &str {
        self.definition().latest_version()
    }

    /// Get tool name
    fn name(&self) -> &str {
        &self.definition().function_name
    }

    /// Get tool description
    fn description(&self) -> &str {
        &self.definition().description
    }

    /// Lifecycle hook: called when tool is registered
    async fn on_register(&self) -> PdfResult<()> {
        Ok(())
    }

    /// Lifecycle hook: called when tool is unregistered
    async fn on_unregister(&self) -> PdfResult<()> {
        Ok(())
    }

    /// Get capability tags
    fn capabilities(&self) -> Vec<String> {
        vec![]
    }

    /// Get tool category
    fn category(&self) -> String {
        "general".to_string()
    }

    /// Get tool dependencies
    fn dependencies(&self) -> Vec<String> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_context_builder() {
        let context = ToolContext::new("exec-123")
            .with_org_id("org-456")
            .with_workflow_id("workflow-789")
            .with_user_id("user-101")
            .with_request_id("req-202")
            .with_metadata("key1", "value1");

        assert_eq!(context.execution_id, "exec-123");
        assert_eq!(context.org_id, Some("org-456".to_string()));
        assert_eq!(context.workflow_id, Some("workflow-789".to_string()));
        assert_eq!(context.user_id, Some("user-101".to_string()));
        assert_eq!(context.request_id, Some("req-202".to_string()));
        assert_eq!(context.metadata.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_tool_execution_options_default() {
        let options = ToolExecutionOptions::default();
        assert!(!options.enable_streaming);
        assert!(options.timeout.is_none());
        assert!(options.enable_cache);
        assert!(options.enable_metrics);
        assert!(options.additional.is_empty());
    }

    #[test]
    fn test_tool_execution_options_builder() {
        let options = ToolExecutionOptions::default()
            .with_streaming()
            .with_timeout(60)
            .without_cache()
            .without_metrics()
            .with_option("custom_key", serde_json::json!("custom_value"));

        assert!(options.enable_streaming);
        assert_eq!(options.timeout, Some(60));
        assert!(!options.enable_cache);
        assert!(!options.enable_metrics);
        assert_eq!(
            options.additional.get("custom_key"),
            Some(&serde_json::json!("custom_value"))
        );
    }
}
