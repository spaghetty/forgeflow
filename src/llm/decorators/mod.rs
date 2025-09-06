/// LLM decorators for adding functionality to base LLM implementations.
///
/// This module contains decorator patterns that can wrap LLM implementations
/// to add additional functionality like retry logic, caching, logging, metrics,
/// rate limiting, and more.
///
/// Decorators follow the decorator pattern where they implement the LLM trait
/// themselves and wrap another LLM implementation, adding their functionality
/// transparently.
///
/// # Available Decorators
///
/// - **Retry Decorators**: Add automatic retry logic for transient failures
///
/// # Future Decorators
///
/// Planned decorators that could be added:
/// - **Caching**: Cache responses to avoid repeated calls
/// - **Logging**: Log all prompts and responses
/// - **Metrics**: Collect performance and usage metrics
/// - **Rate Limiting**: Enforce rate limits to prevent API abuse
/// - **Circuit Breaker**: Fail fast when downstream services are unhealthy
/// - **Timeout**: Add configurable timeouts to prevent hanging requests
pub mod retry;

// Re-export the main retry decorators for convenience
pub use retry::{BoxedRetryLLM, ManualRetryLLM, RetryableLLM};

// Note: BoxedRetryLLM is re-exported for completeness but is typically
// used internally by the LLM factory rather than directly by users.
