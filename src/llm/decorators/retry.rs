//! # LLM Retry Module
//!
//! This module provides retry functionality for LLM operations, specifically designed to handle
//! rate limiting and transient errors that are common when working with LLM APIs.
//!
//! ## Features
//!
//! - **Smart Retry Logic**: Only retries on 429 (rate limit) errors, not on other errors
//! - **Exponential Backoff**: Implements exponential backoff with jitter to avoid thundering herd
//! - **API-Aware Delays**: Respects retry delay hints from Google API error responses
//! - **Two Implementations**: Both tokio-retry based and manual retry implementations
//!
//! ## Usage Examples
//!
//! ### Basic Usage with RetryableLLM
//!
//! ```rust,ignore
//! use forgeflow::llm::retry::RetryableLLM;
//! use forgeflow::llm::LLM;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Wrap any LLM implementation with retry logic
//! let base_llm = /* your LLM implementation */;
//! let mut retryable_llm = RetryableLLM::new(base_llm, 3); // 3 retries
//!
//! // This will automatically retry on 429 errors
//! let response = retryable_llm.prompt("Hello, world!".to_string()).await?;
//! println!("Response: {}", response);
//! # Ok(())
//! # }
//! ```
//!
//! ### Manual Retry Control
//!
//! ```rust,ignore
//! use forgeflow::llm::retry::ManualRetryLLM;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let base_llm = /* your LLM implementation */;
//! let mut manual_retry_llm = ManualRetryLLM::new(
//!     base_llm,
//!     5,                              // max retries
//!     Duration::from_millis(500)      // base delay
//! );
//!
//! let response = manual_retry_llm.prompt("Hello, world!".to_string()).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! The retry logic specifically handles:
//! - **429 Errors**: Rate limiting - will retry with exponential backoff
//! - **Google API Retry Info**: Respects `retryDelay` fields in error responses
//! - **Other Errors**: Permanent failures that should not be retried (4xx, 5xx except 429)
//!

use crate::llm::core::{LLM, LLMError};
use async_trait::async_trait;
use serde_json::Value;
use std::time::Duration;

/// A wrapper for an LLM that adds retry logic using exponential backoff.
///
/// This implementation provides automatic retry functionality for LLM operations,
/// specifically designed to handle rate limiting (429 errors) from LLM APIs.
/// It uses exponential backoff with jitter and respects retry delay hints from
/// API error responses.
///
/// # Example
///
/// ```rust,ignore
/// use forgeflow::llm::retry::RetryableLLM;
/// # use forgeflow::llm::{LLM, LLMError};
/// # use async_trait::async_trait;
/// # struct MockLLM;
/// # #[async_trait]
/// # impl LLM for MockLLM {
/// #     async fn prompt(&mut self, prompt: String) -> Result<String, LLMError> {
/// #         Ok("response".to_string())
/// #     }
/// # }
///
/// # async fn example() -> Result<(), LLMError> {
/// let base_llm = MockLLM;
/// let mut retryable_llm = RetryableLLM::new(base_llm, 3);
///
/// let response = retryable_llm.prompt("Hello!".to_string()).await?;
/// # Ok(())
/// # }
/// ```
pub struct RetryableLLM<L: LLM> {
    llm: L,
    retries: usize,
}

impl<L: LLM> RetryableLLM<L> {
    /// Creates a new `RetryableLLM` with the specified number of retries.
    ///
    /// # Arguments
    ///
    /// * `llm` - The underlying LLM implementation to wrap
    /// * `retries` - Maximum number of retry attempts (0 means no retries)
    pub fn new(llm: L, retries: usize) -> Self {
        Self { llm, retries }
    }

    /// Determines if an error should be retried based on the error content.
    ///
    /// This method analyzes the error to determine if it represents a transient
    /// failure that should be retried. Currently, it only considers 429 (rate limit)
    /// errors as retryable.
    ///
    /// # Arguments
    ///
    /// * `error` - The LLM error to analyze
    ///
    /// # Returns
    ///
    /// `true` if the error is a 429 rate limit error, `false` otherwise
    fn should_retry(error: &LLMError) -> bool {
        let error_str = error.to_string();

        let json_str = error_str
            .strip_prefix("Failed to prompt the model: ")
            .unwrap_or(&error_str);

        if let Ok(json) = serde_json::from_str::<Value>(json_str) {
            if let Some(code) = json["error"]["code"].as_i64() {
                return code == 429; // Retry only on rate limit errors
            }
        }
        false // Don't retry other errors by default
    }

    /// Extracts and waits for the retry delay from Google API error response.
    ///
    /// This method parses the error response looking for Google API retry information.
    /// If a `retryDelay` is specified in the error details, it will sleep for that
    /// duration. This helps respect the API's suggested retry timing.
    ///
    /// # Arguments
    ///
    /// * `error` - The LLM error that may contain retry delay information
    async fn handle_retry_delay(error: &LLMError) {
        let error_str = error.to_string();

        let json_str = error_str
            .strip_prefix("Failed to prompt the model: ")
            .unwrap_or(&error_str);

        if let Ok(json) = serde_json::from_str::<Value>(json_str) {
            if let Some(details) = json["error"]["details"].as_array() {
                for detail in details {
                    if detail["@type"].as_str() == Some("type.googleapis.com/google.rpc.RetryInfo")
                    {
                        if let Some(retry_delay) = detail["retryDelay"].as_str() {
                            if let Ok(duration) = humantime::parse_duration(retry_delay) {
                                tokio::time::sleep(duration).await;
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[async_trait]
impl<L: LLM + Send + Sync> LLM for RetryableLLM<L> {
    async fn prompt(&mut self, prompt: String) -> Result<String, LLMError> {
        let mut last_error = None;
        let base_delay = Duration::from_millis(1000);

        for attempt in 0..=self.retries {
            match self.llm.prompt(prompt.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    let error = last_error.as_ref().unwrap();

                    // Don't retry on the last attempt or if error is not retryable
                    if attempt == self.retries || !Self::should_retry(error) {
                        break;
                    }

                    // Handle retry delay from API response, or use exponential backoff
                    Self::handle_retry_delay(error).await;

                    // Add exponential backoff with simple jitter
                    let delay = base_delay * (2_u32.pow(attempt as u32));
                    let jitter_ms = (attempt as u64 * 50) % 200; // Simple jitter based on attempt
                    let jitter_delay = Duration::from_millis(delay.as_millis() as u64 + jitter_ms);
                    tokio::time::sleep(jitter_delay).await;
                }
            }
        }

        Err(last_error.unwrap())
    }
}

/// Alternative implementation using manual retry logic for more control.
///
/// This implementation provides fine-grained control over retry behavior,
/// allowing you to specify the base delay and maximum retry count. It uses
/// exponential backoff and respects API-provided retry delays.
///
/// # Example
///
/// ```rust,ignore
/// use forgeflow::llm::retry::ManualRetryLLM;
/// use std::time::Duration;
/// # use forgeflow::llm::{LLM, LLMError};
/// # use async_trait::async_trait;
/// # struct MockLLM;
/// # #[async_trait]
/// # impl LLM for MockLLM {
/// #     async fn prompt(&mut self, prompt: String) -> Result<String, LLMError> {
/// #         Ok("response".to_string())
/// #     }
/// # }
///
/// # async fn example() -> Result<(), LLMError> {
/// let base_llm = MockLLM;
/// let mut manual_retry_llm = ManualRetryLLM::new(
///     base_llm,
///     5,                              // max retries
///     Duration::from_millis(1000)     // base delay
/// );
///
/// let response = manual_retry_llm.prompt("Hello!".to_string()).await?;
/// # Ok(())
/// # }
/// ```
pub struct ManualRetryLLM<L: LLM> {
    llm: L,
    max_retries: usize,
    base_delay: Duration,
}

impl<L: LLM> ManualRetryLLM<L> {
    /// Creates a new `ManualRetryLLM` with specified retry parameters.
    ///
    /// # Arguments
    ///
    /// * `llm` - The underlying LLM implementation to wrap
    /// * `max_retries` - Maximum number of retry attempts
    /// * `base_delay` - Base delay for exponential backoff
    pub fn new(llm: L, max_retries: usize, base_delay: Duration) -> Self {
        Self {
            llm,
            max_retries,
            base_delay,
        }
    }

    /// Determines if an error should be retried.
    ///
    /// # Arguments
    ///
    /// * `error` - The LLM error to analyze
    ///
    /// # Returns
    ///
    /// `true` if the error is retryable (429 rate limit), `false` otherwise
    fn should_retry(error: &LLMError) -> bool {
        let error_str = error.to_string();

        let json_str = error_str
            .strip_prefix("Failed to prompt the model: ")
            .unwrap_or(&error_str);

        if let Ok(json) = serde_json::from_str::<Value>(json_str) {
            if let Some(code) = json["error"]["code"].as_i64() {
                return code == 429; // Retry only on rate limit errors
            }
        }
        false
    }

    /// Extracts and waits for the retry delay specified in the error.
    ///
    /// This method will parse the error for Google API retry information and
    /// wait for the specified delay. If no delay is found, it uses the provided
    /// default delay.
    ///
    /// # Arguments
    ///
    /// * `error` - The error that may contain retry delay information
    /// * `default_delay` - Fallback delay if no retry delay is specified
    async fn wait_for_retry_delay(error: &LLMError, default_delay: Duration) {
        let error_str = error.to_string();
        let mut delay = default_delay;

        let json_str = error_str
            .strip_prefix("Failed to prompt the model: ")
            .unwrap_or(&error_str);

        if let Ok(json) = serde_json::from_str::<Value>(json_str) {
            if let Some(details) = json["error"]["details"].as_array() {
                for detail in details {
                    if detail["@type"].as_str() == Some("type.googleapis.com/google.rpc.RetryInfo")
                    {
                        if let Some(retry_delay) = detail["retryDelay"].as_str() {
                            if let Ok(parsed_delay) = humantime::parse_duration(retry_delay) {
                                delay = parsed_delay;
                                break;
                            }
                        }
                    }
                }
            }
        }

        tokio::time::sleep(delay).await;
    }
}

#[async_trait]
impl<L: LLM + Send + Sync> LLM for ManualRetryLLM<L> {
    async fn prompt(&mut self, prompt: String) -> Result<String, LLMError> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.llm.prompt(prompt.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    let error = last_error.as_ref().unwrap();

                    // Don't retry on the last attempt or if error is not retryable
                    if attempt == self.max_retries || !Self::should_retry(error) {
                        break;
                    }

                    // Calculate exponential backoff delay
                    let delay = self.base_delay * (2_u32.pow(attempt as u32));
                    Self::wait_for_retry_delay(error, delay).await;
                }
            }
        }

        Err(last_error.unwrap())
    }
}

/// A retry decorator specifically designed for boxed LLM trait objects.
///
/// This is different from RetryableLLM which works with concrete types.
/// BoxedRetryLLM wraps a Box<dyn LLM> and adds retry functionality.
/// This decorator is typically used internally by the LLM factory.
pub struct BoxedRetryLLM {
    inner: Box<dyn LLM>,
    max_attempts: usize,
}

impl BoxedRetryLLM {
    /// Create a new BoxedRetryLLM wrapper.
    pub fn new(inner: Box<dyn LLM>, max_attempts: usize) -> Self {
        Self {
            inner,
            max_attempts,
        }
    }

    /// Determines if an error should be retried based on the error content.
    fn should_retry(error: &LLMError) -> bool {
        let error_str = error.to_string();

        let json_str = error_str
            .strip_prefix("Failed to prompt the model: ")
            .unwrap_or(&error_str);

        if let Ok(json) = serde_json::from_str::<Value>(json_str) {
            if let Some(code) = json["error"]["code"].as_i64() {
                return code == 429; // Retry only on rate limit errors
            }
        }
        false // Don't retry other errors by default
    }

    /// Extracts and waits for the retry delay from Google API error response.
    async fn handle_retry_delay(error: &LLMError) {
        let error_str = error.to_string();

        let json_str = error_str
            .strip_prefix("Failed to prompt the model: ")
            .unwrap_or(&error_str);

        if let Ok(json) = serde_json::from_str::<Value>(json_str) {
            if let Some(details) = json["error"]["details"].as_array() {
                for detail in details {
                    if detail["@type"].as_str() == Some("type.googleapis.com/google.rpc.RetryInfo")
                    {
                        if let Some(retry_delay) = detail["retryDelay"].as_str() {
                            if let Ok(duration) = humantime::parse_duration(retry_delay) {
                                tokio::time::sleep(duration).await;
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[async_trait]
impl LLM for BoxedRetryLLM {
    async fn prompt(&mut self, prompt: String) -> Result<String, LLMError> {
        let mut last_error = None;
        let base_delay = Duration::from_millis(1000);

        for attempt in 0..=self.max_attempts {
            match self.inner.prompt(prompt.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    let error = last_error.as_ref().unwrap();

                    // Don't retry on the last attempt or if error is not retryable
                    if attempt == self.max_attempts || !Self::should_retry(error) {
                        break;
                    }

                    // Handle retry delay from API response, or use exponential backoff
                    Self::handle_retry_delay(error).await;

                    // Add exponential backoff with simple jitter
                    let delay = base_delay * (2_u32.pow(attempt as u32));
                    let jitter_ms = (attempt as u64 * 50) % 200; // Simple jitter based on attempt
                    let jitter_delay = Duration::from_millis(delay.as_millis() as u64 + jitter_ms);
                    tokio::time::sleep(jitter_delay).await;
                }
            }
        }

        Err(last_error.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockLLM {
        call_count: Arc<AtomicUsize>,
        error_on_call: Option<i64>,
        fail_first_n: Option<usize>,
    }

    impl MockLLM {
        fn new(call_count: Arc<AtomicUsize>) -> Self {
            Self {
                call_count,
                error_on_call: None,
                fail_first_n: None,
            }
        }

        fn with_error(mut self, error_code: i64) -> Self {
            self.error_on_call = Some(error_code);
            self
        }

        fn fail_first_n_calls(mut self, n: usize) -> Self {
            self.fail_first_n = Some(n);
            self
        }
    }

    #[async_trait]
    impl LLM for MockLLM {
        async fn prompt(&mut self, _prompt: String) -> Result<String, LLMError> {
            let count = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;

            // Handle fail_first_n scenario
            if let Some(fail_count) = self.fail_first_n {
                if count <= fail_count {
                    let error_json = serde_json::json!({
                        "error": {
                            "code": 429,
                            "message": "Rate limit exceeded",
                            "status": "RESOURCE_EXHAUSTED",
                            "details": [{
                                "@type": "type.googleapis.com/google.rpc.RetryInfo",
                                "retryDelay": "100ms"
                            }]
                        }
                    });
                    return Err(LLMError::PromptError(error_json.to_string()));
                }
                return Ok("Success after retries".to_string());
            }

            // Handle error_on_call scenario
            if let Some(error_code) = self.error_on_call {
                let error_json = serde_json::json!({
                    "error": {
                        "code": error_code,
                        "message": "An error occurred.",
                        "status": if error_code == 429 { "RESOURCE_EXHAUSTED" } else { "INTERNAL" }
                    }
                });
                Err(LLMError::PromptError(error_json.to_string()))
            } else {
                Ok("Success".to_string())
            }
        }
    }

    #[tokio::test]
    async fn test_no_retry_on_success() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let mock_llm = MockLLM::new(call_count.clone());
        let mut retryable_llm = RetryableLLM::new(mock_llm, 3);

        let result = retryable_llm.prompt("test".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_on_429_error() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let mock_llm = MockLLM::new(call_count.clone()).with_error(429);
        let mut retryable_llm = RetryableLLM::new(mock_llm, 3);

        let result = retryable_llm.prompt("test".to_string()).await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 4); // 1 initial call + 3 retries
    }

    #[tokio::test]
    async fn test_no_retry_on_other_error() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let mock_llm = MockLLM::new(call_count.clone()).with_error(500);
        let mut retryable_llm = RetryableLLM::new(mock_llm, 3);

        let result = retryable_llm.prompt("test".to_string()).await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // No retries for non-429 errors
    }

    #[tokio::test]
    async fn test_success_after_retries() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let mock_llm = MockLLM::new(call_count.clone()).fail_first_n_calls(2);
        let mut retryable_llm = RetryableLLM::new(mock_llm, 3);

        let result = retryable_llm.prompt("test".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success after retries");
        assert_eq!(call_count.load(Ordering::SeqCst), 3); // 2 failed + 1 success
    }

    #[tokio::test]
    async fn test_manual_retry_success() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let mock_llm = MockLLM::new(call_count.clone());
        let mut manual_retry_llm = ManualRetryLLM::new(mock_llm, 3, Duration::from_millis(10));

        let result = manual_retry_llm.prompt("test".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_manual_retry_on_429() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let mock_llm = MockLLM::new(call_count.clone()).fail_first_n_calls(2);
        let mut manual_retry_llm = ManualRetryLLM::new(mock_llm, 3, Duration::from_millis(10));

        let result = manual_retry_llm.prompt("test".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success after retries");
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_manual_retry_no_retry_on_500() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let mock_llm = MockLLM::new(call_count.clone()).with_error(500);
        let mut manual_retry_llm = ManualRetryLLM::new(mock_llm, 3, Duration::from_millis(10));

        let result = manual_retry_llm.prompt("test".to_string()).await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // No retries for non-429 errors
    }
}
