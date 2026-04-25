//! Prometheus metrics endpoint
//! Exposes metrics in Prometheus text format

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use pdf_core::control::MetricsCollector;
use std::sync::Arc;

/// Metrics state
#[derive(Clone)]
pub struct MetricsState {
    pub collector: Arc<MetricsCollector>,
}

/// Build the metrics router
pub fn build_router(state: MetricsState) -> Router {
    Router::new()
        .route("/metrics", get(get_metrics))
        .with_state(state)
}

/// GET /metrics - Prometheus metrics
async fn get_metrics(State(state): State<MetricsState>) -> impl IntoResponse {
    let output = state.collector.export_prometheus().await;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        output,
    )
        .into_response()
}
