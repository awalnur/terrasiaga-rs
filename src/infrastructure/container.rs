/// Enhanced Application Container with PASETO integration
/// Provides comprehensive dependency injection for all Terra Siaga components

use std::sync::Arc;
use actix_web::web;

use crate::config::AppConfig;
use crate::infrastructure::{
    cache::{CacheService, CacheConfig},
    monitoring::health::{HealthService, CacheHealthChecker, DatabaseHealthChecker},
    external_services::{
        notification_service::ExternalNotificationService,
        SmsConfig, EmailConfig, WhatsAppConfig,
    },
    security::paseto_service::{PasetoSecurityService, PasetoConfig},
    repository::user_repository::PostgresUserRepository,
};
use crate::domain::{
    ports::{
        repositories::{UserRepository, DisasterRepository, NotificationRepository},
        services::{AuthService, NotificationService, GeolocationService as GeoService},
    },
    value_objects::Coordinates,
};
use crate::shared::{AppResult, AppError, cache::CacheFactory};
use crate::application::use_cases::*;
use crate::infrastructure::monitoring::{CacheHealthCheck, DatabaseHealthCheck, HealthMonitoringService};

/// Application container holding all dependencies
pub struct AppContainer {
    // Core services
    pub auth_service: Arc<dyn AuthService>,
    pub paseto_service: Arc<PasetoSecurityService>,
    pub cache_service: Arc<dyn CacheService>,
    pub notification_service: Arc<dyn NotificationService>,
    pub health_monitoring: Arc<HealthService>,

    // Repositories
    pub user_repository: Arc<dyn UserRepository>,
    pub disaster_repository: Arc<dyn DisasterRepository>,
    pub notification_repository: Arc<dyn NotificationRepository>,

    // Use cases
    pub register_user_use_case: Arc<RegisterUserUseCase>,
    pub update_user_profile_use_case: Arc<UpdateUserProfileUseCase>,
    pub change_user_status_use_case: Arc<ChangeUserStatusUseCase>,
    pub report_disaster_use_case: Arc<ReportDisasterUseCase>,
    pub update_disaster_status_use_case: Arc<UpdateDisasterStatusUseCase>,
    pub get_nearby_disasters_use_case: Arc<GetNearbyDisastersUseCase>,
    pub dispatch_emergency_response_use_case: Arc<DispatchEmergencyResponseUseCase>,
    pub send_emergency_alert_use_case: Arc<SendEmergencyAlertUseCase>,
    pub send_custom_notification_use_case: Arc<SendCustomNotificationUseCase>,

    // Configuration
    pub config: AppConfig,
}

impl AppContainer {
    /// Build the complete application container
    pub async fn build(config: &AppConfig) -> AppResult<Self> {
        tracing::info!("Building application container...");

        // Build database connection pool
        let database_pool = crate::infrastructure::database::create_connection_pool(config)
            .map_err(|e| AppError::Configuration(format!("Database setup failed: {}", e)))?;
        let database_pool = Arc::new(database_pool);

        // Build cache service
        let cache_service = Self::build_cache_service(config).await?;
        let cache_service = Arc::new(cache_service);

        // Build PASETO security service
        let paseto_config = Self::build_paseto_config(config)?;
        let paseto_service = PasetoSecurityService::new(
            paseto_config,
            cache_service.clone(),
        )?;
        let paseto_service = Arc::new(paseto_service);

        // Build repositories
        let user_repository = Arc::new(PostgresUserRepository::new(
            database_pool.clone(),
            cache_service.clone(),
        )) as Arc<dyn UserRepository>;

        // For now, we'll create placeholder implementations for other repositories
        // In a complete implementation, these would be actual PostgreSQL repositories
        let disaster_repository = Self::create_placeholder_disaster_repository();
        let notification_repository = Self::create_placeholder_notification_repository();

        // Build external services
        let notification_service = Self::build_notification_service(config)?;
        let notification_service = Arc::new(notification_service);

        // Build monitoring service
        let health_monitoring = Self::build_health_monitoring(
            database_pool.clone(),
            cache_service.clone(),
            config,
        ).await?;
        let health_monitoring = Arc::new(health_monitoring);

        // Build use cases
        let auth_service = paseto_service.clone() as Arc<dyn AuthService>;

        let register_user_use_case = Arc::new(RegisterUserUseCase::new(
            user_repository.clone(),
            auth_service.clone(),
            notification_service.clone(),
            Self::create_placeholder_event_publisher(),
        ));

        let update_user_profile_use_case = Arc::new(UpdateUserProfileUseCase::new(
            user_repository.clone(),
        ));

        let change_user_status_use_case = Arc::new(ChangeUserStatusUseCase::new(
            user_repository.clone(),
            Self::create_placeholder_event_publisher(),
        ));

        let report_disaster_use_case = Arc::new(ReportDisasterUseCase::new(
            disaster_repository.clone(),
            notification_service.clone(),
            Self::create_placeholder_event_publisher(),
        ));

        let update_disaster_status_use_case = Arc::new(UpdateDisasterStatusUseCase::new(
            disaster_repository.clone(),
            Self::create_placeholder_event_publisher(),
        ));

        let get_nearby_disasters_use_case = Arc::new(GetNearbyDisastersUseCase::new(
            disaster_repository.clone(),
        ));

        let dispatch_emergency_response_use_case = Arc::new(DispatchEmergencyResponseUseCase::new(
            disaster_repository.clone(),
            user_repository.clone(),
            notification_service.clone(),
            Self::create_placeholder_geo_service(),
            Self::create_placeholder_event_publisher(),
        ));

        let send_emergency_alert_use_case = Arc::new(SendEmergencyAlertUseCase::new(
            notification_repository.clone(),
            user_repository.clone(),
            disaster_repository.clone(),
            notification_service.clone(),
            Self::create_placeholder_geo_service(),
            Self::create_placeholder_event_publisher(),
        ));

        let send_custom_notification_use_case = Arc::new(SendCustomNotificationUseCase::new(
            notification_repository.clone(),
            user_repository.clone(),
            notification_service.clone(),
            Self::create_placeholder_event_publisher(),
        ));

        tracing::info!("Application container built successfully");

        Ok(AppContainer {
            auth_service,
            paseto_service,
            cache_service,
            notification_service,
            health_monitoring,
            user_repository,
            disaster_repository,
            notification_repository,
            register_user_use_case,
            update_user_profile_use_case,
            change_user_status_use_case,
            report_disaster_use_case,
            update_disaster_status_use_case,
            get_nearby_disasters_use_case,
            dispatch_emergency_response_use_case,
            send_emergency_alert_use_case,
            send_custom_notification_use_case,
            config: config.clone(),
        })
    }

    /// Build cache service based on configuration
    async fn build_cache_service(config: &AppConfig) -> AppResult<impl CacheService> {
        let cache_config = CacheConfig {
            redis_url: if config.is_production() {
                Some(config.redis_url())
            } else {
                None
            },
            redis_pool_size: Some(10),
            memory_cache_capacity: if config.is_production() { 10000 } else { 1000 },
            default_ttl_seconds: if config.is_production() { 3600 } else { 600 },
            key_prefix: format!("terra_siaga:{}", config.environment),
        };

        CacheFactory::create(&cache_config).await
    }

    /// Build PASETO configuration
    fn build_paseto_config(config: &AppConfig) -> AppResult<PasetoConfig> {
        // In production, these keys should come from secure key management
        let paseto_config = if config.is_production() {
            PasetoConfig {
                local_key: config.security.jwt_secret.as_bytes()[..32].to_vec(),
                public_key: vec![], // Would be loaded from secure storage
                private_key: vec![], // Would be loaded from secure storage
                token_expiration_hours: config.security.jwt_expiration_hours,
                session_timeout_hours: config.security.session_timeout_hours,
                elevated_session_minutes: 15,
                use_local_tokens: true, // Use encrypted tokens in production
            }
        } else {
            PasetoConfig::default()
        };

        Ok(paseto_config)
    }

    /// Build notification service with all providers
    fn build_notification_service(config: &AppConfig) -> AppResult<ExternalNotificationService> {
        let sms_config = SmsConfig {
            provider: config.sms_provider.provider.clone(),
            api_key: config.sms_provider.api_key.clone(),
            api_secret: config.sms_provider.api_secret.clone(),
            sender_id: config.sms_provider.sender_id.clone(),
            base_url: config.sms_provider.base_url.clone(),
        };

        let email_config = EmailConfig {
            smtp_host: config.email.smtp_host.clone(),
            smtp_port: config.email.smtp_port,
            username: config.email.username.clone(),
            password: config.email.password.clone(),
            from_email: config.email.from_email.clone(),
            from_name: config.email.from_name.clone(),
        };

        let whatsapp_config = WhatsAppConfig {
            access_token: config.whatsapp.access_token.clone(),
            phone_number_id: config.whatsapp.phone_number_id.clone(),
            business_account_id: config.whatsapp.business_account_id.clone(),
            webhook_verify_token: config.whatsapp.webhook_verify_token.clone(),
        };

        Ok(ExternalNotificationService::new(
            sms_config,
            email_config,
            whatsapp_config,
        ))
    }

    /// Build health monitoring service
    async fn build_health_monitoring(
        database_pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
        cache_service: Arc<dyn CacheService>,
        config: &AppConfig,
    ) -> AppResult<HealthMonitoringService> {
        let mut monitoring = HealthMonitoringService::new(
            config.version.clone(),
            config.environment.clone(),
        );

        // Add health checks
        monitoring.add_check(Arc::new(DatabaseHealthChecker::new(database_pool)));
        monitoring.add_check(Arc::new(CacheHealthChecker::new(cache_service)));

        // Add external API health checks
        if !config.external_apis.disaster_api_endpoints.is_empty() {
            for endpoint in &config.external_apis.disaster_api_endpoints {
                let health_check = crate::infrastructure::monitoring::health::ExternalApiHealthCheck::new(
                    "disaster_api".to_string(),
                    endpoint.clone(),
                    std::time::Duration::from_secs(10),
                );
                monitoring.add_check(Arc::new(health_check));
            }
        }

        Ok(monitoring)
    }

    // Placeholder implementations - these would be replaced with actual implementations
    fn create_placeholder_disaster_repository() -> Arc<dyn DisasterRepository> {
        use crate::domain::ports::repositories::DisasterRepository;
        use async_trait::async_trait;
        use crate::domain::entities::Disaster;
        use crate::domain::value_objects::*;

        struct PlaceholderDisasterRepository;

        #[async_trait]
        impl DisasterRepository for PlaceholderDisasterRepository {
            async fn find_by_id(&self, _id: &DisasterId) -> AppResult<Option<Disaster>> {
                Ok(None)
            }

            async fn save(&self, disaster: &Disaster) -> AppResult<Disaster> {
                Ok(disaster.clone())
            }

            async fn delete(&self, _id: &DisasterId) -> AppResult<bool> {
                Ok(false)
            }

            async fn find_nearby(
                &self,
                _center: &Coordinates,
                _radius_km: f64,
                _status_filter: Option<Vec<String>>,
                _severity_filter: Option<Vec<DisasterSeverity>>,
                _limit: Option<u32>,
            ) -> AppResult<Vec<Disaster>> {
                Ok(vec![])
            }
        }

        Arc::new(PlaceholderDisasterRepository)
    }

    fn create_placeholder_notification_repository() -> Arc<dyn NotificationRepository> {
        use crate::domain::ports::repositories::NotificationRepository;
        use async_trait::async_trait;
        use crate::domain::entities::Notification;
        use crate::domain::value_objects::*;

        struct PlaceholderNotificationRepository;

        #[async_trait]
        impl NotificationRepository for PlaceholderNotificationRepository {
            async fn find_by_id(&self, _id: &NotificationId) -> AppResult<Option<Notification>> {
                Ok(None)
            }

            async fn save(&self, notification: &Notification) -> AppResult<Notification> {
                Ok(notification.clone())
            }

            async fn delete(&self, _id: &NotificationId) -> AppResult<bool> {
                Ok(false)
            }

            async fn find_by_user(&self, _user_id: &UserId, _limit: Option<u32>) -> AppResult<Vec<Notification>> {
                Ok(vec![])
            }
        }

        Arc::new(PlaceholderNotificationRepository)
    }

    fn create_placeholder_event_publisher() -> Arc<dyn crate::domain::events::EventPublisher> {
        use crate::domain::events::{EventPublisher, DomainEvent};
        use async_trait::async_trait;

        struct PlaceholderEventPublisher;

        #[async_trait]
        impl EventPublisher for PlaceholderEventPublisher {
            async fn publish(&self, _event: &dyn DomainEvent) -> AppResult<()> {
                Ok(())
            }

            async fn publish_batch(&self, _events: &[&dyn DomainEvent]) -> AppResult<()> {
                Ok(())
            }
        }

        Arc::new(PlaceholderEventPublisher)
    }

    fn create_placeholder_geo_service() -> Arc<dyn GeoService> {
        use async_trait::async_trait;

        struct PlaceholderGeoService;

        #[async_trait]
        impl GeoService for PlaceholderGeoService {
            async fn calculate_distance(
                &self,
                _from: &Coordinates,
                _to: &Coordinates,
            ) -> AppResult<f64> {
                Ok(0.0)
            }

            async fn geocode_address(&self, _address: &str) -> AppResult<Coordinates> {
                Ok(Coordinates::new(-6.2088, 106.8456)?) // Jakarta coordinates
            }

            async fn reverse_geocode(&self, _coordinates: &Coordinates) -> AppResult<String> {
                Ok("Jakarta, Indonesia".to_string())
            }
        }

        Arc::new(PlaceholderGeoService)
    }
}

/// Extension trait for Actix Web data
impl AppContainer {
    /// Convert to Actix Web data
    pub fn into_web_data(self) -> web::Data<AppContainer> {
        web::Data::new(self)
    }
}
