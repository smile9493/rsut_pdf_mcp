//! Plugin management REST API
//! Provides endpoints for tool management, execution, and monitoring

pub mod audit;
pub mod health;
pub mod metrics;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use pdf_core::plugin::{PluginRegistry, ToolHandler};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<dyn PluginRegistry>,
}

/// Tool info response
#[derive(Debug, Serialize)]
pub struct ToolInfoResponse {
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub category: String,
}

/// Tool list response
#[derive(Debug, Serialize)]
pub struct ToolListResponse {
    pub tools: Vec<ToolInfoResponse>,
    pub total: usize,
}

/// Tool execution request
#[derive(Debug, Deserialize)]
pub struct ToolExecuteRequest {
    pub params: HashMap<String, Value>,
}

/// Tool execution response
#[derive(Debug, Serialize)]
pub struct ToolExecuteResponse {
    pub tool_name: String,
    pub result: Value,
    pub elapsed_time_ms: u64,
}

/// API error response
#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub code: u16,
}

/// Build the plugin management API router
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/tools", get(list_tools))
        .route("/api/v1/tools/{name}", get(get_tool))
        .route("/api/v1/tools/{name}/execute", post(execute_tool))
        .with_state(state)
}

/// GET /api/v1/tools - List all registered tools
async fn list_tools(State(state): State<AppState>) -> impl IntoResponse {
    let definitions = state.registry.list_definitions().await;
    let tools: Vec<ToolInfoResponse> = definitions
        .iter()
        .map(|d| ToolInfoResponse {
            name: d.function_name.clone(),
            description: d.description.clone(),
            version: d.latest_version().to_string(),
            capabilities: d.capabilities.clone(),
            category: "general".to_string(),
        })
        .collect();

    let total = tools.len();
    Json(ToolListResponse { tools, total })
}

/// GET /api/v1/tools/{name} - Get tool details
async fn get_tool(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.registry.get(&name).await {
        Ok(tool) => {
            let info = ToolInfoResponse {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                version: tool.version().to_string(),
                capabilities: tool.capabilities(),
                category: tool.category(),
            };
            (StatusCode::OK, Json(info)).into_response()
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(ApiErrorResponse {
                error: e.to_string(),
                code: 404,
            }),
        )
            .into_response(),
    }
}

/// POST /api/v1/tools/{name}/execute - Execute a tool
async fn execute_tool(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<ToolExecuteRequest>,
) -> impl IntoResponse {
    match state.registry.get(&name).await {
        Ok(tool) => {
            // Validate parameters
            if let Err(e) = tool.validate_params(&body.params) {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiErrorResponse {
                        error: format!("Validation failed: {}", e),
                        code: 400,
                    }),
                )
                    .into_response();
            }

            let start = std::time::Instant::now();

            match tool.execute(body.params, None).await {
                Ok(result) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    Json(ToolExecuteResponse {
                        tool_name: name,
                        result: result.output,
                        elapsed_time_ms: elapsed,
                    })
                    .into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiErrorResponse {
                        error: format!("Execution failed: {}", e),
                        code: 500,
                    }),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(ApiErrorResponse {
                error: e.to_string(),
                code: 404,
            }),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_info_response_serialization() {
        let info = ToolInfoResponse {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["file_input".to_string()],
            category: "extraction".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test_tool"));
    }
}
