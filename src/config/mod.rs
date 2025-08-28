/// Configuration management for Terra Siaga
/// Centralized configuration with environment-based settings and validation

use serde::{Deserialize, Serialize};
use std::env;
use chrono::Duration;
use env_logger::Env;
use crate::parse_duration_seconds;
use crate::shared::{AppResult, AppError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app_name: String,
    pub app_version: String,
    pub app_description: String,
    pub environment: String,
    pub debug: String,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub redis: RedisConfig,
    pub external_apis: ExternalApisConfig,
    pub logging: LoggingConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub keep_alive: u64,
    pub client_timeout: Duration,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: std::time::Duration,
    pub idle_timeout: std::time::Duration,
    pub max_lifetime: std::time::Duration,
    pub enable_logging: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub refresh_token_expiry_days: i64,
    pub password_hash_cost: u32,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub default_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalApisConfig {
    pub google_maps_key: String,
    pub weather_api_key: String,
    pub email_service_key: String,
    pub sms_service_key: String,
    pub whatsapp_token: String,
    pub whatsapp_phone_number_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_path: Option<String>,
    pub max_file_size: u64,
    pub max_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub enable_real_time_notifications: bool,
    pub enable_weather_integration: bool,
    pub enable_analytics: bool,
    pub enable_ml_predictions: bool,
    pub enable_mobile_app_support: bool,
}

impl AppConfig {
    /// Load configuration from environment variables with defaults
    pub fn from_env() -> AppResult<Self> {
        let config = Self {
            app_name: env::var("APP_NAME").unwrap_or_else(|_| "Terra Siaga".to_string()),
            app_version: env::var("APP_VERSION").unwrap_or_else(|_| "1.0.1".to_string()),
            environment: env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
            app_description: env::var("APP_DESCRIPTION")
                .unwrap_or_else(|_| "Disaster management platform".to_string()),
            debug: env::var("DEBUG").unwrap_or_else(|_| "Disaster management platform".to_string()),
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .map_err(|_| AppError::Configuration("Invalid PORT value".to_string()))?,
                workers: env::var("WORKERS")
                    .ok()
                    .and_then(|w| w.parse().ok()),
                keep_alive: env::var("KEEP_ALIVE")
                    .unwrap_or_else(|_| "75".to_string())
                    .parse()
                    .unwrap_or(75),
                client_timeout: Duration::milliseconds(
                    env::var("CLIENT_TIMEOUT")
                        .unwrap_or_else(|_| "5000".to_string())
                        .parse()
                        .unwrap_or(5000)
                ),
                cors_origins: env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:3000,http://localhost:8080".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .map_err(|_| AppError::Configuration("DATABASE_URL is required".to_string()))?,
                max_connections: env::var("DB_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                min_connections: env::var("DB_MIN_CONNECTIONS")
                    .unwrap_or_else(|_| "1".to_string())
                    .parse()
                    .unwrap_or(1),
                connection_timeout: parse_duration_seconds("DB_CONNECTION_TIMEOUT",30),
                idle_timeout: parse_duration_seconds("DB_IDLE_TIMEOUT", 600),
                max_lifetime: parse_duration_seconds("DB_MAX_LIFETIME",1800),
                enable_logging: false,

            },
            auth: AuthConfig {
                jwt_secret: env::var("JWT_SECRET")
                    .map_err(|_| AppError::Configuration("JWT_SECRET is required".to_string()))?,
                jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .unwrap_or(24),
                refresh_token_expiry_days: env::var("REFRESH_TOKEN_EXPIRY_DAYS")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
                password_hash_cost: env::var("PASSWORD_HASH_COST")
                    .unwrap_or_else(|_| "12".to_string())
                    .parse()
                    .unwrap_or(12),
                max_login_attempts: env::var("MAX_LOGIN_ATTEMPTS")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
                lockout_duration_minutes: env::var("LOCKOUT_DURATION_MINUTES")
                    .unwrap_or_else(|_| "15".to_string())
                    .parse()
                    .unwrap_or(15),
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                max_connections: env::var("REDIS_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                connection_timeout: env::var("REDIS_CONNECTION_TIMEOUT")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
                default_ttl: env::var("REDIS_DEFAULT_TTL")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()
                    .unwrap_or(3600),
            },
            external_apis: ExternalApisConfig {
                google_maps_key: env::var("GOOGLE_MAPS_API_KEY")
                    .unwrap_or_else(|_| "".to_string()),
                weather_api_key: env::var("WEATHER_API_KEY")
                    .unwrap_or_else(|_| "".to_string()),
                email_service_key: env::var("EMAIL_SERVICE_KEY")
                    .unwrap_or_else(|_| "".to_string()),
                sms_service_key: env::var("SMS_SERVICE_KEY")
                    .unwrap_or_else(|_| "".to_string()),
                whatsapp_token: env::var("WHATSAPP_TOKEN")
                    .unwrap_or_else(|_| "".to_string()),
                whatsapp_phone_number_id: env::var("WHATSAPP_PHONE_NUMBER_ID")
                    .unwrap_or_else(|_| "".to_string()),
            },
            logging: LoggingConfig {
                level: env::var("LOG_LEVEL")
                    .unwrap_or_else(|_| "info".to_string()),
                format: env::var("LOG_FORMAT")
                    .unwrap_or_else(|_| "json".to_string()),
                file_path: env::var("LOG_FILE_PATH").ok(),
                max_file_size: env::var("LOG_MAX_FILE_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                max_files: env::var("LOG_MAX_FILES")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
            },
            features: FeatureFlags {
                enable_real_time_notifications: env::var("ENABLE_REAL_TIME_NOTIFICATIONS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                enable_weather_integration: env::var("ENABLE_WEATHER_INTEGRATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                enable_analytics: env::var("ENABLE_ANALYTICS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                enable_ml_predictions: env::var("ENABLE_ML_PREDICTIONS")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
                enable_mobile_app_support: env::var("ENABLE_MOBILE_APP_SUPPORT")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
        };

        // Validate configuration
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values
    fn validate(&self) -> AppResult<()> {
        // Validate server config
        if self.server.port == 0 {
            return Err(AppError::Configuration("Server port cannot be 0".to_string()));
        }

        if self.server.host.is_empty() {
            return Err(AppError::Configuration("Server host cannot be empty".to_string()));
        }

        // Validate database URL
        if self.database.url.is_empty() {
            return Err(AppError::Configuration("Database URL cannot be empty".to_string()));
        }

        // Validate database connection pool settings
        if self.database.max_connections == 0 {
            return Err(AppError::Configuration("Database max_connections must be greater than 0".to_string()));
        }

        if self.database.min_connections > self.database.max_connections {
            return Err(AppError::Configuration("Database min_connections cannot be greater than max_connections".to_string()));
        }

        // Validate JWT secret
        if self.auth.jwt_secret.len() < 32 {
            return Err(AppError::Configuration("JWT secret must be at least 32 characters".to_string()));
        }

        // Validate JWT expiry
        if self.auth.jwt_expiry_hours <= 0 {
            return Err(AppError::Configuration("JWT expiry hours must be positive".to_string()));
        }

        if self.auth.refresh_token_expiry_days <= 0 {
            return Err(AppError::Configuration("Refresh token expiry days must be positive".to_string()));
        }

        // Validate hash cost
        if self.auth.password_hash_cost < 4 || self.auth.password_hash_cost > 31 {
            return Err(AppError::Configuration("Password hash cost must be between 4 and 31".to_string()));
        }

        // Validate auth limits
        if self.auth.max_login_attempts == 0 {
            return Err(AppError::Configuration("Max login attempts must be greater than 0".to_string()));
        }

        // Validate Redis config
        if !self.redis.url.is_empty() && self.redis.max_connections == 0 {
            return Err(AppError::Configuration("Redis max_connections must be greater than 0 when Redis is enabled".to_string()));
        }

        // Validate logging config
        if !["trace", "debug", "info", "warn", "error"].contains(&self.logging.level.as_str()) {
            return Err(AppError::Configuration("Invalid log level. Must be one of: trace, debug, info, warn, error".to_string()));
        }

        if !["json", "pretty", "compact"].contains(&self.logging.format.as_str()) {
            return Err(AppError::Configuration("Invalid log format. Must be one of: json, pretty, compact".to_string()));
        }

        // Validate external APIs (warn if missing in production)
        if self.is_production() {
            if self.external_apis.google_maps_key.is_empty() {
                tracing::warn!("Google Maps API key is not configured in production");
            }
            if self.external_apis.weather_api_key.is_empty() {
                tracing::warn!("Weather API key is not configured in production");
            }
            if self.external_apis.email_service_key.is_empty() {
                tracing::warn!("Email service key is not configured in production");
            }
        }

        Ok(())
    }

    /// Get environment (development, staging, production)
    pub fn environment(&self) -> String {
        env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
    }

    /// Check if running in development mode
    pub fn is_development(&self) -> bool {
        self.environment() == "development"
    }

    /// Check if running in production mode
    pub fn is_production(&self) -> bool {
        self.environment() == "production"
    }
}
