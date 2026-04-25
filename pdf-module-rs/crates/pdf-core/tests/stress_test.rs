//! Stress tests for the MCP Plugin Architecture
//! Tests system behavior under high concurrency and large data volumes

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

/// Stress test: concurrent PDF extractions
#[tokio::test]
async fn stress_concurrent_pdf_extraction() {
    let pdf_path = PathBuf::from(TEST_PDF);
    let config = ServerConfig::default();
    let service = Arc::new(pdf_core::PdfExtractorService::new(&config).unwrap());

    // Warm up
    let _ = service.extract_text(&pdf_path, None).await.unwrap();

    // 10 concurrent extractions
    let start = Instant::now();
    let mut handles = Vec::new();
    for _ in 0..10 {
        let svc = service.clone();
        let path = pdf_path.clone();
        handles.push(tokio::spawn(async move {
            svc.extract_text(&path, None).await
        }));
    }

    let mut successes = 0;
    for handle in handles {
        if handle.await.unwrap().is_ok() {
            successes += 1;
        }
    }
    let elapsed = start.elapsed();

    println!("[STRESS] 10 concurrent PDF extractions: {}ms ({} successes)", elapsed.as_millis(), successes);
    assert!(successes >= 8, "At least 8 of 10 concurrent extractions should succeed");
}

/// Stress test: concurrent SurrealDB writes
#[tokio::test]
async fn stress_concurrent_db_writes() {
    let store = Arc::new(SurrealStore::with_defaults().await.unwrap());

    // 50 concurrent writes
    let start = Instant::now();
    let mut handles = Vec::new();
    for i in 0..50 {
        let s = store.clone();
        handles.push(tokio::spawn(async move {
            let data = serde_json::json!({
                "concurrent_id": i,
                "data": format!("concurrent data {}", i),
                "nested": {"value": i * 3}
            });
            s.save_etl_result("stress_test", &format!("concurrent_{}", i), data).await
        }));
    }

    let mut successes = 0;
    for handle in handles {
        if handle.await.unwrap().is_ok() {
            successes += 1;
        }
    }
    let elapsed = start.elapsed();

    println!("[STRESS] 50 concurrent DB writes: {}ms ({} successes)", elapsed.as_millis(), successes);
    assert!(successes >= 45, "At least 45 of 50 concurrent writes should succeed");

    // Verify data integrity
    let results = store.query("stress_test", "").await.unwrap();
    assert!(results.len() >= 45, "Should have at least 45 records in DB");
}

/// Stress test: high-volume plugin registry operations
#[tokio::test]
async fn stress_plugin_registry_volume() {
    let registry = Arc::new(ToolRegistry::new());

    // Register 500 tools
    let start = Instant::now();
    for i in 0..500 {
        let tool = Arc::new(MockTool::new(&format!("stress_tool_{}", i), "stress"));
        registry.register(tool).await.unwrap();
    }
    let register_time = start.elapsed();
    assert_eq!(registry.count().await, 500);
    println!("[STRESS] Register 500 tools: {}ms", register_time.as_millis());

    // Concurrent reads
    let start = Instant::now();
    let mut handles = Vec::new();
    for i in 0..100 {
        let r = registry.clone();
        handles.push(tokio::spawn(async move {
            r.get(&format!("stress_tool_{}", i)).await.is_ok()
        }));
    }
    let mut read_successes = 0;
    for handle in handles {
        if handle.await.unwrap() {
            read_successes += 1;
        }
    }
    let read_time = start.elapsed();
    println!("[STRESS] 100 concurrent reads: {}ms ({} successes)", read_time.as_millis(), read_successes);
    assert_eq!(read_successes, 100);

    // Unregister all
    let start = Instant::now();
    for i in 0..500 {
        registry.unregister(&format!("stress_tool_{}", i)).await.unwrap();
    }
    let unregister_time = start.elapsed();
    assert_eq!(registry.count().await, 0);
    println!("[STRESS] Unregister 500 tools: {}ms", unregister_time.as_millis());
}

/// Stress test: circuit breaker under sustained failures
#[tokio::test]
async fn stress_circuit_breaker_sustained() {
    let cb = Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 10,
        success_threshold: 5,
        timeout_ms: 100,
    }));

    // Sustained failures to trip the breaker
    let mut failures = 0;
    let mut rejected = 0;
    for _ in 0..100 {
        if cb.allow_call() {
            cb.record_failure();
            failures += 1;
        } else {
            rejected += 1;
        }
    }
    println!("[STRESS] Circuit breaker: {} failures, {} rejected", failures, rejected);
    assert!(rejected > 0, "Circuit breaker should reject some calls after failures");

    // Wait for timeout and try recovery
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    assert!(cb.allow_call(), "Circuit breaker should allow call after timeout");

    // Recovery phase
    cb.record_success();
    cb.record_success();
    cb.record_success();
    cb.record_success();
    cb.record_success();
    // After 5 successes, should be back to closed
    assert!(cb.allow_call());
}

/// Stress test: rate limiter burst handling
#[tokio::test]
async fn stress_rate_limiter_burst() {
    let rl = Arc::new(RateLimiter::new());
    rl.configure("burst_tool".to_string(), RateLimitConfig {
        requests_per_second: 100,
        burst_size: 50,
    });

    // Send 200 requests rapidly
    let mut allowed = 0;
    let mut rejected = 0;
    for _ in 0..200 {
        if rl.check("burst_tool") {
            allowed += 1;
        } else {
            rejected += 1;
        }
    }
    println!("[STRESS] Rate limiter burst: {} allowed, {} rejected", allowed, rejected);
    assert!(allowed >= 50, "At least burst_size requests should be allowed");
    assert!(rejected > 0, "Some requests should be rejected after burst");
}

/// Stress test: metrics under high volume
#[tokio::test]
async fn stress_metrics_high_volume() {
    let metrics = Arc::new(MetricsCollector::new());
    let now = chrono::Utc::now();

    // Record 50,000 metrics
    let start = Instant::now();
    for i in 0..50000 {
        let metric = ExecutionMetric {
            tool_name: if i % 2 == 0 { "tool_a" } else { "tool_b" }.to_string(),
            execution_id: format!("exec_{}", i),
            start_time: now,
            end_time: now,
            status: if i % 10 == 0 { ExecutionStatus::Failed } else { ExecutionStatus::Success },
            error_message: if i % 10 == 0 { Some("error".to_string()) } else { None },
        };
        metrics.record_execution(&metric).await;
    }
    let record_time = start.elapsed();

    let snapshot = metrics.snapshot().await;
    assert_eq!(snapshot.total_executions, 50000);

    // Export Prometheus format
    let start = Instant::now();
    let prom_output = metrics.export_prometheus().await;
    let export_time = start.elapsed();

    println!("[STRESS] 50k metrics records: {}ms", record_time.as_millis());
    println!("[STRESS] Prometheus export: {}ms ({} bytes)", export_time.as_millis(), prom_output.len());
    assert!(!prom_output.is_empty());
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
            workflow_id: "stress-test".to_string(),
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
