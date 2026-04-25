//! PDF Core Library
//!
//! This crate provides the core functionality for PDF text extraction,
//! including engine abstraction, caching, validation, and keyword extraction.

pub mod audit;
pub mod cache;
pub mod config;
pub mod container;
pub mod control;
pub mod database;
pub mod dto;
pub mod engine;
pub mod error;
pub mod extractor;
pub mod keyword;
pub mod metrics;
pub mod plugin;
pub mod protocol;
pub mod storage;
pub mod streamer;
pub mod validator;

pub use audit::{AuditBackend, AuditFilters, AuditFiltersBuilder, AuditLog, AuditService, ExtractionAudit};
pub use cache::ExtractionCache;
pub use config::ServerConfig;
pub use container::ServiceContainer;
pub use control::{
    AuditLogger, CircuitBreaker, CircuitBreakerConfig, CircuitState, ControlPlane, HealthStatus,
    MetricsCollector, MetricsSnapshot, RateLimitStats, RateLimiter, SchemaDefinition,
    SchemaManager, ToolMetrics,
};
pub use database::{SurrealStore, SurrealStoreConfig};
pub use dto::*;
pub use engine::PdfEngine;
pub use error::{PdfModuleError, PdfResult};
pub use extractor::PdfExtractorService;
pub use keyword::KeywordExtractor;
pub use plugin::{
    CompileTimeDiscovery, DispatchRequest, DispatchResult, DiscoveryConfig, DynamicDiscovery,
    MetadataCache, CacheStats, PluginRegistry, RuntimeDiscovery,
    ToolHandler, ToolDispatcher, ToolRegistration, ToolRegistry,
    UnifiedDiscovery, UnifiedDiscoveryConfig,
};
pub use protocol::{RuntimeVariables, ToolDefinition, ToolSpec};
#[cfg(feature = "s3")]
pub use storage::S3FileStorage;
pub use storage::{FileStorage, FileStorageConfig, FileStorageFactory, LocalFileStorage};
pub use streamer::{
    MessageStreamer, NoOpMessageStreamer, SseMessageStreamer, StdioMessageStreamer, ToolMessage,
};
pub use validator::{FileValidator, PathValidationConfig};

// Re-export pdf-common for unified types accessible through pdf-core
pub use pdf_common;
pub use pdf_common::PdfError as UnifiedPdfError;
