//! Capability adapters module
//! Provides tool plugin implementations for various capabilities

pub mod database;
pub mod etl_workflow;
pub mod minimax;
pub mod pdf_extractor;
pub mod remote;

pub use database::DatabasePlugin;
pub use etl_workflow::EtlWorkflowPlugin;
pub use minimax::{MiniMaxAdapterPlugin, MiniMaxConfig};
pub use pdf_extractor::PdfExtractorPlugin;
pub use remote::{AuthConfig, Protocol, RemotePluginAdapter, RemotePluginConfig};
