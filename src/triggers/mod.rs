pub mod event;
pub mod poll_trigger;

use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};

use crate::triggers::event::TEvent;
pub use crate::triggers::poll_trigger::PollTrigger;

#[derive(Error, Debug)]
pub enum TriggerError {
    #[error("Error activating the trigger")]
    ActivationError,
}

#[async_trait]
pub trait Trigger: Send + Sync {
    /// Launches the trigger's long-running task.
    ///
    /// # Arguments
    /// * `tx` - The sender to send TEvevents back to the agent.
    /// * `shutdown_rx` - A broadcast receiver to listen for a shutdown signal.
    async fn launch(
        &self,
        tx: mpsc::Sender<TEvent>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<tokio::task::JoinHandle<()>, TriggerError>;
}
