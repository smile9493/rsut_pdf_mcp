//! Audit and monitoring module
//!
//! This module provides audit logging functionality for PDF extraction operations:
//! - `ExtractionAudit`: Audit record structure for PDF extraction
//! - `AuditService`: Audit service with multiple backend support
//! - `AuditLog`: Audit log for plugin architecture
//! - `AuditFilter`: Query filter for audit logs

pub mod audit_log;
pub mod audit_service;
pub mod extraction_audit;

pub use audit_log::{AuditFilter, AuditLog};
pub use audit_service::{AuditBackend, AuditFilters, AuditFiltersBuilder, AuditService};
pub use extraction_audit::ExtractionAudit;

// Re-export AuditBackendConfig from config for convenience
pub use crate::config::AuditBackendConfig;
