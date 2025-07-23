/// Validation utilities and custom validators
/// Provides common validation logic used across the application

use validator::{Validate, ValidationError};
use regex::Regex;
use std::collections::HashMap;

/// Validates Indonesian phone numbers
pub fn validate_indonesian_phone(phone: &str) -> Result<(), ValidationError> {
    let phone_regex = Regex::new(r"^\+?62[0-9]{8,12}$").unwrap();
    
    if phone_regex.is_match(phone) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_indonesian_phone"))
    }
}

/// Validates coordinates (latitude/longitude)
pub fn validate_latitude(lat: f64) -> Result<(), ValidationError> {
    if lat >= -90.0 && lat <= 90.0 {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_latitude"))
    }
}

pub fn validate_longitude(lng: f64) -> Result<(), ValidationError> {
    if lng >= -180.0 && lng <= 180.0 {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_longitude"))
    }
}

/// Validates emergency severity levels
pub fn validate_severity(severity: i32) -> Result<(), ValidationError> {
    if (1..=5).contains(&severity) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_severity_level"))
    }
}

/// Common validation rules
pub struct ValidationRules;

impl ValidationRules {
    pub fn password_strength(password: &str) -> bool {
        password.len() >= 8 
            && password.chars().any(|c| c.is_uppercase())
            && password.chars().any(|c| c.is_lowercase())
            && password.chars().any(|c| c.is_numeric())
    }

    pub fn username_format(username: &str) -> bool {
        let username_regex = Regex::new(r"^[a-zA-Z0-9_]{3,30}$").unwrap();
        username_regex.is_match(username)
    }
}
