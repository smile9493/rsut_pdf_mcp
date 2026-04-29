use serde::{Deserialize, Serialize};
use std::time::Duration;

// ──────────────────────────────────────────────
//  VLM Model & Config
// ──────────────────────────────────────────────

/// Supported VLM model variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VlmModel {
    Gpt4o,
    Claude35Sonnet,
}

impl VlmModel {
    /// OpenAI-compatible model identifier string
    pub fn model_id(&self) -> &'static str {
        match self {
            VlmModel::Gpt4o => "gpt-4o",
            VlmModel::Claude35Sonnet => "claude-3.5-sonnet",
        }
    }
}

/// VLM gateway configuration (immutable after construction)
#[derive(Debug, Clone)]
pub struct VlmConfig {
    /// API endpoint URL
    pub endpoint: String,
    /// API Key (read from env or secure config)
    pub api_key: String,
    /// Target model
    pub model: VlmModel,
    /// Request timeout
    pub timeout: Duration,
    /// Max concurrent requests
    pub max_concurrency: usize,
}

impl VlmConfig {
    /// Build from environment variables
    pub fn from_env() -> Result<Self, crate::error::VlmError> {
        let endpoint = std::env::var("VLM_ENDPOINT")
            .map_err(|_| crate::error::VlmError::Config("VLM_ENDPOINT not set".into()))?;
        let api_key = std::env::var("VLM_API_KEY")
            .map_err(|_| crate::error::VlmError::Config("VLM_API_KEY not set".into()))?;

        let model = match std::env::var("VLM_MODEL").as_deref() {
            Ok("claude-3.5-sonnet") => VlmModel::Claude35Sonnet,
            _ => VlmModel::Gpt4o,
        };

        let timeout_secs = std::env::var("VLM_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(30);

        let max_concurrency = std::env::var("VLM_MAX_CONCURRENCY")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(5);

        Ok(Self {
            endpoint,
            api_key,
            model,
            timeout: Duration::from_secs(timeout_secs),
            max_concurrency,
        })
    }
}

// ──────────────────────────────────────────────
//  Layout Result (VLM response)
// ──────────────────────────────────────────────

/// Semantic region type returned by VLM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegionType {
    Title,
    Body,
    Table,
    Image,
    Caption,
}

/// Bounding box in page coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// A semantic region identified by VLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub region_type: RegionType,
    pub bbox: BoundingBox,
    pub content: String,
}

/// Complete layout understanding result from VLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutResult {
    /// Semantic regions identified on the page
    pub regions: Vec<Region>,
    /// Reading order (indices into regions)
    pub reading_order: Vec<usize>,
    /// Overall confidence [0.0, 1.0]
    pub confidence: f32,
}

// ──────────────────────────────────────────────
//  VLM Request/Response Payloads
// ──────────────────────────────────────────────

/// Page metadata injected into VLM request
#[derive(Debug, Clone, Serialize)]
pub struct PayloadMetadata {
    pub page_width: f32,
    pub page_height: f32,
    pub page_number: u32,
}

/// VLM request payload
#[derive(Debug, Clone, Serialize)]
pub struct VlmPayload {
    /// Base64-encoded image data
    pub image: String,
    /// Optional hint text (truncated to 10 000 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// Page metadata
    pub metadata: PayloadMetadata,
    /// Target model identifier
    pub model: String,
}

/// Raw VLM response (JSON from remote API)
#[derive(Debug, Deserialize)]
pub(crate) struct VlmResponseRaw {
    pub regions: Vec<RegionRaw>,
    pub reading_order: Vec<usize>,
    pub confidence: f32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegionRaw {
    #[serde(rename = "type")]
    pub region_type: String,
    pub bbox: [f32; 4],
    pub content: String,
}

// ──────────────────────────────────────────────
//  Detector config
// ──────────────────────────────────────────────

/// Configuration for the escalation detector
#[derive(Debug, Clone)]
pub struct DetectorConfig {
    /// Character count threshold below which VLM OCR is triggered (default 0)
    pub zero_text_threshold: u32,
    /// Layout confidence below which VLM layout understanding is triggered (default 0.3)
    pub layout_confidence_threshold: f32,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            zero_text_threshold: 0,
            layout_confidence_threshold: 0.3,
        }
    }
}

// ──────────────────────────────────────────────
//  Pdfium Extraction (input to detector)
// ──────────────────────────────────────────────

/// Pdfium extraction summary used by the escalation detector
#[derive(Debug, Clone)]
pub struct PdfiumExtraction {
    pub character_count: u32,
    pub layout_confidence: f32,
    pub text: String,
    pub page_width: f32,
    pub page_height: f32,
}

// ──────────────────────────────────────────────
//  Degradation
// ──────────────────────────────────────────────

/// Degradation reason code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DegradationReason {
    Timeout,
    Unavailable,
    ParseError,
    RateLimit,
}

/// Degradation record attached to a result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradationRecord {
    pub reason: DegradationReason,
    pub trace_id: String,
    pub message: String,
}
