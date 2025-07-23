/// JWT authentication utilities for Terra Siaga
/// Handles token generation, validation, and parsing

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::shared::error::{AppError, AppResult};

/// User role types
#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
    Reporter,   // Regular user who can report disasters
    Volunteer,  // Volunteer who can respond to reports
    Admin,      // Administrator with full access
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub exp: usize,         // Expiration time
    pub iat: usize,         // Issued at
    pub role: String,       // User role
}

/// JWT utilities
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    /// Create a new JWT utils instance with the given secret
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
        }
    }

    /// Generate a JWT token for a user
    pub fn generate_token(&self, user_id: &str, role: &str, expires_in_hours: i64) -> AppResult<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AppError::InternalServer("Failed to get current time".to_string()))?
            .as_secs() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + (expires_in_hours * 3600) as usize,
            iat: now,
            role: role.to_string(),
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Jwt(e))?;

        Ok(token)
    }

    /// Validate a JWT token and extract claims
    pub fn validate_token(&self, token: &str) -> AppResult<Claims> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| AppError::Jwt(e))?;

        Ok(token_data.claims)
    }
}

// Convenience function for backward compatibility
pub fn validate_token(token: &str) -> AppResult<Claims> {
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key".to_string());
    let jwt_service = JwtService::new(&secret);
    jwt_service.validate_token(token)
}