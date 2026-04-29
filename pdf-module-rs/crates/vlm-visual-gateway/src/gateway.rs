use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tokio::sync::{broadcast, Semaphore};
use tokio::time::timeout;
use tracing::{info, warn};
use uuid::Uuid;

use crate::error::{VlmError, VlmResult};
use crate::metrics::MetricsCollector;
use crate::types::{
    LayoutResult, PayloadMetadata, Region, RegionType,
    VlmConfig, VlmPayload, VlmResponseRaw,
};

/// VLM Gateway — the core component that dispatches visual perception
/// requests to a remote VLM API and returns layout understanding results.
///
/// **Design**: stateless, clone-friendly (all fields behind `Arc`), non-blocking.
pub struct VlmGateway {
    client: Client,
    config: Arc<VlmConfig>,
    metrics: Arc<MetricsCollector>,
    semaphore: Arc<Semaphore>,
    shutdown_tx: broadcast::Sender<()>,
}

impl Clone for VlmGateway {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: Arc::clone(&self.config),
            metrics: Arc::clone(&self.metrics),
            semaphore: Arc::clone(&self.semaphore),
            shutdown_tx: self.shutdown_tx.clone(),
        }
    }
}

impl VlmGateway {
    /// Create a new gateway from config + shared metrics collector.
    pub fn new(config: VlmConfig, metrics: Arc<MetricsCollector>) -> VlmResult<Self> {
        let client = Client::builder()
            .timeout(config.timeout + Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| VlmError::Config(format!("reqwest client build: {e}")))?;

        let semaphore = Arc::new(Semaphore::new(config.max_concurrency));
        let (shutdown_tx, _) = broadcast::channel(1);

        Ok(Self {
            client,
            config: Arc::new(config),
            metrics,
            semaphore,
            shutdown_tx,
        })
    }

    /// Create from environment variables.
    pub fn from_env(metrics: Arc<MetricsCollector>) -> VlmResult<Self> {
        let config = VlmConfig::from_env()?;
        Self::new(config, metrics)
    }

    // ─── Core public method ───────────────────────────

    /// Perceive layout understanding for a single page.
    ///
    /// `image_data` — raw RGBA pixel bytes (will be Base64-encoded internally).
    /// `hint_text`  — optional Pdfium-extracted text fragments.
    /// `metadata`   — page dimensions and page number.
    pub async fn perceive_layout(
        &self,
        image_data: &[u8],
        hint_text: Option<&str>,
        metadata: &PayloadMetadata,
    ) -> VlmResult<LayoutResult> {
        let trace_id = Uuid::new_v4().to_string();
        let provider = self.config.model.model_id();

        // 1. Validate input
        if image_data.is_empty() {
            return Err(VlmError::InvalidImage("image_data is empty".into()));
        }

        // 2. Encode raw RGBA pixels to Base64
        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, image_data);

        // 3. Truncate hint
        let hint = hint_text.map(|h| {
            if h.len() > 10_000 {
                warn!(trace_id = %trace_id, "hint_text truncated from {} to 10000 chars", h.len());
                h[..10_000].to_string()
            } else {
                h.to_string()
            }
        });

        // 4. Build payload
        let payload = VlmPayload {
            image: b64,
            hint,
            metadata: PayloadMetadata {
                page_width: metadata.page_width,
                page_height: metadata.page_height,
                page_number: metadata.page_number,
            },
            model: provider.to_string(),
        };

        // 5. Check shutdown signal
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        tokio::select! {
            _ = shutdown_rx.recv() => {
                warn!(trace_id = %trace_id, "VLM request cancelled: shutdown signal");
                return Err(VlmError::Unavailable("shutdown".into()));
            }
            result = self.dispatch_request(&payload, &trace_id) => {
                result
            }
        }
    }

    /// Non-blocking fire-and-forget dispatch.
    ///
    /// The main pipeline returns immediately with the local Pdfium result.
    /// The VLM result is sent through `result_tx` when ready.
    pub fn spawn_perceive_layout(
        &self,
        image_data: Vec<u8>,
        hint_text: Option<String>,
        metadata: PayloadMetadata,
        result_tx: tokio::sync::oneshot::Sender<VlmResult<LayoutResult>>,
    ) {
        let gateway = self.clone();

        tokio::spawn(async move {
            let result = gateway
                .perceive_layout(&image_data, hint_text.as_deref(), &metadata)
                .await;
            let _ = result_tx.send(result);
        });
    }

    /// Health check — lightweight GET to the endpoint.
    pub async fn health_check(&self) -> bool {
        self.client
            .get(&self.config.endpoint)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success() || r.status().as_u16() == 404)
            .unwrap_or(false)
    }

    /// Create a handle for graceful shutdown.
    pub fn handle(&self) -> VlmGatewayHandle {
        VlmGatewayHandle {
            shutdown_tx: self.shutdown_tx.clone(),
        }
    }

    // ─── Private helpers ──────────────────────────────

    async fn dispatch_request(
        &self,
        payload: &VlmPayload,
        trace_id: &str,
    ) -> VlmResult<LayoutResult> {
        // Rate-limit via semaphore
        let permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| VlmError::Unavailable("semaphore closed".into()))?;

        let timer = self.metrics.start_request_timer();
        let provider = self.config.model.model_id().to_string();

        let result = timeout(self.config.timeout, self.send_request(payload)).await;

        drop(permit);

        match result {
            Ok(Ok(response)) => {
                let layout = self.parse_response(&response)?;
                timer.observe_success(&provider);
                info!(trace_id = %trace_id, "VLM perceive_layout succeeded");
                Ok(layout)
            }
            Ok(Err(e)) => {
                let reason = match &e {
                    VlmError::RateLimit => "rate_limit",
                    VlmError::Unavailable(_) => "unavailable",
                    _ => "network_error",
                };
                timer.observe_error(&provider);
                self.metrics.record_degradation(reason);
                warn!(trace_id = %trace_id, error = %e, "VLM request failed - degrading");
                Err(e)
            }
            Err(_) => {
                timer.observe_timeout(&provider);
                self.metrics.record_degradation("timeout");
                warn!(trace_id = %trace_id, "VLM request timed out - degrading");
                Err(VlmError::Timeout(self.config.timeout.as_secs()))
            }
        }
    }

    async fn send_request(&self, payload: &VlmPayload) -> VlmResult<String> {
        let resp = self
            .client
            .post(&self.config.endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(payload)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    VlmError::Timeout(self.config.timeout.as_secs())
                } else if e.is_connect() {
                    VlmError::Unavailable(e.to_string())
                } else {
                    VlmError::Network(e.to_string())
                }
            })?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return if status.as_u16() == 429 {
                Err(VlmError::RateLimit)
            } else if status.as_u16() >= 500 {
                Err(VlmError::Unavailable(format!("HTTP {status}: {body}")))
            } else {
                Err(VlmError::Network(format!("HTTP {status}: {body}")))
            };
        }

        resp.text()
            .await
            .map_err(|e| VlmError::Network(e.to_string()))
    }

    fn parse_response(&self, body: &str) -> VlmResult<LayoutResult> {
        let raw: VlmResponseRaw =
            serde_json::from_str(body).map_err(|e| VlmError::ParseError(e.to_string()))?;

        let regions: Vec<Region> = raw
            .regions
            .into_iter()
            .filter_map(|r| {
                let region_type = match r.region_type.as_str() {
                    "title" => RegionType::Title,
                    "body" => RegionType::Body,
                    "table" => RegionType::Table,
                    "image" => RegionType::Image,
                    "caption" => RegionType::Caption,
                    _ => return None,
                };
                if r.bbox.len() != 4 {
                    return None;
                }
                Some(Region {
                    region_type,
                    bbox: crate::types::BoundingBox {
                        x: r.bbox[0],
                        y: r.bbox[1],
                        width: r.bbox[2],
                        height: r.bbox[3],
                    },
                    content: r.content,
                })
            })
            .collect();

        Ok(LayoutResult {
            regions,
            reading_order: raw.reading_order,
            confidence: raw.confidence,
        })
    }

    pub fn config(&self) -> &VlmConfig {
        &self.config
    }
}

/// Handle for graceful shutdown of the VLM gateway.
///
/// Dropping this handle or calling `shutdown()` sends a shutdown signal
/// to all in-flight `perceive_layout` calls.
pub struct VlmGatewayHandle {
    shutdown_tx: broadcast::Sender<()>,
}

impl VlmGatewayHandle {
    /// Send shutdown signal. In-flight requests will observe the signal
    /// at their next `tokio::select!` checkpoint and degrade gracefully.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

impl Drop for VlmGatewayHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Render a PDF page via Pdfium and return raw RGBA pixels + dimensions.
///
/// The caller owns the pixel data and can encode to Base64 as needed.
/// No temporary files are produced — pdfium renders directly into memory.
pub fn render_page_pixels(
    pdf_data: &[u8],
    page_index: u32,
    dpi: f32,
) -> VlmResult<(Vec<u8>, u32, u32)> {
    let pdf_data = pdf_data.to_vec();
    crate::pdfium_guard::catch_pdfium(move || {
        use pdfium_render::prelude::*;
        let pdfium = Pdfium::default();
        let document = pdfium
            .load_pdf_from_byte_slice(&pdf_data, None)
            .map_err(|e| format!("load PDF: {e}"))?;

        let pages = document.pages();
        let idx: u16 = page_index
            .try_into()
            .map_err(|_| format!("page_index {page_index} exceeds u16"))?;
        let page = pages
            .get(idx)
            .map_err(|e| format!("get page {page_index}: {e}"))?;

        // PDF points are 1/72 inch, so scale = dpi / 72.0
        let scale = dpi / 72.0;
        let config = PdfRenderConfig::new().scale_page_by_factor(scale);

        let bitmap = page
            .render_with_config(&config)
            .map_err(|e| format!("render page: {e}"))?;

        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;
        let rgba = bitmap.as_raw_bytes();

        Ok::<_, String>((rgba, width, height))
    })
    .map_err(|e| VlmError::InvalidImage(format!("Pdfium guard: {e}")))?
    .map_err(VlmError::InvalidImage)
}
