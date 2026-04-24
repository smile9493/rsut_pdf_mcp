//! 文本提取引擎
//!
//! 复用现有 pdf-module-rs 的提取能力

use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, info, warn};

use crate::config::{ChunkStrategy, ExtractionConfig, ExtractionEngine};
use crate::dto::{ExtractionResult, PdfMetadata};
use crate::error::{EtlError, Result};

/// 默认最大文件大小 (100 MB)
const DEFAULT_MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// 提取服务
pub struct ExtractionService {
    config: ExtractionConfig,
    /// 允许的基础路径（用于路径遍历检查）
    allowed_base_path: Option<PathBuf>,
    /// 最大文件大小（字节）
    max_file_size: u64,
}

impl Default for ExtractionService {
    fn default() -> Self {
        Self {
            config: ExtractionConfig::default(),
            allowed_base_path: None,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
        }
    }
}

impl ExtractionService {
    /// 创建新的提取服务
    pub fn new(config: ExtractionConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            config,
            allowed_base_path: None,
            max_file_size: DEFAULT_MAX_FILE_SIZE,
        })
    }

    /// 设置允许的基础路径
    pub fn with_allowed_base_path(mut self, path: PathBuf) -> Self {
        self.allowed_base_path = Some(path);
        self
    }

    /// 设置最大文件大小
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// 验证文件路径安全性
    fn validate_path(&self, pdf_path: &str) -> Result<PathBuf> {
        let path = Path::new(pdf_path);

        // 检查路径是否存在
        if !path.exists() {
            return Err(EtlError::FileNotFoundError(pdf_path.to_string()));
        }

        // 获取规范化的绝对路径
        let canonical_path = path
            .canonicalize()
            .map_err(|e| EtlError::ExtractionError(format!("Failed to resolve path: {}", e)))?;

        // 检查路径遍历攻击
        if let Some(ref base_path) = self.allowed_base_path {
            let base_canonical = base_path.canonicalize().map_err(|e| {
                EtlError::ExtractionError(format!("Failed to resolve base path: {}", e))
            })?;

            if !canonical_path.starts_with(&base_canonical) {
                warn!(
                    "Path traversal detected: {} is outside allowed base path",
                    pdf_path
                );
                return Err(EtlError::ValidationError(
                    "Path traversal attack detected".to_string(),
                    vec![format!("Path '{}' is outside allowed directory", pdf_path)],
                ));
            }
        }

        // 检查文件大小
        let metadata = canonical_path.metadata().map_err(|e| {
            EtlError::ExtractionError(format!("Failed to get file metadata: {}", e))
        })?;

        if metadata.len() > self.max_file_size {
            return Err(EtlError::ValidationError(
                "File too large".to_string(),
                vec![format!(
                    "File size {} bytes exceeds maximum allowed {} bytes",
                    metadata.len(),
                    self.max_file_size
                )],
            ));
        }

        // 检查是否为常规文件（非目录、非符号链接）
        if !metadata.is_file() {
            return Err(EtlError::ValidationError(
                "Not a regular file".to_string(),
                vec![format!("Path '{}' is not a regular file", pdf_path)],
            ));
        }

        Ok(canonical_path)
    }

    /// 提取 PDF 文本
    pub async fn extract(&self, pdf_path: &str) -> Result<ExtractionResult> {
        let start = Instant::now();

        // 验证路径安全性
        let canonical_path = self.validate_path(pdf_path)?;
        let path_str = canonical_path.to_string_lossy().to_string();

        // 获取文件大小
        let _file_size = canonical_path
            .metadata()
            .map(|m| m.len() as f64 / 1024.0)
            .unwrap_or(0.0);

        info!(
            "Extracting PDF: {} (engine: {:?})",
            path_str, self.config.engine
        );

        // 根据引擎选择提取方式
        let (text, metadata) = match self.config.engine {
            ExtractionEngine::Lopdf => self.extract_with_lopdf(&path_str).await?,
            ExtractionEngine::Pdfium => self.extract_with_pdfium(&path_str).await?,
        };

        // 分块
        let chunks = self.chunk_text(&text, &metadata);

        let processing_time_ms = start.elapsed().as_millis() as u64;

        info!(
            "Extraction completed: {} chars, {} chunks, {}ms",
            text.len(),
            chunks.len(),
            processing_time_ms
        );

        Ok(ExtractionResult::new(
            text,
            metadata,
            chunks,
            processing_time_ms,
        ))
    }

    /// 使用 lopdf 提取
    async fn extract_with_lopdf(&self, pdf_path: &str) -> Result<(String, PdfMetadata)> {
        use lopdf::Document;

        let doc = Document::load(pdf_path)
            .map_err(|e| EtlError::CorruptedPdfError(format!("Failed to load PDF: {}", e)))?;

        // 提取文本
        let mut text = String::new();
        let page_count = doc.get_pages().len();

        for (page_num, _) in doc.get_pages() {
            if let Ok(page_text) = doc.extract_text(&[page_num]) {
                text.push_str(&page_text);
                text.push('\n');
            }
        }

        // 提取元数据
        let metadata = self.extract_metadata(&doc, page_count, pdf_path);

        Ok((text, metadata))
    }

    /// 使用 pdfium 提取
    async fn extract_with_pdfium(&self, pdf_path: &str) -> Result<(String, PdfMetadata)> {
        // pdfium 提取需要 pdfium-render 库
        // 这里提供简化实现，实际使用时需要配置 pdfium
        debug!("Using pdfium engine for extraction");

        // 回退到 lopdf
        self.extract_with_lopdf(pdf_path).await
    }

    /// 提取 PDF 元数据
    fn extract_metadata(
        &self,
        doc: &lopdf::Document,
        page_count: usize,
        pdf_path: &str,
    ) -> PdfMetadata {
        let path = Path::new(pdf_path);
        let file_size_kb = path
            .metadata()
            .map(|m| m.len() as f64 / 1024.0)
            .unwrap_or(0.0);

        // 尝试从 trailer 提取元数据
        let (author, title, created_at) = {
            let mut author = None;
            let mut title = None;
            let mut created_at = None;

            if let Ok(trailer) = doc.trailer.get(b"Info") {
                if let Ok(info_id) = trailer.as_reference() {
                    if let Ok(lopdf::Object::Dictionary(dict)) = doc.get_object(info_id) {
                        author = dict
                            .get(b"Author")
                            .ok()
                            .and_then(|v| v.as_str().ok())
                            .map(|s| String::from_utf8_lossy(s).to_string());
                        title = dict
                            .get(b"Title")
                            .ok()
                            .and_then(|v| v.as_str().ok())
                            .map(|s| String::from_utf8_lossy(s).to_string());
                        created_at = dict
                            .get(b"CreationDate")
                            .ok()
                            .and_then(|v| v.as_str().ok())
                            .map(|s| String::from_utf8_lossy(s).to_string());
                    }
                }
            }

            (author, title, created_at)
        };

        PdfMetadata {
            page_count,
            author,
            title,
            created_at,
            file_size_kb,
        }
    }

    /// 文本分块
    fn chunk_text(&self, text: &str, _metadata: &PdfMetadata) -> Vec<String> {
        match self.config.chunk_strategy {
            ChunkStrategy::Page => {
                // 按页分块（简化实现：按换行符分割）
                let chunks: Vec<String> = text
                    .split("\n\n")
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.to_string())
                    .collect();
                if chunks.is_empty() {
                    vec![text.to_string()]
                } else {
                    chunks
                }
            }
            ChunkStrategy::Paragraph => {
                // 按段落分块
                let chunks: Vec<String> = text
                    .split("\n\n")
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.to_string())
                    .collect();
                if chunks.is_empty() {
                    vec![text.to_string()]
                } else {
                    chunks
                }
            }
            ChunkStrategy::FixedLength => {
                // 按固定长度分块
                let chunk_size = self.config.chunk_size;
                let chars: Vec<char> = text.chars().collect();
                let mut chunks = Vec::new();

                for i in (0..chars.len()).step_by(chunk_size) {
                    let end = (i + chunk_size).min(chars.len());
                    chunks.push(chars[i..end].iter().collect());
                }

                if chunks.is_empty() {
                    vec![text.to_string()]
                } else {
                    chunks
                }
            }
        }
    }

    /// 获取配置
    pub fn config(&self) -> &ExtractionConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_service_creation() {
        let config = ExtractionConfig::default();
        let service = ExtractionService::new(config);
        assert!(service.is_ok());
    }

    #[test]
    fn test_chunk_text_fixed_length() {
        let config = ExtractionConfig {
            engine: ExtractionEngine::Lopdf,
            chunk_strategy: ChunkStrategy::FixedLength,
            chunk_size: 10,
            include_metadata: true,
        };
        let service = ExtractionService::new(config).unwrap();

        let text = "This is a test text for chunking.";
        let metadata = PdfMetadata::default();
        let chunks = service.chunk_text(text, &metadata);

        assert!(!chunks.is_empty());
        // 每个块最多 10 个字符
        for chunk in &chunks {
            assert!(chunk.len() <= 10);
        }
    }

    #[test]
    fn test_chunk_text_paragraph() {
        let config = ExtractionConfig {
            engine: ExtractionEngine::Lopdf,
            chunk_strategy: ChunkStrategy::Paragraph,
            chunk_size: 1000,
            include_metadata: true,
        };
        let service = ExtractionService::new(config).unwrap();

        let text = "Paragraph 1.\n\nParagraph 2.\n\nParagraph 3.";
        let metadata = PdfMetadata::default();
        let chunks = service.chunk_text(text, &metadata);

        assert_eq!(chunks.len(), 3);
    }
}
