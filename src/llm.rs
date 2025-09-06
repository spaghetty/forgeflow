//! # ForgeFlow LLM Module
//!
//! This module provides a unified interface for working with Language Model (LLM) providers
//! in the ForgeFlow framework. It includes:
//!
//! - Core `LLM` trait for unified LLM interactions
//! - Configuration types for LLM behavior (retry, etc.)
//! - Decorators for adding functionality (retry, caching, metrics, etc.)
//! - Adapters for third-party LLM libraries
//! - Factory for transparent LLM creation with decorators
//!
//! # Quick Start
//!
//! ```rust
//! use forgeflow::llm::{LLM, LLMError};
//! use forgeflow::agent::AgentBuilder;
//!
//! // Any LLM that implements the LLM trait can be used
//! let llm: Box<dyn LLM> = Box::new(your_llm_implementation);
//!
//! // The AgentBuilder will automatically add retry by default
//! let agent = AgentBuilder::new()
//!     .with_model(llm)  // Retry added automatically
//!     .with_prompt_template("{{prompt}}")
//!     .build()?;
//! ```
//!
//! # Manual Decorator Usage
//!
//! For advanced users who want explicit control:
//!
//! ```rust
//! use forgeflow::llm::{LLM, RetryConfig, decorators::RetryableLLM};
//!
//! // Manual retry wrapping
//! let base_llm = your_llm_implementation;
//! let retryable_llm = RetryableLLM::new(base_llm, 3);
//! let llm: Box<dyn LLM> = Box::new(retryable_llm);
//! ```

// Core modules
pub mod config;
pub mod core;

// Implementation modules
pub mod adapters;
pub mod decorators;
pub mod factory;

// === Core Exports ===
// These are the main types users should interact with
pub use config::{RetryConfig, RetryStrategy};
pub use core::{LLM, LLMError};

// === Factory (Internal) ===
// Factory is used internally by AgentBuilder
pub(crate) use factory::LLMFactory;

// === Decorator Exports ===
// For users who want explicit decorator control
pub use decorators::{ManualRetryLLM, RetryableLLM};
