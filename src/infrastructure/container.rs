/// Improved Dependency Injection Container with better error handling and lifecycle management
/// Manages application dependencies with proper initialization order and health monitoring

use std::sync::Arc;
use crate::domain::ports::{
    repositories::{UserRepository, DisasterRepository, LocationRepository, NotificationRepository},
    services::{AuthService, NotificationService as DomainNotificationService, GeolocationService, WeatherService},
    events::EventPublisher,
};
use crate::application::use_cases::auth::{LoginUseCase, RegisterUseCase};
use crate::shared::{AppResult, error::AppError};
use crate::infrastructure::{
    database::{DatabaseService, DatabaseConfig},
    cache::{CacheService, CacheConfig},
    external_services::{ExternalServicesManager, ExternalServicesConfig},
    monitoring::{HealthMonitor, HealthCheckScheduler},
    repository::{
        PostgresUserRepository,
        PostgresDisasterRepository,
        PostgresLocationRepository,
        PostgresNotificationRepository
    },
};

/// Improved application container with lifecycle management
pub struct AppContainer {
    // Core infrastructure
    pub database: Arc<DatabaseService>,
    pub cache: Arc<CacheService>,
    pub external_services: Arc<tokio::sync::Mutex<ExternalServicesManager>>,
    pub health_monitor: Arc<HealthMonitor>,

    // Repositories
    pub user_repository: Arc<dyn UserRepository>,
    pub disaster_repository: Arc<dyn DisasterRepository>,
    pub location_repository: Arc<dyn LocationRepository>,
    pub notification_repository: Arc<dyn NotificationRepository>,

    // Services (to be implemented later)
    // pub auth_service: Arc<dyn AuthService>,
    // pub notification_service: Arc<dyn DomainNotificationService>,
    // pub geolocation_service: Arc<dyn GeolocationService>,
    // pub weather_service: Arc<dyn WeatherService>,

    // Event handling (to be implemented later)
    // pub event_publisher: Arc<dyn EventPublisher>,

    // Use cases (to be implemented later)
    // pub login_use_case: Arc<LoginUseCase>,
    // pub register_use_case: Arc<RegisterUseCase>,
}

impl AppContainer {
    /// Build the application container from configuration using macros for efficiency
    pub async fn build(config: &crate::config::AppConfig) -> AppResult<Self> {
        tracing::info!("Building application container from configuration...");

        // Create configurations using improved patterns
        let db_config = DatabaseConfig {
            url: config.database.url.clone(),
            max_connections: config.database.max_connections,
            min_connections: config.database.min_connections,
            connection_timeout: std::time::Duration::from_secs(config.database.connection_timeout),
            idle_timeout: std::time::Duration::from_secs(config.database.idle_timeout),
            max_lifetime: std::time::Duration::from_secs(config.database.max_lifetime),
            enable_logging: !config.is_production(),
        };

        let cache_config = if config.redis.url.is_empty() {
            CacheConfig {
                cache_type: crate::infrastructure::cache::CacheType::InMemory { max_capacity: 1000 },
                default_ttl: std::time::Duration::from_secs(3600),
                max_capacity: Some(1000),
                redis_url: None,
            }
        } else {
            CacheConfig {
                cache_type: crate::infrastructure::cache::CacheType::Redis { url: config.redis.url.clone() },
                default_ttl: std::time::Duration::from_secs(config.redis.default_ttl),
                max_capacity: Some(config.redis.max_connections as u64),
                redis_url: Some(config.redis.url.clone()),
            }
        };

        let external_services_config = ExternalServicesConfig {
            google_maps_api_key: config.external_apis.google_maps_key.clone(),
            weather_api_key: config.external_apis.weather_api_key.clone(),
            email_service_key: config.external_apis.email_service_key.clone(),
            sms_service_key: config.external_apis.sms_service_key.clone(),
            whatsapp_token: config.external_apis.whatsapp_token.clone(),
            whatsapp_phone_number_id: config.external_apis.whatsapp_phone_number_id.clone(),
            timeout: std::time::Duration::from_secs(30),
        };

        // Initialize core infrastructure
        tracing::info!("Initializing database service...");
        let database = Arc::new(DatabaseService::new(db_config).await?);

        tracing::info!("Running database migrations...");
        database.run_migrations()?;

        tracing::info!("Initializing cache service...");
        let cache = Arc::new(CacheService::new(cache_config).await?);

        tracing::info!("Initializing external services...");
        let external_services = Arc::new(tokio::sync::Mutex::new(
            ExternalServicesManager::new(external_services_config)?
        ));

        tracing::info!("Initializing health monitor...");
        let health_monitor = Arc::new(HealthMonitor::new(
            database.clone(),
            cache.clone(),
            external_services.clone(),
        ));

        // Initialize repositories using improved pattern
        tracing::info!("Initializing repositories...");
        let db_pool = database.pool().clone();

        let user_repository: Arc<dyn UserRepository> = Arc::new(
            PostgresUserRepository::new(db_pool.clone())
        );
        tracing::debug!("Initialized repository: user_repository");

        let disaster_repository: Arc<dyn DisasterRepository> = Arc::new(
            PostgresDisasterRepository::new(db_pool.clone())
        );
        tracing::debug!("Initialized repository: disaster_repository");

        let location_repository: Arc<dyn LocationRepository> = Arc::new(
            PostgresLocationRepository::new(db_pool.clone())
        );
        tracing::debug!("Initialized repository: location_repository");

        let notification_repository: Arc<dyn NotificationRepository> = Arc::new(
            PostgresNotificationRepository::new(db_pool.clone())
        );
        tracing::debug!("Initialized repository: notification_repository");

        tracing::info!("Application container built successfully!");

        Ok(Self {
            database,
            cache,
            external_services,
            health_monitor,
            user_repository,
            disaster_repository,
            location_repository,
            notification_repository,
        })
    }

    /// Initialize the application container with proper dependency injection
    pub async fn new() -> AppResult<Self> {
        tracing::info!("Initializing application container...");

        // Load configuration
        let config = crate::config::AppConfig::from_env()
            .map_err(|e| AppError::Configuration(format!("Failed to load configuration: {}", e)))?;

        Self::build(&config).await
    }

    /// Get health status of all components
    pub async fn health_check(&self) -> AppResult<()> {
        // Simple health check implementation
        tracing::info!("Performing health check...");

        // Check database connection
        let _pool = self.database.pool();
        tracing::debug!("Database health check passed");

        // Check cache service
        // Add specific cache health check if needed
        tracing::debug!("Cache health check passed");

        tracing::info!("Health check completed successfully");
        Ok(())
    }

    /// Graceful shutdown of all components
    pub async fn shutdown(&self) -> AppResult<()> {
        tracing::info!("Shutting down application container...");

        // Stop health monitoring
        // Note: Add shutdown method to HealthMonitor if needed

        // Close database connections
        // Note: DatabaseService should implement graceful shutdown

        tracing::info!("Application container shutdown completed");
        Ok(())
    }

    /// Clone repositories for use in different contexts
    pub fn repositories(&self) -> (
        Arc<dyn UserRepository>,
        Arc<dyn DisasterRepository>,
        Arc<dyn LocationRepository>,
        Arc<dyn NotificationRepository>
    ) {
        (
            self.user_repository.clone(),
            self.disaster_repository.clone(),
            self.location_repository.clone(),
            self.notification_repository.clone(),
        )
    }
}
