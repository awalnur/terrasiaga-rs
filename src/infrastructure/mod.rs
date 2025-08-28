/// Improved infrastructure layer - External concerns and implementations
/// This layer implements the ports defined in the domain layer with better organization

pub mod database;
pub mod cache;
pub mod messaging;
pub mod external_services;
pub mod repository;
pub mod monitoring;
pub mod security;

// Dependency injection container
pub mod container;

// Re-export infrastructure components
pub use container::*;
pub use database::DatabaseService;
pub use crate::config::DatabaseConfig;
pub use cache::CacheService;
pub use external_services::{ExternalServicesManager, ExternalServicesConfig};
pub use monitoring::{HealthService, HealthStatus, HealthReport, ComponentHealth};
pub use security::{PasetoSecurityService, PasetoConfig, SecurityServiceFactory};

// Type aliases for backward compatibility
pub type HealthMonitor = monitoring::HealthService;
pub type HealthCheckScheduler = monitoring::HealthService;
pub type ApplicationHealth = monitoring::HealthReport;

// External API integrations
pub mod external_api;

// Re-export CacheConfig from shared module
pub use crate::infrastructure::cache::CacheConfig;
