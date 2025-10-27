/// Enhanced validation system for Terra Siaga
/// Provides comprehensive input validation with detailed error reporting

use garde::Validate;
use serde::de::DeserializeOwned;

use crate::shared::error::{AppError, AppResult};
use crate::shared::types::{is_valid_email, is_valid_phone};

/// Validate request using garde validation
pub fn validate_request<T>(request: &T) -> AppResult<()>
where
    T: Validate<Context = ()>,
{
    match request.validate(&()) {
        Ok(_) => Ok(()),
        Err(errors) => {
            // Use garde's built-in formatter for robust, readable messages
            Err(AppError::Validation(format!(
                "Validation failed: {}",
                errors
            )))
        }
    }
}

/// Validate JSON payload and deserialize
pub fn validate_json<T>(json_str: &str) -> AppResult<T>
where
    T: DeserializeOwned + Validate<Context = ()>,
{
    let data: T = serde_json::from_str(json_str)
        .map_err(|e| AppError::Validation(format!("Invalid JSON: {}", e)))?;

    validate_request(&data).unwrap_or_else(|e|
        panic!("Validation failed: {}", e));
    Ok(data)
}

/// Custom validation functions
pub mod validators {
    use super::*;

    /// Validate email format
    pub fn validate_email(email: &str) -> AppResult<()> {
        if !is_valid_email(email) {
            return Err(AppError::Validation("Invalid email format".to_string()));
        }
        Ok(())
    }

    /// Validate phone number format
    pub fn validate_phone(phone: &str) -> AppResult<()> {
        if !is_valid_phone(phone) {
            return Err(AppError::Validation("Invalid phone number format".to_string()));
        }
        Ok(())
    }

    /// Validate password strength
    pub fn validate_password(password: &str) -> AppResult<()> {
        if password.len() < 8 {
            return Err(AppError::Validation(
                "Password must be at least 8 characters long".to_string()
            ));
        }

        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

        if !has_uppercase {
            return Err(AppError::Validation(
                "Password must contain at least one uppercase letter".to_string()
            ));
        }

        if !has_lowercase {
            return Err(AppError::Validation(
                "Password must contain at least one lowercase letter".to_string()
            ));
        }

        if !has_digit {
            return Err(AppError::Validation(
                "Password must contain at least one digit".to_string()
            ));
        }

        if !has_special {
            return Err(AppError::Validation(
                "Password must contain at least one special character".to_string()
            ));
        }

        Ok(())
    }

    /// Validate coordinates
    pub fn validate_coordinates(lat: f64, lon: f64) -> AppResult<()> {
        if lat < -90.0 || lat > 90.0 {
            return Err(AppError::Validation(
                "Latitude must be between -90 and 90 degrees".to_string()
            ));
        }

        if lon < -180.0 || lon > 180.0 {
            return Err(AppError::Validation(
                "Longitude must be between -180 and 180 degrees".to_string()
            ));
        }

        Ok(())
    }

    /// Validate disaster severity level
    pub fn validate_severity_level(level: i32) -> AppResult<()> {
        if level < 1 || level > 5 {
            return Err(AppError::Validation(
                "Severity level must be between 1 and 5".to_string()
            ));
        }
        Ok(())
    }

    /// Validate pagination parameters
    pub fn validate_pagination(limit: u32, _offset: u32) -> AppResult<()> {
        if limit == 0 {
            return Err(AppError::Validation(
                "Limit must be greater than 0".to_string()
            ));
        }

        if limit > 1000 {
            return Err(AppError::Validation(
                "Limit cannot exceed 1000".to_string()
            ));
        }

        Ok(())
    }

    /// Validate file size
    pub fn validate_file_size(size: usize, max_size: usize) -> AppResult<()> {
        if size > max_size {
            return Err(AppError::Validation(format!(
                "File size ({} bytes) exceeds maximum allowed size ({} bytes)",
                size, max_size
            )));
        }
        Ok(())
    }

    /// Validate file extension
    pub fn validate_file_extension(filename: &str, allowed_extensions: &[&str]) -> AppResult<()> {
        let extension = filename
            .split('.')
            .last()
            .unwrap_or("")
            .to_lowercase();

        if !allowed_extensions.contains(&extension.as_str()) {
            return Err(AppError::Validation(format!(
                "File extension '{}' is not allowed. Allowed extensions: {}",
                extension,
                allowed_extensions.join(", ")
            )));
        }

        Ok(())
    }

    /// Validate UUID format
    pub fn validate_uuid(uuid_str: &str) -> AppResult<uuid::Uuid> {
        uuid::Uuid::parse_str(uuid_str)
            .map_err(|_| AppError::Validation("Invalid UUID format".to_string()))
    }

    /// Validate date range
    pub fn validate_date_range(start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> AppResult<()> {
        if start >= end {
            return Err(AppError::Validation(
                "Start date must be before end date".to_string()
            ));
        }

        let now = chrono::Utc::now();
        if end > now + chrono::Duration::days(365) {
            return Err(AppError::Validation(
                "End date cannot be more than one year in the future".to_string()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::validators::*;

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("").is_err());
    }

    #[test]
    fn test_phone_validation() {
        assert!(validate_phone("+1234567890").is_ok());
        assert!(validate_phone("1234567890").is_ok());
        assert!(validate_phone("123-456-7890").is_ok());
        assert!(validate_phone("invalid").is_err());
        assert!(validate_phone("123").is_err());
    }

    #[test]
    fn test_password_validation() {
        assert!(validate_password("Password123!").is_ok());
        assert!(validate_password("weak").is_err());
        assert!(validate_password("PASSWORD123!").is_err()); // no lowercase
        assert!(validate_password("password123!").is_err()); // no uppercase
        assert!(validate_password("Password!").is_err()); // no digit
        assert!(validate_password("Password123").is_err()); // no special char
    }

    #[test]
    fn test_coordinates_validation() {
        assert!(validate_coordinates(45.0, 90.0).is_ok());
        assert!(validate_coordinates(-90.0, -180.0).is_ok());
        assert!(validate_coordinates(91.0, 0.0).is_err());
        assert!(validate_coordinates(0.0, 181.0).is_err());
    }

    #[test]
    fn test_severity_validation() {
        assert!(validate_severity_level(1).is_ok());
        assert!(validate_severity_level(5).is_ok());
        assert!(validate_severity_level(0).is_err());
        assert!(validate_severity_level(6).is_err());
    }

    #[test]
    fn test_pagination_validation() {
        assert!(validate_pagination(20, 0).is_ok());
        assert!(validate_pagination(1000, 100).is_ok());
        assert!(validate_pagination(0, 0).is_err());
        assert!(validate_pagination(1001, 0).is_err());
    }

    #[test]
    fn test_file_extension_validation() {
        let allowed = &["jpg", "png", "pdf"];
        assert!(validate_file_extension("image.jpg", allowed).is_ok());
        assert!(validate_file_extension("document.pdf", allowed).is_ok());
        assert!(validate_file_extension("virus.exe", allowed).is_err());
    }

    #[test]
    fn test_uuid_validation() {
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(validate_uuid("invalid-uuid").is_err());
    }
}
