//! Health check endpoints
//! Provides health, liveness, and readiness probes

use axum::{extract::State, response::IntoResponse, routing::get, Json, Router};
use chrono::Utc;
use pdf_core::plugin::PluginRegistry;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

/// Health state
#[derive(Clone)]
pub struct HealthState {
    pub registry: Arc<dyn PluginRegistry>,
}

/// Health status response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub checks: HashMap<String, ComponentHealth>,
}

/// Component health
#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub status: String,
    pub message: Option<String>,
}

/// Build the health check router
pub fn build_router(state: HealthState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/live", get(liveness_probe))
        .route("/health/ready", get(readiness_probe))
        .with_state(state)
}

/// GET /health - Full health check
async fn health_check(State(state): State<HealthState>) -> impl IntoResponse {
    let mut checks = HashMap::new();

    // Check registry
    let tool_count = state.registry.count().await;
    checks.insert(
        "registry".to_string(),
        ComponentHealth {
            status: "healthy".to_string(),
            message: Some(format!("{} tools registered", tool_count)),
        },
    );

    let status = if tool_count > 0 {
        "healthy"
    } else {
        "degraded"
    };

    Json(HealthResponse {
        status: status.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        checks,
    })
}

/// GET /health/live - Liveness probe (is process alive?)
async fn liveness_probe() -> impl IntoResponse {
    Json(HealthResponse {
        status: "alive".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        checks: HashMap::new(),
    })
}

/// GET /health/ready - Readiness probe (can accept requests?)
async fn readiness_probe(State(state): State<HealthState>) -> impl IntoResponse {
    let tool_count = state.registry.count().await;

    let mut checks = HashMap::new();
    checks.insert(
        "tools".to_string(),
        ComponentHealth {
            status: if tool_count > 0 {
                "ready".to_string()
            } else {
                "not_ready".to_string()
            },
            message: Some(format!("{} tools available", tool_count)),
        },
    );

    let status = if tool_count > 0 { "ready" } else { "not_ready" };

    Json(HealthResponse {
        status: status.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        checks,
    })
}
