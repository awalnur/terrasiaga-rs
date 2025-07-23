/// Disaster management API endpoints
/// Handles disaster reporting, tracking, and response coordination

use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use crate::infrastructure::AppContainer;

#[derive(Debug, Deserialize)]
pub struct CreateDisasterRequest {
    pub title: String,
    pub description: String,
    pub disaster_type: String, // earthquake, flood, fire, etc.
    pub severity: String,      // low, medium, high, critical
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub affected_population: Option<u32>,
    pub images: Option<Vec<String>>,
    pub contact_info: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDisasterRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>, // reported, verified, responding, resolved
    pub affected_population: Option<u32>,
    pub response_notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DisasterSearchQuery {
    pub status: Option<String>,
    pub severity: Option<String>,
    pub disaster_type: Option<String>,
    pub location: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub radius: Option<f64>, // km radius for location search
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct AssignResponderRequest {
    pub responder_id: String,
    pub priority: Option<String>, // low, medium, high
    pub notes: Option<String>,
}

/// POST /api/v1/disasters
async fn create_disaster_report(
    req: web::Json<CreateDisasterRequest>,
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    // TODO: Implement create disaster report logic
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Disaster report created successfully",
        "title": req.title,
        "severity": req.severity
    })))
}

/// GET /api/v1/disasters
async fn list_disasters(
    query: web::Query<DisasterSearchQuery>,
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    // TODO: Implement disaster listing with filters and pagination
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Disasters list",
        "filters": query.into_inner()
    })))
}

/// GET /api/v1/disasters/{disaster_id}
async fn get_disaster_by_id(
    path: web::Path<String>,
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    let disaster_id = path.into_inner();
    // TODO: Implement get disaster by ID logic
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Disaster details",
        "disaster_id": disaster_id
    })))
}

/// PUT /api/v1/disasters/{disaster_id}
async fn update_disaster(
    path: web::Path<String>,
    req: web::Json<UpdateDisasterRequest>,
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    let disaster_id = path.into_inner();
    // TODO: Implement update disaster logic
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Disaster updated successfully",
        "disaster_id": disaster_id
    })))
}

/// DELETE /api/v1/disasters/{disaster_id}
async fn delete_disaster(
    path: web::Path<String>,
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    let disaster_id = path.into_inner();
    // TODO: Implement delete disaster logic (admin only)
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Disaster deleted",
        "disaster_id": disaster_id
    })))
}

/// POST /api/v1/disasters/{disaster_id}/assign
async fn assign_responder(
    path: web::Path<String>,
    req: web::Json<AssignResponderRequest>,
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    let disaster_id = path.into_inner();
    // TODO: Implement assign responder logic
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Responder assigned to disaster",
        "disaster_id": disaster_id,
        "responder_id": req.responder_id
    })))
}

/// POST /api/v1/disasters/{disaster_id}/verify
async fn verify_disaster(
    path: web::Path<String>,
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    let disaster_id = path.into_inner();
    // TODO: Implement verify disaster logic (responder/admin only)
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Disaster verified",
        "disaster_id": disaster_id
    })))
}

/// POST /api/v1/disasters/{disaster_id}/resolve
async fn resolve_disaster(
    path: web::Path<String>,
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    let disaster_id = path.into_inner();
    // TODO: Implement resolve disaster logic
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Disaster resolved",
        "disaster_id": disaster_id
    })))
}

/// GET /api/v1/disasters/{disaster_id}/responders
async fn get_disaster_responders(
    path: web::Path<String>,
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    let disaster_id = path.into_inner();
    // TODO: Implement get assigned responders
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Disaster responders",
        "disaster_id": disaster_id
    })))
}

/// GET /api/v1/disasters/{disaster_id}/timeline
async fn get_disaster_timeline(
    path: web::Path<String>,
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    let disaster_id = path.into_inner();
    // TODO: Implement get disaster timeline/history
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Disaster timeline",
        "disaster_id": disaster_id
    })))
}

/// GET /api/v1/disasters/nearby
async fn get_nearby_disasters(
    query: web::Query<DisasterSearchQuery>,
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    // TODO: Implement get nearby disasters based on coordinates
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Nearby disasters",
        "coordinates": {
            "lat": query.lat,
            "lng": query.lng,
            "radius": query.radius.unwrap_or(10.0)
        }
    })))
}

/// GET /api/v1/disasters/active
async fn get_active_disasters(
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    // TODO: Implement get active disasters
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Active disasters"
    })))
}

/// GET /api/v1/disasters/stats
async fn get_disaster_statistics(
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    // TODO: Implement get disaster statistics
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Disaster statistics",
        "total_reports": 0,
        "active_disasters": 0,
        "resolved_disasters": 0,
        "by_severity": {
            "low": 0,
            "medium": 0,
            "high": 0,
            "critical": 0
        }
    })))
}

pub fn configure_disaster_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("", web::post().to(create_disaster_report))
        .route("", web::get().to(list_disasters))
        .route("/{disaster_id}", web::get().to(get_disaster_by_id))
        .route("/{disaster_id}", web::put().to(update_disaster))
        .route("/{disaster_id}", web::delete().to(delete_disaster))
        .route("/{disaster_id}/assign", web::post().to(assign_responder))
        .route("/{disaster_id}/verify", web::post().to(verify_disaster))
        .route("/{disaster_id}/resolve", web::post().to(resolve_disaster))
        .route("/{disaster_id}/responders", web::get().to(get_disaster_responders))
        .route("/{disaster_id}/timeline", web::get().to(get_disaster_timeline))
        .route("/nearby", web::get().to(get_nearby_disasters))
        .route("/active", web::get().to(get_active_disasters))
        .route("/stats", web::get().to(get_disaster_statistics));
}
