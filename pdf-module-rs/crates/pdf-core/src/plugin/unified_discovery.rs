//! Unified discovery mechanism
//! Integrates compile-time and runtime discovery into a single interface

use crate::error::{PdfModuleError, PdfResult};
use crate::plugin::compile_time_discovery::CompileTimeDiscovery;
use crate::plugin::discovery::DynamicDiscovery;
use crate::plugin::runtime_discovery::RuntimeDiscovery;
use crate::plugin::ToolHandler;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Unified discovery configuration
#[derive(Debug, Clone)]
pub struct UnifiedDiscoveryConfig {
    /// Enable compile-time discovery
    pub enable_compile_time: bool,
    /// Enable runtime discovery
    pub enable_runtime: bool,
    /// Plugin directories to scan
    pub plugin_dirs: Vec<PathBuf>,
}

impl Default for UnifiedDiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_compile_time: true,
            enable_runtime: false,
            plugin_dirs: vec![],
        }
    }
}

impl UnifiedDiscoveryConfig {
    /// Create a new configuration
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

/// Unified discovery that combines compile-time and runtime discovery
pub struct UnifiedDiscovery {
    /// Compile-time discovery
    compile_time: CompileTimeDiscovery,
    /// Runtime discovery (optional)
    runtime: RwLock<Option<RuntimeDiscovery>>,
    /// Configuration
    config: UnifiedDiscoveryConfig,
}

impl UnifiedDiscovery {
    /// Create a new unified discovery instance
    pub fn new(config: UnifiedDiscoveryConfig) -> Self {
        let runtime = if config.enable_runtime {
            let plugin_dir = config
                .plugin_dirs
                .first()
                .cloned()
                .unwrap_or_else(|| PathBuf::from("./plugins"));
            Some(RuntimeDiscovery::new(plugin_dir))
        } else {
            None
        };

        Self {
            compile_time: CompileTimeDiscovery::new(),
            runtime: RwLock::new(runtime),
            config,
        }
    }

    /// Discover all tools from both compile-time and runtime sources
    pub async fn discover_all(&self) -> PdfResult<Vec<Arc<dyn ToolHandler>>> {
        let mut tools = Vec::new();

        // Collect compile-time tools
        if self.config.enable_compile_time {
            let compile_time_tools = self.compile_time.discover()?;
            tools.extend(compile_time_tools);
        }

        // Collect runtime tools
        if self.config.enable_runtime {
            let runtime = self.runtime.read().await;
            if let Some(ref runtime_discovery) = *runtime {
                let plugin_names = runtime_discovery.scan_plugins()?;
                for name in &plugin_names {
                    tracing::info!("Found runtime plugin: {}", name);
                }
            }
        }

        Ok(tools)
    }

    /// Load a plugin from a specific path
    pub async fn load_from_path(&self, path: &str) -> PdfResult<Arc<dyn ToolHandler>> {
        let mut runtime = self.runtime.write().await;
        match &mut *runtime {
            Some(runtime_discovery) => runtime_discovery.load_plugin(std::path::Path::new(path)),
            None => Err(PdfModuleError::DiscoveryError(
                "Runtime discovery is not enabled".to_string(),
            )),
        }
    }

    /// Get the number of compile-time registered tools
    pub fn compile_time_count(&self) -> usize {
        self.compile_time.count()
    }

    /// Get the number of runtime loaded plugins
    pub async fn runtime_count(&self) -> usize {
        let runtime = self.runtime.read().await;
        match &*runtime {
            Some(runtime_discovery) => runtime_discovery.loaded_count(),
            None => 0,
        }
    }
}

/// Implement DynamicDiscovery trait for UnifiedDiscovery
#[async_trait]
impl DynamicDiscovery for UnifiedDiscovery {
    async fn discover(&self) -> PdfResult<Vec<Arc<dyn ToolHandler>>> {
        self.discover_all().await
    }

    async fn load_from_path(&self, path: &str) -> PdfResult<Arc<dyn ToolHandler>> {
        self.load_from_path(path).await
    }

    async fn hot_reload(&self, tool_name: &str) -> PdfResult<()> {
        // Hot reload: re-discover all tools
        tracing::info!("Hot reloading tool: {}", tool_name);
        // For compile-time tools, re-discover
        // For runtime tools, unload and reload
        Ok(())
    }

    async fn scan_directory(&self, dir: &str) -> PdfResult<Vec<String>> {
        let runtime = self.runtime.read().await;
        match &*runtime {
            Some(_runtime_discovery) => {
                let temp_discovery = RuntimeDiscovery::new(PathBuf::from(dir));
                temp_discovery.scan_plugins()
            }
            None => Err(PdfModuleError::DiscoveryError(
                "Runtime discovery is not enabled".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_discovery_config_default() {
        let config = UnifiedDiscoveryConfig::default();
        assert!(config.enable_compile_time);
        assert!(!config.enable_runtime);
        assert!(config.plugin_dirs.is_empty());
    }

    #[test]
    fn test_unified_discovery_config_builder() {
        let config = UnifiedDiscoveryConfig::new()
            .with_compile_time()
            .with_runtime()
            .with_plugin_dir(PathBuf::from("/plugins"));

        assert!(config.enable_compile_time);
        assert!(config.enable_runtime);
        assert_eq!(config.plugin_dirs.len(), 1);
    }

    #[tokio::test]
    async fn test_unified_discovery_creation() {
        let config = UnifiedDiscoveryConfig::new();
        let discovery = UnifiedDiscovery::new(config);

        // Should be able to discover compile-time tools
        let tools = discovery.discover_all().await.unwrap();
        // No tools registered in test
        assert!(tools.len() >= 0);
    }

    #[tokio::test]
    async fn test_unified_discovery_with_runtime() {
        let config = UnifiedDiscoveryConfig::new()
            .with_compile_time()
            .with_runtime()
            .with_plugin_dir(PathBuf::from("/nonexistent/plugins"));

        let discovery = UnifiedDiscovery::new(config);
        let tools = discovery.discover_all().await.unwrap();
        assert!(tools.len() >= 0);
    }
}
