//! Pdfium engine implementation
//! High-compatibility fallback engine

use async_trait::async_trait;
use std::path::Path;

use crate::dto::{
    ExtractOptions, FileInfo, PageMetadata, StructuredExtractionResult, TextExtractionResult,
};
use crate::engine::PdfEngine;
use crate::error::{PdfModuleError, PdfResult};

/// High-compatibility PDF engine based on PDFium
/// Used as fallback for complex documents
pub struct PdfiumEngine;

impl PdfiumEngine {
    pub fn new() -> PdfResult<Self> {
        Ok(Self)
    }
}

impl Default for PdfiumEngine {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::error!("Failed to create PdfiumEngine: {}", e);
            Self
        })
    }
}

/// Convert PdfiumError to PdfModuleError
fn pdfium_error(e: pdfium_render::prelude::PdfiumError) -> PdfModuleError {
    PdfModuleError::Extraction(format!("Pdfium error: {}", e))
}

#[async_trait]
impl PdfEngine for PdfiumEngine {
    fn id(&self) -> &str {
        "pdfium"
    }

    fn name(&self) -> &str {
        "PdfiumEngine"
    }

    fn description(&self) -> &str {
        "High-compatibility PDF engine based on PDFium"
    }

    async fn extract_text(&self, file_path: &Path) -> PdfResult<TextExtractionResult> {
        // Use pdfium-render for text extraction
        use pdfium_render::prelude::*;

        let pdfium = Pdfium::default();
        let document = pdfium
            .load_pdf_from_file(file_path, None)
            .map_err(pdfium_error)?;

        let mut all_text = String::new();
        let pages = document.pages();

        for i in 0..pages.len() {
            let page = pages.get(i).map_err(pdfium_error)?;
            let text = page.text().map_err(pdfium_error)?;
            // Convert PdfPageText to String
            let text_str = text.all();
            all_text.push_str(&text_str);
            all_text.push('\n');
        }

        Ok(TextExtractionResult {
            extracted_text: all_text.trim().to_string(),
            extraction_metadata: None,
        })
    }

    async fn extract_structured(
        &self,
        file_path: &Path,
        _options: &ExtractOptions,
    ) -> PdfResult<StructuredExtractionResult> {
        use pdfium_render::prelude::*;

        let pdfium = Pdfium::default();
        let document = pdfium
            .load_pdf_from_file(file_path, None)
            .map_err(pdfium_error)?;

        let pages = document.pages();
        let page_count = pages.len() as u32;
        let mut all_text = String::new();
        let mut page_metas = Vec::with_capacity(page_count as usize);

        for i in 0..pages.len() {
            let page = pages.get(i).map_err(pdfium_error)?;
            let text = page.text().map_err(pdfium_error)?;
            // Convert PdfPageText to String
            let text_str = text.all();

            // Get page dimensions as bbox
            // width() and height() return PdfPoints directly, not Result
            // Convert f32 to f64 for bbox
            let width = page.width().value as f64;
            let height = page.height().value as f64;
            let bbox = Some((0.0, 0.0, width, height));

            page_metas.push(PageMetadata {
                page_number: (i + 1) as u32,
                text: text_str.trim().to_string(),
                bbox,
                lines: vec![],
            });

            all_text.push_str(&text_str);
            all_text.push('\n');
        }

        let file_info = FileInfo::from_path(file_path)?;

        Ok(StructuredExtractionResult {
            extracted_text: all_text.trim().to_string(),
            page_count,
            pages: page_metas,
            extraction_metadata: None,
            file_info,
        })
    }

    async fn get_page_count(&self, file_path: &Path) -> PdfResult<u32> {
        use pdfium_render::prelude::*;

        let pdfium = Pdfium::default();
        let document = pdfium
            .load_pdf_from_file(file_path, None)
            .map_err(pdfium_error)?;

        Ok(document.pages().len() as u32)
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
        let engine = PdfiumEngine::new().unwrap();
        assert_eq!(engine.id(), "pdfium");
        assert_eq!(engine.name(), "PdfiumEngine");
        assert!(engine.test_connection());
    }
}
