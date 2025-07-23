use actix_web::{web, HttpResponse};
use uuid::Uuid;
use crate::shared::error::AppResult;
use crate::application::services::emergency::EmergencyService;
use crate::domain::entities::disaster::CreateDisasterRequest;

/// Create new emergency report
pub async fn create_emergency(
    emergency_service: web::Data<EmergencyService>,
    req: web::Json<CreateDisasterRequest>,
) -> AppResult<HttpResponse> {
    let disaster = emergency_service.create_disaster(req.into_inner()).await?;
    Ok(HttpResponse::Created().json(disaster))
}

/// Get emergency by ID
pub async fn get_emergency(
    emergency_service: web::Data<EmergencyService>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let disaster_id = path.into_inner();
    let disaster = emergency_service.get_disaster_by_id(disaster_id).await?;
    Ok(HttpResponse::Ok().json(disaster))
}

/// List all emergencies with pagination
pub async fn list_emergencies(
    emergency_service: web::Data<EmergencyService>,
    query: web::Query<serde_json::Value>,
) -> AppResult<HttpResponse> {
    let disasters = emergency_service.list_disasters(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(disasters))
}

/// Update emergency status
pub async fn update_emergency_status(
    emergency_service: web::Data<EmergencyService>,
    path: web::Path<Uuid>,
    req: web::Json<serde_json::Value>,
) -> AppResult<HttpResponse> {
    let disaster_id = path.into_inner();
    let disaster = emergency_service.update_disaster_status(disaster_id, req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(disaster))
}

/// Delete emergency
pub async fn delete_emergency(
    emergency_service: web::Data<EmergencyService>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let disaster_id = path.into_inner();
    emergency_service.delete_disaster(disaster_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
