use crate::config::ServerConfig;
use crate::dto::{ExtractOptions, StructuredExtractionResult, TextExtractionResult};
use crate::engine::{PdfEngine, PdfiumEngine};
use crate::error::PdfResult;
use crate::validator::FileValidator;
use std::path::Path;

pub struct McpPdfPipeline {
    engine: PdfiumEngine,
    validator: FileValidator,
}

impl McpPdfPipeline {
    pub fn new(config: &ServerConfig) -> PdfResult<Self> {
        Ok(Self {
            engine: PdfiumEngine::new()?,
            validator: FileValidator::new(config.security.max_file_size_mb as u32),
        })
    }

    pub async fn extract_text(&self, file_path: &Path) -> PdfResult<TextExtractionResult> {
        self.validator.validate(file_path)?;
        self.engine.extract_text(file_path).await
    }

    pub async fn extract_structured(
        &self,
        file_path: &Path,
        options: &ExtractOptions,
    ) -> PdfResult<StructuredExtractionResult> {
        self.validator.validate(file_path)?;
        self.engine.extract_structured(file_path, options).await
    }

    pub async fn get_page_count(&self, file_path: &Path) -> PdfResult<u32> {
        self.validator.validate(file_path)?;
        self.engine.get_page_count(file_path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let config = ServerConfig::default();
        let pipeline = McpPdfPipeline::new(&config).unwrap();
        assert_eq!(pipeline.engine.id(), "pdfium");
    }
}
