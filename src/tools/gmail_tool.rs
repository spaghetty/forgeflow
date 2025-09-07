// The `gmail_actions` module provides a tool for interacting with the Gmail API.

use crate::utils::context_hub::ContextHub;
use crate::utils::google_auth::GmailHubType;
use google_gmail1::api::{ModifyMessageRequest, Scope};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use thiserror::Error;

/// The `GmailToolError` enum defines the possible errors that can occur within the `GmailTool`.
#[derive(Debug, Error)]
pub enum GmailToolError {
    /// An error occurred while building the tool.
    #[error("Gmail tool build error: {0}")]
    BuildError(String),

    /// An error occurred while marking a message as unread.
    #[error("Failed to mark message as unread: {0}")]
    MarkUnreadError(String),
}

/// A builder for [`GmailTool`].
pub struct GmailToolBuilder {
    hub: Arc<ContextHub>,
}

impl GmailToolBuilder {
    /// Creates a new `GmailToolBuilder`.
    ///
    /// This method registers the required `Modify` scope with the provided [`ContextHub`].
    ///
    /// # Arguments
    ///
    /// * `hub` - A shared [`ContextHub`] for managing authentication.
    pub fn new(hub: Arc<ContextHub>) -> Self {
        hub.add_scope(Scope::Modify);
        Self { hub }
    }

    /// Builds a [`GmailTool`].
    ///
    /// This method authenticates with the Gmail API (if not already authenticated)
    /// and creates a [`GmailTool`].
    pub async fn build(&self) -> Result<GmailTool, GmailToolError> {
        let hub = self
            .hub
            .get_hub()
            .await
            .map_err(|e| GmailToolError::BuildError(e.to_string()))?;
        Ok(GmailTool { hub })
    }
}

/// The arguments for the `GmailTool`.
#[derive(Deserialize)]
pub struct GTArgs {
    /// The ID of the message to mark as read.
    message_id: String,
}

/// A tool for interacting with the Gmail API.
#[derive(Clone)]
pub struct GmailTool {
    hub: GmailHubType,
}

impl Tool for GmailTool {
    const NAME: &'static str = "gmail.tool";

    type Args = GTArgs;
    type Error = GmailToolError;
    type Output = ();

    /// Returns the definition of the tool.
    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Mark a specific message as read in Gmail by removing the UNREAD label."
                .to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "message_id": {
                        "type": "string",
                        "description": "The ID of the message to mark as read."
                    }
                },
                "required": ["message_id"]
            }),
        }
    }

    /// Calls the tool to mark a message as read.
    async fn call(&self, params: Self::Args) -> Result<Self::Output, Self::Error> {
        self.hub
            .users()
            .messages_modify(
                ModifyMessageRequest {
                    add_label_ids: None,
                    remove_label_ids: Some(vec!["UNREAD".to_string()]),
                },
                "me",
                &params.message_id,
            )
            .doit()
            .await
            .map_err(|e| GmailToolError::MarkUnreadError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::google_auth::{GConf, GoogleAuthFlow, InnerConf};
    use std::path::Path;

    #[tokio::test]
    async fn gmail_tool_call_succeeds() {
        // --- 1. Arrange ---
        // This test requires a valid message ID to run successfully.
        // Since we can't guarantee a message ID, this test primarily checks
        // that the tool can be built and the call method can be invoked
        // without panicking. A proper integration test would require a dedicated
        // test account with a known message.
        let message_id = "test_message_id_which_will_fail".to_string();

        let conf = GConf::from(Arc::new(InnerConf {
            credentials_path: Path::new("./tmp/credential.json").to_path_buf(),
            token_path: Path::new("./tmp/token.json").to_path_buf(),
            flow: GoogleAuthFlow::default(),
        }));

        let hub = Arc::new(ContextHub::new(conf));
        let builder = GmailToolBuilder::new(hub);
        let tool_result = builder.build().await;

        if tool_result.is_err() {
            println!("Skipping test because of missing credentials or auth error.");
            return;
        }

        let tool = tool_result.unwrap();
        let args = GTArgs { message_id };

        // --- 2. Act ---
        let result = tool.call(args).await;

        // --- 3. Assert ---
        // We expect an error because the message_id is invalid.
        // The important part is that the call didn't panic.
        assert!(result.is_err());
        match result.unwrap_err() {
            GmailToolError::MarkUnreadError(_) => {
                // This is the expected error.
            }
            e => panic!("Expected MarkUnreadError, but got {:?}", e),
        }
    }
}
