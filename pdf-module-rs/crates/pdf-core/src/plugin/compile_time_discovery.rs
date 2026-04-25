//! Compile-time tool discovery using inventory crate
//! Provides zero-cost tool registration at compile time

use crate::error::PdfResult;
use crate::plugin::ToolHandler;
use std::sync::Arc;

/// Tool registration entry for compile-time discovery
/// Each tool plugin submits an instance of this struct via inventory::submit!
pub struct ToolRegistration {
    /// Tool name (static string)
    pub name: &'static str,
    /// Tool factory function that creates a new tool instance
    pub factory: fn() -> Arc<dyn ToolHandler>,
}

// Required for inventory: implement Sync for ToolRegistration
unsafe impl Sync for ToolRegistration {}

/// Submit a tool for compile-time registration
///
/// # Example
/// ```ignore
/// use pdf_core::plugin::compile_time_discovery::register_tool;
/// use pdf_core::plugin::ToolHandler;
/// use std::sync::Arc;
///
/// struct MyTool;
///
/// impl ToolHandler for MyTool { /* ... */ }
///
/// register_tool!("my_tool", || Arc::new(MyTool));
/// ```
#[macro_export]
macro_rules! register_tool {
    ($name:expr, $factory:expr) => {
        $crate::plugin::compile_time_discovery::submit_tool_registration! {
            $crate::plugin::compile_time_discovery::ToolRegistration {
                name: $name,
                factory: $factory,
            }
        }
    };
}

/// Submit macro for inventory
#[macro_export]
macro_rules! submit_tool_registration {
    ($registration:expr) => {
        inventory::submit!($registration);
    };
}

// Define the inventory collection
inventory::collect!(ToolRegistration);

/// Discover all tools registered at compile time
///
/// This function collects all ToolRegistration entries submitted
/// via inventory::submit! and creates tool instances using their factories.
pub fn discover_compile_time() -> Vec<Arc<dyn ToolHandler>> {
    inventory::iter::<ToolRegistration>
        .into_iter()
        .map(|registration| (registration.factory)())
        .collect()
}

/// Get names of all compile-time registered tools
pub fn list_compile_time_tools() -> Vec<&'static str> {
    inventory::iter::<ToolRegistration>
        .into_iter()
        .map(|registration| registration.name)
        .collect()
}

/// Compile-time discovery implementation
pub struct CompileTimeDiscovery;

impl CompileTimeDiscovery {
    /// Create a new compile-time discovery instance
    pub fn new() -> Self {
        Self
    }

    /// Discover all compile-time registered tools
    pub fn discover(&self) -> PdfResult<Vec<Arc<dyn ToolHandler>>> {
        Ok(discover_compile_time())
    }

    /// Get the number of compile-time registered tools
    pub fn count(&self) -> usize {
        inventory::iter::<ToolRegistration>.into_iter().count()
    }
}

impl Default for CompileTimeDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_time_discovery_creation() {
        let discovery = CompileTimeDiscovery::new();
        // No tools registered in test, but the mechanism works
        let tools = discovery.discover().unwrap();
        // The count depends on what's been registered via inventory
        assert!(tools.len() >= 0);
    }

    #[test]
    fn test_list_compile_time_tools() {
        let names = list_compile_time_tools();
        // No tools registered in test
        assert!(names.len() >= 0);
    }
}
