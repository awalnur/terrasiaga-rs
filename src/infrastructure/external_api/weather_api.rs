/// Weather API integration
/// Handles weather data retrieval for disaster monitoring

use reqwest::Client;
use serde_json::Value;
use crate::shared::error::AppResult;

pub struct WeatherService {
    client: Client,
    api_key: String,
}

impl WeatherService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn get_current_weather(&self, lat: f64, lng: f64) -> AppResult<Value> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}",
            lat, lng, self.api_key
        );

        let response = self.client.get(&url).send().await.unwrap();
        let data: Value = response.json().await.unwrap();

        Ok(data)
    }

    pub async fn get_weather_forecast(&self, lat: f64, lng: f64) -> AppResult<Value> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/forecast?lat={}&lon={}&appid={}",
            lat, lng, self.api_key
        );

        let response = self.client.get(&url).send().await.unwrap();
        let data: Value = response.json().await.unwrap();

        Ok(data)
    }

    pub async fn get_weather_alerts(&self, lat: f64, lng: f64) -> AppResult<Value> {
        let url = format!(
            "https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&exclude=minutely,hourly,daily&appid={}",
            lat, lng, self.api_key
        );

        let response = self.client.get(&url).send().await.unwrap();
        let data: Value = response.json().await.unwrap();

        Ok(data)
    }
}
