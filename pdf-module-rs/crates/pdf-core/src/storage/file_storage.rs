//! File storage abstraction
//! Defines the unified interface for file storage operations

use crate::dto::{FileMetadata, StorageFileInfo, StorageType};
use crate::error::{PdfModuleError, PdfResult};
use async_trait::async_trait;
use bytes::Bytes;

/// File storage abstraction trait
/// Defines unified interface for file storage operations across different backends
#[async_trait]
pub trait FileStorage: Send + Sync {
    /// Read file content
    async fn read(&self, path: &str) -> PdfResult<Bytes>;

    /// Write file content
    async fn write(&self, path: &str, data: &[u8]) -> PdfResult<()>;

    /// Check if file exists
    async fn exists(&self, path: &str) -> PdfResult<bool>;

    /// Delete file
    async fn delete(&self, path: &str) -> PdfResult<()>;

    /// List files in directory
    async fn list(&self, path: &str, recursive: bool) -> PdfResult<Vec<StorageFileInfo>>;

    /// Get file metadata
    async fn metadata(&self, path: &str) -> PdfResult<FileMetadata>;

    /// Get storage type
    fn storage_type(&self) -> StorageType;

    /// Check if storage is available
    async fn is_available(&self) -> bool {
        true
    }
}

/// File storage configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileStorageConfig {
    pub storage_type: StorageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local: Option<crate::dto::LocalStorageConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3: Option<crate::dto::S3StorageConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcs: Option<crate::dto::GCSStorageConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub azure: Option<crate::dto::AzureStorageConfig>,
}

impl FileStorageConfig {
    /// Create a new local storage configuration
    pub fn local(base_dir: String) -> Self {
        Self {
            storage_type: StorageType::Local,
            local: Some(crate::dto::LocalStorageConfig { base_dir }),
            s3: None,
            gcs: None,
            azure: None,
        }
    }

    /// Create a new S3 storage configuration
    pub fn s3(
        bucket: String,
        region: String,
        prefix: Option<String>,
        access_key: Option<String>,
        secret_key: Option<String>,
        endpoint: Option<String>,
    ) -> Self {
        Self {
            storage_type: StorageType::S3,
            local: None,
            s3: Some(crate::dto::S3StorageConfig {
                bucket,
                region,
                prefix,
                access_key,
                secret_key,
                endpoint,
            }),
            gcs: None,
            azure: None,
        }
    }

    /// Create a new GCS storage configuration
    pub fn gcs(bucket: String, credentials_path: String) -> Self {
        Self {
            storage_type: StorageType::Gcs,
            local: None,
            s3: None,
            gcs: Some(crate::dto::GCSStorageConfig {
                bucket,
                credentials_path,
            }),
            azure: None,
        }
    }

    /// Create a new Azure storage configuration
    pub fn azure(account: String, key: String, container: String) -> Self {
        Self {
            storage_type: StorageType::AzureBlob,
            local: None,
            s3: None,
            gcs: None,
            azure: Some(crate::dto::AzureStorageConfig {
                account,
                key,
                container,
            }),
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> PdfResult<()> {
        match self.storage_type {
            StorageType::Local
                if self.local.is_none() => {
                    return Err(PdfModuleError::ConfigError(
                        "Local storage config is required for local storage type".to_string(),
                    ));
                }
            StorageType::S3
                if self.s3.is_none() => {
                    return Err(PdfModuleError::ConfigError(
                        "S3 storage config is required for S3 storage type".to_string(),
                    ));
                }
            StorageType::Gcs
                if self.gcs.is_none() => {
                    return Err(PdfModuleError::ConfigError(
                        "GCS storage config is required for GCS storage type".to_string(),
                    ));
                }
            StorageType::AzureBlob
                if self.azure.is_none() => {
                    return Err(PdfModuleError::ConfigError(
                        "Azure storage config is required for Azure storage type".to_string(),
                    ));
                }
            _ => {
                // Other storage types may not require config
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_storage_config_local() {
        let config = FileStorageConfig::local("./data".to_string());
        assert_eq!(config.storage_type, StorageType::Local);
        assert!(config.local.is_some());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_file_storage_config_s3() {
        let config = FileStorageConfig::s3(
            "my-bucket".to_string(),
            "us-east-1".to_string(),
            Some("prefix".to_string()),
            Some("access-key".to_string()),
            Some("secret-key".to_string()),
            Some("https://s3.example.com".to_string()),
        );

        assert_eq!(config.storage_type, StorageType::S3);
        assert!(config.s3.is_some());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_file_storage_config_validation() {
        // Missing local config
        let config = FileStorageConfig {
            storage_type: StorageType::Local,
            local: None,
            s3: None,
            gcs: None,
            azure: None,
        };
        assert!(config.validate().is_err());

        // Missing S3 config
        let config = FileStorageConfig {
            storage_type: StorageType::S3,
            local: None,
            s3: None,
            gcs: None,
            azure: None,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_file_storage_config_serialization() {
        let config = FileStorageConfig::local("./data".to_string());

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"local\""));

        let deserialized: FileStorageConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.storage_type, StorageType::Local);
    }
}
