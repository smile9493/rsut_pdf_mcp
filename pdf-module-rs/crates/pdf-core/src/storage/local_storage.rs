//! Local file system storage implementation
//! Implements FileStorage trait for local filesystem

use crate::dto::{FileMetadata, StorageFileInfo, StorageType};
use crate::error::{PdfModuleError, PdfResult};
use crate::storage::file_storage::FileStorage;
use async_trait::async_trait;
use bytes::Bytes;
use std::path::PathBuf;
use tokio::fs;
use walkdir::WalkDir;
use chrono::{DateTime, Utc};

/// Local file system storage implementation
pub struct LocalFileStorage {
    base_dir: PathBuf,
}

impl LocalFileStorage {
    /// Create a new local file storage
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Get the full path for a relative path
    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = path.trim_start_matches('/');
        let path = path.trim_start_matches('\\');
        self.base_dir.join(path)
    }

    /// Sanitize a path to prevent directory traversal
    fn sanitize_path(&self, path: &str) -> PdfResult<PathBuf> {
        let path = path.trim_start_matches('/')
            .trim_start_matches('\\');

        // Check for path traversal attempts
        if path.contains("..") || path.contains("~") {
            return Err(PdfModuleError::StorageError(
                format!("Invalid path: {}", path),
            ));
        }

        Ok(self.base_dir.join(path))
    }
}

#[async_trait]
impl FileStorage for LocalFileStorage {
    async fn read(&self, path: &str) -> PdfResult<Bytes> {
        let full_path = self.sanitize_path(path)?;
        let data = fs::read(&full_path).await
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to read file: {}", e)))?;
        Ok(Bytes::from(data))
    }

    async fn write(&self, path: &str, data: &[u8]) -> PdfResult<()> {
        let full_path = self.sanitize_path(path)?;
        
        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| PdfModuleError::StorageError(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(&full_path, data).await
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to write file: {}", e)))?;
        Ok(())
    }

    async fn exists(&self, path: &str) -> PdfResult<bool> {
        let full_path = self.sanitize_path(path)?;
        Ok(full_path.exists())
    }

    async fn delete(&self, path: &str) -> PdfResult<()> {
        let full_path = self.sanitize_path(path)?;
        fs::remove_file(&full_path).await
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to delete file: {}", e)))?;
        Ok(())
    }

    async fn list(&self, path: &str, recursive: bool) -> PdfResult<Vec<StorageFileInfo>> {
        let full_path = self.sanitize_path(path)?;
        let mut files = vec![];

        if !full_path.exists() {
            return Err(PdfModuleError::StorageError(format!("Path does not exist: {}", path)));
        }

        if !full_path.is_dir() {
            return Err(PdfModuleError::StorageError(format!("Path is not a directory: {}", path)));
        }

        if recursive {
            for entry in WalkDir::new(&full_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let metadata = entry.metadata()
                    .map_err(|e| PdfModuleError::StorageError(format!("Failed to get metadata: {}", e)))?;
                files.push(StorageFileInfo {
                    path: entry.path().to_string_lossy().to_string(),
                    size: metadata.len(),
                });
            }
        } else {
            let mut entries = fs::read_dir(&full_path).await
                .map_err(|e| PdfModuleError::StorageError(format!("Failed to read directory: {}", e)))?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| PdfModuleError::StorageError(format!("Failed to read directory entry: {}", e)))? 
            {
                let path = entry.path();
                if path.is_file() {
                    let metadata = entry.metadata().await
                        .map_err(|e| PdfModuleError::StorageError(format!("Failed to get metadata: {}", e)))?;
                    files.push(StorageFileInfo {
                        path: path.to_string_lossy().to_string(),
                        size: metadata.len(),
                    });
                }
            }
        }

        Ok(files)
    }

    async fn metadata(&self, path: &str) -> PdfResult<FileMetadata> {
        let full_path = self.sanitize_path(path)?;
        let metadata = fs::metadata(&full_path).await
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to get metadata: {}", e)))?;
        
        let modified: DateTime<Utc> = metadata.modified()
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to get modified time: {}", e)))?
            .into();

        // TODO: Implement content type detection using infer crate
        let content_type: Option<String> = None;

        Ok(FileMetadata {
            path: path.to_string(),
            size: metadata.len(),
            modified: modified.to_rfc3339(),
            content_type,
        })
    }

    fn storage_type(&self) -> StorageType {
        StorageType::Local
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_file_storage_read_write() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalFileStorage::new(temp_dir.path().to_path_buf());

        // Write a file
        let data = b"Hello, World!";
        storage.write("test.txt", data).await.unwrap();

        // Read the file
        let read_data = storage.read("test.txt").await.unwrap();
        assert_eq!(read_data.as_ref(), data);

        // Check file exists
        assert!(storage.exists("test.txt").await.unwrap());

        // Get metadata
        let metadata = storage.metadata("test.txt").await.unwrap();
        assert_eq!(metadata.path, "test.txt");
        assert_eq!(metadata.size, 13);
    }

    #[tokio::test]
    async fn test_local_file_storage_delete() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalFileStorage::new(temp_dir.path().to_path_buf());

        // Write and then delete a file
        storage.write("test.txt", b"test").await.unwrap();
        assert!(storage.exists("test.txt").await.unwrap());

        storage.delete("test.txt").await.unwrap();
        assert!(!storage.exists("test.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_local_file_storage_list() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalFileStorage::new(temp_dir.path().to_path_buf());

        // Create some files
        storage.write("file1.txt", b"test1").await.unwrap();
        storage.write("file2.txt", b"test2").await.unwrap();

        // List files (non-recursive)
        let files = storage.list(".", false).await.unwrap();
        assert_eq!(files.len(), 2);
        
        let file_names: Vec<&str> = files.iter().map(|f| f.path.rsplit('/').next().unwrap_or("")).collect();
        assert!(file_names.contains(&"file1.txt"));
        assert!(file_names.contains(&"file2.txt"));
    }

    #[tokio::test]
    async fn test_local_file_storage_list_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalFileStorage::new(temp_dir.path().to_path_buf());

        // Create files in subdirectory
        fs::create_dir_all(temp_dir.path().join("subdir")).await.unwrap();
        storage.write("subdir/file1.txt", b"test1").await.unwrap();
        storage.write("file2.txt", b"test2").await.unwrap();

        // List files recursively
        let files = storage.list(".", true).await.unwrap();
        assert_eq!(files.len(), 2);
        
        let file_names: Vec<&str> = files.iter().map(|f| f.path.rsplit('/').next().unwrap_or("")).collect();
        assert!(file_names.contains(&"file1.txt"));
        assert!(file_names.contains(&"file2.txt"));
    }

    #[tokio::test]
    async fn test_local_file_storage_path_sanitization() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalFileStorage::new(temp_dir.path().to_path_buf());

        // Test path traversal prevention
        let result = storage.read("../etc/passwd").await;
        assert!(result.is_err());

        // Test tilde path prevention
        let result = storage.read("~/config").await;
        assert!(result.is_err());

        // Test normal path
        storage.write("test.txt", b"test").await.unwrap();
        let result = storage.read("test.txt").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_local_file_storage_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalFileStorage::new(temp_dir.path().to_path_buf());

        // Create a file
        storage.write("test.txt", b"Hello, World!").await.unwrap();

        // Get metadata
        let metadata = storage.metadata("test.txt").await.unwrap();
        assert_eq!(metadata.path, "test.txt");
        assert_eq!(metadata.size, 13);
        assert!(metadata.modified.len() > 0); // Should have a timestamp
    }

    #[tokio::test]
    async fn test_local_file_storage_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalFileStorage::new(temp_dir.path().to_path_buf());

        // Try to read nonexistent file
        let result = storage.read("nonexistent.txt").await;
        assert!(result.is_err());

        // Check exists
        assert!(!storage.exists("nonexistent.txt").await.unwrap());
    }

    #[test]
    fn test_local_file_storage_type() {
        let storage = LocalFileStorage::new(PathBuf::from("/tmp"));
        assert_eq!(storage.storage_type(), StorageType::Local);
    }

    #[tokio::test]
    async fn test_local_file_storage_directory_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalFileStorage::new(temp_dir.path().to_path_buf());

        // Write file in subdirectory (should create directory)
        storage.write("subdir/test.txt", b"test").await.unwrap();
        assert!(storage.exists("subdir/test.txt").await.unwrap());

        // List subdirectory
        let files = storage.list("subdir", false).await.unwrap();
        assert_eq!(files.len(), 1);
    }
}
