//! # pdf-dashboard
//!
//! Lightweight HTTP server for PDF module monitoring dashboard.
//! Exposes real-time metrics, system status, and tool usage via REST API.

use axum::{
    extract::State,
    http::{header, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    stats: Arc<ToolStats>,
    activity_log: Arc<ActivityLog>,
    start_time: u64,
}

#[derive(Default)]
struct ToolStats {
    extract_text_calls: AtomicU64,
    extract_text_latency_ms: AtomicU64,
    extract_text_errors: AtomicU64,
    extract_structured_calls: AtomicU64,
    extract_structured_latency_ms: AtomicU64,
    extract_structured_errors: AtomicU64,
    get_page_count_calls: AtomicU64,
    get_page_count_latency_ms: AtomicU64,
    get_page_count_errors: AtomicU64,
    search_keywords_calls: AtomicU64,
    search_keywords_latency_ms: AtomicU64,
    search_keywords_errors: AtomicU64,
    files_processed: AtomicU64,
}

impl ToolStats {
    fn record(&self, tool: &str, latency_ms: u64, success: bool) {
        match tool {
            "extract_text" => {
                self.extract_text_calls.fetch_add(1, Ordering::Relaxed);
                self.extract_text_latency_ms
                    .fetch_add(latency_ms, Ordering::Relaxed);
                if !success {
                    self.extract_text_errors.fetch_add(1, Ordering::Relaxed);
                }
            }
            "extract_structured" => {
                self.extract_structured_calls.fetch_add(1, Ordering::Relaxed);
                self.extract_structured_latency_ms
                    .fetch_add(latency_ms, Ordering::Relaxed);
                if !success {
                    self.extract_structured_errors.fetch_add(1, Ordering::Relaxed);
                }
            }
            "get_page_count" => {
                self.get_page_count_calls.fetch_add(1, Ordering::Relaxed);
                self.get_page_count_latency_ms
                    .fetch_add(latency_ms, Ordering::Relaxed);
                if !success {
                    self.get_page_count_errors.fetch_add(1, Ordering::Relaxed);
                }
            }
            "search_keywords" => {
                self.search_keywords_calls.fetch_add(1, Ordering::Relaxed);
                self.search_keywords_latency_ms
                    .fetch_add(latency_ms, Ordering::Relaxed);
                if !success {
                    self.search_keywords_errors.fetch_add(1, Ordering::Relaxed);
                }
            }
            _ => {}
        }
        self.files_processed.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Serialize)]
struct ToolStat {
    name: String,
    calls: u64,
    latency: u64,
    success_rate: f64,
}

#[derive(Serialize)]
struct DashboardMetrics {
    total_calls: u64,
    avg_latency_ms: u64,
    success_rate: f64,
    files_processed: u64,
    tools: Vec<ToolStat>,
    uptime_secs: u64,
    start_timestamp: u64,
}

#[derive(Serialize)]
struct SystemStatus {
    memory_percent: f64,
    pdfium_ready: bool,
    pdfium_version: String,
    queue_length: u32,
    vlm_enabled: bool,
    vlm_model: String,
    vlm_thinking: bool,
    vlm_function_call: bool,
    vlm_multi_model_routing: bool,
}

#[derive(Serialize)]
struct HealthCheck {
    status: String,
    mcp_healthy: bool,
    client_connections: u32,
    uptime_secs: u64,
    version: String,
}

#[derive(Clone)]
struct ActivityLog {
    entries: Arc<parking_lot::RwLock<Vec<LogEntry>>>,
}

#[derive(Serialize, Clone)]
struct LogEntry {
    level: String,
    time: String,
    message: String,
}

impl ActivityLog {
    fn new() -> Self {
        Self {
            entries: Arc::new(parking_lot::RwLock::new(Vec::with_capacity(200))),
        }
    }

    fn add(&self, level: &str, message: &str) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let secs = now.as_secs();
        let time = format!(
            "{:02}:{:02}:{:02}",
            (secs / 3600) % 24,
            (secs / 60) % 60,
            secs % 60
        );
        let mut entries = self.entries.write();
        entries.push(LogEntry {
            level: level.to_string(),
            time,
            message: message.to_string(),
        });
        if entries.len() > 200 {
            entries.remove(0);
        }
    }

    fn get(&self) -> Vec<LogEntry> {
        self.entries.read().clone()
    }
}

async fn health(State(state): State<AppState>) -> Json<HealthCheck> {
    Json(HealthCheck {
        status: "ok".to_string(),
        mcp_healthy: true,
        client_connections: 1,
        uptime_secs: current_uptime(&state),
        version: "0.3.0".to_string(),
    })
}

async fn metrics(State(state): State<AppState>) -> Json<DashboardMetrics> {
    let stats = &state.stats;
    let total_calls = stats.extract_text_calls.load(Ordering::Relaxed)
        + stats.extract_structured_calls.load(Ordering::Relaxed)
        + stats.get_page_count_calls.load(Ordering::Relaxed)
        + stats.search_keywords_calls.load(Ordering::Relaxed);

    let total_latency = stats.extract_text_latency_ms.load(Ordering::Relaxed)
        + stats.extract_structured_latency_ms.load(Ordering::Relaxed)
        + stats.get_page_count_latency_ms.load(Ordering::Relaxed)
        + stats.search_keywords_latency_ms.load(Ordering::Relaxed);

    let total_errors = stats.extract_text_errors.load(Ordering::Relaxed)
        + stats.extract_structured_errors.load(Ordering::Relaxed)
        + stats.get_page_count_errors.load(Ordering::Relaxed)
        + stats.search_keywords_errors.load(Ordering::Relaxed);

    let avg_latency = if total_calls > 0 {
        total_latency / total_calls
    } else {
        0
    };

    let success_rate = if total_calls > 0 {
        ((total_calls - total_errors) as f64 / total_calls as f64) * 100.0
    } else {
        100.0
    };

    let files_processed = stats.files_processed.load(Ordering::Relaxed);

    fn tool_stat(
        name: &str,
        calls: u64,
        latency: u64,
        errors: u64,
    ) -> ToolStat {
        let avg = if calls > 0 { latency / calls } else { 0 };
        let rate = if calls > 0 {
            ((calls - errors) as f64 / calls as f64) * 100.0
        } else {
            100.0
        };
        ToolStat {
            name: name.to_string(),
            calls,
            latency: avg,
            success_rate: rate,
        }
    }

    let tools = vec![
        tool_stat(
            "extract_text",
            stats.extract_text_calls.load(Ordering::Relaxed),
            stats.extract_text_latency_ms.load(Ordering::Relaxed),
            stats.extract_text_errors.load(Ordering::Relaxed),
        ),
        tool_stat(
            "extract_structured",
            stats.extract_structured_calls.load(Ordering::Relaxed),
            stats.extract_structured_latency_ms.load(Ordering::Relaxed),
            stats.extract_structured_errors.load(Ordering::Relaxed),
        ),
        tool_stat(
            "get_page_count",
            stats.get_page_count_calls.load(Ordering::Relaxed),
            stats.get_page_count_latency_ms.load(Ordering::Relaxed),
            stats.get_page_count_errors.load(Ordering::Relaxed),
        ),
        tool_stat(
            "search_keywords",
            stats.search_keywords_calls.load(Ordering::Relaxed),
            stats.search_keywords_latency_ms.load(Ordering::Relaxed),
            stats.search_keywords_errors.load(Ordering::Relaxed),
        ),
    ];

    Json(DashboardMetrics {
        total_calls,
        avg_latency_ms: avg_latency,
        success_rate,
        files_processed,
        tools,
        uptime_secs: current_uptime(&state),
        start_timestamp: state.start_time,
    })
}

async fn system_status(State(_state): State<AppState>) -> Json<SystemStatus> {
    let mem = read_memory_usage().unwrap_or(0.0);

    Json(SystemStatus {
        memory_percent: mem,
        pdfium_ready: true,
        pdfium_version: "4.04.0".to_string(),
        queue_length: 0,
        vlm_enabled: std::env::var("VLM_MODEL").is_ok(),
        vlm_model: std::env::var("VLM_MODEL").unwrap_or_else(|_| "none".to_string()),
        vlm_thinking: std::env::var("VLM_ENABLE_THINKING")
            .map(|v| v == "true")
            .unwrap_or(true),
        vlm_function_call: std::env::var("VLM_ENABLE_FUNCTION_CALL")
            .map(|v| v == "true")
            .unwrap_or(false),
        vlm_multi_model_routing: std::env::var("VLM_ENABLE_MULTI_MODEL_ROUTING")
            .map(|v| v != "false")
            .unwrap_or(true),
    })
}

async fn get_activity_log(State(state): State<AppState>) -> Json<Vec<LogEntry>> {
    Json(state.activity_log.get())
}

async fn clear_log(State(state): State<AppState>) -> impl IntoResponse {
    state.activity_log.entries.write().clear();
    (StatusCode::OK, "cleared")
}

async fn record_tool_call(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let tool = body.get("tool").and_then(|v| v.as_str()).unwrap_or("");
    let latency = body.get("latency_ms").and_then(|v| v.as_u64()).unwrap_or(0);
    let success = body.get("success").and_then(|v| v.as_bool()).unwrap_or(true);

    state.stats.record(tool, latency, success);

    let level = if success { "info" } else { "error" };
    let msg = if success {
        format!("{} completed in {}ms", tool, latency)
    } else {
        format!("{} failed after {}ms", tool, latency)
    };
    state.activity_log.add(level, &msg);

    StatusCode::OK
}

fn current_uptime(state: &AppState) -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() - state.start_time)
        .unwrap_or(0)
}

fn read_memory_usage() -> Option<f64> {
    // Read from /proc/self/status on Linux
    #[cfg(target_os = "linux")]
    {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open("/proc/self/status").ok()?;
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<f64>() {
                        let total_mem = total_memory_kb().unwrap_or(1_000_000);
                        return Some((kb / total_mem as f64) * 100.0);
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn total_memory_kb() -> Option<u64> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open("/proc/meminfo").ok()?;
    let reader = BufReader::new(file);
    for line in reader.lines().flatten() {
        if line.starts_with("MemTotal:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1].parse().ok();
            }
        }
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn total_memory_kb() -> Option<u64> {
    None
}

pub async fn run_dashboard(bind: &str) -> anyhow::Result<()> {
    let addr: SocketAddr = bind.parse()?;

    let stats = Arc::new(ToolStats::default());
    let activity_log = Arc::new(ActivityLog::new());

    let start_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let state = AppState {
        stats,
        activity_log: activity_log.clone(),
        start_time,
    };

    activity_log.add("info", "Dashboard server starting");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/metrics", get(metrics))
        .route("/api/status", get(system_status))
        .route("/api/logs", get(get_activity_log))
        .route("/api/logs/clear", post(clear_log))
        .route("/api/record", post(record_tool_call))
        .layer(cors)
        .with_state(state);

    info!("Dashboard listening on http://{}", addr);
    info!("Dashboard endpoints: /api/health, /api/metrics, /api/status, /api/logs");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    activity_log.add("info", "Dashboard server started");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    warn!("Shutdown signal received, stopping dashboard...");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let bind = std::env::var("DASHBOARD_BIND").unwrap_or_else(|_| "0.0.0.0:8000".to_string());

    info!("Starting PDF Module Dashboard Server");
    info!("Bind address: {}", bind);

    run_dashboard(&bind).await
}
