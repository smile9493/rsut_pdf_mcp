//! Performance tests for the MCP Plugin Architecture
//! Measures throughput and latency of key operations

use async_trait::async_trait;
use pdf_core::{
    control::{CircuitBreaker, CircuitBreakerConfig, MetricsCollector, RateLimiter},
    database::SurrealStore,
    dto::*,
    plugin::{ToolHandler, ToolRegistry},
    RuntimeVariables, ServerConfig, ToolDefinition, ToolExecutionResult, ToolSpec,
};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

const TEST_PDF: &str = "/opt/pdf-module/深入理解Nginx.PDF";

/// Measure PDF extraction performance
#[tokio::test]
async fn perf_pdf_extraction() {
    let pdf_path = PathBuf::from(TEST_PDF);
    let config = ServerConfig::default();
    let service = pdf_core::PdfExtractorService::new(&config).unwrap();

    // Warm up
    let _ = service.extract_text(&pdf_path, None).await.unwrap();

    // Measure 5 extractions
    let mut times = Vec::new();
    for _ in 0..5 {
        let start = Instant::now();
        let result = service.extract_text(&pdf_path, None).await.unwrap();
        let elapsed = start.elapsed();
        times.push(elapsed.as_millis());
        assert!(!result.extracted_text.is_empty());
    }

    let avg: u128 = times.iter().sum::<u128>() / times.len() as u128;
    let min = times.iter().min().unwrap();
    let max = times.iter().max().unwrap();

    println!("[PERF] PDF extraction: avg={}ms, min={}ms, max={}ms", avg, min, max);
    assert!(avg < 10000, "Average extraction time should be under 10s");
}

/// Measure plugin registry performance
#[tokio::test]
async fn perf_plugin_registry() {
    let registry = Arc::new(ToolRegistry::new());

    // Register 100 tools
    let start = Instant::now();
    for i in 0..100 {
        let tool = Arc::new(MockTool::new(&format!("tool_{}", i), "test"));
        registry.register(tool).await.unwrap();
    }
    let register_time = start.elapsed();
    println!("[PERF] Register 100 tools: {}ms", register_time.as_millis());

    // Query tools
    let start = Instant::now();
    for i in 0..100 {
        let _ = registry.get(&format!("tool_{}", i)).await.unwrap();
    }
    let query_time = start.elapsed();
    println!("[PERF] Query 100 tools: {}ms", query_time.as_millis());

    // List all tools
    let start = Instant::now();
    let definitions = registry.list_definitions().await;
    let list_time = start.elapsed();
    assert_eq!(definitions.len(), 100);
    println!("[PERF] List 100 tools: {}ms", list_time.as_millis());

    assert!(register_time.as_millis() < 5000, "Registration should be fast");
}

/// Measure SurrealDB write performance
#[tokio::test]
async fn perf_surrealdb_writes() {
    let store = SurrealStore::with_defaults().await.unwrap();

    // Write 100 records
    let start = Instant::now();
    for i in 0..100 {
        let data = serde_json::json!({
            "index": i,
            "data": format!("test data {}", i),
            "nested": {"value": i * 2}
        });
        store.save_etl_result("perf_test", &format!("record_{}", i), data).await.unwrap();
    }
    let write_time = start.elapsed();
    let writes_per_sec = 100_000.0 / write_time.as_secs_f64();
    println!("[PERF] SurrealDB 100 writes: {}ms ({:.0} writes/sec)", write_time.as_millis(), writes_per_sec);

    // Read 100 records
    let start = Instant::now();
    for i in 0..100 {
        let _ = store.get(&format!("perf_test:record_{}", i)).await.unwrap();
    }
    let read_time = start.elapsed();
    let reads_per_sec = 100_000.0 / read_time.as_secs_f64();
    println!("[PERF] SurrealDB 100 reads: {}ms ({:.0} reads/sec)", read_time.as_millis(), reads_per_sec);

    // Query all records
    let start = Instant::now();
    let results = store.query("perf_test", "").await.unwrap();
    let query_time = start.elapsed();
    assert_eq!(results.len(), 100);
    println!("[PERF] SurrealDB query 100 records: {}ms", query_time.as_millis());
}

/// Measure control plane performance
#[tokio::test]
async fn perf_control_plane() {
    // Rate limiter
    let rate_limiter = RateLimiter::new();
    rate_limiter.configure("perf_tool".to_string(), RateLimitConfig {
        requests_per_second: 10000,
        burst_size: 1000,
    });

    let start = Instant::now();
    let mut allowed = 0;
    for _ in 0..10000 {
        if rate_limiter.check("perf_tool") {
            allowed += 1;
        }
    }
    let rate_time = start.elapsed();
    println!("[PERF] Rate limiter 10k checks: {}ms ({} allowed)", rate_time.as_millis(), allowed);

    // Circuit breaker
    let cb = CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 100,
        success_threshold: 10,
        timeout_ms: 1000,
    });

    let start = Instant::now();
    for _ in 0..10000 {
        if cb.allow_call() {
            cb.record_success();
        }
    }
    let cb_time = start.elapsed();
    println!("[PERF] Circuit breaker 10k checks: {}ms", cb_time.as_millis());

    // Metrics
    let metrics = MetricsCollector::new();
    let now = chrono::Utc::now();
    let metric = ExecutionMetric {
        tool_name: "perf_tool".to_string(),
        execution_id: "perf".to_string(),
        start_time: now,
        end_time: now,
        status: ExecutionStatus::Success,
        error_message: None,
    };

    let start = Instant::now();
    for _ in 0..10000 {
        metrics.record_execution(&metric).await;
    }
    let metrics_time = start.elapsed();
    println!("[PERF] Metrics 10k records: {}ms", metrics_time.as_millis());

    let snapshot = metrics.snapshot().await;
    assert_eq!(snapshot.total_executions, 10000);
}

// ============================================================
// Mock Tool for Testing
// ============================================================
struct MockTool {
    definition: ToolDefinition,
    spec: ToolSpec,
    variables: RuntimeVariables,
}

impl MockTool {
    fn new(name: &str, _category: &str) -> Self {
        let definition = ToolDefinition::new(
            format!("{} Display", name),
            name.to_string(),
            format!("{} description", name),
            vec![],
            InputType::File,
            OutputType::File,
        );
        let spec = ToolSpec::new(name.to_string(), "1.0.0".to_string());
        let variables = RuntimeVariables::new("Test".to_string(), "Test".to_string());
        Self { definition, spec, variables }
    }
}

#[async_trait]
impl ToolHandler for MockTool {
    fn definition(&self) -> &ToolDefinition { &self.definition }
    fn spec(&self) -> &ToolSpec { &self.spec }
    fn variables(&self) -> &RuntimeVariables { &self.variables }

    async fn execute(
        &self,
        _params: HashMap<String, Value>,
        _streamer: Option<&mut dyn pdf_core::MessageStreamer>,
    ) -> pdf_core::PdfResult<ToolExecutionResult> {
        Ok(ToolExecutionResult {
            workflow_id: "perf-test".to_string(),
            elapsed_time: 1,
            output: serde_json::json!({"result": "ok"}),
            metadata: Some(ExecutionMetadata {
                file_name: "test.pdf".to_string(),
                file_size: 1024,
                processing_time: 1,
                cache_hit: false,
                adapter_used: "test".to_string(),
            }),
        })
    }

    fn validate_params(&self, _params: &HashMap<String, Value>) -> pdf_core::PdfResult<()> { Ok(()) }

    fn metadata(&self) -> ExecutionMetadata {
        ExecutionMetadata {
            file_name: "test.pdf".to_string(),
            file_size: 1024,
            processing_time: 1,
            cache_hit: false,
            adapter_used: "test".to_string(),
        }
    }
}
