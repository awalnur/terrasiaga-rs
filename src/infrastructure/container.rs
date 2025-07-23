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
    /// Build the application container from configuration
    pub async fn build(config: &crate::config::AppConfig) -> AppResult<Self> {
        tracing::info!("Building application container from configuration...");

        // Create database config from app config
        let db_config = crate::infrastructure::database::DatabaseConfig {
            url: config.database.url.clone(),
            max_connections: config.database.max_connections,
            min_connections: config.database.min_connections,
            connection_timeout: std::time::Duration::from_secs(config.database.connection_timeout),
            idle_timeout: std::time::Duration::from_secs(config.database.idle_timeout),
            max_lifetime: std::time::Duration::from_secs(config.database.max_lifetime),
            enable_logging: !config.is_production(),
        };

        // Create cache config from app config
        let cache_config = crate::infrastructure::cache::CacheConfig {
            cache_type: if config.redis.url.is_empty() {
                crate::infrastructure::cache::CacheType::InMemory { max_capacity: 1000 }
            } else {
                crate::infrastructure::cache::CacheType::Redis { url: config.redis.url.clone() }
            },
            default_ttl: std::time::Duration::from_secs(config.redis.default_ttl),
            max_capacity: Some(config.redis.max_connections as u64),
            redis_url: Some(config.redis.url.clone()),
        };

        // Create external services config from app config
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

        // Initialize repositories
        tracing::info!("Initializing repositories...");
        let db_pool = database.pool().clone();

        let user_repository: Arc<dyn UserRepository> = Arc::new(
            PostgresUserRepository::new(db_pool.clone())
        );

        let disaster_repository: Arc<dyn DisasterRepository> = Arc::new(
            PostgresDisasterRepository::new(db_pool.clone())
        );

        let location_repository: Arc<dyn LocationRepository> = Arc::new(
            PostgresLocationRepository::new(db_pool.clone())
        );

        let notification_repository: Arc<dyn NotificationRepository> = Arc::new(
            PostgresNotificationRepository::new(db_pool.clone())
        );

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

        // Load configurations
        let db_config = DatabaseConfig::from_env()?;
        let cache_config = CacheConfig::from_env()?;
        let external_services_config = ExternalServicesConfig::from_env()?;

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

        // Initialize repositories
        tracing::info!("Initializing repositories...");
        let db_pool = database.pool().clone();

        let user_repository: Arc<dyn UserRepository> = Arc::new(
            PostgresUserRepository::new(db_pool.clone())
        );

        let disaster_repository: Arc<dyn DisasterRepository> = Arc::new(
            PostgresDisasterRepository::new(db_pool.clone())
        );

        let location_repository: Arc<dyn LocationRepository> = Arc::new(
            PostgresLocationRepository::new(db_pool.clone())
        );

        let notification_repository: Arc<dyn NotificationRepository> = Arc::new(
            PostgresNotificationRepository::new(db_pool.clone())
        );

        tracing::info!("Application container initialized successfully");

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

    /// Get database service
    pub fn database(&self) -> &Arc<DatabaseService> {
        &self.database
    }

    /// Get cache service
    pub fn cache(&self) -> &Arc<CacheService> {
        &self.cache
    }

    /// Get external services manager
    pub fn external_services(&self) -> &Arc<tokio::sync::Mutex<ExternalServicesManager>> {
        &self.external_services
    }

    /// Get health monitor
    pub fn health_monitor(&self) -> &Arc<HealthMonitor> {
        &self.health_monitor
    }

    /// Perform application health check
    pub async fn health_check(&self) -> AppResult<crate::infrastructure::monitoring::ApplicationHealth> {
        self.health_monitor.health_check().await
    }

    /// Graceful shutdown of all services
    pub async fn shutdown(&self) -> AppResult<()> {
        tracing::info!("Shutting down application container...");

        // Shutdown external services
        {
            let _external_services = self.external_services.lock().await;
            // External services don't need explicit shutdown in current implementation
        }

        // Cache doesn't need explicit shutdown
        // Database pool will be dropped automatically

        tracing::info!("Application container shutdown completed");
        Ok(())
    }

    /// Start background health monitoring
    pub async fn start_health_monitoring(&self) {
        let scheduler = HealthCheckScheduler::new(
            self.health_monitor.clone(),
            std::time::Duration::from_secs(30), // Check every 30 seconds
        );

        tokio::spawn(async move {
            scheduler.start().await;
        });

        tracing::info!("Health monitoring started");
    }

    /// Validate container configuration
    pub fn validate_configuration(&self) -> AppResult<()> {
        // Validate database configuration
        let db_config = self.database.config();
        if db_config.max_connections == 0 {
            return Err(AppError::Configuration(
                "Database max_connections must be greater than 0".to_string(),
            ));
        }

        // Add more validation as needed
        tracing::info!("Container configuration validation passed");
        Ok(())
    }
}

/// Container builder for more flexible initialization
pub struct AppContainerBuilder {
    db_config: Option<DatabaseConfig>,
    cache_config: Option<CacheConfig>,
    external_services_config: Option<ExternalServicesConfig>,
    enable_health_monitoring: bool,
}

impl AppContainerBuilder {
    /// Create new container builder
    pub fn new() -> Self {
        Self {
            db_config: None,
            cache_config: None,
            external_services_config: None,
            enable_health_monitoring: true,
        }
    }

    /// Set database configuration
    pub fn with_database_config(mut self, config: DatabaseConfig) -> Self {
        self.db_config = Some(config);
        self
    }

    /// Set cache configuration
    pub fn with_cache_config(mut self, config: CacheConfig) -> Self {
        self.cache_config = Some(config);
        self
    }

    /// Set external services configuration
    pub fn with_external_services_config(mut self, config: ExternalServicesConfig) -> Self {
        self.external_services_config = Some(config);
        self
    }

    /// Enable or disable health monitoring
    pub fn with_health_monitoring(mut self, enable: bool) -> Self {
        self.enable_health_monitoring = enable;
        self
    }

    /// Build the container
    pub async fn build(self) -> AppResult<AppContainer> {
        let db_config = self.db_config.unwrap_or(DatabaseConfig::from_env()?);
        let cache_config = self.cache_config.unwrap_or(CacheConfig::from_env()?);
        let external_services_config = self.external_services_config
            .unwrap_or(ExternalServicesConfig::from_env()?);

        // Initialize core infrastructure
        let database = Arc::new(DatabaseService::new(db_config).await?);
        database.run_migrations()?;

        let cache = Arc::new(CacheService::new(cache_config).await?);

        let external_services = Arc::new(tokio::sync::Mutex::new(
            ExternalServicesManager::new(external_services_config)?
        ));

        let health_monitor = Arc::new(HealthMonitor::new(
            database.clone(),
            cache.clone(),
            external_services.clone(),
        ));

        // Initialize repositories
        let db_pool = database.pool().clone();

        let user_repository: Arc<dyn UserRepository> = Arc::new(
            PostgresUserRepository::new(db_pool.clone())
        );

        let disaster_repository: Arc<dyn DisasterRepository> = Arc::new(
            PostgresDisasterRepository::new(db_pool.clone())
        );

        let location_repository: Arc<dyn LocationRepository> = Arc::new(
            PostgresLocationRepository::new(db_pool.clone())
        );

        let notification_repository: Arc<dyn NotificationRepository> = Arc::new(
            PostgresNotificationRepository::new(db_pool.clone())
        );

        let container = AppContainer {
            database,
            cache,
            external_services,
            health_monitor,
            user_repository,
            disaster_repository,
            location_repository,
            notification_repository,
        };

        if self.enable_health_monitoring {
            container.start_health_monitoring().await;
        }

        Ok(container)
    }
}

impl Default for AppContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Container factory for different environments
pub struct ContainerFactory;

impl ContainerFactory {
    /// Create container for development environment
    pub async fn development() -> AppResult<AppContainer> {
        AppContainerBuilder::new()
            .with_health_monitoring(true)
            .build()
            .await
    }

    /// Create container for testing environment
    pub async fn testing() -> AppResult<AppContainer> {
        AppContainerBuilder::new()
            .with_health_monitoring(false)
            .build()
            .await
    }

    /// Create container for production environment
    pub async fn production() -> AppResult<AppContainer> {
        let container = AppContainerBuilder::new()
            .with_health_monitoring(true)
            .build()
            .await?;

        // Additional production validations
        container.validate_configuration()?;

        Ok(container)
    }
}
