/// Shared utilities and common types
/// This module contains utilities used across the entire application

pub mod database;
pub mod error;
pub mod response;
pub mod security;
pub mod types;
pub mod validation;
pub mod macros;

// Replace JWT with PASETO
pub mod paseto_auth;

// Enhanced authentication and middleware
pub mod auth_middleware;

// Cache and performance utilities
pub mod cache;

// Event handling and messaging
pub mod events;

// Rate limiting utilities
pub mod rate_limiter;

// Geographic utilities
pub mod geo_utils;

// Re-export commonly used types and functions
pub use error::{AppError, AppResult};
pub use types::*;
pub use security::{
    PasswordSecurity, SecurityConfig, PasswordStrength, SecurityError,
    hash_password, verify_password, validate_password_strength, generate_secure_password
};
pub use paseto_auth::{PasetoService, TokenClaims, TokenPair, TokenType};
pub use auth_middleware::{AuthService, AuthMiddleware};
pub use response::ApiResponse;
pub use cache::{CacheService, CacheConfig};
pub use rate_limiter::{RateLimiter, RateLimitConfig};
pub use security::{PasswordService, SecurityService};

/// Security utilities
pub mod security_utils {
    use super::security::PasswordSecurity;
    use std::sync::OnceLock;

    /// Global password security instance
    static GLOBAL_SECURITY: OnceLock<PasswordSecurity> = OnceLock::new();

    /// Get global password security instance
    pub fn get_security() -> &'static PasswordSecurity {
        GLOBAL_SECURITY.get_or_init(|| {
            PasswordSecurity::new()
                .with_min_length(8)
                .with_require_uppercase(true)
                .with_require_lowercase(true)
                .with_require_numbers(true)
                .with_require_special_chars(true)
        })
    }

    /// Initialize security with custom configuration
    pub fn init_security(config: PasswordSecurity) -> Result<(), &'static str> {
        GLOBAL_SECURITY.set(config).map_err(|_| "Security already initialized")
    }
}

// Enhanced validation utilities
pub mod validation_utils {
    use super::types::*;
    use garde::Validate;
    use regex::Regex;
    use std::sync::OnceLock;

    static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
    static PHONE_REGEX: OnceLock<Regex> = OnceLock::new();

    fn get_email_regex() -> &'static Regex {
        EMAIL_REGEX.get_or_init(|| {
            Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
        })
    }

    fn get_phone_regex() -> &'static Regex {
        PHONE_REGEX.get_or_init(|| {
            Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap()
        })
    }

    /// Validate email address
    pub fn validate_email(email: &str) -> bool {
        get_email_regex().is_match(email)
    }

    /// Validate phone number (international format)
    pub fn validate_phone(phone: &str) -> bool {
        get_phone_regex().is_match(phone)
    }

    /// Validate coordinates
    pub fn validate_coordinates(coords: &Coordinates) -> Result<(), String> {
        coords.validate(&())
            .map_err(|e| format!("Invalid coordinates: {}", e))
    }

    /// Validate disaster type against business rules
    pub fn validate_disaster_context(
        disaster_type: &DisasterType,
        severity: &SeverityLevel,
        location: &LocationInfo,
    ) -> Result<(), String> {
        // Check if severity matches disaster type expectations
        let expected_severity = disaster_type.default_severity();
        if severity < &expected_severity && severity != &SeverityLevel::Low {
            return Err(format!(
                "Severity level {:?} seems low for disaster type {:?}",
                severity, disaster_type
            ));
        }

        // Validate location accuracy for critical disasters
        if severity == &SeverityLevel::Critical {
            if location.accuracy_radius.is_none() ||
               location.accuracy_radius.unwrap() > 100.0 {
                return Err("Critical disasters require precise location (accuracy < 100m)".to_string());
            }
        }

        Ok(())
    }
}

// Geographic calculation utilities
pub mod geo_calculations {
    use super::types::{Coordinates, GeoBounds};
    use super::types::constants::EARTH_RADIUS_KM;

    /// Calculate bounding box for a point with given radius
    pub fn calculate_bounding_box(center: &Coordinates, radius_km: f64) -> GeoBounds {
        let lat_delta = radius_km / 111.32; // Approximate km per degree latitude
        let lng_delta = radius_km / (111.32 * center.latitude.to_radians().cos());

        GeoBounds {
            north_east: Coordinates {
                latitude: center.latitude + lat_delta,
                longitude: center.longitude + lng_delta,
                altitude: None,
            },
            south_west: Coordinates {
                latitude: center.latitude - lat_delta,
                longitude: center.longitude - lng_delta,
                altitude: None,
            },
        }
    }

    /// Find center point of multiple coordinates
    pub fn calculate_center_point(points: &[Coordinates]) -> Option<Coordinates> {
        if points.is_empty() {
            return None;
        }

        let sum_lat: f64 = points.iter().map(|p| p.latitude).sum();
        let sum_lng: f64 = points.iter().map(|p| p.longitude).sum();
        let count = points.len() as f64;

        Some(Coordinates {
            latitude: sum_lat / count,
            longitude: sum_lng / count,
            altitude: None,
        })
    }

    /// Check if point is within polygon (simple implementation)
    pub fn point_in_polygon(point: &Coordinates, polygon: &[Coordinates]) -> bool {
        if polygon.len() < 3 {
            return false;
        }

        let mut inside = false;
        let mut j = polygon.len() - 1;

        for i in 0..polygon.len() {
            if ((polygon[i].latitude > point.latitude) != (polygon[j].latitude > point.latitude))
                && (point.longitude < (polygon[j].longitude - polygon[i].longitude) *
                   (point.latitude - polygon[i].latitude) /
                   (polygon[j].latitude - polygon[i].latitude) + polygon[i].longitude)
            {
                inside = !inside;
            }
            j = i;
        }

        inside
    }
}

// Time and scheduling utilities
pub mod time_utils {
    use chrono::{DateTime, Utc, Duration as ChronoDuration};
    use super::types::Duration;

    /// Check if timestamp is within business hours
    pub fn is_business_hours(timestamp: &DateTime<Utc>, timezone: &str) -> bool {
        // This is a simplified implementation
        // In production, you'd want proper timezone handling
        let hour = timestamp.hour();
        hour >= 9 && hour < 17 // 9 AM to 5 PM
    }

    /// Calculate time until next business hour
    pub fn time_until_business_hours(timestamp: &DateTime<Utc>) -> Duration {
        let hour = timestamp.hour();
        if hour < 9 {
            Duration::from_hours((9 - hour) as u64)
        } else if hour >= 17 {
            Duration::from_hours((24 - hour + 9) as u64)
        } else {
            Duration::from_minutes(0)
        }
    }

    /// Format duration for human consumption
    pub fn humanize_duration(duration: &Duration) -> String {
        duration.as_human_readable()
    }
}
