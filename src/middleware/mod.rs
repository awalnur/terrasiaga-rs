/// Middleware modules for Terra Siaga
/// Contains authentication, CORS, and logging middleware

pub mod auth;
pub mod cors;
pub mod logging;
pub mod errors;

// Re-export middleware functions and types
pub use auth::{AuthMiddleware, AuthSession};
pub use cors::configure_cors;
pub use logging::{init_logger, configure_logger as configure_request_logger};
pub use errors::ErrorHandler;

// Create jwt_middleware alias for backward compatibility
// Provide backward compatibility aliases
