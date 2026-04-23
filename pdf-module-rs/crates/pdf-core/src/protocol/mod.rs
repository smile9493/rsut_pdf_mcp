//! Tool protocol definitions
//!
//! This module defines the standardized protocols for MCP tools:
//! - `ToolDefinition`: Tool metadata and schema
//! - `ToolSpec`: Runtime configuration schema
//! - `RuntimeVariables`: Environment variables for tool execution

pub mod tool_definition;
pub mod tool_spec;
pub mod runtime_variables;

pub use tool_definition::ToolDefinition;
pub use tool_spec::ToolSpec;
pub use runtime_variables::RuntimeVariables;
