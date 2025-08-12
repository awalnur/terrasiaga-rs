/// Cache utilities for Terra Siaga
/// Provides Redis-based caching and in-memory caching with TTL support

use async_trait::async_trait;
use moka::future::Cache as MokaCache;
use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use deadpool_redis::{Pool as RedisPool, Runtime};
use tokio::time::sleep;
use tracing::{debug, warn, error};

use crate::shared::error::{AppError, AppResult};
use crate::shared::types::constants::DEFAULT_CACHE_TTL_SECONDS;

/// Cache trait for different implementations
#[async_trait]
pub trait CacheService: Send + Sync {
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send;

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync;

    async fn delete(&self, key: &str) -> AppResult<()>;
    async fn exists(&self, key: &str) -> AppResult<bool>;
    async fn clear_pattern(&self, pattern: &str) -> AppResult<u64>;
    async fn increment(&self, key: &str, delta: i64) -> AppResult<i64>;
    async fn decrement(&self, key: &str, delta: i64) -> AppResult<i64>;
}

/// Redis-based cache implementation
pub struct RedisCache {
    pool: RedisPool,
    key_prefix: String,
}

impl RedisCache {
    pub fn new(pool: RedisPool, key_prefix: String) -> Self {
        Self { pool, key_prefix }
    }

    fn prefixed_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }
}

#[async_trait]
impl CacheService for RedisCache {
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let result: Option<String> = conn.get(&prefixed_key).await
            .map_err(|e| AppError::InternalServer(format!("Redis get error: {}", e)))?;

        match result {
            Some(data) => {
                let value = serde_json::from_str(&data)
                    .map_err(|e| AppError::InternalServer(format!("Cache deserialization error: {}", e)))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync,
    {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let serialized = serde_json::to_string(value)
            .map_err(|e| AppError::InternalServer(format!("Cache serialization error: {}", e)))?;

        match ttl {
            Some(duration) => {
                let seconds = duration.as_secs() as i64;
                conn.setex(&prefixed_key, seconds, &serialized).await
                    .map_err(|e| AppError::InternalServer(format!("Redis setex error: {}", e)))?;
            }
            None => {
                conn.set(&prefixed_key, &serialized).await
                    .map_err(|e| AppError::InternalServer(format!("Redis set error: {}", e)))?;
            }
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        conn.del(&prefixed_key).await
            .map_err(|e| AppError::InternalServer(format!("Redis delete error: {}", e)))?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let exists: bool = conn.exists(&prefixed_key).await
            .map_err(|e| AppError::InternalServer(format!("Redis exists error: {}", e)))?;

        Ok(exists)
    }

    async fn clear_pattern(&self, pattern: &str) -> AppResult<u64> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_pattern = format!("{}:{}", self.key_prefix, pattern);
        let keys: Vec<String> = conn.keys(&prefixed_pattern).await
            .map_err(|e| AppError::InternalServer(format!("Redis keys error: {}", e)))?;

        if keys.is_empty() {
            return Ok(0);
        }

        let deleted: u64 = conn.del(&keys).await
            .map_err(|e| AppError::InternalServer(format!("Redis delete pattern error: {}", e)))?;

        Ok(deleted)
    }

    async fn increment(&self, key: &str, delta: i64) -> AppResult<i64> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let result: i64 = conn.incr(&prefixed_key, delta).await
            .map_err(|e| AppError::InternalServer(format!("Redis increment error: {}", e)))?;

        Ok(result)
    }

    async fn decrement(&self, key: &str, delta: i64) -> AppResult<i64> {
        let mut conn = self.pool.get().await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let result: i64 = conn.decr(&prefixed_key, delta).await
            .map_err(|e| AppError::InternalServer(format!("Redis decrement error: {}", e)))?;

        Ok(result)
    }
}

/// In-memory cache implementation using Moka
pub struct InMemoryCache {
    cache: Arc<MokaCache<String, String>>,
    default_ttl: Duration,
}

impl InMemoryCache {
    pub fn new(max_capacity: u64, default_ttl_seconds: u64) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(default_ttl_seconds))
            .build();

        Self {
            cache: Arc::new(cache),
            default_ttl: Duration::from_secs(default_ttl_seconds),
        }
    }

    pub fn with_custom_ttl(max_capacity: u64, default_ttl: Duration) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(default_ttl)
            .build();

        Self {
            cache: Arc::new(cache),
            default_ttl,
        }
    }
}

#[async_trait]
impl CacheService for InMemoryCache {
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        match self.cache.get(key).await {
            Some(data) => {
                let value = serde_json::from_str(&data)
                    .map_err(|e| AppError::InternalServer(format!("Cache deserialization error: {}", e)))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync,
    {
        let serialized = serde_json::to_string(value)
            .map_err(|e| AppError::InternalServer(format!("Cache serialization error: {}", e)))?;

        // For in-memory cache, we use the cache's built-in TTL
        // Custom TTL per key is not directly supported by Moka
        self.cache.insert(key.to_string(), serialized).await;
        Ok(())
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        self.cache.invalidate(key).await;
        Ok(())
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        Ok(self.cache.contains_key(key))
    }

    async fn clear_pattern(&self, pattern: &str) -> AppResult<u64> {
        // This is a simplified implementation for in-memory cache
        // In a real implementation, you might want to track keys by pattern
        let mut deleted = 0u64;

        // Note: Moka doesn't have a direct way to iterate keys
        // This is a limitation of the in-memory implementation
        self.cache.invalidate_all();

        Ok(deleted)
    }

    async fn increment(&self, key: &str, delta: i64) -> AppResult<i64> {
        let current: i64 = match self.get::<i64>(key).await? {
            Some(val) => val,
            None => 0,
        };

        let new_value = current + delta;
        self.set(key, &new_value, None).await?;
        Ok(new_value)
    }

    async fn decrement(&self, key: &str, delta: i64) -> AppResult<i64> {
        let current: i64 = match self.get::<i64>(key).await? {
            Some(val) => val,
            None => 0,
        };

        let new_value = current - delta;
        self.set(key, &new_value, None).await?;
        Ok(new_value)
    }
}

/// Cache layer that provides fallback between Redis and in-memory cache
pub struct LayeredCache {
    primary: Arc<dyn CacheService>,
    fallback: Arc<dyn CacheService>,
}

impl LayeredCache {
    pub fn new(primary: Arc<dyn CacheService>, fallback: Arc<dyn CacheService>) -> Self {
        Self { primary, fallback }
    }
}

#[async_trait]
impl CacheService for LayeredCache {
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        // Try primary cache first
        match self.primary.get(key).await {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => {
                // Try fallback cache
                match self.fallback.get(key).await {
                    Ok(Some(value)) => {
                        // Write back to primary cache
                        if let Err(e) = self.primary.set(key, &value, None).await {
                            log::warn!("Failed to write back to primary cache: {}", e);
                        }
                        Ok(Some(value))
                    }
                    result => result,
                }
            }
            Err(_) => {
                // If primary fails, try fallback
                self.fallback.get(key).await
            }
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync,
    {
        // Try to set in both caches
        let primary_result = self.primary.set(key, value, ttl).await;
        let fallback_result = self.fallback.set(key, value, ttl).await;

        // Return error only if both fail
        match (primary_result, fallback_result) {
            (Ok(()), _) | (_, Ok(())) => Ok(()),
            (Err(e1), Err(e2)) => {
                log::error!("Both cache layers failed: primary={}, fallback={}", e1, e2);
                Err(e1) // Return primary error
            }
        }
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        // Delete from both caches
        let _ = self.primary.delete(key).await;
        let _ = self.fallback.delete(key).await;
        Ok(())
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        // Check primary first, then fallback
        match self.primary.exists(key).await {
            Ok(true) => Ok(true),
            _ => self.fallback.exists(key).await,
        }
    }

    async fn clear_pattern(&self, pattern: &str) -> AppResult<u64> {
        let primary_count = self.primary.clear_pattern(pattern).await.unwrap_or(0);
        let fallback_count = self.fallback.clear_pattern(pattern).await.unwrap_or(0);
        Ok(primary_count + fallback_count)
    }

    async fn increment(&self, key: &str, delta: i64) -> AppResult<i64> {
        // Use primary cache for atomic operations
        self.primary.increment(key, delta).await
    }

    async fn decrement(&self, key: &str, delta: i64) -> AppResult<i64> {
        // Use primary cache for atomic operations
        self.primary.decrement(key, delta).await
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub redis_url: Option<String>,
    pub redis_pool_size: Option<u32>,
    pub memory_cache_capacity: u64,
    pub default_ttl_seconds: u64,
    pub key_prefix: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: None,
            redis_pool_size: Some(10),
            memory_cache_capacity: 10000,
            default_ttl_seconds: 3600, // 1 hour
            key_prefix: "terra_siaga".to_string(),
        }
    }
}

/// Cache factory for creating appropriate cache implementations
pub struct CacheFactory;

impl CacheFactory {
    /// Create a cache service based on configuration
    pub async fn create(config: &CacheConfig) -> AppResult<Arc<dyn CacheService>> {
        match &config.redis_url {
            Some(redis_url) => {
                // Try to create Redis cache with in-memory fallback
                match Self::create_redis_cache(redis_url, &config).await {
                    Ok(redis_cache) => {
                        let memory_cache = Self::create_memory_cache(config);
                        Ok(Arc::new(LayeredCache::new(redis_cache, memory_cache)))
                    }
                    Err(e) => {
                        log::warn!("Failed to create Redis cache, falling back to memory cache: {}", e);
                        Ok(Self::create_memory_cache(config))
                    }
                }
            }
            None => {
                // Use in-memory cache only
                Ok(Self::create_memory_cache(config))
            }
        }
    }

    /// Create Redis cache
    async fn create_redis_cache(
        redis_url: &str,
        config: &CacheConfig,
    ) -> AppResult<Arc<dyn CacheService>> {
        let redis_config = deadpool_redis::Config::from_url(redis_url);
        let pool = redis_config
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| AppError::InternalServer(format!("Failed to create Redis pool: {}", e)))?;

        // Test connection
        let mut conn = pool.get().await
            .map_err(|e| AppError::InternalServer(format!("Failed to get Redis connection: {}", e)))?;

        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis ping failed: {}", e)))?;

        Ok(Arc::new(RedisCache::new(pool, config.key_prefix.clone())))
    }

    /// Create in-memory cache
    fn create_memory_cache(config: &CacheConfig) -> Arc<dyn CacheService> {
        Arc::new(InMemoryCache::new(
            config.memory_cache_capacity,
            config.default_ttl_seconds,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache() {
        let cache = InMemoryCache::new(1000, 3600);

        // Test set and get
        cache.set("test_key", &"test_value", None).await.unwrap();
        let value: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Test exists
        assert!(cache.exists("test_key").await.unwrap());
        assert!(!cache.exists("nonexistent").await.unwrap());

        // Test delete
        cache.delete("test_key").await.unwrap();
        let value: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_increment_decrement() {
        let cache = InMemoryCache::new(1000, 3600);

        // Test increment
        let result = cache.increment("counter", 5).await.unwrap();
        assert_eq!(result, 5);

        let result = cache.increment("counter", 3).await.unwrap();
        assert_eq!(result, 8);

        // Test decrement
        let result = cache.decrement("counter", 2).await.unwrap();
        assert_eq!(result, 6);
    }
}
