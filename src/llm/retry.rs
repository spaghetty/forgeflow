use crate::llm::{LLM, LLMError};
use async_trait::async_trait;
use serde_json::Value;
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};

/// A wrapper for an LLM that adds retry logic.
pub struct RetryableLLM<L: LLM> {
    llm: L,
    retries: usize,
}

impl<L: LLM> RetryableLLM<L> {
    /// Creates a new `RetryableLLM`.
    pub fn new(llm: L, retries: usize) -> Self {
        Self { llm, retries }
    }
}

#[async_trait]
impl<L: LLM + Send + Sync> LLM for RetryableLLM<L> {
    async fn prompt(&mut self, prompt: String) -> Result<String, LLMError> {
        let strategy = ExponentialBackoff::from_millis(1000)
            .map(jitter)
            .take(self.retries);

        Retry::spawn(strategy, || async {
            match self.llm.prompt(prompt.clone()).await {
                Ok(result) => Ok(result),
                Err(e) => {
                    let error_str = e.to_string();
                    if let Ok(json) = serde_json::from_str::<Value>(&error_str) {
                        if json["error"]["code"].as_i64() == Some(429) {
                            if let Some(details) = json["error"]["details"].as_array() {
                                for detail in details {
                                    if detail["@type"].as_str()
                                        == Some("type.googleapis.com/google.rpc.RetryInfo")
                                    {
                                        if let Some(retry_delay) = detail["retryDelay"].as_str() {
                                            if let Ok(duration) =
                                                humantime::parse_duration(retry_delay)
                                            {
                                                tokio::time::sleep(duration).await;
                                            }
                                        }
                                    }
                                }
                            }
                            return Err(RetryError::Transient(e));
                        }
                    }
                    Err(RetryError::Permanent(e))
                }
            }
        })
        .await
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
    }

    #[async_trait]
    impl LLM for MockLLM {
        async fn prompt(&mut self, _prompt: String) -> Result<String, LLMError> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            if let Some(error_code) = self.error_on_call {
                let error_json = serde_json::json!({
                    "error": {
                        "code": error_code,
                        "message": "An error occurred.",
                        "status": "RESOURCE_EXHAUSTED"
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
        let mock_llm = MockLLM {
            call_count: call_count.clone(),
            error_on_call: None,
        };
        let mut retryable_llm = RetryableLLM::new(mock_llm, 3);

        let result = retryable_llm.prompt("test".to_string()).await;

        assert!(result.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_on_429_error() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let mock_llm = MockLLM {
            call_count: call_count.clone(),
            error_on_call: Some(429),
        };
        let mut retryable_llm = RetryableLLM::new(mock_llm, 3);

        let result = retryable_llm.prompt("test".to_string()).await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 4); // 1 initial call + 3 retries
    }

    #[tokio::test]
    async fn test_no_retry_on_other_error() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let mock_llm = MockLLM {
            call_count: call_count.clone(),
            error_on_call: Some(500),
        };
        let mut retryable_llm = RetryableLLM::new(mock_llm, 3);

        let result = retryable_llm.prompt("test".to_string()).await;

        assert!(result.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }
}
