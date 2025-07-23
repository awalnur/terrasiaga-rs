/// Global error handling for Terra Siaga
/// Defines common error types and result type alias
use actix_web::{HttpResponse, ResponseError};
use diesel::result::Error as DieselError;
use serde_json::json;
use std::fmt;
use thiserror::Error;

/// Application-wide result type
pub type AppResult<T> = Result<T, AppError>;
pub type DomainResult<T> = Result<T, AppError>;

/// Main application error enum
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] DieselError),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error: {0}")]
    InternalServer(String),
    
    #[error("External service error: {0}")]
    ExternalService(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),
    
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    
    #[error("Redis error: {0}")]
    Redis(String),
}

// Add Redis error conversion
impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::Redis(err.to_string())
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status_code = match self {
            AppError::Validation(_) | AppError::BadRequest(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::Authentication(_) | AppError::Unauthorized(_) => actix_web::http::StatusCode::UNAUTHORIZED,
            AppError::Authorization(_) => actix_web::http::StatusCode::FORBIDDEN,
            AppError::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            AppError::Conflict(_) => actix_web::http::StatusCode::CONFLICT,
            AppError::Configuration(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        HttpResponse::build(status_code).json(json!({
            "error": {
                "message": self.to_string(),
                "type": format!("{:?}", self).split('(').next().unwrap_or("Unknown"),
                "timestamp": chrono::Utc::now(),
            }
        }))
    }
}

/// Domain-specific error for business logic
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Resource not available: {0}")]
    ResourceNotAvailable(String),
}

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::BusinessRuleViolation(msg) => AppError::BusinessRuleViolation(msg),
            DomainError::InvalidState(msg) => AppError::Conflict(msg),
            DomainError::ResourceNotAvailable(msg) => AppError::NotFound(msg),
        }
    }
}
