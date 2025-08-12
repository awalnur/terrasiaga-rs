/// Monitoring module for Terra Siaga
/// Provides health checks and system monitoring capabilities

pub mod health;

// Re-export main monitoring components
pub use health::{
    HealthService, HealthStatus, HealthReport, ComponentHealth,
    HealthChecker, DatabaseHealthChecker, CacheHealthChecker,
    ExternalServiceHealthChecker, DiskSpaceHealthChecker,
    HealthConfig, create_health_service
};

// Type aliases for backward compatibility
pub type HealthMonitoringService = HealthService;
pub type DatabaseHealthCheck = DatabaseHealthChecker;
pub type CacheHealthCheck = CacheHealthChecker;
pub type ExternalApiHealthCheck = ExternalServiceHealthChecker;
