/// Domain service ports - Business service interfaces
/// These define contracts for business services that infrastructure must implement

use async_trait::async_trait;
use crate::shared::{AppResult, UserId, Coordinates};

// Authentication service interface
#[async_trait]
pub trait AuthService: Send + Sync {
    async fn hash_password(&self, password: &str) -> AppResult<String>;
    async fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool>;
    async fn generate_tokens(&self, user_id: UserId) -> AppResult<TokenPair>;
    async fn validate_token(&self, token: &str) -> AppResult<TokenClaims>;
    async fn refresh_token(&self, refresh_token: &str) -> AppResult<TokenPair>;
    async fn revoke_token(&self, token: &str) -> AppResult<()>;
}

#[derive(Debug, Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone)]
pub struct TokenClaims {
    pub user_id: UserId,
    pub email: String,
    pub role: String,
    pub exp: i64,
}

// Notification service interface
#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> AppResult<()>;
    async fn send_sms(&self, to: &str, message: &str) -> AppResult<()>;
    async fn send_whatsapp(&self, to: &str, message: &str) -> AppResult<()>;
    async fn send_push_notification(&self, user_id: UserId, title: &str, body: &str) -> AppResult<()>;
}

// Geolocation service interface
#[async_trait]
pub trait GeolocationService: Send + Sync {
    async fn geocode(&self, address: &str) -> AppResult<Coordinates>;
    async fn reverse_geocode(&self, coordinates: Coordinates) -> AppResult<String>;
    async fn calculate_distance(&self, from: Coordinates, to: Coordinates) -> AppResult<f64>;
    async fn get_nearby_locations(&self, center: Coordinates, radius_km: f64) -> AppResult<Vec<Coordinates>>;
}

// Weather service interface
#[async_trait]
pub trait WeatherService: Send + Sync {
    async fn get_current_weather(&self, coordinates: Coordinates) -> AppResult<WeatherData>;
    async fn get_weather_forecast(&self, coordinates: Coordinates, days: u8) -> AppResult<Vec<WeatherData>>;
    async fn get_weather_alerts(&self, coordinates: Coordinates) -> AppResult<Vec<WeatherAlert>>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: f64,
    pub wind_speed: f64,
    pub wind_direction: f64,
    pub pressure: f64,
    pub visibility: f64,
    pub condition: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WeatherAlert {
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
}
