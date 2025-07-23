// Application handlers - coordinate between web layer and application services
// These handlers process HTTP requests and delegate to appropriate services

use actix_web::{web, HttpResponse};
use crate::shared::error::AppResult;
use crate::application::services::*;
use chrono::Utc;
use serde_json::json;

/// Emergency response handlers
pub mod emergency;
/// User management handlers  
pub mod user;
/// Authentication handlers
pub mod auth;
/// Notification handlers
pub mod notification;
/// Analytics and reporting handlers
pub mod analytics;
/// Map and location handlers
pub mod map;

/// Health check handler
pub async fn health_check() -> AppResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "service": "terra-siaga",
        "version": "1.0.0"
    })))
}
