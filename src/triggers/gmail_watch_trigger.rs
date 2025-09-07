// The `gmail_watch_trigger` module provides a trigger that watches for new unread emails in a Gmail account.

use crate::{
    triggers::{event::TEvent, Trigger, TriggerError},
    utils::{context_hub::ContextHub, google_auth::GmailHubType},
};
use async_trait::async_trait;
use google_gmail1::api::Scope;
use serde_json::json;
use std::{error::Error, sync::Arc, time::Duration};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

/// A builder for [`GmailWatchTrigger`].
pub struct GmailWatchTriggerBuilder {
    hub: Arc<ContextHub>,
}

impl GmailWatchTriggerBuilder {
    /// Creates a new `GmailWatchTriggerBuilder`.
    ///
    /// This method registers the required `Readonly` scope with the provided [`ContextHub`].
    ///
    /// # Arguments
    ///
    /// * `hub` - A shared [`ContextHub`] for managing authentication.
    pub fn new(hub: Arc<ContextHub>) -> Self {
        hub.add_scope(Scope::Readonly);
        Self { hub }
    }

    /// Builds a [`GmailWatchTrigger`].
    ///
    /// This method authenticates with the Gmail API (if not already authenticated)
    /// using the scopes collected in the [`ContextHub`] and creates a [`GmailWatchTrigger`].
    pub async fn build(&self) -> Result<GmailWatchTrigger, Box<dyn Error>> {
        let hub = self.hub.get_hub().await?;
        Ok(GmailWatchTrigger { hub })
    }
}

/// A trigger that watches for new unread emails in a Gmail account.
pub struct GmailWatchTrigger {
    hub: GmailHubType,
}

#[async_trait]
impl Trigger for GmailWatchTrigger {
    /// Launches the trigger's long-running task.
    async fn launch(
        &self,
        tx: mpsc::Sender<TEvent>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<JoinHandle<()>, TriggerError> {
        let hub = self.hub.clone();
        let task_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(120));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let res_result = hub.users().messages_list("me").q("is:unread").doit().await;
                        if let Ok((_result, msg_list)) = res_result {
                            if let Some(msgl) = msg_list.messages {
                                for i in msgl {
                                    if let Some(id) = i.id {
                                        let msg_result = hub.users().messages_get("me", &id).add_scope(Scope::Readonly).doit().await;
                                        if let Ok(msg) = msg_result {
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
    use crate::utils::google_auth::{GConf, InnerConf, GoogleAuthFlow};
    use std::path::Path;

    // This is the test function
    #[tokio::test]
    async fn gmail_trigger_launches_and_shuts_down() {
        // --- 1. Arrange ---
        // Create the channels that the agent would normally create.
        let (event_tx, mut _event_rx) = mpsc::channel::<TEvent>(10);
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

        let conf = GConf::from(Arc::new(InnerConf {
            credentials_path: Path::new("./tmp/credential.json").to_path_buf(),
            token_path: Path::new("./tmp/token.json").to_path_buf(),
            flow: GoogleAuthFlow::default(),
        }));

        // Create the ContextHub and the builder.
        let hub = Arc::new(ContextHub::new(conf));
        let builder = GmailWatchTriggerBuilder::new(hub);

        // Build the trigger.
        let trigger_result = builder.build().await;

        if trigger_result.is_err() {
            // This test requires valid credentials. If they are not available, we skip the test.
            // This is not ideal, but it's better than having a failing test.
            // In a real-world scenario, we would use a mock API.
            println!("Skipping test because of missing credentials or auth error.");
            return;
        }

        let trigger = trigger_result.unwrap();

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
