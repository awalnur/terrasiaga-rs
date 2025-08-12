/// Health check endpoints
/// Provides system health and readiness checks

use actix_web::{get, web, HttpResponse, Result};
use serde_json::json;
use chrono::Utc;
use std::sync::Arc;

use crate::infrastructure::monitoring::HealthMonitoringService;

#[get("/health")]
pub async fn health_check(
    monitoring_service: web::Data<Arc<HealthMonitoringService>>,
) -> Result<HttpResponse> {
    let health = monitoring_service.check_health().await;

    let status_code = match health.overall_status {
        crate::infrastructure::monitoring::HealthStatus::Healthy => 200,
        crate::infrastructure::monitoring::HealthStatus::Degraded => 200,
        crate::infrastructure::monitoring::HealthStatus::Unhealthy => 503,
        crate::infrastructure::monitoring::HealthStatus::Unknown => 503,
    };

    Ok(HttpResponse::build(actix_web::http::StatusCode::from_u16(status_code).unwrap())
        .json(json!({
            "status": health.overall_status,
            "timestamp": health.checked_at,
            "service": "terra-siaga",
            "version": health.version,
            "uptime_seconds": health.uptime_seconds,
            "environment": health.environment,
            "components": health.components
        })))
}

#[get("/ready")]
pub async fn readiness_check(
    monitoring_service: web::Data<Arc<HealthMonitoringService>>,
) -> Result<HttpResponse> {
    let is_ready = monitoring_service.is_ready().await;

    if is_ready {
        Ok(HttpResponse::Ok().json(json!({
            "status": "ready",
            "timestamp": Utc::now(),
            "message": "Service is ready to accept traffic"
        })))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(json!({
            "status": "not_ready",
            "timestamp": Utc::now(),
            "message": "Service is not ready to accept traffic"
        })))
    }
}

#[get("/live")]
pub async fn liveness_check(
    monitoring_service: web::Data<Arc<HealthMonitoringService>>,
) -> Result<HttpResponse> {
    let is_alive = monitoring_service.is_alive().await;

    if is_alive {
        Ok(HttpResponse::Ok().json(json!({
            "status": "alive",
            "timestamp": Utc::now(),
            "message": "Service is alive"
        })))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(json!({
            "status": "dead",
            "timestamp": Utc::now(),
            "message": "Service is not responding"
        })))
    }
}

#[get("/metrics")]
pub async fn metrics_endpoint(
    monitoring_service: web::Data<Arc<HealthMonitoringService>>,
) -> Result<HttpResponse> {
    let metrics = monitoring_service.get_metrics().await;

    Ok(HttpResponse::Ok().json(json!({
        "metrics": metrics,
        "collected_at": metrics.collected_at
    })))
}
