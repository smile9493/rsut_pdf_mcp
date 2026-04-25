//! Plugin module
//! Provides tool plugin architecture with registration and execution

pub mod adapters;
pub mod compile_time_discovery;
pub mod discovery;
pub mod dispatcher;
pub mod metadata_cache;
pub mod registry;
pub mod registry_trait;
pub mod runtime_discovery;
pub mod tool_handler;
pub mod unified_discovery;

pub use adapters::{
    AuthConfig, DatabasePlugin, EtlWorkflowPlugin, MiniMaxAdapterPlugin, MiniMaxConfig,
    PdfExtractorPlugin, Protocol, RemotePluginAdapter, RemotePluginConfig,
};
pub use compile_time_discovery::{
    CompileTimeDiscovery, ToolRegistration as CompileTimeToolRegistration,
};
pub use discovery::{DiscoveryConfig, DynamicDiscovery, ToolRegistration};
pub use dispatcher::{DispatchRequest, DispatchResult, ToolDispatcher};
pub use metadata_cache::{CacheStats, MetadataCache};
pub use registry::ToolRegistry;
pub use registry_trait::PluginRegistry;
pub use runtime_discovery::RuntimeDiscovery;
pub use tool_handler::{ToolContext, ToolExecutionOptions, ToolHandler};
pub use unified_discovery::{UnifiedDiscovery, UnifiedDiscoveryConfig};
