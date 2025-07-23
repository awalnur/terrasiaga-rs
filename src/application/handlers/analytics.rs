use actix_web::{web, HttpResponse};
use uuid::Uuid;
use crate::shared::error::AppResult;
use crate::application::services::analytics::AnalyticsService;
use crate::domain::entities::analytics::{AnalyticsFilter, ReportRequest};

/// Get disaster analytics
pub async fn get_disaster_analytics(
    analytics_service: web::Data<AnalyticsService>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let disaster_id = path.into_inner();
    let analytics = analytics_service.get_disaster_analytics(disaster_id).await?;
    Ok(HttpResponse::Ok().json(analytics))
}

/// Get dashboard statistics
pub async fn get_dashboard_stats(
    analytics_service: web::Data<AnalyticsService>,
    query: web::Query<AnalyticsFilter>,
) -> AppResult<HttpResponse> {
    let stats = analytics_service.get_dashboard_stats(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(stats))
}

/// Generate disaster report
pub async fn generate_report(
    analytics_service: web::Data<AnalyticsService>,
    req: web::Json<ReportRequest>,
) -> AppResult<HttpResponse> {
    let report = analytics_service.generate_report(req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(report))
}

/// Get regional statistics
pub async fn get_regional_stats(
    analytics_service: web::Data<AnalyticsService>,
    query: web::Query<AnalyticsFilter>,
) -> AppResult<HttpResponse> {
    let stats = analytics_service.get_regional_stats(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(stats))
}

/// Get disaster trends
pub async fn get_disaster_trends(
    analytics_service: web::Data<AnalyticsService>,
    query: web::Query<AnalyticsFilter>,
) -> AppResult<HttpResponse> {
    let trends = analytics_service.get_disaster_trends(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(trends))
}

/// Get resource utilization analytics
pub async fn get_resource_analytics(
    analytics_service: web::Data<AnalyticsService>,
    query: web::Query<AnalyticsFilter>,
) -> AppResult<HttpResponse> {
    let analytics = analytics_service.get_resource_analytics(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(analytics))
}

/// Get response time analytics
pub async fn get_response_time_analytics(
    analytics_service: web::Data<AnalyticsService>,
    query: web::Query<AnalyticsFilter>,
) -> AppResult<HttpResponse> {
    let analytics = analytics_service.get_response_time_analytics(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(analytics))
}

/// Export analytics data
pub async fn export_analytics(
    analytics_service: web::Data<AnalyticsService>,
    req: web::Json<serde_json::Value>,
) -> AppResult<HttpResponse> {
    let export_data = analytics_service.export_analytics(req.into_inner()).await?;
    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .append_header(("Content-Disposition", "attachment; filename=analytics_export.csv"))
        .body(export_data))
}
