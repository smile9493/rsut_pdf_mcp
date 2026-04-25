//! Tool handler trait
//! Defines the interface for tool execution and management

use crate::dto::{ExecutionMetadata, ToolExecutionResult};
use crate::error::PdfResult;
use crate::protocol::{RuntimeVariables, ToolDefinition, ToolSpec};
use crate::streamer::MessageStreamer;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

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

/// Tool context
/// Provides context information for tool execution
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// Tool execution ID
    pub execution_id: String,
    /// Organization ID
    pub org_id: Option<String>,
    /// Workflow ID
    pub workflow_id: Option<String>,
    /// User ID
    pub user_id: Option<String>,
    /// Request ID
    pub request_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ToolContext {
    /// Create a new tool context
    pub fn new(execution_id: String) -> Self {
        Self {
            execution_id,
            org_id: None,
            workflow_id: None,
            user_id: None,
            request_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Set organization ID
    pub fn with_org_id(mut self, org_id: String) -> Self {
        self.org_id = Some(org_id);
        self
    }

    /// Set workflow ID
    pub fn with_workflow_id(mut self, workflow_id: String) -> Self {
        self.workflow_id = Some(workflow_id);
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Tool execution options
#[derive(Debug, Clone)]
pub struct ToolExecutionOptions {
    /// Enable streaming
    pub enable_streaming: bool,
    /// Timeout in seconds
    pub timeout: Option<u64>,
    /// Enable caching
    pub enable_cache: bool,
    /// Enable metrics
    pub enable_metrics: bool,
    /// Additional options
    pub additional: HashMap<String, Value>,
}

impl Default for ToolExecutionOptions {
    fn default() -> Self {
        Self {
            enable_streaming: false,
            timeout: None,
            enable_cache: true,
            enable_metrics: true,
            additional: HashMap::new(),
        }
    }
}

impl ToolExecutionOptions {
    /// Enable streaming
    pub fn with_streaming(mut self) -> Self {
        self.enable_streaming = true;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Disable caching
    pub fn without_cache(mut self) -> Self {
        self.enable_cache = false;
        self
    }

    /// Disable metrics
    pub fn without_metrics(mut self) -> Self {
        self.enable_metrics = false;
        self
    }

    /// Add additional option
    pub fn with_option(mut self, key: String, value: Value) -> Self {
        self.additional.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_context_builder() {
        let context = ToolContext::new("exec-123".to_string())
            .with_org_id("org-456".to_string())
            .with_workflow_id("workflow-789".to_string())
            .with_user_id("user-101".to_string())
            .with_request_id("req-202".to_string())
            .with_metadata("key1".to_string(), "value1".to_string());

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
            .with_option("custom_key".to_string(), serde_json::json!("custom_value"));

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
