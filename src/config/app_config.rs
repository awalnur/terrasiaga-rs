/// Configuration management for Terra Siaga
/// Handles environment-specific settings, secrets, and feature flags

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

use crate::shared::{AppResult, AppError};

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
    pub ssl_mode: String,
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub database: u8,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub command_timeout: u64,
}

/// SMS provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsProviderConfig {
    pub provider: String, // "twilio", "nexmo", "local"
    pub api_key: String,
    pub api_secret: String,
    pub sender_id: String,
    pub base_url: String,
    pub webhook_url: Option<String>,
}

/// Email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
    pub use_tls: bool,
    pub use_ssl: bool,
}

/// Push notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotificationConfig {
    pub firebase_server_key: String,
    pub firebase_sender_id: String,
    pub apns_key_id: Option<String>,
    pub apns_team_id: Option<String>,
    pub apns_bundle_id: Option<String>,
}

/// WhatsApp Business API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConfig {
    pub access_token: String,
    pub phone_number_id: String,
    pub business_account_id: String,
    pub webhook_verify_token: String,
    pub webhook_url: String,
}

/// External API integrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalApiConfig {
    pub bmkg_api_key: Option<String>, // Indonesian weather service
    pub google_maps_api_key: Option<String>,
    pub mapbox_api_key: Option<String>,
    pub opencage_api_key: Option<String>, // Geocoding
    pub disaster_api_endpoints: Vec<String>,
}

/// Monitoring and observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub enable_tracing: bool,
    pub enable_logging: bool,
    pub log_level: String,
    pub metrics_endpoint: String,
    pub jaeger_endpoint: Option<String>,
    pub prometheus_endpoint: Option<String>,
    pub sentry_dsn: Option<String>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub session_timeout_hours: u64,
    pub max_failed_attempts: u32,
    pub lockout_duration_minutes: u64,
    pub password_min_length: usize,
    pub require_special_chars: bool,
    pub cors_origins: Vec<String>,
    pub rate_limit_requests_per_minute: u32,
}

/// Feature flags configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub enable_sms_notifications: bool,
    pub enable_email_notifications: bool,
    pub enable_push_notifications: bool,
    pub enable_whatsapp_notifications: bool,
    pub enable_real_time_tracking: bool,
    pub enable_advanced_analytics: bool,
    pub enable_emergency_broadcast: bool,
    pub enable_volunteer_matching: bool,
    pub enable_predictive_alerts: bool,
    pub maintenance_mode: bool,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<u32>,
    pub keep_alive: u64,
    pub client_timeout: u64,
    pub client_shutdown: u64,
    pub max_connections: u32,
    pub max_connection_rate: u32,
}

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub environment: String, // "development", "staging", "production"
    pub service_name: String,
    pub version: String,
    pub debug: bool,
    
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
    pub feature_flags: FeatureFlags,
    
    // External services
    pub sms_provider: SmsProviderConfig,
    pub email: EmailConfig,
    pub push_notifications: PushNotificationConfig,
    pub whatsapp: WhatsAppConfig,
    pub external_apis: ExternalApiConfig,
}

impl AppConfig {
    /// Load configuration from environment variables and config files
    pub fn load() -> AppResult<Self> {
        // Start with default configuration
        let mut config = Self::default();
        
        // Override with environment-specific config file
        let env = env::var("TERRA_ENV").unwrap_or_else(|_| "development".to_string());
        config.environment = env.clone();
        
        let config_file = format!("config/{}.toml", env);
        if Path::new(&config_file).exists() {
            let file_content = fs::read_to_string(&config_file)
                .map_err(|e| AppError::Configuration(format!("Failed to read config file: {}", e)))?;
            
            let file_config: AppConfig = toml::from_str(&file_content)
                .map_err(|e| AppError::Configuration(format!("Failed to parse config file: {}", e)))?;
            
            config = file_config;
        }
        
        // Override with environment variables
        config.load_from_env()?;
        
        // Validate configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Load configuration from environment variables
    fn load_from_env(&mut self) -> AppResult<()> {
        // Server configuration
        if let Ok(host) = env::var("SERVER_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            self.server.port = port.parse()
                .map_err(|_| AppError::Configuration("Invalid SERVER_PORT".to_string()))?;
        }
        
        // Database configuration
        if let Ok(db_url) = env::var("DATABASE_URL") {
            self.parse_database_url(&db_url)?;
        }
        
        // Redis configuration
        if let Ok(redis_url) = env::var("REDIS_URL") {
            self.parse_redis_url(&redis_url)?;
        }
        
        // Security configuration
        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            self.security.jwt_secret = jwt_secret;
        }
        
        // External service configurations
        if let Ok(twilio_key) = env::var("TWILIO_API_KEY") {
            self.sms_provider.api_key = twilio_key;
        }
        if let Ok(twilio_secret) = env::var("TWILIO_API_SECRET") {
            self.sms_provider.api_secret = twilio_secret;
        }
        
        if let Ok(smtp_host) = env::var("SMTP_HOST") {
            self.email.smtp_host = smtp_host;
        }
        if let Ok(smtp_user) = env::var("SMTP_USERNAME") {
            self.email.username = smtp_user;
        }
        if let Ok(smtp_pass) = env::var("SMTP_PASSWORD") {
            self.email.password = smtp_pass;
        }
        
        if let Ok(firebase_key) = env::var("FIREBASE_SERVER_KEY") {
            self.push_notifications.firebase_server_key = firebase_key;
        }
        
        if let Ok(whatsapp_token) = env::var("WHATSAPP_ACCESS_TOKEN") {
            self.whatsapp.access_token = whatsapp_token;
        }
        
        // Monitoring configuration
        if let Ok(sentry_dsn) = env::var("SENTRY_DSN") {
            self.monitoring.sentry_dsn = Some(sentry_dsn);
        }
        
        // Feature flags from environment
        if let Ok(maintenance) = env::var("MAINTENANCE_MODE") {
            self.feature_flags.maintenance_mode = maintenance.parse().unwrap_or(false);
        }
        
        Ok(())
    }
    
    /// Parse database URL
    fn parse_database_url(&mut self, url: &str) -> AppResult<()> {
        // Simple URL parsing for PostgreSQL
        // postgres://username:password@host:port/database
        if let Some(captures) = regex::Regex::new(r"postgres://([^:]+):([^@]+)@([^:]+):(\d+)/(.+)")
            .unwrap()
            .captures(url) {
            
            self.database.username = captures[1].to_string();
            self.database.password = captures[2].to_string();
            self.database.host = captures[3].to_string();
            self.database.port = captures[4].parse()
                .map_err(|_| AppError::Configuration("Invalid database port".to_string()))?;
            self.database.database_name = captures[5].to_string();
        }
        
        Ok(())
    }
    
    /// Parse Redis URL
    fn parse_redis_url(&mut self, url: &str) -> AppResult<()> {
        // Simple URL parsing for Redis
        // redis://password@host:port/database
        if let Some(captures) = regex::Regex::new(r"redis://(?:([^@]+)@)?([^:]+):(\d+)(?:/(\d+))?")
            .unwrap()
            .captures(url) {
            
            if let Some(password) = captures.get(1) {
                self.redis.password = Some(password.as_str().to_string());
            }
            self.redis.host = captures[2].to_string();
            self.redis.port = captures[3].parse()
                .map_err(|_| AppError::Configuration("Invalid Redis port".to_string()))?;
            if let Some(db) = captures.get(4) {
                self.redis.database = db.as_str().parse()
                    .map_err(|_| AppError::Configuration("Invalid Redis database".to_string()))?;
            }
        }
        
        Ok(())
    }
    
    /// Validate configuration
    fn validate(&self) -> AppResult<()> {
        // Validate required fields
        if self.security.jwt_secret == "change-me-in-production" && self.environment == "production" {
            return Err(AppError::Configuration("JWT secret must be changed in production".to_string()));
        }
        
        if self.database.password.is_empty() {
            return Err(AppError::Configuration("Database password is required".to_string()));
        }
        
        // Validate port ranges
        if self.server.port == 0 || self.server.port > 65535 {
            return Err(AppError::Configuration("Invalid server port".to_string()));
        }
        
        // Validate environment
        if !["development", "staging", "production"].contains(&self.environment.as_str()) {
            return Err(AppError::Configuration("Invalid environment".to_string()));
        }
        
        Ok(())
    }
    
    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
    
    /// Check if running in development
    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }
    
    /// Get database connection string
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database_name
        )
    }
    
    /// Get Redis connection string
    pub fn redis_url(&self) -> String {
        match &self.redis.password {
            Some(password) => format!(
                "redis://{}@{}:{}/{}",
                password,
                self.redis.host,
                self.redis.port,
                self.redis.database
            ),
            None => format!(
                "redis://{}:{}/{}",
                self.redis.host,
                self.redis.port,
                self.redis.database
            ),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            service_name: "terra-siaga".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            debug: true,
            
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: None,
                keep_alive: 75,
                client_timeout: 5000,
                client_shutdown: 5000,
                max_connections: 25000,
                max_connection_rate: 256,
            },
            
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "terra_siaga".to_string(),
                password: "password".to_string(),
                database_name: "terra_siaga".to_string(),
                max_connections: 10,
                min_connections: 1,
                connection_timeout: 30,
                idle_timeout: 600,
                ssl_mode: "prefer".to_string(),
            },
            
            redis: RedisConfig {
                host: "localhost".to_string(),
                port: 6379,
                password: None,
                database: 0,
                max_connections: 10,
                connection_timeout: 30,
                command_timeout: 30,
            },
            
            security: SecurityConfig {
                jwt_secret: "change-me-in-production".to_string(),
                jwt_expiration_hours: 24,
                session_timeout_hours: 72,
                max_failed_attempts: 5,
                lockout_duration_minutes: 30,
                password_min_length: 8,
                require_special_chars: true,
                cors_origins: vec!["http://localhost:3000".to_string()],
                rate_limit_requests_per_minute: 60,
            },
            
            monitoring: MonitoringConfig {
                enable_metrics: true,
                enable_tracing: true,
                enable_logging: true,
                log_level: "info".to_string(),
                metrics_endpoint: "/metrics".to_string(),
                jaeger_endpoint: None,
                prometheus_endpoint: None,
                sentry_dsn: None,
            },
            
            feature_flags: FeatureFlags {
                enable_sms_notifications: true,
                enable_email_notifications: true,
                enable_push_notifications: true,
                enable_whatsapp_notifications: false,
                enable_real_time_tracking: true,
                enable_advanced_analytics: false,
                enable_emergency_broadcast: true,
                enable_volunteer_matching: true,
                enable_predictive_alerts: false,
                maintenance_mode: false,
            },
            
            sms_provider: SmsProviderConfig {
                provider: "twilio".to_string(),
                api_key: "".to_string(),
                api_secret: "".to_string(),
                sender_id: "TerraSiaga".to_string(),
                base_url: "https://api.twilio.com".to_string(),
                webhook_url: None,
            },
            
            email: EmailConfig {
                smtp_host: "smtp.gmail.com".to_string(),
                smtp_port: 587,
                username: "".to_string(),
                password: "".to_string(),
                from_email: "noreply@terrasiaga.id".to_string(),
                from_name: "Terra Siaga".to_string(),
                use_tls: true,
                use_ssl: false,
            },
            
            push_notifications: PushNotificationConfig {
                firebase_server_key: "".to_string(),
                firebase_sender_id: "".to_string(),
                apns_key_id: None,
                apns_team_id: None,
                apns_bundle_id: None,
            },
            
            whatsapp: WhatsAppConfig {
                access_token: "".to_string(),
                phone_number_id: "".to_string(),
                business_account_id: "".to_string(),
                webhook_verify_token: "".to_string(),
                webhook_url: "".to_string(),
            },
            
            external_apis: ExternalApiConfig {
                bmkg_api_key: None,
                google_maps_api_key: None,
                mapbox_api_key: None,
                opencage_api_key: None,
                disaster_api_endpoints: vec![],
            },
        }
    }
}
