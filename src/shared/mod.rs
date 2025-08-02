/// Shared utilities and common types
/// This module contains utilities used across the entire application

pub mod auth_middleware;
pub mod database;
pub mod error;
pub mod jwt;
pub mod response;
pub mod security;
pub mod types;
pub mod validation;
pub mod macros;

// Re-export commonly used types and functions
pub use error::{AppError, AppResult};
pub use types::*;
pub use security::{
    PasswordSecurity, SecurityConfig, PasswordStrength, SecurityError,
    hash_password, verify_password, validate_password_strength, generate_secure_password
};
pub use auth_middleware::{AuthService, SecurityMiddleware};

// Security utilities
pub mod security_utils {
    use super::security::PasswordSecurity;
    use std::sync::OnceLock;

    /// Global password security instance
    static GLOBAL_SECURITY: OnceLock<PasswordSecurity> = OnceLock::new();

    /// Get global password security instance
    pub fn get_security() -> &'static PasswordSecurity {
        GLOBAL_SECURITY.get_or_init(|| PasswordSecurity::new())
    }

    /// Initialize global security with custom configuration
    pub fn init_security(security: PasswordSecurity) -> Result<(), PasswordSecurity> {
        GLOBAL_SECURITY.set(security)
    }
}
