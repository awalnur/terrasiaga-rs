use actix_web::{web, HttpResponse};
use uuid::Uuid;
use crate::shared::error::AppResult;
use crate::application::services::notification::NotificationService;
use crate::domain::entities::notification::{CreateNotificationRequest, NotificationFilter};

/// Send notification
pub async fn send_notification(
    notification_service: web::Data<NotificationService>,
    req: web::Json<CreateNotificationRequest>,
) -> AppResult<HttpResponse> {
    let notification = notification_service.send_notification(req.into_inner()).await?;
    Ok(HttpResponse::Created().json(notification))
}

/// Get notification by ID
pub async fn get_notification(
    notification_service: web::Data<NotificationService>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let notification_id = path.into_inner();
    let notification = notification_service.get_notification_by_id(notification_id).await?;
    Ok(HttpResponse::Ok().json(notification))
}

/// Get user notifications
pub async fn get_user_notifications(
    notification_service: web::Data<NotificationService>,
    req: actix_web::HttpRequest,
    query: web::Query<NotificationFilter>,
) -> AppResult<HttpResponse> {
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| crate::shared::error::AppError::Unauthorized("User not authenticated".to_string()))?;
    
    let notifications = notification_service.get_user_notifications(user_id, query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(notifications))
}

/// Mark notification as read
pub async fn mark_as_read(
    notification_service: web::Data<NotificationService>,
    path: web::Path<Uuid>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let notification_id = path.into_inner();
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| crate::shared::error::AppError::Unauthorized("User not authenticated".to_string()))?;
    
    notification_service.mark_as_read(notification_id, user_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Notification marked as read"
    })))
}

/// Mark all notifications as read
pub async fn mark_all_as_read(
    notification_service: web::Data<NotificationService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| crate::shared::error::AppError::Unauthorized("User not authenticated".to_string()))?;
    
    notification_service.mark_all_as_read(user_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "All notifications marked as read"
    })))
}

/// Delete notification
pub async fn delete_notification(
    notification_service: web::Data<NotificationService>,
    path: web::Path<Uuid>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let notification_id = path.into_inner();
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| crate::shared::error::AppError::Unauthorized("User not authenticated".to_string()))?;
    
    notification_service.delete_notification(notification_id, user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Get notification statistics
pub async fn get_notification_stats(
    notification_service: web::Data<NotificationService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| crate::shared::error::AppError::Unauthorized("User not authenticated".to_string()))?;
    
    let stats = notification_service.get_notification_stats(user_id).await?;
    Ok(HttpResponse::Ok().json(stats))
}
