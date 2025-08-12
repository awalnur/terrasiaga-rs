/// Rate limiting utilities for Terra Siaga API
/// Implements various rate limiting strategies for security and performance

use async_trait::async_trait;
use governor::{
    clock::{DefaultClock, QuantaClock},
    state::{InMemoryState, keyed::KeyedStateStore},
    middleware::NoOpMiddleware,
    Quota, RateLimiter as GovernorRateLimiter,
};
use std::collections::HashMap;
use std::hash::Hash;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::shared::cache::CacheService;
use crate::shared::error::{AppError, AppResult};
use crate::shared::types::{UserId, UserRole, constants::*};

/// Rate limiting strategy
#[derive(Debug, Clone)]
pub enum RateLimitStrategy {
    /// Fixed window (requests per time window)
    FixedWindow {
        requests: u32,
        window: Duration,
    },
    /// Sliding window (more accurate but resource intensive)
    SlidingWindow {
        requests: u32,
        window: Duration,
    },
    /// Token bucket (allows burst)
    TokenBucket {
        capacity: u32,
        refill_rate: u32,
        refill_interval: Duration,
    },
}

/// Rate limit configuration for different user roles and endpoints
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub default_strategy: RateLimitStrategy,
    pub role_overrides: HashMap<UserRole, RateLimitStrategy>,
    pub endpoint_overrides: HashMap<String, RateLimitStrategy>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        let mut role_overrides = HashMap::new();
        let mut endpoint_overrides = HashMap::new();

        // Role-based rate limits
        role_overrides.insert(
            UserRole::Reporter,
            RateLimitStrategy::FixedWindow {
                requests: 100,
                window: Duration::from_secs(3600), // 100 requests per hour
            },
        );
        role_overrides.insert(
            UserRole::Volunteer,
            RateLimitStrategy::FixedWindow {
                requests: 300,
                window: Duration::from_secs(3600), // 300 requests per hour
            },
        );
        role_overrides.insert(
            UserRole::Coordinator,
            RateLimitStrategy::FixedWindow {
                requests: 1000,
                window: Duration::from_secs(3600), // 1000 requests per hour
            },
        );
        role_overrides.insert(
            UserRole::OrgAdmin,
            RateLimitStrategy::FixedWindow {
                requests: 5000,
                window: Duration::from_secs(3600), // 5000 requests per hour
            },
        );
        role_overrides.insert(
            UserRole::SystemAdmin,
            RateLimitStrategy::FixedWindow {
                requests: 10000,
                window: Duration::from_secs(3600), // 10000 requests per hour
            },
        );

        // Endpoint-specific rate limits
        endpoint_overrides.insert(
            "/api/auth/login".to_string(),
            RateLimitStrategy::FixedWindow {
                requests: 5,
                window: Duration::from_secs(300), // 5 login attempts per 5 minutes
            },
        );
        endpoint_overrides.insert(
            "/api/auth/register".to_string(),
            RateLimitStrategy::FixedWindow {
                requests: 3,
                window: Duration::from_secs(3600), // 3 registrations per hour
            },
        );
        endpoint_overrides.insert(
            "/api/emergency/report".to_string(),
            RateLimitStrategy::TokenBucket {
                capacity: 10,
                refill_rate: 1,
                refill_interval: Duration::from_secs(60), // 1 report per minute, burst of 10
            },
        );

        Self {
            default_strategy: RateLimitStrategy::FixedWindow {
                requests: DEFAULT_RATE_LIMIT_PER_MINUTE,
                window: Duration::from_secs(60),
            },
            role_overrides,
            endpoint_overrides,
        }
    }
}

/// Rate limit result
#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed,
    Denied {
        retry_after: Duration,
        limit: u32,
        remaining: u32,
        reset_time: std::time::SystemTime,
    },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed)
    }
}

/// Rate limiter trait for different implementations
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if request is allowed for given key
    async fn check_rate_limit(&self, key: &str, strategy: &RateLimitStrategy) -> AppResult<bool>;

    /// Check rate limit with custom identifier
    async fn check_limit(&self, identifier: &str) -> AppResult<bool>;

    /// Get remaining requests for a key
    async fn get_remaining(&self, key: &str) -> AppResult<u32>;

    /// Get rate limit info
    async fn get_limit_info(&self, key: &str) -> AppResult<RateLimitInfo>;

    /// Reset rate limit for a key
    async fn reset_limit(&self, key: &str) -> AppResult<()>;
}

/// Rate limit information
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_time: std::time::SystemTime,
    pub retry_after: Option<Duration>,
}

/// In-memory rate limiter using governor
pub struct InMemoryRateLimiter {
    limiters: Arc<RwLock<HashMap<String, Arc<GovernorRateLimiter<String, InMemoryState, DefaultClock>>>>>,
    config: RateLimitConfig,
}

impl InMemoryRateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiters: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    async fn get_or_create_limiter(&self, key: &str, strategy: &RateLimitStrategy) -> Arc<GovernorRateLimiter<String, InMemoryState, DefaultClock>> {
        let limiters = self.limiters.read().await;

        if let Some(limiter) = limiters.get(key) {
            return limiter.clone();
        }

        drop(limiters);

        let mut limiters = self.limiters.write().await;

        // Double-check pattern
        if let Some(limiter) = limiters.get(key) {
            return limiter.clone();
        }

        let quota = match strategy {
            RateLimitStrategy::FixedWindow { requests, window } => {
                Quota::with_period(*window).unwrap().allow_burst(NonZeroU32::new(*requests).unwrap())
            }
            RateLimitStrategy::SlidingWindow { requests, window } => {
                // Governor doesn't have sliding window, so we use fixed window as approximation
                Quota::with_period(*window).unwrap().allow_burst(NonZeroU32::new(*requests).unwrap())
            }
            RateLimitStrategy::TokenBucket { capacity, refill_rate: _, refill_interval } => {
                Quota::with_period(*refill_interval).unwrap().allow_burst(NonZeroU32::new(*capacity).unwrap())
            }
        };

        let limiter = Arc::new(GovernorRateLimiter::keyed(quota));
        limiters.insert(key.to_string(), limiter.clone());

        limiter
    }
}

#[async_trait]
impl RateLimiter for InMemoryRateLimiter {
    async fn check_rate_limit(&self, key: &str, strategy: &RateLimitStrategy) -> AppResult<bool> {
        let limiter = self.get_or_create_limiter(key, strategy).await;

        match limiter.check_key(key) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn check_limit(&self, identifier: &str) -> AppResult<bool> {
        self.check_rate_limit(identifier, &self.config.default_strategy).await
    }

    async fn get_remaining(&self, key: &str) -> AppResult<u32> {
        // Governor doesn't directly expose remaining count
        // This is a simplified implementation
        Ok(0)
    }

    async fn get_limit_info(&self, key: &str) -> AppResult<RateLimitInfo> {
        // Simplified implementation
        Ok(RateLimitInfo {
            limit: 100,
            remaining: 50,
            reset_time: std::time::SystemTime::now() + Duration::from_secs(3600),
            retry_after: None,
        })
    }

    async fn reset_limit(&self, key: &str) -> AppResult<()> {
        let mut limiters = self.limiters.write().await;
        limiters.remove(key);
        Ok(())
    }
}

/// Cache-based rate limiter using external cache (Redis)
pub struct CacheRateLimiter {
    cache: Arc<dyn CacheService>,
    config: RateLimitConfig,
}

impl CacheRateLimiter {
    pub fn new(cache: Arc<dyn CacheService>, config: RateLimitConfig) -> Self {
        Self { cache, config }
    }

    fn rate_limit_key(&self, key: &str, strategy: &RateLimitStrategy) -> String {
        let window = match strategy {
            RateLimitStrategy::FixedWindow { window, .. } => window.as_secs(),
            RateLimitStrategy::SlidingWindow { window, .. } => window.as_secs(),
            RateLimitStrategy::TokenBucket { refill_interval, .. } => refill_interval.as_secs(),
        };

        let current_window = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() / window;

        format!("rate_limit:{}:{}", key, current_window)
    }
}

#[async_trait]
impl RateLimiter for CacheRateLimiter {
    async fn check_rate_limit(&self, key: &str, strategy: &RateLimitStrategy) -> AppResult<bool> {
        let cache_key = self.rate_limit_key(key, strategy);

        let limit = match strategy {
            RateLimitStrategy::FixedWindow { requests, .. } => *requests,
            RateLimitStrategy::SlidingWindow { requests, .. } => *requests,
            RateLimitStrategy::TokenBucket { capacity, .. } => *capacity,
        };

        let current_count = self.cache.increment(&cache_key, 1).await?;

        if current_count == 1 {
            // Set TTL for the key
            let ttl = match strategy {
                RateLimitStrategy::FixedWindow { window, .. } => window.as_secs() as u32,
                RateLimitStrategy::SlidingWindow { window, .. } => window.as_secs() as u32,
                RateLimitStrategy::TokenBucket { refill_interval, .. } => refill_interval.as_secs() as u32,
            };

            self.cache.set(&cache_key, &current_count, Some(ttl)).await?;
        }

        Ok(current_count <= limit as i64)
    }

    async fn check_limit(&self, identifier: &str) -> AppResult<bool> {
        self.check_rate_limit(identifier, &self.config.default_strategy).await
    }

    async fn get_remaining(&self, key: &str) -> AppResult<u32> {
        let cache_key = self.rate_limit_key(key, &self.config.default_strategy);
        let current_count: i64 = self.cache.get(&cache_key).await?.unwrap_or(0);

        let limit = match &self.config.default_strategy {
            RateLimitStrategy::FixedWindow { requests, .. } => *requests,
            RateLimitStrategy::SlidingWindow { requests, .. } => *requests,
            RateLimitStrategy::TokenBucket { capacity, .. } => *capacity,
        };

        Ok((limit as i64 - current_count).max(0) as u32)
    }

    async fn get_limit_info(&self, key: &str) -> AppResult<RateLimitInfo> {
        let remaining = self.get_remaining(key).await?;
        let limit = match &self.config.default_strategy {
            RateLimitStrategy::FixedWindow { requests, window } => {
                RateLimitInfo {
                    limit: *requests,
                    remaining,
                    reset_time: std::time::SystemTime::now() + *window,
                    retry_after: if remaining == 0 { Some(*window) } else { None },
                }
            }
            RateLimitStrategy::SlidingWindow { requests, window } => {
                RateLimitInfo {
                    limit: *requests,
                    remaining,
                    reset_time: std::time::SystemTime::now() + *window,
                    retry_after: if remaining == 0 { Some(*window) } else { None },
                }
            }
            RateLimitStrategy::TokenBucket { capacity, refill_interval, .. } => {
                RateLimitInfo {
                    limit: *capacity,
                    remaining,
                    reset_time: std::time::SystemTime::now() + *refill_interval,
                    retry_after: if remaining == 0 { Some(*refill_interval) } else { None },
                }
            }
        };

        Ok(limit)
    }

    async fn reset_limit(&self, key: &str) -> AppResult<()> {
        let cache_key = self.rate_limit_key(key, &self.config.default_strategy);
        self.cache.delete(&cache_key).await?;
        Ok(())
    }
}

/// Rate limit middleware for HTTP requests
pub struct RateLimitMiddleware {
    rate_limiter: Arc<dyn RateLimiter>,
    config: RateLimitConfig,
}

impl RateLimitMiddleware {
    pub fn new(rate_limiter: Arc<dyn RateLimiter>, config: RateLimitConfig) -> Self {
        Self {
            rate_limiter,
            config,
        }
    }

    pub async fn check_request_limit(
        &self,
        user_id: Option<&UserId>,
        user_role: Option<&UserRole>,
        endpoint: &str,
        ip_address: &str,
    ) -> AppResult<bool> {
        // Determine rate limit strategy
        let strategy = if let Some(endpoint_strategy) = self.config.endpoint_overrides.get(endpoint) {
            endpoint_strategy.clone()
        } else if let Some(role) = user_role {
            self.config.role_overrides.get(role).cloned()
                .unwrap_or(self.config.default_strategy.clone())
        } else {
            self.config.default_strategy.clone()
        };

        // Create rate limit key
        let key = if let Some(user_id) = user_id {
            format!("user:{}:{}", user_id, endpoint)
        } else {
            format!("ip:{}:{}", ip_address, endpoint)
        };

        self.rate_limiter.check_rate_limit(&key, &strategy).await
    }

    pub async fn get_rate_limit_headers(&self, key: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        if let Ok(info) = self.rate_limiter.get_limit_info(key).await {
            headers.insert("X-RateLimit-Limit".to_string(), info.limit.to_string());
            headers.insert("X-RateLimit-Remaining".to_string(), info.remaining.to_string());

            if let Ok(duration) = info.reset_time.duration_since(std::time::UNIX_EPOCH) {
                headers.insert("X-RateLimit-Reset".to_string(), duration.as_secs().to_string());
            }

            if let Some(retry_after) = info.retry_after {
                headers.insert("Retry-After".to_string(), retry_after.as_secs().to_string());
            }
        }

        headers
    }
}

/// Rate limiter factory
pub struct RateLimiterFactory;

impl RateLimiterFactory {
    /// Create rate limiter based on cache availability
    pub fn create(cache: Option<Arc<dyn CacheService>>, config: RateLimitConfig) -> Arc<dyn RateLimiter> {
        match cache {
            Some(cache_service) => {
                Arc::new(CacheRateLimiter::new(cache_service, config))
            }
            None => {
                Arc::new(InMemoryRateLimiter::new(config))
            }
        }
    }

    /// Create default rate limiter
    pub fn create_default() -> Arc<dyn RateLimiter> {
        Arc::new(InMemoryRateLimiter::new(RateLimitConfig::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_rate_limiter() {
        let config = RateLimitConfig::default();
        let limiter = InMemoryRateLimiter::new(config);

        let strategy = RateLimitStrategy::FixedWindow {
            requests: 2,
            window: Duration::from_secs(60),
        };

        // First two requests should pass
        assert!(limiter.check_rate_limit("test_key", &strategy).await.unwrap());
        assert!(limiter.check_rate_limit("test_key", &strategy).await.unwrap());

        // Third request should fail
        assert!(!limiter.check_rate_limit("test_key", &strategy).await.unwrap());

        // Reset and try again
        limiter.reset_limit("test_key").await.unwrap();
        assert!(limiter.check_rate_limit("test_key", &strategy).await.unwrap());
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();

        // Check that role overrides are set
        assert!(config.role_overrides.contains_key(&UserRole::Reporter));
        assert!(config.role_overrides.contains_key(&UserRole::SystemAdmin));

        // Check endpoint overrides
        assert!(config.endpoint_overrides.contains_key("/api/auth/login"));
        assert!(config.endpoint_overrides.contains_key("/api/emergency/report"));
    }
}
