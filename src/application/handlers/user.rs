use actix_web::{web, HttpResponse};
use uuid::Uuid;
use crate::shared::error::AppResult;
use crate::application::services::user::UserService;
use crate::domain::entities::user::{CreateUserRequest, UpdateUserRequest};

/// Create new user
pub async fn create_user(
    user_service: web::Data<UserService>,
    req: web::Json<CreateUserRequest>,
) -> AppResult<HttpResponse> {
    let user = user_service.create_user(req.into_inner()).await?;
    Ok(HttpResponse::Created().json(user))
}

/// Get user by ID
pub async fn get_user(
    user_service: web::Data<UserService>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let user = user_service.get_user_by_id(user_id).await?;
    Ok(HttpResponse::Ok().json(user))
}

/// Get current user profile
pub async fn get_profile(
    user_service: web::Data<UserService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    // Extract user ID from JWT token in middleware
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| crate::shared::error::AppError::Unauthorized("User not authenticated".to_string()))?;
    
    let user = user_service.get_user_by_id(user_id).await?;
    Ok(HttpResponse::Ok().json(user))
}

/// Update user profile
pub async fn update_user(
    user_service: web::Data<UserService>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateUserRequest>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    let user = user_service.update_user(user_id, req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(user))
}

/// List users with pagination and filters
pub async fn list_users(
    user_service: web::Data<UserService>,
    query: web::Query<serde_json::Value>,
) -> AppResult<HttpResponse> {
    let users = user_service.list_users(query.into_inner()).await?;
    Ok(HttpResponse::Ok().json(users))
}

/// Delete user
pub async fn delete_user(
    user_service: web::Data<UserService>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let user_id = path.into_inner();
    user_service.delete_user(user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Verify user account
pub async fn verify_user(
    user_service: web::Data<UserService>,
    req: web::Json<serde_json::Value>,
) -> AppResult<HttpResponse> {
    let result = user_service.verify_user(req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}
