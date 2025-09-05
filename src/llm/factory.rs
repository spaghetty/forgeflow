use crate::llm::core::LLM;
use crate::llm::config::RetryConfig;
use crate::llm::decorators::BoxedRetryLLM;

/// Factory for creating LLM instances with optional decorators.
/// 
/// This factory provides a clean interface for wrapping base LLM implementations
/// with various decorators (like retry logic) based on configuration. It abstracts 
/// away the complexity of manually wrapping LLMs while providing transparent
/// functionality enhancement.
/// 
/// The factory uses the decorator pattern to add functionality to LLMs without
/// changing their interface, allowing for composition of multiple behaviors.
pub struct LLMFactory;

impl LLMFactory {
    /// Create an LLM instance with optional retry decoration.
    /// 
    /// This method takes a base LLM and optional retry configuration, returning
    /// either the base LLM (if retry is disabled) or a retry-decorated version.
    /// 
    /// # Arguments
    /// 
    /// * `base_llm` - The base LLM implementation to potentially wrap
    /// * `retry_config` - Optional retry configuration. If None, no retry is applied.
    ///                   If Some but with max_attempts = 0, no retry is applied.
    /// 
    /// # Returns
    /// 
    /// A Box<dyn LLM> that may or may not have retry functionality depending on config.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use forgeflow::llm::{LLMFactory, RetryConfig};
    /// 
    /// // No retry
    /// let llm = LLMFactory::create(base_llm, None);
    /// 
    /// // With retry
    /// let llm = LLMFactory::create(base_llm, Some(RetryConfig::default()));
    /// 
    /// // Retry disabled
    /// let llm = LLMFactory::create(base_llm, Some(RetryConfig::disabled()));
    /// ```
    pub fn create(
        base_llm: Box<dyn LLM>, 
        retry_config: Option<RetryConfig>
    ) -> Box<dyn LLM> {
        match retry_config {
            Some(config) if config.max_attempts > 0 => {
                tracing::debug!(
                    max_attempts = config.max_attempts,
                    base_delay_ms = config.base_delay.as_millis(),
                    strategy = ?config.strategy,
                    only_rate_limits = config.only_retry_rate_limits,
                    "Wrapping LLM with retry decorator"
                );
                Box::new(BoxedRetryLLM::new(base_llm, config.max_attempts))
            },
            Some(_) => {
                tracing::debug!("Retry config provided but max_attempts is 0, using base LLM without retry");
                base_llm
            },
            None => {
                tracing::debug!("No retry config provided, using base LLM without retry");
                base_llm
            }
        }
    }

    /// Create an LLM instance with default retry configuration.
    /// 
    /// This is a convenience method equivalent to calling `create(base_llm, Some(RetryConfig::default()))`.
    /// 
    /// # Arguments
    /// 
    /// * `base_llm` - The base LLM implementation to wrap
    /// 
    /// # Returns
    /// 
    /// A Box<dyn LLM> with default retry functionality applied.
    pub fn create_with_default_retry(base_llm: Box<dyn LLM>) -> Box<dyn LLM> {
        Self::create(base_llm, Some(RetryConfig::default()))
    }

    /// Create an LLM instance without any retry functionality.
    /// 
    /// This is a convenience method equivalent to calling `create(base_llm, None)`.
    /// Useful when you want to explicitly ensure no retry is applied.
    /// 
    /// # Arguments
    /// 
    /// * `base_llm` - The base LLM implementation to use as-is
    /// 
    /// # Returns
    /// 
    /// The original Box<dyn LLM> without any decorators.
    pub fn create_without_retry(base_llm: Box<dyn LLM>) -> Box<dyn LLM> {
        Self::create(base_llm, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::LLMError;
    use async_trait::async_trait;

    // Mock LLM for testing
    struct MockLLM {
        name: String,
    }

    impl MockLLM {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl LLM for MockLLM {
        async fn prompt(&mut self, prompt: String) -> Result<String, LLMError> {
            Ok(format!("{}: {}", self.name, prompt))
        }
    }

    #[tokio::test]
    async fn test_create_without_retry_config() {
        let base_llm = Box::new(MockLLM::new("base"));
        let mut llm = LLMFactory::create(base_llm, None);
        
        // Test that the LLM works by calling prompt
        let result = llm.prompt("test".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "base: test");
    }

    #[tokio::test]
    async fn test_create_with_retry_config() {
        let base_llm = Box::new(MockLLM::new("base"));
        let config = RetryConfig::default();
        let mut llm = LLMFactory::create(base_llm, Some(config));
        
        // The returned LLM should be wrapped with retry logic and still work
        let result = llm.prompt("test".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "base: test");
    }

    #[tokio::test] 
    async fn test_create_with_disabled_retry() {
        let base_llm = Box::new(MockLLM::new("base"));
        let config = RetryConfig::disabled();
        let mut llm = LLMFactory::create(base_llm, Some(config));
        
        // Should return the base LLM without retry wrapping since max_attempts = 0
        let result = llm.prompt("test".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "base: test");
    }

    #[tokio::test]
    async fn test_create_with_default_retry() {
        let base_llm = Box::new(MockLLM::new("base"));
        let mut llm = LLMFactory::create_with_default_retry(base_llm);
        
        // Should create LLM with default retry configuration and still work
        let result = llm.prompt("test".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "base: test");
    }

    #[tokio::test]
    async fn test_create_without_retry() {
        let base_llm = Box::new(MockLLM::new("base"));
        let mut llm = LLMFactory::create_without_retry(base_llm);
        
        // Should return the base LLM unchanged
        let result = llm.prompt("test".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "base: test");
    }

    #[test]
    fn test_factory_decision_logic() {
        // Test the core decision logic without actually creating LLMs
        
        // Case 1: None config should not add retry
        let should_retry_none = match None::<RetryConfig> {
            Some(config) if config.max_attempts > 0 => true,
            _ => false,
        };
        assert!(!should_retry_none);
        
        // Case 2: Some config with attempts > 0 should add retry
        let should_retry_enabled = match Some(RetryConfig::default()) {
            Some(config) if config.max_attempts > 0 => true,
            _ => false,
        };
        assert!(should_retry_enabled);
        
        // Case 3: Some config with attempts = 0 should not add retry
        let should_retry_disabled = match Some(RetryConfig::disabled()) {
            Some(config) if config.max_attempts > 0 => true,
            _ => false,
        };
        assert!(!should_retry_disabled);
    }
}
