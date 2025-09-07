// The `poll_trigger` module provides a trigger that fires an event at a regular interval.

use crate::triggers::{event::TEvent, Trigger, TriggerError};
use async_trait::async_trait;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tracing::{debug, info, warn};

/// A builder for [`PollTrigger`].
pub struct PollTriggerBuilder {
    event_name: String,
    interval: Duration,
    hot_start: bool,
}

impl PollTriggerBuilder {
    /// Creates a new `PollTriggerBuilder`.
    ///
    /// # Arguments
    ///
    /// * `event_name` - The name of the event to fire.
    /// * `interval` - The interval at which to fire the event.
    pub fn new(event_name: &str, interval: Duration) -> Self {
        Self {
            event_name: event_name.to_string(),
            interval,
            hot_start: false, // Default value
        }
    }

    /// Sets whether the trigger should fire an event immediately upon launch.
    pub fn with_hot_start(mut self, hot_start: bool) -> Self {
        self.hot_start = hot_start;
        self
    }

    /// Builds a `PollTrigger`.
    pub fn build(&self) -> PollTrigger {
        PollTrigger {
            event_name: self.event_name.clone(),
            interval: self.interval,
            hot_start: self.hot_start,
        }
    }
}

/// A trigger that fires an event at a regular interval.
pub struct PollTrigger {
    /// The name of the event to fire.
    event_name: String,
    /// The interval at which to fire the event.
    interval: Duration,
    /// Whether to fire an event immediately upon launch.
    hot_start: bool,
}

#[async_trait]
impl Trigger for PollTrigger {
    /// Launches the trigger's long-running task.
    async fn launch(
        &self,
        tx: mpsc::Sender<TEvent>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<JoinHandle<()>, TriggerError> {
        let interval = self.interval;
        let event_name = self.event_name.clone();
        let hot_start = self.hot_start;

        let task_handle = tokio::spawn(async move {
            let mut start_time = Instant::now();
            if !hot_start {
                start_time += interval;
            }
            let mut ticker = tokio::time::interval_at(start_time, interval);

            info!(trigger_name = %event_name, interval_secs = interval.as_secs(), "PollTrigger started");

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!(trigger_name = %event_name, "PollTrigger received shutdown signal, terminating");
                        break;
                    }

                    _ = ticker.tick() => {
                        let event = TEvent {
                            name: event_name.clone(),
                            payload: None,
                        };

                        debug!(trigger_name = %event_name, event_name = %event.name, "Firing event");

                        if let Err(e) = tx.send(event).await {
                            warn!(trigger_name = %event_name, error = %e, "Main channel closed, stopping trigger");
                            break;
                        }
                    }
                }
            }
            debug!(trigger_name = %event_name, "PollTrigger task completed");
        });

        Ok(task_handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn poll_trigger_sends_event_and_shuts_down() {
        // --- 1. Arrange ---
        let (event_tx, mut event_rx) = mpsc::channel::<TEvent>(10);
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

        let test_interval = Duration::from_millis(50);
        let test_event_name = "TestEvent";

        // Use the builder to create the PollTrigger instance.
        let trigger = PollTriggerBuilder::new(test_event_name, test_interval).build();

        // --- 2. Act ---
        let trigger_handle = trigger.launch(event_tx, shutdown_rx).await.unwrap();

        // --- 3. Assert ---
        let reception_timeout = Duration::from_millis(100);
        let received_event = tokio::time::timeout(reception_timeout, event_rx.recv())
            .await
            .expect("Test timed out waiting for trigger event");

        assert!(received_event.is_some(), "Did not receive an event");
        assert_eq!(received_event.unwrap().name, test_event_name);

        // --- 4. Test Shutdown ---
        println!("Test: Sending shutdown signal.");
        shutdown_tx.send(()).unwrap();

        let shutdown_timeout = Duration::from_millis(50);
        let result = tokio::time::timeout(shutdown_timeout, trigger_handle).await;

        assert!(
            result.is_ok(),
            "Trigger failed to shut down gracefully within the timeout"
        );
        println!("Test: Trigger shut down gracefully.");
    }
}
