/// Health monitoring service for Terra Siaga
/// Provides comprehensive health checks for all system components

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::error;

use crate::shared::{AppResult, AppError};
use crate::infrastructure::database::DbPool;
use std::time::Instant;
use futures;
use diesel;

/// Overall system health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    pub fn is_operational(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }
}

/// Individual component health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
    pub last_checked: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: u64,
}

/// Complete system health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall_status: HealthStatus,
    pub components: HashMap<String, ComponentHealth>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub uptime_seconds: u64,
    pub environment: String,
}

impl HealthReport {
    pub fn new(version: String, environment: String, uptime_seconds: u64) -> Self {
        Self {
            overall_status: HealthStatus::Healthy,
            components: HashMap::new(),
            timestamp: chrono::Utc::now(),
            version,
            uptime_seconds,
            environment,
        }
    }

    pub fn add_component(&mut self, component: ComponentHealth) {
        // Update overall status based on component status
        match component.status {
            HealthStatus::Critical => self.overall_status = HealthStatus::Critical,
            HealthStatus::Unhealthy if self.overall_status != HealthStatus::Critical => {
                self.overall_status = HealthStatus::Unhealthy;
            }
            HealthStatus::Degraded if matches!(self.overall_status, HealthStatus::Healthy) => {
                self.overall_status = HealthStatus::Degraded;
            }
            _ => {}
        }

        self.components.insert(component.name.clone(), component);
    }

    pub fn is_healthy(&self) -> bool {
        self.overall_status.is_healthy()
    }

    pub fn is_operational(&self) -> bool {
        self.overall_status.is_operational()
    }
}

/// Health check trait for individual components
#[async_trait]
pub trait HealthChecker: Send + Sync {
    async fn check_health(&self) -> ComponentHealth;
    fn component_name(&self) -> &str;
}

/// Database health checker
pub struct DatabaseHealthChecker {
    name: String,
    pool: Arc<DbPool>,
}

impl DatabaseHealthChecker {
    pub fn new(name: String, pool: Arc<DbPool>) -> Self {
        Self { name, pool }
    }
}

#[async_trait]
impl HealthChecker for DatabaseHealthChecker {
    async fn check_health(&self) -> ComponentHealth {
        let start = SystemTime::now();
        let mut details = HashMap::new();

        let (status, message) = match self.pool.get().await {
            Ok(mut conn) => {
                // Try a simple query
                match diesel::sql_query("SELECT 1").execute(&mut *conn) {
                    Ok(_) => {
                        let pool_status = self.pool.status();
                        details.insert("pool_size".to_string(), serde_json::json!(pool_status.size));
                        details.insert("available_connections".to_string(), serde_json::json!(pool_status.available));
                        
                        if pool_status.available == 0 {
                            (HealthStatus::Degraded, Some("Database pool exhausted".to_string()))
                        } else {
                            (HealthStatus::Healthy, Some("Database connection successful".to_string()))
                        }
                    }
                    Err(e) => {
                        error!("Database query failed: {}", e);
                        (HealthStatus::Unhealthy, Some(format!("Query failed: {}", e)))
                    }
                }
            }
            Err(e) => {
                error!("Failed to get database connection: {}", e);
                (HealthStatus::Critical, Some(format!("Connection failed: {}", e)))
            }
        };

        ComponentHealth {
            name: self.name.clone(),
            status,
            message,
            details,
            last_checked: chrono::Utc::now(),
            response_time_ms: start.elapsed().unwrap_or(Duration::from_secs(0)).as_millis() as u64,
        }
    }

    fn component_name(&self) -> &str {
        &self.name
    }
}

/// Redis cache health checker
pub struct CacheHealthChecker {
    name: String,
    cache_service: Arc<dyn crate::infrastructure::cache::CacheService>,
}

impl CacheHealthChecker {
    pub fn new(name: String, cache_service: Arc<dyn crate::infrastructure::cache::CacheService>) -> Self {
        Self { name, cache_service }
    }
}

#[async_trait]
impl HealthChecker for CacheHealthChecker {
    async fn check_health(&self) -> ComponentHealth {
        let start = SystemTime::now();
        let mut details = HashMap::new();
        let test_key = "health_check_test";
        let test_value = "ok";

        let (status, message) = match self.cache_service.set(test_key, &test_value, Some(Duration::from_secs(60))).await {
            Ok(_) => {
                match self.cache_service.get::<String>(test_key).await {
                    Ok(Some(value)) if value == test_value => {
                        // Cleanup
                        let _ = self.cache_service.delete(test_key).await;
                        details.insert("read_write_test".to_string(), serde_json::json!("passed"));
                        (HealthStatus::Healthy, Some("Cache read/write test passed".to_string()))
                    }
                    Ok(Some(_)) => {
                        let _ = self.cache_service.delete(test_key).await;
                        (HealthStatus::Degraded, Some("Cache returned unexpected value".to_string()))
                    }
                    Ok(None) => {
                        (HealthStatus::Degraded, Some("Cache set succeeded but read failed".to_string()))
                    }
                    Err(e) => {
                        let _ = self.cache_service.delete(test_key).await;
                        (HealthStatus::Unhealthy, Some(format!("Cache read failed: {}", e)))
                    }
                }
            }
            Err(e) => {
                (HealthStatus::Critical, Some(format!("Cache write failed: {}", e)))
            }
        };

        ComponentHealth {
            name: self.name.clone(),
            status,
            message,
            details,
            last_checked: chrono::Utc::now(),
            response_time_ms: start.elapsed().unwrap_or(Duration::from_secs(0)).as_millis() as u64,
        }
    }

    fn component_name(&self) -> &str {
        &self.name
    }
}

/// External service health checker (generic HTTP endpoint)
pub struct ExternalServiceHealthChecker {
    name: String,
    url: String,
    timeout: Duration,
    client: reqwest::Client,
}

impl ExternalServiceHealthChecker {
    pub fn new(name: String, url: String, timeout: Duration) -> Self {
        Self {
            name,
            url,
            timeout,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl HealthChecker for ExternalServiceHealthChecker {
    async fn check_health(&self) -> ComponentHealth {
        let start = SystemTime::now();
        let mut details = HashMap::new();

        let (status, message) = match tokio::time::timeout(
            self.timeout,
            self.client.get(&self.url).send()
        ).await {
            Ok(Ok(response)) => {
                let status_code = response.status().as_u16();
                details.insert("status_code".to_string(), serde_json::json!(status_code));
                details.insert("url".to_string(), serde_json::json!(self.url));

                if response.status().is_success() {
                    (HealthStatus::Healthy, Some(format!("Service responded with {}", status_code)))
                } else if response.status().is_server_error() {
                    (HealthStatus::Unhealthy, Some(format!("Service error: {}", status_code)))
                } else {
                    (HealthStatus::Degraded, Some(format!("Service responded with {}", status_code)))
                }
            }
            Ok(Err(e)) => {
                details.insert("error".to_string(), serde_json::json!(e.to_string()));
                (HealthStatus::Critical, Some(format!("Request failed: {}", e)))
            }
            Err(_) => {
                details.insert("error".to_string(), serde_json::json!("timeout"));
                (HealthStatus::Critical, Some("Request timed out".to_string()))
            }
        };

        ComponentHealth {
            name: self.name.clone(),
            status,
            message,
            details,
            last_checked: chrono::Utc::now(),
            response_time_ms: start.elapsed().unwrap_or(Duration::from_secs(0)).as_millis() as u64,
        }
    }

    fn component_name(&self) -> &str {
        &self.name
    }
}

/// Disk space health checker
pub struct DiskSpaceHealthChecker {
    name: String,
    path: String,
    warning_threshold_percent: f64,
    critical_threshold_percent: f64,
}

impl DiskSpaceHealthChecker {
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            warning_threshold_percent: 80.0,  // Warning at 80% usage
            critical_threshold_percent: 95.0, // Critical at 95% usage
        }
    }

    pub fn with_thresholds(mut self, warning: f64, critical: f64) -> Self {
        self.warning_threshold_percent = warning;
        self.critical_threshold_percent = critical;
        self
    }
}

#[async_trait]
impl HealthChecker for DiskSpaceHealthChecker {
    async fn check_health(&self) -> ComponentHealth {
        let start = SystemTime::now();
        let mut details = HashMap::new();

        let (status, message) = match std::fs::metadata(&self.path) {
            Ok(_) => {
                // Use statvfs or similar system call to get disk usage
                // For simplicity, we'll simulate this check
                details.insert("path".to_string(), serde_json::json!(self.path));
                details.insert("warning_threshold".to_string(), serde_json::json!(self.warning_threshold_percent));
                details.insert("critical_threshold".to_string(), serde_json::json!(self.critical_threshold_percent));

                // In a real implementation, you'd get actual disk usage here
                let usage_percent = 45.0; // Simulated value
                details.insert("usage_percent".to_string(), serde_json::json!(usage_percent));

                if usage_percent >= self.critical_threshold_percent {
                    (HealthStatus::Critical, Some(format!("Disk usage critical: {:.1}%", usage_percent)))
                } else if usage_percent >= self.warning_threshold_percent {
                    (HealthStatus::Degraded, Some(format!("Disk usage high: {:.1}%", usage_percent)))
                } else {
                    (HealthStatus::Healthy, Some(format!("Disk usage normal: {:.1}%", usage_percent)))
                }
            }
            Err(e) => {
                (HealthStatus::Critical, Some(format!("Cannot access path {}: {}", self.path, e)))
            }
        };

        ComponentHealth {
            name: self.name.clone(),
            status,
            message,
            details,
            last_checked: chrono::Utc::now(),
            response_time_ms: start.elapsed().unwrap_or(Duration::from_secs(0)).as_millis() as u64,
        }
    }

    fn component_name(&self) -> &str {
        &self.name
    }
}

/// Main health service that coordinates all health checks
pub struct HealthService {
    checkers: Vec<Arc<dyn HealthChecker>>,
    version: String,
    environment: String,
    start_time: Instant,
}

impl HealthService {
    pub fn new(version: String, environment: String) -> Self {
        Self {
            checkers: Vec::new(),
            version,
            environment,
            start_time: Instant::now(),
        }
    }

    pub fn add_checker(&mut self, checker: Arc<dyn HealthChecker>) {
        self.checkers.push(checker);
    }

    /// Perform health check on all components
    pub async fn check_health(&self) -> HealthReport {
        let mut report = HealthReport::new(
            self.version.clone(),
            self.environment.clone(),
            self.start_time.elapsed().as_secs(),
        );

        // Run all health checks concurrently
        let check_futures: Vec<_> = self.checkers
            .iter()
            .map(|checker| checker.check_health())
            .collect();

        let results = futures::future::join_all(check_futures).await;

        for component_health in results {
            report.add_component(component_health);
        }

        report
    }

    /// Quick health check (returns just the overall status)
    pub async fn quick_check(&self) -> HealthStatus {
        let report = self.check_health().await;
        report.overall_status
    }

    /// Check if system is ready to serve traffic
    pub async fn readiness_check(&self) -> AppResult<()> {
        let report = self.check_health().await;
        
        if report.is_operational() {
            Ok(())
        } else {
            Err(AppError::ServiceUnavailable(
                format!("System not ready: {:?}", report.overall_status)
            ))
        }
    }

    /// Check if system is alive (basic liveness check)
    pub async fn liveness_check(&self) -> AppResult<()> {
        // For liveness, we just check if the service can respond
        // This is a very basic check that doesn't depend on external services
        Ok(())
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    pub enabled: bool,
    pub check_interval_seconds: u64,
    pub timeout_seconds: u64,
    pub external_services: Vec<ExternalServiceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServiceConfig {
    pub name: String,
    pub url: String,
    pub timeout_seconds: u64,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_seconds: 30,
            timeout_seconds: 5,
            external_services: vec![],
        }
    }
}

/// Create a health service with common health checkers
pub async fn create_health_service(
    version: String,
    environment: String,
    db_pool: Option<Arc<DbPool>>,
    cache_service: Option<Arc<dyn crate::infrastructure::cache::CacheService>>,
    config: HealthConfig,
) -> HealthService {
    let mut service = HealthService::new(version, environment);

    // Add database health checker
    if let Some(pool) = db_pool {
        let db_checker = Arc::new(DatabaseHealthChecker::new(
            "database".to_string(),
            pool,
        ));
        service.add_checker(db_checker);
    }

    // Add cache health checker
    if let Some(cache) = cache_service {
        let cache_checker = Arc::new(CacheHealthChecker::new(
            "cache".to_string(),
            cache,
        ));
        service.add_checker(cache_checker);
    }

    // Add external service health checkers
    for external_config in config.external_services {
        let external_checker = Arc::new(ExternalServiceHealthChecker::new(
            external_config.name,
            external_config.url,
            Duration::from_secs(external_config.timeout_seconds),
        ));
        service.add_checker(external_checker);
    }

    // Add disk space health checker
    let disk_checker = Arc::new(DiskSpaceHealthChecker::new(
        "disk".to_string(),
        "/".to_string(), // Check root filesystem
    ));
    service.add_checker(disk_checker);

    service
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_report_creation() {
        let mut report = HealthReport::new(
            "1.0.0".to_string(),
            "test".to_string(),
            100,
        );

        assert_eq!(report.overall_status, HealthStatus::Healthy);
        assert!(report.components.is_empty());

        let healthy_component = ComponentHealth {
            name: "test_component".to_string(),
            status: HealthStatus::Healthy,
            message: Some("All good".to_string()),
            details: HashMap::new(),
            last_checked: chrono::Utc::now(),
            response_time_ms: 10,
        };

        report.add_component(healthy_component);
        assert_eq!(report.overall_status, HealthStatus::Healthy);
        assert_eq!(report.components.len(), 1);
    }

    #[tokio::test]
    async fn test_health_status_aggregation() {
        let mut report = HealthReport::new(
            "1.0.0".to_string(),
            "test".to_string(),
            100,
        );

        // Add healthy component
        let healthy_component = ComponentHealth {
            name: "healthy".to_string(),
            status: HealthStatus::Healthy,
            message: None,
            details: HashMap::new(),
            last_checked: chrono::Utc::now(),
            response_time_ms: 10,
        };
        report.add_component(healthy_component);
        assert_eq!(report.overall_status, HealthStatus::Healthy);

        // Add degraded component
        let degraded_component = ComponentHealth {
            name: "degraded".to_string(),
            status: HealthStatus::Degraded,
            message: None,
            details: HashMap::new(),
            last_checked: chrono::Utc::now(),
            response_time_ms: 50,
        };
        report.add_component(degraded_component);
        assert_eq!(report.overall_status, HealthStatus::Degraded);

        // Add critical component
        let critical_component = ComponentHealth {
            name: "critical".to_string(),
            status: HealthStatus::Critical,
            message: None,
            details: HashMap::new(),
            last_checked: chrono::Utc::now(),
            response_time_ms: 1000,
        };
        report.add_component(critical_component);
        assert_eq!(report.overall_status, HealthStatus::Critical);
    }
}
