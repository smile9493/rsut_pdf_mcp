//! Pdf-extract engine implementation
//! Corresponds to Python: adapters/pdfplumber_adapter.py

use async_trait::async_trait;
use lopdf::Document;
use std::path::Path;

use crate::dto::{
    ExtractOptions, FileInfo, PageMetadata, StructuredExtractionResult, TextExtractionResult,
};
use crate::engine::PdfEngine;
use crate::error::{PdfModuleError, PdfResult};

/// Fast text extraction engine based on pdf-extract
/// Corresponds to Python: pdfplumber adapter
pub struct PdfExtractEngine;

impl PdfExtractEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PdfExtractEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PdfEngine for PdfExtractEngine {
    fn id(&self) -> &str {
        "pdf-extract"
    }

    fn name(&self) -> &str {
        "PdfExtractEngine"
    }

    fn description(&self) -> &str {
        "Fast text extraction engine based on pdf-extract"
    }

    async fn extract_text(&self, file_path: &Path) -> PdfResult<TextExtractionResult> {
        // Use pdf_extract crate for fast text extraction
        let content = std::fs::read(file_path)
            .map_err(|e| PdfModuleError::Extraction(format!("Failed to read PDF: {}", e)))?;

        let text = pdf_extract::extract_text_from_mem(&content)
            .map_err(|e| PdfModuleError::Extraction(format!("Failed to extract text: {}", e)))?;

        Ok(TextExtractionResult {
            extracted_text: text.trim().to_string(),
            extraction_metadata: None,
        })
    }

    async fn extract_structured(
        &self,
        file_path: &Path,
        _options: &ExtractOptions,
    ) -> PdfResult<StructuredExtractionResult> {
        // pdf-extract doesn't provide structured output, so we combine with lopdf for page info
        let content = std::fs::read(file_path)
            .map_err(|e| PdfModuleError::Extraction(format!("Failed to read PDF: {}", e)))?;

        let text = pdf_extract::extract_text_from_mem(&content)
            .map_err(|e| PdfModuleError::Extraction(format!("Failed to extract text: {}", e)))?;

        // Get page count from lopdf
        let doc = Document::load(file_path).map_err(|e| {
            PdfModuleError::Extraction(format!("Failed to load PDF for page count: {}", e))
        })?;
        let page_count = doc.get_pages().len() as u32;

        // Create a single page with all text (pdf-extract doesn't provide per-page info)
        let pages = vec![PageMetadata {
            page_number: 1,
            text: text.trim().to_string(),
            bbox: None,
            lines: vec![],
        }];

        let file_info = FileInfo::from_path(file_path)?;

        Ok(StructuredExtractionResult {
            extracted_text: text.trim().to_string(),
            page_count,
            pages,
            extraction_metadata: None,
            file_info,
        })
    }

    async fn get_page_count(&self, file_path: &Path) -> PdfResult<u32> {
        // Use lopdf for page count (pdf-extract doesn't have this API)
        let doc = Document::load(file_path)
            .map_err(|e| PdfModuleError::Extraction(format!("Failed to load PDF: {}", e)))?;
        Ok(doc.get_pages().len() as u32)
    }

    fn test_connection(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_metadata() {
        let engine = PdfExtractEngine::new();
        assert_eq!(engine.id(), "pdf-extract");
        assert_eq!(engine.name(), "PdfExtractEngine");
        assert!(engine.test_connection());
    }
}
