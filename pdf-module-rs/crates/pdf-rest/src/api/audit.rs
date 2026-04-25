//! Audit log visualization API
//! Provides endpoints for querying and exporting audit logs

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use pdf_core::audit::{AuditFilter, AuditLog};
use pdf_core::control::AuditLogger;
use pdf_core::dto::ExecutionStatus;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Audit API state
#[derive(Clone)]
pub struct AuditState {
    pub logger: Arc<AuditLogger>,
}

/// Audit log query parameters
#[derive(Debug, Deserialize)]
pub struct AuditQueryParams {
    pub tool_name: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub status: Option<String>,
    pub caller: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// Audit log list response
#[derive(Debug, Serialize)]
pub struct AuditListResponse {
    pub logs: Vec<AuditLog>,
    pub page: u32,
    pub page_size: u32,
    pub total: usize,
}

/// Audit statistics response
#[derive(Debug, Serialize)]
pub struct AuditStatsResponse {
    pub total_executions: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub avg_elapsed_ms: u64,
    pub by_tool: HashMap<String, u64>,
}

/// Export query parameters
#[derive(Debug, Deserialize)]
pub struct ExportQueryParams {
    pub format: Option<String>,
    pub tool_name: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

/// Build the audit API router
pub fn build_router(state: AuditState) -> Router {
    Router::new()
        .route("/api/v1/audit-logs", get(query_audit_logs))
        .route("/api/v1/audit-logs/stats", get(get_audit_stats))
        .route("/api/v1/audit-logs/export", get(export_audit_logs))
        .with_state(state)
}

/// GET /api/v1/audit-logs - Query audit logs
async fn query_audit_logs(
    State(state): State<AuditState>,
    Query(params): Query<AuditQueryParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(0);
    let page_size = params.page_size.unwrap_or(20);

    let mut filter = AuditFilter::new();
    if let Some(tool_name) = params.tool_name {
        filter = filter.with_tool_name(tool_name);
    }
    if let Some(status) = params.status {
        let status = match status.as_str() {
            "success" => ExecutionStatus::Success,
            "failed" => ExecutionStatus::Failed,
            "timeout" => ExecutionStatus::Timeout,
            "cancelled" => ExecutionStatus::Cancelled,
            _ => ExecutionStatus::Success,
        };
        filter = filter.with_status(status);
    }
    if let Some(caller) = params.caller {
        filter = filter.with_caller(caller);
    }

    match state.logger.query(&filter, page, page_size).await {
        Ok(logs) => {
            let total = logs.len();
            Json(AuditListResponse {
                logs,
                page,
                page_size,
                total,
            })
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// GET /api/v1/audit-logs/stats - Get audit statistics
async fn get_audit_stats(State(_state): State<AuditState>) -> impl IntoResponse {
    // TODO: Implement actual statistics from audit storage
    let stats = AuditStatsResponse {
        total_executions: 0,
        success_count: 0,
        failure_count: 0,
        avg_elapsed_ms: 0,
        by_tool: HashMap::new(),
    };
    Json(stats)
}

/// GET /api/v1/audit-logs/export - Export audit logs
async fn export_audit_logs(
    State(state): State<AuditState>,
    Query(params): Query<ExportQueryParams>,
) -> impl IntoResponse {
    let format = params.format.unwrap_or_else(|| "json".to_string());

    let filter = AuditFilter::new();
    match state.logger.query(&filter, 0, 10000).await {
        Ok(logs) => match format.as_str() {
            "csv" => {
                let mut csv = String::from("id,timestamp,tool_name,status,elapsed_time_ms,error_message\n");
                for log in &logs {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{}\n",
                        log.id,
                        log.timestamp,
                        log.tool_name,
                        serde_json::to_string(&log.status).unwrap_or_default(),
                        log.elapsed_time_ms,
                        log.error_message.as_deref().unwrap_or("")
                    ));
                }
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "text/csv; charset=utf-8")],
                    csv,
                )
                    .into_response()
            }
            _ => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                serde_json::to_string(&logs).unwrap_or_default(),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
