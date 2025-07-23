/// Domain value objects
/// Immutable objects that represent concepts with no identity, only values

use serde::{Deserialize, Serialize};
use std::fmt;
use crate::shared::{AppResult, AppError};

/// Email value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> AppResult<Self> {
        let email = email.trim().to_lowercase();
        
        // Basic email validation
        if !email.contains('@') || !email.contains('.') {
            return Err(AppError::Validation("Invalid email format".to_string()));
        }
        
        // More thorough validation using regex
        let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if !email_regex.is_match(&email) {
            return Err(AppError::Validation("Invalid email format".to_string()));
        }
        
        Ok(Self(email))
    }
    
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Username value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Username(String);

impl Username {
    pub fn new(username: String) -> AppResult<Self> {
        let username = username.trim().to_lowercase();
        
        // Username validation rules
        if username.len() < 3 {
            return Err(AppError::Validation("Username must be at least 3 characters".to_string()));
        }
        
        if username.len() > 30 {
            return Err(AppError::Validation("Username must be at most 30 characters".to_string()));
        }
        
        let username_regex = regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
        if !username_regex.is_match(&username) {
            return Err(AppError::Validation("Username can only contain letters, numbers, and underscores".to_string()));
        }
        
        Ok(Self(username))
    }
    
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Indonesian phone number value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    pub fn new(phone: String) -> AppResult<Self> {
        let phone = phone.trim().replace(" ", "").replace("-", "");
        
        // Indonesian phone number validation
        let phone_regex = regex::Regex::new(r"^(\+62|62|0)[0-9]{8,12}$").unwrap();
        if !phone_regex.is_match(&phone) {
            return Err(AppError::Validation("Invalid Indonesian phone number format".to_string()));
        }
        
        // Normalize to +62 format
        let normalized = if phone.starts_with("0") {
            format!("+62{}", &phone[1..])
        } else if phone.starts_with("62") {
            format!("+{}", phone)
        } else if phone.starts_with("+62") {
            phone
        } else {
            return Err(AppError::Validation("Invalid phone number format".to_string()));
        };
        
        Ok(Self(normalized))
    }
    
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Geographic coordinates value object
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> AppResult<Self> {
        if latitude < -90.0 || latitude > 90.0 {
            return Err(AppError::Validation("Latitude must be between -90 and 90 degrees".to_string()));
        }
        
        if longitude < -180.0 || longitude > 180.0 {
            return Err(AppError::Validation("Longitude must be between -180 and 180 degrees".to_string()));
        }
        
        Ok(Self { latitude, longitude })
    }
    
    /// Calculate distance to another point in kilometers using Haversine formula
    pub fn distance_to(&self, other: &Coordinates) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;
        
        let lat1_rad = self.latitude.to_radians();
        let lat2_rad = other.latitude.to_radians();
        let delta_lat = (other.latitude - self.latitude).to_radians();
        let delta_lon = (other.longitude - self.longitude).to_radians();
        
        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        EARTH_RADIUS_KM * c
    }
    
    /// Check if coordinates are within Indonesia bounds (approximately)
    pub fn is_within_indonesia(&self) -> bool {
        // Approximate bounds of Indonesia
        self.latitude >= -11.0 && self.latitude <= 6.0
            && self.longitude >= 95.0 && self.longitude <= 141.0
    }
}

/// Address value object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub province: String,
    pub postal_code: Option<String>,
    pub country: String,
}

impl Address {
    pub fn new(
        street: String,
        city: String,
        province: String,
        postal_code: Option<String>,
        country: Option<String>,
    ) -> AppResult<Self> {
        if street.trim().is_empty() {
            return Err(AppError::Validation("Street address cannot be empty".to_string()));
        }
        
        if city.trim().is_empty() {
            return Err(AppError::Validation("City cannot be empty".to_string()));
        }
        
        if province.trim().is_empty() {
            return Err(AppError::Validation("Province cannot be empty".to_string()));
        }
        
        Ok(Self {
            street: street.trim().to_string(),
            city: city.trim().to_string(),
            province: province.trim().to_string(),
            postal_code: postal_code.map(|pc| pc.trim().to_string()),
            country: country.unwrap_or_else(|| "Indonesia".to_string()),
        })
    }
    
    pub fn full_address(&self) -> String {
        let mut parts = vec![&self.street, &self.city, &self.province];
        
        if let Some(ref postal_code) = self.postal_code {
            parts.push(postal_code);
        }
        
        parts.push(&self.country);
        parts.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(", ")
    }
}
