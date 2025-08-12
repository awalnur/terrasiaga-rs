/// Enhanced authentication middleware with PASETO support
/// Provides secure token validation and role-based access control

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, web, FromRequest,
};
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

    /// Check if user has role level or higher
    pub fn has_role_level(&self, required_role: &UserRole) -> bool {
        self.role.has_permission_level(required_role)
    }
}

/// JWT middleware for backward compatibility
pub struct JwtMiddleware<S> {
    service: Rc<S>,
    paseto_service: Arc<PasetoService>,
}

impl<S, B> Service<ServiceRequest> for JwtMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let paseto_service = Arc::clone(&self.paseto_service);

        Box::pin(async move {
            // Extract authorization header
            let auth_header = req.headers().get("Authorization");

            if let Some(header_value) = auth_header {
                if let Ok(header_str) = header_value.to_str() {
                    if header_str.starts_with("Bearer ") {
                        let token = &header_str[7..];

                        match paseto_service.validate_token(token, TokenType::Access).await {
                            Ok(claims) => {
                                // Insert auth session into request extensions
                                let auth_session = AuthSession {
                                    user_id: claims.user_id,
                                    role: claims.role,
                                    permissions: claims.permissions,
                                    session_id: claims.session_id,
                                    device_fingerprint: claims.device_fingerprint,
                                    ip_address: req.connection_info().realip_remote_addr().map(String::from),
                                };
                                req.extensions_mut().insert(auth_session);
                            }
                            Err(e) => {
                                warn!("Token validation failed: {}", e);
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
            }

            service.call(req).await
        })
    }
}

/// JWT middleware factory for actix-web
pub struct JwtMiddlewareFactory {
    paseto_service: Arc<PasetoService>,
}

impl JwtMiddlewareFactory {
    pub fn new(paseto_service: Arc<PasetoService>) -> Self {
        Self { paseto_service }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddleware {
            service: Rc::new(service),
            paseto_service: Arc::clone(&self.paseto_service),
        }))
    }
}

/// Extractor for authentication session
impl FromRequest for AuthSession {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(auth_session) = req.extensions().get::<AuthSession>() {
            ready(Ok(auth_session.clone()))
        } else {
            ready(Err(AppError::Unauthorized("Authentication required".to_string())))
        }
    }
}

/// Authentication middleware configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub required_permissions: Vec<String>,
    pub required_role: Option<UserRole>,
    pub require_elevated: bool,
    pub require_mfa: bool,
    pub validate_device: bool,
    pub validate_ip: bool,
    pub allow_refresh_token: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            required_permissions: vec![],
            required_role: None,
            require_elevated: false,
            require_mfa: false,
            validate_device: false,
            validate_ip: false,
            allow_refresh_token: false,
        }
    }
}

impl AuthConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Require specific permissions
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.required_permissions = permissions;
        self
    }

    /// Require minimum role level
    pub fn with_role(mut self, role: UserRole) -> Self {
        self.required_role = Some(role);
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

    /// Enable device fingerprint validation
    pub fn validate_device(mut self) -> Self {
        self.validate_device = true;
        self
    }

    /// Enable IP address validation
    pub fn validate_ip(mut self) -> Self {
        self.validate_ip = true;
        self
    }

    /// Allow refresh tokens (normally only access tokens are allowed)
    pub fn allow_refresh_token(mut self) -> Self {
        self.allow_refresh_token = true;
        self
    }
}

/// Authentication middleware factory
pub struct PasetoAuthMiddleware {
    config: AuthConfig,
}

impl PasetoAuthMiddleware {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }

    /// Create middleware with default configuration
    pub fn default() -> Self {
        Self::new(AuthConfig::default())
    }

    /// Create middleware requiring specific permissions
    pub fn with_permissions(permissions: Vec<String>) -> Self {
        Self::new(AuthConfig::new().with_permissions(permissions))
    }

    /// Create middleware requiring minimum role
    pub fn with_role(role: UserRole) -> Self {
        Self::new(AuthConfig::new().with_role(role))
    }
}

impl<S, B> Transform<S, ServiceRequest> for PasetoAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
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
            service: Rc::new(service),
            config: self.config.clone(),
        }))
    }
}

pub struct PasetoAuthMiddlewareService<S> {
    service: Rc<S>,
    config: AuthConfig,
}

impl<S, B> Service<ServiceRequest> for PasetoAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let config = self.config.clone();

        Box::pin(async move {
            // Extract PASETO service from app data
            let paseto_service = match req.app_data::<web::Data<Arc<PasetoService>>>() {
                Some(service) => service.get_ref().clone(),
                None => {
                    error!("PASETO service not found in app data");
                    return Err(actix_web::error::ErrorInternalServerError("Authentication service unavailable"));
                }
            };

            // Extract authorization header
            let auth_header = req.headers().get("Authorization");
            let token = match auth_header {
                Some(header) => {
                    let header_str = match header.to_str() {
                        Ok(s) => s,
                        Err(_) => {
                            warn!("Invalid authorization header format");
                            return Err(actix_web::error::ErrorUnauthorized("Invalid authorization header"));
                        }
                    };

                    if header_str.starts_with("Bearer ") {
                        &header_str[7..]
                    } else {
                        warn!("Authorization header must start with 'Bearer '");
                        return Err(actix_web::error::ErrorUnauthorized("Invalid authorization scheme"));
                    }
                }
                None => {
                    debug!("No authorization header found");
                    return Err(actix_web::error::ErrorUnauthorized("Authorization header required"));
                }
            };

            // Get client information for validation
            let client_ip = req.connection_info().realip_remote_addr()
                .map(|ip| ip.to_string());

            let user_agent = req.headers().get("User-Agent")
                .and_then(|ua| ua.to_str().ok())
                .map(|ua| ua.to_string());

            // Validate token with requirements
            let required_permissions: Vec<String> = config.required_permissions;
            let validate_device = if config.validate_device { user_agent.as_deref() } else { None };
            let validate_ip = if config.validate_ip { client_ip.as_deref() } else { None };

            let claims = match paseto_service.validate_token_requirements(
                token,
                &required_permissions,
                config.required_role,
                validate_device,
                validate_ip,
            ) {
                Ok(claims) => claims,
                Err(AppError::Unauthorized(msg)) => {
                    warn!("Token validation failed: {}", msg);
                    return Err(actix_web::error::ErrorUnauthorized(msg));
                }
                Err(AppError::Forbidden(msg)) => {
                    warn!("Authorization failed: {}", msg);
                    return Err(actix_web::error::ErrorForbidden(msg));
                }
                Err(e) => {
                    error!("Authentication error: {}", e);
                    return Err(actix_web::error::ErrorInternalServerError("Authentication error"));
                }
            };

            // Validate token type
            match claims.token_type {
                TokenType::Access => {
                    // Access tokens are always allowed
                }
                TokenType::Refresh => {
                    if !config.allow_refresh_token {
                        warn!("Refresh token not allowed for this endpoint");
                        return Err(actix_web::error::ErrorUnauthorized("Access token required"));
                    }
                }
                TokenType::Reset | TokenType::Verify => {
                    warn!("Special purpose token not allowed for general access");
                    return Err(actix_web::error::ErrorUnauthorized("Invalid token type"));
                }
            }

            // Parse user ID
            let user_id = match uuid::Uuid::parse_str(&claims.sub) {
                Ok(id) => UserId(id),
                Err(_) => {
                    error!("Invalid user ID in token: {}", claims.sub);
                    return Err(actix_web::error::ErrorUnauthorized("Invalid token"));
                }
            };

            // Create authentication session
            let auth_session = AuthSession {
                user_id,
                role: claims.role,
                permissions: claims.permissions,
                session_id: claims.session_id,
                device_fingerprint: claims.device_fingerprint,
                ip_address: claims.ip_address,
            };

            // Store authentication session in request extensions
            req.extensions_mut().insert(auth_session);

            debug!("Authentication successful for user: {}", user_id.0);

            // Continue to next service
            service.call(req).await
        })
    }
}

/// Helper function to create authentication middleware with permissions
pub fn require_permissions(permissions: Vec<&str>) -> PasetoAuthMiddleware {
    PasetoAuthMiddleware::with_permissions(
        permissions.into_iter().map(|s| s.to_string()).collect()
    )
}

/// Helper function to create authentication middleware with role
pub fn require_role(role: UserRole) -> PasetoAuthMiddleware {
    PasetoAuthMiddleware::with_role(role)
}

/// Helper function to create authentication middleware for admin only
pub fn admin_only() -> PasetoAuthMiddleware {
    PasetoAuthMiddleware::with_role(UserRole::Admin)
}

/// Helper function to create authentication middleware for elevated operations
pub fn elevated_access() -> PasetoAuthMiddleware {
    PasetoAuthMiddleware::new(
        AuthConfig::new()
            .require_elevated()
            .validate_device()
            .validate_ip()
    )
}

/// Authentication middleware factory
pub struct AuthMiddleware {
    required_permissions: Vec<String>,
    minimum_role: Option<UserRole>,
    allow_expired_for_refresh: bool,
}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {
            required_permissions: vec![],
            minimum_role: None,
            allow_expired_for_refresh: false,
        }
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.required_permissions = permissions;
        self
    }

    pub fn with_minimum_role(mut self, role: UserRole) -> Self {
        self.minimum_role = Some(role);
        self
    }

    pub fn allow_expired_for_refresh(mut self) -> Self {
        self.allow_expired_for_refresh = true;
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
            service: Rc::new(service),
            required_permissions: self.required_permissions.clone(),
            minimum_role: self.minimum_role.clone(),
            allow_expired_for_refresh: self.allow_expired_for_refresh,
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    required_permissions: Vec<String>,
    minimum_role: Option<UserRole>,
    allow_expired_for_refresh: bool,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
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
        let service = Rc::clone(&self.service);
        let required_permissions = self.required_permissions.clone();
        let minimum_role = self.minimum_role.clone();
        let allow_expired_for_refresh = self.allow_expired_for_refresh;

        Box::pin(async move {
            // Extract auth header
            let auth_header = req.headers().get("Authorization");

            if let Some(header_value) = auth_header {
                if let Ok(auth_str) = header_value.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = &auth_str[7..];

                        // Get services from app data
                        if let Some(paseto_service) = req.app_data::<web::Data<Arc<PasetoService>>>() {
                            // Extract client info
                            let client_ip = req.connection_info().realip_remote_addr()
                                .map(|ip| ip.to_string());

                            let device_fingerprint = req.headers()
                                .get("X-Device-Fingerprint")
                                .and_then(|v| v.to_str().ok())
                                .map(|s| s.to_string());

                            // Validate token with context
                            match paseto_service.validate_token_with_context(
                                token,
                                client_ip.as_deref(),
                                device_fingerprint.as_deref(),
                            ) {
                                Ok(claims) => {
                                    // Check if it's an access token (or allow refresh tokens for specific endpoints)
                                    if claims.token_type != crate::shared::paseto_auth::TokenType::Access && !allow_expired_for_refresh {
                                        warn!("Invalid token type for request: {:?}", claims.token_type);
                                        return Ok(req.into_response(
                                            HttpResponse::Unauthorized()
                                                .json(serde_json::json!({
                                                    "error": "Invalid token type",
                                                    "message": "Access token required"
                                                }))
                                                .into_body()
                                        ));
                                    }

                                    let session = AuthSession {
                                        user_id: UserId(uuid::Uuid::parse_str(&claims.sub).unwrap()),
                                        role: claims.role.clone(),
                                        permissions: claims.permissions.clone(),
                                        session_id: claims.session_id.clone(),
                                        device_fingerprint: claims.device_fingerprint.clone(),
                                        ip_address: claims.ip_address.clone(),
                                    };

                                    // Check minimum role requirement
                                    if let Some(ref min_role) = minimum_role {
                                        if !session.has_minimum_role(min_role) {
                                            warn!("Insufficient role for user {}: required {:?}, has {:?}",
                                                  session.user_id.0, min_role, session.role);
                                            return Ok(req.into_response(
                                                HttpResponse::Forbidden()
                                                    .json(serde_json::json!({
                                                        "error": "Insufficient permissions",
                                                        "message": "Higher role required"
                                                    }))
                                                    .into_body()
                                            ));
                                        }
                                    }

                                    // Check required permissions
                                    if !required_permissions.is_empty() {
                                        let required_refs: Vec<&str> = required_permissions.iter().map(|s| s.as_str()).collect();
                                        if !session.has_all_permissions(&required_refs) {
                                            warn!("Insufficient permissions for user {}: required {:?}, has {:?}",
                                                  session.user_id.0, required_permissions, session.permissions);
                                            return Ok(req.into_response(
                                                HttpResponse::Forbidden()
                                                    .json(serde_json::json!({
                                                        "error": "Insufficient permissions",
                                                        "message": "Required permissions not granted",
                                                        "required": required_permissions
                                                    }))
                                                    .into_body()
                                            ));
                                        }
                                    }

                                    // Insert session into request extensions
                                    req.extensions_mut().insert(session);

                                    // Log successful authentication
                                    debug!("Authenticated user: {} with role: {:?}", claims.sub, claims.role);

                                    return service.call(req).await;
                                }
                                Err(e) => {
                                    warn!("Token validation failed: {}", e);
                                }
                            }
                        }
                    }
                }
            }

            // Authentication failed
            Ok(req.into_response(
                HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "Authentication required",
                        "message": "Valid authentication token required"
                    }))
                    .into_body()
            ))
        })
    }
}

/// Convenience macros for role-based route protection
#[macro_export]
macro_rules! require_role {
    ($role:expr) => {
        crate::middleware::auth::AuthMiddleware::new()
            .with_minimum_role($role)
    };
}

#[macro_export]
macro_rules! require_permissions {
    ($($perm:expr),+) => {
        crate::middleware::auth::AuthMiddleware::new()
            .with_permissions(vec![$($perm.to_string()),+])
    };
}

#[macro_export]
macro_rules! require_role_and_permissions {
    ($role:expr, $($perm:expr),+) => {
        crate::middleware::auth::AuthMiddleware::new()
            .with_minimum_role($role)
            .with_permissions(vec![$($perm.to_string()),+])
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};
    use crate::shared::paseto_auth::utils;

    async fn test_handler(session: AuthSession) -> HttpResponse {
        HttpResponse::Ok().json(serde_json::json!({
            "user_id": session.user_id.0,
            "role": session.role,
            "permissions": session.permissions
        }))
    }

    #[actix_web::test]
    async fn test_auth_middleware_success() {
        let key = utils::generate_secure_key();
        let paseto_service = Arc::new(PasetoService::new(&key).unwrap());

        let user_id = UserId(uuid::Uuid::new_v4());
        let role = UserRole::User;
        let permissions = vec!["read".to_string()];
        let session_id = utils::generate_session_id();

        let token_pair = paseto_service.generate_token_pair(
            user_id,
            role,
            permissions,
            session_id,
            None,
            None,
        ).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(paseto_service))
                .service(
                    web::resource("/test")
                        .wrap(PasetoAuthMiddleware::default())
                        .route(web::get().to(test_handler))
                )
        ).await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("Authorization", format!("Bearer {}", token_pair.access_token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_auth_middleware_no_token() {
        let key = utils::generate_secure_key();
        let paseto_service = Arc::new(PasetoService::new(&key).unwrap());

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(paseto_service))
                .service(
                    web::resource("/test")
                        .wrap(PasetoAuthMiddleware::default())
                        .route(web::get().to(test_handler))
                )
        ).await;

        let req = test::TestRequest::get()
            .uri("/test")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn test_auth_middleware_insufficient_permissions() {
        let key = utils::generate_secure_key();
        let paseto_service = Arc::new(PasetoService::new(&key).unwrap());

        let user_id = UserId(uuid::Uuid::new_v4());
        let role = UserRole::User;
        let permissions = vec!["read".to_string()];
        let session_id = utils::generate_session_id();

        let token_pair = paseto_service.generate_token_pair(
            user_id,
            role,
            permissions,
            session_id,
            None,
            None,
        ).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(paseto_service))
                .service(
                    web::resource("/test")
                        .wrap(require_permissions(vec!["admin"]))
                        .route(web::get().to(test_handler))
                )
        ).await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("Authorization", format!("Bearer {}", token_pair.access_token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 403);
    }
}
