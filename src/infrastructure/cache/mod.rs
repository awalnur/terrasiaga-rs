// Improved cache infrastructure with better design patterns
// Uses enum-based cache selection instead of trait objects for better maintainability

use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::shared::error::AppResult;
use crate::shared::error::AppError;

/// Cache configuration for different environments
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub cache_type: CacheType,
    pub default_ttl: Duration,
    pub max_capacity: Option<u64>,
    pub redis_url: Option<String>,
}

#[derive(Debug, Clone)]
pub enum CacheType {
    InMemory { max_capacity: u64 },
    Redis { url: String },
    Disabled,
}

/// Unified cache implementation using enum dispatch
pub enum CacheService {
    InMemory(InMemoryCache),
    Redis(RedisCache),
    Disabled,
}

impl CacheService {
    /// Create cache service based on configuration
    pub async fn new(config: CacheConfig) -> AppResult<Self> {
        match config.cache_type {
            CacheType::InMemory { max_capacity } => {
                Ok(Self::InMemory(InMemoryCache::new(max_capacity, config.default_ttl)))
            }
            CacheType::Redis { url } => {
                let redis_cache = RedisCache::new(&url).await?;
                Ok(Self::Redis(redis_cache))
            }
            CacheType::Disabled => Ok(Self::Disabled),
        }
    }

    /// Get value from cache
    pub async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        match self {
            Self::InMemory(cache) => cache.get(key).await,
            Self::Redis(cache) => cache.get(key).await,
            Self::Disabled => Ok(None),
        }
    }

    /// Set value in cache with optional TTL
    pub async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    where
        T: Serialize + Send + Sync,
    {
        match self {
            Self::InMemory(cache) => cache.set(key, value, ttl).await,
            Self::Redis(cache) => cache.set(key, value, ttl).await,
            Self::Disabled => Ok(()),
        }
    }

    /// Delete value from cache
    pub async fn delete(&self, key: &str) -> AppResult<()> {
        match self {
            Self::InMemory(cache) => cache.delete(key).await,
            Self::Redis(cache) => cache.delete(key).await,
            Self::Disabled => Ok(()),
        }
    }

    /// Check if key exists in cache
    pub async fn exists(&self, key: &str) -> AppResult<bool> {
        match self {
            Self::InMemory(cache) => cache.exists(key).await,
            Self::Redis(cache) => cache.exists(key).await,
            Self::Disabled => Ok(false),
        }
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> AppResult<()> {
        match self {
            Self::InMemory(cache) => cache.clear().await,
            Self::Redis(cache) => cache.clear().await,
            Self::Disabled => Ok(()),
        }
    }

    /// Get cache statistics
    pub async fn stats(&self) -> AppResult<CacheStats> {
        match self {
            Self::InMemory(cache) => cache.stats().await,
            Self::Redis(cache) => cache.stats().await,
            Self::Disabled => Ok(CacheStats::default()),
        }
    }
}

impl CacheConfig {
    /// Create cache config from environment
    pub fn from_env() -> AppResult<Self> {
        let cache_type = match std::env::var("CACHE_TYPE").as_deref() {
            Ok("redis") => {
                let redis_url = std::env::var("REDIS_URL")
                    .map_err(|_| AppError::Configuration(
                        "REDIS_URL is required when CACHE_TYPE=redis".to_string()
                    ))?;
                CacheType::Redis { url: redis_url }
            }
            Ok("memory") => {
                let capacity = std::env::var("CACHE_MAX_CAPACITY")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .unwrap_or(1000);
                CacheType::InMemory { max_capacity: capacity }
            }
            Ok("disabled") => CacheType::Disabled,
            _ => {
                // Default to in-memory for development
                CacheType::InMemory { max_capacity: 1000 }
            }
        };

        let default_ttl_secs = std::env::var("CACHE_DEFAULT_TTL")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .unwrap_or(3600);

        Ok(Self {
            cache_type,
            default_ttl: Duration::from_secs(default_ttl_secs),
            max_capacity: None,
            redis_url: None,
        })
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: u64,
    pub memory_usage: u64,
    pub hit_ratio: f64,
}

/// In-memory cache implementation
pub struct InMemoryCache {
    store: tokio::sync::RwLock<std::collections::HashMap<String, CacheEntry>>,
    max_capacity: u64,
    default_ttl: Duration,
    stats: tokio::sync::RwLock<CacheStats>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    expires_at: Option<std::time::Instant>,
}

impl InMemoryCache {
    pub fn new(max_capacity: u64, default_ttl: Duration) -> Self {
        Self {
            store: tokio::sync::RwLock::new(std::collections::HashMap::new()),
            max_capacity,
            default_ttl,
            stats: tokio::sync::RwLock::new(CacheStats::default()),
        }
    }

    pub async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let store = self.store.read().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = store.get(key) {
            // Check if expired
            if let Some(expires_at) = entry.expires_at {
                if std::time::Instant::now() > expires_at {
                    stats.misses += 1;
                    return Ok(None);
                }
            }

            stats.hits += 1;
            stats.hit_ratio = stats.hits as f64 / (stats.hits + stats.misses) as f64;

            let value: T = serde_json::from_slice(&entry.data)
                .map_err(|e| AppError::Serialization(e))?;
            Ok(Some(value))
        } else {
            stats.misses += 1;
            stats.hit_ratio = stats.hits as f64 / (stats.hits + stats.misses) as f64;
            Ok(None)
        }
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    where
        T: Serialize,
    {
        let data = serde_json::to_vec(value)
            .map_err(|e| AppError::Serialization(e))?;

        let expires_at = ttl.or(Some(self.default_ttl))
            .map(|duration| std::time::Instant::now() + duration);

        let entry = CacheEntry { data, expires_at };

        let mut store = self.store.write().await;
        let mut stats = self.stats.write().await;

        // Simple eviction if at capacity
        if store.len() >= self.max_capacity as usize && !store.contains_key(key) {
            // Remove oldest entry (simple FIFO)
            if let Some(first_key) = store.keys().next().cloned() {
                store.remove(&first_key);
                stats.entries = stats.entries.saturating_sub(1);
            }
        }

        let is_new = !store.contains_key(key);
        store.insert(key.to_string(), entry);

        if is_new {
            stats.entries += 1;
        }

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> AppResult<()> {
        let mut store = self.store.write().await;
        let mut stats = self.stats.write().await;

        if store.remove(key).is_some() {
            stats.entries = stats.entries.saturating_sub(1);
        }

        Ok(())
    }

    pub async fn exists(&self, key: &str) -> AppResult<bool> {
        let store = self.store.read().await;

        if let Some(entry) = store.get(key) {
            // Check if expired
            if let Some(expires_at) = entry.expires_at {
                if std::time::Instant::now() > expires_at {
                    return Ok(false);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn clear(&self) -> AppResult<()> {
        let mut store = self.store.write().await;
        let mut stats = self.stats.write().await;

        store.clear();
        stats.entries = 0;

        Ok(())
    }

    pub async fn stats(&self) -> AppResult<CacheStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
}

/// Redis cache implementation
pub struct RedisCache {
    client: redis::Client,
    connection: tokio::sync::Mutex<redis::aio::Connection>,
}

impl RedisCache {
    pub async fn new(redis_url: &str) -> AppResult<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        let connection = client.get_async_connection().await
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        Ok(Self {
            client,
            connection: tokio::sync::Mutex::new(connection),
        })
    }

    pub async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        use redis::AsyncCommands;

        let mut conn = self.connection.lock().await;
        let data: Option<String> = conn.get(key).await
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        if let Some(json_data) = data {
            let value: T = serde_json::from_str(&json_data)
                .map_err(|e| AppError::Serialization(e))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> AppResult<()>
    where
        T: Serialize,
    {
        use redis::AsyncCommands;

        let json_data = serde_json::to_string(value)
            .map_err(|e| crate::shared::error::AppError::Serialization(e))?;

        let mut conn = self.connection.lock().await;

        if let Some(ttl) = ttl {
            let _: () = conn.set_ex::<_, _, ()>(key, json_data, ttl.as_secs()).await
                .map_err(|e| crate::shared::error::AppError::ExternalService(e.to_string()))?;
        } else {
            let _: () = conn.set::<_, _, ()>(key, json_data).await
                .map_err(|e| crate::shared::error::AppError::ExternalService(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> AppResult<()> {
        use redis::AsyncCommands;

        let mut conn = self.connection.lock().await;
        let _: i32 = conn.del(key).await
            .map_err(|e| crate::shared::error::AppError::ExternalService(e.to_string()))?;

        Ok(())
    }

    pub async fn exists(&self, key: &str) -> AppResult<bool> {
        use redis::AsyncCommands;

        let mut conn = self.connection.lock().await;
        let exists: bool = conn.exists(key).await
            .map_err(|e| crate::shared::error::AppError::ExternalService(e.to_string()))?;

        Ok(exists)
    }

    pub async fn clear(&self) -> AppResult<()> {
        use redis::AsyncCommands;

        let mut conn = self.connection.lock().await;
        let _: () = redis::cmd("FLUSHDB").query_async(&mut *conn).await
            .map_err(|e| crate::shared::error::AppError::ExternalService(e.to_string()))?;

        Ok(())
    }

    pub async fn stats(&self) -> AppResult<CacheStats> {
        // Redis stats would require INFO commands and parsing
        // For now, return default stats
        Ok(CacheStats::default())
    }
}

/// Cache key generators for different entities
pub mod keys {
    use uuid::Uuid;
    
    pub fn user_key(user_id: Uuid) -> String {
        format!("user:{}", user_id)
    }
    
    pub fn emergency_key(emergency_id: Uuid) -> String {
        format!("emergency:{}", emergency_id)
    }
    
    pub fn user_session_key(session_id: &str) -> String {
        format!("session:{}", session_id)
    }
    
    pub fn emergency_list_key(status: &str) -> String {
        format!("emergencies:status:{}", status)
    }
    
    pub fn analytics_key(metric: &str, period: &str) -> String {
        format!("analytics:{}:{}", metric, period)
    }
}
