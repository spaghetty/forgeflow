use crate::triggers::{Trigger, TriggerError, event::TEvent};
use async_trait::async_trait;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tracing::{debug, info, warn};

pub struct PollTrigger {
    event_name: String,
    interval: Duration,
    hot_start: bool,
}

impl PollTrigger {
    pub fn new(payload: &str, frequency: Duration, hot_start: bool) -> Box<Self> {
        Box::new(PollTrigger {
            event_name: payload.to_string(),
            interval: frequency,
            hot_start,
        })
    }
}

#[async_trait]
impl Trigger for PollTrigger {
    async fn launch(
        &self,
        tx: mpsc::Sender<TEvent>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<JoinHandle<()>, TriggerError> {
        let interval = self.interval;
        let event_name = self.event_name.clone();
        let hot_start = self.hot_start;

        // Spawn the task and keep its handle
        let task_handle = tokio::spawn(async move {
            let mut start_time = Instant::now();
            if !hot_start {
                start_time += interval;
            }
            let mut ticker = tokio::time::interval_at(start_time, interval);

            info!(trigger_name = %event_name, interval_secs = interval.as_secs(), "PollTrigger started");

            loop {
                // select! allows us to wait on multiple futures at once.
                // It completes when the FIRST future completes.
                tokio::select! {
                    // Branch 1: The shutdown signal is received.
                    _ = shutdown_rx.recv() => {
                        info!(trigger_name = %event_name, "PollTrigger received shutdown signal, terminating");
                        // Break the loop to exit the task.
                        break;
                    }

                    // Branch 2: The timer ticks.
                    _ = ticker.tick() => {
                        let event = TEvent {
                            name: event_name.clone(),
                            payload: None,
                        };

                        debug!(trigger_name = %event_name, event_name = %event.name, "Firing event");

                        if let Err(e) = tx.send(event).await {
                            // Agent's main channel closed, so we can also stop.
                            warn!(trigger_name = %event_name, error = %e, "Main channel closed, stopping trigger");
                            break;
                        }
                    }
                }
            }
            debug!(trigger_name = %event_name, "PollTrigger task completed");
        });

        // Return the handle so the agent can wait for it.
        Ok(task_handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // This is the test function
    #[tokio::test]
    async fn poll_trigger_sends_event_and_shuts_down() {
        // --- 1. Arrange ---
        // Create the channels that the agent would normally create.
        let (event_tx, mut event_rx) = mpsc::channel::<TEvent>(10);
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

        // Define a short interval for the test to run quickly.
        let test_interval = Duration::from_millis(50);
        let test_event_name = "TestEvent".to_string();

        // Create the PollTrigger instance.
        let trigger = PollTrigger {
            interval: test_interval,
            event_name: test_event_name.clone(),
            hot_start: false,
        };

        // --- 2. Act ---
        // Launch the trigger and get its task handle.
        let trigger_handle = trigger.launch(event_tx, shutdown_rx).await.unwrap();

        // --- 3. Assert ---
        // Assert that we receive the event.
        // We use `tokio::time::timeout` to prevent the test from hanging forever
        // if the trigger fails to send an event.
        let reception_timeout = Duration::from_millis(100);
        let received_event = tokio::time::timeout(reception_timeout, event_rx.recv())
            .await
            .expect("Test timed out waiting for trigger event");

        // Check that the received event is not None and has the correct name.
        assert!(received_event.is_some(), "Did not receive an event");
        assert_eq!(received_event.unwrap().name, test_event_name);

        // --- 4. Test Shutdown ---
        // Send the shutdown signal.
        println!("Test: Sending shutdown signal.");
        shutdown_tx.send(()).unwrap();

        // Wait for the trigger's task to complete.
        // Again, use a timeout to prevent the test from hanging if shutdown fails.
        let shutdown_timeout = Duration::from_millis(50);
        let result = tokio::time::timeout(shutdown_timeout, trigger_handle).await;

        // Assert that the handle completed successfully within the timeout.
        assert!(
            result.is_ok(),
            "Trigger failed to shut down gracefully within the timeout"
        );
        println!("Test: Trigger shut down gracefully.");
    }
}
