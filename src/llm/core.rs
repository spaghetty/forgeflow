use async_trait::async_trait;
use thiserror::Error;

/// A custom error type for LLM operations.
///
/// This error type provides a consistent interface for handling failures
/// that can occur during LLM interactions, regardless of the underlying
/// LLM provider or implementation.
#[derive(Error, Debug)]
pub enum LLMError {
    /// An error occurred while prompting the model.
    /// 
    /// This typically wraps the underlying error from the LLM provider,
    /// providing a consistent error format across different implementations.
    #[error("Failed to prompt the model: {0}")]
    PromptError(String),
}

/// A trait that defines the contract for any LLM processor our agent can use.
///
/// This trait provides a unified interface for interacting with language models,
/// allowing the ForgeFlow framework to work with any LLM provider that implements
/// this trait. The trait is designed to be simple yet flexible, focusing on the
/// core prompt-response interaction pattern.
///
/// # Examples
///
/// ```rust
/// use forgeflow::llm::{LLM, LLMError};
/// use async_trait::async_trait;
///
/// struct MockLLM;
///
/// #[async_trait]
/// impl LLM for MockLLM {
///     async fn prompt(&mut self, text: String) -> Result<String, LLMError> {
///         Ok(format!("Mock response to: {}", text))
///     }
/// }
/// ```
///
/// # Thread Safety
///
/// The trait requires `Send + Sync` to ensure LLM implementations can be safely
/// used across thread boundaries in async contexts.
#[async_trait]
pub trait LLM: Send + Sync {
    /// Sends a text prompt to the language model and gets a response.
    ///
    /// This is the core method of the LLM trait. Implementations should:
    /// 
    /// 1. Send the provided prompt to their underlying LLM service
    /// 2. Wait for and retrieve the response
    /// 3. Return the response as a String, or an error if something went wrong
    ///
    /// # Arguments
    ///
    /// * `text` - The prompt text to send to the language model
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The response from the language model
    /// * `Err(LLMError)` - An error occurred during the prompt operation
    ///
    /// # Errors
    ///
    /// This method should return `LLMError::PromptError` for any failures
    /// during the prompt-response cycle, including:
    /// 
    /// * Network errors
    /// * API rate limiting  
    /// * Invalid responses
    /// * Authentication failures
    /// * Service unavailability
    async fn prompt(&mut self, text: String) -> Result<String, LLMError>;
}
