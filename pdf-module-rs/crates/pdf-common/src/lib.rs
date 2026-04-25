//! Common types and utilities for the PDF module workspace.
//!
//! This crate provides:
//! - **Unified error type** (`PdfError`) consolidating pdf-core and pdf-etl errors
//! - **Shared DTOs** (`ToolContext`, `ToolExecutionOptions`, extraction types)
//! - **Circuit breaker** (`CircuitBreaker`, `EngineCircuitBreaker`)
//! - **Unified configuration** (`AppConfig`)
//! - **Core traits** (`FileStorage`, `PluginRegistry`, `ToolHandler`, etc.)

pub mod error;
pub mod dto;
pub mod config;
pub mod circuit_breaker;
pub mod traits;

// Re-export commonly used types at the crate root.
pub use error::{PdfError, Result, ErrorCategory};
pub use dto::{ToolContext, ToolExecutionOptions};
pub use config::AppConfig;
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState, EngineCircuitBreaker};
