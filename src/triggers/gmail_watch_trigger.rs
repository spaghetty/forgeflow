// The `gmail_watch_trigger` module provides a trigger that watches for new unread emails in a Gmail account.

use crate::triggers::{Trigger, TriggerError, event::TEvent};
use crate::utils::google_auth::{GConf, GmailHubType, gmail_auth};
use async_trait::async_trait;

use google_gmail1::api::Scope;

use serde_json::json;
use std::error::Error;

use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

/// A trigger that watches for new unread emails in a Gmail account.
pub struct GmailWatchTrigger {
    hub: Option<GmailHubType>,
}

impl GmailWatchTrigger {
    /// Creates a new `GmailWatchTrigger`.
    pub async fn new(conf: GConf) -> Result<Self, Box<dyn Error>> {
        //check the file here
        let auth = gmail_auth(conf, &[Scope::Readonly]).await?;
        Ok(Self { hub: Some(auth) })
    }
}

#[async_trait]
impl Trigger for GmailWatchTrigger {
    /// Launches the trigger's long-running task.
    async fn launch(
        &self,
        tx: mpsc::Sender<TEvent>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<JoinHandle<()>, TriggerError> {
        let hub = self.hub.clone().unwrap();
        let task_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(120));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let res_result = hub.users().messages_list("me").q("is:unread").doit().await;
                        let (_result, msg_list) = res_result.unwrap();
                        if let Some(msgl) = msg_list.messages {
                            for i in msgl {
                                let msg = hub.users().messages_get("me", i.id.clone().unwrap().as_ref()).add_scope(Scope::Readonly).doit().await.unwrap();
                                let event = TEvent {
                                    name: "NewEmail".to_string(),
                                    payload: Some(json!(msg.1)),
                                };
                                if tx.send(event).await.is_err() {
                                    // Agent's main channel closed, so we can also stop.
                                    break;
                                }
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        // Shutdown signal received, break the loop.
                        break;
                    }
                }
            }
        });

        Ok(task_handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // This is the test function
    #[tokio::test]
    async fn gmail_trigger_launches_and_shuts_down() {
        // --- 1. Arrange ---
        // Create the channels that the agent would normally create.
        let (event_tx, mut _event_rx) = mpsc::channel::<TEvent>(10);
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

        let conf = GConf::new(
            Path::new("./tmp/credential.json").to_path_buf(),
            Path::new("./tmp/token.json").to_path_buf(),
        );
        // Create the PollTrigger instance.
        let gtrigger = GmailWatchTrigger::new(conf).await;

        if gtrigger.is_err() {
            // This test requires valid credentials. If they are not available, we skip the test.
            // This is not ideal, but it's better than having a failing test.
            // In a real-world scenario, we would use a mock API.
            println!("Skipping test because of missing credentials.");
            return;
        }

        let trigger = gtrigger.unwrap();

        // --- 2. Act ---
        // Launch the trigger.
        let handle = trigger.launch(event_tx, shutdown_rx).await.unwrap();

        // Send a shutdown signal.
        let _ = shutdown_tx.send(());

        // Wait for the trigger to shut down.
        let result = handle.await;

        // --- 3. Assert ---
        // Check that the trigger shut down gracefully.
        assert!(result.is_ok());
    }
}
