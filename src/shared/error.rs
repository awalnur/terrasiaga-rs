/// Global error handling for Terra Siaga
/// Defines common error types and result type alias with enhanced error tracking

use actix_web::{HttpResponse, ResponseError};
use diesel::result::Error as DieselError;
use r2d2::Error as R2D2Error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use thiserror::Error;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Application-wide result type
pub type AppResult<T> = Result<T, AppError>;
pub type DomainResult<T> = Result<T, DomainError>;

#[derive(Debug, Error)]
pub enum DatabaseError{
    #[error("Diesel error: {0}")]
    Diesel(#[from] DieselError),

    #[error("Connection pool error: {0}")]
    ConnectionPool(#[from] R2D2Error),

    #[error("Database error: {0}")]
    Other(String),
}

// Allow direct conversion from underlying database errors to AppError
impl From<DieselError> for AppError {
    fn from(e: DieselError) -> Self { AppError::Database(DatabaseError::Diesel(e)) }
}
impl From<R2D2Error> for AppError {
    fn from(e: R2D2Error) -> Self { AppError::Database(DatabaseError::ConnectionPool(e)) }
}

/// Enhanced application error with context and tracing
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
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
    
    // Enhanced error types for Terra Siaga domain
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Event processing error: {0}")]
    EventProcessing(String),

    #[error("Geolocation error: {0}")]
    Geolocation(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Token error: {0}")]
    Token(String),

    #[error("Disaster response error: {0}")]
    DisasterResponse(String),

    #[error("Emergency protocol error: {0}")]
    EmergencyProtocol(String),

    #[error("Communication error: {0}")]
    Communication(String),

    #[error("Resource allocation error: {0}")]
    ResourceAllocation(String),

    #[error("Volunteer coordination error: {0}")]
    VolunteerCoordination(String),

    #[error("Alert system error: {0}")]
    AlertSystem(String),

    #[error("Analytics error: {0}")]
    Analytics(String),

    #[error("Integration error: {0}")]
    Integration(String),

    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Data consistency error: {0}")]
    DataConsistency(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Multiple errors occurred")]
    Multiple(Vec<AppError>),
}

/// Domain-specific errors with rich context
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum DomainError {
    #[error("Invalid disaster report: {reason}")]
    InvalidDisasterReport { reason: String },

    #[error("Invalid coordinates: lat={latitude}, lng={longitude}")]
    InvalidCoordinates { latitude: f64, longitude: f64 },

    #[error("Emergency response already exists for disaster {disaster_id}")]
    EmergencyResponseExists { disaster_id: String },

    #[error("Volunteer {volunteer_id} is not available")]
    VolunteerNotAvailable { volunteer_id: String },

    #[error("Resource {resource_id} insufficient: required={required}, available={available}")]
    InsufficientResources {
        resource_id: String,
        required: u32,
        available: u32,
    },

    #[error("Invalid alert configuration: {reason}")]
    InvalidAlertConfiguration { reason: String },

    #[error("Duplicate report for the same incident")]
    DuplicateReport,

    #[error("User {user_id} lacks permission {permission}")]
    InsufficientPermissions {
        user_id: String,
        permission: String,
    },

    #[error("Invalid time range: start={start}, end={end}")]
    InvalidTimeRange { start: String, end: String },

    #[error("Evacuation route {route_id} is blocked")]
    EvacuationRouteBlocked { route_id: String },

    #[error("Shelter {shelter_id} at capacity: max={max_capacity}, current={current_occupancy}")]
    ShelterAtCapacity {
        shelter_id: String,
        max_capacity: u32,
        current_occupancy: u32,
    },

    #[error("Communication channel {channel} is down")]
    CommunicationChannelDown { channel: String },

    #[error("Invalid severity escalation: from={from} to={to}")]
    InvalidSeverityEscalation { from: String, to: String },

    #[error("Response deadline exceeded: deadline={deadline}, current_time={current_time}")]
    ResponseDeadlineExceeded {
        deadline: String,
        current_time: String,
    },
}

/// Error context for better debugging and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub endpoint: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub additional_data: serde_json::Value,
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self {
            error_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: None,
            session_id: None,
            request_id: None,
            endpoint: None,
            user_agent: None,
            ip_address: None,
            trace_id: None,
            span_id: None,
            additional_data: serde_json::Value::Null,
        }
    }
}

/// Enhanced error with context
#[derive(Debug)]
pub struct ContextualError {
    pub error: AppError,
    pub context: ErrorContext,
}

impl ContextualError {
    pub fn new(error: AppError) -> Self {
        Self {
            error,
            context: ErrorContext::default(),
        }
    }

    pub fn with_context(error: AppError, context: ErrorContext) -> Self {
        Self { error, context }
    }

    pub fn with_user(mut self, user_id: String) -> Self {
        self.context.user_id = Some(user_id);
        self
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.context.session_id = Some(session_id);
        self
    }

    pub fn with_request(mut self, request_id: String) -> Self {
        self.context.request_id = Some(request_id);
        self
    }

    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.context.endpoint = Some(endpoint);
        self
    }

    pub fn with_additional_data(mut self, data: serde_json::Value) -> Self {
        self.context.additional_data = data;
        self
    }
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (error_id: {})", self.error, self.context.error_id)
    }
}

impl std::error::Error for ContextualError {}

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,      // Warnings, non-critical issues
    Medium,   // Recoverable errors that affect user experience
    High,     // Errors that prevent normal operation
    Critical, // System-threatening errors requiring immediate attention
}

impl ErrorSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorSeverity::Low => "low",
            ErrorSeverity::Medium => "medium",
            ErrorSeverity::High => "high",
            ErrorSeverity::Critical => "critical",
        }
    }
}

/// Error classification for Terra Siaga domain
impl AppError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AppError::Validation(_) | AppError::BadRequest(_) => ErrorSeverity::Low,
            AppError::NotFound(_) | AppError::Authentication(_) => ErrorSeverity::Medium,
            AppError::Database(_) | AppError::ExternalService(_) => ErrorSeverity::High,
            AppError::InternalServer(_) | AppError::Configuration(_) => ErrorSeverity::Critical,
            AppError::BusinessRuleViolation(_) => ErrorSeverity::Medium,
            AppError::EmergencyProtocol(_) | AppError::DisasterResponse(_) => ErrorSeverity::Critical,
            AppError::AlertSystem(_) | AppError::Communication(_) => ErrorSeverity::High,
            AppError::ServiceUnavailable(_) => ErrorSeverity::Critical,
            _ => ErrorSeverity::Medium,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AppError::ExternalService(_)
                | AppError::Network(_)
                | AppError::Timeout(_)
                | AppError::ServiceUnavailable(_)
                | AppError::ResourceExhausted(_)
        )
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Authentication(_) => "AUTHENTICATION_ERROR",
            AppError::Authorization(_) => "AUTHORIZATION_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::InternalServer(_) => "INTERNAL_SERVER_ERROR",
            AppError::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::Configuration(_) => "CONFIGURATION_ERROR",
            AppError::BusinessRuleViolation(_) => "BUSINESS_RULE_VIOLATION",
            AppError::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            AppError::Cache(_) => "CACHE_ERROR",
            AppError::EventProcessing(_) => "EVENT_PROCESSING_ERROR",
            AppError::Geolocation(_) => "GEOLOCATION_ERROR",
            AppError::PermissionDenied(_) => "PERMISSION_DENIED",
            AppError::ResourceExhausted(_) => "RESOURCE_EXHAUSTED",
            AppError::Timeout(_) => "TIMEOUT_ERROR",
            AppError::Serialization(_) => "SERIALIZATION_ERROR",
            AppError::Network(_) => "NETWORK_ERROR",
            AppError::FileSystem(_) => "FILE_SYSTEM_ERROR",
            AppError::Encryption(_) => "ENCRYPTION_ERROR",
            AppError::Token(_) => "TOKEN_ERROR",
            AppError::DisasterResponse(_) => "DISASTER_RESPONSE_ERROR",
            AppError::EmergencyProtocol(_) => "EMERGENCY_PROTOCOL_ERROR",
            AppError::Communication(_) => "COMMUNICATION_ERROR",
            AppError::ResourceAllocation(_) => "RESOURCE_ALLOCATION_ERROR",
            AppError::VolunteerCoordination(_) => "VOLUNTEER_COORDINATION_ERROR",
            AppError::AlertSystem(_) => "ALERT_SYSTEM_ERROR",
            AppError::Analytics(_) => "ANALYTICS_ERROR",
            AppError::Integration(_) => "INTEGRATION_ERROR",
            AppError::PolicyViolation(_) => "POLICY_VIOLATION",
            AppError::DataConsistency(_) => "DATA_CONSISTENCY_ERROR",
            AppError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            AppError::Multiple(_) => "MULTIPLE_ERRORS",
            AppError::Forbidden(_) => "FORBIDDEN",
        }
    }

    pub fn http_status_code(&self) -> u16 {
        match self {
            AppError::Validation(_) | AppError::BadRequest(_) => 400,
            AppError::Authentication(_) | AppError::Unauthorized(_) => 401,
            AppError::Authorization(_) | AppError::PermissionDenied(_) => 403,
            AppError::NotFound(_) => 404,
            AppError::Conflict(_) => 409,
            AppError::RateLimitExceeded(_) => 429,
            AppError::Database(_)
            | AppError::InternalServer(_)
            | AppError::Configuration(_)
            | AppError::Cache(_)
            | AppError::EventProcessing(_)
            | AppError::Serialization(_)
            | AppError::FileSystem(_)
            | AppError::Encryption(_) => 500,
            AppError::ExternalService(_)
            | AppError::Network(_)
            | AppError::ServiceUnavailable(_) => 502,
            AppError::Timeout(_) => 504,
            _ => 500,
        }
    }
}

/// Convert domain errors to application errors
impl From<DomainError> for AppError {
    fn from(domain_error: DomainError) -> Self {
        match domain_error {
            DomainError::InvalidDisasterReport { reason } => {
                AppError::Validation(format!("Invalid disaster report: {}", reason))
            }
            DomainError::InvalidCoordinates { latitude, longitude } => {
                AppError::Geolocation(format!("Invalid coordinates: {}, {}", latitude, longitude))
            }
            DomainError::EmergencyResponseExists { disaster_id } => {
                AppError::Conflict(format!("Emergency response already exists for disaster {}", disaster_id))
            }
            DomainError::VolunteerNotAvailable { volunteer_id } => {
                AppError::ResourceAllocation(format!("Volunteer {} is not available", volunteer_id))
            }
            DomainError::InsufficientResources { resource_id, required, available } => {
                AppError::ResourceExhausted(format!(
                    "Insufficient resources for {}: required {}, available {}",
                    resource_id, required, available
                ))
            }
            DomainError::InvalidAlertConfiguration { reason } => {
                AppError::AlertSystem(format!("Invalid alert configuration: {}", reason))
            }
            DomainError::DuplicateReport => {
                AppError::Conflict("Duplicate report for the same incident".to_string())
            }
            DomainError::InsufficientPermissions { user_id, permission } => {
                AppError::PermissionDenied(format!("User {} lacks permission {}", user_id, permission))
            }
            DomainError::InvalidTimeRange { start, end } => {
                AppError::Validation(format!("Invalid time range: {} to {}", start, end))
            }
            DomainError::EvacuationRouteBlocked { route_id } => {
                AppError::EmergencyProtocol(format!("Evacuation route {} is blocked", route_id))
            }
            DomainError::ShelterAtCapacity { shelter_id, max_capacity, current_occupancy } => {
                AppError::ResourceExhausted(format!(
                    "Shelter {} at capacity: {}/{}",
                    shelter_id, current_occupancy, max_capacity
                ))
            }
            DomainError::CommunicationChannelDown { channel } => {
                AppError::Communication(format!("Communication channel {} is down", channel))
            }
            DomainError::InvalidSeverityEscalation { from, to } => {
                AppError::BusinessRuleViolation(format!(
                    "Invalid severity escalation from {} to {}",
                    from, to
                ))
            }
            DomainError::ResponseDeadlineExceeded { deadline, current_time } => {
                AppError::EmergencyProtocol(format!(
                    "Response deadline exceeded: deadline {}, current time {}",
                    deadline, current_time
                ))
            }
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status_code = self.http_status_code();
        let error_code = self.error_code();
        let severity = self.severity();

        HttpResponse::build(actix_web::http::StatusCode::from_u16(status_code).unwrap())
            .json(json!({
                "error": {
                    "code": error_code,
                    "message": self.to_string(),
                    "severity": severity.as_str(),
                    "timestamp": Utc::now().to_rfc3339(),
                    "retryable": self.is_retryable()
                }
            }))
    }
}

/// Error recovery strategies
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Determine if an error should trigger a retry
    pub fn should_retry(error: &AppError, attempt: u32, max_attempts: u32) -> bool {
        if attempt >= max_attempts {
            return false;
        }

        error.is_retryable()
    }

    /// Calculate backoff delay for retry attempts
    pub fn calculate_backoff(attempt: u32, base_delay_ms: u64, max_delay_ms: u64) -> std::time::Duration {
        let delay_ms = std::cmp::min(
            base_delay_ms * 2_u64.pow(attempt),
            max_delay_ms,
        );
        std::time::Duration::from_millis(delay_ms)
    }

    /// Create a circuit breaker state based on error patterns
    pub fn should_open_circuit(
        consecutive_failures: u32,
        failure_threshold: u32,
        error: &AppError,
    ) -> bool {
        consecutive_failures >= failure_threshold
            && matches!(
                error,
                AppError::ExternalService(_)
                    | AppError::ServiceUnavailable(_)
                    | AppError::Timeout(_)
                    | AppError::Network(_)
            )
    }
}

/// Error metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub error_count_by_type: std::collections::HashMap<String, u64>,
    pub error_count_by_severity: std::collections::HashMap<String, u64>,
    pub total_errors: u64,
    pub errors_per_minute: f64,
    pub mean_time_to_recovery: Option<f64>,
}

/// Error reporting service for external monitoring
#[async_trait::async_trait]
pub trait ErrorReporter: Send + Sync {
    async fn report_error(&self, error: &ContextualError) -> AppResult<()>;
    async fn report_metrics(&self, metrics: &ErrorMetrics) -> AppResult<()>;
}

/// Convenience macros for error creation
#[macro_export]
macro_rules! validation_error {
    ($msg:expr) => {
        AppError::Validation($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        AppError::Validation(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! business_error {
    ($msg:expr) => {
        AppError::BusinessRuleViolation($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        AppError::BusinessRuleViolation(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! emergency_error {
    ($msg:expr) => {
        AppError::EmergencyProtocol($msg.to_string())
    };
    ($fmt:expr, $($arg:tt)*) => {
        AppError::EmergencyProtocol(format!($fmt, $($arg)*))
    };
}

/// Helper functions for common error scenarios
pub fn not_found<T: fmt::Display>(resource: &str, id: T) -> AppError {
    AppError::NotFound(format!("{} with id {} not found", resource, id))
}

pub fn already_exists<T: fmt::Display>(resource: &str, id: T) -> AppError {
    AppError::Conflict(format!("{} with id {} already exists", resource, id))
}

pub fn insufficient_permissions(user_id: &str, action: &str) -> AppError {
    AppError::PermissionDenied(format!("User {} does not have permission to {}", user_id, action))
}

pub fn invalid_input<T: fmt::Display>(field: &str, value: T, reason: &str) -> AppError {
    AppError::Validation(format!("Invalid {}: '{}' - {}", field, value, reason))
}

pub fn service_unavailable(service: &str, reason: &str) -> AppError {
    AppError::ServiceUnavailable(format!("Service {} is unavailable: {}", service, reason))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_severity_classification() {
        assert_eq!(AppError::Validation("test".to_string()).severity(), ErrorSeverity::Low);
        assert_eq!(AppError::Database(DatabaseError::Diesel(DieselError::NotFound)).severity(), ErrorSeverity::High);
        assert_eq!(AppError::EmergencyProtocol("test".to_string()).severity(), ErrorSeverity::Critical);
    }

    #[test]
    fn test_error_retryability() {
        assert!(AppError::ExternalService("test".to_string()).is_retryable());
        assert!(!AppError::Validation("test".to_string()).is_retryable());
        assert!(AppError::Timeout("test".to_string()).is_retryable());
    }

    #[test]
    fn test_domain_error_conversion() {
        let domain_error = DomainError::InvalidCoordinates {
            latitude: 91.0,
            longitude: 181.0,
        };
        let app_error: AppError = domain_error.into();

        assert!(matches!(app_error, AppError::Geolocation(_)));
    }

    #[test]
    fn test_error_context() {
        let error = AppError::Validation("test error".to_string());
        let contextual = ContextualError::new(error)
            .with_user("user123".to_string())
            .with_endpoint("/api/test".to_string());

        assert_eq!(contextual.context.user_id, Some("user123".to_string()));
        assert_eq!(contextual.context.endpoint, Some("/api/test".to_string()));
    }

    #[test]
    fn test_helper_functions() {
        let error = not_found("User", "123");
        assert!(matches!(error, AppError::NotFound(_)));

        let error = insufficient_permissions("user1", "delete_user");
        assert!(matches!(error, AppError::PermissionDenied(_)));
    }
}
