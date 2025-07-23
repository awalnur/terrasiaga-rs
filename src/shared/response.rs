/// API Response utilities
/// Standardized response formats for the Terra Siaga API

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::shared::error::AppError;

/// Standard API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

impl<T> ApiResponse<T> {
    /// Create a successful response with data
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            timestamp: Utc::now(),
            version: "1.0".to_string(),
        }
    }

    /// Create a successful response with custom message
    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message),
            timestamp: Utc::now(),
            version: "1.0".to_string(),
        }
    }
}

impl ApiResponse<()> {
    /// Create an error response
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
            timestamp: Utc::now(),
            version: "1.0".to_string(),
        }
    }

    /// Create an error response from AppError
    pub fn from_error(error: AppError) -> Self {
        Self::error(error.to_string())
    }
}