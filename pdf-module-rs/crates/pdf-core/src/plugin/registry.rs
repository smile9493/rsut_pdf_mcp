//! Tool registry
//! Manages tool registration, discovery, and execution

use crate::error::{PdfModuleError, PdfResult};
use crate::plugin::tool_handler::{ToolContext, ToolExecutionOptions, ToolHandler};
use crate::plugin::PluginRegistry;
use crate::protocol::ToolDefinition;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tool registry
/// Manages tool registration, discovery, and execution
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn ToolHandler>>>,
    /// Category index: category -> tool names
    categories: RwLock<HashMap<String, Vec<String>>>,
    /// Capability index: capability -> tool names
    capabilities: RwLock<HashMap<String, Vec<String>>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            categories: RwLock::new(HashMap::new()),
            capabilities: RwLock::new(HashMap::new()),
        }
    }

    /// Register a tool
    pub async fn register(&self, tool: Arc<dyn ToolHandler>) -> PdfResult<()> {
        let name = tool.name().to_string();
        
        // Call lifecycle hook
        tool.on_register().await?;
        
        let mut tools = self.tools.write().await;

        if tools.contains_key(&name) {
            return Err(PdfModuleError::ToolAlreadyRegistered(name));
        }

        // Update category index
        let category = tool.category();
        let mut categories = self.categories.write().await;
        categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(name.clone());

        // Update capability index
        let tool_capabilities = tool.capabilities();
        let mut capabilities = self.capabilities.write().await;
        for cap in tool_capabilities {
            capabilities
                .entry(cap)
                .or_insert_with(Vec::new)
                .push(name.clone());
        }

        tools.insert(name.clone(), tool);
        Ok(())
    }

    /// Unregister a tool
    pub async fn unregister(&self, name: &str) -> PdfResult<()> {
        let mut tools = self.tools.write().await;

        let tool = tools.remove(name).ok_or_else(|| {
            PdfModuleError::ToolNotFound(format!("Tool '{}' is not registered", name))
        })?;

        // Call lifecycle hook
        tool.on_unregister().await?;

        // Clean up category index
        let category = tool.category();
        let mut categories = self.categories.write().await;
        if let Some(tool_names) = categories.get_mut(&category) {
            tool_names.retain(|n| n != name);
        }

        // Clean up capability index
        let tool_capabilities = tool.capabilities();
        let mut capabilities = self.capabilities.write().await;
        for cap in tool_capabilities {
            if let Some(tool_names) = capabilities.get_mut(&cap) {
                tool_names.retain(|n| n != name);
            }
        }

        Ok(())
    }

    /// Get a tool by name
    pub async fn get(&self, name: &str) -> PdfResult<Arc<dyn ToolHandler>> {
        let tools = self.tools.read().await;

        tools.get(name).cloned().ok_or_else(|| {
            PdfModuleError::ToolNotFound(format!("Tool '{}' is not registered", name))
        })
    }

    /// Check if a tool is registered
    pub async fn is_registered(&self, name: &str) -> bool {
        let tools = self.tools.read().await;
        tools.contains_key(name)
    }

    /// List all registered tools
    pub async fn list_tools(&self) -> Vec<String> {
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }

    /// Get all tool definitions
    pub async fn list_definitions(&self) -> Vec<ToolDefinition> {
        let tools = self.tools.read().await;
        tools
            .values()
            .map(|tool| tool.definition().clone())
            .collect()
    }

    /// Execute a tool by name
    pub async fn execute(
        &self,
        name: &str,
        params: HashMap<String, Value>,
        _context: Option<ToolContext>,
        _options: Option<ToolExecutionOptions>,
        streamer: Option<&mut dyn crate::streamer::MessageStreamer>,
    ) -> PdfResult<crate::dto::ToolExecutionResult> {
        let tool = self.get(name).await?;

        // Validate parameters
        tool.validate_params(&params)?;

        // Execute the tool
        tool.execute(params, streamer).await
    }

    /// Get tool count
    pub async fn count(&self) -> usize {
        let tools = self.tools.read().await;
        tools.len()
    }

    /// Clear all tools
    pub async fn clear(&self) {
        let mut tools = self.tools.write().await;
        tools.clear();
    }

    /// Check if any tools are registered
    pub async fn is_empty(&self) -> bool {
        let tools = self.tools.read().await;
        tools.is_empty()
    }

    /// Query tools by capability
    pub async fn query_by_capability(&self, capability: &str) -> Vec<ToolDefinition> {
        let capabilities = self.capabilities.read().await;
        let tools = self.tools.read().await;

        capabilities
            .get(capability)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| tools.get(name).map(|t| t.definition().clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Query tools by category
    pub async fn query_by_category(&self, category: &str) -> Vec<ToolDefinition> {
        let categories = self.categories.read().await;
        let tools = self.tools.read().await;

        categories
            .get(category)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| tools.get(name).map(|t| t.definition().clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Implement PluginRegistry trait for ToolRegistry
#[async_trait]
impl PluginRegistry for ToolRegistry {
    async fn register(&self, tool: Arc<dyn ToolHandler>) -> PdfResult<()> {
        Self::register(self, tool).await
    }

    async fn unregister(&self, name: &str) -> PdfResult<()> {
        Self::unregister(self, name).await
    }

    async fn get(&self, name: &str) -> PdfResult<Arc<dyn ToolHandler>> {
        Self::get(self, name).await
    }

    async fn is_registered(&self, name: &str) -> bool {
        Self::is_registered(self, name).await
    }

    async fn list_tools(&self) -> Vec<String> {
        Self::list_tools(self).await
    }

    async fn list_definitions(&self) -> Vec<ToolDefinition> {
        Self::list_definitions(self).await
    }

    async fn query_by_capability(&self, capability: &str) -> Vec<ToolDefinition> {
        Self::query_by_capability(self, capability).await
    }

    async fn query_by_category(&self, category: &str) -> Vec<ToolDefinition> {
        Self::query_by_category(self, category).await
    }

    async fn count(&self) -> usize {
        Self::count(self).await
    }

    async fn clear(&self) {
        Self::clear(self).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{InputType, OutputType};
    use crate::plugin::tool_handler::ToolHandler;
    use crate::protocol::{RuntimeVariables, ToolDefinition, ToolSpec};
    use async_trait::async_trait;

    // Mock tool handler for testing
    struct MockToolHandler {
        name: String,
        definition: ToolDefinition,
        spec: ToolSpec,
        variables: RuntimeVariables,
    }

    impl MockToolHandler {
        fn new(name: &str) -> Self {
            let definition = ToolDefinition::new(
                format!("{} Display", name),
                name.to_string(),
                format!("{} description", name),
                vec![],
                InputType::File,
                OutputType::File,
            );

            let spec = ToolSpec::new(name.to_string(), "test".to_string());

            let variables =
                RuntimeVariables::new("Test Variables".to_string(), "Test Description".to_string());

            Self {
                name: name.to_string(),
                definition,
                spec,
                variables,
            }
        }
    }

    #[async_trait]
    impl ToolHandler for MockToolHandler {
        fn definition(&self) -> &ToolDefinition {
            &self.definition
        }

        fn spec(&self) -> &ToolSpec {
            &self.spec
        }

        fn variables(&self) -> &RuntimeVariables {
            &self.variables
        }

        async fn execute(
            &self,
            _params: HashMap<String, Value>,
            _streamer: Option<&mut dyn crate::streamer::MessageStreamer>,
        ) -> PdfResult<crate::dto::ToolExecutionResult> {
            Ok(crate::dto::ToolExecutionResult {
                workflow_id: "test-workflow".to_string(),
                elapsed_time: 100,
                output: serde_json::json!({"result": "test"}),
                metadata: Some(crate::dto::ExecutionMetadata {
                    file_name: "test.pdf".to_string(),
                    file_size: 1024,
                    processing_time: 100,
                    cache_hit: false,
                    adapter_used: "test".to_string(),
                }),
            })
        }

        fn validate_params(&self, _params: &HashMap<String, Value>) -> PdfResult<()> {
            Ok(())
        }

        fn metadata(&self) -> crate::dto::ExecutionMetadata {
            crate::dto::ExecutionMetadata {
                file_name: "test.pdf".to_string(),
                file_size: 1024,
                processing_time: 100,
                cache_hit: false,
                adapter_used: "test".to_string(),
            }
        }
    }

    #[tokio::test]
    async fn test_tool_registry_register() {
        let registry = ToolRegistry::new();
        let tool = Arc::new(MockToolHandler::new("test_tool"));

        registry.register(tool.clone()).await.unwrap();
        assert!(registry.is_registered("test_tool").await);
        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_tool_registry_duplicate_registration() {
        let registry = ToolRegistry::new();
        let tool = Arc::new(MockToolHandler::new("test_tool"));

        registry.register(tool.clone()).await.unwrap();
        let result = registry.register(tool).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tool_registry_unregister() {
        let registry = ToolRegistry::new();
        let tool = Arc::new(MockToolHandler::new("test_tool"));

        registry.register(tool).await.unwrap();
        assert!(registry.is_registered("test_tool").await);

        registry.unregister("test_tool").await.unwrap();
        assert!(!registry.is_registered("test_tool").await);
        assert_eq!(registry.count().await, 0);
    }

    #[tokio::test]
    async fn test_tool_registry_get() {
        let registry = ToolRegistry::new();
        let tool = Arc::new(MockToolHandler::new("test_tool"));

        registry.register(tool.clone()).await.unwrap();
        let retrieved = registry.get("test_tool").await.unwrap();
        assert_eq!(retrieved.name(), "test_tool");
    }

    #[tokio::test]
    async fn test_tool_registry_get_not_found() {
        let registry = ToolRegistry::new();
        let result = registry.get("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tool_registry_list_tools() {
        let registry = ToolRegistry::new();

        let tool1 = Arc::new(MockToolHandler::new("tool1"));
        let tool2 = Arc::new(MockToolHandler::new("tool2"));

        registry.register(tool1).await.unwrap();
        registry.register(tool2).await.unwrap();

        let tools = registry.list_tools().await;
        assert_eq!(tools.len(), 2);
        assert!(tools.contains(&"tool1".to_string()));
        assert!(tools.contains(&"tool2".to_string()));
    }

    #[tokio::test]
    async fn test_tool_registry_list_definitions() {
        let registry = ToolRegistry::new();

        let tool1 = Arc::new(MockToolHandler::new("tool1"));
        let tool2 = Arc::new(MockToolHandler::new("tool2"));

        registry.register(tool1).await.unwrap();
        registry.register(tool2).await.unwrap();

        let definitions = registry.list_definitions().await;
        assert_eq!(definitions.len(), 2);

        let names: Vec<&str> = definitions
            .iter()
            .map(|d| d.function_name.as_str())
            .collect();
        assert!(names.contains(&"tool1"));
        assert!(names.contains(&"tool2"));
    }

    #[tokio::test]
    async fn test_tool_registry_execute() {
        let registry = ToolRegistry::new();
        let tool = Arc::new(MockToolHandler::new("test_tool"));

        registry.register(tool).await.unwrap();

        let params = HashMap::new();
        let result = registry
            .execute("test_tool", params, None, None, None)
            .await
            .unwrap();
        assert_eq!(result.workflow_id, "test-workflow");
        assert!(result.output.get("result").is_some());
    }

    #[tokio::test]
    async fn test_tool_registry_clear() {
        let registry = ToolRegistry::new();

        let tool1 = Arc::new(MockToolHandler::new("tool1"));
        let tool2 = Arc::new(MockToolHandler::new("tool2"));

        registry.register(tool1).await.unwrap();
        registry.register(tool2).await.unwrap();
        assert_eq!(registry.count().await, 2);

        registry.clear().await;
        assert_eq!(registry.count().await, 0);
        assert!(registry.is_empty().await);
    }

    #[tokio::test]
    async fn test_tool_registry_is_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty().await);

        let tool = Arc::new(MockToolHandler::new("test_tool"));
        registry.register(tool).await.unwrap();
        assert!(!registry.is_empty().await);
    }
}
