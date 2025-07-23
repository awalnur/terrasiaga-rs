/// Geographic service integration
/// Handles geocoding, reverse geocoding, and geographic calculations

use reqwest::Client;
use serde_json::Value;
use crate::shared::error::AppResult;

pub struct GeoService {
    client: Client,
    api_key: String,
}

impl GeoService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn geocode(&self, address: &str) -> AppResult<(f64, f64)> {
        let url = format!(
            "https://api.geocoding.com/v1/geocode?q={}&key={}",
            urlencoding::encode(address),
            self.api_key
        );

        let response = self.client.get(&url).send().await?;
        let data: Value = response.json().await?;

        if let Some(results) = data["results"].as_array() {
            if let Some(first_result) = results.first() {
                let lat = first_result["geometry"]["location"]["lat"]
                    .as_f64()
                    .ok_or_else(|| crate::shared::error::AppError::ExternalService("Invalid latitude".to_string()))?;
                let lng = first_result["geometry"]["location"]["lng"]
                    .as_f64()
                    .ok_or_else(|| crate::shared::error::AppError::ExternalService("Invalid longitude".to_string()))?;
                
                return Ok((lat, lng));
            }
        }

        Err(crate::shared::error::AppError::NotFound("Address not found".to_string()))
    }

    pub async fn reverse_geocode(&self, lat: f64, lng: f64) -> AppResult<String> {
        let url = format!(
            "https://api.geocoding.com/v1/reverse?lat={}&lng={}&key={}",
            lat, lng, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        let data: Value = response.json().await?;

        if let Some(results) = data["results"].as_array() {
            if let Some(first_result) = results.first() {
                if let Some(address) = first_result["formatted_address"].as_str() {
                    return Ok(address.to_string());
                }
            }
        }

        Err(crate::shared::error::AppError::NotFound("Address not found".to_string()))
    }
}
