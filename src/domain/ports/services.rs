/// Domain service ports - Business service interfaces
/// These define contracts for business services that infrastructure must implement

use async_trait::async_trait;
use crate::domain::User;
use crate::shared::{AppResult, AppError, UserId, Coordinates};

// Authentication service interface
#[async_trait]
pub trait AuthService: Send + Sync {
    async fn hash_password(&self, password: &str) -> AppResult<String>;
    async fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool>;
    async fn generate_tokens(&self, user_id: UserId) -> AppResult<TokenPair>;
    // Optional convenience; default returns configuration error unless overridden
    async fn generate_token(&self, _user: &User) -> AppResult<String> {
        Err(AppError::Configuration("generate_token is not supported by this AuthService".to_string()))
    }
    async fn verify_token(&self, token: &str) -> AppResult<UserId>;
    async fn refresh_token(&self, refresh_token: &str) -> AppResult<TokenPair>;
    // Optional; default is a no-op
    async fn revoke_token(&self, _token: &str) -> AppResult<()> { Ok(()) }
    // Optional account lock helpers
    async fn track_failed_login(&self, _email: &str, _ip_address: Option<&str>) -> AppResult<bool> { Ok(true) }
    async fn is_account_locked(&self, _email: &str, _ip_address: Option<&str>) -> AppResult<bool> { Ok(false) }
    async fn clear_failed_attempts(&self, _email: &str) -> AppResult<()> { Ok(()) }
}

#[derive(Debug, Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

// Notification service interface
#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> AppResult<()>;
    async fn send_sms(&self, to: &str, message: &str) -> AppResult<()>;
    async fn send_whatsapp(&self, to: &str, message: &str) -> AppResult<()>;
    async fn send_push_notification(&self, user_id: UserId, title: &str, body: &str) -> AppResult<()>;

    // Add missing method for emergency response notifications
    async fn notify_emergency_dispatch(
        &self,
        disaster: &crate::domain::entities::disaster::Disaster,
        response: &EmergencyResponse
    ) -> AppResult<()>;

    async fn notify_disaster_update(&self, disaster: &crate::domain::entities::disaster::Disaster) -> AppResult<()>;
    async fn notify_volunteers(&self, disaster: &crate::domain::entities::disaster::Disaster, volunteers: &[UserId]) -> AppResult<()>;
    async fn send_emergency_alert(&self, message: &str, recipients: &[UserId]) -> AppResult<()>;
}

// Emergency response struct
#[derive(Debug, Clone)]
pub struct EmergencyResponse {
    pub id: crate::shared::EmergencyResponseId,
    pub disaster_id: crate::shared::DisasterId,
    pub response_team_type: String,
    pub assigned_station: Option<String>,
    pub estimated_arrival: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub resources_deployed: Vec<String>,
}

// Geolocation service interface
#[async_trait]
pub trait GeolocationService: Send + Sync {
    async fn geocode(&self, address: &str) -> AppResult<Coordinates>;
    async fn reverse_geocode(&self, coordinates: &Coordinates) -> AppResult<String>;
    async fn calculate_distance(&self, from: &Coordinates, to: &Coordinates) -> AppResult<f64>;
    async fn get_nearby_locations(&self, center: &Coordinates, radius_km: f64) -> AppResult<Vec<Coordinates>>;
}

// Weather service interface
#[async_trait]
pub trait WeatherService: Send + Sync {
    async fn get_current_weather(&self, coordinates: Coordinates) -> AppResult<WeatherData>;
    async fn get_weather_forecast(&self, coordinates: Coordinates, days: u8) -> AppResult<Vec<WeatherData>>;
    async fn get_weather_alerts(&self, coordinates: Coordinates) -> AppResult<Vec<WeatherAlert>>;
}

#[derive(Debug, Clone)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: f64,
    pub wind_speed: f64,
    pub wind_direction: String,
    pub conditions: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct WeatherAlert {
    pub alert_type: String,
    pub severity: String,
    pub description: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

// Type alias for backward compatibility
pub type GeoService = dyn GeolocationService;
