// The `gmail_actions` module provides a tool for interacting with the Gmail API.

use crate::utils::google_auth::{GConf, gmail_auth};
use google_gmail1::api::{ModifyMessageRequest, Scope};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;

/// The `GmailToolError` enum defines the possible errors that can occur within the `GmailTool`.
#[derive(Debug, Error)]
pub enum GmailToolError {
    /// An error occurred while authenticating with the Gmail API.
    #[error("Gmail authentication error for tools: {0}")]
    GmailAuthError(String),
    /// An error occurred while marking a message as unread.
    #[error("Failed to mark message as unread: {0}")]
    MarkUnreadError(String),
    /// An error occurred while spawning a task.
    #[error("Task spawn error: {0}")]
    SpawnError(String),
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
    gconf: GConf,
}

impl GmailTool {
    /// Creates a new `GmailTool`.
    pub fn new(gconf: GConf) -> Self {
        Self { gconf }
    }
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
        let gconf = self.gconf.clone();
        let message_id = params.message_id.clone();

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let hub = gmail_auth(gconf, &[Scope::Modify])
                    .await
                    .map_err(|e| GmailToolError::GmailAuthError(e.to_string()))?;

                hub.users()
                    .messages_modify(
                        ModifyMessageRequest {
                            add_label_ids: None,
                            remove_label_ids: Some(vec!["UNREAD".to_string()]),
                        },
                        "me",
                        &message_id,
                    )
                    .doit()
                    .await
                    .map_err(|e| GmailToolError::MarkUnreadError(e.to_string()))?;
                Ok(())
            })
        })
        .await
        .map_err(|e| GmailToolError::SpawnError(e.to_string()))?
    }
}
