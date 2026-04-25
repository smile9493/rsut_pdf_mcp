//! Runtime tool discovery using libloading
//! Provides dynamic loading of tool plugins from shared libraries

use crate::error::{PdfModuleError, PdfResult};
use crate::plugin::ToolHandler;
use libloading::Library;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Plugin ABI version for compatibility check
const PLUGIN_ABI_VERSION: u32 = 1;

/// Runtime discovery for dynamically loaded plugins
pub struct RuntimeDiscovery {
    /// Loaded shared libraries
    libraries: Vec<Library>,
    /// Plugin directory to scan
    plugin_dir: PathBuf,
}

impl RuntimeDiscovery {
    /// Create a new runtime discovery instance
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            libraries: Vec::new(),
            plugin_dir,
        }
    }

    /// Scan the plugin directory for available plugins
    pub fn scan_plugins(&self) -> PdfResult<Vec<String>> {
        let mut plugins = Vec::new();

        if !self.plugin_dir.exists() {
            return Ok(plugins);
        }

        for entry in std::fs::read_dir(&self.plugin_dir)? {
            let entry = entry?;
            let path = entry.path();

            if Self::is_plugin_file(&path) {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    plugins.push(name.to_string());
                }
            }
        }

        Ok(plugins)
    }

    /// Load a plugin from a shared library path
    ///
    /// # Safety
    /// This function uses unsafe code to load a dynamic library and
    /// resolve symbols. The plugin must implement the correct ABI.
    pub fn load_plugin(&mut self, path: &Path) -> PdfResult<Arc<dyn ToolHandler>> {
        unsafe {
            let lib = Library::new(path).map_err(|e| {
                PdfModuleError::PluginLoadError(format!(
                    "Failed to load library '{}': {}",
                    path.display(),
                    e
                ))
            })?;

            // Check ABI version
            let abi_version: Result<libloading::Symbol<u32>, _> = lib.get(b"plugin_abi_version");
            match abi_version {
                Ok(version) => {
                    if *version != PLUGIN_ABI_VERSION {
                        return Err(PdfModuleError::PluginLoadError(format!(
                            "ABI version mismatch: expected {}, got {}",
                            PLUGIN_ABI_VERSION,
                            *version
                        )));
                    }
                }
                Err(_) => {
                    return Err(PdfModuleError::PluginLoadError(
                        "Plugin missing abi_version symbol".to_string(),
                    ));
                }
            }

            // Get plugin factory
            #[allow(clippy::type_complexity)]
            let factory: Result<libloading::Symbol<fn() -> Arc<dyn ToolHandler>>, _> =
                lib.get(b"plugin_factory");
            match factory {
                Ok(factory) => {
                    let plugin = factory();
                    self.libraries.push(lib);
                    Ok(plugin)
                }
                Err(_) => Err(PdfModuleError::PluginLoadError(format!(
                    "Plugin '{}' missing plugin_factory symbol",
                    path.display()
                ))),
            }
        }
    }

    /// Unload a plugin by index
    pub fn unload_plugin(&mut self, index: usize) -> PdfResult<()> {
        if index < self.libraries.len() {
            self.libraries.remove(index);
            Ok(())
        } else {
            Err(PdfModuleError::PluginLoadError(format!(
                "Invalid plugin index: {}",
                index
            )))
        }
    }

    /// Get the number of loaded plugins
    pub fn loaded_count(&self) -> usize {
        self.libraries.len()
    }

    /// Check if a file is a valid plugin (shared library)
    fn is_plugin_file(path: &Path) -> bool {
        #[cfg(target_os = "linux")]
        {
            path.extension().and_then(|e| e.to_str()) == Some("so")
        }
        #[cfg(target_os = "macos")]
        {
            path.extension().and_then(|e| e.to_str()) == Some("dylib")
        }
        #[cfg(target_os = "windows")]
        {
            path.extension().and_then(|e| e.to_str()) == Some("dll")
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            false
        }
    }

    /// Get the plugin directory path
    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }
}

impl Drop for RuntimeDiscovery {
    fn drop(&mut self) {
        // Libraries are automatically unloaded when dropped
        self.libraries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_runtime_discovery_creation() {
        let discovery = RuntimeDiscovery::new(PathBuf::from("/tmp/plugins"));
        assert_eq!(discovery.loaded_count(), 0);
        assert_eq!(discovery.plugin_dir(), Path::new("/tmp/plugins"));
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let discovery = RuntimeDiscovery::new(PathBuf::from("/nonexistent/plugins"));
        let plugins = discovery.scan_plugins().unwrap();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_is_plugin_file() {
        #[cfg(target_os = "linux")]
        {
            assert!(RuntimeDiscovery::is_plugin_file(Path::new("libplugin.so")));
            assert!(!RuntimeDiscovery::is_plugin_file(Path::new("plugin.txt")));
        }
    }
}
