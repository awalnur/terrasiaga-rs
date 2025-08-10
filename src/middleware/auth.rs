/// Enhanced authentication middleware with PASETO support
/// Provides secure token validation and role-based access control

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::sync::Arc;

use crate::infrastructure::security::{PasetoSecurityService, SecureAuthSession};
use crate::domain::value_objects::UserRole;
use crate::shared::AppError;

/// Authentication middleware factory
pub struct PasetoAuthMiddleware {
    security_service: Arc<PasetoSecurityService>,
    required_permissions: Vec<String>,
    require_elevated: bool,
    require_mfa: bool,
}

impl PasetoAuthMiddleware {
    pub fn new(security_service: Arc<PasetoSecurityService>) -> Self {
        Self {
            security_service,
            required_permissions: vec![],
            require_elevated: false,
            require_mfa: false,
        }
    }

    /// Require specific permissions
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.required_permissions = permissions;
        self
    }

    /// Require elevated session for sensitive operations
    pub fn require_elevated(mut self) -> Self {
        self.require_elevated = true;
        self
    }

    /// Require MFA verification
    pub fn require_mfa(mut self) -> Self {
        self.require_mfa = true;
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for PasetoAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PasetoAuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PasetoAuthMiddlewareService {
            service,
            security_service: self.security_service.clone(),
            required_permissions: self.required_permissions.clone(),
            require_elevated: self.require_elevated,
            require_mfa: self.require_mfa,
        }))
    }
}

pub struct PasetoAuthMiddlewareService<S> {
    service: S,
    security_service: Arc<PasetoSecurityService>,
    required_permissions: Vec<String>,
    require_elevated: bool,
    require_mfa: bool,
}

impl<S, B> Service<ServiceRequest> for PasetoAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let security_service = self.security_service.clone();
        let required_permissions = self.required_permissions.clone();
        let require_elevated = self.require_elevated;
        let require_mfa = self.require_mfa;

        Box::pin(async move {
            // Extract Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| {
                    if h.starts_with("Bearer ") {
                        Some(h[7..].to_string())
                    } else {
                        None
                    }
                });

            let token = match auth_header {
                Some(token) => token,
                None => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "unauthorized",
                            "message": "Missing or invalid Authorization header"
                        }));
                    return Ok(req.into_response(response));
                }
            };

            // Validate PASETO token
            let session = match security_service.validate_paseto_token(&token).await {
                Ok(session) => session,
                Err(e) => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "unauthorized",
                            "message": format!("Token validation failed: {}", e)
                        }));
                    return Ok(req.into_response(response));
                }
            };

            // Check if session is revoked
            if let Ok(true) = security_service.is_session_revoked(&session.session_id).await {
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "unauthorized",
                        "message": "Session has been revoked"
                    }));
                return Ok(req.into_response(response));
            }

            // Check required permissions
            if !required_permissions.is_empty() {
                let has_permission = if session.permissions.contains(&"all_permissions".to_string()) {
                    true
                } else {
                    required_permissions.iter().all(|perm| session.permissions.contains(perm))
                };

                if !has_permission {
                    let response = HttpResponse::Forbidden()
                        .json(serde_json::json!({
                            "error": "forbidden",
                            "message": "Insufficient permissions"
                        }));
                    return Ok(req.into_response(response));
                }
            }

            // Check elevated session requirement
            if require_elevated && !session.is_elevated {
                let response = HttpResponse::Forbidden()
                    .json(serde_json::json!({
                        "error": "forbidden",
                        "message": "Elevated session required for this operation"
                    }));
                return Ok(req.into_response(response));
            }

            // Check MFA requirement
            if require_mfa && !session.mfa_verified {
                let response = HttpResponse::Forbidden()
                    .json(serde_json::json!({
                        "error": "forbidden",
                        "message": "Multi-factor authentication required"
                    }));
                return Ok(req.into_response(response));
            }

            // Add session to request extensions
            req.extensions_mut().insert(session);

            // Continue with the request
            let res = self.service.call(req).await?;
            Ok(res)
        })
    }
}
