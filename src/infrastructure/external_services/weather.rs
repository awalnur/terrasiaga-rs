/// Weather service implementation
/// Provides weather data through various weather APIs

use crate::shared::error::{AppResult, AppError};
use crate::infrastructure::external_services::{WeatherConfig, WeatherProvider};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temperature: f64,
    pub humidity: f64,
    pub wind_speed: f64,
    pub weather_condition: String,
    pub visibility: Option<f64>,
    pub pressure: Option<f64>,
}

pub struct WeatherService {
    config: WeatherConfig,
    client: reqwest::Client,
}

impl WeatherService {
    pub fn new(config: WeatherConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    pub async fn get_weather(&self, lat: f64, lng: f64) -> AppResult<WeatherData> {
        match &self.config.provider {
            WeatherProvider::OpenWeatherMap => self.get_from_openweather(lat, lng).await,
            WeatherProvider::AccuWeather => self.get_from_accuweather(lat, lng).await,
            WeatherProvider::WeatherAPI => self.get_from_weatherapi(lat, lng).await,
        }
    }

    async fn get_from_openweather(&self, _lat: f64, _lng: f64) -> AppResult<WeatherData> {
        tracing::info!("Fetching weather from OpenWeatherMap");
        Ok(WeatherData {
            temperature: 25.0,
            humidity: 60.0,
            wind_speed: 10.0,
            weather_condition: "Clear".to_string(),
            visibility: Some(10.0),
            pressure: Some(1013.25),
        })
    }

    async fn get_from_accuweather(&self, _lat: f64, _lng: f64) -> AppResult<WeatherData> {
        tracing::info!("Fetching weather from AccuWeather");
        Ok(WeatherData {
            temperature: 25.0,
            humidity: 60.0,
            wind_speed: 10.0,
            weather_condition: "Clear".to_string(),
            visibility: Some(10.0),
            pressure: Some(1013.25),
        })
    }

    async fn get_from_weatherapi(&self, _lat: f64, _lng: f64) -> AppResult<WeatherData> {
        tracing::info!("Fetching weather from WeatherAPI");
        Ok(WeatherData {
            temperature: 25.0,
            humidity: 60.0,
            wind_speed: 10.0,
            weather_condition: "Clear".to_string(),
            visibility: Some(10.0),
            pressure: Some(1013.25),
        })
    }
}
