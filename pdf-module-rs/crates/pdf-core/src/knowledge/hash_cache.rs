//! Hash cache for incremental compilation.
//!
//! Tracks file hashes to detect which PDFs have changed since last compilation.
//! Stored as a simple JSON file at `<knowledge_base>/.hash_cache`.
//! Can be fully rebuilt by scanning all `raw/` entries and their source PDFs.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::error::{PdfModuleError, PdfResult};

/// A single entry in the hash cache, mapping a source PDF path to its hash
/// and the list of wiki entries compiled from it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// SHA-256 hex digest of the source PDF file.
    pub source_hash: String,
    /// Relative paths of wiki entries compiled from this source.
    pub compiled_entries: Vec<String>,
    /// Timestamp of last compilation (RFC 3339).
    pub last_compiled: String,
}

/// Merkle-style hash cache for incremental compilation.
///
/// Layout on disk:
/// ```json
/// {
///   "raw/paper.pdf": {
///     "source_hash": "abc123...",
///     "compiled_entries": ["wiki/it/concept1.md", "wiki/it/concept2.md"],
///     "last_compiled": "2026-05-04T10:00:00Z"
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashCache {
    #[serde(flatten)]
    entries: HashMap<String, CacheEntry>,
    #[serde(skip)]
    cache_path: PathBuf,
}

impl HashCache {
    /// Load the hash cache from disk, or create an empty one if not found.
    pub fn load_or_create(knowledge_base: &Path) -> PdfResult<Self> {
        let cache_path = knowledge_base.join(".hash_cache");
        if cache_path.exists() {
            let content = fs::read_to_string(&cache_path).map_err(|e| {
                PdfModuleError::StorageError(format!("Failed to read hash cache: {}", e))
            })?;
            let mut cache: Self = serde_json::from_str(&content).map_err(|e| {
                PdfModuleError::StorageError(format!("Failed to parse hash cache: {}", e))
            })?;
            cache.cache_path = cache_path;
            info!(entries = cache.entries.len(), "Loaded hash cache");
            Ok(cache)
        } else {
            info!("No hash cache found, creating empty cache");
            Ok(Self {
                entries: HashMap::new(),
                cache_path,
            })
        }
    }

    /// Persist the cache to disk.
    pub fn save(&self) -> PdfResult<()> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            PdfModuleError::StorageError(format!("Failed to serialize hash cache: {}", e))
        })?;
        if let Some(parent) = self.cache_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                PdfModuleError::StorageError(format!("Failed to create cache dir: {}", e))
            })?;
        }
        fs::write(&self.cache_path, json).map_err(|e| {
            PdfModuleError::StorageError(format!("Failed to write hash cache: {}", e))
        })?;
        debug!(path = ?self.cache_path, "Hash cache saved");
        Ok(())
    }

    /// Compute SHA-256 of a file.
    pub fn hash_file(path: &Path) -> PdfResult<String> {
        let bytes = fs::read(path).map_err(|e| {
            PdfModuleError::StorageError(format!("Failed to read file for hashing: {}", e))
        })?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Compute SHA-256 of a byte slice.
    pub fn hash_bytes(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Check if a source PDF needs recompilation.
    /// Returns `true` if:
    /// - The source is not in the cache (new file), or
    /// - The source hash has changed (file modified), or
    /// - No compiled entries exist (compilation incomplete).
    pub fn needs_compile(&self, source_path: &Path) -> PdfResult<bool> {
        let key = source_path.to_string_lossy().to_string();
        let current_hash = Self::hash_file(source_path)?;

        match self.entries.get(&key) {
            None => {
                debug!(source = %key, "New source, needs compile");
                Ok(true)
            }
            Some(entry) => {
                let changed = entry.source_hash != current_hash;
                let empty = entry.compiled_entries.is_empty();
                if changed {
                    debug!(source = %key, "Source hash changed, needs recompile");
                } else if empty {
                    debug!(source = %key, "No compiled entries, needs compile");
                }
                Ok(changed || empty)
            }
        }
    }

    /// Get the list of sources that need compilation.
    pub fn get_pending_sources(&self, raw_dir: &Path) -> PdfResult<Vec<PathBuf>> {
        let mut pending = Vec::new();
        if !raw_dir.exists() {
            return Ok(pending);
        }
        for entry in fs::read_dir(raw_dir)
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to read raw dir: {}", e)))?
        {
            let entry = entry.map_err(|e| {
                PdfModuleError::StorageError(format!("Failed to read dir entry: {}", e))
            })?;
            let path = entry.path();
            if path.extension().map(|e| e == "pdf").unwrap_or(false) {
                if self.needs_compile(&path)? {
                    pending.push(path);
                }
            }
        }
        Ok(pending)
    }

    /// Record a successful compilation.
    pub fn record_compile(
        &mut self,
        source_path: &Path,
        compiled_entries: Vec<String>,
    ) -> PdfResult<()> {
        let key = source_path.to_string_lossy().to_string();
        let hash = Self::hash_file(source_path)?;
        let now = chrono::Utc::now().to_rfc3339();

        self.entries.insert(
            key,
            CacheEntry {
                source_hash: hash,
                compiled_entries,
                last_compiled: now,
            },
        );
        Ok(())
    }

    /// Get the cache entry for a specific source.
    pub fn get(&self, source_path: &Path) -> Option<&CacheEntry> {
        let key = source_path.to_string_lossy().to_string();
        self.entries.get(&key)
    }

    /// Remove a source from the cache (e.g., when the PDF is deleted).
    pub fn remove(&mut self, source_path: &Path) -> Option<CacheEntry> {
        let key = source_path.to_string_lossy().to_string();
        self.entries.remove(&key)
    }

    /// Return the number of cached sources.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all entries.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &CacheEntry)> {
        self.entries.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_hash_cache_roundtrip() {
        let dir = TempDir::new().unwrap();
        let cache_path = dir.path().join(".hash_cache");

        let mut cache = HashCache {
            entries: HashMap::new(),
            cache_path: cache_path.clone(),
        };

        // Create a fake PDF
        let pdf_path = dir.path().join("test.pdf");
        let mut f = fs::File::create(&pdf_path).unwrap();
        f.write_all(b"%PDF-1.4 fake content").unwrap();

        cache
            .record_compile(&pdf_path, vec!["wiki/it/concept.md".into()])
            .unwrap();
        cache.save().unwrap();

        // Reload and verify
        let loaded = HashCache::load_or_create(dir.path()).unwrap();
        assert_eq!(loaded.len(), 1);
        let entry = loaded.get(&pdf_path).unwrap();
        assert_eq!(entry.compiled_entries, vec!["wiki/it/concept.md"]);
    }

    #[test]
    fn test_needs_compile() {
        let dir = TempDir::new().unwrap();
        let mut cache = HashCache {
            entries: HashMap::new(),
            cache_path: dir.path().join(".hash_cache"),
        };

        let pdf_path = dir.path().join("test.pdf");
        let mut f = fs::File::create(&pdf_path).unwrap();
        f.write_all(b"%PDF-1.4 content v1").unwrap();

        // New file → needs compile
        assert!(cache.needs_compile(&pdf_path).unwrap());

        // Record compile
        cache
            .record_compile(&pdf_path, vec!["wiki/it/c.md".into()])
            .unwrap();
        assert!(!cache.needs_compile(&pdf_path).unwrap());

        // Modify file → needs recompile
        let mut f = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&pdf_path)
            .unwrap();
        f.write_all(b"%PDF-1.4 content v2 modified").unwrap();
        assert!(cache.needs_compile(&pdf_path).unwrap());
    }
}
