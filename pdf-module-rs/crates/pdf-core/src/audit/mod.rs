//! Audit and monitoring module
//!
//! This module provides audit logging functionality for PDF extraction operations:
//! - `ExtractionAudit`: Audit record structure for PDF extraction
//! - `AuditService`: Audit service with multiple backend support

pub mod extraction_audit;
pub mod audit_service;

pub use extraction_audit::ExtractionAudit;
pub use audit_service::{AuditService, AuditBackend, AuditFilters, AuditFiltersBuilder};

// Re-export AuditBackendConfig from config for convenience
pub use crate::config::AuditBackendConfig;
