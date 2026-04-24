//! File storage factory
//! Provides factory methods to create different file storage implementations

use crate::dto::{
    AzureStorageConfig, GCSStorageConfig, LocalStorageConfig, S3StorageConfig, StorageType,
};
use crate::error::{PdfModuleError, PdfResult};
use crate::storage::file_storage::{FileStorage, FileStorageConfig};
use crate::storage::local_storage::LocalFileStorage;
use std::path::PathBuf;
use std::sync::Arc;

/// File storage factory
/// Provides methods to create different file storage implementations
pub struct FileStorageFactory;

impl FileStorageFactory {
    /// Create a file storage from configuration
    pub fn from_config(config: FileStorageConfig) -> PdfResult<Arc<dyn FileStorage>> {
        config.validate()?;

        match config.storage_type {
            StorageType::Local => {
                let local_config = config.local.ok_or_else(|| {
                    PdfModuleError::ConfigError("Local storage config is required".to_string())
                })?;
                Self::create_local(&local_config)
            }
            StorageType::S3 => {
                let s3_config = config.s3.ok_or_else(|| {
                    PdfModuleError::ConfigError("S3 storage config is required".to_string())
                })?;
                Self::create_s3(&s3_config)
            }
            StorageType::Gcs => {
                let gcs_config = config.gcs.ok_or_else(|| {
                    PdfModuleError::ConfigError("GCS storage config is required".to_string())
                })?;
                Self::create_gcs(&gcs_config)
            }
            StorageType::AzureBlob => {
                let azure_config = config.azure.ok_or_else(|| {
                    PdfModuleError::ConfigError("Azure storage config is required".to_string())
                })?;
                Self::create_azure(&azure_config)
            }
            _ => Err(PdfModuleError::ConfigError(format!(
                "Unsupported storage type: {:?}",
                config.storage_type
            ))),
        }
    }

    /// Create a local file storage
    pub fn create_local(config: &LocalStorageConfig) -> PdfResult<Arc<dyn FileStorage>> {
        let base_dir = PathBuf::from(&config.base_dir);

        // Create base directory if it doesn't exist
        if !base_dir.exists() {
            std::fs::create_dir_all(&base_dir).map_err(|e| {
                PdfModuleError::StorageError(format!("Failed to create base directory: {}", e))
            })?;
        }

        Ok(Arc::new(LocalFileStorage::new(base_dir)))
    }

    /// Create an S3 file storage
    pub fn create_s3(_config: &S3StorageConfig) -> PdfResult<Arc<dyn FileStorage>> {
        // Placeholder implementation
        // Would use aws-sdk-s3 and aws-config crates
        Err(PdfModuleError::StorageError(
            "S3 storage is not yet implemented".to_string(),
        ))
    }

    /// Create a GCS file storage
    pub fn create_gcs(_config: &GCSStorageConfig) -> PdfResult<Arc<dyn FileStorage>> {
        // Placeholder implementation
        // Would use google-cloud-storage crate
        Err(PdfModuleError::StorageError(
            "GCS storage is not yet implemented".to_string(),
        ))
    }

    /// Create an Azure Blob storage
    pub fn create_azure(_config: &AzureStorageConfig) -> PdfResult<Arc<dyn FileStorage>> {
        // Placeholder implementation
        // Would use azure_storage_blobs crate
        Err(PdfModuleError::StorageError(
            "Azure Blob storage is not yet implemented".to_string(),
        ))
    }

    /// Create a file storage from environment variables
    pub fn from_env() -> PdfResult<Arc<dyn FileStorage>> {
        use std::env;

        let storage_type = env::var("STORAGE_TYPE")
            .unwrap_or_else(|_| "local".to_string())
            .to_lowercase();

        match storage_type.as_str() {
            "local" => {
                let base_dir =
                    env::var("STORAGE_LOCAL_DIR").unwrap_or_else(|_| "./data".to_string());
                let config = LocalStorageConfig { base_dir };
                Self::create_local(&config)
            }
            "s3" => {
                let bucket = env::var("STORAGE_S3_BUCKET").map_err(|_| {
                    PdfModuleError::ConfigError("STORAGE_S3_BUCKET is required".to_string())
                })?;
                let region =
                    env::var("STORAGE_S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
                let prefix = env::var("STORAGE_S3_PREFIX").ok();
                let access_key = env::var("STORAGE_S3_ACCESS_KEY").ok();
                let secret_key = env::var("STORAGE_S3_SECRET_KEY").ok();
                let endpoint = env::var("STORAGE_S3_ENDPOINT").ok();

                let config = S3StorageConfig {
                    bucket,
                    region,
                    prefix,
                    access_key,
                    secret_key,
                    endpoint,
                };

                Self::create_s3(&config)
            }
            "gcs" => {
                let bucket = env::var("STORAGE_GCS_BUCKET").map_err(|_| {
                    PdfModuleError::ConfigError("STORAGE_GCS_BUCKET is required".to_string())
                })?;
                let credentials_path = env::var("STORAGE_GCS_CREDENTIALS_PATH").map_err(|_| {
                    PdfModuleError::ConfigError(
                        "STORAGE_GCS_CREDENTIALS_PATH is required".to_string(),
                    )
                })?;

                let config = GCSStorageConfig {
                    bucket,
                    credentials_path,
                };

                Self::create_gcs(&config)
            }
            "azure" => {
                let account = env::var("STORAGE_AZURE_ACCOUNT").map_err(|_| {
                    PdfModuleError::ConfigError("STORAGE_AZURE_ACCOUNT is required".to_string())
                })?;
                let key = env::var("STORAGE_AZURE_KEY").map_err(|_| {
                    PdfModuleError::ConfigError("STORAGE_AZURE_KEY is required".to_string())
                })?;
                let container = env::var("STORAGE_AZURE_CONTAINER").map_err(|_| {
                    PdfModuleError::ConfigError("STORAGE_AZURE_CONTAINER is required".to_string())
                })?;

                let config = AzureStorageConfig {
                    account,
                    key,
                    container,
                };

                Self::create_azure(&config)
            }
            _ => Err(PdfModuleError::ConfigError(format!(
                "Unsupported storage type: {}",
                storage_type
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_local_storage() {
        let temp_dir = TempDir::new().unwrap();
        let config = LocalStorageConfig {
            base_dir: temp_dir.path().to_string_lossy().to_string(),
        };

        let storage = FileStorageFactory::create_local(&config);
        assert!(storage.is_ok());
        assert_eq!(storage.unwrap().storage_type(), StorageType::Local);
    }

    #[test]
    fn test_create_local_storage_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path().join("nested").join("dir");
        let config = LocalStorageConfig {
            base_dir: base_dir.to_string_lossy().to_string(),
        };

        assert!(!base_dir.exists());

        let storage = FileStorageFactory::create_local(&config);
        assert!(storage.is_ok());
        assert!(base_dir.exists());
    }

    #[test]
    fn test_create_storage_from_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = FileStorageConfig::local(temp_dir.path().to_string_lossy().to_string());

        let storage = FileStorageFactory::from_config(config);
        assert!(storage.is_ok());
        assert_eq!(storage.unwrap().storage_type(), StorageType::Local);
    }

    #[test]
    fn test_create_storage_from_config_validation() {
        // Missing local config
        let config = FileStorageConfig {
            storage_type: StorageType::Local,
            local: None,
            s3: None,
            gcs: None,
            azure: None,
        };

        let storage = FileStorageFactory::from_config(config);
        assert!(storage.is_err());
    }

    #[test]
    fn test_s3_storage_not_implemented() {
        let config = S3StorageConfig {
            bucket: "test-bucket".to_string(),
            region: "us-east-1".to_string(),
            prefix: None,
            access_key: None,
            secret_key: None,
            endpoint: None,
        };

        let storage = FileStorageFactory::create_s3(&config);
        assert!(storage.is_err());
        if let Err(PdfModuleError::StorageError(_)) = storage {
            // Expected error
        } else {
            panic!("Expected StorageError");
        }
    }

    #[test]
    fn test_gcs_storage_not_implemented() {
        let config = GCSStorageConfig {
            bucket: "test-bucket".to_string(),
            credentials_path: "/path/to/credentials.json".to_string(),
        };

        let storage = FileStorageFactory::create_gcs(&config);
        assert!(storage.is_err());
        if let Err(PdfModuleError::StorageError(_)) = storage {
            // Expected error
        } else {
            panic!("Expected StorageError");
        }
    }

    #[test]
    fn test_azure_storage_not_implemented() {
        let config = AzureStorageConfig {
            account: "testaccount".to_string(),
            key: "testkey".to_string(),
            container: "testcontainer".to_string(),
        };

        let storage = FileStorageFactory::create_azure(&config);
        assert!(storage.is_err());
        if let Err(PdfModuleError::StorageError(_)) = storage {
            // Expected error
        } else {
            panic!("Expected StorageError");
        }
    }
}
