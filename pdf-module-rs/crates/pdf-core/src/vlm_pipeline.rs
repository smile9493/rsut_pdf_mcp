//! VLM-enhanced PDF extraction pipeline.
//!
//! Integrates `vlm-visual-gateway` with the existing Pdfium-based pipeline,
//! implementing the conditional escalation strategy:
//! - 90% normal documents stay on local Pdfium
//! - 10% extreme cases (scans, chaotic layouts) escalate to VLM

use std::path::Path;
use std::sync::Arc;

use tracing::{info, warn};

use vlm_visual_gateway::{
    DetectionResult, EscalationDetector, LayoutResult, MetricsCollector, PayloadMetadata,
    VlmConfig, VlmGateway,
};

use crate::config::ServerConfig;
use crate::dto::{ExtractOptions, PageMetadata, StructuredExtractionResult, TextExtractionResult};
use crate::engine::{PdfEngine, PdfiumEngine};
use crate::error::{PdfModuleError, PdfResult};
use crate::validator::FileValidator;

/// Configuration for the VLM-enhanced pipeline.
#[derive(Debug, Clone)]
pub struct VlmPipelineConfig {
    pub vlm_enabled: bool,
    pub render_dpi: f32,
}

impl Default for VlmPipelineConfig {
    fn default() -> Self {
        Self {
            vlm_enabled: false,
            render_dpi: 150.0,
        }
    }
}

/// Enhanced extraction result that may include VLM layout understanding.
#[derive(Debug, Clone)]
pub struct VlmEnhancedResult {
    pub base_result: StructuredExtractionResult,
    pub layout_results: Vec<Option<LayoutResult>>,
    pub vlm_triggered: bool,
}

/// VLM-enhanced PDF extraction pipeline.
pub struct VlmEnhancedPipeline {
    engine: PdfiumEngine,
    validator: FileValidator,
    detector: EscalationDetector,
    gateway: Option<VlmGateway>,
    metrics: Arc<MetricsCollector>,
    config: VlmPipelineConfig,
}

impl VlmEnhancedPipeline {
    pub fn new(server_config: &ServerConfig, vlm_config: Option<VlmConfig>) -> PdfResult<Self> {
        let detector = EscalationDetector::default();
        let metrics = Arc::new(MetricsCollector::with_default_registry());

        let gateway = match vlm_config {
            Some(config) => Some(
                VlmGateway::new(config, Arc::clone(&metrics))
                    .map_err(|e| PdfModuleError::ConfigError(format!("VLM gateway: {e}")))?,
            ),
            None => None,
        };

        let vlm_enabled = gateway.is_some();

        Ok(Self {
            engine: PdfiumEngine::new()?,
            validator: FileValidator::new(server_config.security.max_file_size_mb as u32),
            detector,
            gateway,
            metrics,
            config: VlmPipelineConfig {
                vlm_enabled,
                ..Default::default()
            },
        })
    }

    pub fn with_config(
        server_config: &ServerConfig,
        vlm_config: Option<VlmConfig>,
        pipeline_config: VlmPipelineConfig,
    ) -> PdfResult<Self> {
        let mut pipeline = Self::new(server_config, vlm_config)?;
        pipeline.config = pipeline_config;
        Ok(pipeline)
    }

    pub fn local_only(server_config: &ServerConfig) -> PdfResult<Self> {
        Self::new(server_config, None)
    }

    pub async fn extract_text(&self, file_path: &Path) -> PdfResult<TextExtractionResult> {
        self.validator.validate(file_path)?;
        self.engine.extract_text(file_path).await
    }

    pub async fn extract_structured(
        &self,
        file_path: &Path,
        options: &ExtractOptions,
    ) -> PdfResult<VlmEnhancedResult> {
        self.validator.validate(file_path)?;

        let base_result = self.engine.extract_structured(file_path, options).await?;

        if !self.config.vlm_enabled || self.gateway.is_none() {
            let page_count = base_result.pages.len();
            return Ok(VlmEnhancedResult {
                base_result,
                layout_results: vec![None; page_count],
                vlm_triggered: false,
            });
        }

        let mut layout_results: Vec<Option<LayoutResult>> = Vec::new();
        let mut vlm_triggered = false;

        for page in &base_result.pages {
            let extraction = vlm_visual_gateway::PdfiumExtraction {
                character_count: page.text.len() as u32,
                layout_confidence: self.estimate_layout_confidence(page),
                text: page.text.clone(),
                page_width: page.bbox.map(|b| b.2 as f32).unwrap_or(612.0),
                page_height: page.bbox.map(|b| b.3 as f32).unwrap_or(792.0),
            };

            let detection = self.detector.detect(&extraction);

            match detection {
                DetectionResult::Normal => {
                    layout_results.push(None);
                }
                DetectionResult::ZeroText | DetectionResult::LayoutChaos => {
                    vlm_triggered = true;
                    info!(
                        page_number = page.page_number,
                        detection = ?detection,
                        "VLM escalation detected"
                    );
                    layout_results.push(None);
                }
            }
        }

        Ok(VlmEnhancedResult {
            base_result,
            layout_results,
            vlm_triggered,
        })
    }

    pub async fn get_page_count(&self, file_path: &Path) -> PdfResult<u32> {
        self.validator.validate(file_path)?;
        self.engine.get_page_count(file_path).await
    }

    pub fn metrics(&self) -> &Arc<MetricsCollector> {
        &self.metrics
    }

    pub fn gateway(&self) -> Option<&VlmGateway> {
        self.gateway.as_ref()
    }

    /// Render a page and send to VLM for layout understanding.
    ///
    /// Data flow:
    ///   Pdfium render → raw RGBA Vec<u8> → perceive_layout (internal Base64) → VLM API
    pub async fn perceive_page(
        &self,
        file_path: &Path,
        page_index: u32,
    ) -> PdfResult<Option<LayoutResult>> {
        let gateway = self.gateway.as_ref().ok_or_else(|| {
            PdfModuleError::ConfigError("VLM gateway not configured".into())
        })?;

        let data = std::fs::read(file_path)?;

        // render_page_pixels returns raw RGBA bytes — no encoding yet
        let (pixels, width, height) = vlm_visual_gateway::render_page_pixels(
            &data,
            page_index,
            self.config.render_dpi,
        )
        .map_err(|e| PdfModuleError::Extraction(format!("render page: {e}")))?;

        let metadata = PayloadMetadata {
            page_width: width as f32,
            page_height: height as f32,
            page_number: page_index + 1,
        };

        // perceive_layout accepts raw RGBA bytes and does Base64 internally
        match gateway
            .perceive_layout(&pixels, None, &metadata)
            .await
        {
            Ok(layout) => Ok(Some(layout)),
            Err(e) => {
                warn!("VLM perceive failed: {e} - degrading to local");
                Ok(None)
            }
        }
    }

    fn estimate_layout_confidence(&self, page: &PageMetadata) -> f32 {
        let line_count = page.lines.len() as f32;
        let text_len = page.text.len() as f32;

        if text_len < 10.0 {
            return 0.0;
        }

        let line_score = (line_count / 10.0).min(1.0);
        let text_score = (text_len / 500.0).min(1.0);

        (line_score * 0.5 + text_score * 0.5).max(0.0).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_local_only() {
        let config = ServerConfig::default();
        let pipeline = VlmEnhancedPipeline::local_only(&config).unwrap();
        assert!(!pipeline.config.vlm_enabled);
        assert!(pipeline.gateway.is_none());
    }

    #[test]
    fn test_layout_confidence_estimation() {
        let config = ServerConfig::default();
        let pipeline = VlmEnhancedPipeline::local_only(&config).unwrap();

        let page = PageMetadata {
            page_number: 1,
            text: String::new(),
            bbox: Some((0.0, 0.0, 612.0, 792.0)),
            lines: vec![],
        };
        assert_eq!(pipeline.estimate_layout_confidence(&page), 0.0);

        let page = PageMetadata {
            page_number: 1,
            text: "A".repeat(1000),
            bbox: Some((0.0, 0.0, 612.0, 792.0)),
            lines: (0..20)
                .map(|i| crate::dto::LineInfo {
                    bbox: vec![0.0, i as f64 * 20.0, 500.0, 15.0],
                    text: format!("line {i}"),
                })
                .collect(),
        };
        assert!(pipeline.estimate_layout_confidence(&page) > 0.8);
    }
}
