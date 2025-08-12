/// Authentication middleware for Terra Siaga using PASETO tokens
/// Provides secure authentication and authorization for API endpoints

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpRequest, HttpResponse,
    web::Data,
};
use futures_util::future::{ok, Ready};
use std::future::{ready, Future};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use crate::shared::cache::CacheService;
use crate::shared::error::{AppError, AppResult};
use crate::shared::paseto_auth::{PasetoService, TokenClaims, TokenType};
use crate::shared::rate_limiter::{RateLimiter, RateLimitMiddleware};
use crate::shared::types::{UserId, UserRole};

/// Authenticated user information
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub role: UserRole,
    pub permissions: Vec<String>,
    pub session_id: String,
}

/// Authentication service that manages token validation and user sessions
pub struct AuthService {
    paseto_service: Arc<PasetoService>,
    cache_service: Arc<dyn CacheService>,
    rate_limiter: Arc<dyn RateLimiter>,
}

impl AuthService {
    pub fn new(
        paseto_service: Arc<PasetoService>,
        cache_service: Arc<dyn CacheService>,
        rate_limiter: Arc<dyn RateLimiter>,
    ) -> Self {
        Self {
            paseto_service,
            cache_service,
            rate_limiter,
        }
    }

    /// Validate token and extract user information
    pub async fn validate_token(&self, token: &str) -> AppResult<AuthenticatedUser> {
        // Verify token signature and decode claims
        let claims = self.paseto_service.verify_token(token)?;

        // Check if it's an access token
        if claims.token_type != TokenType::Access {
            return Err(AppError::Unauthorized("Invalid token type".to_string()));
        }

        // Check if session is still valid (not revoked)
        let session_key = format!("session:{}", claims.session_id);
        if !self.cache_service.exists(&session_key).await? {
            return Err(AppError::Unauthorized("Session has been revoked".to_string()));
        }

        // Parse user ID
        let user_id = uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))?;

        Ok(AuthenticatedUser {
            user_id: UserId(user_id),
            role: claims.role,
            permissions: claims.permissions,
            session_id: claims.session_id,
        })
    }

    /// Extract token from request headers
    pub fn extract_token_from_request(&self, req: &HttpRequest) -> AppResult<String> {
        let auth_header = req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized("Invalid authorization header format".to_string()));
        }

        Ok(auth_header[7..].to_string())
    }

    /// Check if user has required permission
    pub fn has_permission(&self, user: &AuthenticatedUser, required_permission: &str) -> bool {
        user.permissions.contains(&required_permission.to_string())
            || user.role == UserRole::Admin // Admin has all permissions
    }

    /// Check if user has required role
    pub fn has_role(&self, user: &AuthenticatedUser, required_role: UserRole) -> bool {
        user.role == required_role || user.role == UserRole::Admin
    }

    /// Revoke a session
    pub async fn revoke_session(&self, session_id: &str) -> AppResult<()> {
        let session_key = format!("session:{}", session_id);
        self.cache_service.delete(&session_key).await?;
        Ok(())
    }

    /// Create a new session
    pub async fn create_session(&self, user_id: UserId, session_id: &str) -> AppResult<()> {
        let session_key = format!("session:{}", session_id);
        let session_data = serde_json::json!({
            "user_id": user_id.0.to_string(),
            "created_at": chrono::Utc::now().to_rfc3339(),
        });

        // Store session for 7 days (same as refresh token expiry)
        self.cache_service.set(&session_key, &session_data.to_string(), Some(7 * 24 * 3600)).await?;
        Ok(())
    }
}

/// Authentication middleware factory
pub struct AuthMiddleware {
    required_permissions: Vec<String>,
    required_role: Option<UserRole>,
    allow_anonymous: bool,
}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {
            required_permissions: Vec::new(),
            required_role: None,
            allow_anonymous: false,
        }
    }

    pub fn require_permission(mut self, permission: &str) -> Self {
        self.required_permissions.push(permission.to_string());
        self
    }

    pub fn require_role(mut self, role: UserRole) -> Self {
        self.required_role = Some(role);
        self
    }

    pub fn allow_anonymous(mut self) -> Self {
        self.allow_anonymous = true;
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service,
            required_permissions: self.required_permissions.clone(),
            required_role: self.required_role.clone(),
            allow_anonymous: self.allow_anonymous,
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    required_permissions: Vec<String>,
    required_role: Option<UserRole>,
    allow_anonymous: bool,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let required_permissions = self.required_permissions.clone();
        let required_role = self.required_role.clone();
        let allow_anonymous = self.allow_anonymous;

        Box::pin(async move {
            // Try to get auth service from app data
            let auth_service = req.app_data::<Data<AuthService>>();

            if auth_service.is_none() {
                return Ok(req.into_response(
                    HttpResponse::InternalServerError()
                        .json(serde_json::json!({
                            "error": "Authentication service not configured"
                        }))
                        .into_body()
                ));
            }

            let auth_service = auth_service.unwrap();

            // Extract token from request
            let token_result = auth_service.extract_token_from_request(req.request());

            match token_result {
                Ok(token) => {
                    // Validate token
                    match auth_service.validate_token(&token).await {
                        Ok(user) => {
                            // Check permissions
                            for permission in &required_permissions {
                                if !auth_service.has_permission(&user, permission) {
                                    return Ok(req.into_response(
                                        HttpResponse::Forbidden()
                                            .json(serde_json::json!({
                                                "error": format!("Missing required permission: {}", permission)
                                            }))
                                            .into_body()
                                    ));
                                }
                            }

                            // Check role
                            if let Some(role) = &required_role {
                                if !auth_service.has_role(&user, role.clone()) {
                                    return Ok(req.into_response(
                                        HttpResponse::Forbidden()
                                            .json(serde_json::json!({
                                                "error": format!("Missing required role: {:?}", role)
                                            }))
                                            .into_body()
                                    ));
                                }
                            }

                            // Add user to request extensions
                            req.extensions_mut().insert(user);
                        }
                        Err(_) => {
                            if !allow_anonymous {
                                return Ok(req.into_response(
                                    HttpResponse::Unauthorized()
                                        .json(serde_json::json!({
                                            "error": "Invalid or expired token"
                                        }))
                                        .into_body()
                                ));
                            }
                        }
                    }
                }
                Err(_) => {
                    if !allow_anonymous {
                        return Ok(req.into_response(
                            HttpResponse::Unauthorized()
                                .json(serde_json::json!({
                                    "error": "Missing or invalid authorization header"
                                }))
                                .into_body()
                        ));
                    }
                }
            }

            // Continue to the next service
            let fut = req.into_inner().0.call(req);
            fut.await
        })
    }
}

/// Helper function to get authenticated user from request
pub fn get_authenticated_user(req: &HttpRequest) -> Option<&AuthenticatedUser> {
    req.extensions().get::<AuthenticatedUser>()
}

/// Convenience macro for creating permission-based middleware
#[macro_export]
macro_rules! require_permission {
    ($permission:expr) => {
        AuthMiddleware::new().require_permission($permission)
    };
}

