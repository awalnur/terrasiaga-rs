use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::error::{AppError, AppResult};
use crate::shared::auth_middleware::{AuthService, AuthenticatedUser, get_authenticated_user};
use crate::shared::paseto_auth::{PasetoService, utils};
use crate::shared::types::{UserId, UserRole};
use crate::shared::validation::validate_request;

/// Request/Response DTOs for authentication
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub phone_number: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub role: UserRole,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
    pub expires_in: i64,
}

/// User registration
pub async fn register(
    auth_service: web::Data<AuthService>,
    paseto_service: web::Data<PasetoService>,
    req: web::Json<RegisterRequest>,
) -> AppResult<HttpResponse> {
    // Validate request
    validate_request(&req)?;

    // Check if user already exists
    // This would typically involve checking the database
    // For now, we'll simulate user creation

    // Create new user ID
    let user_id = UserId(Uuid::new_v4());
    let session_id = utils::generate_session_id();

    // Default permissions for new users
    let permissions = vec![
        "reports:read".to_string(),
        "reports:write".to_string(),
        "alerts:read".to_string(),
        "data:read_public".to_string(),
    ];

    // Generate token pair
    let token_pair = paseto_service.generate_token_pair(
        user_id,
        UserRole::Reporter, // Updated to use Reporter instead of User
        permissions.clone(),
        session_id.clone(),
    )?;

    // Create session
    auth_service.create_session(user_id, &session_id).await?;

    let response = AuthResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        expires_in: token_pair.expires_in,
        refresh_expires_in: token_pair.refresh_expires_in,
        user: UserInfo {
            id: user_id.0.to_string(),
            email: req.email.clone(),
            full_name: req.full_name.clone(),
            role: UserRole::Reporter, // Updated to use Reporter
            permissions,
        },
    };

    Ok(HttpResponse::Created().json(response))
}

/// User login
pub async fn login(
    auth_service: web::Data<AuthService>,
    paseto_service: web::Data<PasetoService>,
    req: web::Json<LoginRequest>,
) -> AppResult<HttpResponse> {
    // Validate request
    validate_request(&req)?;

    // Authenticate user (this would involve database lookup and password verification)
    // For now, we'll simulate successful authentication
    let user_id = UserId(Uuid::new_v4());
    let session_id = utils::generate_session_id();

    // Get user permissions based on role (this would come from database)
    let role = UserRole::Reporter; // This would be fetched from database
    let permissions = get_user_permissions(&role);

    // Generate token pair
    let token_pair = paseto_service.generate_token_pair(
        user_id,
        role.clone(),
        permissions.clone(),
        session_id.clone(),
    )?;

    // Create session
    auth_service.create_session(user_id, &session_id).await?;

    let response = AuthResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        expires_in: token_pair.expires_in,
        refresh_expires_in: token_pair.refresh_expires_in,
        user: UserInfo {
            id: user_id.0.to_string(),
            email: req.email.clone(),
            full_name: "User Name".to_string(), // This would come from database
            role,
            permissions,
        },
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Refresh access token
pub async fn refresh_token(
    paseto_service: web::Data<PasetoService>,
    req: web::Json<RefreshTokenRequest>,
) -> AppResult<HttpResponse> {
    // Validate request
    validate_request(&req)?;

    // Refresh the access token
    let new_access_token = paseto_service.refresh_access_token(&req.refresh_token)?;
    let expires_in = 15 * 60; // 15 minutes in seconds

    let response = RefreshResponse {
        access_token: new_access_token,
        expires_in,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// User logout
pub async fn logout(
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    // Get authenticated user from middleware
    let user = get_authenticated_user(&req)
        .ok_or_else(|| AppError::Unauthorized("User not authenticated".to_string()))?;

    // Revoke the session
    auth_service.revoke_session(&user.session_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Successfully logged out"
    })))
}

/// Logout from all devices
pub async fn logout_all(
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    // Get authenticated user from middleware
    let user = get_authenticated_user(&req)
        .ok_or_else(|| AppError::Unauthorized("User not authenticated".to_string()))?;

    // Revoke all sessions for this user
    // This would typically involve finding all sessions for the user and revoking them
    // For now, we'll just revoke the current session
    auth_service.revoke_session(&user.session_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Successfully logged out from all devices"
    })))
}

/// Get current user profile
pub async fn me(req: HttpRequest) -> AppResult<HttpResponse> {
    // Get authenticated user from middleware
    let user = get_authenticated_user(&req)
        .ok_or_else(|| AppError::Unauthorized("User not authenticated".to_string()))?;

    let user_info = UserInfo {
        id: user.user_id.0.to_string(),
        email: "user@example.com".to_string(), // This would come from database
        full_name: "User Name".to_string(), // This would come from database
        role: user.role.clone(),
        permissions: user.permissions.clone(),
    };

    Ok(HttpResponse::Ok().json(user_info))
}

/// Verify token endpoint
pub async fn verify_token(
    paseto_service: web::Data<PasetoService>,
    req: HttpRequest,
) -> AppResult<HttpResponse> {
    // Get authenticated user from middleware
    let user = get_authenticated_user(&req)
        .ok_or_else(|| AppError::Unauthorized("Invalid token".to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "valid": true,
        "user_id": user.user_id.0.to_string(),
        "role": user.role,
        "permissions": user.permissions
    })))
}

/// Change password
pub async fn change_password(
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
    payload: web::Json<ChangePasswordRequest>,
) -> AppResult<HttpResponse> {
    // Get authenticated user from middleware
    let user = get_authenticated_user(&req)
        .ok_or_else(|| AppError::Unauthorized("User not authenticated".to_string()))?;

    // Validate request
    validate_request(&payload)?;

    // Here you would:
    // 1. Verify current password
    // 2. Update password in database
    // 3. Optionally revoke all existing sessions

    // For now, we'll just return success
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password changed successfully"
    })))
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Helper function to get user permissions based on role
fn get_user_permissions(role: &UserRole) -> Vec<String> {
    use crate::shared::types::Permission;

    match role {
        UserRole::SystemAdmin => vec![
            Permission::ManageSystem.as_str().to_string(),
            Permission::ReadSystemData.as_str().to_string(),
            Permission::WriteSystemData.as_str().to_string(),
            Permission::ManageUsers.as_str().to_string(),
            Permission::ReadAnalytics.as_str().to_string(),
            Permission::WriteAnalytics.as_str().to_string(),
            Permission::ManageEmergencyResponse.as_str().to_string(),
            Permission::ReadReports.as_str().to_string(),
            Permission::WriteReports.as_str().to_string(),
            Permission::ReadAlerts.as_str().to_string(),
            Permission::WriteAlerts.as_str().to_string(),
        ],
        UserRole::OrgAdmin => vec![
            Permission::ManageOrganization.as_str().to_string(),
            Permission::ManageUsers.as_str().to_string(),
            Permission::ReadAnalytics.as_str().to_string(),
            Permission::WriteAnalytics.as_str().to_string(),
            Permission::ManageEmergencyResponse.as_str().to_string(),
            Permission::ManageVolunteers.as_str().to_string(),
            Permission::ReadReports.as_str().to_string(),
            Permission::WriteReports.as_str().to_string(),
            Permission::ReadAlerts.as_str().to_string(),
            Permission::WriteAlerts.as_str().to_string(),
        ],
        UserRole::Coordinator => vec![
            Permission::ManageEmergencyResponse.as_str().to_string(),
            Permission::UpdateEmergencyStatus.as_str().to_string(),
            Permission::AssignEmergencyResponse.as_str().to_string(),
            Permission::ManageVolunteers.as_str().to_string(),
            Permission::ReadAnalytics.as_str().to_string(),
            Permission::ReadReports.as_str().to_string(),
            Permission::WriteReports.as_str().to_string(),
            Permission::ReadAlerts.as_str().to_string(),
            Permission::WriteAlerts.as_str().to_string(),
        ],
        UserRole::Volunteer => vec![
            Permission::WriteVolunteerResponse.as_str().to_string(),
            Permission::ReadVolunteerData.as_str().to_string(),
            Permission::UpdateEmergencyStatus.as_str().to_string(),
            Permission::ReadReports.as_str().to_string(),
            Permission::WriteReports.as_str().to_string(),
            Permission::ReadAlerts.as_str().to_string(),
        ],
        UserRole::Reporter => vec![
            Permission::ReadReports.as_str().to_string(),
            Permission::WriteReports.as_str().to_string(),
            Permission::ReadOwnReports.as_str().to_string(),
            Permission::ReadAlerts.as_str().to_string(),
            Permission::ReadPublicData.as_str().to_string(),
            Permission::ReadOwnProfile.as_str().to_string(),
            Permission::UpdateOwnProfile.as_str().to_string(),
        ],
    }
}
