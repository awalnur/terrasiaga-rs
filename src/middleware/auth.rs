/// Enhanced authentication middleware with PASETO support
/// Provides secure token validation and role-based access control

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, web, FromRequest,
};
use actix_web::body::EitherBody;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::sync::Arc;
use std::rc::Rc;
use tracing::{error, warn, debug};
use crate::shared::paseto_auth::{PasetoService, TokenClaims, TokenType};
use crate::shared::types::{UserRole, UserId};
use crate::shared::error::AppError;

/// Authentication session extracted from PASETO token
#[derive(Debug, Clone)]
pub struct AuthSession {
    pub user_id: UserId,
    pub role: UserRole,
    pub permissions: Vec<String>,
    pub session_id: String,
    pub device_fingerprint: Option<String>,
    pub ip_address: Option<String>,
}

impl AuthSession {
    /// Check if user has specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }

    /// Check if user has all required permissions
    pub fn has_all_permissions(&self, required_permissions: &[String]) -> bool {
        required_permissions.iter().all(|perm| self.permissions.contains(perm))
    }

    /// Check if user has minimum role level
    pub fn has_minimum_role(&self, min_role: &UserRole) -> bool {
        self.role.has_minimum_level(min_role)
    }

    /// Check if user has role level or higher
    pub fn has_role_level(&self, required_role: &UserRole) -> bool {
        self.role.has_minimum_level(required_role)
    }
}

/// PASETO authentication middleware
pub struct AuthMiddleware {
    required_role: Option<UserRole>,
    required_permissions: Option<Vec<String>>,
    token_type: TokenType,
}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {
            required_role: None,
            required_permissions: None,
            token_type: TokenType::Access,
        }
    }

    pub fn with_role(role: UserRole) -> Self {
        Self {
            required_role: Some(role),
            required_permissions: None,
            token_type: TokenType::Access,
        }
    }

    pub fn with_permissions(permissions: Vec<String>) -> Self {
        Self {
            required_role: None,
            required_permissions: Some(permissions),
            token_type: TokenType::Access,
        }
    }

    pub fn with_token_type(mut self, token_type: TokenType) -> Self {
        self.token_type = token_type;
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            required_role: self.required_role.clone(),
            required_permissions: self.required_permissions.clone(),
            token_type: self.token_type.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    required_role: Option<UserRole>,
    required_permissions: Option<Vec<String>>,
    token_type: TokenType,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let required_role = self.required_role.clone();
        let required_permissions = self.required_permissions.clone();
        let token_type = self.token_type.clone();

        Box::pin(async move {
            // Get PASETO service from app data
            let paseto = match req.app_data::<web::Data<Arc<PasetoService>>>() {
                Some(service) => service.get_ref().clone(),
                None => {
                    error!("PASETO service not found in app data");
                    let resp = HttpResponse::InternalServerError()
                        .json(serde_json::json!({
                            "error": "Authentication service unavailable"
                        }))
                        .map_into_right_body();
                    return Ok(req.into_response(resp));
                }
            };

            // Extract authorization header
            let auth_header = req.headers().get("Authorization");

            if let Some(header_value) = auth_header {
                if let Ok(header_str) = header_value.to_str() {
                    if header_str.starts_with("Bearer ") {
                        let token = &header_str[7..];

                        match paseto.verify_token(token) {
                            Ok(claims) => {
                                // Check token type
                                if claims.token_type != token_type {
                                    let resp = HttpResponse::Unauthorized()
                                        .json(serde_json::json!({
                                            "error": "Invalid token type",
                                            "message": "Access token required"
                                        }))
                                        .map_into_right_body();
                                    return Ok(req.into_response(resp));
                                }

                                // Resolve user id from token
                                let user_id = match paseto.extract_user_id(token) {
                                    Ok(id) => id,
                                    Err(e) => {
                                        warn!("Failed to extract user id: {}", e);
                                        let resp = HttpResponse::Unauthorized()
                                            .json(serde_json::json!({
                                                "error": "Invalid token",
                                                "message": e.to_string()
                                            }))
                                            .map_into_right_body();
                                        return Ok(req.into_response(resp));
                                    }
                                };

                                // Create auth session
                                let session = AuthSession {
                                    user_id,
                                    role: claims.role,
                                    permissions: claims.permissions,
                                    session_id: claims.session_id,
                                    device_fingerprint: claims.device_fingerprint,
                                    ip_address: req.connection_info().realip_remote_addr().map(String::from),
                                };

                                // Check role requirements
                                if let Some(min_role) = &required_role {
                                    if !session.has_minimum_role(min_role) {
                                        let resp = HttpResponse::Forbidden()
                                            .json(serde_json::json!({
                                                "error": "Insufficient permissions",
                                                "message": "Higher role required"
                                            }))
                                            .map_into_right_body();
                                        return Ok(req.into_response(resp));
                                    }
                                }

                                // Check permission requirements
                                if let Some(required_permissions) = &required_permissions {
                                    let required_refs: Vec<String> = required_permissions.clone();
                                    if !session.has_all_permissions(&required_refs) {
                                        let resp = HttpResponse::Forbidden()
                                            .json(serde_json::json!({
                                                "error": "Insufficient permissions",
                                                "message": "Required permissions not granted",
                                                "required": required_permissions
                                            }))
                                            .map_into_right_body();
                                        return Ok(req.into_response(resp));
                                    }
                                }

                                // Insert auth session into request extensions
                                req.extensions_mut().insert(session);
                            }
                            Err(e) => {
                                warn!("Token validation failed: {}", e);
                                let resp = HttpResponse::Unauthorized()
                                    .json(serde_json::json!({
                                        "error": "Invalid or expired token",
                                        "message": e.to_string()
                                    }))
                                    .map_into_right_body();
                                return Ok(req.into_response(resp));
                            }
                        }
                    }
                }
            } else {
                // No authorization header
                let resp = HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "Authentication required",
                        "message": "Valid authentication token required"
                    }))
                    .map_into_right_body();
                return Ok(req.into_response(resp));
            }

            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

// Helper functions for common auth requirements
pub fn require_auth() -> AuthMiddleware {
    AuthMiddleware::new()
}

pub fn require_role(role: UserRole) -> AuthMiddleware {
    AuthMiddleware::with_role(role)
}

pub fn require_admin() -> AuthMiddleware {
    AuthMiddleware::with_role(UserRole::Admin)
}

pub fn require_permissions(permissions: Vec<&str>) -> AuthMiddleware {
    AuthMiddleware::with_permissions(
        permissions.into_iter().map(String::from).collect()
    )
}

// FromRequest implementation for AuthSession
impl FromRequest for AuthSession {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(session) = req.extensions().get::<AuthSession>() {
            ready(Ok(session.clone()))
        } else {
            ready(Err(actix_web::error::ErrorUnauthorized("No authentication session found")))
        }
    }
}
