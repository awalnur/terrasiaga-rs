/// Middleware modules for Terra Siaga
/// Contains authentication, CORS, and logging middleware

pub mod auth;
pub mod cors;
pub mod logging;

// Re-export middleware functions and types
pub use auth::jwt_middleware;
pub use cors::configure_cors;
pub use logging::{init_logger, configure_logger as configure_request_logger};

// Provide backward compatibility aliases
pub use jwt_middleware as AuthMiddleware;
pub use configure_request_logger as RequestLogger;
