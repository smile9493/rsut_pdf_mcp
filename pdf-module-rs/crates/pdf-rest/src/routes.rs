//! REST API routes

use axum::{
    extract::{Multipart, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use pdf_core::{
    dto::{AdapterInfo, CacheStats, ExtractOptions, StructuredExtractionResult},
    PdfExtractorService,
};
use std::sync::Arc;
use tempfile::NamedTempFile;

/// Build the REST API router
pub fn build_router(service: Arc<PdfExtractorService>) -> Router {
    Router::new()
        .route("/api/v1/x2text/health", get(health))
        .route("/api/v1/x2text/extract", post(extract_text_file))
        .route("/api/v1/x2text/extract-json", post(extract_structured_json))
        .route("/api/v1/x2text/info", post(get_pdf_info))
        .route("/api/v1/x2text/adapters", get(list_adapters))
        .route("/api/v1/x2text/cache/stats", get(cache_stats))
        .with_state(service)
}

/// GET /api/v1/x2text/health
async fn health() -> &'static str {
    "OK"
}

/// POST /api/v1/x2text/extract - Returns text file
async fn extract_text_file(
    State(service): State<Arc<PdfExtractorService>>,
    mut multipart: Multipart,
) -> Result<Response, ApiError> {
    let (file_path, adapter) = parse_multipart(&mut multipart).await?;

    let result = service.extract_text(&file_path, adapter.as_deref()).await?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"extracted.txt\"",
        )
        .body(result.extracted_text.into())
        .map_err(|e| ApiError::Internal(format!("Failed to build response: {}", e)))
}

/// POST /api/v1/x2text/extract-json - Returns JSON
async fn extract_structured_json(
    State(service): State<Arc<PdfExtractorService>>,
    mut multipart: Multipart,
) -> Result<Json<StructuredExtractionResult>, ApiError> {
    let (file_path, adapter) = parse_multipart(&mut multipart).await?;

    let result = service
        .extract_structured(&file_path, adapter.as_deref(), &ExtractOptions::default())
        .await?;

    Ok(Json(result))
}

/// POST /api/v1/x2text/info - Returns page count info
async fn get_pdf_info(
    State(service): State<Arc<PdfExtractorService>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (file_path, _) = parse_multipart(&mut multipart).await?;

    let page_count = service.get_page_count(&file_path).await?;
    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(Json(serde_json::json!({
        "filename": filename,
        "page_count": page_count,
        "mime_type": "application/pdf"
    })))
}

/// GET /api/v1/x2text/adapters
async fn list_adapters(State(service): State<Arc<PdfExtractorService>>) -> Json<serde_json::Value> {
    let adapters: Vec<AdapterInfo> = service.list_engines();
    Json(serde_json::json!({ "adapters": adapters }))
}

/// GET /api/v1/x2text/cache/stats
async fn cache_stats(State(service): State<Arc<PdfExtractorService>>) -> Json<CacheStats> {
    Json(service.cache_stats())
}

/// Parse multipart form data and return temp file path and optional adapter
async fn parse_multipart(
    multipart: &mut Multipart,
) -> Result<(std::path::PathBuf, Option<String>), ApiError> {
    use std::io::Write;

    let mut file_path = None;
    let mut adapter = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to parse multipart: {}", e)))?
    {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "file" => {
                // Save uploaded file to temp location
                let data = field.bytes().await.map_err(|e| {
                    ApiError::BadRequest(format!("Failed to read file data: {}", e))
                })?;

                let mut temp_file = NamedTempFile::new().map_err(|e| {
                    ApiError::Internal(format!("Failed to create temp file: {}", e))
                })?;

                temp_file
                    .write_all(&data)
                    .map_err(|e| ApiError::Internal(format!("Failed to write temp file: {}", e)))?;

                // Keep the temp file and get its path
                let (_file, path) = temp_file
                    .keep()
                    .map_err(|e| ApiError::Internal(format!("Failed to keep temp file: {}", e)))?;

                file_path = Some(path);
            }
            "adapter" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("Failed to read adapter: {}", e)))?;
                adapter = Some(text);
            }
            _ => {}
        }
    }

    let path = file_path.ok_or_else(|| ApiError::BadRequest("No file uploaded".to_string()))?;
    Ok((path, adapter))
}

/// API Error type
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": status.canonical_reason().unwrap_or("Error"),
            "message": message,
            "status_code": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

impl From<pdf_core::PdfModuleError> for ApiError {
    fn from(err: pdf_core::PdfModuleError) -> Self {
        match err.status_code() {
            404 => ApiError::NotFound(err.to_string()),
            400 => ApiError::BadRequest(err.to_string()),
            _ => ApiError::Internal(err.to_string()),
        }
    }
}
