//! Circuit breaker implementation (unified).
//!
//! Consolidates the two previous implementations:
//! - `pdf-core::control::circuit_breaker::CircuitBreaker` (general-purpose, atomic)
//! - `pdf-core::extractor::CircuitBreaker` (per-engine, mutex-based)
//!
//! This version uses a lock-free atomic implementation with per-engine state support.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests allowed.
    Closed = 0,
    /// Fault detected - requests blocked.
    Open = 1,
    /// Testing recovery - limited requests allowed.
    HalfOpen = 2,
}

impl CircuitState {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }
}

/// Circuit breaker configuration.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening.
    pub failure_threshold: u64,
    /// Number of successes in half-open before closing.
    pub success_threshold: u64,
    /// Time in milliseconds before transitioning from open to half-open.
    pub timeout_ms: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_ms: 30_000,
        }
    }
}

/// Single-engine circuit breaker (lock-free).
///
/// Uses atomic operations for all state transitions.
pub struct CircuitBreaker {
    state: AtomicU8,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure_time: AtomicU64,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration.
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: AtomicU8::new(CircuitState::Closed as u8),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure_time: AtomicU64::new(0),
            config,
        }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// Check if a call is allowed.
    pub fn allow_call(&self) -> bool {
        let state = CircuitState::from_u8(self.state.load(Ordering::Relaxed));
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let elapsed = self.elapsed_since_last_failure();
                if elapsed >= self.config.timeout_ms {
                    self.state.store(CircuitState::HalfOpen as u8, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful call.
    pub fn record_success(&self) {
        let state = CircuitState::from_u8(self.state.load(Ordering::Relaxed));
        match state {
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                let success = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if success >= self.config.success_threshold {
                    self.state.store(CircuitState::Closed as u8, Ordering::Relaxed);
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::Open => {}
        }
    }

    /// Record a failed call.
    pub fn record_failure(&self) {
        let state = CircuitState::from_u8(self.state.load(Ordering::Relaxed));
        match state {
            CircuitState::Closed => {
                let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if failures >= self.config.failure_threshold {
                    self.state.store(CircuitState::Open as u8, Ordering::Relaxed);
                    self.last_failure_time
                        .store(current_time_ms(), Ordering::Relaxed);
                }
            }
            CircuitState::HalfOpen => {
                self.state.store(CircuitState::Open as u8, Ordering::Relaxed);
                self.last_failure_time
                    .store(current_time_ms(), Ordering::Relaxed);
            }
            CircuitState::Open => {}
        }
    }

    /// Get the current state.
    pub fn state(&self) -> CircuitState {
        CircuitState::from_u8(self.state.load(Ordering::Relaxed))
    }

    /// Get current failure count.
    pub fn failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::Relaxed)
    }

    /// Get current success count.
    pub fn success_count(&self) -> u64 {
        self.success_count.load(Ordering::Relaxed)
    }

    /// Reset to closed state.
    pub fn reset(&self) {
        self.state
            .store(CircuitState::Closed as u8, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
    }

    fn elapsed_since_last_failure(&self) -> u64 {
        let last = self.last_failure_time.load(Ordering::Relaxed);
        current_time_ms().saturating_sub(last)
    }
}

/// Per-engine circuit breaker registry.
///
/// Replaces the inline `CircuitBreaker` in `pdf-core::extractor` that used
/// a `Mutex<HashMap<String, EngineBreaker>>`. This version still uses a
/// Mutex for the registry map but delegates per-engine state to lock-free
/// `CircuitBreaker` instances.
pub struct EngineCircuitBreaker {
    failure_threshold: u64,
    timeout_ms: u64,
    breakers: Mutex<HashMap<String, CircuitBreaker>>,
}

impl EngineCircuitBreaker {
    /// Create a new per-engine circuit breaker.
    pub fn new(failure_threshold: u64, timeout: Duration) -> Self {
        Self {
            failure_threshold,
            timeout_ms: timeout.as_millis() as u64,
            breakers: Mutex::new(HashMap::new()),
        }
    }

    /// Register an engine.
    pub fn register_engine(&self, engine_name: &str) {
        let config = CircuitBreakerConfig {
            failure_threshold: self.failure_threshold,
            success_threshold: 1, // Single success recovers
            timeout_ms: self.timeout_ms,
        };
        let mut breakers = self.breakers.lock().unwrap();
        breakers.insert(engine_name.to_string(), CircuitBreaker::new(config));
    }

    /// Check if an engine is available.
    pub fn is_available(&self, engine: &str) -> bool {
        let breakers = self.breakers.lock().unwrap();
        breakers
            .get(engine)
            .map(|cb| cb.allow_call())
            .unwrap_or(true)
    }

    /// Record a successful operation on an engine.
    pub fn record_success(&self, engine: &str) {
        let breakers = self.breakers.lock().unwrap();
        if let Some(cb) = breakers.get(engine) {
            cb.record_success();
        }
    }

    /// Record a failed operation on an engine.
    pub fn record_failure(&self, engine: &str) {
        let breakers = self.breakers.lock().unwrap();
        if let Some(cb) = breakers.get(engine) {
            cb.record_failure();
        }
    }

    /// Get the circuit state for a specific engine.
    pub fn state(&self, engine: &str) -> CircuitState {
        let breakers = self.breakers.lock().unwrap();
        breakers
            .get(engine)
            .map(|cb| cb.state())
            .unwrap_or(CircuitState::Closed)
    }

    /// Reset all circuit breakers.
    pub fn reset_all(&self) {
        let breakers = self.breakers.lock().unwrap();
        for cb in breakers.values() {
            cb.reset();
        }
    }
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::with_defaults();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_call());
    }

    #[test]
    fn test_circuit_breaker_opens_on_failures() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout_ms: 1000,
        });
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_call());
    }

    #[test]
    fn test_circuit_breaker_half_open_after_timeout() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout_ms: 0, // Immediate transition
        });
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        assert!(cb.allow_call());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_breaker_closes_on_success() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout_ms: 0,
        });
        cb.record_failure();
        assert!(cb.allow_call()); // Half-open
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_half_open_failure_reopens() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout_ms: 0,
        });
        cb.record_failure();
        assert!(cb.allow_call()); // Half-open

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout_ms: 10000,
        });
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_call());
    }

    #[test]
    fn test_engine_circuit_breaker() {
        // Use zero timeout so Open->HalfOpen transition is immediate
        let ecb = EngineCircuitBreaker::new(3, Duration::from_secs(0));
        ecb.register_engine("lopdf");
        ecb.register_engine("pdfium");

        assert!(ecb.is_available("lopdf"));
        assert!(ecb.is_available("pdfium"));
        assert!(ecb.is_available("unknown")); // Unknown engines are available

        ecb.record_failure("lopdf");
        ecb.record_failure("lopdf");
        ecb.record_failure("lopdf");
        // After 3 failures (>= threshold of 3), circuit is open
        // But is_available with zero timeout transitions to half-open
        assert!(ecb.is_available("lopdf"));
        assert!(ecb.is_available("pdfium")); // Other engines unaffected

        // Success in half-open closes the circuit (success_threshold = 1)
        ecb.record_success("lopdf");
        assert!(ecb.is_available("lopdf"));
    }
}
