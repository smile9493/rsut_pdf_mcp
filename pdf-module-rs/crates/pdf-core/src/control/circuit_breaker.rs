//! Circuit breaker implementation
//! Provides state machine based circuit breaker for fault tolerance

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Circuit state
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Normal operation - requests are allowed
    Closed,
    /// Fault detected - requests are blocked
    Open,
    /// Testing recovery - limited requests allowed
    HalfOpen,
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

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening
    pub failure_threshold: u64,
    /// Number of successes in half-open before closing
    pub success_threshold: u64,
    /// Time in ms before transitioning from open to half-open
    pub timeout_ms: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_ms: 30000,
        }
    }
}

/// Circuit breaker
/// Protects against cascading failures using a state machine
pub struct CircuitBreaker {
    /// Current state: 0=Closed, 1=Open, 2=HalfOpen
    state: AtomicU8,
    /// Consecutive failure count
    failure_count: AtomicU64,
    /// Consecutive success count (in half-open)
    success_count: AtomicU64,
    /// Configuration
    config: CircuitBreakerConfig,
    /// Timestamp of last failure (ms since epoch)
    last_failure_time: AtomicU64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: AtomicU8::new(0),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            config,
            last_failure_time: AtomicU64::new(0),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// Check if a call is allowed
    pub fn allow_call(&self) -> bool {
        let state = CircuitState::from_u8(self.state.load(Ordering::Relaxed));

        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let elapsed = self.elapsed_since_last_failure();
                if elapsed >= self.config.timeout_ms {
                    // Transition to half-open
                    self.state.store(2, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful call
    pub fn record_success(&self) {
        let state = CircuitState::from_u8(self.state.load(Ordering::Relaxed));

        match state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                let success = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if success >= self.config.success_threshold {
                    // Transition to closed
                    self.state.store(0, Ordering::Relaxed);
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::Open => {}
        }
    }

    /// Record a failed call
    pub fn record_failure(&self) {
        let state = CircuitState::from_u8(self.state.load(Ordering::Relaxed));

        match state {
            CircuitState::Closed => {
                let failure = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if failure >= self.config.failure_threshold {
                    // Transition to open
                    self.state.store(1, Ordering::Relaxed);
                    self.last_failure_time.store(
                        current_time_ms(),
                        Ordering::Relaxed,
                    );
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open goes back to open
                self.state.store(1, Ordering::Relaxed);
                self.last_failure_time.store(
                    current_time_ms(),
                    Ordering::Relaxed,
                );
            }
            CircuitState::Open => {}
        }
    }

    /// Get current state
    pub fn state(&self) -> CircuitState {
        CircuitState::from_u8(self.state.load(Ordering::Relaxed))
    }

    /// Get failure count
    pub fn failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::Relaxed)
    }

    /// Get success count
    pub fn success_count(&self) -> u64 {
        self.success_count.load(Ordering::Relaxed)
    }

    /// Reset the circuit breaker to closed state
    pub fn reset(&self) {
        self.state.store(0, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
    }

    /// Calculate elapsed time since last failure
    fn elapsed_since_last_failure(&self) -> u64 {
        let last = self.last_failure_time.load(Ordering::Relaxed);
        let now = current_time_ms();
        now.saturating_sub(last)
    }
}

/// Get current time in milliseconds since epoch
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

        // After timeout, should transition to half-open
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
        assert_eq!(cb.state(), CircuitState::Open);

        // Transition to half-open
        assert!(cb.allow_call());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Success in half-open
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
        assert!(cb.allow_call()); // Transition to half-open

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
}
