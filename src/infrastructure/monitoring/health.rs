/// Health check and monitoring system for Terra Siaga
/// Provides comprehensive health monitoring for all system components

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::infrastructure::cache::CacheService;
use crate::shared::{AppResult, AppError};

/// Health status levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Individual component health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub component: String,
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub message: Option<String>,
    pub last_checked: DateTime<Utc>,
    pub details: HashMap<String, serde_json::Value>,
}

/// System-wide health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: HealthStatus,
    pub components: Vec<ComponentHealth>,
    pub checked_at: DateTime<Utc>,
    pub version: String,
    pub uptime_seconds: u64,
    pub environment: String,
}

/// Health check trait for individual components
#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> ComponentHealth;
    fn component_name(&self) -> &str;
}

/// Database health check
pub struct DatabaseHealthCheck {
    pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
}

impl DatabaseHealthCheck {
    pub fn new(pool: Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();
        let checked_at = Utc::now();
        
        match self.pool.get() {
            Ok(mut conn) => {
                // Test with a simple query
                match diesel::sql_query("SELECT 1").execute(&mut conn) {
                    Ok(_) => {
                        let response_time = start.elapsed().as_millis() as u64;
                        let mut details = HashMap::new();
                        details.insert("pool_size".to_string(), 
                                     serde_json::Value::Number(serde_json::Number::from(self.pool.state().connections as u64)));
                        details.insert("idle_connections".to_string(), 
                                     serde_json::Value::Number(serde_json::Number::from(self.pool.state().idle_connections as u64)));
                        
                        ComponentHealth {
                            component: self.component_name().to_string(),
                            status: if response_time > 1000 { HealthStatus::Degraded } else { HealthStatus::Healthy },
                            response_time_ms: response_time,
                            message: Some("Database connection successful".to_string()),
                            last_checked: checked_at,
                            details,
                        }
                    }
                    Err(e) => ComponentHealth {
                        component: self.component_name().to_string(),
                        status: HealthStatus::Unhealthy,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        message: Some(format!("Database query failed: {}", e)),
                        last_checked: checked_at,
                        details: HashMap::new(),
                    }
                }
            }
            Err(e) => ComponentHealth {
                component: self.component_name().to_string(),
                status: HealthStatus::Unhealthy,
                response_time_ms: start.elapsed().as_millis() as u64,
                message: Some(format!("Failed to get database connection: {}", e)),
                last_checked: checked_at,
                details: HashMap::new(),
            }
        }
    }

    fn component_name(&self) -> &str {
        "database"
    }
}

/// Redis cache health check
pub struct CacheHealthCheck {
    cache: Arc<dyn CacheService>,
}

impl CacheHealthCheck {
    pub fn new(cache: Arc<dyn CacheService>) -> Self {
        Self { cache }
    }
}

#[async_trait]
impl HealthCheck for CacheHealthCheck {
    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();
        let checked_at = Utc::now();
        let test_key = format!("health_check_{}", Uuid::new_v4());
        let test_value = "health_check_value";

        // Test cache operations
        match self.cache.set(&test_key, &test_value, Some(Duration::from_secs(60))).await {
            Ok(_) => {
                match self.cache.get::<String>(&test_key).await {
                    Ok(Some(value)) if value == test_value => {
                        let _ = self.cache.delete(&test_key).await;
                        let response_time = start.elapsed().as_millis() as u64;
                        
                        ComponentHealth {
                            component: self.component_name().to_string(),
                            status: if response_time > 500 { HealthStatus::Degraded } else { HealthStatus::Healthy },
                            response_time_ms: response_time,
                            message: Some("Cache operations successful".to_string()),
                            last_checked: checked_at,
                            details: HashMap::new(),
                        }
                    }
                    Ok(Some(_)) => ComponentHealth {
                        component: self.component_name().to_string(),
                        status: HealthStatus::Unhealthy,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        message: Some("Cache returned incorrect value".to_string()),
                        last_checked: checked_at,
                        details: HashMap::new(),
                    },
                    Ok(None) => ComponentHealth {
                        component: self.component_name().to_string(),
                        status: HealthStatus::Unhealthy,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        message: Some("Cache failed to retrieve value".to_string()),
                        last_checked: checked_at,
                        details: HashMap::new(),
                    },
                    Err(e) => ComponentHealth {
                        component: self.component_name().to_string(),
                        status: HealthStatus::Unhealthy,
                        response_time_ms: start.elapsed().as_millis() as u64,
                        message: Some(format!("Cache get failed: {}", e)),
                        last_checked: checked_at,
                        details: HashMap::new(),
                    }
                }
            }
            Err(e) => ComponentHealth {
                component: self.component_name().to_string(),
                status: HealthStatus::Unhealthy,
                response_time_ms: start.elapsed().as_millis() as u64,
                message: Some(format!("Cache set failed: {}", e)),
                last_checked: checked_at,
                details: HashMap::new(),
            }
        }
    }

    fn component_name(&self) -> &str {
        "cache"
    }
}

/// External API health check
pub struct ExternalApiHealthCheck {
    api_name: String,
    endpoint_url: String,
    timeout: Duration,
    client: reqwest::Client,
}

impl ExternalApiHealthCheck {
    pub fn new(api_name: String, endpoint_url: String, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            api_name,
            endpoint_url,
            timeout,
            client,
        }
    }
}

#[async_trait]
impl HealthCheck for ExternalApiHealthCheck {
    async fn check(&self) -> ComponentHealth {
        let start = Instant::now();
        let checked_at = Utc::now();

        match self.client.get(&self.endpoint_url).send().await {
            Ok(response) => {
                let response_time = start.elapsed().as_millis() as u64;
                let status_code = response.status().as_u16();
                
                let mut details = HashMap::new();
                details.insert("status_code".to_string(), 
                             serde_json::Value::Number(serde_json::Number::from(status_code)));
                details.insert("endpoint".to_string(), 
                             serde_json::Value::String(self.endpoint_url.clone()));

                let status = if response.status().is_success() {
                    if response_time > 2000 { HealthStatus::Degraded } else { HealthStatus::Healthy }
                } else if response.status().is_server_error() {
                    HealthStatus::Unhealthy
                } else {
                    HealthStatus::Degraded
                };

                ComponentHealth {
                    component: self.component_name().to_string(),
                    status,
                    response_time_ms: response_time,
                    message: Some(format!("HTTP {} from {}", status_code, self.api_name)),
                    last_checked: checked_at,
                    details,
                }
            }
            Err(e) => {
                let mut details = HashMap::new();
                details.insert("endpoint".to_string(), 
                             serde_json::Value::String(self.endpoint_url.clone()));
                details.insert("error".to_string(), 
                             serde_json::Value::String(e.to_string()));

                ComponentHealth {
                    component: self.component_name().to_string(),
                    status: HealthStatus::Unhealthy,
                    response_time_ms: start.elapsed().as_millis() as u64,
                    message: Some(format!("Failed to connect to {}: {}", self.api_name, e)),
                    last_checked: checked_at,
                    details,
                }
            }
        }
    }

    fn component_name(&self) -> &str {
        &self.api_name
    }
}

/// System metrics collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub network_connections: u32,
    pub active_requests: u32,
    pub error_rate_percent: f64,
    pub response_time_p95_ms: u64,
    pub collected_at: DateTime<Utc>,
}

/// Health monitoring service
pub struct HealthMonitoringService {
    checks: Vec<Arc<dyn HealthCheck>>,
    start_time: Instant,
    version: String,
    environment: String,
}

impl HealthMonitoringService {
    pub fn new(version: String, environment: String) -> Self {
        Self {
            checks: Vec::new(),
            start_time: Instant::now(),
            version,
            environment,
        }
    }

    /// Add a health check component
    pub fn add_check(&mut self, check: Arc<dyn HealthCheck>) {
        self.checks.push(check);
    }

    /// Perform all health checks
    pub async fn check_health(&self) -> SystemHealth {
        let mut component_healths = Vec::new();
        
        // Run all health checks concurrently
        let check_futures = self.checks.iter().map(|check| check.check());
        let results = futures::future::join_all(check_futures).await;
        
        component_healths.extend(results);

        // Determine overall status
        let overall_status = self.determine_overall_status(&component_healths);
        
        SystemHealth {
            overall_status,
            components: component_healths,
            checked_at: Utc::now(),
            version: self.version.clone(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            environment: self.environment.clone(),
        }
    }

    /// Determine overall system health based on component health
    fn determine_overall_status(&self, components: &[ComponentHealth]) -> HealthStatus {
        if components.is_empty() {
            return HealthStatus::Unknown;
        }

        let unhealthy_count = components.iter()
            .filter(|c| c.status == HealthStatus::Unhealthy)
            .count();
        
        let degraded_count = components.iter()
            .filter(|c| c.status == HealthStatus::Degraded)
            .count();

        let critical_components = ["database", "cache"];
        let critical_unhealthy = components.iter()
            .any(|c| critical_components.contains(&c.component.as_str()) && 
                    c.status == HealthStatus::Unhealthy);

        if critical_unhealthy || unhealthy_count > components.len() / 2 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 || unhealthy_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get system metrics (simplified implementation)
    pub async fn get_metrics(&self) -> SystemMetrics {
        // In a real implementation, you'd collect actual system metrics
        // using libraries like `sysinfo` or integrating with monitoring tools
        
        SystemMetrics {
            memory_usage_mb: 512, // Mock data
            cpu_usage_percent: 25.0,
            disk_usage_percent: 45.0,
            network_connections: 150,
            active_requests: 10,
            error_rate_percent: 0.5,
            response_time_p95_ms: 150,
            collected_at: Utc::now(),
        }
    }

    /// Check if system is ready to serve traffic
    pub async fn is_ready(&self) -> bool {
        let health = self.check_health().await;
        
        // System is ready if critical components are healthy
        let critical_components = ["database"];
        
        for component in &health.components {
            if critical_components.contains(&component.component.as_str()) {
                if component.status == HealthStatus::Unhealthy {
                    return false;
                }
            }
        }
        
        true
    }

    /// Check if system is alive (basic liveness check)
    pub async fn is_alive(&self) -> bool {
        // Simple liveness check - if we can respond, we're alive
        true
    }
}

/// Performance metrics tracking
pub struct PerformanceTracker {
    request_times: Vec<Duration>,
    error_count: std::sync::atomic::AtomicU64,
    total_requests: std::sync::atomic::AtomicU64,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            request_times: Vec::new(),
            error_count: std::sync::atomic::AtomicU64::new(0),
            total_requests: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub fn record_request(&mut self, duration: Duration) {
        self.request_times.push(duration);
        self.total_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Keep only recent measurements (last 1000)
        if self.request_times.len() > 1000 {
            self.request_times.drain(0..100);
        }
    }

    pub fn record_error(&self) {
        self.error_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_p95_response_time(&self) -> Duration {
        if self.request_times.is_empty() {
            return Duration::from_millis(0);
        }

        let mut sorted_times = self.request_times.clone();
        sorted_times.sort();
        
        let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
        sorted_times.get(p95_index).copied().unwrap_or_default()
    }

    pub fn get_error_rate(&self) -> f64 {
        let total = self.total_requests.load(std::sync::atomic::Ordering::Relaxed);
        let errors = self.error_count.load(std::sync::atomic::Ordering::Relaxed);
        
        if total == 0 {
            0.0
        } else {
            (errors as f64 / total as f64) * 100.0
        }
    }
}
