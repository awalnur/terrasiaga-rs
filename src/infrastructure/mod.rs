/// Improved infrastructure layer - External concerns and implementations
/// This layer implements the ports defined in the domain layer with better organization

pub mod database;
pub mod cache;
pub mod messaging;
pub mod external_services;
pub mod repository;
pub mod monitoring;

// Dependency injection container
pub mod container;

// Re-export infrastructure components
pub use container::*;
pub use database::{DatabaseService, DatabaseConfig};
pub use cache::{CacheService, CacheConfig};
pub use external_services::{ExternalServicesManager, ExternalServicesConfig};
pub use monitoring::{HealthMonitor, HealthCheckScheduler, ApplicationHealth};

// External API integrations
pub mod external_api;


