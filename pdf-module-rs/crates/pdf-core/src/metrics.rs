//! Prometheus metrics definitions
//! Provides observability for PDF extraction operations
//!
//! Uses the `metrics` crate for Prometheus-compatible metrics.

/// Initialize Prometheus metrics exporter
pub fn init_metrics() {
    // Metrics are registered lazily when first used
    // This function can be used for any setup if needed
    tracing::info!("Metrics system initialized");
}

/// Metrics definitions for PDF module
pub mod metrics_def {
    use metrics::{counter, gauge, histogram};

    /// Record extraction duration in milliseconds
    /// Label: engine = lopdf | pdf-extract | pdfium
    pub fn extraction_duration_ms(engine: &str, duration_ms: f64) {
        histogram!("pdf_extraction_duration_ms", "engine" => engine.to_string())
            .record(duration_ms);
    }

    /// Record extraction total count
    /// Labels: engine, result = success | failure
    pub fn extraction_total(engine: &str, result: &str) {
        counter!("pdf_extraction_total", "engine" => engine.to_string(), "result" => result.to_string())
            .increment(1);
    }

    /// Record cache hit rate
    pub fn cache_hit_rate(hit_rate: f64) {
        gauge!("pdf_cache_hit_rate").set(hit_rate);
    }

    /// Record cache operation count
    /// Label: operation = hit | miss | eviction
    pub fn cache_operations(operation: &str) {
        counter!("pdf_cache_operations", "operation" => operation.to_string())
            .increment(1);
    }

    /// Record smart router distribution
    /// Label: engine = lopdf | pdf-extract | pdfium
    pub fn route_distribution(engine: &str) {
        counter!("pdf_route_distribution", "engine" => engine.to_string())
            .increment(1);
    }

    /// Record circuit breaker state
    /// Labels: engine, state = closed | open | half_open
    pub fn circuit_breaker_state(engine: &str, state: &str) {
        gauge!("pdf_circuit_breaker_state", "engine" => engine.to_string())
            .set(match state {
                "closed" => 0.0,
                "half_open" => 1.0,
                "open" => 2.0,
                _ => -1.0,
            });
    }

    /// Record file size in MB
    pub fn file_size_mb(size_mb: f64) {
        histogram!("pdf_file_size_mb").record(size_mb);
    }

    /// Record keyword search duration in milliseconds
    pub fn keyword_search_duration_ms(duration_ms: f64) {
        histogram!("pdf_keyword_search_duration_ms").record(duration_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_do_not_panic() {
        // Just verify metrics recording doesn't panic
        metrics_def::extraction_duration_ms("lopdf", 10.5);
        metrics_def::extraction_total("lopdf", "success");
        metrics_def::cache_hit_rate(0.85);
        metrics_def::cache_operations("hit");
        metrics_def::route_distribution("lopdf");
        metrics_def::circuit_breaker_state("lopdf", "closed");
        metrics_def::file_size_mb(5.5);
        metrics_def::keyword_search_duration_ms(2.5);
    }
}
