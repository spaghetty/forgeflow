use async_trait::async_trait;
use std::time::Duration;
use tracing::info;

/// A trait for sources that can trigger a graceful shutdown of the agent.
#[async_trait]
pub trait Shutdown: Send + Sync {
    /// This future resolves when a shutdown signal is received.
    async fn wait_for_signal(&mut self);
}

pub struct CtrlCShutdown;

impl CtrlCShutdown {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Shutdown for CtrlCShutdown {
    async fn wait_for_signal(&mut self) {
        // Wait for the ctrl_c signal from Tokio.
        // We use .ok() because we don't care about a potential error
        // receiving the signal; we only care that it was triggered.
        let _ = tokio::signal::ctrl_c().await;
        info!("Ctrl-C received, initiating graceful shutdown");
    }
}

pub struct TimeBasedShutdown {
    duration: Duration,
}

impl TimeBasedShutdown {
    /// Creates a new handler that will trigger a shutdown after the given duration.
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
}

#[async_trait]
impl Shutdown for TimeBasedShutdown {
    async fn wait_for_signal(&mut self) {
        info!(
            duration_secs = self.duration.as_secs(),
            "Agent shutdown scheduled"
        );
        // Simply sleep for the specified duration.
        tokio::time::sleep(self.duration).await;
        info!(duration_secs = self.duration.as_secs(), "Time-based shutdown triggered");
    }
}
