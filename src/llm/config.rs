use std::time::Duration;

/// Configuration for LLM retry behavior.
///
/// This struct defines how the LLM should behave when encountering errors,
/// specifically rate limiting (429) errors from LLM providers.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 means no retries)
    pub max_attempts: usize,
    /// Base delay for retry backoff strategy
    pub base_delay: Duration,
    /// The retry strategy to use
    pub strategy: RetryStrategy,
    /// Whether to only retry on rate limit (429) errors
    pub only_retry_rate_limits: bool,
}

/// Retry strategy for handling failed LLM requests.
#[derive(Debug, Clone)]
pub enum RetryStrategy {
    /// Fixed delay between retries
    Fixed,
    /// Exponential backoff without jitter
    ExponentialBackoff,
    /// Exponential backoff with jitter to avoid thundering herd
    ExponentialBackoffWithJitter,
}

impl Default for RetryConfig {
    /// Default retry configuration optimized for LLM rate limiting.
    /// 
    /// - 3 retry attempts (reasonable for rate limits)
    /// - 1 second base delay
    /// - Exponential backoff with jitter (production-safe)
    /// - Only retry on 429 rate limit errors (safe default)
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            strategy: RetryStrategy::ExponentialBackoffWithJitter,
            only_retry_rate_limits: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration with custom parameters.
    pub fn new(
        max_attempts: usize,
        base_delay: Duration,
        strategy: RetryStrategy,
    ) -> Self {
        Self {
            max_attempts,
            base_delay,
            strategy,
            only_retry_rate_limits: true,
        }
    }

    /// Create a configuration that retries all errors (not just rate limits).
    /// 
    /// **Warning**: This can mask real errors and should be used carefully.
    pub fn retry_all_errors(mut self) -> Self {
        self.only_retry_rate_limits = false;
        self
    }

    /// Create a configuration for aggressive retry (more attempts, shorter delays).
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_millis(500),
            strategy: RetryStrategy::ExponentialBackoffWithJitter,
            only_retry_rate_limits: true,
        }
    }

    /// Create a configuration for conservative retry (fewer attempts, longer delays).
    pub fn conservative() -> Self {
        Self {
            max_attempts: 2,
            base_delay: Duration::from_millis(2000),
            strategy: RetryStrategy::ExponentialBackoff,
            only_retry_rate_limits: true,
        }
    }

    /// Create a configuration with no retry (for explicit opt-out).
    pub fn disabled() -> Self {
        Self {
            max_attempts: 0,
            base_delay: Duration::from_millis(0),
            strategy: RetryStrategy::Fixed,
            only_retry_rate_limits: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.base_delay, Duration::from_millis(1000));
        assert!(matches!(config.strategy, RetryStrategy::ExponentialBackoffWithJitter));
        assert!(config.only_retry_rate_limits);
    }

    #[test]
    fn test_disabled_config() {
        let config = RetryConfig::disabled();
        assert_eq!(config.max_attempts, 0);
        assert_eq!(config.base_delay, Duration::from_millis(0));
    }

    #[test]
    fn test_aggressive_config() {
        let config = RetryConfig::aggressive();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.base_delay, Duration::from_millis(500));
        assert!(config.only_retry_rate_limits);
    }

    #[test]
    fn test_conservative_config() {
        let config = RetryConfig::conservative();
        assert_eq!(config.max_attempts, 2);
        assert_eq!(config.base_delay, Duration::from_millis(2000));
    }

    #[test]
    fn test_retry_all_errors() {
        let config = RetryConfig::default().retry_all_errors();
        assert!(!config.only_retry_rate_limits);
    }
}
