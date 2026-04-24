//! Extraction cache using moka
//! Corresponds to Python: cache.py

use crate::error::PdfResult;
use moka::sync::Cache;
use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

/// Cache entry with serialized result
#[derive(Clone)]
struct CacheEntry {
    result: String,
    #[allow(dead_code)]
    timestamp: Instant,
}

/// File metadata for fast cache key generation
#[derive(Debug, Clone)]
struct FileMetadata {
    modified: SystemTime,
    size: u64,
}

/// Extraction result cache
/// Corresponds to Python: ExtractionCache
/// Uses moka for concurrent-safe caching with TTL support
pub struct ExtractionCache {
    cache: Cache<String, CacheEntry>,
    max_size: u64,
    hits: AtomicU64,
    misses: AtomicU64,
    /// Threshold for using full hash (in bytes). Files larger than this use partial hash.
    full_hash_threshold: u64,
}

impl ExtractionCache {
    /// Create a new cache with max size and TTL in seconds
    pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_size as u64)
            .time_to_idle(Duration::from_secs(ttl_seconds))
            .build();

        Self {
            cache,
            max_size: max_size as u64,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            // Use full hash for files < 10MB, partial hash for larger files
            full_hash_threshold: 10 * 1024 * 1024,
        }
    }

    /// Get file metadata (modified time and size) for fast cache key
    fn get_file_metadata(file_path: &Path) -> PdfResult<FileMetadata> {
        let metadata = std::fs::metadata(file_path)?;
        Ok(FileMetadata {
            modified: metadata.modified()?,
            size: metadata.len(),
        })
    }

    /// Compute partial SHA256 hash (first 1MB + last 1MB + size)
    /// Much faster for large files while still providing good uniqueness
    fn partial_file_hash(file_path: &Path, size: u64) -> PdfResult<String> {
        let mut hasher = Sha256::new();
        let mut file = std::fs::File::open(file_path)?;

        // Hash file size first
        hasher.update(size.to_le_bytes());

        // Hash first 1MB
        let first_chunk_size = 1024 * 1024;
        let mut buf = vec![0u8; first_chunk_size];
        let n = file.read(&mut buf)?;
        hasher.update(&buf[..n]);

        // Hash last 1MB if file is large enough
        if size > first_chunk_size as u64 * 2 {
            let last_chunk_size = 1024 * 1024;
            file.seek(SeekFrom::End(-(last_chunk_size as i64)))?;
            let mut buf = vec![0u8; last_chunk_size];
            let n = file.read(&mut buf)?;
            hasher.update(&buf[..n]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Compute SHA256 hash of a file
    /// Corresponds to Python: ExtractionCache.file_hash()
    /// Optimized: uses partial hash for large files
    pub fn file_hash(file_path: &Path) -> PdfResult<String> {
        let metadata = std::fs::metadata(file_path)?;
        let size = metadata.len();

        // For small files, use full hash
        if size <= 10 * 1024 * 1024 {
            let mut hasher = Sha256::new();
            let mut file = std::fs::File::open(file_path)?;
            let mut buf = [0u8; 8192];

            loop {
                let n = file.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }

            Ok(format!("{:x}", hasher.finalize()))
        } else {
            // For large files, use partial hash
            Self::partial_file_hash(file_path, size)
        }
    }

    /// Generate cache key from file path, adapter name, and kwargs
    /// Optimized: uses file metadata (mtime + size) as fast key component
    fn make_key(&self, file_path: &Path, adapter: &str, kwargs: &str) -> PdfResult<String> {
        let metadata = Self::get_file_metadata(file_path)?;

        // Use metadata as part of cache key for fast lookup
        // This avoids computing full hash for unchanged files
        let mtime = metadata
            .modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs();

        // For small files, use metadata-based key (very fast)
        // For large files, combine metadata with partial hash
        if metadata.size <= self.full_hash_threshold {
            // Fast path: metadata only
            Ok(format!(
                "{}|{}|{}|{}|{}",
                file_path.display(),
                mtime,
                metadata.size,
                adapter,
                kwargs
            ))
        } else {
            // Large file: metadata + partial hash for better uniqueness
            let hash = Self::partial_file_hash(file_path, metadata.size)?;
            Ok(format!("{}|{}|{}|{}", hash, adapter, kwargs, metadata.size))
        }
    }

    /// Get cached result
    /// Corresponds to Python: ExtractionCache.get()
    pub fn get<T: serde::de::DeserializeOwned>(
        &self,
        file_path: &Path,
        adapter: &str,
    ) -> PdfResult<Option<T>> {
        let key = self.make_key(file_path, adapter, "")?;

        if let Some(entry) = self.cache.get(&key) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            match serde_json::from_str(&entry.result) {
                Ok(result) => return Ok(Some(result)),
                Err(_) => {
                    // Deserialization failed, treat as miss
                    self.misses.fetch_add(1, Ordering::Relaxed);
                }
            }
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }

        Ok(None)
    }

    /// Set cached result
    /// Corresponds to Python: ExtractionCache.set()
    pub fn set<T: serde::Serialize>(
        &self,
        file_path: &Path,
        adapter: &str,
        result: &T,
    ) -> PdfResult<()> {
        let key = self.make_key(file_path, adapter, "")?;
        let json = serde_json::to_string(result)
            .map_err(|e| crate::error::PdfModuleError::Extraction(e.to_string()))?;

        self.cache.insert(
            key,
            CacheEntry {
                result: json,
                timestamp: Instant::now(),
            },
        );

        Ok(())
    }

    /// Get cache statistics
    /// Corresponds to Python: ExtractionCache.stats()
    pub fn stats(&self) -> serde_json::Value {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };

        serde_json::json!({
            "size": self.cache.entry_count(),
            "max_size": self.max_size,
            "hits": hits,
            "misses": misses,
            "hit_rate": hit_rate,
        })
    }

    /// Invalidate all entries
    pub fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_hash() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "test content").unwrap();

        let hash = ExtractionCache::file_hash(temp_file.path()).unwrap();
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex chars
    }

    #[test]
    fn test_cache_set_get() {
        let cache = ExtractionCache::new(100, 3600);
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "test content").unwrap();

        let data = serde_json::json!({"text": "hello"});
        cache.set(temp_file.path(), "lopdf", &data).unwrap();

        let result: Option<serde_json::Value> = cache.get(temp_file.path(), "lopdf").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap()["text"], "hello");
    }

    #[test]
    fn test_cache_miss() {
        let cache = ExtractionCache::new(100, 3600);
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "test content").unwrap();

        let result: Option<serde_json::Value> = cache.get(temp_file.path(), "lopdf").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_stats() {
        let cache = ExtractionCache::new(100, 3600);
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "test content").unwrap();

        // Miss
        let _: Option<serde_json::Value> = cache.get(temp_file.path(), "lopdf").unwrap();

        // Set and hit
        let data = serde_json::json!({"text": "hello"});
        cache.set(temp_file.path(), "lopdf", &data).unwrap();
        let _: Option<serde_json::Value> = cache.get(temp_file.path(), "lopdf").unwrap();

        let stats = cache.stats();
        assert_eq!(stats["hits"], 1);
        assert_eq!(stats["misses"], 1);
    }
}
