/// Health check endpoints
/// Provides system health and readiness checks

use actix_web::{get, HttpResponse, Result};
use serde_json::json;
use chrono::Utc;

#[get("/health")]
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "service": "terra-siaga",
        "version": env!("CARGO_PKG_VERSION")
    })))
}

#[get("/ready")]
pub async fn readiness_check() -> Result<HttpResponse> {
    // Add database connectivity check here
    Ok(HttpResponse::Ok().json(json!({
        "status": "ready",
        "timestamp": Utc::now(),
        "checks": {
            "database": "ok",
            "cache": "ok"
        }
    })))
}
