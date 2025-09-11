/// Enhanced authentication API endpoints with PASETO support
/// Provides secure login, registration, and session management

use actix_web::{web, App, HttpMessage, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::AppError;
use crate::application::ValidatedUseCase;
use crate::infrastructure::{
    container::AppContainer,
    security::{SecureAuthSession},
};
use crate::domain::value_objects::{Email, UserRole as DomainUserRole};
use crate::middleware::AuthMiddleware;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    pub device_fingerprint: Option<String>,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(length(min = 2, message = "Full name must be at least 2 characters"))]
    pub full_name: String,
    pub phone_number: Option<String>,
    pub role: Option<String>, // Default to "citizen"
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<AuthData>,
}

#[derive(Debug, Serialize)]
pub struct AuthData {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub user: UserInfo,
    pub session_id: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ElevateSessionRequest {
    pub mfa_token: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

/// POST /api/v1/auth/login
/// Enhanced login endpoint with PASETO tokens
pub async fn login(
    req: HttpRequest,
    login_req: web::Json<LoginRequest>,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {

    // Validate request
    if let Err(validation_errors) = login_req.validate() {
        println!("{:?}", validation_errors);
        return Ok(HttpResponse::BadRequest().json(AuthResponse {
            success: false,
            message: format!("Validation failed: {:?}", validation_errors),
            data: None,
        }));
    }

    // Extract client information
    let ip_address = req.peer_addr().map(|addr| addr.ip().to_string());

    let user_agent = req.headers()
        .get("User-Agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    println!("User-Agent: {}", user_agent.clone().unwrap_or_else(|| "None".to_string()));
    // Check rate limiting and account lockout
    let email_for_check = &login_req.email;
    if let Ok(false) = container.auth_service.is_account_locked(email_for_check, ip_address.as_deref()).await {
        // Account is locked
        return Ok(HttpResponse::TooManyRequests().json(AuthResponse {
            success: false,
            message: "Account temporarily locked due to too many failed attempts".to_string(),
            data: None,
        }));
    }

    // Find user by email
    let email = match Email::new(login_req.email.clone()) {
        Ok(email) => email,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(AuthResponse {
                success: false,
                message: "Invalid email format".to_string(),
                data: None,
            }));
        }
    };

    let user = match container.user_repository.find_by_email(&email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            // Track failed attempt even for non-existent users
            let _ = container.auth_service.track_failed_login(&login_req.email, ip_address.as_deref()).await;
            return Ok(HttpResponse::Unauthorized().json(AuthResponse {
                success: false,
                message: "Invalid credentials".to_string(),
                data: None,
            }));
        }
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: "Authentication service temporarily unavailable".to_string(),
                data: None,
            }));
        }
    };

    // Verify password
    let password_valid = match container.auth_service.verify_password(&login_req.password, &user.password_hash).await {
        Ok(valid) => valid,
        Err(_) => false,
    };

    if !password_valid {
        // Track failed attempt
        let _ = container.auth_service.track_failed_login(&login_req.email, ip_address.as_deref()).await;
        return Ok(HttpResponse::Unauthorized().json(AuthResponse {
            success: false,
            message: "Invalid credentials".to_string(),
            data: None,
        }));
    }

    // Check if user account is active
    if !user.is_active() {
        return Ok(HttpResponse::Forbidden().json(AuthResponse {
            success: false,
            message: format!("Account is {}", user.status()),
            data: None,
        }));
    }

    // Clear failed attempts on successful login
    let _ = container.auth_service.clear_failed_attempts(&login_req.email).await;

    // Create PASETO token
    let token = match container.paseto_service.create_paseto_token(
        &user,
        ip_address,
        user_agent,
        login_req.device_fingerprint.clone(),
    ).await {
        Ok(token) => token,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(AuthResponse {
                success: false,
                message: "Failed to generate authentication token".to_string(),
                data: None,
            }));
        }
    };
    println!("Token: {}", token);
    // Get session for response
    let session = match container.paseto_service.validate_paseto_token(&token).await {
        Ok(session) => session,
        Err(AppError) => {
            println!("Failed to validate token {}", AppError);
            return Ok(HttpResponse::Unauthorized().json(AuthResponse {
                success: false,
                message: "Invalid authentication token".to_string(),
                data: None,
            }));
        }
    };

    let expires_in = (session.expires_at - chrono::Utc::now()).num_seconds() as u64;

    Ok(HttpResponse::Ok().json(AuthResponse {
        success: true,
        message: "Login successful".to_string(),
        data: Some(AuthData {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in,
            user: UserInfo {
                id: user.id().value().to_string(),
                email: user.email().value().to_string(),
                full_name: user.full_name().to_string(),
                role: format!("{:?}", user.role()),
                status: user.status().to_string(),
            },
            session_id: session.session_id,
            permissions: session.permissions,
        }),
    }))
}

/// Enhanced registration endpoint
pub async fn register(
    req: HttpRequest,
    register_req: web::Json<RegisterRequest>,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    // Validate request
    if let Err(validation_errors) = register_req.validate() {
        return Ok(HttpResponse::BadRequest().json(AuthResponse {
            success: false,
            message: format!("Validation failed: {:?}", validation_errors),
            data: None,
        }));
    }

    // Parse role
    let role = match register_req.role.as_deref() {
        Some("volunteer") => DomainUserRole::Volunteer,
        Some("responder") => DomainUserRole::Responder,
        Some("admin") => DomainUserRole::Admin,
        _ => DomainUserRole::Citizen, // Default role
    };

    // Create use case request
    let use_case_request = crate::application::use_cases::RegisterUserRequest {
        email: register_req.email.clone(),
        password: register_req.password.clone(),
        full_name: register_req.full_name.clone(),
        phone_number: register_req.phone_number.clone(),
        role,
        address: None, // Can be updated later
        emergency_contact: None,
    };

    // Execute registration use case
    match container.register_user_use_case.execute_validated(use_case_request).await {
        Ok(user_response) => {
            Ok(HttpResponse::Created().json(AuthResponse {
                success: true,
                message: "Registration successful. Please check your email for verification.".to_string(),
                data: Some(AuthData {
                    access_token: "".to_string(), // Don't auto-login after registration
                    token_type: "Bearer".to_string(),
                    expires_in: 0,
                    user: UserInfo {
                        id: user_response.id.value().to_string(),
                        email: user_response.email.value().to_string(),
                        full_name: user_response.full_name,
                        role: format!("{:?}", user_response.role),
                        status: user_response.status,
                    },
                    session_id: "".to_string(),
                    permissions: vec![],
                }),
            }))
        }
        Err(e) => {
            Ok(HttpResponse::BadRequest().json(AuthResponse {
                success: false,
                message: e.to_string(),
                data: None,
            }))
        }
    }
}

/// Elevate session for sensitive operations
pub async fn elevate_session(
    req: HttpRequest,
    elevate_req: web::Json<ElevateSessionRequest>,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    // Extract current session from middleware
    let session = match req.extensions().get::<SecureAuthSession>() {
        Some(session) => session.clone(),
        None => {
            return Ok(HttpResponse::Unauthorized().json(AuthResponse {
                success: false,
                message: "No active session found".to_string(),
                data: None,
            }));
        }
    };

    // Create elevated session
    match container.paseto_service.create_elevated_session(
        &session.session_id,
        elevate_req.mfa_token.as_deref(),
    ).await {
        Ok(_) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Session elevated successfully",
                "elevated_until": chrono::Utc::now() + chrono::Duration::minutes(15)
            })))
        }
        Err(e) => {
            Ok(HttpResponse::BadRequest().json(AuthResponse {
                success: false,
                message: format!("Failed to elevate session: {}", e),
                data: None,
            }))
        }
    }
}

/// POST /api/v1/auth/refresh
async fn refresh_token(
    req: web::Json<RefreshTokenRequest>,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    // TODO: Implement token refresh logic
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Token refreshed successfully"
    })))
}

/// Logout endpoint
pub async fn logout(
    req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    // Extract current session from middleware
    let session = match req.extensions().get::<SecureAuthSession>() {
        Some(session) => session.clone(),
        None => {
            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Already logged out"
            })));
        }
    };

    // Revoke session
    match container.paseto_service.revoke_session(&session.session_id).await {
        Ok(_) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Logout successful"
            })))
        }
        Err(_) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": "Failed to logout properly"
            })))
        }
    }
}

/// Get current user info
pub async fn me(req: HttpRequest) -> Result<HttpResponse> {
    // Extract current session from middleware
    let session = match req.extensions().get::<SecureAuthSession>() {
        Some(session) => session.clone(),
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "No active session"
            })));
        }
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user": {
            "id": session.user_id.value(),
            "email": session.email.value(),
            "role": format!("{:?}", session.role),
            "permissions": session.permissions,
            "is_elevated": session.is_elevated,
            "mfa_verified": session.mfa_verified,
            "session_id": session.session_id,
            "last_activity": session.last_activity
        }
    })))
}

/// POST /api/v1/auth/change-password
async fn change_password(
    req: web::Json<ChangePasswordRequest>,
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    // TODO: Implement change password logic
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password changed successfully"
    })))
}

/// POST /api/v1/auth/reset-password
async fn reset_password(
    req: web::Json<ResetPasswordRequest>,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    // TODO: Implement password reset request logic
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password reset email sent"
    })))
}

/// POST /api/v1/auth/confirm-reset-password
async fn confirm_reset_password(
    req: web::Json<ConfirmResetPasswordRequest>,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    // TODO: Implement password reset confirmation logic
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password reset confirmed"
    })))
}

/// GET /api/v1/auth/verify-email/{token}
async fn verify_email(
    path: web::Path<String>,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    let token = path.into_inner();
    // TODO: Implement email verification logic
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Email verified successfully",
        "token": token
    })))
}

/// POST /api/v1/auth/resend-verification
async fn resend_verification_email(
    http_req: HttpRequest,
    container: web::Data<AppContainer>,
) -> Result<HttpResponse> {
    // TODO: Implement resend verification email logic
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Verification email sent"
    })))
}

pub fn configure_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/login", web::post().to(login))
        .route("/register", web::post().to(register))
        .route("/refresh", web::post().to(refresh_token))
        .route("/logout", web::post().to(logout))
        .route("/me", web::get().to(me).wrap(AuthMiddleware::new()))
        .route("/change-password", web::post().to(change_password))
        .route("/reset-password", web::post().to(reset_password))
        .route("/confirm-reset-password", web::post().to(confirm_reset_password))
        .route("/verify-email/{token}", web::get().to(verify_email))
        .route("/resend-verification", web::post().to(resend_verification_email));
}
