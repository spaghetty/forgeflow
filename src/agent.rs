// The `Agent` module provides the core functionality for the Forgeflow framework.
// It defines the `Agent` struct, which is responsible for managing triggers, interacting with language models, and executing actions using tools.
use crate::llm::{LLM, LLMFactory, RetryConfig};
use crate::shutdown::Shutdown;
use crate::triggers::{Trigger, event::TEvent};
use crate::utils::{TEngine, TEngineError};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
//use tokio_util::task::TaskTracker;
use tracing::{debug, error, info};

/// The `AgentError` enum defines the possible errors that can occur within the `Agent`.
#[derive(Error, Debug)]
pub enum AgentError {
    /// An I/O error occurred.
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    /// An error occurred within the `rig` crate.
    #[error("Rig error")]
    RigError(),
    /// An error occurred while rendering a Handlebars template.
    #[error("Handlebars template error")]
    TemplateError(#[from] TEngineError),
    /// An error occurred while building the agent.
    #[error("Agent build error: {0}")]
    BuildError(String),
}

/// The `Agent` struct is the central component of the Forgeflow framework.
/// It is responsible for coordinating the other components and executing the main logic.
pub struct Agent {
    /// A vector of triggers that can initiate agent actions.
    triggers: Vec<Box<dyn Trigger>>,
    /// An optional shutdown handler that can be used to gracefully shut down the agent.
    shutdown_handler: Box<dyn Shutdown>,
    /// An optional language model that the agent can use to process events and generate responses.
    model: Box<dyn LLM>,
    /// An optional prompt template that the agent can use to generate prompts for the language model.
    prompt_template: String,
    /// The Handlebars template engine used by the agent.
    handlebars: TEngine,
    /// An atomic counter for the number of in-flight requests.
    inflight: AtomicUsize,
}

/// The `AgentBuilder` struct is used to construct an `Agent`.
pub struct AgentBuilder {
    triggers: Vec<Box<dyn Trigger>>,
    shutdown_handler: Option<Box<dyn Shutdown>>,
    model: Option<Box<dyn LLM>>,
    prompt_template: Option<String>,
    retry_config: Option<RetryConfig>,
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentBuilder {
    /// Creates a new `AgentBuilder`.
    pub fn new() -> Self {
        Self {
            triggers: Vec::new(),
            shutdown_handler: None,
            model: None,
            prompt_template: None,
            retry_config: None,
        }
    }

    /// Sets the language model for the agent.
    pub fn with_model(mut self, model: Box<dyn LLM>) -> Self {
        self.model = Some(model);
        self
    }

    /// Sets the prompt template for the agent.
    pub fn with_prompt_template(mut self, template: String) -> Self {
        self.prompt_template = Some(template);
        self
    }

    /// Adds a trigger to the agent.
    pub fn add_trigger(mut self, t: Box<dyn Trigger>) -> Self {
        self.triggers.push(t);
        self
    }

    /// Sets the shutdown handler for the agent.
    pub fn with_shutdown_handler(mut self, handler: impl Shutdown + 'static) -> Self {
        self.shutdown_handler = Some(Box::new(handler));
        self
    }

    /// Enable retry with default configuration.
    ///
    /// This enables automatic retry for rate limit (429) errors with sensible defaults:
    /// - 3 retry attempts
    /// - 1 second base delay
    /// - Exponential backoff with jitter
    /// - Only retries on rate limit errors
    pub fn with_retry(mut self) -> Self {
        self.retry_config = Some(RetryConfig::default());
        self
    }

    /// Configure retry behavior with custom settings.
    ///
    /// # Example
    /// ```rust,ignore
    /// use forgeflow::llm::{RetryConfig, RetryStrategy};
    /// use std::time::Duration;
    ///
    /// let config = RetryConfig::new(
    ///     5, // max attempts
    ///     Duration::from_millis(500), // base delay
    ///     RetryStrategy::ExponentialBackoffWithJitter
    /// );
    ///
    /// let agent = AgentBuilder::new()
    ///     .with_retry_config(config)
    ///     .build()?;
    /// ```
    pub fn with_retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = Some(config);
        self
    }

    /// Explicitly disable retry functionality.
    ///
    /// By default, retry is enabled. Use this method to explicitly opt-out.
    /// This is equivalent to `.with_retry_config(RetryConfig::disabled())`.
    pub fn without_retry(mut self) -> Self {
        self.retry_config = Some(RetryConfig::disabled());
        self
    }

    /// Builds the `Agent`.
    pub fn build(self) -> Result<Agent, AgentError> {
        if self.model.is_none() {
            return Err(AgentError::BuildError("A model is required.".to_string()));
        }

        let mut handlebars = TEngine::new();
        if let Some(template) = &self.prompt_template {
            handlebars.register_template_string("prompt", template)?;
        } else {
            return Err(AgentError::BuildError(
                "A prompt template is required.".to_string(),
            ));
        }

        let shutdown_handler = self
            .shutdown_handler
            .unwrap_or_else(|| Box::new(crate::shutdown::CtrlCShutdown::new()));

        // Apply retry configuration: default is to enable retry unless explicitly configured otherwise
        let retry_config = self.retry_config.unwrap_or_else(|| {
            tracing::debug!("No retry configuration specified, using default retry behavior");
            RetryConfig::default()
        });

        // Use the LLM factory to transparently apply retry configuration
        let base_model = self.model.unwrap();
        let final_model = LLMFactory::create(base_model, Some(retry_config));

        Ok(Agent {
            triggers: self.triggers,
            shutdown_handler,
            model: final_model,
            prompt_template: self.prompt_template.unwrap(),
            handlebars,
            inflight: AtomicUsize::new(0),
        })
    }
}

impl Agent {
    /// Runs the agent.
    pub async fn run(mut self) -> Result<(), AgentError> {
        let (_, event_rx, shutdown_tx, trigger_handles) = self.launch_triggers().await;
        let mut shutdown_handler = self.shutdown_handler.clone();

        tokio::select! {
            _ = self.event_loop(event_rx) => {
                info!("Event loop completed normally");
            },
            _ = shutdown_handler.wait_for_signal() => {
                info!("External shutdown signal triggered termination");
            }
        }

        self.shutdown_triggers(shutdown_tx, trigger_handles).await;

        info!("Agent has shut down gracefully");
        Ok(())
    }

    /// The main event loop for the agent.
    async fn event_loop(&mut self, mut event_rx: mpsc::Receiver<TEvent>) {
        info!("Agent event loop started, waiting for events");
        while let Some(event) = event_rx.recv().await {
            info!(event_name = %event.name, "Received event");

            self.process_single_event(event).await;
        }
        debug!("Event loop terminated - no more events to process");
    }

    /// Processes a single event.
    async fn process_single_event(&mut self, event: TEvent) {
        let provider_client = &mut self.model;
        let template = &self.prompt_template;
        let json_context = &json!(event);
        match self.handlebars.render_template(template, json_context) {
            Ok(prompt) => {
                debug!("Prompt: {}", prompt);
                self.inflight.fetch_add(1, Ordering::Relaxed);
                let response = provider_client.prompt(prompt).await;
                self.inflight.fetch_sub(1, Ordering::Relaxed);
                match response {
                    Ok(response) => info!("here we are: {}", response),
                    Err(x) => error!("troubles here {}", x),
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to render prompt template");
            }
        }
    }

    /// Launches the triggers for the agent.
    async fn launch_triggers(
        &self,
    ) -> (
        mpsc::Sender<TEvent>,
        mpsc::Receiver<TEvent>,
        broadcast::Sender<()>,
        Vec<JoinHandle<()>>,
    ) {
        let (event_tx, event_rx) = mpsc::channel(100);
        let (shutdown_tx, _) = broadcast::channel(1);
        let mut trigger_handles = Vec::new();

        info!(trigger_count = self.triggers.len(), "Launching triggers");
        for (index, trigger) in self.triggers.iter().enumerate() {
            let shutdown_rx = shutdown_tx.subscribe();
            match trigger.launch(event_tx.clone(), shutdown_rx).await {
                Ok(handle) => {
                    debug!(trigger_index = index, "Trigger launched successfully");
                    trigger_handles.push(handle);
                }
                Err(e) => {
                    error!(trigger_index = index, error = %e, "Failed to launch trigger");
                }
            }
        }
        info!(
            launched_count = trigger_handles.len(),
            "All triggers launched"
        );

        (event_tx, event_rx, shutdown_tx, trigger_handles)
    }

    /// Shuts down the triggers for the agent.
    async fn shutdown_triggers(
        &self,
        shutdown_tx: broadcast::Sender<()>,
        trigger_handles: Vec<JoinHandle<()>>,
    ) {
        info!(
            trigger_count = trigger_handles.len(),
            "Sending shutdown signal to all triggers"
        );
        let _ = shutdown_tx.send(());
        debug!("Waiting for triggers to terminate");
        for (index, handle) in trigger_handles.into_iter().enumerate() {
            if let Err(e) = handle.await {
                error!(
                    trigger_index = index,
                    error = %e,
                    "Error waiting for trigger to terminate"
                );
            } else {
                debug!(trigger_index = index, "Trigger terminated successfully");
            }
        }
        info!("All triggers have been shut down");
        let residual = self.inflight.load(Ordering::Relaxed);
        if residual != 0 {
            info!("residual inflight process: {}", residual);
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
        info!(
            "waited for inflight request to complete, killed {}",
            self.inflight.load(Ordering::Relaxed)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{RetryConfig, RetryStrategy};
    use std::time::Duration;

    // Mock LLM for testing
    struct MockLLM;

    #[async_trait::async_trait]
    impl LLM for MockLLM {
        async fn prompt(&mut self, _prompt: String) -> Result<String, crate::llm::LLMError> {
            Ok("test response".to_string())
        }
    }

    #[test]
    fn test_agent_builder_new_has_no_retry_config() {
        let builder = AgentBuilder::new();
        assert!(builder.retry_config.is_none());
    }

    #[test]
    fn test_agent_builder_with_retry() {
        let builder = AgentBuilder::new().with_retry();
        assert!(builder.retry_config.is_some());
        let config = builder.retry_config.unwrap();
        assert_eq!(config.max_attempts, 3); // Default value
    }

    #[test]
    fn test_agent_builder_with_custom_retry_config() {
        let custom_config = RetryConfig::new(5, Duration::from_millis(2000), RetryStrategy::Fixed);
        let builder = AgentBuilder::new().with_retry_config(custom_config.clone());
        assert!(builder.retry_config.is_some());
        let stored_config = builder.retry_config.unwrap();
        assert_eq!(stored_config.max_attempts, 5);
        assert_eq!(stored_config.base_delay, Duration::from_millis(2000));
    }

    #[test]
    fn test_agent_builder_without_retry() {
        let builder = AgentBuilder::new().without_retry();
        assert!(builder.retry_config.is_some());
        let config = builder.retry_config.unwrap();
        assert_eq!(config.max_attempts, 0); // Disabled
    }

    #[test]
    fn test_agent_builder_build_applies_default_retry() {
        // This test verifies that build() applies default retry when none is specified
        let mock_llm = MockLLM;
        let builder = AgentBuilder::new()
            .with_model(Box::new(mock_llm))
            .with_prompt_template("test template".to_string());

        // Should not panic and should build successfully
        // The default retry config should be applied internally
        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
