//! Audit service implementation
//! Provides audit logging functionality with multiple backend support

use crate::audit::extraction_audit::ExtractionAudit;
use crate::config::AuditBackendConfig;
use crate::dto::ExtractionStatus;
use crate::error::{PdfModuleError, PdfResult};
use chrono::NaiveDate;
use serde_json;
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

/// Audit backend configuration
#[derive(Debug, Clone)]
pub enum AuditBackend {
    /// File backend
    File { log_dir: PathBuf },
    /// Database backend (placeholder)
    Database {
        connection_string: String,
        table_name: String,
    },
    /// Remote service backend (placeholder)
    Remote {
        endpoint: String,
        api_key: String,
    },
    /// Memory backend (for testing)
    Memory,
}

/// Audit service
/// Manages audit logging with support for multiple backends
pub struct AuditService {
    backend: AuditBackend,
    retention_days: u32,
}

impl AuditService {
    /// Create a new audit service
    pub fn new(backend: AuditBackend, retention_days: u32) -> Self {
        Self {
            backend,
            retention_days,
        }
    }

    /// Create from configuration
    pub fn from_config(config: &AuditBackendConfig, retention_days: u32) -> Self {
        let backend = match config {
            AuditBackendConfig::File { log_dir } => AuditBackend::File {
                log_dir: PathBuf::from(log_dir),
            },
            AuditBackendConfig::Database { connection_string, table_name } => {
                AuditBackend::Database {
                    connection_string: connection_string.clone(),
                    table_name: table_name.clone(),
                }
            },
            AuditBackendConfig::Remote { endpoint, api_key } => {
                AuditBackend::Remote {
                    endpoint: endpoint.clone(),
                    api_key: api_key.clone(),
                }
            },
            AuditBackendConfig::Memory => AuditBackend::Memory,
        };

        Self::new(backend, retention_days)
    }

    /// Record an extraction audit
    pub async fn log_extraction(&self, audit: ExtractionAudit) -> PdfResult<()> {
        match &self.backend {
            AuditBackend::File { log_dir } => {
                self.log_to_file(log_dir, &audit).await
            },
            AuditBackend::Database { .. } => {
                self.log_to_database(&audit).await
            },
            AuditBackend::Remote { .. } => {
                self.log_to_remote(&audit).await
            },
            AuditBackend::Memory => {
                // Memory backend: just log (for testing)
                tracing::debug!("Audit logged to memory: {:?}", audit.id);
                Ok(())
            },
        }
    }

    /// Log to file backend
    async fn log_to_file(&self, log_dir: &Path, audit: &ExtractionAudit) -> PdfResult<()> {
        tokio::fs::create_dir_all(log_dir).await
            .map_err(|e| PdfModuleError::AuditError(format!("Failed to create log directory: {}", e)))?;

        let date = audit.created_at.format("%Y-%m-%d").to_string();
        let filename = format!("audit_{}.jsonl", date);
        let filepath = log_dir.join(filename);

        let line = serde_json::to_string(audit)
            .map_err(|e| PdfModuleError::AuditError(format!("Failed to serialize audit: {}", e)))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&filepath)
            .await
            .map_err(|e| PdfModuleError::AuditError(format!("Failed to open audit file: {}", e)))?;

        file.write_all(line.as_bytes()).await
            .map_err(|e| PdfModuleError::AuditError(format!("Failed to write audit: {}", e)))?;

        file.write_all(b"\n").await
            .map_err(|e| PdfModuleError::AuditError(format!("Failed to write newline: {}", e)))?;

        Ok(())
    }

    /// Log to database backend (placeholder)
    async fn log_to_database(&self, _audit: &ExtractionAudit) -> PdfResult<()> {
        // TODO: Implement database logging
        tracing::warn!("Database audit backend not yet implemented");
        Ok(())
    }

    /// Log to remote service backend (placeholder)
    async fn log_to_remote(&self, _audit: &ExtractionAudit) -> PdfResult<()> {
        // TODO: Implement remote API logging
        tracing::warn!("Remote audit backend not yet implemented");
        Ok(())
    }

    /// Query audit records
    pub async fn query_audits(
        &self,
        filters: AuditFilters,
    ) -> PdfResult<Vec<ExtractionAudit>> {
        match &self.backend {
            AuditBackend::File { log_dir } => {
                self.query_from_file(log_dir, filters).await
            },
            _ => {
                // TODO: Implement query for other backends
                Ok(vec![])
            },
        }
    }

    /// Query from file backend
    async fn query_from_file(
        &self,
        log_dir: &Path,
        filters: AuditFilters,
    ) -> PdfResult<Vec<ExtractionAudit>> {
        let mut audits = vec![];

        // If no date filters specified, query all files in directory
        if filters.start_date.is_none() {
            if log_dir.exists() {
                let mut entries = tokio::fs::read_dir(log_dir).await
                    .map_err(|e| PdfModuleError::AuditError(format!("Failed to read log directory: {}", e)))?;

                while let Some(entry) = entries.next_entry().await
                    .map_err(|e| PdfModuleError::AuditError(format!("Failed to read directory entry: {}", e)))? 
                {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                            if filename.starts_with("audit_") && filename.ends_with(".jsonl") {
                                let content = tokio::fs::read_to_string(&path).await
                                    .map_err(|e| PdfModuleError::AuditError(format!("Failed to read audit file: {}", e)))?;

                                for line in content.lines() {
                                    if let Ok(audit) = serde_json::from_str::<ExtractionAudit>(line) {
                                        if filters.matches(&audit) {
                                            audits.push(audit);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if let Some(start_date) = filters.start_date {
            let end_date = filters.end_date.unwrap_or_else(|| chrono::Utc::now().naive_utc().date());
            let mut current_date = start_date;

            while current_date <= end_date {
                let filename = format!("audit_{}.jsonl", current_date.format("%Y-%m-%d"));
                let filepath = log_dir.join(filename);

                if filepath.exists() {
                    let content = tokio::fs::read_to_string(&filepath).await
                        .map_err(|e| PdfModuleError::AuditError(format!("Failed to read audit file: {}", e)))?;

                    for line in content.lines() {
                        if let Ok(audit) = serde_json::from_str::<ExtractionAudit>(line) {
                            if filters.matches(&audit) {
                                audits.push(audit);
                            }
                        }
                    }
                }

                current_date = current_date.succ_opt().unwrap();
            }
        }

        Ok(audits)
    }

    /// Clean up old audit logs
    pub async fn cleanup_old_logs(&self) -> PdfResult<usize> {
        match &self.backend {
            AuditBackend::File { log_dir } => {
                self.cleanup_file_logs(log_dir).await
            },
            _ => {
                // TODO: Implement cleanup for other backends
                Ok(0)
            },
        }
    }

    /// Clean up old file logs
    async fn cleanup_file_logs(&self, log_dir: &Path) -> PdfResult<usize> {
        let cutoff_date = chrono::Utc::now().naive_utc().date() - chrono::Duration::days(self.retention_days as i64);
        let mut cleaned_count = 0;

        if !log_dir.exists() {
            return Ok(0);
        }

        let mut entries = tokio::fs::read_dir(log_dir).await
            .map_err(|e| PdfModuleError::AuditError(format!("Failed to read log directory: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| PdfModuleError::AuditError(format!("Failed to read directory entry: {}", e)))? 
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename.starts_with("audit_") && filename.ends_with(".jsonl") {
                        // Extract date from filename
                        let date_str = filename.strip_prefix("audit_")
                            .and_then(|s| s.strip_suffix(".jsonl"));

                        if let Some(date_str) = date_str {
                            if let Ok(file_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                                if file_date < cutoff_date {
                                    tokio::fs::remove_file(&path).await
                                        .map_err(|e| PdfModuleError::AuditError(format!("Failed to remove old log file: {}", e)))?;
                                    cleaned_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(cleaned_count)
    }
}

/// Audit query filters
#[derive(Debug, Clone, Default)]
pub struct AuditFilters {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub org_id: Option<String>,
    pub workflow_id: Option<String>,
    pub status: Option<ExtractionStatus>,
    pub file_name: Option<String>,
}

impl AuditFilters {
    /// Check if an audit record matches the filters
    pub fn matches(&self, audit: &ExtractionAudit) -> bool {
        if let Some(org_id) = &self.org_id {
            if audit.org_id.as_ref() != Some(org_id) {
                return false;
            }
        }

        if let Some(workflow_id) = &self.workflow_id {
            if audit.workflow_id.as_ref() != Some(workflow_id) {
                return false;
            }
        }

        if let Some(status) = &self.status {
            if &audit.status != status {
                return false;
            }
        }

        if let Some(file_name) = &self.file_name {
            if !audit.file_name.contains(file_name) {
                return false;
            }
        }

        true
    }

    /// Create a new filter builder
    pub fn builder() -> AuditFiltersBuilder {
        AuditFiltersBuilder::new()
    }
}

/// Audit filters builder
pub struct AuditFiltersBuilder {
    filters: AuditFilters,
}

impl AuditFiltersBuilder {
    pub fn new() -> Self {
        Self {
            filters: AuditFilters::default(),
        }
    }

    pub fn start_date(mut self, date: NaiveDate) -> Self {
        self.filters.start_date = Some(date);
        self
    }

    pub fn end_date(mut self, date: NaiveDate) -> Self {
        self.filters.end_date = Some(date);
        self
    }

    pub fn org_id(mut self, org_id: String) -> Self {
        self.filters.org_id = Some(org_id);
        self
    }

    pub fn workflow_id(mut self, workflow_id: String) -> Self {
        self.filters.workflow_id = Some(workflow_id);
        self
    }

    pub fn status(mut self, status: ExtractionStatus) -> Self {
        self.filters.status = Some(status);
        self
    }

    pub fn file_name(mut self, file_name: String) -> Self {
        self.filters.file_name = Some(file_name);
        self
    }

    pub fn build(self) -> AuditFilters {
        self.filters
    }
}

impl Default for AuditFiltersBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_audit_service_file_backend() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("audit");

        let service = AuditService::new(
            AuditBackend::File { log_dir: log_dir.clone() },
            30,
        );

        let audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        )
        .with_status(ExtractionStatus::Completed)
        .with_processing_time(1000)
        .with_extracted_text_length(5000);

        let result = service.log_extraction(audit.clone()).await;
        assert!(result.is_ok());

        // Give some time for file write to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Verify file was created
        let date = chrono::Utc::now().naive_utc().date().format("%Y-%m-%d").to_string();
        let filename = format!("audit_{}.jsonl", date);
        let filepath = log_dir.join(filename);
        assert!(filepath.exists());

        // Verify content
        let content = tokio::fs::read_to_string(&filepath).await.unwrap();
        assert!(content.contains("test.pdf"));
        assert!(content.contains("\"completed\"")); // JSON serialization uses lowercase
    }

    #[tokio::test]
    async fn test_audit_service_query() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("audit");

        let service = AuditService::new(
            AuditBackend::File { log_dir: log_dir.clone() },
            30,
        );

        // Log multiple audits
        let audit1 = ExtractionAudit::new(
            "test1.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        )
        .with_org_id("org-123".to_string())
        .with_status(ExtractionStatus::Completed)
        .with_processing_time(1000)
        .with_extracted_text_length(5000);

        let audit2 = ExtractionAudit::new(
            "test2.pdf".to_string(),
            "pdf".to_string(),
            2048.0,
            "pdfium".to_string(),
        )
        .with_status(ExtractionStatus::Failed)
        .with_error("File corrupted".to_string());

        service.log_extraction(audit1).await.unwrap();
        service.log_extraction(audit2).await.unwrap();

        // Give some time for file writes to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Query by status
        let filters = AuditFilters::builder()
            .status(ExtractionStatus::Completed)
            .build();

        let results = service.query_audits(filters).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_name, "test1.pdf");

        // Query by org_id
        let filters = AuditFilters::builder()
            .org_id("org-123".to_string())
            .build();

        let results = service.query_audits(filters).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].org_id, Some("org-123".to_string()));
    }

    #[tokio::test]
    async fn test_audit_service_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().join("audit");

        let service = AuditService::new(
            AuditBackend::File { log_dir: log_dir.clone() },
            1, // 1 day retention
        );

        // Create an old audit file (simulated)
        let old_date = chrono::Utc::now().naive_utc().date() - chrono::Duration::days(2);
        let old_filename = format!("audit_{}.jsonl", old_date.format("%Y-%m-%d"));
        let old_filepath = log_dir.join(old_filename);
        
        // Ensure directory exists before writing
        tokio::fs::create_dir_all(&log_dir).await.unwrap();
        tokio::fs::write(&old_filepath, "test").await.unwrap();

        // Create a recent audit file
        let audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        );
        service.log_extraction(audit).await.unwrap();

        // Give some time for file write to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Cleanup
        let cleaned = service.cleanup_old_logs().await.unwrap();
        assert_eq!(cleaned, 1);

        // Verify old file was removed
        assert!(!old_filepath.exists());

        // Verify recent file still exists
        let recent_date = chrono::Utc::now().naive_utc().date().format("%Y-%m-%d").to_string();
        let recent_filename = format!("audit_{}.jsonl", recent_date);
        let recent_filepath = log_dir.join(recent_filename);
        assert!(recent_filepath.exists());
    }

    #[tokio::test]
    async fn test_audit_service_memory_backend() {
        let service = AuditService::new(
            AuditBackend::Memory,
            30,
        );

        let audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        )
        .with_status(ExtractionStatus::Completed)
        .with_processing_time(1000)
        .with_extracted_text_length(5000);

        // Should succeed without error
        let result = service.log_extraction(audit).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_audit_filters_builder() {
        let filters = AuditFilters::builder()
            .org_id("org-123".to_string())
            .status(ExtractionStatus::Completed)
            .file_name("test".to_string())
            .build();

        assert_eq!(filters.org_id, Some("org-123".to_string()));
        assert_eq!(filters.status, Some(ExtractionStatus::Completed));
        assert_eq!(filters.file_name, Some("test".to_string()));
    }

    #[test]
    fn test_audit_filters_matching() {
        let audit = ExtractionAudit::new(
            "test.pdf".to_string(),
            "pdf".to_string(),
            1024.0,
            "lopdf".to_string(),
        )
        .with_org_id("org-123".to_string())
        .with_status(ExtractionStatus::Completed);

        let filters = AuditFilters::builder()
            .org_id("org-123".to_string())
            .status(ExtractionStatus::Completed)
            .build();

        assert!(filters.matches(&audit));

        let filters = AuditFilters::builder()
            .org_id("org-456".to_string())
            .build();

        assert!(!filters.matches(&audit));

        let filters = AuditFilters::builder()
            .file_name("test".to_string())
            .build();

        assert!(filters.matches(&audit));

        let filters = AuditFilters::builder()
            .file_name("other".to_string())
            .build();

        assert!(!filters.matches(&audit));
    }
}
