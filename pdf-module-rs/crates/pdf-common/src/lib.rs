pub mod error;
pub mod dto;
pub mod config;
pub mod traits;

pub use error::{PdfError, Result, ErrorCategory};
pub use dto::{ToolContext, ToolExecutionOptions};
pub use config::AppConfig;
