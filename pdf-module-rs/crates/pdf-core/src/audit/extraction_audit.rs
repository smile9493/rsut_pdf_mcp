//! Extraction audit model
//! Defines the audit record structure for PDF extraction operations

use crate::dto::ExtractionStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// PDF extraction audit record
/// Contains complete information about a PDF extraction operation for audit purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionAudit {
    /// Audit record ID
    pub id: Uuid,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Organization ID (multi-tenant support)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,

    /// Workflow ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,

    /// File execution ID
    pub file_execution_id: Uuid,

    /// File name
    pub file_name: String,

    /// File type
    pub file_type: String,

    /// File size in KB
    pub file_size_kb: f64,

    /// Extraction status
    pub status: ExtractionStatus,

    /// Adapter used for extraction
    pub adapter_used: String,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Whether cache was hit
    pub cache_hit: bool,

    /// Length of extracted text
    pub extracted_text_length: usize,

    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,

    /// Cost trace (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,

    /// Cost units
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_units: Option<String>,
}

impl ExtractionAudit {
    /// Create a new extraction audit record
    pub fn new(
        file_name: String,
        file_type: String,
        file_size_kb: f64,
        adapter_used: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            org_id: None,
            workflow_id: None,
            file_execution_id: Uuid::new_v4(),
            file_name,
            file_type,
            file_size_kb,
            status: ExtractionStatus::Pending,
            adapter_used,
            processing_time_ms: 0,
            cache_hit: false,
            extracted_text_length: 0,
            error_message: None,
            cost: None,
            cost_units: None,
        }
    }

    /// Set the organization ID
    pub fn with_org_id(mut self, org_id: String) -> Self {
        self.org_id = Some(org_id);
        self
    }

    /// Set the workflow ID
    pub fn with_workflow_id(mut self, workflow_id: String) -> Self {
        self.workflow_id = Some(workflow_id);
        self
    }

    /// Set the file execution ID
    pub fn with_file_execution_id(mut self, file_execution_id: Uuid) -> Self {
        self.file_execution_id = file_execution_id;
        self
    }

    /// Set the status
    pub fn with_status(mut self, status: ExtractionStatus) -> Self {
        self.status = status;
        self
    }

    /// Set the processing time
    pub fn with_processing_time(mut self, processing_time_ms: u64) -> Self {
        self.processing_time_ms = processing_time_ms;
        self
    }

    /// Set cache hit status
    pub fn with_cache_hit(mut self, cache_hit: bool) -> Self {
        self.cache_hit = cache_hit;
        self
    }

    /// Set the extracted text length
    pub fn with_extracted_text_length(mut self, length: usize) -> Self {
        self.extracted_text_length = length;
        self
    }

    /// Set the error message
    pub fn with_error(mut self, error: String) -> Self {
        self.error_message = Some(error);
        self.status = ExtractionStatus::Failed;
        self
    }

    /// Set the cost
    pub fn with_cost(mut self, cost: f64, units: String) -> Self {
        self.cost = Some(cost);
        self.cost_units = Some(units);
        self
    }

    /// Mark as processing
    pub fn mark_processing(&mut self) {
        self.status = ExtractionStatus::Processing;
    }

    /// Mark as completed
    pub fn mark_completed(&mut self, processing_time_ms: u64, extracted_text_length: usize) {
        self.status = ExtractionStatus::Completed;
        self.processing_time_ms = processing_time_ms;
        self.extracted_text_length = extracted_text_length;
    }

    /// Mark as failed
    pub fn mark_failed(&mut self, error: String) {
        self.status = ExtractionStatus::Failed;
        self.error_message = Some(error);
    }

    /// Check if the extraction was successful
    pub fn is_successful(&self) -> bool {
        matches!(self.status, ExtractionStatus::Completed)
    }

    /// Check if the extraction failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, ExtractionStatus::Failed)
    }

    /// Get the duration as a formatted string
    pub fn duration_str(&self) -> String {
        if self.processing_time_ms < 1000 {
            format!("{}ms", self.processing_time_ms)
        } else {
            format!("{:.2}s", self.processing_time_ms as f64 / 1000.0)
        }
    }

    /// Get the file size as a formatted string
    pub fn file_size_str(&self) -> String {
        if self.file_size_kb < 1024.0 {
            format!("{:.2} KB", self.file_size_kb)
        } else {
            format!("{:.2} MB", self.file_size_kb / 1024.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_audit_creation() {
        let audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        );

        assert_eq!(audit.file_name, "test.pdf");
        assert_eq!(audit.file_type, "pdf");
        assert_eq!(audit.adapter_used, "lopdf");
        assert_eq!(audit.status, ExtractionStatus::Pending);
        assert!(!audit.cache_hit);
        assert!(audit.id != Uuid::nil());
    }

    #[test]
    fn test_extraction_audit_builder_pattern() {
        let audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        )
        .with_org_id("org-123".to_string())
        .with_workflow_id("workflow-456".to_string())
        .with_status(ExtractionStatus::Processing)
        .with_processing_time(1000)
        .with_cache_hit(true)
        .with_extracted_text_length(5000);

        assert_eq!(audit.org_id, Some("org-123".to_string()));
        assert_eq!(audit.workflow_id, Some("workflow-456".to_string()));
        assert_eq!(audit.status, ExtractionStatus::Processing);
        assert_eq!(audit.processing_time_ms, 1000);
        assert!(audit.cache_hit);
        assert_eq!(audit.extracted_text_length, 5000);
    }

    #[test]
    fn test_extraction_audit_mark_completed() {
        let mut audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        );

        assert_eq!(audit.status, ExtractionStatus::Pending);

        audit.mark_completed(1500, 10000);

        assert_eq!(audit.status, ExtractionStatus::Completed);
        assert_eq!(audit.processing_time_ms, 1500);
        assert_eq!(audit.extracted_text_length, 10000);
        assert!(audit.is_successful());
        assert!(!audit.is_failed());
    }

    #[test]
    fn test_extraction_audit_mark_failed() {
        let mut audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        );

        assert_eq!(audit.status, ExtractionStatus::Pending);

        audit.mark_failed("File corrupted".to_string());

        assert_eq!(audit.status, ExtractionStatus::Failed);
        assert_eq!(audit.error_message, Some("File corrupted".to_string()));
        assert!(!audit.is_successful());
        assert!(audit.is_failed());
    }

    #[test]
    fn test_extraction_audit_with_error() {
        let audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        )
        .with_error("Cannot read file".to_string());

        assert_eq!(audit.status, ExtractionStatus::Failed);
        assert_eq!(audit.error_message, Some("Cannot read file".to_string()));
        assert!(audit.is_failed());
    }

    #[test]
    fn test_extraction_audit_with_cost() {
        let audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        )
        .with_cost(0.05, "USD".to_string());

        assert_eq!(audit.cost, Some(0.05));
        assert_eq!(audit.cost_units, Some("USD".to_string()));
    }

    #[test]
    fn test_extraction_audit_duration_str() {
        let audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        )
        .with_processing_time(500);

        assert_eq!(audit.duration_str(), "500ms");

        let audit = audit.with_processing_time(2500);
        assert_eq!(audit.duration_str(), "2.50s");
    }

    #[test]
    fn test_extraction_audit_file_size_str() {
        let audit1 = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            512.0,
            "lopdf".to_string(),
        );

        assert_eq!(audit1.file_size_str(), "512.00 KB");

        let audit2 = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            2048.0,
            "lopdf".to_string(),
        );
        assert_eq!(audit2.file_size_str(), "2.00 MB");
    }

    #[test]
    fn test_extraction_audit_serialization() {
        let audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        )
        .with_org_id("org-123".to_string())
        .with_workflow_id("workflow-456".to_string())
        .with_status(ExtractionStatus::Completed)
        .with_processing_time(1000)
        .with_extracted_text_length(5000);

        let json = serde_json::to_string(&audit).unwrap();
        assert!(json.contains("test.pdf"));
        assert!(json.contains("org-123"));
        assert!(json.contains("workflow-456"));

        let deserialized: ExtractionAudit = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.file_name, "test.pdf");
        assert_eq!(deserialized.org_id, Some("org-123".to_string()));
    }

    #[test]
    fn test_extraction_audit_status_transitions() {
        let mut audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        );

        // Initial status
        assert_eq!(audit.status, ExtractionStatus::Pending);

        // Mark as processing
        audit.mark_processing();
        assert_eq!(audit.status, ExtractionStatus::Processing);

        // Mark as completed
        audit.mark_completed(1000, 5000);
        assert_eq!(audit.status, ExtractionStatus::Completed);
        assert!(audit.is_successful());
    }

    #[test]
    fn test_extraction_audit_optional_fields() {
        let audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        );

        // Check that optional fields are None by default
        assert!(audit.org_id.is_none());
        assert!(audit.workflow_id.is_none());
        assert!(audit.error_message.is_none());
        assert!(audit.cost.is_none());
        assert!(audit.cost_units.is_none());
    }
}
