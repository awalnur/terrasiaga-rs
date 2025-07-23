/// Shared utilities and common types
/// This module contains utilities used across the entire application

pub mod database;
pub mod error;
pub mod jwt;
pub mod response;
pub mod types;
pub mod validation;

// Re-export commonly used types for convenience
pub use error::{AppError, AppResult, DomainError, DomainResult};
pub use response::ApiResponse;
pub use types::*;
