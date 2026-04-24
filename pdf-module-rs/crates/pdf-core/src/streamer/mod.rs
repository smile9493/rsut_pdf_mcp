//! Message streaming module
//!
//! This module provides message streaming capabilities for MCP tools:
//! - `ToolMessage`: Various message types for tool communication
//! - `MessageStreamer`: Trait for sending streaming messages
//! - `StdioMessageStreamer`: Implementation for stdio communication
//! - `SseMessageStreamer`: Implementation for SSE communication

pub mod sse;
pub mod stdio;
#[allow(clippy::module_inception)]
pub mod streamer;
pub mod tool_message;

pub use sse::SseMessageStreamer;
pub use stdio::StdioMessageStreamer;
pub use streamer::{MessageStreamer, NoOpMessageStreamer};
pub use tool_message::ToolMessage;
