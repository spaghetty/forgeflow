use crate::triggers::{TEvent, Trigger, TriggerError};
use async_trait::async_trait;
use google_gmail1::yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use google_gmail1::{Gmail, api::Scope};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use hyper_util::{
    client::legacy::Client, client::legacy::connect::HttpConnector, rt::TokioExecutor,
};
use rustls::crypto::{CryptoProvider, ring::default_provider};
use serde_json::json;
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio_util::bytes;
use tracing::info;

pub type HttpsConnectorType = HttpsConnector<HttpConnector>;
pub type HyperClient = Client<HttpsConnectorType, http_body_util::Full<bytes::Bytes>>;
pub type AuthType = google_gmail1::yup_oauth2::authenticator::Authenticator<HttpsConnectorType>;
pub type GmailHubType = Gmail<HttpsConnectorType>;

pub struct GConf {
    pub credentials_path: PathBuf,
    pub token_path: PathBuf,
}

pub struct GmailWatchTrigger {
    hub: Option<GmailHubType>,
    config: GConf,
}

impl GmailWatchTrigger {
    pub async fn new(conf: GConf) -> Result<Self, Box<dyn Error>> {
        //check the file here
        let mut resource = Self {
            hub: None,
            config: conf,
        };
        resource.auth().await?;
        Ok(resource)
    }

    pub async fn auth(&mut self) -> Result<(), TriggerError> {
        info!("Authenticating with Gmail API");

        // Read application secret
        let secret =
            google_gmail1::yup_oauth2::read_application_secret(&self.config.credentials_path)
                .await
                .expect("credential file missing");

        // Set up OAuth2 authenticator with required Gmail scopes
        let scopes = [Scope::Readonly];
        let auth =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
                .persist_tokens_to_disk(&self.config.token_path)
                .build()
                .await
                .unwrap();

        // Request initial token to ensure authentication works
        let _token = auth.token(&scopes).await.unwrap();

        // Initialize the crypto provider
        _ = CryptoProvider::install_default(default_provider());

        // Create HTTP client with native roots
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .unwrap()
            .https_or_http()
            .enable_http1()
            .build();

        let client = Client::builder(TokioExecutor::new()).build(https);

        // Create Gmail hub
        self.hub = Some(Gmail::new(client, auth));
        info!("Successfully authenticated with Gmail API");
        Ok(())
    }
}

#[async_trait]
impl Trigger for GmailWatchTrigger {
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
                        //info!("{:?}", result);
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
    async fn gmail_trigger_handle_event_and_shuts_down() {
        // --- 1. Arrange ---
        // Create the channels that the agent would normally create.
        let (_event_tx, mut _event_rx) = mpsc::channel::<TEvent>(10);
        let (_shutdown_tx, _shutdown_rx) = broadcast::channel::<()>(1);

        let conf = GConf {
            credentials_path: Path::new("./tmp/credential.json").to_path_buf(),
            token_path: Path::new("./tmp/token.json").to_path_buf(),
        };
        // Create the PollTrigger instance.
        let gtrigger = GmailWatchTrigger::new(conf).await;

        let tmp = gtrigger.expect("Something went wrong").auth().await;
        assert!(tmp.is_ok());
        println!("{:?}", tmp);
        // --- 2. Act ---
        // Launch the trigger and get its task handle.
        //let trigger_handle = trigger.launch(event_tx, shutdown_rx).await.unwrap();

        // --- 3. Assert ---
        // Assert that we receive the event.
        // We use `tokio::time::timeout` to prevent the test from hanging forever
        // if the trigger fails to send an event.
        //let reception_timeout = Duration::from_millis(100);
        //let received_event = tokio::time::timeout(reception_timeout, event_rx.recv())
        //    .await
        //    .expect("Test timed out waiting for trigger event");

        // Check that the received event is not None and has the correct name.
        //assert!(received_event.is_some(), "Did not receive an event");
        //assert_eq!(received_event.unwrap().name, test_event_name);

        // --- 4. Test Shutdown ---
        // Send the shutdown signal.
        //println!("Test: Sending shutdown signal.");
        //shutdown_tx.send(()).unwrap();

        // Wait for the trigger's task to complete.
        // Again, use a timeout to prevent the test from hanging if shutdown fails.
        //let shutdown_timeout = Duration::from_millis(50);
        //let result = tokio::time::timeout(shutdown_timeout, trigger_handle).await;

        // Assert that the handle completed successfully within the timeout.
        //assert!(
        //    result.is_ok(),
        //    "Trigger failed to shut down gracefully within the timeout"
        //);
        //println!("Test: Trigger shut down gracefully.");
        assert!(false);
    }
}
