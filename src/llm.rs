pub mod retry;

// The `llm` module provides a trait for interacting with language models.

use async_trait::async_trait;
use rig::{agent::Agent as RigAgent, completion::CompletionModel}; // Alias rig's Agent to avoid name collision
use thiserror::Error;
use tracing::debug;

/// A custom error type for LLM operations.
#[derive(Error, Debug)]
pub enum LLMError {
    /// An error occurred while prompting the model.
    #[error("Failed to prompt the model: {0}")]
    PromptError(String),
}

/// A trait that defines the contract for any LLM processor our agent can use.
#[async_trait]
pub trait LLM: Send + Sync {
    /// Sends a text prompt to the language model and gets a response.
    async fn prompt(&mut self, text: String) -> Result<String, LLMError>;
}

/// The adapter implementation. This teaches our `LLM` trait how to use a `rig::Agent`.
#[async_trait]
impl<M> LLM for RigAgent<M>
where
    M: CompletionModel,
{
    async fn prompt(&mut self, text: String) -> Result<String, LLMError> {
        rig::completion::Prompt::prompt(self, text)
            .await
            .map(|response| response.to_string())
            .map_err(|e| {
                debug!("this is the error: {}", e);
                LLMError::PromptError(e.to_string())
            })
    }
}
