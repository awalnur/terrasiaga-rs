/// Cache service implementation for Terra Siaga
/// Provides high-performance Redis-based caching with fallback mechanisms

use async_trait::async_trait;
use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, warn, debug, info};
use uuid::Uuid;

use crate::shared::{AppResult, AppError};

/// Cache service trait for dependency injection
#[async_trait]
pub trait CacheService: Send + Sync {
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send;

    async fn set<T>(&self, key: &str, value: &T, expiry: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync;

    async fn delete(&self, key: &str) -> AppResult<bool>;
    async fn exists(&self, key: &str) -> AppResult<bool>;
    async fn increment(&self, key: &str, delta: i64) -> AppResult<i64>;
    async fn set_nx<T>(&self, key: &str, value: &T, expiry: Option<Duration>) -> AppResult<bool>
    where
        T: Serialize + Send + Sync;

    async fn get_multiple<T>(&self, keys: &[String]) -> AppResult<Vec<Option<T>>>
    where
        T: for<'de> Deserialize<'de> + Send;

    async fn set_multiple<T>(&self, items: &[(String, T)], expiry: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync;

    async fn flush_pattern(&self, pattern: &str) -> AppResult<u64>;
}

/// Redis-based cache implementation
pub struct RedisCacheService {
    client: redis::Client,
    connection_pool: redis::aio::ConnectionManager,
    key_prefix: String,
    default_expiry: Duration,
}

/// In-memory cache implementation for testing and fallback
pub struct InMemoryCache {
    store: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, (Vec<u8>, std::time::Instant)>>>,
    default_expiry: Duration,
}

/// Hybrid cache that uses both Redis and in-memory cache
pub struct HybridCache {
    redis: RedisCacheService,
    memory: InMemoryCache,
}

/// Redis cache implementation
pub struct RedisCache {
    client: redis::Client,
    connection_pool: redis::aio::ConnectionManager,
    key_prefix: String,
    default_expiry: Duration,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub redis_url: String,
    pub key_prefix: String,
    pub default_expiry_secs: u64,
    pub max_memory_size: usize,
    pub enable_compression: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            key_prefix: "terra_siaga".to_string(),
            default_expiry_secs: 3600,
            max_memory_size: 100_000_000, // 100MB
            enable_compression: true,
        }
    }
}

/// Cache keys for different entities
pub struct CacheKeys;

impl CacheKeys {
    pub fn user(user_id: &Uuid) -> String {
        format!("user:{}", user_id)
    }
    
    pub fn user_session(session_id: &str) -> String {
        format!("session:{}", session_id)
    }
    
    pub fn disaster(disaster_id: &Uuid) -> String {
        format!("disaster:{}", disaster_id)
    }
    
    pub fn location(location_id: &Uuid) -> String {
        format!("location:{}", location_id)
    }
    
    pub fn notification(notification_id: &Uuid) -> String {
        format!("notification:{}", notification_id)
    }
    
    pub fn rate_limit(identifier: &str) -> String {
        format!("rate_limit:{}", identifier)
    }
}

impl RedisCacheService {
    /// Create a new Redis cache service
    pub async fn new(redis_url: &str, key_prefix: Option<String>) -> AppResult<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| {
                error!("Failed to create Redis client: {}", e);
                AppError::InternalServer(format!("Redis client error: {}", e))
            })?;

        let connection_pool = redis::aio::ConnectionManager::new(client.clone())
            .await
            .map_err(|e| {
                error!("Failed to create Redis connection pool: {}", e);
                AppError::InternalServer(format!("Redis connection error: {}", e))
            })?;

        info!("✅ Redis cache service initialized");

        Ok(Self {
            client,
            connection_pool,
            key_prefix: key_prefix.unwrap_or_else(|| "terra_siaga".to_string()),
            default_expiry: Duration::from_hours(1),
        })
    }

    /// Create prefixed key
    fn make_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    /// Get Redis connection with retry logic
    async fn get_connection(&self) -> AppResult<redis::aio::ConnectionManager> {
        // For now, clone the existing connection
        // In production, you might want to implement connection pooling
        Ok(self.connection_pool.clone())
    }

    /// Serialize value to JSON string
    fn serialize<T: Serialize>(&self, value: &T) -> AppResult<String> {
        serde_json::to_string(value)
            .map_err(|e| {
                error!("Failed to serialize cache value: {}", e);
                AppError::InternalServer(format!("Serialization error: {}", e))
            })
    }

    /// Deserialize JSON string to value
    fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &str) -> AppResult<T> {
        serde_json::from_str(data)
            .map_err(|e| {
                error!("Failed to deserialize cache value: {}", e);
                AppError::InternalServer(format!("Deserialization error: {}", e))
            })
    }
}

#[async_trait]
impl CacheService for RedisCacheService {
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        let cache_key = self.make_key(key);
        let mut conn = self.get_connection().await?;

        match conn.get::<_, Option<String>>(&cache_key).await {
            Ok(Some(data)) => {
                debug!("Cache hit for key: {}", cache_key);
                match self.deserialize::<T>(&data) {
                    Ok(value) => Ok(Some(value)),
                    Err(e) => {
                        warn!("Failed to deserialize cached value for key {}: {}", cache_key, e);
                        // Delete corrupted cache entry
                        let _ = conn.del::<_, ()>(&cache_key).await;
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                debug!("Cache miss for key: {}", cache_key);
                Ok(None)
            }
            Err(e) => {
                error!("Redis get error for key {}: {}", cache_key, e);
                Err(AppError::InternalServer(format!("Cache get error: {}", e)))
            }
        }
    }

    async fn set<T>(&self, key: &str, value: &T, expiry: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync,
    {
        let cache_key = self.make_key(key);
        let data = self.serialize(value)?;
        let mut conn = self.get_connection().await?;

        let expiry_seconds = expiry.unwrap_or(self.default_expiry).as_secs();

        let result: RedisResult<()> = conn.set_ex(&cache_key, &data, expiry_seconds).await;

        match result {
            Ok(_) => {
                debug!("Cache set for key: {} (expires in {}s)", cache_key, expiry_seconds);
                Ok(())
            }
            Err(e) => {
                error!("Redis set error for key {}: {}", cache_key, e);
                Err(AppError::InternalServer(format!("Cache set error: {}", e)))
            }
        }
    }

    async fn delete(&self, key: &str) -> AppResult<bool> {
        let cache_key = self.make_key(key);
        let mut conn = self.get_connection().await?;

        match conn.del::<_, i32>(&cache_key).await {
            Ok(deleted_count) => {
                debug!("Cache delete for key: {} (deleted: {})", cache_key, deleted_count > 0);
                Ok(deleted_count > 0)
            }
            Err(e) => {
                error!("Redis delete error for key {}: {}", cache_key, e);
                Err(AppError::InternalServer(format!("Cache delete error: {}", e)))
            }
        }
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        let cache_key = self.make_key(key);
        let mut conn = self.get_connection().await?;

        match conn.exists::<_, bool>(&cache_key).await {
            Ok(exists) => {
                debug!("Cache exists check for key: {} -> {}", cache_key, exists);
                Ok(exists)
            }
            Err(e) => {
                error!("Redis exists error for key {}: {}", cache_key, e);
                Err(AppError::InternalServer(format!("Cache exists error: {}", e)))
            }
        }
    }

    async fn increment(&self, key: &str, delta: i64) -> AppResult<i64> {
        let cache_key = self.make_key(key);
        let mut conn = self.get_connection().await?;

        match conn.incr::<_, _, i64>(&cache_key, delta).await {
            Ok(new_value) => {
                debug!("Cache increment for key: {} by {} -> {}", cache_key, delta, new_value);
                Ok(new_value)
            }
            Err(e) => {
                error!("Redis increment error for key {}: {}", cache_key, e);
                Err(AppError::InternalServer(format!("Cache increment error: {}", e)))
            }
        }
    }

    async fn set_nx<T>(&self, key: &str, value: &T, expiry: Option<Duration>) -> AppResult<bool>
    where
        T: Serialize + Send + Sync,
    {
        let cache_key = self.make_key(key);
        let data = self.serialize(value)?;
        let mut conn = self.get_connection().await?;

        let expiry_seconds = expiry.unwrap_or(self.default_expiry).as_secs();

        // Use SET with NX and EX options
        let result: RedisResult<Option<String>> = redis::cmd("SET")
            .arg(&cache_key)
            .arg(&data)
            .arg("NX")
            .arg("EX")
            .arg(expiry_seconds)
            .query_async(&mut conn)
            .await;

        match result {
            Ok(Some(_)) => {
                debug!("Cache set_nx success for key: {}", cache_key);
                Ok(true)
            }
            Ok(None) => {
                debug!("Cache set_nx failed (key exists) for key: {}", cache_key);
                Ok(false)
            }
            Err(e) => {
                error!("Redis set_nx error for key {}: {}", cache_key, e);
                Err(AppError::InternalServer(format!("Cache set_nx error: {}", e)))
            }
        }
    }

    async fn get_multiple<T>(&self, keys: &[String]) -> AppResult<Vec<Option<T>>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        if keys.is_empty() {
            return Ok(vec![]);
        }

        let cache_keys: Vec<String> = keys.iter().map(|k| self.make_key(k)).collect();
        let mut conn = self.get_connection().await?;

        match conn.get::<_, Vec<Option<String>>>(&cache_keys).await {
            Ok(values) => {
                let mut results = Vec::with_capacity(values.len());

                for (i, value_opt) in values.into_iter().enumerate() {
                    match value_opt {
                        Some(data) => {
                            match self.deserialize::<T>(&data) {
                                Ok(value) => results.push(Some(value)),
                                Err(_) => {
                                    warn!("Failed to deserialize cached value for key: {}", cache_keys[i]);
                                    // Delete corrupted cache entry
                                    let _ = conn.del::<_, ()>(&cache_keys[i]).await;
                                    results.push(None);
                                }
                            }
                        }
                        None => results.push(None),
                    }
                }

                debug!("Cache get_multiple for {} keys", keys.len());
                Ok(results)
            }
            Err(e) => {
                error!("Redis get_multiple error: {}", e);
                Err(AppError::InternalServer(format!("Cache get_multiple error: {}", e)))
            }
        }
    }

    async fn set_multiple<T>(&self, items: &[(String, T)], expiry: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync,
    {
        if items.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        let expiry_seconds = expiry.unwrap_or(self.default_expiry).as_secs();

        // Use pipeline for better performance
        let mut pipe = redis::pipe();

        for (key, value) in items {
            let cache_key = self.make_key(key);
            let data = self.serialize(value)?;
            pipe.set_ex(&cache_key, &data, expiry_seconds);
        }

        match pipe.query_async::<_, ()>(&mut conn).await {
            Ok(_) => {
                debug!("Cache set_multiple for {} items", items.len());
                Ok(())
            }
            Err(e) => {
                error!("Redis set_multiple error: {}", e);
                Err(AppError::InternalServer(format!("Cache set_multiple error: {}", e)))
            }
        }
    }

    async fn flush_pattern(&self, pattern: &str) -> AppResult<u64> {
        let cache_pattern = self.make_key(pattern);
        let mut conn = self.get_connection().await?;

        // First, find all keys matching the pattern
        let keys: Vec<String> = match conn.keys(&cache_pattern).await {
            Ok(keys) => keys,
            Err(e) => {
                error!("Redis keys error for pattern {}: {}", cache_pattern, e);
                return Err(AppError::InternalServer(format!("Cache keys error: {}", e)));
            }
        };

        if keys.is_empty() {
            debug!("No keys found for pattern: {}", cache_pattern);
            return Ok(0);
        }

        // Delete all matching keys
        match conn.del::<_, u64>(&keys).await {
            Ok(deleted_count) => {
                info!("Cache flush_pattern for '{}' deleted {} keys", cache_pattern, deleted_count);
                Ok(deleted_count)
            }
            Err(e) => {
                error!("Redis delete error for pattern {}: {}", cache_pattern, e);
                Err(AppError::InternalServer(format!("Cache flush error: {}", e)))
            }
        }
    }
}

/// In-memory cache implementation for testing and fallback
pub struct InMemoryCacheService {
    // In a real implementation, you'd use a more sophisticated in-memory cache
    // like moka or an LRU cache
    data: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, (String, std::time::Instant)>>>,
    default_expiry: Duration,
}

impl InMemoryCacheService {
    pub fn new() -> Self {
        Self {
            data: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            default_expiry: Duration::from_hours(1),
        }
    }

    fn serialize<T: Serialize>(&self, value: &T) -> AppResult<String> {
        serde_json::to_string(value)
            .map_err(|e| AppError::InternalServer(format!("Serialization error: {}", e)))
    }

    fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &str) -> AppResult<T> {
        serde_json::from_str(data)
            .map_err(|e| AppError::InternalServer(format!("Deserialization error: {}", e)))
    }

    async fn cleanup_expired(&self) {
        let mut data = self.data.write().await;
        let now = std::time::Instant::now();
        data.retain(|_, (_, expiry)| now < *expiry);
    }
}

#[async_trait]
impl CacheService for InMemoryCacheService {
    async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        self.cleanup_expired().await;

        let data = self.data.read().await;
        match data.get(key) {
            Some((value, expiry)) => {
                if std::time::Instant::now() < *expiry {
                    match self.deserialize::<T>(value) {
                        Ok(val) => Ok(Some(val)),
                        Err(_) => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    async fn set<T>(&self, key: &str, value: &T, expiry: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync,
    {
        let data_str = self.serialize(value)?;
        let expiry_time = std::time::Instant::now() + expiry.unwrap_or(self.default_expiry);

        let mut data = self.data.write().await;
        data.insert(key.to_string(), (data_str, expiry_time));

        Ok(())
    }

    async fn delete(&self, key: &str) -> AppResult<bool> {
        let mut data = self.data.write().await;
        Ok(data.remove(key).is_some())
    }

    async fn exists(&self, key: &str) -> AppResult<bool> {
        self.cleanup_expired().await;

        let data = self.data.read().await;
        match data.get(key) {
            Some((_, expiry)) => Ok(std::time::Instant::now() < *expiry),
            None => Ok(false),
        }
    }

    async fn increment(&self, _key: &str, _delta: i64) -> AppResult<i64> {
        // Simplified implementation - in real scenario you'd properly handle numeric values
        Err(AppError::InternalServer("Increment not implemented for in-memory cache".to_string()))
    }

    async fn set_nx<T>(&self, key: &str, value: &T, expiry: Option<Duration>) -> AppResult<bool>
    where
        T: Serialize + Send + Sync,
    {
        if self.exists(key).await? {
            Ok(false)
        } else {
            self.set(key, value, expiry).await?;
            Ok(true)
        }
    }

    async fn get_multiple<T>(&self, keys: &[String]) -> AppResult<Vec<Option<T>>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.get(key).await?);
        }
        Ok(results)
    }

    async fn set_multiple<T>(&self, items: &[(String, T)], expiry: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync,
    {
        for (key, value) in items {
            self.set(key, value, expiry).await?;
        }
        Ok(())
    }

    async fn flush_pattern(&self, pattern: &str) -> AppResult<u64> {
        let mut data = self.data.write().await;
        let keys_to_remove: Vec<_> = data.keys()
            .filter(|k| k.contains(pattern))
            .cloned()
            .collect();

        let count = keys_to_remove.len() as u64;
        for key in keys_to_remove {
            data.remove(&key);
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_cache() {
        let cache = InMemoryCacheService::new();

        // Test set and get
        cache.set("test_key", &"test_value", Some(Duration::from_secs(60))).await.unwrap();
        let result: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(result, Some("test_value".to_string()));

        // Test delete
        assert!(cache.delete("test_key").await.unwrap());
        let result: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_expiry() {
        let cache = InMemoryCacheService::new();

        // Set with very short expiry
        cache.set("expire_key", &"expire_value", Some(Duration::from_millis(1))).await.unwrap();

        // Wait for expiry
        tokio::time::sleep(Duration::from_millis(10)).await;

        let result: Option<String> = cache.get("expire_key").await.unwrap();
        assert_eq!(result, None);
    }
}
