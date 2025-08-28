// filepath: /Users/development/RUST/terra-siaga/src/domain/value_objects/address.rs
use serde::{Deserialize, Serialize};
use crate::shared::error::{AppError, AppResult};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub province: String,
    pub country: Option<String>,
    pub postal_code: Option<String>,
}

impl Address {
    pub fn new(
        street: String,
        city: String,
        province: String,
        postal_code: Option<String>,
        country: Option<String>,
    ) -> AppResult<Self> {
        if street.trim().is_empty() || city.trim().is_empty() || province.trim().is_empty() {
            return Err(AppError::Validation("Street, city, and province are required".to_string()));
        }
        Ok(Self {
            street: street.trim().to_string(),
            city: city.trim().to_string(),
            province: province.trim().to_string(),
            country: country.map(|c| c.trim().to_string()).filter(|s| !s.is_empty()),
            postal_code: postal_code.map(|p| p.trim().to_string()).filter(|s| !s.is_empty()),
        })
    }
}

