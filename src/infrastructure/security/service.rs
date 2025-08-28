/// Security service implementation for Terra Siaga
/// Handles authentication, authorization, rate limiting, and security policies

use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use uuid::Uuid;

use crate::domain::entities::User;
use crate::domain::value_objects::{UserRole, Email};
use crate::shared::UserId;
use crate::domain::ports::services::AuthService;
use crate::infrastructure::cache::CacheService;
use crate::shared::{AppResult, AppError};
use crate::domain::ports::services::TokenPair;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,        // User ID
    pub email: String,      // User email
    pub role: String,       // User role
    pub session_id: String, // Session ID for revocation
    pub iat: u64,          // Issued at
    pub exp: u64,          // Expiration time
    pub aud: String,       // Audience (terra-siaga)
    pub iss: String,       // Issuer
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub user_id: UserId,
    pub email: Email,
    pub role: UserRole,
    pub session_id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub last_activity: u64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Simple rate limiting configuration for this service
#[derive(Debug, Clone)]
pub struct SimpleRateLimitConfig {
    pub max_requests: u32,
    pub window_seconds: u64,
    pub burst_allowance: u32,
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub session_timeout_hours: u64,
    pub max_failed_attempts: u32,
    pub lockout_duration_minutes: u64,
    pub password_min_length: usize,
    pub require_special_chars: bool,
    pub rate_limits: HashMap<String, SimpleRateLimitConfig>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut rate_limits = HashMap::new();

        // Default rate limits for different endpoints
        rate_limits.insert(
            "auth_login".to_string(),
            SimpleRateLimitConfig { max_requests: 5, window_seconds: 300, burst_allowance: 2 },
        );

        rate_limits.insert(
            "disaster_report".to_string(),
            SimpleRateLimitConfig { max_requests: 10, window_seconds: 3600, burst_allowance: 3 },
        );

        rate_limits.insert(
            "general_api".to_string(),
            SimpleRateLimitConfig { max_requests: 100, window_seconds: 60, burst_allowance: 20 },
        );

        Self {
            jwt_secret: "your-super-secret-jwt-key-change-in-production".to_string(),
            jwt_expiration_hours: 24,
            session_timeout_hours: 72,
            max_failed_attempts: 5,
            lockout_duration_minutes: 30,
            password_min_length: 8,
            require_special_chars: true,
            rate_limits,
        }
    }
}

/// Production security service implementation
pub struct ProductionSecurityService {
    config: SecurityConfig,
    cache: Arc<dyn CacheService>,
    argon2: Argon2<'static>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl ProductionSecurityService {
    pub fn new(config: SecurityConfig, cache: Arc<dyn CacheService>) -> AppResult<Self> {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_ref());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_ref());

        Ok(Self { config, cache, argon2: Argon2::default(), encoding_key, decoding_key })
    }

    /// Generate secure session ID
    fn generate_session_id() -> String { Uuid::new_v4().to_string() }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    /// Create JWT token for user
    pub async fn create_jwt_token(&self, user: &User, ip_address: Option<String>, user_agent: Option<String>) -> AppResult<String> {
        let session_id = Self::generate_session_id();
        let now = Self::current_timestamp();
        let exp = now + (self.config.jwt_expiration_hours * 3600);

        let claims = JwtClaims {
            sub: user.id().value().to_string(),
            email: user.email().value().to_string(),
            role: format!("{:?}", user.role()),
            session_id: session_id.clone(),
            iat: now,
            exp,
            aud: "terra-siaga".to_string(),
            iss: "terra-siaga-auth".to_string(),
        };

        // Store session in cache
        let session = AuthSession {
            user_id: *user.id(),
            email: user.email().clone(),
            role: user.role().clone(),
            session_id: session_id.clone(),
            created_at: now,
            expires_at: now + (self.config.session_timeout_hours * 3600),
            last_activity: now,
            ip_address,
            user_agent,
        };

        let session_key = format!("session:{}", session_id);
        let session_json = serde_json::to_string(&session)
            .map_err(|e| AppError::InternalServer(format!("Failed to serialize session: {}", e)))?;
        self.cache
            .set_string(&session_key, session_json, Some(Duration::from_secs(self.config.session_timeout_hours * 3600)))
            .await?;

        // Generate JWT
        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::InternalServer(format!("JWT encoding failed: {}", e)))?;

        Ok(token)
    }

    /// Validate JWT token and return session
    pub async fn validate_jwt_token(&self, token: &str) -> AppResult<AuthSession> {
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

        let claims = token_data.claims;

        // Check if token is expired
        let now = Self::current_timestamp();
        if claims.exp < now { return Err(AppError::Unauthorized("Token expired".to_string())); }

        // Get session from cache
        let session_key = format!("session:{}", claims.session_id);
        let session_json = self.cache.get_string(&session_key).await?;
        let session_str = session_json.ok_or_else(|| AppError::Unauthorized("Session not found".to_string()))?;
        let session: AuthSession = serde_json::from_str(&session_str)
            .map_err(|e| AppError::InternalServer(format!("Failed to deserialize session: {}", e)))?;

        // Check session expiration
        if session.expires_at < now {
            self.cache.delete(&session_key).await?;
            return Err(AppError::Unauthorized("Session expired".to_string()));
        }

        Ok(session)
    }

    /// Revoke user session
    pub async fn revoke_session(&self, session_id: &str) -> AppResult<()> {
        let session_key = format!("session:{}", session_id);
        self.cache.delete(&session_key).await?;
        Ok(())
    }

    /// Check and enforce rate limits (simple counter)
    pub async fn check_rate_limit(&self, user_id: &str, action: &str, ip_address: Option<&str>) -> AppResult<bool> {
        let config = self
            .config
            .rate_limits
            .get(action)
            .unwrap_or(self.config.rate_limits.get("general_api").expect("general_api rate limit must exist"));

        // Create rate limit key based on user or IP
        let rate_key = if !user_id.is_empty() {
            format!("rate_limit:user:{}:{}", user_id, action)
        } else if let Some(ip) = ip_address { format!("rate_limit:ip:{}:{}", ip, action) } else { format!("rate_limit:anonymous:{}", action) };

        // Get current request count
        let current_count = match self.cache.get_string(&rate_key).await? {
            Some(v) => v.parse::<i64>().unwrap_or(0),
            None => 0,
        };

        if current_count as u32 >= config.max_requests { return Ok(false); }

        // Increment counter
        let new_count = self.cache.increment(&rate_key, 1).await?;

        // Set expiration on first request
        if new_count == 1 { self.cache.expire(&rate_key, Duration::from_secs(config.window_seconds)).await?; }

        Ok(true)
    }

    /// Track failed login attempts
    pub async fn track_failed_login(&self, email: &str, ip_address: Option<&str>) -> AppResult<bool> {
        let key = format!("failed_login:{}", email);
        let attempts = self.cache.increment(&key, 1).await?;

        if attempts == 1 { // Set expiration for failed attempts tracking
            self.cache.expire(&key, Duration::from_secs(self.config.lockout_duration_minutes * 60)).await?;
        }

        if attempts as u32 >= self.config.max_failed_attempts {
            // Account is locked
            let lockout_key = format!("lockout:{}", email);
            self.cache.set_string(&lockout_key, "true".to_string(), Some(Duration::from_secs(self.config.lockout_duration_minutes * 60))).await?;

            // Also track IP-based lockout if available
            if let Some(ip) = ip_address {
                let ip_lockout_key = format!("lockout:ip:{}", ip);
                self.cache.set_string(&ip_lockout_key, "true".to_string(), Some(Duration::from_secs(self.config.lockout_duration_minutes * 60))).await?;
            }

            return Ok(false); // Account locked
        }

        Ok(true) // Still allowed
    }

    /// Check if account is locked
    pub async fn is_account_locked(&self, email: &str, ip_address: Option<&str>) -> AppResult<bool> {
        let lockout_key = format!("lockout:{}", email);
        let is_locked = self.cache.exists(&lockout_key).await?;
        if is_locked { return Ok(true); }

        // Also check IP-based lockout
        if let Some(ip) = ip_address {
            let ip_lockout_key = format!("lockout:ip:{}", ip);
            let ip_locked = self.cache.exists(&ip_lockout_key).await?;
            if ip_locked { return Ok(true); }
        }

        Ok(false)
    }

    /// Clear failed login attempts after successful login
    pub async fn clear_failed_attempts(&self, email: &str) -> AppResult<()> {
        let key = format!("failed_login:{}", email);
        self.cache.delete(&key).await?;
        let lockout_key = format!("lockout:{}", email);
        self.cache.delete(&lockout_key).await?;
        Ok(())
    }

    /// Validate password strength
    pub fn validate_password_strength(&self, password: &str) -> AppResult<()> {
        if password.len() < self.config.password_min_length {
            return Err(AppError::Validation(format!(
                "Password must be at least {} characters long",
                self.config.password_min_length
            )));
        }

        if self.config.require_special_chars {
            let has_uppercase = password.chars().any(|c| c.is_uppercase());
            let has_lowercase = password.chars().any(|c| c.is_lowercase());
            let has_digit = password.chars().any(|c| c.is_ascii_digit());
            let has_special = password.chars().any(|c| !c.is_alphanumeric());

            if !has_uppercase { return Err(AppError::Validation("Password must contain at least one uppercase letter".to_string())); }
            if !has_lowercase { return Err(AppError::Validation("Password must contain at least one lowercase letter".to_string())); }
            if !has_digit { return Err(AppError::Validation("Password must contain at least one digit".to_string())); }
            if !has_special { return Err(AppError::Validation("Password must contain at least one special character".to_string())); }
        }

        Ok(())
    }

    /// Update session activity
    pub async fn update_session_activity(&self, session_id: &str) -> AppResult<()> {
        let session_key = format!("session:{}", session_id);

        if let Some(session_json) = self.cache.get_string(&session_key).await? {
            let mut session: AuthSession = serde_json::from_str(&session_json)
                .map_err(|e| AppError::InternalServer(format!("Failed to deserialize session: {}", e)))?;
            session.last_activity = Self::current_timestamp();
            let updated = serde_json::to_string(&session)
                .map_err(|e| AppError::InternalServer(format!("Failed to serialize session: {}", e)))?;
            self.cache.set_string(&session_key, updated, Some(Duration::from_secs(self.config.session_timeout_hours * 3600))).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl AuthService for ProductionSecurityService {
    async fn hash_password(&self, password: &str) -> AppResult<String> {
        // Validate password strength first
        self.validate_password_strength(password)?;

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::InternalServer(format!("Password hashing failed: {}", e)))?
            .to_string();

        Ok(password_hash)
    }

    async fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::InternalServer(format!("Invalid password hash: {}", e)))?;

        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn generate_token(&self, user: &User) -> AppResult<String> {
        self.create_jwt_token(user, None, None).await
    }

    async fn verify_token(&self, token: &str) -> AppResult<UserId> {
        let session = self.validate_jwt_token(token).await?;
        Ok(session.user_id)
    }

    async fn generate_tokens(&self, _user_id: UserId) -> AppResult<TokenPair> {
        Err(AppError::Configuration("generate_tokens is not supported by JWT-based ProductionSecurityService; use PasetoService or call generate_token with a User".to_string()))
    }

    async fn refresh_token(&self, _refresh_token: &str) -> AppResult<TokenPair> {
        Err(AppError::Configuration("refresh_token is not supported by JWT-based ProductionSecurityService; use PasetoService".to_string()))
    }

    async fn revoke_token(&self, token: &str) -> AppResult<()> {
        // Extract session ID from token while ignoring expiration (still verifies signature)
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;
        self.revoke_session(&token_data.claims.session_id).await
    }

    async fn track_failed_login(&self, email: &str, ip_address: Option<&str>) -> AppResult<bool> {
        ProductionSecurityService::track_failed_login(self, email, ip_address).await
    }

    async fn is_account_locked(&self, email: &str, ip_address: Option<&str>) -> AppResult<bool> {
        ProductionSecurityService::is_account_locked(self, email, ip_address).await
    }

    async fn clear_failed_attempts(&self, email: &str) -> AppResult<()> {
        ProductionSecurityService::clear_failed_attempts(self, email).await
    }
}
