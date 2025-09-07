use crate::llm::core::{LLM, LLMError};
use async_trait::async_trait;
use rig::{agent::Agent as RigAgent, completion::CompletionModel};
use tracing::debug;

/// Adapter implementations for third-party LLM providers.
/// 
/// This module contains implementations of the `LLM` trait for various
/// third-party LLM libraries and services, allowing them to be used
/// seamlessly with the ForgeFlow framework.

/// Implementation of the `LLM` trait for `rig::Agent`.
/// 
/// This adapter allows any `rig::Agent` to be used as an LLM in ForgeFlow.
/// The `rig` library provides agents that can interact with various LLM 
/// providers like OpenAI, Anthropic, Google Gemini, and others.
/// 
/// # Example
/// 
/// ```rust,ignore
/// use forgeflow::llm::LLM;
/// use rig::{providers::openai, client::CompletionClient};
/// 
/// // Create a rig agent
/// let openai_client = openai::Client::from_env();
/// let agent = openai_client
///     .agent("gpt-4")
///     .preamble("You are a helpful assistant")
///     .build();
/// 
/// // Use it as an LLM in ForgeFlow
/// let mut llm: Box<dyn LLM> = Box::new(agent);
/// ```
/// 
/// # Thread Safety
/// 
/// The adapter maintains the thread safety requirements of the `LLM` trait
/// by leveraging rig's thread-safe implementations.
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
                debug!("Rig agent error: {}", e);
                LLMError::PromptError(e.to_string())
            })
    }
}

// Future: Add more adapters for other LLM libraries
// 
// Examples of what could be added:
// - Direct OpenAI client adapters
// - Hugging Face transformers adapters  
// - Local model adapters (llama.cpp, etc.)
// - Custom HTTP client adapters
// 
// Each would implement the LLM trait and provide seamless integration
// with the ForgeFlow framework.
