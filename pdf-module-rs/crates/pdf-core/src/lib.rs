//! PDF Core Library
//!
//! This crate provides the core functionality for PDF text extraction,
//! including engine abstraction, caching, validation, and keyword extraction.

pub mod error;
pub mod dto;
pub mod config;
pub mod validator;
pub mod cache;
pub mod engine;
pub mod keyword;
pub mod metrics;
pub mod extractor;
pub mod protocol;
pub mod streamer;
pub mod audit;
pub mod storage;
pub mod plugin;

pub use error::{PdfModuleError, PdfResult};
pub use dto::*;
pub use config::ServerConfig;
pub use validator::{FileValidator, PathValidationConfig};
pub use cache::ExtractionCache;
pub use engine::PdfEngine;
pub use keyword::KeywordExtractor;
pub use extractor::PdfExtractorService;
pub use protocol::{ToolDefinition, ToolSpec, RuntimeVariables};
pub use streamer::{ToolMessage, MessageStreamer, StdioMessageStreamer, SseMessageStreamer, NoOpMessageStreamer};
pub use audit::{ExtractionAudit, AuditService, AuditBackend, AuditFilters, AuditFiltersBuilder};
pub use storage::{FileStorage, FileStorageConfig, FileStorageFactory, LocalFileStorage};
#[cfg(feature = "s3")]
pub use storage::S3FileStorage;
pub use plugin::{ToolHandler, ToolContext, ToolExecutionOptions, ToolRegistry};
