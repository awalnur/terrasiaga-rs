/// Enhanced Application Container with PASETO integration
/// Provides comprehensive dependency injection for all Terra Siaga components

use std::sync::Arc;
use std::env;
use actix_web::web;
use futures_util::TryFutureExt;
use diesel::r2d2::{Pool, ConnectionManager};
use diesel::PgConnection;

use crate::config::AppConfig;
use crate::infrastructure::{
    cache::{CacheService, CacheFactory, CacheConfig},
    monitoring::health::HealthChecker,
    external_services::{
        notification_service::ExternalNotificationService,
        SmsConfig, EmailConfig, WhatsAppConfig,
        EmailProvider, SmsProvider, WhatsAppProvider,
    },
    security::paseto_service::{PasetoSecurityService, PasetoConfig},
    repository::user_repository::PostgresUserRepository,
    repository::notification_repository::PostgresNotificationRepository,
    database::DatabaseService,
};
use crate::domain::{
    ports::{
        repositories::{UserRepository, DisasterRepository, NotificationRepository},
        services::{NotificationService, GeolocationService, AuthService},
    },
    // removed value_objects::Coordinates import to avoid type mismatch
};
use crate::shared::{AppResult, AppError};
use crate::application::use_cases::*;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::monitoring::HealthMonitoringService;

/// Application container holding all dependencies
pub struct AppContainer {
    // Core services
    pub paseto_service: Arc<PasetoSecurityService>,
    pub auth_service: Arc<dyn AuthService>,
    pub cache_service: Arc<dyn CacheService>,
    pub notification_service: Arc<dyn NotificationService>,
    pub health_monitoring: HealthMonitoringService,

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

    // Database pool for direct access if needed
    pub database_pool: Option<Arc<DatabaseService>>,

    // Configuration
    pub config: AppConfig,
}

impl AppContainer {
    /// Build the complete application container
    pub async fn build(config: &AppConfig) -> AppResult<Self> {
        tracing::info!("Building application container...");

        // Build database connection pool
        let database_pool = match DatabaseService::new(config.database.clone()).await {
            Ok(pool) => Some(Arc::new(pool)),
            Err(e) => {
                tracing::warn!("Failed to create database pool: {}", e);
                None
            }
        };

        // Build cache service
        let cache_service = Self::build_cache_service(config).await?;

        // Build PASETO security service
        let paseto_config = Self::build_paseto_config(config)?;
        let paseto_service = PasetoSecurityService::new(
            paseto_config,
            cache_service.clone(),
        )?;
        let paseto_service = Arc::new(paseto_service);

        // Build JWT-based auth service for password operations and legacy flows
        let jwt_auth_service: Arc<dyn crate::domain::ports::services::AuthService> = Arc::new(
            crate::infrastructure::security::service::ProductionSecurityService::new(
                crate::infrastructure::security::service::SecurityConfig::default(),
                cache_service.clone(),
            )?
        );

        // Build repositories
        let user_repository = if let Some(db_pool) = &database_pool {
            Arc::new(PostgresUserRepository::new(
                db_pool.pool().clone(),
                cache_service.clone(),
            )) as Arc<dyn UserRepository>
        } else {
            return Err(AppError::Integration("Database pool is required for UserRepository".to_string()));
        };
        let disaster_repository = Self::create_placeholder_disaster_repository();
        let notification_repository: Arc<dyn NotificationRepository> = if let Some(db_pool) = &database_pool {
            Arc::new(PostgresNotificationRepository::new(db_pool.pool().clone()))
        } else {
            return Err(AppError::Integration("Database pool is required for NotificationRepository".to_string()));
        };

        // Build external services
        let notification_service = Self::build_notification_service(config)?;

        // Build health monitoring service using helper
        let mut external_services = vec![];
        if let Ok(endpoints_csv) = env::var("DISASTER_API_ENDPOINTS") {
            for endpoint in endpoints_csv.split(',').filter(|s| !s.is_empty()) {
                external_services.push(crate::infrastructure::monitoring::health::ExternalServiceConfig {
                    name: "disaster_api".to_string(),
                    url: endpoint.to_string(),
                    timeout_seconds: 5,
                });
            }
        }
        let health_config = crate::infrastructure::monitoring::health::HealthConfig {
            enabled: true,
            check_interval_seconds: 30,
            timeout_seconds: 5,
            external_services,
        };
        let db_pool_arc = database_pool.as_ref().map(|db| Arc::new(db.pool().clone()));
        let health_monitoring = crate::infrastructure::monitoring::create_health_service(
            env!("CARGO_PKG_VERSION").to_string(),
            config.environment.clone(),
            db_pool_arc,
            Some(cache_service.clone()),
            health_config,
        ).await;

        // Build use cases
        let register_user_use_case = Arc::new(RegisterUserUseCase::new(
            user_repository.clone(),
            jwt_auth_service.clone(),
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
            paseto_service,
            auth_service: jwt_auth_service,
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
            database_pool,
            config: config.clone(),
        })
    }

    /// Build cache service based on configuration
    async fn build_cache_service(config: &AppConfig) -> AppResult<Arc<dyn CacheService>> {
        let cache_config = CacheConfig {
            redis_url: if config.is_production() {
                Some(config.redis.url.clone())
            } else {
                Some(config.redis.url.clone())
            },
            redis_pool_size: Some(10),
            memory_cache_capacity: if config.is_production() { 10000 } else { 1000 },
            default_ttl_seconds: if config.is_production() { 3600 } else { 600 },
            key_prefix: format!("terra_siaga:{}", config.environment),
        };

        CacheFactory::create_redis_cache(&cache_config).await
    }

    /// Build PASETO configuration
    fn build_paseto_config(config: &AppConfig) -> AppResult<PasetoConfig> {
        // Get security key from environment or use default
        let secret_key = env::var("TERRA_SIAGA_SECRET_KEY")
            .unwrap_or_else(|_| "default-secret-key-change-in-production".to_string());

        let paseto_config = if config.is_production() {
            PasetoConfig {
                local_key: secret_key.as_bytes()[..32.min(secret_key.len())].to_vec(),
                public_key: vec![], // Would be loaded from secure storage
                private_key: vec![], // Would be loaded from secure storage
                token_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .unwrap_or(24),
                session_timeout_hours: env::var("SESSION_TIMEOUT_HOURS")
                    .unwrap_or_else(|_| "8".to_string())
                    .parse()
                    .unwrap_or(8),
                elevated_session_minutes: 15,
                use_local_tokens: true, // Use encrypted tokens in production
                access_token_ttl_minutes: env::var("ACCESS_TOKEN_TTL_MINUTES")
                    .unwrap_or_else(|_| "15".to_string())
                    .parse()
                    .unwrap_or(15),
                refresh_token_ttl_days: env::var("REFRESH_TOKEN_TTL_DAYS")
                    .unwrap_or_else(|_| "7".to_string())
                    .parse()
                    .unwrap_or(7),
            }
        } else {
            PasetoConfig::default()
        };

        Ok(paseto_config)
    }

    /// Build notification service with all providers
    fn build_notification_service(config: &AppConfig) -> AppResult<Arc<dyn NotificationService>> {
        // Use environment variables or defaults for notification configs
        let sms_provider = match env::var("SMS_PROVIDER").unwrap_or_else(|_| "twilio".to_string()).to_lowercase().as_str() {
            "twilio" => SmsProvider::Twilio,
            "vonage" => SmsProvider::Vonage,
            "aws_sns" | "aws-sns" => SmsProvider::AWS_SNS,
            _ => SmsProvider::Twilio,
        };
        let sms_config = SmsConfig {
            provider: sms_provider,
            api_key: env::var("SMS_API_KEY").unwrap_or_default(),
            from_number: env::var("SMS_FROM_NUMBER").ok(),
            timeout: std::time::Duration::from_secs(30),
            api_secret: env::var("SMS_API_SECRET").unwrap_or_default(),
        };

        let email_provider = match env::var("EMAIL_PROVIDER").unwrap_or_else(|_| "smtp".to_string()).to_lowercase().as_str() {
            "sendgrid" => EmailProvider::SendGrid,
            "mailgun" => EmailProvider::Mailgun,
            "smtp" => {
                let host = env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
                let port = env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string()).parse().unwrap_or(587);
                let username = env::var("SMTP_USERNAME").unwrap_or_default();
                let password = env::var("SMTP_PASSWORD").unwrap_or_default();
                EmailProvider::SMTP { host, port, username, password }
            }
            _ => EmailProvider::SendGrid,
        };
        let email_config = EmailConfig {
            provider: email_provider,
            api_key: env::var("EMAIL_API_KEY").unwrap_or_default(),
            from_email: env::var("EMAIL_FROM_ADDRESS").unwrap_or_else(|_| "noreply@terrasiaga.id".to_string()),
            from_name: env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "Terra Siaga".to_string()),
            timeout: std::time::Duration::from_secs(30),
        };

        let whatsapp_provider = match env::var("WHATSAPP_PROVIDER").unwrap_or_else(|_| "meta".to_string()).to_lowercase().as_str() {
            "meta" => WhatsAppProvider::Meta,
            "twilio" => WhatsAppProvider::Twilio,
            _ => WhatsAppProvider::Meta,
        };
        let whatsapp_config = WhatsAppConfig {
            provider: whatsapp_provider,
            api_key: env::var("WHATSAPP_API_KEY").unwrap_or_default(),
            phone_number_id: env::var("WHATSAPP_PHONE_NUMBER_ID").unwrap_or_default(),
            timeout: std::time::Duration::from_secs(30),
        };

        let service = ExternalNotificationService::new(
            sms_config,
            email_config,
            whatsapp_config,
        );

        Ok(Arc::new(service))
    }

    // Placeholder implementations - these would be replaced with actual implementations
    fn create_placeholder_disaster_repository() -> Arc<dyn DisasterRepository> {
        use crate::domain::ports::repositories::DisasterRepository;
        use async_trait::async_trait;
        use crate::domain::entities::disaster::Disaster;
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

            async fn update(&self, disaster: &Disaster) -> AppResult<Disaster> {
                Ok(disaster.clone())
            }

            async fn delete(&self, _id: &DisasterId) -> AppResult<bool> {
                Ok(false)
            }

            async fn find_all(&self) -> AppResult<Vec<Disaster>> {
                Ok(vec![])
            }

            async fn find_by_status(&self, _status: crate::domain::entities::disaster::DisasterStatus) -> AppResult<Vec<Disaster>> {
                Ok(vec![])
            }

            async fn find_by_severity(&self, _severity: crate::domain::entities::disaster::DisasterSeverity) -> AppResult<Vec<Disaster>> {
                Ok(vec![])
            }

            async fn find_by_reporter(&self, _reporter_id: &UserId) -> AppResult<Vec<Disaster>> {
                Ok(vec![])
            }

            async fn find_nearby(&self, _lat: f64, _lng: f64, _radius_km: f64) -> AppResult<Vec<Disaster>> {
                Ok(vec![])
            }

            async fn find_active(&self) -> AppResult<Vec<Disaster>> {
                Ok(vec![])
            }

            async fn update_status(&self, _id: &DisasterId, _status: crate::domain::entities::disaster::DisasterStatus) -> AppResult<bool> {
                Ok(true)
            }

            async fn assign_responder(&self, _disaster_id: &DisasterId, _responder_id: &UserId) -> AppResult<bool> {
                Ok(true)
            }

            async fn find_by_location(&self, _location_id: &LocationId) -> AppResult<Vec<Disaster>> {
                Ok(vec![])
            }
        }

        Arc::new(PlaceholderDisasterRepository)
    }

    fn create_placeholder_notification_repository() -> Arc<dyn NotificationRepository> {
        // removed: now using real PostgresNotificationRepository in build()
        use crate::domain::ports::repositories::NotificationRepository;
        use crate::domain::entities::notification::{Notification, NotificationStatus, NotificationChannel};
        use crate::domain::value_objects::*;
        use async_trait::async_trait;

        struct PlaceholderNotificationRepository;

        #[async_trait]
        impl NotificationRepository for PlaceholderNotificationRepository {
            async fn find_by_id(&self, _id: &NotificationId) -> AppResult<Option<Notification>> { Ok(None) }
            async fn save(&self, notification: &Notification) -> AppResult<Notification> { Ok(notification.clone()) }
            async fn update(&self, notification: &Notification) -> AppResult<Notification> { Ok(notification.clone()) }
            async fn delete(&self, _id: &NotificationId) -> AppResult<bool> { Ok(false) }
            async fn find_all(&self) -> AppResult<Vec<Notification>> { Ok(vec![]) }
            async fn find_by_recipient(&self, _recipient_id: &UserId) -> AppResult<Vec<Notification>> { Ok(vec![]) }
            async fn find_unread(&self, _recipient_id: &UserId) -> AppResult<Vec<Notification>> { Ok(vec![]) }
            async fn mark_as_read(&self, _id: &NotificationId) -> AppResult<bool> { Ok(true) }
            async fn mark_all_as_read(&self, _recipient_id: &UserId) -> AppResult<u64> { Ok(0) }
            async fn delete_old_notifications(&self, _days: u32) -> AppResult<u64> { Ok(0) }
            async fn count_unread(&self, _recipient_id: &UserId) -> AppResult<u64> { Ok(0) }
            async fn find_by_status(&self, _status: NotificationStatus) -> AppResult<Vec<Notification>> { Ok(vec![]) }
            async fn save_notification(&self, notification: &Notification) -> AppResult<Notification> { Ok(notification.clone()) }
            async fn find_by_user(&self, _user_id: &UserId, _limit: Option<u32>) -> AppResult<Vec<Notification>> { Ok(vec![]) }
            async fn find_unread_by_recipient(&self, _recipient_id: UserId) -> AppResult<Vec<Notification>> { Ok(vec![]) }

            async fn find_by_channel(&self, channel: NotificationChannel) -> AppResult<Vec<Notification>> {
                todo!()
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

    fn create_placeholder_geo_service() -> Arc<dyn GeolocationService> {
        use async_trait::async_trait;
        use crate::shared::types::Coordinates;
        use crate::shared::AppResult;

        struct PlaceholderGeoService;

        #[async_trait]
        impl GeolocationService for PlaceholderGeoService {
            async fn calculate_distance(&self, _from: &Coordinates, _to: &Coordinates) -> AppResult<f64> {
                Ok(0.0)
            }

            async fn geocode(&self, _address: &str) -> AppResult<Coordinates> {
                Ok(Coordinates::new(-6.2088, 106.8456).unwrap())
            }

            async fn reverse_geocode(&self, _coordinates: &Coordinates) -> AppResult<String> {
                Ok("Jakarta, Indonesia".to_string())
            }

            async fn get_nearby_locations(&self, _center: &Coordinates, _radius_km: f64) -> AppResult<Vec<Coordinates>> {
                Ok(Vec::new())
            }
        }

        Arc::new(PlaceholderGeoService)
    }

    // Convenience methods for accessing services
    pub fn database_pool(&self) -> Option<&Arc<DatabaseService>> { self.database_pool.as_ref() }
    pub fn cache_service(&self) -> &Arc<dyn CacheService> { &self.cache_service }
    pub fn paseto_service(&self) -> &Arc<PasetoSecurityService> { &self.paseto_service }
}
