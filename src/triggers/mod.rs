// The `triggers` module provides a collection of triggers that can be used to initiate agent actions.

pub mod event;
pub mod gmail_watch_trigger;
pub mod poll_trigger;

use crate::utils::google_auth::AuthError;
use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};

use crate::triggers::event::TEvent;
pub use crate::triggers::gmail_watch_trigger::GmailWatchTrigger;
pub use crate::triggers::poll_trigger::PollTrigger;

/// The `TriggerError` enum defines the possible errors that can occur within a trigger.
#[derive(Error, Debug)]
pub enum TriggerError {
    /// An error occurred while activating the trigger.
    #[error("Error activating the trigger")]
    ActivationError,
    /// An error occurred while authenticating the trigger.
    #[error("Error authenticating the trigger")]
    AuthError(#[from] AuthError),
}

/// The `Trigger` trait defines the contract for any trigger that can be used by the agent.
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
