/// Infrastructure layer macros for reducing boilerplate and improving productivity
/// These macros help with dependency injection, configuration, and repository initialization

use std::sync::Arc;

/// Macro untuk batch initialization repositories dengan error handling yang konsisten
#[macro_export]
macro_rules! init_repositories {
    ($pool:expr; $($repo_name:ident: $repo_type:ty => $trait_type:ty),* $(,)?) => {
        {
            use std::sync::Arc;
            let pool = $pool;

            $(
                let $repo_name: Arc<$trait_type> = Arc::new(
                    <$repo_type>::new(pool.clone())
                );
                tracing::debug!("Initialized repository: {}", stringify!($repo_name));
            )*

            ($($repo_name,)*)
        }
    };
}

/// Macro untuk membuat repository constructor dengan error handling yang lebih baik
#[macro_export]
macro_rules! create_repository {
    ($repo_type:ty, $pool:expr) => {
        std::sync::Arc::new(<$repo_type>::new($pool)) as std::sync::Arc<dyn _>
    };
    ($repo_type:ty, $pool:expr, $trait:ty) => {
        std::sync::Arc::new(<$repo_type>::new($pool)) as std::sync::Arc<$trait>
    };
}

/// Macro untuk membuat service dengan dependency injection yang lebih fleksibel
#[macro_export]
macro_rules! create_service {
    ($service_type:ty, $($deps:expr),*) => {
        std::sync::Arc::new(<$service_type>::new($($deps),*))
    };
    ($service_type:ty) => {
        std::sync::Arc::new(<$service_type>::default())
    };
}

/// Macro untuk database connection handling dengan automatic retry
#[macro_export]
macro_rules! with_db_connection {
    ($pool:expr, $operation:expr) => {
        {
            use $crate::shared::error::AppError;
            use tokio::time::{sleep, Duration};

            let mut retry_count = 0;
            const MAX_RETRIES: u32 = 3;

            loop {
                match $pool.get() {
                    Ok(mut conn) => {
                        let result = tokio::task::spawn_blocking(move || {
                            $operation(&mut conn)
                        }).await;

                        match result {
                            Ok(Ok(value)) => break Ok(value),
                            Ok(Err(e)) => {
                                if retry_count >= MAX_RETRIES {
                                    break Err(AppError::Database {
                                        message: format!("Database operation failed after {} retries: {}", MAX_RETRIES, e),
                                        source: Some(Box::new(e)),
                                    });
                                }
                                retry_count += 1;
                                tracing::warn!("Database operation failed, retrying ({}/{}): {}", retry_count, MAX_RETRIES, e);
                                sleep(Duration::from_millis(100 * retry_count as u64)).await;
                            }
                            Err(join_error) => {
                                break Err(AppError::Database {
                                    message: format!("Task join error: {}", join_error),
                                    source: Some(Box::new(join_error)),
                                });
                            }
                        }
                    }
                    Err(pool_error) => {
                        if retry_count >= MAX_RETRIES {
                            break Err(AppError::Database {
                                message: format!("Failed to get database connection after {} retries: {}", MAX_RETRIES, pool_error),
                                source: Some(Box::new(pool_error)),
                            });
                        }
                        retry_count += 1;
                        tracing::warn!("Failed to get database connection, retrying ({}/{}): {}", retry_count, MAX_RETRIES, pool_error);
                        sleep(Duration::from_millis(100 * retry_count as u64)).await;
                    }
                }
            }
        }
    };
}

/// Macro untuk konfigurasi database dengan default values yang lebih comprehensive
#[macro_export]
macro_rules! database_config {
    ($url:expr) => {
        $crate::infrastructure::database::DatabaseConfig {
            url: $url.to_string(),
            max_connections: 10,
            min_connections: 1,
            connection_timeout: std::time::Duration::from_secs(30),
            idle_timeout: std::time::Duration::from_secs(600),
            max_lifetime: std::time::Duration::from_secs(1800),
            enable_logging: cfg!(debug_assertions),
        }
    };
    ($url:expr, max_conn: $max:expr, min_conn: $min:expr) => {
        $crate::infrastructure::database::DatabaseConfig {
            url: $url.to_string(),
            max_connections: $max,
            min_connections: $min,
            connection_timeout: std::time::Duration::from_secs(30),
            idle_timeout: std::time::Duration::from_secs(600),
            max_lifetime: std::time::Duration::from_secs(1800),
            enable_logging: cfg!(debug_assertions),
        }
    };
    ($url:expr, {
        max_connections: $max:expr,
        min_connections: $min:expr,
        connection_timeout: $conn_timeout:expr,
        idle_timeout: $idle_timeout:expr,
        max_lifetime: $max_lifetime:expr,
        enable_logging: $logging:expr
    }) => {
        $crate::infrastructure::database::DatabaseConfig {
            url: $url.to_string(),
            max_connections: $max,
            min_connections: $min,
            connection_timeout: std::time::Duration::from_secs($conn_timeout),
            idle_timeout: std::time::Duration::from_secs($idle_timeout),
            max_lifetime: std::time::Duration::from_secs($max_lifetime),
            enable_logging: $logging,
        }
    };
}

/// Macro untuk konfigurasi cache dengan pattern matching yang lebih baik
#[macro_export]
macro_rules! cache_config {
    (in_memory) => {
        $crate::infrastructure::cache::CacheConfig {
            cache_type: $crate::infrastructure::cache::CacheType::InMemory { max_capacity: 1000 },
            default_ttl: std::time::Duration::from_secs(3600),
            max_capacity: Some(1000),
            redis_url: None,
        }
    };
    (in_memory, capacity: $cap:expr) => {
        $crate::infrastructure::cache::CacheConfig {
            cache_type: $crate::infrastructure::cache::CacheType::InMemory { max_capacity: $cap },
            default_ttl: std::time::Duration::from_secs(3600),
            max_capacity: Some($cap),
            redis_url: None,
        }
    };
    (redis, $url:expr) => {
        $crate::infrastructure::cache::CacheConfig {
            cache_type: $crate::infrastructure::cache::CacheType::Redis { url: $url.to_string() },
            default_ttl: std::time::Duration::from_secs(3600),
            max_capacity: Some(100),
            redis_url: Some($url.to_string()),
        }
    };
    ($cache_type:expr, {
        default_ttl: $ttl:expr,
        max_capacity: $capacity:expr
    }) => {
        $crate::infrastructure::cache::CacheConfig {
            cache_type: $cache_type,
            default_ttl: std::time::Duration::from_secs($ttl),
            max_capacity: Some($capacity),
            redis_url: None,
        }
    };
}

/// Macro untuk inisialisasi infrastructure components dengan error handling dan tracing
#[macro_export]
macro_rules! init_infrastructure {
    ($config:expr; $($component:ident: $component_type:ty),* $(,)?) => {
        {
            use std::sync::Arc;

            $(
                tracing::info!("Initializing {}...", stringify!($component));
                let $component = Arc::new(<$component_type>::new($config.clone()).await?);
                tracing::info!("{} initialized successfully", stringify!($component));
            )*

            ($($component,)*)
        }
    };
}

/// Macro untuk health check implementation
#[macro_export]
macro_rules! impl_health_check {
    ($struct_name:ident, $check_fn:expr) => {
        #[async_trait::async_trait]
        impl $crate::infrastructure::monitoring::HealthCheck for $struct_name {
            async fn health_check(&self) -> $crate::shared::AppResult<()> {
                $check_fn(self).await
            }

            fn component_name(&self) -> &'static str {
                stringify!($struct_name)
            }
        }
    };
}

/// Macro untuk dependency injection container builder pattern
#[macro_export]
macro_rules! container_builder {
    (
        struct $container_name:ident {
            $($field:ident: $field_type:ty),* $(,)?
        }

        impl {
            $($method_name:ident($($param:ident: $param_type:ty),*) -> $return_type:ty $method_body:block)*
        }
    ) => {
        pub struct $container_name {
            $(pub $field: $field_type,)*
        }

        impl $container_name {
            $(
                pub async fn $method_name($($param: $param_type),*) -> $return_type $method_body
            )*
        }
    };
}

/// Macro untuk repository trait implementation boilerplate
#[macro_export]
macro_rules! impl_repository_base {
    ($repo_struct:ident, $pool_type:ty) => {
        impl $repo_struct {
            pub fn new(pool: $pool_type) -> Self {
                Self { pool }
            }

            fn get_connection(&self) -> $crate::shared::AppResult<$crate::infrastructure::database::DbConnection> {
                self.pool.get().map_err(|e| $crate::shared::error::AppError::Database {
                    message: format!("Failed to get database connection: {}", e),
                    source: Some(Box::new(e)),
                })
            }
        }
    };
}

/// Macro untuk async operation dengan timeout dan retry
#[macro_export]
macro_rules! with_timeout_retry {
    ($operation:expr, timeout: $timeout:expr, retries: $retries:expr) => {
        {
            use tokio::time::{timeout, Duration, sleep};
            use $crate::shared::error::AppError;

            let mut attempts = 0;
            let max_attempts = $retries + 1;

            loop {
                attempts += 1;

                match timeout(Duration::from_secs($timeout), $operation).await {
                    Ok(Ok(result)) => break Ok(result),
                    Ok(Err(e)) if attempts >= max_attempts => {
                        break Err(AppError::External {
                            message: format!("Operation failed after {} attempts: {}", max_attempts, e),
                            source: Some(Box::new(e)),
                        });
                    }
                    Ok(Err(e)) => {
                        tracing::warn!("Operation failed (attempt {}/{}): {}", attempts, max_attempts, e);
                        sleep(Duration::from_millis(1000 * attempts as u64)).await;
                    }
                    Err(_timeout_err) if attempts >= max_attempts => {
                        break Err(AppError::External {
                            message: format!("Operation timed out after {} attempts", max_attempts),
                            source: None,
                        });
                    }
                    Err(_timeout_err) => {
                        tracing::warn!("Operation timed out (attempt {}/{})", attempts, max_attempts);
                        sleep(Duration::from_millis(1000 * attempts as u64)).await;
                    }
                }
            }
        }
    };
}

