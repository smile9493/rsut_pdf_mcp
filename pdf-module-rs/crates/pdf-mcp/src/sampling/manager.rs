//! MCP Sampling manager with backpressure and graceful shutdown.
//!
//! Manages sampling requests from the PDF MCP server, implementing bounded
//! channels for backpressure and cancellation tokens for graceful shutdown.

use super::{SamplingRequest, SamplingResponse};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;

/// Errors that can occur during sampling operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SamplingError {
    #[error("Service overloaded: {0}")]
    Overloaded(String),

    #[error("Channel closed")]
    ChannelClosed,

    #[error("Response timeout")]
    ResponseTimeout,

    #[error("Service shutting down")]
    ShuttingDown,

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Internal task that pairs a request with a oneshot response channel.
struct SamplingTask {
    request: SamplingRequest,
    response_tx: oneshot::Sender<Result<SamplingResponse, SamplingError>>,
}

/// Manager for MCP sampling requests with backpressure and graceful shutdown.
///
/// Implements bounded channel for backpressure and cancellation token for
/// graceful shutdown support. When the service is at capacity, new requests
/// are rejected with `SamplingError::Overloaded` so upstream can return 503.
pub struct SamplingManager {
    request_tx: mpsc::Sender<SamplingTask>,
    cancel_token: CancellationToken,
    active_requests: Arc<AtomicUsize>,
    max_requests: usize,
}

impl SamplingManager {
    /// Create a new sampling manager with specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Bounded channel size (queue depth for pending requests)
    /// * `max_requests` - Maximum active requests before returning overloaded
    pub fn new(capacity: usize, max_requests: usize) -> Self {
        let (request_tx, mut request_rx) = mpsc::channel::<SamplingTask>(capacity);
        let cancel_token = CancellationToken::new();
        let cancel_token_clone = cancel_token.clone();
        let active_requests = Arc::new(AtomicUsize::new(0));
        let active_requests_clone = Arc::clone(&active_requests);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(task) = request_rx.recv() => {
                        let response = Self::handle_sampling_request(task.request).await;
                        let _ = task.response_tx.send(response);
                        active_requests_clone.fetch_sub(1, Ordering::Relaxed);
                    }
                    _ = cancel_token_clone.cancelled() => {
                        tracing::info!("Sampling manager shutting down gracefully");
                        break;
                    }
                    else => break,
                }
            }
        });

        Self {
            request_tx,
            cancel_token,
            active_requests,
            max_requests,
        }
    }

    /// Request sampling with backpressure support.
    ///
    /// Returns `SamplingError::Overloaded` when service is at capacity,
    /// allowing upstream to return 503 to clients.
    pub async fn request_sampling(
        &self,
        request: SamplingRequest,
    ) -> Result<SamplingResponse, SamplingError> {
        // Check if shutting down
        if self.cancel_token.is_cancelled() {
            return Err(SamplingError::ShuttingDown);
        }

        // Check resource limit (backpressure)
        let current = self.active_requests.fetch_add(1, Ordering::Relaxed);
        if current >= self.max_requests {
            self.active_requests.fetch_sub(1, Ordering::Relaxed);
            return Err(SamplingError::Overloaded(
                "Service temporarily unavailable due to high load".to_string(),
            ));
        }

        let (response_tx, response_rx) = oneshot::channel();

        let result = self
            .request_tx
            .send(SamplingTask {
                request,
                response_tx,
            })
            .await
            .map_err(|_| {
                self.active_requests
                    .fetch_sub(1, Ordering::Relaxed);
                SamplingError::ChannelClosed
            });

        match result {
            Ok(_) => response_rx.await.map_err(|_| SamplingError::ResponseTimeout)?,
            Err(e) => Err(e),
        }
    }

    /// Gracefully shutdown the sampling manager.
    pub fn shutdown(&self) {
        self.cancel_token.cancel();
    }

    /// Check if the manager is shutting down.
    pub fn is_shutting_down(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// Get the number of currently active requests.
    pub fn active_request_count(&self) -> usize {
        self.active_requests.load(Ordering::Relaxed)
    }

    /// Handle a single sampling request.
    ///
    /// In a real implementation, this would forward the request to an LLM
    /// via the MCP client's sampling endpoint.
    async fn handle_sampling_request(
        _request: SamplingRequest,
    ) -> Result<SamplingResponse, SamplingError> {
        // TODO: Implement actual LLM call via MCP client sampling endpoint
        // For now, return a placeholder response
        Err(SamplingError::Internal(
            "Sampling not yet implemented - requires MCP client support".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sampling_manager_backpressure() {
        let manager = SamplingManager::new(2, 2);

        // Send requests up to capacity
        let req1 = manager.request_sampling(SamplingRequest::default());
        let req2 = manager.request_sampling(SamplingRequest::default());

        // Wait for the first two to complete (they will error with Internal since not implemented)
        let _ = req1.await;
        let _ = req2.await;

        // After draining, a new request should work
        let req3 = manager.request_sampling(SamplingRequest::default());
        // This should not be overloaded since we've released slots
        assert!(!matches!(req3.await, Err(SamplingError::Overloaded(_))));
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let manager = SamplingManager::new(10, 5);
        manager.shutdown();

        let result = manager.request_sampling(SamplingRequest::default()).await;
        assert!(matches!(result, Err(SamplingError::ShuttingDown)));
    }

    #[test]
    fn test_active_request_count() {
        let manager = SamplingManager::new(10, 5);
        assert_eq!(manager.active_request_count(), 0);
        assert!(!manager.is_shutting_down());
    }
}
