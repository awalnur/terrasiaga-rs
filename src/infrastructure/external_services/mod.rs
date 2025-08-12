/// External services infrastructure with improved error handling and configuration
/// Provides integrations with third-party APIs and services

pub mod email;
pub mod sms;
pub mod whatsapp;
pub mod weather;
pub mod geolocation;
pub mod notification;
pub mod notification_service;

use crate::shared::error::{AppResult, AppError};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for external services
#[derive(Debug, Clone)]
pub struct ExternalServicesConfig {
    pub google_maps_api_key: String,
    pub weather_api_key: String,
    pub email_service_key: String,
    pub sms_service_key: String,
    pub whatsapp_token: String,
    pub whatsapp_phone_number_id: String,
    pub timeout: Duration,
}

impl ExternalServicesConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> AppResult<Self> {
        Ok(Self {
            google_maps_api_key: std::env::var("GOOGLE_MAPS_API_KEY")
                .unwrap_or_else(|_| "".to_string()),
            weather_api_key: std::env::var("WEATHER_API_KEY")
                .unwrap_or_else(|_| "".to_string()),
            email_service_key: std::env::var("EMAIL_SERVICE_KEY")
                .unwrap_or_else(|_| "".to_string()),
            sms_service_key: std::env::var("SMS_SERVICE_KEY")
                .unwrap_or_else(|_| "".to_string()),
            whatsapp_token: std::env::var("WHATSAPP_TOKEN")
                .unwrap_or_else(|_| "".to_string()),
            whatsapp_phone_number_id: std::env::var("WHATSAPP_PHONE_NUMBER_ID")
                .unwrap_or_else(|_| "".to_string()),
            timeout: Duration::from_secs(30),
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> AppResult<()> {
        if self.google_maps_api_key.is_empty() {
            tracing::warn!("Google Maps API key is not configured");
        }
        if self.weather_api_key.is_empty() {
            tracing::warn!("Weather API key is not configured");
        }
        if self.email_service_key.is_empty() {
            tracing::warn!("Email service key is not configured");
        }
        if self.sms_service_key.is_empty() {
            tracing::warn!("SMS service key is not configured");
        }
        if self.whatsapp_token.is_empty() {
            tracing::warn!("WhatsApp token is not configured");
        }
        Ok(())
    }
}

/// Detailed configuration for individual services (for future extensibility)
#[derive(Debug, Clone)]
pub struct DetailedExternalServicesConfig {
    pub email: Option<EmailConfig>,
    pub sms: Option<SmsConfig>,
    pub whatsapp: Option<WhatsAppConfig>,
    pub weather: Option<WeatherConfig>,
    pub geolocation: Option<GeolocationConfig>,
}

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub provider: EmailProvider,
    pub api_key: String,
    pub from_email: String,
    pub from_name: String,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub enum EmailProvider {
    SendGrid,
    Mailgun,
    SMTP { host: String, port: u16, username: String, password: String },
}

#[derive(Debug, Clone)]
pub struct SmsConfig {
    pub provider: SmsProvider,
    pub api_key: String,
    pub from_number: Option<String>,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub enum SmsProvider {
    Twilio,
    Vonage,
    AWS_SNS,
}

#[derive(Debug, Clone)]
pub struct WhatsAppConfig {
    pub provider: WhatsAppProvider,
    pub api_key: String,
    pub phone_number_id: String,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub enum WhatsAppProvider {
    Meta,
    Twilio,
}

#[derive(Debug, Clone)]
pub struct WeatherConfig {
    pub provider: WeatherProvider,
    pub api_key: String,
    pub timeout: Duration,
    pub cache_duration: Duration,
}

#[derive(Debug, Clone)]
pub enum WeatherProvider {
    OpenWeatherMap,
    AccuWeather,
    WeatherAPI,
}

#[derive(Debug, Clone)]
pub struct GeolocationConfig {
    pub provider: GeolocationProvider,
    pub api_key: Option<String>,
    pub timeout: Duration,
    pub cache_duration: Duration,
}

#[derive(Debug, Clone)]
pub enum GeolocationProvider {
    Google,
    Nominatim,
    MapBox,
}

/// External services manager with circuit breaker and retry logic
pub struct ExternalServicesManager {
    config: ExternalServicesConfig,
    http_client: reqwest::Client,
    circuit_breakers: std::collections::HashMap<String, CircuitBreaker>,
}

impl ExternalServicesManager {
    /// Create new external services manager
    pub fn new(config: ExternalServicesConfig) -> AppResult<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("terra-siaga/1.0")
            .build()
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        let mut circuit_breakers = std::collections::HashMap::new();

        // Initialize circuit breakers for each service
        circuit_breakers.insert("email".to_string(), CircuitBreaker::new(5, Duration::from_secs(60)));
        circuit_breakers.insert("sms".to_string(), CircuitBreaker::new(5, Duration::from_secs(60)));
        circuit_breakers.insert("whatsapp".to_string(), CircuitBreaker::new(5, Duration::from_secs(60)));
        circuit_breakers.insert("weather".to_string(), CircuitBreaker::new(3, Duration::from_secs(30)));
        circuit_breakers.insert("geolocation".to_string(), CircuitBreaker::new(3, Duration::from_secs(30)));

        Ok(Self {
            config,
            http_client,
            circuit_breakers,
        })
    }

    /// Get HTTP client
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// Get configuration
    pub fn config(&self) -> &ExternalServicesConfig {
        &self.config
    }

    /// Execute request with circuit breaker protection
    pub async fn execute_with_circuit_breaker<F, T>(
        &mut self,
        service_name: &str,
        operation: F,
    ) -> AppResult<T>
    where
        F: std::future::Future<Output = AppResult<T>>,
    {
        let circuit_breaker = self.circuit_breakers
            .get_mut(service_name)
            .ok_or_else(|| AppError::Configuration(format!("Unknown service: {}", service_name)))?;

        if circuit_breaker.is_open() {
            return Err(AppError::ExternalService(format!("Circuit breaker open for service: {}", service_name)));
        }

        match operation.await {
            Ok(result) => {
                circuit_breaker.record_success();
                Ok(result)
            }
            Err(error) => {
                circuit_breaker.record_failure();
                Err(error)
            }
        }
    }

    /// Health check for all external services
    pub async fn health_check(&mut self) -> AppResult<ExternalServicesHealth> {
        let mut service_statuses = std::collections::HashMap::new();

        // Check email service
        if !self.config.email_service_key.is_empty() {
            let status = self.check_email_health().await;
            service_statuses.insert("email".to_string(), status);
        }

        // Check SMS service
        if !self.config.sms_service_key.is_empty() {
            let status = self.check_sms_health().await;
            service_statuses.insert("sms".to_string(), status);
        }

        // Check WhatsApp service
        if !self.config.whatsapp_token.is_empty() {
            let status = self.check_whatsapp_health().await;
            service_statuses.insert("whatsapp".to_string(), status);
        }

        // Check weather service
        if !self.config.weather_api_key.is_empty() {
            let status = self.check_weather_health().await;
            service_statuses.insert("weather".to_string(), status);
        }

        // Check geolocation service
        if !self.config.google_maps_api_key.is_empty() {
            let status = self.check_geolocation_health().await;
            service_statuses.insert("geolocation".to_string(), status);
        }

        let overall_status = if service_statuses.values().all(|s| matches!(s.status, ServiceStatus::Healthy)) {
            ServiceStatus::Healthy
        } else if service_statuses.values().any(|s| matches!(s.status, ServiceStatus::Healthy)) {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Unhealthy
        };

        Ok(ExternalServicesHealth {
            overall_status,
            services: service_statuses,
        })
    }

    async fn check_email_health(&mut self) -> ServiceHealth {
        // Implement email health check
        ServiceHealth {
            status: ServiceStatus::Healthy,
            latency_ms: 0,
            last_check: chrono::Utc::now(),
            error: None,
        }
    }

    async fn check_sms_health(&mut self) -> ServiceHealth {
        // Implement SMS health check
        ServiceHealth {
            status: ServiceStatus::Healthy,
            latency_ms: 0,
            last_check: chrono::Utc::now(),
            error: None,
        }
    }

    async fn check_whatsapp_health(&mut self) -> ServiceHealth {
        // Implement WhatsApp health check
        ServiceHealth {
            status: ServiceStatus::Healthy,
            latency_ms: 0,
            last_check: chrono::Utc::now(),
            error: None,
        }
    }

    async fn check_weather_health(&mut self) -> ServiceHealth {
        // Implement weather service health check
        ServiceHealth {
            status: ServiceStatus::Healthy,
            latency_ms: 0,
            last_check: chrono::Utc::now(),
            error: None,
        }
    }

    async fn check_geolocation_health(&mut self) -> ServiceHealth {
        // Implement geolocation service health check
        ServiceHealth {
            status: ServiceStatus::Healthy,
            latency_ms: 0,
            last_check: chrono::Utc::now(),
            error: None,
        }
    }
}

impl EmailConfig {
    fn from_env() -> AppResult<Self> {
        let provider = match std::env::var("EMAIL_PROVIDER").as_deref() {
            Ok("sendgrid") => EmailProvider::SendGrid,
            Ok("mailgun") => EmailProvider::Mailgun,
            Ok("smtp") => {
                let host = std::env::var("SMTP_HOST")
                    .map_err(|_| AppError::Configuration("SMTP_HOST required".to_string()))?;
                let port = std::env::var("SMTP_PORT")
                    .unwrap_or_else(|_| "587".to_string())
                    .parse()
                    .map_err(|_| AppError::Configuration("Invalid SMTP_PORT".to_string()))?;
                let username = std::env::var("SMTP_USERNAME")
                    .map_err(|_| AppError::Configuration("SMTP_USERNAME required".to_string()))?;
                let password = std::env::var("SMTP_PASSWORD")
                    .map_err(|_| AppError::Configuration("SMTP_PASSWORD required".to_string()))?;

                EmailProvider::SMTP { host, port, username, password }
            }
            _ => return Err(AppError::Configuration("EMAIL_PROVIDER must be sendgrid, mailgun, or smtp".to_string())),
        };

        let api_key = std::env::var("EMAIL_API_KEY")
            .map_err(|_| AppError::Configuration("EMAIL_API_KEY required".to_string()))?;
        let from_email = std::env::var("EMAIL_FROM_ADDRESS")
            .map_err(|_| AppError::Configuration("EMAIL_FROM_ADDRESS required".to_string()))?;
        let from_name = std::env::var("EMAIL_FROM_NAME")
            .unwrap_or_else(|_| "Terra Siaga".to_string());

        Ok(Self {
            provider,
            api_key,
            from_email,
            from_name,
            timeout: Duration::from_secs(30),
        })
    }
}

impl SmsConfig {
    fn from_env() -> AppResult<Self> {
        let provider = match std::env::var("SMS_PROVIDER").as_deref() {
            Ok("twilio") => SmsProvider::Twilio,
            Ok("vonage") => SmsProvider::Vonage,
            Ok("aws_sns") => SmsProvider::AWS_SNS,
            _ => return Err(AppError::Configuration("SMS_PROVIDER must be twilio, vonage, or aws_sns".to_string())),
        };

        let api_key = std::env::var("SMS_API_KEY")
            .map_err(|_| AppError::Configuration("SMS_API_KEY required".to_string()))?;
        let from_number = std::env::var("SMS_FROM_NUMBER").ok();

        Ok(Self {
            provider,
            api_key,
            from_number,
            timeout: Duration::from_secs(30),
        })
    }
}

impl WhatsAppConfig {
    fn from_env() -> AppResult<Self> {
        let provider = match std::env::var("WHATSAPP_PROVIDER").as_deref() {
            Ok("meta") => WhatsAppProvider::Meta,
            Ok("twilio") => WhatsAppProvider::Twilio,
            _ => return Err(AppError::Configuration("WHATSAPP_PROVIDER must be meta or twilio".to_string())),
        };

        let api_key = std::env::var("WHATSAPP_API_KEY")
            .map_err(|_| AppError::Configuration("WHATSAPP_API_KEY required".to_string()))?;
        let phone_number_id = std::env::var("WHATSAPP_PHONE_NUMBER_ID")
            .map_err(|_| AppError::Configuration("WHATSAPP_PHONE_NUMBER_ID required".to_string()))?;

        Ok(Self {
            provider,
            api_key,
            phone_number_id,
            timeout: Duration::from_secs(30),
        })
    }
}

impl WeatherConfig {
    fn from_env() -> AppResult<Self> {
        let provider = match std::env::var("WEATHER_PROVIDER").as_deref() {
            Ok("openweathermap") => WeatherProvider::OpenWeatherMap,
            Ok("accuweather") => WeatherProvider::AccuWeather,
            Ok("weatherapi") => WeatherProvider::WeatherAPI,
            _ => return Err(AppError::Configuration("WEATHER_PROVIDER must be openweathermap, accuweather, or weatherapi".to_string())),
        };

        let api_key = std::env::var("WEATHER_API_KEY")
            .map_err(|_| AppError::Configuration("WEATHER_API_KEY required".to_string()))?;

        Ok(Self {
            provider,
            api_key,
            timeout: Duration::from_secs(10),
            cache_duration: Duration::from_secs(300), // 5 minutes
        })
    }
}

impl GeolocationConfig {
    fn from_env() -> AppResult<Self> {
        let provider = match std::env::var("GEOLOCATION_PROVIDER").as_deref() {
            Ok("google") => GeolocationProvider::Google,
            Ok("nominatim") => GeolocationProvider::Nominatim,
            Ok("mapbox") => GeolocationProvider::MapBox,
            _ => return Err(AppError::Configuration("GEOLOCATION_PROVIDER must be google, nominatim, or mapbox".to_string())),
        };

        let api_key = std::env::var("GEOLOCATION_API_KEY").ok();

        Ok(Self {
            provider,
            api_key,
            timeout: Duration::from_secs(10),
            cache_duration: Duration::from_secs(86400), // 24 hours
        })
    }
}

/// Circuit breaker implementation
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_count: u32,
    failure_threshold: u32,
    last_failure_time: Option<std::time::Instant>,
    timeout: Duration,
    state: CircuitBreakerState,
}

#[derive(Debug, PartialEq)]
enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_count: 0,
            failure_threshold,
            last_failure_time: None,
            timeout,
            state: CircuitBreakerState::Closed,
        }
    }

    pub fn is_open(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() > self.timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
        self.last_failure_time = None;
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(std::time::Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }
}

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual service health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: ServiceStatus,
    pub latency_ms: u64,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub error: Option<String>,
}

/// Overall external services health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServicesHealth {
    pub overall_status: ServiceStatus,
    pub services: std::collections::HashMap<String, ServiceHealth>,
}
