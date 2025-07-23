use actix_web::{web, HttpResponse};
use uuid::Uuid;
use crate::shared::error::AppResult;
use crate::application::services::map::MapService;
use crate::domain::entities::map::{LocationFilter, MapBounds};

/// Get disaster locations on map
pub async fn get_disaster_locations(
    map_service: web::Data<MapService>,
    query: web::Query<LocationFilter>,
) -> AppResult<HttpResponse> {
    let locations = map_service.get_disaster_locations(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(locations))
}

/// Get location details
pub async fn get_location(
    map_service: web::Data<MapService>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let location_id = path.into_inner();
    let location = map_service.get_location_by_id(location_id).await?;
    Ok(HttpResponse::Ok().json(location))
}

/// Search locations
pub async fn search_locations(
    map_service: web::Data<MapService>,
    query: web::Query<serde_json::Value>,
) -> AppResult<HttpResponse> {
    let locations = map_service.search_locations(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(locations))
}

/// Get nearby disasters
pub async fn get_nearby_disasters(
    map_service: web::Data<MapService>,
    query: web::Query<serde_json::Value>,
) -> AppResult<HttpResponse> {
    let disasters = map_service.get_nearby_disasters(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(disasters))
}

/// Get map data within bounds
pub async fn get_map_data(
    map_service: web::Data<MapService>,
    req: web::Json<MapBounds>,
) -> AppResult<HttpResponse> {
    let map_data = map_service.get_map_data(req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(map_data))
}

/// Get heatmap data
pub async fn get_heatmap_data(
    map_service: web::Data<MapService>,
    query: web::Query<LocationFilter>,
) -> AppResult<HttpResponse> {
    let heatmap_data = map_service.get_heatmap_data(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(heatmap_data))
}

/// Get evacuation routes
pub async fn get_evacuation_routes(
    map_service: web::Data<MapService>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let location_id = path.into_inner();
    let routes = map_service.get_evacuation_routes(location_id).await?;
    Ok(HttpResponse::Ok().json(routes))
}

/// Get safe zones
pub async fn get_safe_zones(
    map_service: web::Data<MapService>,
    query: web::Query<LocationFilter>,
) -> AppResult<HttpResponse> {
    let safe_zones = map_service.get_safe_zones(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(safe_zones))
}
