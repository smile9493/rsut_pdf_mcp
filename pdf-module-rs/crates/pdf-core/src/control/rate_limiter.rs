//! Rate limiter implementation
//! Provides token bucket based rate limiting for tool execution

use crate::dto::RateLimitConfig;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Rate limit statistics
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    /// Total requests allowed
    pub allowed: u64,
    /// Total requests rejected
    pub rejected: u64,
    /// Current window count
    pub current_count: u64,
}

/// Rate limiter
/// Provides per-tool rate limiting using token bucket algorithm
pub struct RateLimiter {
    /// Request counters per tool
    counters: DashMap<String, AtomicU64>,
    /// Rate limit configurations per tool
    configs: DashMap<String, RateLimitConfig>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new() -> Self {
        Self {
            counters: DashMap::new(),
            configs: DashMap::new(),
        }
    }

    /// Configure rate limit for a tool
    pub fn configure(&self, tool_name: String, config: RateLimitConfig) {
        self.configs.insert(tool_name, config);
    }

    /// Remove rate limit configuration for a tool
    pub fn remove_config(&self, tool_name: &str) {
        self.configs.remove(tool_name);
        self.counters.remove(tool_name);
    }

    /// Check if a request is allowed
    /// Returns true if allowed, false if rate limit exceeded
    pub fn check(&self, tool_name: &str) -> bool {
        let config = self.configs.get(tool_name);
        if config.is_none() {
            return true; // No config = no limit
        }

        let config = config.unwrap();
        let counter = self
            .counters
            .entry(tool_name.to_string())
            .or_insert_with(|| AtomicU64::new(0));

        let current = counter.load(Ordering::Relaxed);
        let limit = config.requests_per_second as u64;

        if current >= limit {
            // Check if we can use burst capacity
            let burst = config.burst_size as u64;
            if current < limit + burst {
                counter.fetch_add(1, Ordering::Relaxed);
                return true;
            }
            return false;
        }

        counter.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// Record that a request has completed (decrement counter)
    pub fn record_completion(&self, tool_name: &str) {
        if let Some(counter) = self.counters.get(tool_name) {
            counter.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Get statistics for a tool
    pub fn get_stats(&self, tool_name: &str) -> Option<RateLimitStats> {
        let _config = self.configs.get(tool_name)?;
        let counter = self.counters.get(tool_name)?;

        Some(RateLimitStats {
            allowed: 0, // Would need additional tracking for accurate stats
            rejected: 0,
            current_count: counter.load(Ordering::Relaxed),
        })
    }

    /// Get the configured limit for a tool
    pub fn get_limit(&self, tool_name: &str) -> Option<u32> {
        self.configs.get(tool_name).map(|c| c.requests_per_second)
    }

    /// Reset counter for a tool
    pub fn reset(&self, tool_name: &str) {
        if let Some(counter) = self.counters.get(tool_name) {
            counter.store(0, Ordering::Relaxed);
        }
    }

    /// Reset all counters
    pub fn reset_all(&self) {
        for entry in self.counters.iter() {
            entry.value().store(0, Ordering::Relaxed);
        }
    }

    /// List all configured tools
    pub fn configured_tools(&self) -> Vec<String> {
        self.configs.iter().map(|e| e.key().clone()).collect()
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_no_config() {
        let limiter = RateLimiter::new();
        // No config = always allowed
        assert!(limiter.check("unknown_tool"));
    }

    #[test]
    fn test_rate_limiter_within_limit() {
        let limiter = RateLimiter::new();
        limiter.configure(
            "test_tool".to_string(),
            RateLimitConfig {
                requests_per_second: 10,
                burst_size: 5,
            },
        );

        for _ in 0..10 {
            assert!(limiter.check("test_tool"));
        }
    }

    #[test]
    fn test_rate_limiter_exceed_limit() {
        let limiter = RateLimiter::new();
        limiter.configure(
            "test_tool".to_string(),
            RateLimitConfig {
                requests_per_second: 5,
                burst_size: 2,
            },
        );

        // Should allow up to 5 + 2 = 7 requests
        for _ in 0..7 {
            assert!(limiter.check("test_tool"));
        }

        // 8th request should be rejected
        assert!(!limiter.check("test_tool"));
    }

    #[test]
    fn test_rate_limiter_reset() {
        let limiter = RateLimiter::new();
        limiter.configure(
            "test_tool".to_string(),
            RateLimitConfig {
                requests_per_second: 2,
                burst_size: 0,
            },
        );

        assert!(limiter.check("test_tool"));
        assert!(limiter.check("test_tool"));
        assert!(!limiter.check("test_tool"));

        limiter.reset("test_tool");
        assert!(limiter.check("test_tool"));
    }
}
