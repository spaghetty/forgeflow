//! # ForgeFlow: A Rust-based framework for building autonomous agents.

/// The `agent` module provides the core functionality for the Forgeflow framework.
pub mod agent;
/// The `llm` module provides a trait for interacting with language models.
pub mod llm;
/// The `shutdown` module provides a trait for gracefully shutting down the agent.
pub mod shutdown;
/// The `tools` module provides a collection of tools that can be used by the agent.
pub mod tools;
/// The `triggers` module provides a collection of triggers that can be used to initiate agent actions.
pub mod triggers;
/// The `utils` module provides utility functions for the framework.
pub mod utils;

pub use tools::{
    DailySummaryWriter, DailySummaryWriterBuilder, GmailTool, GmailToolBuilder, SimpleFileWriter,
    SimpleFileWriterBuilder,
};
pub use triggers::{
    GmailWatchTrigger, GmailWatchTriggerBuilder, PollTrigger, PollTriggerBuilder,
    TelegramBotTrigger, TelegramBotTriggerBuilder,
};
pub use utils::context_hub::ContextHub;
