//! Metrics collector implementation
//! Provides execution metrics collection and Prometheus exposition

use crate::dto::{ExecutionMetric, ExecutionStatus};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;

/// Metrics snapshot
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub success_count: u64,
    /// Failed executions
    pub failure_count: u64,
    /// Timed out executions
    pub timeout_count: u64,
    /// Average execution time in ms
    pub avg_execution_time_ms: u64,
    /// Cache hit count
    pub cache_hits: u64,
    /// Currently active executions
    pub active_executions: u64,
}

/// Per-tool metrics
#[derive(Debug, Clone)]
pub struct ToolMetrics {
    /// Tool name
    pub tool_name: String,
    /// Total execution count
    pub total: u64,
    /// Success count
    pub success: u64,
    /// Failure count
    pub failure: u64,
    /// Total execution time in ms
    pub total_time_ms: u64,
}

/// Metrics collector
/// Collects and exposes execution metrics for monitoring
pub struct MetricsCollector {
    /// Total executions
    total_executions: AtomicU64,
    /// Success count
    success_count: AtomicU64,
    /// Failure count
    failure_count: AtomicU64,
    /// Timeout count
    timeout_count: AtomicU64,
    /// Total execution time
    total_time_ms: AtomicU64,
    /// Cache hits
    cache_hits: AtomicU64,
    /// Active executions
    active_executions: AtomicU64,
    /// Per-tool metrics
    tool_metrics: RwLock<Vec<ToolMetrics>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            total_executions: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            failure_count: AtomicU64::new(0),
            timeout_count: AtomicU64::new(0),
            total_time_ms: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            active_executions: AtomicU64::new(0),
            tool_metrics: RwLock::new(Vec::new()),
        }
    }

    /// Record an execution metric
    pub async fn record_execution(&self, metric: &ExecutionMetric) {
        self.total_executions.fetch_add(1, Ordering::Relaxed);

        match metric.status {
            ExecutionStatus::Success => {
                self.success_count.fetch_add(1, Ordering::Relaxed);
            }
            ExecutionStatus::Failed => {
                self.failure_count.fetch_add(1, Ordering::Relaxed);
            }
            ExecutionStatus::Timeout => {
                self.timeout_count.fetch_add(1, Ordering::Relaxed);
            }
            ExecutionStatus::Cancelled => {}
        }

        let elapsed = (metric.end_time - metric.start_time)
            .num_milliseconds()
            .max(0) as u64;
        self.total_time_ms.fetch_add(elapsed, Ordering::Relaxed);

        // Update per-tool metrics
        let mut tool_metrics = self.tool_metrics.write().await;
        if let Some(tm) = tool_metrics.iter_mut().find(|m| m.tool_name == metric.tool_name) {
            tm.total += 1;
            match metric.status {
                ExecutionStatus::Success => tm.success += 1,
                ExecutionStatus::Failed | ExecutionStatus::Timeout => tm.failure += 1,
                ExecutionStatus::Cancelled => {}
            }
            tm.total_time_ms += elapsed;
        } else {
            let mut tm = ToolMetrics {
                tool_name: metric.tool_name.clone(),
                total: 1,
                success: 0,
                failure: 0,
                total_time_ms: elapsed,
            };
            match metric.status {
                ExecutionStatus::Success => tm.success = 1,
                ExecutionStatus::Failed | ExecutionStatus::Timeout => tm.failure = 1,
                ExecutionStatus::Cancelled => {}
            }
            tool_metrics.push(tm);
        }
    }

    /// Increment cache hit counter
    pub fn increment_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Set active execution count
    pub fn set_active_executions(&self, count: u64) {
        self.active_executions.store(count, Ordering::Relaxed);
    }

    /// Increment active executions
    pub fn increment_active(&self) {
        self.active_executions.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active executions
    pub fn decrement_active(&self) {
        self.active_executions.fetch_sub(1, Ordering::Relaxed);
    }

    /// Get a snapshot of current metrics
    pub async fn snapshot(&self) -> MetricsSnapshot {
        let total = self.total_executions.load(Ordering::Relaxed);
        let total_time = self.total_time_ms.load(Ordering::Relaxed);

        MetricsSnapshot {
            total_executions: total,
            success_count: self.success_count.load(Ordering::Relaxed),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            timeout_count: self.timeout_count.load(Ordering::Relaxed),
            avg_execution_time_ms: total_time.checked_div(total).unwrap_or(0),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            active_executions: self.active_executions.load(Ordering::Relaxed),
        }
    }

    /// Get per-tool metrics
    pub async fn tool_metrics(&self) -> Vec<ToolMetrics> {
        let metrics = self.tool_metrics.read().await;
        metrics.clone()
    }

    /// Get metrics for a specific tool
    pub async fn get_tool_metrics(&self, tool_name: &str) -> Option<ToolMetrics> {
        let metrics = self.tool_metrics.read().await;
        metrics.iter().find(|m| m.tool_name == tool_name).cloned()
    }

    /// Export metrics in Prometheus text format
    pub async fn export_prometheus(&self) -> String {
        let snapshot = self.snapshot().await;
        let tool_metrics = self.tool_metrics().await;

        let mut output = String::new();

        // Global metrics
        output.push_str("# HELP tool_execution_total Total number of tool executions\n");
        output.push_str("# TYPE tool_execution_total counter\n");
        output.push_str(&format!(
            "tool_execution_total {}\n\n",
            snapshot.total_executions
        ));

        output.push_str("# HELP tool_success_total Total successful executions\n");
        output.push_str("# TYPE tool_success_total counter\n");
        output.push_str(&format!(
            "tool_success_total {}\n\n",
            snapshot.success_count
        ));

        output.push_str("# HELP tool_failure_total Total failed executions\n");
        output.push_str("# TYPE tool_failure_total counter\n");
        output.push_str(&format!(
            "tool_failure_total {}\n\n",
            snapshot.failure_count
        ));

        output.push_str("# HELP tool_avg_duration_ms Average execution duration\n");
        output.push_str("# TYPE tool_avg_duration_ms gauge\n");
        output.push_str(&format!(
            "tool_avg_duration_ms {}\n\n",
            snapshot.avg_execution_time_ms
        ));

        output.push_str("# HELP tool_active_executions Currently active executions\n");
        output.push_str("# TYPE tool_active_executions gauge\n");
        output.push_str(&format!(
            "tool_active_executions {}\n\n",
            snapshot.active_executions
        ));

        // Per-tool metrics
        for tm in &tool_metrics {
            let avg = tm.total_time_ms.checked_div(tm.total).unwrap_or(0);
            output.push_str(&format!(
                "tool_execution_total{{tool=\"{}\"}} {}\n",
                tm.tool_name, tm.total
            ));
            output.push_str(&format!(
                "tool_success_total{{tool=\"{}\"}} {}\n",
                tm.tool_name, tm.success
            ));
            output.push_str(&format!(
                "tool_failure_total{{tool=\"{}\"}} {}\n",
                tm.tool_name, tm.failure
            ));
            output.push_str(&format!(
                "tool_avg_duration_ms{{tool=\"{}\"}} {}\n",
                tm.tool_name, avg
            ));
        }

        output
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        self.total_executions.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.timeout_count.store(0, Ordering::Relaxed);
        self.total_time_ms.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.active_executions.store(0, Ordering::Relaxed);
        self.tool_metrics.write().await.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        let metric = ExecutionMetric {
            tool_name: "test_tool".to_string(),
            execution_id: "exec-1".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::milliseconds(100),
            status: ExecutionStatus::Success,
            error_message: None,
        };

        collector.record_execution(&metric).await;

        let snapshot = collector.snapshot().await;
        assert_eq!(snapshot.total_executions, 1);
        assert_eq!(snapshot.success_count, 1);
    }

    #[tokio::test]
    async fn test_metrics_prometheus_export() {
        let collector = MetricsCollector::new();

        let metric = ExecutionMetric {
            tool_name: "test_tool".to_string(),
            execution_id: "exec-1".to_string(),
            start_time: Utc::now(),
            end_time: Utc::now() + chrono::Duration::milliseconds(50),
            status: ExecutionStatus::Success,
            error_message: None,
        };

        collector.record_execution(&metric).await;

        let output = collector.export_prometheus().await;
        assert!(output.contains("tool_execution_total 1"));
        assert!(output.contains("tool_success_total 1"));
    }
}
