//! Dynamic discovery trait
//! Defines the interface for dynamic tool discovery

use crate::error::PdfResult;
use crate::plugin::ToolHandler;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;

/// Tool registration entry (for compile-time discovery)
pub struct ToolRegistration {
    /// Tool name
    pub name: &'static str,
    /// Tool factory function
    pub factory: fn() -> Arc<dyn ToolHandler>,
}

/// Dynamic discovery trait
/// Defines the interface for discovering and loading tools
#[async_trait]
pub trait DynamicDiscovery: Send + Sync {
    /// Discover all available tools
    async fn discover(&self) -> PdfResult<Vec<Arc<dyn ToolHandler>>>;

    /// Load a tool from a specific path
    async fn load_from_path(&self, path: &str) -> PdfResult<Arc<dyn ToolHandler>>;

    /// Hot reload a tool by name
    async fn hot_reload(&self, tool_name: &str) -> PdfResult<()>;

    /// Scan a directory for plugins
    async fn scan_directory(&self, dir: &str) -> PdfResult<Vec<String>>;
}

/// Discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Enable compile-time discovery
    pub enable_compile_time: bool,
    /// Enable runtime discovery
    pub enable_runtime: bool,
    /// Plugin directories to scan
    pub plugin_dirs: Vec<PathBuf>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_compile_time: true,
            enable_runtime: false,
            plugin_dirs: vec![],
        }
    }
}

impl DiscoveryConfig {
    /// Create a new discovery configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable compile-time discovery
    pub fn with_compile_time(mut self) -> Self {
        self.enable_compile_time = true;
        self
    }

    /// Enable runtime discovery
    pub fn with_runtime(mut self) -> Self {
        self.enable_runtime = true;
        self
    }

    /// Add a plugin directory
    pub fn with_plugin_dir(mut self, dir: PathBuf) -> Self {
        self.plugin_dirs.push(dir);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert!(config.enable_compile_time);
        assert!(!config.enable_runtime);
        assert!(config.plugin_dirs.is_empty());
    }

    #[test]
    fn test_discovery_config_builder() {
        let config = DiscoveryConfig::new()
            .with_compile_time()
            .with_runtime()
            .with_plugin_dir(PathBuf::from("/plugins"));

        assert!(config.enable_compile_time);
        assert!(config.enable_runtime);
        assert_eq!(config.plugin_dirs.len(), 1);
    }
}
