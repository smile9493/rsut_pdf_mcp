use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Write as IoWrite;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::dto::StructuredExtractionResult;
use crate::error::{PdfModuleError, PdfResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMetadata {
    pub source_file: String,
    pub file_hash: String,
    pub extraction_time: DateTime<Utc>,
    pub page_count: u32,
    pub quality_score: f64,
    pub extraction_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawDocument {
    pub metadata: RawMetadata,
    pub content: String,
}

pub struct WikiStorage {
    base_path: PathBuf,
}

impl WikiStorage {
    pub fn new(base_path: impl AsRef<Path>) -> PdfResult<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        
        fs::create_dir_all(base_path.join("raw"))
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to create raw dir: {}", e)))?;
        fs::create_dir_all(base_path.join("wiki"))
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to create wiki dir: {}", e)))?;
        fs::create_dir_all(base_path.join("scheme"))
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to create scheme dir: {}", e)))?;

        Ok(Self { base_path })
    }

    pub fn save_raw(
        &self,
        extraction_result: &StructuredExtractionResult,
        source_file: &Path,
        quality_score: f64,
        extraction_method: &str,
    ) -> PdfResult<PathBuf> {
        let file_hash = Self::compute_file_hash(&extraction_result.extracted_text);
        let raw_filename = format!("{}.raw.md", file_hash);
        let raw_path = self.base_path.join("raw").join(&raw_filename);

        let metadata = RawMetadata {
            source_file: source_file.to_string_lossy().to_string(),
            file_hash: file_hash.clone(),
            extraction_time: Utc::now(),
            page_count: extraction_result.page_count,
            quality_score,
            extraction_method: extraction_method.to_string(),
        };

        let raw_doc = RawDocument {
            metadata,
            content: extraction_result.extracted_text.clone(),
        };

        let yaml_frontmatter = serde_yaml::to_string(&raw_doc.metadata)
            .map_err(|e| PdfModuleError::StorageError(format!("YAML serialization: {}", e)))?;

        let full_content = format!("---\n{}---\n\n{}", yaml_frontmatter, raw_doc.content);

        let mut file = File::create(&raw_path)
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to create raw file: {}", e)))?;
        
        file.write_all(full_content.as_bytes())
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to write raw file: {}", e)))?;

        Ok(raw_path)
    }

    pub fn generate_map(&self) -> PdfResult<PathBuf> {
        let map_path = self.base_path.join("MAP.md");
        let raw_dir = self.base_path.join("raw");

        let mut map_content = String::new();
        map_content.push_str("# PDF Knowledge Map\n\n");
        map_content.push_str("## Raw Extractions\n\n");

        if raw_dir.exists() {
            let entries: Vec<_> = fs::read_dir(&raw_dir)
                .map_err(|e| PdfModuleError::StorageError(format!("Failed to read raw dir: {}", e)))?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
                .collect();

            let count = entries.len();
            
            for entry in &entries {
                let filename = entry.file_name().to_string_lossy().to_string();
                map_content.push_str(&format!("- [{}](raw/{})\n", filename, filename));
            }

            map_content.push_str(&format!("\n**Total documents**: {}\n", count));
        }

        let mut file = File::create(&map_path)
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to create MAP.md: {}", e)))?;
        
        file.write_all(map_content.as_bytes())
            .map_err(|e| PdfModuleError::StorageError(format!("Failed to write MAP.md: {}", e)))?;

        Ok(map_path)
    }

    pub fn raw_path(&self) -> PathBuf {
        self.base_path.join("raw")
    }

    pub fn wiki_path(&self) -> PathBuf {
        self.base_path.join("wiki")
    }

    pub fn scheme_path(&self) -> PathBuf {
        self.base_path.join("scheme")
    }

    fn compute_file_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)[..16].to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPayload {
    pub metadata: RawMetadata,
    pub content: String,
    pub prompt: String,
}

impl AgentPayload {
    pub fn from_extraction(
        extraction_result: &StructuredExtractionResult,
        source_file: &Path,
        quality_score: f64,
        extraction_method: &str,
    ) -> Self {
        let file_hash = WikiStorage::compute_file_hash(&extraction_result.extracted_text);
        
        let metadata = RawMetadata {
            source_file: source_file.to_string_lossy().to_string(),
            file_hash,
            extraction_time: Utc::now(),
            page_count: extraction_result.page_count,
            quality_score,
            extraction_method: extraction_method.to_string(),
        };

        let prompt = Self::generate_prompt(&metadata);

        Self {
            metadata,
            content: extraction_result.extracted_text.clone(),
            prompt,
        }
    }

    fn generate_prompt(metadata: &RawMetadata) -> String {
        format!(
            r#"# PDF Extraction Complete

Your PDF has been successfully extracted. Here's what you should do:

## 1. Create Local Wiki Structure

Create the following directory structure on your local machine:

```
wiki/
├── raw/
│   └── {}.raw.md
├── wiki/
│   └── (entity pages will go here)
└── scheme/
    └── (type constraints will go here)
```

## 2. Save the Raw Document

Save the content below (including YAML frontmatter) to:
`wiki/raw/{}.raw.md`

## 3. Process the Content

- Extract entities and create individual wiki pages
- Build cross-references between related documents
- Update your local MAP.md index

## Extraction Metadata

- **Source**: {}
- **Pages**: {}
- **Quality Score**: {:.2}
- **Method**: {}
- **Time**: {}

---

# Extracted Content

See below for the full extraction with YAML frontmatter.
"#,
            metadata.file_hash,
            metadata.file_hash,
            metadata.source_file,
            metadata.page_count,
            metadata.quality_score,
            metadata.extraction_method,
            metadata.extraction_time.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }

    pub fn to_markdown(&self) -> String {
        let yaml_frontmatter = serde_yaml::to_string(&self.metadata)
            .unwrap_or_else(|_| "metadata: error".to_string());

        format!("---\n{}---\n\n{}\n\n{}", yaml_frontmatter, self.prompt, self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_wiki_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = WikiStorage::new(temp_dir.path()).unwrap();
        
        assert!(storage.raw_path().exists());
        assert!(storage.wiki_path().exists());
        assert!(storage.scheme_path().exists());
    }

    #[test]
    fn test_compute_file_hash() {
        let hash1 = WikiStorage::compute_file_hash("test content");
        let hash2 = WikiStorage::compute_file_hash("test content");
        let hash3 = WikiStorage::compute_file_hash("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 16);
    }
}
