//! Message streaming module
//!
//! This module provides message streaming capabilities for MCP tools:
//! - `ToolMessage`: Various message types for tool communication
//! - `MessageStreamer`: Trait for sending streaming messages
//! - `StdioMessageStreamer`: Implementation for stdio communication
//! - `SseMessageStreamer`: Implementation for SSE communication

pub mod tool_message;
pub mod streamer;
pub mod stdio;
pub mod sse;

pub use tool_message::ToolMessage;
pub use streamer::{MessageStreamer, NoOpMessageStreamer};
pub use stdio::StdioMessageStreamer;
pub use sse::SseMessageStreamer;
