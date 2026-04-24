//! Tool dispatcher trait
//! Defines the interface for tool dispatching and execution

use crate::dto::ToolExecutionResult;
use crate::error::PdfResult;
use crate::plugin::{ToolContext, ToolExecutionOptions};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Dispatch request
#[derive(Debug, Clone)]
pub struct DispatchRequest {
    /// Tool name
    pub tool_name: String,
    /// Tool parameters
    pub params: HashMap<String, Value>,
    /// Execution context
    pub context: Option<ToolContext>,
    /// Execution options
    pub options: Option<ToolExecutionOptions>,
}

impl DispatchRequest {
    /// Create a new dispatch request
    pub fn new(tool_name: String, params: HashMap<String, Value>) -> Self {
        Self {
            tool_name,
            params,
            context: None,
            options: None,
        }
    }

    /// Set execution context
    pub fn with_context(mut self, context: ToolContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Set execution options
    pub fn with_options(mut self, options: ToolExecutionOptions) -> Self {
        self.options = Some(options);
        self
    }
}

/// Dispatch result
#[derive(Debug)]
pub struct DispatchResult {
    /// Original request
    pub request: DispatchRequest,
    /// Execution result
    pub result: PdfResult<ToolExecutionResult>,
}

/// Tool dispatcher trait
/// Defines the interface for dispatching tool execution requests
#[async_trait]
pub trait ToolDispatcher: Send + Sync {
    /// Dispatch and execute a tool
    async fn dispatch(
        &self,
        tool_name: &str,
        params: HashMap<String, Value>,
        context: Option<ToolContext>,
        options: Option<ToolExecutionOptions>,
    ) -> PdfResult<ToolExecutionResult>;

    /// Dispatch and execute multiple tools in batch
    async fn dispatch_batch(&self, requests: Vec<DispatchRequest>) -> Vec<DispatchResult>;

    /// Perform health check on all tools
    async fn health_check(&self) -> HashMap<String, bool>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_request_builder() {
        let params = HashMap::new();
        let context = ToolContext::new("exec-123".to_string());
        let options = ToolExecutionOptions::default();

        let request = DispatchRequest::new("test_tool".to_string(), params)
            .with_context(context)
            .with_options(options);

        assert_eq!(request.tool_name, "test_tool");
        assert!(request.context.is_some());
        assert!(request.options.is_some());
    }
}
