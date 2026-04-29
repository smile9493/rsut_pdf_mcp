//! # vlm-visual-gateway
//!
//! VLM Visual Gateway: conditional escalation from Pdfium local parsing
//! to remote VLM layout understanding.
//!
//! ## Architecture
//!
//! ```text
//! MCP Call -> PDF Pipeline -> Pdfium (local) -> EscalationDetector
//!                                               |
//!                            +------ Normal -----+----- Escalate ----+
//!                            |                                       |
//!                      Return local                          VlmGateway -> VLM API
//!                      result fast                           Return enhanced result
//! ```
//!
//! ## Key components
//!
//! - [`VlmGateway`] — async VLM client with timeout + degradation
//! - [`VlmGatewayHandle`] — graceful shutdown for in-flight requests
//! - [`VisualBuffer`] — bumpalo Arena-backed pixel buffer (zero temp files)
//! - [`EscalationDetector`] — zero-text + layout-chaos probes
//! - [`PdfiumGuard`] — FFI panic isolation + serialisation
//! - [`MetricsCollector`] — Prometheus metrics (requests, duration, degradations)

pub mod buffer;
pub mod detector;
pub mod error;
pub mod gateway;
pub mod metrics;
pub mod pdfium_guard;
pub mod types;

pub use buffer::VisualBuffer;
pub use detector::{DetectionResult, EscalationDetector};
pub use error::{PdfiumGuardError, PdfiumGuardResult, VlmError, VlmResult};
pub use gateway::{render_page_pixels, VlmGateway, VlmGatewayHandle};
pub use metrics::{MetricsCollector, RequestTimer};
pub use pdfium_guard::{catch_pdfium, PdfiumGuard};
pub use types::{
    LayoutResult, PageComplexity, PayloadMetadata, PdfiumExtraction, Region, RegionType, VlmConfig,
    VlmModel,
};
