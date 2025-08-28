/// Health check endpoints
/// Provides system health and readiness checks

use actix_web::{get, web, HttpResponse, Result};
use serde_json::json;
use chrono::Utc;
use std::sync::Arc;

use crate::infrastructure::monitoring::{HealthMonitoringService, HealthStatus};

#[tracing::instrument(name = "health_check", skip(monitoring_service))]
#[get("/health")]
pub async fn health_check(
    monitoring_service: web::Data<Arc<HealthMonitoringService>>,
) -> Result<HttpResponse> {
    println!("Health check");
    // Perform async health check across components
    let health = monitoring_service.check_health().await;

    let status_code = match health.overall_status {
        HealthStatus::Healthy => 200,
        HealthStatus::Degraded => 200,
        HealthStatus::Unhealthy => 503,
        HealthStatus::Critical => 503,
    };

    println!("{:?}", health);
    Ok(HttpResponse::build(actix_web::http::StatusCode::from_u16(status_code).unwrap())
        .json(json!({
            "status": health.overall_status,
            "timestamp": health.timestamp,
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
    // readiness_check returns AppResult; Ok means ready
    match monitoring_service.readiness_check().await {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "status": "ready",
            "timestamp": Utc::now(),
            "message": "Service is ready to accept traffic"
        }))),
        Err(_e) => Ok(HttpResponse::ServiceUnavailable().json(json!({
            "status": "not_ready",
            "timestamp": Utc::now(),
            "message": "Service is not ready to accept traffic"
        }).to_string())),
    }
}

#[get("/live")]
pub async fn liveness_check(
    monitoring_service: web::Data<Arc<HealthMonitoringService>>,
) -> Result<HttpResponse> {
    match monitoring_service.liveness_check().await {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "status": "alive",
            "timestamp": Utc::now(),
            "message": "Service is alive"
        }))),
        Err(_e) => Ok(HttpResponse::ServiceUnavailable().json(json!({
            "status": "dead",
            "timestamp": Utc::now(),
            "message": "Service is not responding"
        }))),
    }
}

#[get("/metrics")]
pub async fn metrics_endpoint(
    monitoring_service: web::Data<Arc<HealthMonitoringService>>,
) -> Result<HttpResponse> {
    // Provide a lightweight status metric snapshot
    let status = monitoring_service.quick_check().await;

    Ok(HttpResponse::Ok().json(json!({
        "status": status,
        "collected_at": Utc::now()
    })))
}
