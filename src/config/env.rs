/// Environment configuration for Terra Siaga
/// Handles loading and accessing environment variables

use std::env;

#[derive(Debug, Clone)]
pub struct EnvConfig {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub redis_url: String,
    pub environment: Environment,

    // External API keys
    pub google_maps_key: Option<String>,
    pub weather_api_key: Option<String>,
    pub email_service_key: Option<String>,
    pub whatsapp_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Testing,
    Production,
}

impl Environment {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" | "prod" => Environment::Production,
            "testing" | "test" => Environment::Testing,
            _ => Environment::Development,
        }
    }
}

impl EnvConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        // Load .env file if it exists (for development)
        if let Err(_) = dotenv::dotenv() {
            // .env file not found, which is fine for production
        }

        let environment = Environment::from_str(
            &env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
        );

        // Required environment variables
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable is required".to_string())?;

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET environment variable is required".to_string())?;

        // Validate JWT secret length
        if jwt_secret.len() < 32 {
            return Err("JWT_SECRET must be at least 32 characters long".to_string());
        }

        // Optional environment variables with defaults
        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| "SERVER_PORT must be a valid port number".to_string())?;

        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        // Optional external API keys
        let google_maps_key = env::var("GOOGLE_MAPS_API_KEY").ok();
        let weather_api_key = env::var("WEATHER_API_KEY").ok();
        let email_service_key = env::var("EMAIL_SERVICE_KEY").ok();
        let whatsapp_token = env::var("WHATSAPP_TOKEN").ok();

        Ok(Self {
            database_url,
            server_host,
            server_port,
            jwt_secret,
            redis_url,
            environment,
            google_maps_key,
            weather_api_key,
            email_service_key,
            whatsapp_token,
        })
    }

    /// Check if running in development mode
    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }

    /// Check if running in production mode
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    /// Check if running in testing mode
    pub fn is_testing(&self) -> bool {
        self.environment == Environment::Testing
    }

    /// Get database connection pool size based on environment
    pub fn db_pool_size(&self) -> u32 {
        match self.environment {
            Environment::Production => 20,
            Environment::Testing => 5,
            Environment::Development => 10,
        }
    }

    /// Get log level based on environment
    pub fn log_level(&self) -> &str {
        match self.environment {
            Environment::Production => "info",
            Environment::Testing => "debug",
            Environment::Development => "debug",
        }
    }
}