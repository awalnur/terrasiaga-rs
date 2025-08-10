/// Cache implementation for Terra Siaga
/// Provides Redis-based caching with fallback to in-memory cache

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use redis::{AsyncCommands, Client as RedisClient};
use moka::future::Cache as MokaCache;

use crate::shared::{AppResult, AppError};

/// Cache interface
#[async_trait]
pub trait CacheService: Send + Sync {
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + 'static;

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync;

    async fn delete(&self, key: &str) -> AppResult<()>;
    async fn exists(&self, key: &str) -> AppResult<bool>;
    async fn clear_pattern(&self, pattern: &str) -> AppResult<u64>;
    async fn increment(&self, key: &str, delta: i64) -> AppResult<i64>;
    async fn expire(&self, key: &str, ttl: Duration) -> AppResult<bool>;
}

/// Redis cache implementation
pub struct RedisCache {
    client: RedisClient,
    default_ttl: Duration,
    key_prefix: String,
}

impl RedisCache {
    pub fn new(redis_url: &str, default_ttl: Duration, key_prefix: String) -> AppResult<Self> {
        let client = RedisClient::open(redis_url)
            .map_err(|e| AppError::Cache(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self {
            client,
            default_ttl,
            key_prefix,
        })
    }

    fn prefixed_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }
}

#[async_trait]
impl CacheService for RedisCache {
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
    {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::Cache(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let value: Option<String> = conn.get(&prefixed_key).await
            .map_err(|e| AppError::Cache(format!("Redis get error: {}", e)))?;

        match value {
            Some(json_str) => {
                let deserialized = serde_json::from_str(&json_str)
                    .map_err(|e| AppError::Cache(format!("Deserialization error: {}", e)))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync,
    {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::Cache(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let json_str = serde_json::to_string(value)
            .map_err(|e| AppError::Cache(format!("Serialization error: {}", e)))?;

        let ttl_seconds = ttl.unwrap_or(self.default_ttl).as_secs();

        conn.set_ex(&prefixed_key, json_str, ttl_seconds).await
            .map_err(|e| AppError::Cache(format!("Redis set error: {}", e)))?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::Cache(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        conn.del(&prefixed_key).await
            .map_err(|e| AppError::Cache(format!("Redis delete error: {}", e)))?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::Cache(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let exists: bool = conn.exists(&prefixed_key).await
            .map_err(|e| AppError::Cache(format!("Redis exists error: {}", e)))?;

        Ok(exists)
    }

    async fn clear_pattern(&self, pattern: &str) -> AppResult<u64> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::Cache(format!("Redis connection error: {}", e)))?;

        let search_pattern = format!("{}:{}", self.key_prefix, pattern);
        let keys: Vec<String> = conn.keys(&search_pattern).await
            .map_err(|e| AppError::Cache(format!("Redis keys error: {}", e)))?;

        if keys.is_empty() {
            return Ok(0);
        }

        let deleted: u64 = conn.del(&keys).await
            .map_err(|e| AppError::Cache(format!("Redis delete pattern error: {}", e)))?;

        Ok(deleted)
    }

    async fn increment(&self, key: &str, delta: i64) -> AppResult<i64> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::Cache(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let result: i64 = conn.incr(&prefixed_key, delta).await
            .map_err(|e| AppError::Cache(format!("Redis increment error: {}", e)))?;

        Ok(result)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> AppResult<bool> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::Cache(format!("Redis connection error: {}", e)))?;

        let prefixed_key = self.prefixed_key(key);
        let result: bool = conn.expire(&prefixed_key, ttl.as_secs() as usize).await
            .map_err(|e| AppError::Cache(format!("Redis expire error: {}", e)))?;

        Ok(result)
    }
}

/// In-memory cache implementation using Moka
pub struct InMemoryCache {
    cache: MokaCache<String, String>,
    default_ttl: Duration,
}

impl InMemoryCache {
    pub fn new(max_capacity: u64, default_ttl: Duration) -> Self {
        let cache = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(default_ttl)
            .build();

        Self {
            cache,
            default_ttl,
        }
    }
}

#[async_trait]
impl CacheService for InMemoryCache {
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
    {
        match self.cache.get(key) {
            Some(json_str) => {
                let deserialized = serde_json::from_str(&json_str)
                    .map_err(|e| AppError::Cache(format!("Deserialization error: {}", e)))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync,
    {
        let json_str = serde_json::to_string(value)
            .map_err(|e| AppError::Cache(format!("Serialization error: {}", e)))?;

        // Moka doesn't support per-key TTL easily, so we use the default
        self.cache.insert(key.to_string(), json_str).await;
        Ok(())
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        self.cache.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        Ok(self.cache.contains_key(key))
    }

    async fn clear_pattern(&self, pattern: &str) -> AppResult<u64> {
        // Simple pattern matching for in-memory cache
        let mut count = 0u64;

        // This is a simplified implementation - in production you'd want more sophisticated pattern matching
        for (key, _) in self.cache.iter() {
            if key.contains(pattern) {
                self.cache.remove(&key);
                count += 1;
            }
        }

        Ok(count)
    }

    async fn increment(&self, key: &str, delta: i64) -> AppResult<i64> {
        // Get current value or default to 0
        let current: i64 = match self.cache.get(key) {
            Some(value_str) => value_str.parse().unwrap_or(0),
            None => 0,
        };

        let new_value = current + delta;
        self.cache.insert(key.to_string(), new_value.to_string()).await;

        Ok(new_value)
    }

    async fn expire(&self, key: &str, _ttl: Duration) -> AppResult<bool> {
        // Moka handles TTL globally, so we just check if key exists
        Ok(self.cache.contains_key(key))
    }
}

/// Hybrid cache that uses both Redis and in-memory cache
pub struct HybridCache {
    redis_cache: RedisCache,
    memory_cache: InMemoryCache,
    prefer_redis: bool,
}

impl HybridCache {
    pub fn new(
        redis_cache: RedisCache,
        memory_cache: InMemoryCache,
        prefer_redis: bool,
    ) -> Self {
        Self {
            redis_cache,
            memory_cache,
            prefer_redis,
        }
    }
}

#[async_trait]
impl CacheService for HybridCache {
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send + 'static,
    {
        // Try memory cache first for speed
        if let Ok(Some(value)) = self.memory_cache.get(key).await {
            return Ok(Some(value));
        }

        // Fall back to Redis
        if let Ok(Some(value)) = self.redis_cache.get(key).await {
            // Store in memory cache for next time
            let _ = self.memory_cache.set(key, &value, None).await;
            return Ok(Some(value));
        }

        Ok(None)
    }

    async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync,
    {
        // Set in both caches
        let redis_result = self.redis_cache.set(key, value, ttl).await;
        let memory_result = self.memory_cache.set(key, value, ttl).await;

        // If Redis is preferred, return its result
        if self.prefer_redis {
            redis_result
        } else {
            memory_result
        }
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
        let _ = self.memory_cache.set(key, &result, None).await;

        Ok(result)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> AppResult<bool> {
        let redis_result = self.redis_cache.expire(key, ttl).await.unwrap_or(false);
        let memory_result = self.memory_cache.expire(key, ttl).await.unwrap_or(false);

        Ok(redis_result || memory_result)
    }
}

/// Cache key builders for different entities
pub struct CacheKeys;

impl CacheKeys {
    pub fn user(user_id: &str) -> String {
        format!("user:{}", user_id)
    }

    pub fn user_by_email(email: &str) -> String {
        format!("user:email:{}", email)
    }

    pub fn disaster(disaster_id: &str) -> String {
        format!("disaster:{}", disaster_id)
    }

    pub fn disasters_nearby(lat: f64, lon: f64, radius: f64) -> String {
        format!("disasters:nearby:{}:{}:{}", lat, lon, radius)
    }

    pub fn user_session(session_id: &str) -> String {
        format!("session:{}", session_id)
    }

    pub fn rate_limit(user_id: &str, action: &str) -> String {
        format!("rate_limit:{}:{}", user_id, action)
    }

    pub fn notification_settings(user_id: &str) -> String {
        format!("notifications:settings:{}", user_id)
    }

    pub fn emergency_contacts(user_id: &str) -> String {
        format!("emergency:contacts:{}", user_id)
    }
}
