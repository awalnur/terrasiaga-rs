/// Comprehensive monitoring infrastructure for Terra Siaga
/// Provides health monitoring, metrics collection, and observability

use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::shared::error::{AppResult, AppError};
use crate::infrastructure::database::{DatabaseService, DatabaseHealth};
use crate::infrastructure::cache::CacheService;
use crate::infrastructure::external_services::{ExternalServicesManager, ExternalServicesHealth};

/// Application health monitor with comprehensive status tracking
pub struct HealthMonitor {
    database: Arc<DatabaseService>,
    cache: Arc<CacheService>,
    external_services: Arc<tokio::sync::Mutex<ExternalServicesManager>>,
    metrics: Arc<RwLock<ApplicationMetrics>>,
    start_time: Instant,
}

impl HealthMonitor {
    /// Create new health monitor
    pub fn new(
        database: Arc<DatabaseService>,
        cache: Arc<CacheService>,
        external_services: Arc<tokio::sync::Mutex<ExternalServicesManager>>,
    ) -> Self {
        Self {
            database,
            cache,
            external_services,
            metrics: Arc::new(RwLock::new(ApplicationMetrics::new())),
            start_time: Instant::now(),
        }
    }

    /// Perform comprehensive health check
    pub async fn health_check(&self) -> AppResult<ApplicationHealth> {
        let start = Instant::now();

        // Check database health
        let database_health = self.database.health_check().await?;
        
        // Check cache health
        let cache_stats = self.cache.stats().await?;
        let cache_health = CacheHealth {
            status: if cache_stats.hit_ratio > 0.5 { 
                HealthStatus::Healthy 
            } else { 
                HealthStatus::Degraded 
            },
            hit_ratio: cache_stats.hit_ratio,
            entries: cache_stats.entries,
            memory_usage: cache_stats.memory_usage,
        };

        // Check external services health
        let external_health = {
            let mut services = self.external_services.lock().await;
            services.health_check().await?
        };

        // Calculate overall status
        let overall_status = self.calculate_overall_status(
            &database_health.status,
            &cache_health.status,
            &external_health.overall_status,
        );

        let check_duration = start.elapsed();

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.update_health_check(check_duration, overall_status.clone());
        }

        Ok(ApplicationHealth {
            status: overall_status,
            timestamp: chrono::Utc::now(),
            uptime: self.start_time.elapsed(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database: database_health,
            cache: cache_health,
            external_services: external_health,
            metrics: self.get_current_metrics().await,
        })
    }

    /// Get current application metrics
    pub async fn get_current_metrics(&self) -> ApplicationMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Record request metrics
    pub async fn record_request(&self, method: &str, path: &str, status_code: u16, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.record_request(method, path, status_code, duration);
    }

    /// Record error
    pub async fn record_error(&self, error_type: &str, context: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.record_error(error_type, context);
    }

    /// Get detailed system information
    pub async fn get_system_info(&self) -> SystemInfo {
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: self.start_time.elapsed(),
            memory_usage: self.get_memory_usage(),
        }
    }

    fn calculate_overall_status(
        &self,
        db_status: &crate::infrastructure::database::HealthStatus,
        cache_status: &HealthStatus,
        external_status: &crate::infrastructure::external_services::ServiceStatus,
    ) -> HealthStatus {
        use crate::infrastructure::database::HealthStatus as DbHealthStatus;
        use crate::infrastructure::external_services::ServiceStatus;

        let db_healthy = matches!(db_status, DbHealthStatus::Healthy);
        let cache_healthy = matches!(cache_status, HealthStatus::Healthy);
        let external_healthy = matches!(external_status, ServiceStatus::Healthy);

        if db_healthy && cache_healthy && external_healthy {
            HealthStatus::Healthy
        } else if db_healthy && (cache_healthy || external_healthy) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        }
    }

    fn get_memory_usage(&self) -> MemoryUsage {
        // Basic memory usage info - in production, you might want to use a more sophisticated approach
        MemoryUsage {
            used_mb: 0, // Would need system-specific implementation
            total_mb: 0,
            percentage: 0.0,
        }
    }
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Complete application health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationHealth {
    pub status: HealthStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime: Duration,
    pub version: String,
    pub database: DatabaseHealth,
    pub cache: CacheHealth,
    pub external_services: ExternalServicesHealth,
    pub metrics: ApplicationMetrics,
}

/// Cache health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHealth {
    pub status: HealthStatus,
    pub hit_ratio: f64,
    pub entries: u64,
    pub memory_usage: u64,
}

/// Application metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    pub requests: RequestMetrics,
    pub errors: ErrorMetrics,
    pub performance: PerformanceMetrics,
    pub business: BusinessMetrics,
}

impl ApplicationMetrics {
    pub fn new() -> Self {
        Self {
            requests: RequestMetrics::new(),
            errors: ErrorMetrics::new(),
            performance: PerformanceMetrics::new(),
            business: BusinessMetrics::new(),
        }
    }

    pub fn update_health_check(&mut self, duration: Duration, status: HealthStatus) {
        self.performance.health_check_duration = duration;
        self.performance.last_health_check = chrono::Utc::now();
        
        match status {
            HealthStatus::Healthy => self.performance.healthy_checks += 1,
            HealthStatus::Degraded => self.performance.degraded_checks += 1,
            HealthStatus::Unhealthy => self.performance.unhealthy_checks += 1,
        }
    }

    pub fn record_request(&mut self, method: &str, path: &str, status_code: u16, duration: Duration) {
        self.requests.total_requests += 1;
        
        match status_code {
            200..=299 => self.requests.successful_requests += 1,
            400..=499 => self.requests.client_errors += 1,
            500..=599 => self.requests.server_errors += 1,
            _ => {}
        }

        // Update response time metrics
        let duration_ms = duration.as_millis() as f64;
        self.performance.update_response_time(duration_ms);

        // Track endpoint metrics
        let endpoint_key = format!("{} {}", method, path);
        let endpoint_metrics = self.requests.endpoints
            .entry(endpoint_key)
            .or_insert_with(EndpointMetrics::new);
        endpoint_metrics.total_requests += 1;
        endpoint_metrics.total_duration += duration;
        endpoint_metrics.avg_duration = endpoint_metrics.total_duration / endpoint_metrics.total_requests;
    }

    pub fn record_error(&mut self, error_type: &str, context: &str) {
        self.errors.total_errors += 1;
        
        let error_count = self.errors.error_types
            .entry(error_type.to_string())
            .or_insert(0);
        *error_count += 1;

        self.errors.recent_errors.push(ErrorEvent {
            error_type: error_type.to_string(),
            context: context.to_string(),
            timestamp: chrono::Utc::now(),
        });

        // Keep only last 100 errors
        if self.errors.recent_errors.len() > 100 {
            self.errors.recent_errors.remove(0);
        }
    }
}

/// Request metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub client_errors: u64,
    pub server_errors: u64,
    pub endpoints: HashMap<String, EndpointMetrics>,
}

impl RequestMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            client_errors: 0,
            server_errors: 0,
            endpoints: HashMap::new(),
        }
    }
}

/// Per-endpoint metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub total_requests: u32,
    pub total_duration: Duration,
    pub avg_duration: Duration,
}

impl EndpointMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: 0,
            total_duration: Duration::from_millis(0),
            avg_duration: Duration::from_millis(0),
        }
    }
}

/// Error metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub error_types: HashMap<String, u64>,
    pub recent_errors: Vec<ErrorEvent>,
}

impl ErrorMetrics {
    pub fn new() -> Self {
        Self {
            total_errors: 0,
            error_types: HashMap::new(),
            recent_errors: Vec::new(),
        }
    }
}

/// Error event for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub error_type: String,
    pub context: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_response_time_ms: f64,
    pub min_response_time_ms: f64,
    pub max_response_time_ms: f64,
    pub p95_response_time_ms: f64,
    pub health_check_duration: Duration,
    pub last_health_check: chrono::DateTime<chrono::Utc>,
    pub healthy_checks: u64,
    pub degraded_checks: u64,
    pub unhealthy_checks: u64,
    response_times: Vec<f64>, // Keep for percentile calculation
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            avg_response_time_ms: 0.0,
            min_response_time_ms: f64::MAX,
            max_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            health_check_duration: Duration::from_millis(0),
            last_health_check: chrono::Utc::now(),
            healthy_checks: 0,
            degraded_checks: 0,
            unhealthy_checks: 0,
            response_times: Vec::new(),
        }
    }

    pub fn update_response_time(&mut self, duration_ms: f64) {
        self.response_times.push(duration_ms);
        
        // Keep only last 1000 response times for percentile calculation
        if self.response_times.len() > 1000 {
            self.response_times.remove(0);
        }

        // Update min/max
        self.min_response_time_ms = self.min_response_time_ms.min(duration_ms);
        self.max_response_time_ms = self.max_response_time_ms.max(duration_ms);

        // Calculate average
        let sum: f64 = self.response_times.iter().sum();
        self.avg_response_time_ms = sum / self.response_times.len() as f64;

        // Calculate P95
        if !self.response_times.is_empty() {
            let mut sorted_times = self.response_times.clone();
            sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
            if p95_index < sorted_times.len() {
                self.p95_response_time_ms = sorted_times[p95_index];
            }
        }
    }
}

/// Business metrics specific to Terra Siaga
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub active_emergencies: u64,
    pub total_users: u64,
    pub notifications_sent_today: u64,
    pub avg_response_time_minutes: f64,
}

impl BusinessMetrics {
    pub fn new() -> Self {
        Self {
            active_emergencies: 0,
            total_users: 0,
            notifications_sent_today: 0,
            avg_response_time_minutes: 0.0,
        }
    }
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub rust_version: String,
    pub app_version: String,
    pub uptime: Duration,
    pub memory_usage: MemoryUsage,
}

/// Memory usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub used_mb: u64,
    pub total_mb: u64,
    pub percentage: f64,
}

/// Middleware for automatic metrics collection
pub struct MetricsMiddleware {
    health_monitor: Arc<HealthMonitor>,
}

impl MetricsMiddleware {
    pub fn new(health_monitor: Arc<HealthMonitor>) -> Self {
        Self { health_monitor }
    }

    pub async fn record_request_metrics(
        &self,
        method: &str,
        path: &str,
        status_code: u16,
        duration: Duration,
    ) {
        self.health_monitor
            .record_request(method, path, status_code, duration)
            .await;
    }
}

/// Health check scheduler for periodic monitoring
pub struct HealthCheckScheduler {
    health_monitor: Arc<HealthMonitor>,
    interval: Duration,
}

impl HealthCheckScheduler {
    pub fn new(health_monitor: Arc<HealthMonitor>, interval: Duration) -> Self {
        Self {
            health_monitor,
            interval,
        }
    }

    /// Start periodic health checks
    pub async fn start(&self) {
        let mut interval_timer = tokio::time::interval(self.interval);
        
        loop {
            interval_timer.tick().await;
            
            match self.health_monitor.health_check().await {
                Ok(health) => {
                    tracing::info!("Health check completed: status={:?}", health.status);
                    
                    if matches!(health.status, HealthStatus::Unhealthy) {
                        tracing::error!("Application is unhealthy: {:?}", health);
                    }
                }
                Err(e) => {
                    tracing::error!("Health check failed: {}", e);
                }
            }
        }
    }
}
