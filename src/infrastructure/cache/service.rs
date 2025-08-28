/// Cache utilities for Terra Siaga
/// Provides Redis-based caching and in-memory caching with TTL support
use async_trait::async_trait;
use deadpool_redis::redis::{cmd, AsyncCommands};
use deadpool_redis::{Pool as RedisPool, Runtime};
use moka::future::Cache as MokaCache;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, warn};

use crate::shared::error::{AppError, AppResult};
use crate::shared::types::constants::DEFAULT_CACHE_TTL_SECONDS;

/// Cache trait for different implementations
#[async_trait]
pub trait CacheService: Send + Sync {
    // async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    // where
    //     T: for<'de> Deserialize<'de> + Send + Sync + Hash + Eq;
    // async fn set<T>(&self, key: &str, value: T, ttl: Option<Duration>) -> AppResult<()>
    // where
    //     T: Serialize + Send + Sync;
    async fn get_string(&self, key: &str) -> AppResult<Option<String>>;
    async fn set_string(&self, key: &str, value: String, ttl: Option<Duration>) -> AppResult<()>;

    // New: set key expiration/TTL after key is created or incremented
    async fn expire(&self, key: &str, ttl: Duration) -> AppResult<()>;

    async fn delete(&self, key: &str) -> AppResult<()>;
    async fn exists(&self, key: &str) -> AppResult<bool>;
    async fn clear_pattern(&self, pattern: &str) -> AppResult<u64>;
    async fn increment(&self, key: &str, delta: i64) -> AppResult<i64>;
    async fn decrement(&self, key: &str, delta: i64) -> AppResult<i64>;
}


/// Extension trait for typed operations (not dyn-compatible but provides convenience)
#[async_trait]
pub trait TypedCacheExt: CacheService {
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        match self.get_string(key).await? {
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
        self.set_string(key, serialized, ttl).await
    }
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
    // async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    // where
    //     T: for<'de> Deserialize<'de> + Send + Sync + Hash + Eq
    // {
    //     let mut conn = self.pool.get().await
    //         .map_err(|e| AppError::Cache(format!("Redis connection error: {}", e)))?;
    //
    //     let prefixed_key = self.prefixed_key(key);
    //     let value: Option<String> = conn.get(&prefixed_key).await
    //         .map_err(|e| AppError::Cache(format!("Redis get error: {}", e)))?;
    //
    //     match value {
    //         Some(json_str) => {
    //             let deserialized = serde_json::from_str(&json_str)
    //                 .map_err(|e| AppError::Cache(format!("Deserialization error: {}", e)))?;
    //             Ok(Some(deserialized))
    //         }
    //         None => Ok(None),
    //     }
    // }
    //
    // async fn set<T>(&self, key: &str, value: T, ttl: Option<Duration>) -> AppResult<()>
    // where
    //     T: Serialize + Send + Sync
    // {
    //     let mut conn = self.pool.get().await
    //         .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;
    //
    //
    //     let prefixed_key = self.prefixed_key(key);
    //     let json_str = serde_json::to_string(value)
    //         .map_err(|e| AppError::Cache(format!("Serialization error: {}", e)))?;
    //
    //     let ttl_seconds = ttl.unwrap_or(Duration::from_secs(1000)).as_secs();
    //
    //     conn.set_ex(&prefixed_key, json_str, ttl_seconds).await
    //         .map_err(|e| AppError::Cache(format!("Redis set error: {}", e)))?;
    //
    //     Ok(())
    // }

    async fn get_string(&self, key: &str) -> AppResult<Option<String>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let result: Option<String> = conn
            .get(&prefixed_key)
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis get error: {}", e)))?;

        // match result {
        //     Some(json_str) => {
        //         let deserialized = serde_json::from_str(&json_str)
        //             .map_err(|e| AppError::Cache(format!("Deserialization error: {}", e)))?;
        //         Ok(Some(deserialized))
        //     }
        //     None => Ok(None),
        // }

        Ok(Some(result.unwrap_or_default()))
    }

    async fn set_string(&self, key: &str, value: String, ttl: Option<Duration>) -> AppResult<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);

        match ttl {
            Some(duration) => {
                let seconds = duration.as_secs() as u64;
                conn.set_ex(&prefixed_key, value, seconds)
                    .await
                    .map_err(|e| AppError::InternalServer(format!("Redis setex error: {}", e)))?;
            }
            None => {
                conn.set(&prefixed_key, &value)
                    .await
                    .map_err(|e| AppError::InternalServer(format!("Redis set error: {}", e)))?;
            }
        }

        Ok(())
    }

    async fn expire(&self, key: &str, ttl: Duration) -> AppResult<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;
        let prefixed_key = self.prefixed_key(key);
        let seconds = ttl.as_secs() as usize;
        deadpool_redis::redis::cmd("EXPIRE")
            .arg(&prefixed_key)
            .arg(seconds)
            .query_async::<_>(&mut conn)
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis expire error: {}", e)))?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        conn.del(&prefixed_key)
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis delete error: {}", e)))?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let exists: bool = conn
            .exists(&prefixed_key)
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis exists error: {}", e)))?;

        Ok(exists)
    }

    async fn clear_pattern(&self, pattern: &str) -> AppResult<u64> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_pattern = format!("{}:{}", self.key_prefix, pattern);
        let keys: Vec<String> = conn
            .keys(&prefixed_pattern)
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis keys error: {}", e)))?;

        if keys.is_empty() {
            return Ok(0);
        }

        let deleted: u64 = conn
            .del(&keys)
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis delete pattern error: {}", e)))?;

        Ok(deleted)
    }

    async fn increment(&self, key: &str, delta: i64) -> AppResult<i64> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let result: i64 = conn
            .incr(&prefixed_key, delta)
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis increment error: {}", e)))?;

        Ok(result)
    }

    async fn decrement(&self, key: &str, delta: i64) -> AppResult<i64> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let result: i64 = conn
            .decr(&prefixed_key, delta)
            .await
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
    // async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    // where
    //     T: for<'de> Deserialize<'de> + Send + 'static,
    // {
    //     match self.cache.get(key) {
    //         Some(json_str) => {
    //             let deserialized = serde_json::from_str(&json_str)
    //                 .map_err(|e| AppError::Cache(format!("Deserialization error: {}", e)))?;
    //             Ok(Some(deserialized))
    //         }
    //         None => Ok(None),
    //     }
    // }
    //
    // async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    // where
    //     T: Serialize + Send + Sync,
    // {
    //     let json_str = serde_json::to_string(value)
    //         .map_err(|e| AppError::Cache(format!("Serialization error: {}", e)))?;
    //
    //     // Moka doesn't support per-key TTL easily, so we use the default
    //     self.cache.insert(key.to_string(), json_str).await;
    //     Ok(())
    // }

    async fn get_string(&self, key: &str) -> AppResult<Option<String>> {
        Ok(self.cache.get(key).await)
    }

    async fn set_string(&self, key: &str, value: String, _ttl: Option<Duration>) -> AppResult<()> {
        // For in-memory cache, we use the cache's built-in TTL
        // Custom TTL per key is not directly supported by Moka
        self.cache.insert(key.to_string(), value).await;
        Ok(())
    }

    async fn expire(&self, key: &str, ttl: Duration) -> AppResult<()> {
        let cache = self.cache.clone();
        let key_owned = key.to_string();
        tokio::spawn(async move {
            sleep(ttl).await;
            cache.invalidate(&key_owned).await;
        });
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
        let current: i64 = match self.get_string(key).await? {
            Some(val) => val.parse::<i64>().unwrap_or(0),
            None => 0,
        };

        let new_value = current + delta;
        self.set_string(key, new_value.to_string(), None).await?;
        Ok(new_value)
    }

    async fn decrement(&self, key: &str, delta: i64) -> AppResult<i64> {
        let current: i64 = match self.get_string(key).await? {
            Some(val) => val.parse::<i64>().unwrap_or(0),
            None => 0,
        };

        let new_value = current - delta;
        self.set_string(key, new_value.to_string(), None).await?;
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
    // async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    // where
    //     T: for<'de> Deserialize<'de> + Send + 'static,
    // {
    //     match self.primary.get::<T>(key).await {
    //         Ok(Some(value)) => Ok(Some(value)),
    //         Ok(None) => {
    //             // Try fallback cache
    //             match self.fallback.get::<T>(key).await {
    //                 Ok(Some(value)) => {
    //                     // Write back to primary cache
    //                     if let Err(e) = self.primary.set::<T>(key, value.clone(), None).await {
    //                         warn!("Failed to write back to primary cache: {}", e);
    //                     }
    //                     Ok(Some(value))
    //                 }
    //                 result => result,
    //             }
    //         }
    //         Err(_) => {
    //             // If primary fails, try fallback
    //             self.fallback.get::<T>(key).await
    //         }
    //     }
    // }
    //
    // async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    // where
    //     T: Serialize + Send + Sync,
    // {
    //     // Set in both caches
    //     let primary_result = self.primary.set::<T>(key, value, ttl).await;
    //     let fallback_result = self.fallback.set::<T>(key, value, ttl).await;
    //
    //     match (primary_result, fallback_result) {
    //         (Ok(()), _) | (_, Ok(())) => Ok(()),
    //         (Err(e1), Err(e2)) => {
    //             error!("Both cache layers failed: primary={}, fallback={}", e1, e2);
    //             Err(e1) // Return primary error
    //         }
    //     }
    // }
    async fn get_string(&self, key: &str) -> AppResult<Option<String>> {
        // try primary then fallback
        match self.primary.get_string(key).await? {
            some @ Some(_) => Ok(some),
            None => self.fallback.get_string(key).await,
        }
    }

    async fn set_string(&self, key: &str, value: String, ttl: Option<Duration>) -> AppResult<()> {
        self.primary.set_string(key, value, ttl).await
    }

    async fn expire(&self, key: &str, ttl: Duration) -> AppResult<()> {
        self.primary.expire(key, ttl).await
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        self.primary.delete(key).await
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        self.primary.exists(key).await
    }

    async fn clear_pattern(&self, pattern: &str) -> AppResult<u64> {
        self.primary.clear_pattern(pattern).await
    }

    async fn increment(&self, key: &str, delta: i64) -> AppResult<i64> {
        self.primary.increment(key, delta).await
    }

    async fn decrement(&self, key: &str, delta: i64) -> AppResult<i64> {
        self.primary.decrement(key, delta).await
    }
}

pub struct HybridCache {
    redis_cache: RedisCache,
    memory_cache: InMemoryCache,
    prefer_redis: bool,
}
impl HybridCache {
    pub fn new(redis_cache: RedisCache, memory_cache: InMemoryCache, prefer_redis: bool) -> Self {
        Self {
            redis_cache,
            memory_cache,
            prefer_redis,
        }
    }
}

#[async_trait]
impl CacheService for HybridCache {
    // async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    // where
    //     T: for<'de> Deserialize<'de> + Send + 'static,
    // {
    //     // Try memory cache first for speed
    //     if let Ok(Some(value)) = self.memory_cache.get(key).await {
    //         return Ok(Some(value));
    //     }
    //
    //     // Fall back to Redis
    //     if let Ok(Some(value)) = self.redis_cache.get(key).await {
    //         // Store in memory cache for next time
    //         let _ = self.memory_cache.set(key, &value, None).await;
    //         return Ok(Some(value));
    //     }
    //
    //     Ok(None)
    // }
    //
    // async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    // where
    //     T: Serialize + Send + Sync,
    // {
    //     // Set in both caches
    //     let redis_result = self.redis_cache.set(key, value, ttl).await;
    //     let memory_result = self.memory_cache.set(key, value, ttl).await;
    //
    //     // If Redis is preferred, return its result
    //     if self.prefer_redis {
    //         redis_result
    //     } else {
    //         memory_result
    //     }
    // }

    async fn get_string(&self, key: &str) -> AppResult<Option<String>> {
        // Prefer memory cache first for speed
        if let Some(val) = self.memory_cache.get_string(key).await? {
            return Ok(Some(val));
        }
        // Fall back to Redis
        let redis_val = self.redis_cache.get_string(key).await?;
        if let Some(ref v) = redis_val {
            // Warm memory cache
            let _ = self.memory_cache.set_string(key, v.clone(), None).await;
        }
        Ok(redis_val)
    }

    async fn set_string(&self, key: &str, value: String, ttl: Option<Duration>) -> AppResult<()> {
        // Write to Redis (authoritative)
        self.redis_cache.set_string(key, value.clone(), ttl).await?;
        // Best-effort in-memory write
        let _ = self.memory_cache.set_string(key, value, ttl).await;
        Ok(())
    }

    async fn expire(&self, key: &str, ttl: Duration) -> AppResult<()> {
        // Set TTL in Redis; schedule memory invalidation
        self.redis_cache.expire(key, ttl).await?;
        let _ = self.memory_cache.expire(key, ttl).await;
        Ok(())
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        // Delete from both caches
        let _ = self.redis_cache.delete(key).await;
        let _ = self.memory_cache.delete(key).await;
        Ok(())
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        // Check memory cache first
        if self.memory_cache.exists(key).await? {
            return Ok(true);
        }

        // Check Redis
        self.redis_cache.exists(key).await
    }

    async fn clear_pattern(&self, pattern: &str) -> AppResult<u64> {
        let redis_count = self.redis_cache.clear_pattern(pattern).await.unwrap_or(0);
        let memory_count = self.memory_cache.clear_pattern(pattern).await.unwrap_or(0);

        Ok(redis_count + memory_count)
    }

    async fn increment(&self, key: &str, delta: i64) -> AppResult<i64> {
        // Use Redis for atomic operations
        let result = self.redis_cache.increment(key, delta).await?;

        // Update memory cache
        let _ = self
            .memory_cache
            .set_string(key, result.to_string(), None)
            .await;

        Ok(result)
    }

    async fn decrement(&self, key: &str, delta: i64) -> AppResult<i64> {
        // Use Redis for atomic operations
        let result = self.redis_cache.decrement(key, delta).await?;
        // Update memory cache
        let _ = self
            .memory_cache
            .set_string(key, result.to_string(), None)
            .await;
        Ok(result)
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
    /// Create a cache service based on configuration
    pub async fn create_redis_cache(config: &CacheConfig) -> AppResult<Arc<dyn CacheService>> {
        let redis_config = deadpool_redis::Config::from_url(config.redis_url.as_ref().unwrap());
        let pool = redis_config
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| AppError::InternalServer(format!("Failed to create Redis pool: {}", e)))?;

        // Test connection
        let mut conn = pool.get().await.map_err(|e| {
            AppError::InternalServer(format!("Failed to get Redis connection: {}", e))
        })?;

        // Fix the Redis ping command
        cmd("PING")
            .query_async(&mut *conn)
            .await
            .map_err(|e| AppError::InternalServer(format!("Redis ping failed: {}", e)))?;

        Ok(Arc::new(RedisCache::new(
            pool,
            config.key_prefix.to_string(),
        )))
    }

    /// Create in-memory cache
    pub fn create_memory_cache(
        max_capacity: u64,
        default_ttl_seconds: u64,
    ) -> Arc<dyn CacheService> {
        Arc::new(InMemoryCache::new(max_capacity, default_ttl_seconds))
    }

    /// Create layered cache with Redis primary and memory fallback
    pub async fn create_layered_cache(
        redis_url: &str,
        key_prefix: &str,
        memory_capacity: u64,
        default_ttl_seconds: u64,
    ) -> AppResult<Arc<dyn CacheService>> {
        let config = CacheConfig {
            redis_url: redis_url.to_string().into(),
            redis_pool_size: Some(10),
            memory_cache_capacity: memory_capacity,
            default_ttl_seconds: default_ttl_seconds,
            key_prefix: key_prefix.to_string(),
        };
        let redis_cache = Self::create_redis_cache(&config).await?;
        let memory_cache = Self::create_memory_cache(memory_capacity, default_ttl_seconds);

        Ok(Arc::new(LayeredCache::new(redis_cache, memory_cache)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache() {
        let mut cache = InMemoryCache::new(1000, 3600);

        // Test set and get
        cache
            .set_string("test_key", "test_value".to_string(), None)
            .await
            .unwrap();
        let value: Option<String> = cache.get_string("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Test exists
        assert!(cache.exists("test_key").await.unwrap());
        assert!(!cache.exists("nonexistent").await.unwrap());

        // Test delete
        cache.delete("test_key").await.unwrap();
        let value: Option<String> = cache.get_string("test_key").await.unwrap();
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
