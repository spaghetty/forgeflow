// The `Agent` module provides the core functionality for the Forgeflow framework.
// It defines the `Agent` struct, which is responsible for managing triggers, interacting with language models, and executing actions using tools.

use crate::llm::LLM;
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
use tracing::{debug, error, info, warn};

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
}

/// The `Agent` struct is the central component of the Forgeflow framework.
/// It is responsible for coordinating the other components and executing the main logic.
pub struct Agent {
    /// A vector of triggers that can initiate agent actions.
    triggers: Vec<Box<dyn Trigger>>,
    /// An optional shutdown handler that can be used to gracefully shut down the agent.
    shutdown_handler: Option<Box<dyn Shutdown>>,
    /// An optional language model that the agent can use to process events and generate responses.
    model: Option<Box<dyn LLM>>,
    /// An optional prompt template that the agent can use to generate prompts for the language model.
    prompt_template: Option<String>,
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
}

impl AgentBuilder {
    /// Creates a new `AgentBuilder`.
    pub fn new() -> Self {
        Self {
            triggers: Vec::new(),
            shutdown_handler: None,
            model: None,
            prompt_template: None,
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

    /// Builds the `Agent`.
    pub fn build(self) -> Result<Agent, AgentError> {
        let mut handlebars = TEngine::new();
        if let Some(template) = &self.prompt_template {
            handlebars.register_template_string("prompt", template)?;
        }

        Ok(Agent {
            triggers: self.triggers,
            shutdown_handler: self.shutdown_handler,
            model: self.model,
            prompt_template: self.prompt_template,
            handlebars,
            inflight: AtomicUsize::new(0),
        })
    }
}

impl Agent {
    /// Creates a new `Agent`.
    pub fn new() -> Result<Self, AgentError> {
        Ok(Agent {
            triggers: Vec::new(),
            shutdown_handler: None,
            model: None,
            prompt_template: None,
            handlebars: TEngine::new(),
            inflight: AtomicUsize::new(0),
        })
    }

    /// Sets the language model for the agent.
    pub fn with_model(mut self, model: Box<dyn LLM>) -> Self {
        self.model = Some(model);
        self
    }

    /// Sets the prompt template for the agent.
    pub fn with_prompt_template(mut self, template: String) -> Result<Self, AgentError> {
        self.handlebars
            .register_template_string("prompt", &template)?;
        self.prompt_template = Some(template);
        Ok(self)
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

    /// Runs the agent.
    pub async fn run(mut self) -> Result<(), AgentError> {
        let (_, event_rx, shutdown_tx, trigger_handles) = self.launch_triggers().await;

        if let Some(mut handler) = self.shutdown_handler.take() {
            tokio::select! {
                _ = self.event_loop(event_rx) => {
                    info!("Event loop completed normally");
                },
                _ = handler.wait_for_signal() => {
                    info!("External shutdown signal triggered termination");
                }
            }
        } else {
            self.event_loop(event_rx).await;
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
        if let (Some(provider_client), Some(template)) = (&mut self.model, &self.prompt_template) {
            let json_context = &json!(event);
            match self.handlebars.render_template(template, json_context) {
                Ok(prompt) => {
                    info!("Prompt: {}", prompt);
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
        } else {
            warn!("No model or prompt template configured, skipping LLM interaction.");
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

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
