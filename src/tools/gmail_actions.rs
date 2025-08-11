use crate::utils::google_auth::{GConf, gmail_auth};
use google_gmail1::api::{ModifyMessageRequest, Scope};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GmailToolError {
    #[error("Gmail authentication error for tools: {0}")]
    GmailAuthError(String),
    #[error("Failed to mark message as unread: {0}")]
    MarkUnreadError(String),
    #[error("Task spawn error: {0}")]
    SpawnError(String),
}

#[derive(Deserialize)]
pub struct GTArgs {
    message_id: String,
}

#[derive(Clone)]
pub struct GmailTool {
    gconf: GConf,
}

impl GmailTool {
    pub fn new(gconf: GConf) -> Self {
        Self { gconf }
    }
}

impl Tool for GmailTool {
    const NAME: &'static str = "gmail.tool";

    type Args = GTArgs;
    type Error = GmailToolError;
    type Output = ();

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
