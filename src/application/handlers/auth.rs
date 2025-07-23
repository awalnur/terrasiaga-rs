use actix_web::{web, HttpResponse};
use uuid::Uuid;
use crate::shared::error::AppResult;
use crate::application::services::auth::AuthService;
use crate::domain::entities::auth::{LoginRequest, RegisterRequest, RefreshTokenRequest};

/// User registration
pub async fn register(
    auth_service: web::Data<AuthService>,
    req: web::Json<RegisterRequest>,
) -> AppResult<HttpResponse> {
    let result = auth_service.register(req.into_inner()).await?;
    Ok(HttpResponse::Created().json(result))
}

/// User login
pub async fn login(
    auth_service: web::Data<AuthService>,
    req: web::Json<LoginRequest>,
) -> AppResult<HttpResponse> {
    let result = auth_service.login(req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

/// Refresh access token
pub async fn refresh_token(
    auth_service: web::Data<AuthService>,
    req: web::Json<RefreshTokenRequest>,
) -> AppResult<HttpResponse> {
    let result = auth_service.refresh_token(req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(result))
}

/// User logout
pub async fn logout(
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    // Extract user ID from JWT token in middleware
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| crate::shared::error::AppError::Unauthorized("User not authenticated".to_string()))?;
    
    auth_service.logout(user_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Successfully logged out"
    })))
}

/// Logout from all devices
pub async fn logout_all(
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| crate::shared::error::AppError::Unauthorized("User not authenticated".to_string()))?;
    
    auth_service.logout_all_devices(user_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Successfully logged out from all devices"
    })))
}

/// Request password reset
pub async fn forgot_password(
    auth_service: web::Data<AuthService>,
    req: web::Json<serde_json::Value>,
) -> AppResult<HttpResponse> {
    auth_service.forgot_password(req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password reset instructions sent to your email"
    })))
}

/// Reset password with token
pub async fn reset_password(
    auth_service: web::Data<AuthService>,
    req: web::Json<serde_json::Value>,
) -> AppResult<HttpResponse> {
    auth_service.reset_password(req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password successfully reset"
    })))
}

/// Change password
pub async fn change_password(
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
    body: web::Json<serde_json::Value>,
) -> AppResult<HttpResponse> {
    let user_id = req.extensions().get::<Uuid>().cloned()
        .ok_or_else(|| crate::shared::error::AppError::Unauthorized("User not authenticated".to_string()))?;
    
    auth_service.change_password(user_id, body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password successfully changed"
    })))
}
