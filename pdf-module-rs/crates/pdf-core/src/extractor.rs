use crate::config::ServerConfig;
use crate::dto::{ExtractOptions, StructuredExtractionResult, TextExtractionResult};
use crate::engine::PdfiumEngine;
use crate::error::PdfResult;
use crate::mmap_loader::MmapPdfLoader;
use crate::quality_probe::{ExtractionMethod, QualityProbe, QualityReport};
use crate::validator::FileValidator;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};
use vlm_visual_gateway::{MetricsCollector, VlmConfig, VlmGateway};

pub struct McpPdfPipeline {
    validator: FileValidator,
    #[allow(dead_code)]
    vlm_gateway: Option<VlmGateway>,
    #[allow(dead_code)]
    metrics: Arc<MetricsCollector>,
}

pub struct ExtractionContext {
    pub quality_report: QualityReport,
    pub loader: MmapPdfLoader,
}

impl McpPdfPipeline {
    pub fn new(config: &ServerConfig) -> PdfResult<Self> {
        let metrics = Arc::new(MetricsCollector::with_default_registry());
        
        let vlm_gateway = match VlmConfig::from_env() {
            Ok(vlm_config) => {
                match VlmGateway::new(vlm_config, Arc::clone(&metrics)) {
                    Ok(gateway) => {
                        info!("VLM gateway initialized successfully");
                        Some(gateway)
                    }
                    Err(e) => {
                        warn!("Failed to initialize VLM gateway: {} - operating in local-only mode", e);
                        None
                    }
                }
            }
            Err(_) => {
                info!("VLM not configured - operating in local-only mode");
                None
            }
        };

        Ok(Self {
            validator: FileValidator::new(config.security.max_file_size_mb as u32),
            vlm_gateway,
            metrics,
        })
    }

    pub fn with_vlm(config: &ServerConfig, vlm_config: VlmConfig) -> PdfResult<Self> {
        let metrics = Arc::new(MetricsCollector::with_default_registry());
        let vlm_gateway = VlmGateway::new(vlm_config, Arc::clone(&metrics))
            .map_err(|e| crate::error::PdfModuleError::ConfigError(format!("VLM gateway: {}", e)))?;

        Ok(Self {
            validator: FileValidator::new(config.security.max_file_size_mb as u32),
            vlm_gateway: Some(vlm_gateway),
            metrics,
        })
    }

    fn probe_and_load(&self, file_path: &Path) -> PdfResult<ExtractionContext> {
        self.validator.validate(file_path)?;
        let loader = MmapPdfLoader::load(file_path)?;
        let quality_report = QualityProbe::probe_with_pdfium(loader.as_bytes())?;
        
        info!(
            file = ?file_path,
            quality = ?quality_report.quality,
            text_density = quality_report.text_density,
            needs_vlm = quality_report.needs_vlm,
            extraction_method = ?quality_report.extraction_method,
            "PDF quality analysis complete"
        );

        Ok(ExtractionContext {
            quality_report,
            loader,
        })
    }

    pub async fn extract_text(&self, file_path: &Path) -> PdfResult<TextExtractionResult> {
        let ctx = self.probe_and_load(file_path)?;
        
        match ctx.quality_report.extraction_method {
            ExtractionMethod::Pdfium => {
                let text = PdfiumEngine::extract_text_from_mmap(&ctx.loader)?;
                Ok(TextExtractionResult {
                    extracted_text: text,
                    extraction_metadata: None,
                })
            }
            ExtractionMethod::Vlm => {
                warn!(
                    file = ?file_path,
                    "VLM-only extraction not yet implemented, falling back to Pdfium"
                );
                let text = PdfiumEngine::extract_text_from_mmap(&ctx.loader)?;
                Ok(TextExtractionResult {
                    extracted_text: text,
                    extraction_metadata: None,
                })
            }
            ExtractionMethod::Hybrid => {
                let text = PdfiumEngine::extract_text_from_mmap(&ctx.loader)?;
                Ok(TextExtractionResult {
                    extracted_text: text,
                    extraction_metadata: None,
                })
            }
        }
    }

    pub async fn extract_structured(
        &self,
        file_path: &Path,
        _options: &ExtractOptions,
    ) -> PdfResult<StructuredExtractionResult> {
        let ctx = self.probe_and_load(file_path)?;
        
        match ctx.quality_report.extraction_method {
            ExtractionMethod::Pdfium => {
                PdfiumEngine::extract_structured_from_mmap(&ctx.loader, file_path)
            }
            ExtractionMethod::Vlm => {
                warn!(
                    file = ?file_path,
                    "VLM-only extraction not yet implemented, falling back to Pdfium"
                );
                PdfiumEngine::extract_structured_from_mmap(&ctx.loader, file_path)
            }
            ExtractionMethod::Hybrid => {
                PdfiumEngine::extract_structured_from_mmap(&ctx.loader, file_path)
            }
        }
    }

    pub async fn get_page_count(&self, file_path: &Path) -> PdfResult<u32> {
        let ctx = self.probe_and_load(file_path)?;
        PdfiumEngine::get_page_count_from_mmap(&ctx.loader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let config = ServerConfig::default();
        let pipeline = McpPdfPipeline::new(&config).unwrap();
        assert!(pipeline.vlm_gateway.is_none());
    }
}
