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
use crate::infrastructure::security::paseto_service::PasetoSecurityService;
use crate::infrastructure::security::SecureAuthSession;
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

impl SecureAuthSession {
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
}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {
            required_role: None,
            required_permissions: None,
        }
    }

    pub fn with_role(role: UserRole) -> Self {
        Self {
            required_role: Some(role),
            required_permissions: None,
        }
    }

    pub fn with_permissions(permissions: Vec<String>) -> Self {
        Self {
            required_role: None,
            required_permissions: Some(permissions),
        }
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
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    required_role: Option<UserRole>,
    required_permissions: Option<Vec<String>>,
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

        Box::pin(async move {
            // Get PASETO service from app data
            let paseto = match req.app_data::<web::Data<Arc<PasetoSecurityService>>>() {
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
            let token_opt = req.headers().get("Authorization").and_then(|hv| hv.to_str().ok()).and_then(|s| {
                if s.starts_with("Bearer ") { Some(s[7..].to_string()) } else { None }
            });

            let token = match token_opt {
                Some(t) if !t.is_empty() => t,
                _ => {
                    let resp = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Authentication required",
                            "message": "Valid Bearer token is required"
                        }))
                        .map_into_right_body();
                    return Ok(req.into_response(resp));
                }
            };

            // Validate access token
            let session_res = paseto.validate_paseto_token(&token).await;
            let session = match session_res {
                Ok(s) => s,
                Err(e) => {
                    debug!("Token validation failed: {:?}", e);
                    let mut status = match e {
                        AppError::Unauthorized(_) => HttpResponse::Unauthorized(),
                        _ => HttpResponse::InternalServerError(),
                    };
                    let resp = status
                        .json(serde_json::json!({
                            "error": "Invalid or expired token"
                        }))
                        .map_into_right_body();
                    return Ok(req.into_response(resp));
                }
            };

            // Build AuthSession
            let auth_session = SecureAuthSession {
                user_id: session.user_id.clone(),
                email: session.email,
                role: session.role.clone(),
                permissions: session.permissions.clone(),
                is_elevated: false,
                mfa_verified: false,
                refresh_token_id: None,
                session_id: session.session_id.clone(),
                token_id: "".to_string(),
                created_at: Default::default(),
                expires_at: Default::default(),
                device_fingerprint: session.device_fingerprint.clone(),
                ip_address: session.ip_address.clone(),

                last_activity: Default::default(),
                user_agent: None,
                refresh_expires_at: None,
            };

            // Enforce role and permission requirements if configured
            if let Some(ref min_role) = required_role {
                if !auth_session.has_minimum_role(min_role) {
                    let resp = HttpResponse::Forbidden()
                        .json(serde_json::json!({
                            "error": "Insufficient role",
                            "required_role": format!("{:?}", min_role)
                        }))
                        .map_into_right_body();
                    return Ok(req.into_response(resp));
                }
            }

            if let Some(ref perms) = required_permissions {
                if !auth_session.has_all_permissions(perms) {
                    let resp = HttpResponse::Forbidden()
                        .json(serde_json::json!({
                            "error": "Missing required permissions",
                            "required_permissions": perms
                        }))
                        .map_into_right_body();
                    return Ok(req.into_response(resp));
                }
            }

            // Attach session to request extensions for handlers to use
            req.extensions_mut().insert(auth_session);

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
impl FromRequest for SecureAuthSession {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(session) = req.extensions().get::<SecureAuthSession>() {
            ready(Ok(session.clone()))
        } else {
            ready(Err(actix_web::error::ErrorUnauthorized("No authentication session found")))
        }
    }
}
