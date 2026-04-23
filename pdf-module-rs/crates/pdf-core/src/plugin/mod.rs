//! Plugin module
//! Provides tool plugin architecture with registration and execution

pub mod tool_handler;
pub mod registry;

pub use tool_handler::{ToolHandler, ToolContext, ToolExecutionOptions};
pub use registry::ToolRegistry;
