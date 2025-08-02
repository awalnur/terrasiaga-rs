/// Advanced Infrastructure Macros for Terra Siaga
/// Makro khusus untuk pola-pola yang berulang dalam infrastructure layer

/// Macro untuk membuat service configuration dari environment variables
#[macro_export]
macro_rules! env_config {
    (
        $config_name:ident {
            $(
                $field:ident: $env_var:expr => $default:expr
            ),* $(,)?
        }
    ) => {
        impl $config_name {
            pub fn from_env() -> $crate::shared::AppResult<Self> {
                Ok(Self {
                    $(
                        $field: std::env::var($env_var)
                            .unwrap_or_else(|_| $default.to_string()),
                    )*
                })
            }

            pub fn validate(&self) -> $crate::shared::AppResult<()> {
                $(
                    if self.$field.is_empty() {
                        tracing::warn!("Configuration field {} is empty", stringify!($field));
                    }
                )*
                Ok(())
            }
        }
    };
}

/// Macro untuk membuat enum service dengan dispatch pattern
#[macro_export]
macro_rules! service_enum {
    (
        $service_name:ident {
            $(
                $variant:ident($impl_type:ty)
            ),* $(,)?
        }
    ) => {
        pub enum $service_name {
            $(
                $variant($impl_type),
            )*
            Disabled,
        }

        impl $service_name {
            pub async fn new(config: impl Into<ServiceConfig>) -> $crate::shared::AppResult<Self> {
                let config = config.into();
                match config.service_type {
                    $(
                        ServiceType::$variant => {
                            let service = <$impl_type>::new(config).await?;
                            Ok(Self::$variant(service))
                        }
                    )*
                    ServiceType::Disabled => Ok(Self::Disabled),
                }
            }
        }
    };
}

/// Macro untuk connection pool management pattern
#[macro_export]
macro_rules! connection_pool {
    (
        $pool_name:ident,
        $connection_type:ty,
        $config_type:ty,
        $error_type:ty
    ) => {
        pub struct $pool_name {
            pool: Pool<ConnectionManager<$connection_type>>,
            config: $config_type,
        }

        impl $pool_name {
            pub async fn new(config: $config_type) -> Result<Self, $error_type> {
                let manager = ConnectionManager::<$connection_type>::new(&config.url);

                let pool = Pool::builder()
                    .max_size(config.max_connections)
                    .min_idle(Some(config.min_connections))
                    .connection_timeout(config.connection_timeout)
                    .idle_timeout(Some(config.idle_timeout))
                    .max_lifetime(Some(config.max_lifetime))
                    .test_on_check_out(true)
                    .build(manager)?;

                // Test connection
                {
                    let mut conn = pool.get()?;
                    // Health check implementation
                }

                Ok(Self { pool, config })
            }

            pub fn get_connection(&self) -> Result<PooledConnection<ConnectionManager<$connection_type>>, $error_type> {
                self.pool.get().map_err(Into::into)
            }

            pub fn pool(&self) -> &Pool<ConnectionManager<$connection_type>> {
                &self.pool
            }

            pub fn config(&self) -> &$config_type {
                &self.config
            }
        }
    };
}

/// Macro untuk health check pattern
#[macro_export]
macro_rules! health_checker {
    (
        $health_name:ident {
            $(
                $component:ident: $component_type:ty => $check_method:ident
            ),* $(,)?
        }
    ) => {
        pub struct $health_name {
            $(
                $component: std::sync::Arc<$component_type>,
            )*
        }

        impl $health_name {
            pub fn new(
                $(
                    $component: std::sync::Arc<$component_type>,
                )*
            ) -> Self {
                Self {
                    $(
                        $component,
                    )*
                }
            }

            pub async fn check_health(&self) -> $crate::infrastructure::monitoring::ApplicationHealth {
                use $crate::infrastructure::monitoring::{HealthStatus, ComponentHealth};

                let mut components = std::collections::HashMap::new();
                let mut overall_healthy = true;

                $(
                    let health = self.$component.$check_method().await;
                    let is_healthy = matches!(health.status, HealthStatus::Healthy);
                    overall_healthy &= is_healthy;
                    components.insert(stringify!($component).to_string(), health);
                )*

                let overall_status = if overall_healthy {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Degraded
                };

                $crate::infrastructure::monitoring::ApplicationHealth {
                    overall_status,
                    components,
                    timestamp: chrono::Utc::now(),
                    uptime: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default(),
                }
            }
        }
    };
}

/// Macro untuk async retry pattern dengan exponential backoff
#[macro_export]
macro_rules! retry_with_backoff {
    (
        $operation:expr,
        max_attempts: $max_attempts:expr,
        initial_delay: $initial_delay:expr,
        max_delay: $max_delay:expr
    ) => {
        {
            let mut attempts = 0;
            let mut delay = $initial_delay;

            loop {
                attempts += 1;

                match $operation.await {
                    Ok(result) => break Ok(result),
                    Err(e) if attempts >= $max_attempts => break Err(e),
                    Err(e) => {
                        tracing::warn!(
                            "Operation failed (attempt {}/{}): {}. Retrying in {:?}...",
                            attempts, $max_attempts, e, delay
                        );

                        tokio::time::sleep(delay).await;
                        delay = std::cmp::min(delay * 2, $max_delay);
                    }
                }
            }
        }
    };
}

/// Macro untuk circuit breaker pattern
#[macro_export]
macro_rules! circuit_breaker {
    ($service_name:ident, $error_threshold:expr, $timeout:expr) => {
        pub struct $service_name {
            inner: std::sync::Arc<tokio::sync::Mutex<CircuitBreakerState>>,
        }

        struct CircuitBreakerState {
            failure_count: u32,
            last_failure: Option<std::time::Instant>,
            is_open: bool,
        }

        impl $service_name {
            pub fn new() -> Self {
                Self {
                    inner: std::sync::Arc::new(tokio::sync::Mutex::new(CircuitBreakerState {
                        failure_count: 0,
                        last_failure: None,
                        is_open: false,
                    })),
                }
            }

            pub async fn call<F, T, E>(&self, operation: F) -> Result<T, E>
            where
                F: std::future::Future<Output = Result<T, E>>,
                E: std::fmt::Display,
            {
                let mut state = self.inner.lock().await;

                // Check if circuit is open
                if state.is_open {
                    if let Some(last_failure) = state.last_failure {
                        if last_failure.elapsed() < $timeout {
                            return Err(format!("Circuit breaker is open").into());
                        } else {
                            // Half-open state
                            state.is_open = false;
                        }
                    }
                }

                drop(state);

                match operation.await {
                    Ok(result) => {
                        // Reset on success
                        let mut state = self.inner.lock().await;
                        state.failure_count = 0;
                        state.last_failure = None;
                        Ok(result)
                    }
                    Err(e) => {
                        let mut state = self.inner.lock().await;
                        state.failure_count += 1;
                        state.last_failure = Some(std::time::Instant::now());

                        if state.failure_count >= $error_threshold {
                            state.is_open = true;
                            tracing::error!("Circuit breaker opened due to {} failures", state.failure_count);
                        }

                        Err(e)
                    }
                }
            }
        }
    };
}

/// Macro untuk caching decorator pattern
#[macro_export]
macro_rules! cached_service {
    (
        $service_name:ident,
        $cache_type:ty,
        $key_type:ty,
        $value_type:ty,
        $ttl:expr
    ) => {
        pub struct $service_name<T> {
            inner: T,
            cache: std::sync::Arc<$cache_type>,
        }

        impl<T> $service_name<T> {
            pub fn new(inner: T, cache: std::sync::Arc<$cache_type>) -> Self {
                Self { inner, cache }
            }

            pub async fn get_or_compute<F, Fut>(
                &self,
                key: $key_type,
                compute: F,
            ) -> $crate::shared::AppResult<$value_type>
            where
                F: FnOnce() -> Fut,
                Fut: std::future::Future<Output = $crate::shared::AppResult<$value_type>>,
                $value_type: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone,
            {
                // Try cache first
                if let Ok(Some(cached)) = self.cache.get(&key.to_string()).await {
                    trace_infra!(debug, "cache", &format!("Cache hit for key: {}", key));
                    return Ok(cached);
                }

                // Compute value
                let value = compute().await?;

                // Store in cache
                if let Err(e) = self.cache.set(&key.to_string(), &value, Some($ttl)).await {
                    trace_infra!(warn, "cache", &format!("Failed to cache value: {}", e));
                }

                Ok(value)
            }
        }
    };
}

/// Macro untuk monitoring metrics
#[macro_export]
macro_rules! metrics_collector {
    (
        $collector_name:ident {
            $(
                $metric_name:ident: $metric_type:ident
            ),* $(,)?
        }
    ) => {
        pub struct $collector_name {
            $(
                $metric_name: std::sync::Arc<std::sync::atomic::AtomicU64>,
            )*
        }

        impl $collector_name {
            pub fn new() -> Self {
                Self {
                    $(
                        $metric_name: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
                    )*
                }
            }

            $(
                pub fn increment_$metric_name(&self) {
                    self.$metric_name.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }

                pub fn get_$metric_name(&self) -> u64 {
                    self.$metric_name.load(std::sync::atomic::Ordering::Relaxed)
                }
            )*

            pub fn get_all_metrics(&self) -> std::collections::HashMap<String, u64> {
                let mut metrics = std::collections::HashMap::new();
                $(
                    metrics.insert(
                        stringify!($metric_name).to_string(),
                        self.get_$metric_name()
                    );
                )*
                metrics
            }
        }
    };
}

/// Macro untuk rate limiting
#[macro_export]
macro_rules! rate_limiter {
    ($limiter_name:ident, $max_requests:expr, $window:expr) => {
        pub struct $limiter_name {
            requests: std::sync::Arc<tokio::sync::Mutex<std::collections::VecDeque<std::time::Instant>>>,
            max_requests: usize,
            window: std::time::Duration,
        }

        impl $limiter_name {
            pub fn new() -> Self {
                Self {
                    requests: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::VecDeque::new())),
                    max_requests: $max_requests,
                    window: $window,
                }
            }

            pub async fn check_rate_limit(&self) -> bool {
                let mut requests = self.requests.lock().await;
                let now = std::time::Instant::now();

                // Remove old requests outside the window
                while let Some(&front) = requests.front() {
                    if now.duration_since(front) > self.window {
                        requests.pop_front();
                    } else {
                        break;
                    }
                }

                // Check if we're under the limit
                if requests.len() < self.max_requests {
                    requests.push_back(now);
                    true
                } else {
                    false
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_config_macro() {
        // Test that env_config macro compiles
        struct TestConfig {
            api_key: String,
            timeout: String,
        }

        env_config!(TestConfig {
            api_key: "API_KEY" => "default_key",
            timeout: "TIMEOUT" => "30"
        });

        // Test would require actual implementation
    }
}
