//! Plugin registry trait
//! Defines the interface for plugin registration and management

use crate::error::PdfResult;
use crate::plugin::ToolHandler;
use crate::protocol::ToolDefinition;
use async_trait::async_trait;
use std::sync::Arc;

/// Plugin registry trait
/// Defines the interface for managing tool plugins
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Register a tool
    async fn register(&self, tool: Arc<dyn ToolHandler>) -> PdfResult<()>;

    /// Unregister a tool by name
    async fn unregister(&self, name: &str) -> PdfResult<()>;

    /// Get a tool by name
    async fn get(&self, name: &str) -> PdfResult<Arc<dyn ToolHandler>>;

    /// Check if a tool is registered
    async fn is_registered(&self, name: &str) -> bool;

    /// List all tool names
    async fn list_tools(&self) -> Vec<String>;

    /// List all tool definitions
    async fn list_definitions(&self) -> Vec<ToolDefinition>;

    /// Query tools by capability
    async fn query_by_capability(&self, capability: &str) -> Vec<ToolDefinition>;

    /// Query tools by category
    async fn query_by_category(&self, category: &str) -> Vec<ToolDefinition>;

    /// Get the number of registered tools
    async fn count(&self) -> usize;

    /// Clear all registered tools
    async fn clear(&self);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockRegistry;

    #[async_trait]
    impl PluginRegistry for MockRegistry {
        async fn register(&self, _tool: Arc<dyn ToolHandler>) -> PdfResult<()> {
            Ok(())
        }

        async fn unregister(&self, _name: &str) -> PdfResult<()> {
            Ok(())
        }

        async fn get(&self, name: &str) -> PdfResult<Arc<dyn ToolHandler>> {
            Err(crate::error::PdfModuleError::ToolNotFound(name.to_string()))
        }

        async fn is_registered(&self, _name: &str) -> bool {
            false
        }

        async fn list_tools(&self) -> Vec<String> {
            vec![]
        }

        async fn list_definitions(&self) -> Vec<ToolDefinition> {
            vec![]
        }

        async fn query_by_capability(&self, _capability: &str) -> Vec<ToolDefinition> {
            vec![]
        }

        async fn query_by_category(&self, _category: &str) -> Vec<ToolDefinition> {
            vec![]
        }

        async fn count(&self) -> usize {
            0
        }

        async fn clear(&self) {}
    }

    #[tokio::test]
    async fn test_mock_registry() {
        let registry = MockRegistry;
        assert_eq!(registry.count().await, 0);
        assert!(!registry.is_registered("test").await);
        assert!(registry.list_tools().await.is_empty());
    }
}
