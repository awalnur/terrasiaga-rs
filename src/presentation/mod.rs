/// Presentation layer - API controllers and HTTP handling
/// This layer handles HTTP requests/responses and delegates to application services

pub mod api;

// Re-export commonly used presentation components
pub use api::*;
