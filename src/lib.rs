/// Terra Siaga - Emergency Response and Disaster Management System
/// Clean Architecture implementation with proper separation of concerns

// Core domain layer - business logic and entities
pub mod domain;

// Application layer - use cases and application services
pub mod application;

// Handlers layer - HTTP request handlers and service coordination
pub mod middleware;

// Infrastructure layer - external concerns (database, web, etc.)
pub mod infrastructure;

// Presentation layer - API controllers and DTOs
pub mod presentation;

// Shared utilities and cross-cutting concerns
pub mod shared;

// Configuration and environment setup
pub mod config;

// Re-export commonly used types for convenience
pub use shared::error::{AppError, AppResult};
pub use shared::types::*;
