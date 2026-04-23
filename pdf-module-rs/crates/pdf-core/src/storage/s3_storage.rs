//! S3 file storage implementation
//! Implements FileStorage trait for AWS S3

#[cfg(feature = "s3")]
use crate::dto::{FileMetadata, StorageFileInfo, StorageType};
#[cfg(feature = "s3")]
use crate::error::{PdfModuleError, PdfResult};
#[cfg(feature = "s3")]
use crate::storage::file_storage::FileStorage;
#[cfg(feature = "s3")]
use async_trait::async_trait;
#[cfg(feature = "s3")]
use bytes::Bytes;
#[cfg(feature = "s3")]
use aws_sdk_s3::{Client, types::ByteStream};
#[cfg(feature = "s3")]
use chrono::Utc;

#[cfg(feature = "s3")]
/// S3 file storage implementation
pub struct S3FileStorage {
    client: Client,
    bucket: String,
    prefix: Option<String>,
}

#[cfg(feature = "s3")]
impl S3FileStorage {
    /// Create a new S3 file storage
    pub fn new(client: Client, bucket: String, prefix: Option<String>) -> Self {
        Self {
            client,
            bucket,
            prefix,
        }
    }

    /// Get the full S3 key for a path
    fn resolve_key(&self, path: &str) -> String {
        let path = path.trim_start_matches('/')
            .trim_start_matches('\\');

        match &self.prefix {
            Some(prefix) => {
                let prefix = prefix.trim_end_matches('/');
                let path = path.trim_start_matches('/');
                format!("{}/{}", prefix, path)
            },
            None => path.to_string(),
        }
    }

    /// Get the prefix for listing operations
    fn resolve_prefix(&self, path: &str) -> String {
        let path = path.trim_start_matches('/')
            .trim_start_matches('\\');

        let key = self.resolve_key(path);
        format!("{}/", key.trim_end_matches('/'))
    }
}

#[cfg(feature = "s3")]
#[async_trait]
impl FileStorage for S3FileStorage {
    async fn read(&self, path: &str) -> PdfResult<Bytes> {
        let key = self.resolve_key(path);
        
        let resp = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| PdfModuleError::StorageError(format!("S3 get_object failed: {}", e)))?;

        let data = resp.body
            .collect()
            .await
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to read S3 response: {}", e)))?
            .into_bytes();

        Ok(data)
    }

    async fn write(&self, path: &str, data: &[u8]) -> PdfResult<()> {
        let key = self.resolve_key(path);
        
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(Bytes::from(data.to_vec())))
            .send()
            .await
            .map_err(|e| PdfModuleError::StorageError(format!("S3 put_object failed: {}", e)))?;

        Ok(())
    }

    async fn exists(&self, path: &str) -> PdfResult<bool> {
        let key = self.resolve_key(path);
        
        match self.client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                if let Some(service_err) = e.as_service_error() {
                    if matches!(
                        service_err.kind(),
                        aws_sdk_s3::error::HeadObjectErrorKind::NotFound
                    ) {
                        return Ok(false);
                    }
                }
                Err(PdfModuleError::StorageError(format!("S3 head_object failed: {}", e)))
            },
        }
    }

    async fn delete(&self, path: &str) -> PdfResult<()> {
        let key = self.resolve_key(path);
        
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| PdfModuleError::StorageError(format!("S3 delete_object failed: {}", e)))?;

        Ok(())
    }

    async fn list(&self, path: &str, recursive: bool) -> PdfResult<Vec<StorageFileInfo>> {
        let prefix = self.resolve_prefix(path);
        let mut files = vec![];

        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self.client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&prefix);

            if !recursive {
                request = request.delimiter("/");
            }

            if let Some(token) = continuation_token {
                request = request.continuation_token(&token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| PdfModuleError::StorageError(format!("S3 list_objects_v2 failed: {}", e)))?;

            if let Some(contents) = response.contents {
                for obj in contents {
                    if let Some(key) = obj.key() {
                        // Skip directory markers
                        if key.ends_with('/') {
                            continue;
                        }

                        if let Some(size) = obj.size() {
                            files.push(StorageFileInfo {
                                path: key.clone(),
                                size: size as u64,
                            });
                        }
                    }
                }
            }

            continuation_token = response.next_continuation_token().map(String::from);
            if continuation_token.is_none() {
                break;
            }
        }

        Ok(files)
    }

    async fn metadata(&self, path: &str) -> PdfResult<FileMetadata> {
        let key = self.resolve_key(path);
        
        let resp = self.client
            .head_object()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| PdfModuleError::StorageError(format!("S3 head_object failed: {}", e)))?;

        let size = resp.content_length()
            .map(|s| s as u64)
            .unwrap_or(0);

        let modified = resp.last_modified()
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|_| Utc::now().to_rfc3339());

        let content_type = resp.content_type().map(String::from);

        Ok(FileMetadata {
            path: path.to_string(),
            size,
            modified,
            content_type,
        })
    }

    fn storage_type(&self) -> StorageType {
        StorageType::S3
    }
}

#[cfg(all(test, feature = "s3"))]
mod tests {
    use super::*;

    #[test]
    fn test_s3_file_storage_key_resolution() {
        let storage = S3FileStorage {
            client: Client::from_conf(aws_sdk_s3::config::Builder::new().build()),
            bucket: "test-bucket".to_string(),
            prefix: None,
        };

        assert_eq!(storage.resolve_key("file.txt"), "file.txt");
        assert_eq!(storage.resolve_key("/file.txt"), "file.txt");
    }

    #[test]
    fn test_s3_file_storage_key_resolution_with_prefix() {
        let storage = S3FileStorage {
            client: Client::from_conf(aws_sdk_s3::config::Builder::new().build()),
            bucket: "test-bucket".to_string(),
            prefix: Some("prefix".to_string()),
        };

        assert_eq!(storage.resolve_key("file.txt"), "prefix/file.txt");
        assert_eq!(storage.resolve_key("/file.txt"), "prefix/file.txt");
        assert_eq!(storage.resolve_key("/dir/file.txt"), "prefix/dir/file.txt");
    }

    #[test]
    fn test_s3_file_storage_prefix_resolution() {
        let storage = S3FileStorage {
            client: Client::from_conf(aws_sdk_s3::config::Builder::new().build()),
            bucket: "test-bucket".to_string(),
            prefix: Some("prefix".to_string()),
        };

        assert_eq!(storage.resolve_prefix("dir"), "prefix/dir/");
        assert_eq!(storage.resolve_prefix("/dir"), "prefix/dir/");
        assert_eq!(storage.resolve_prefix(""), "prefix/");
    }

    #[test]
    fn test_s3_file_storage_type() {
        let storage = S3FileStorage {
            client: Client::from_conf(aws_sdk_s3::config::Builder::new().build()),
            bucket: "test-bucket".to_string(),
            prefix: None,
        };

        assert_eq!(storage.storage_type(), StorageType::S3);
    }
}
