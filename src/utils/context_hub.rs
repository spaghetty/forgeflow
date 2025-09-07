// The `context_hub` module provides a centralized hub for managing Google API
// authentication and context.

use super::google_auth::{AuthError, GConf, GmailHubType, gmail_auth};
use google_gmail1::api::Scope;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;
use tracing;

/// A hub for managing Google API authentication and context.
///
/// `ContextHub` is designed to centralize the authentication process for Google services,
/// ensuring that the application requests all necessary permissions (scopes) at once
/// and authenticates only a single time. This avoids multiple authentication prompts
/// for the user, improving the overall user experience.
///
/// The hub collects scopes from various components (tools, triggers) that require
/// Google API access. When the `get_hub` method is called for the first time, it
/// performs the OAuth2 flow with all the collected scopes and then caches the
/// authenticated `GmailHubType`. Subsequent calls to `get_hub` will return the cached
/// hub, avoiding repeated authentications.
///
/// This struct is intended to be wrapped in an `Arc` to be shared safely across
/// different components and threads.
pub struct ContextHub {
    gconf: GConf,
    scopes: Mutex<Vec<Scope>>,
    hub: TokioMutex<Option<GmailHubType>>,
}

impl ContextHub {
    /// Creates a new `ContextHub`.
    ///
    /// # Arguments
    ///
    /// * `gconf` - The Google authentication configuration.
    pub fn new(gconf: GConf) -> Self {
        Self {
            gconf,
            scopes: Mutex::new(Vec::new()),
            hub: TokioMutex::new(None),
        }
    }

    /// Adds a new scope to the hub. This operation is synchronous.
    ///
    /// # Arguments
    ///
    /// * `scope` - The scope to add.
    pub fn add_scope(&self, scope: Scope) {
        let mut scopes = self.scopes.lock().unwrap();
        if !scopes.contains(&scope) {
            scopes.push(scope);
        }
        tracing::info!("Added scopes: {:?}", scopes);
    }

    /// Returns the authenticated `GmailHubType`.
    ///
    /// If the hub has not been authenticated yet, this method will trigger the
    /// authentication process with all the scopes that have been added to the hub.
    /// If the hub has already been authenticated, it will return the cached hub.
    pub async fn get_hub(&self) -> Result<GmailHubType, AuthError> {
        let mut hub_guard = self.hub.lock().await;
        if let Some(hub) = hub_guard.as_ref() {
            return Ok(hub.clone());
        }

        // Clone the scopes to release the mutex lock before the .await call,
        // preventing the lock from being held across an await point.
        let scopes_clone = self.scopes.lock().unwrap().clone();
        let hub = gmail_auth(self.gconf.clone(), &scopes_clone).await?;
        *hub_guard = Some(hub.clone());

        Ok(hub)
    }
}
