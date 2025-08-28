/// PASETO-based security service for Terra Siaga
/// Provides more secure token management compared to JWT with built-in encryption

use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use pasetors::keys::{SymmetricKey, AsymmetricSecretKey, AsymmetricPublicKey};
use pasetors::token::UntrustedToken;
use pasetors::{Local, Public, version4::V4};
use pasetors::claims::{Claims, ClaimsValidationRules};
use argon2::Argon2;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use crate::domain::entities::User;
use crate::domain::value_objects::{ UserRole, Email};
use crate::infrastructure::cache::{CacheService};
use crate::shared::{AppResult, AppError};
use crate::UserId;

/// PASETO token claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct PasetoTokenClaims {
    pub sub: String,        // Subject (User ID)
    pub email: String,      // User email
    pub role: String,       // User role
    pub session_id: String, // Session ID for revocation
    pub iat: String,        // Issued at (ISO 8601)
    pub exp: String,        // Expiration time (ISO 8601)
    pub aud: String,        // Audience
    pub iss: String,        // Issuer
    pub jti: String,        // JWT ID (unique token identifier)
    pub scope: Vec<String>, // Token scopes/permissions
}

/// Enhanced authentication session with more security features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureAuthSession {
    pub user_id: UserId,
    pub email: Email,
    pub role: UserRole,
    pub session_id: String,
    pub token_id: String,        // Unique token identifier
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
    pub permissions: Vec<String>,
    pub is_elevated: bool,       // For sensitive operations
    pub mfa_verified: bool,      // Multi-factor authentication status
}

/// PASETO configuration
#[derive(Debug, Clone)]
pub struct PasetoConfig {
    pub local_key: Vec<u8>,      // For encrypted local tokens
    pub public_key: Vec<u8>,     // For signed public tokens
    pub private_key: Vec<u8>,    // For signing public tokens
    pub token_expiration_hours: u64,
    pub session_timeout_hours: u64,
    pub elevated_session_minutes: u64,
    pub use_local_tokens: bool,   // Use encrypted tokens vs signed tokens
}

impl Default for PasetoConfig {
    fn default() -> Self {
        // Generate secure random keys (in production, these should be from config)
        let local_key = (0..32).map(|_| rand::random::<u8>()).collect();
        let (public_key, private_key) = Self::generate_asymmetric_keys();

        Self {
            local_key,
            public_key,
            private_key,
            token_expiration_hours: 24,
            session_timeout_hours: 72,
            elevated_session_minutes: 15,
            use_local_tokens: true, // Default to encrypted tokens for better security
        }
    }
}

impl PasetoConfig {
    fn generate_asymmetric_keys() -> (Vec<u8>, Vec<u8>) {
        // In production, use proper key generation
        let private_key: Vec<u8> = (0..64).map(|_| rand::random::<u8>()).collect();
        let public_key: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
        (public_key, private_key)
    }
}

/// Enhanced PASETO security service
pub struct PasetoSecurityService {
    config: PasetoConfig,
    cache: Arc<dyn CacheService>,
    argon2: Argon2<'static>,
    local_key: SymmetricKey<V4>,
    private_key: Option<AsymmetricSecretKey<V4>>,
    public_key: Option<AsymmetricPublicKey<V4>>,
}

impl PasetoSecurityService {
    pub fn new(config: PasetoConfig, cache: Arc<dyn CacheService>) -> AppResult<Self> {
        // Initialize PASETO keys
        let local_key = SymmetricKey::<V4>::from(&config.local_key)
            .map_err(|e| AppError::InternalServer(format!("Invalid local key: {}", e)))?;

        let (private_key, public_key) = if !config.use_local_tokens {
            let private_key = AsymmetricSecretKey::<V4>::from(&config.private_key)
                .map_err(|e| AppError::InternalServer(format!("Invalid private key: {}", e)))?;
            let public_key = AsymmetricPublicKey::<V4>::from(&config.public_key)
                .map_err(|e| AppError::InternalServer(format!("Invalid public key: {}", e)))?;
            (Some(private_key), Some(public_key))
        } else {
            (None, None)
        };

        Ok(Self {
            config,
            cache,
            argon2: Argon2::default(),
            local_key,
            private_key,
            public_key,
        })
    }

    /// Generate secure session ID
    fn generate_session_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Generate unique token ID
    fn generate_token_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Get user permissions based on role
    fn get_user_permissions(role: &UserRole) -> Vec<String> {
        match role {
            UserRole::Citizen => vec![
                "report_disaster".to_string(),
                "view_disasters".to_string(),
                "receive_notifications".to_string(),
            ],
            UserRole::Volunteer => vec![
                "report_disaster".to_string(),
                "view_disasters".to_string(),
                "receive_notifications".to_string(),
                "volunteer_response".to_string(),
            ],
            UserRole::Admin => vec![
                "report_disaster".to_string(),
                "view_disasters".to_string(),
                "receive_notifications".to_string(),
                "respond_to_disaster".to_string(),
                "update_disaster_status".to_string(),
                "manage_users".to_string(),
                "manage_disasters".to_string(),
                "send_emergency_alerts".to_string(),
            ],
            UserRole::SystemAdmin => vec![
                "all_permissions".to_string(),

            ],
            UserRole::SuperAdmin => vec![
                "all_permissions".to_string(),
            ],
            _ => vec![]

        }
    }

    /// Create PASETO token for user
    pub async fn create_paseto_token(
        &self,
        user: &User,
        ip_address: Option<String>,
        user_agent: Option<String>,
        device_fingerprint: Option<String>,
    ) -> AppResult<String> {
        let session_id = Self::generate_session_id();
        let token_id = Self::generate_token_id();
        let now = Utc::now();
        let exp = now + ChronoDuration::hours(self.config.token_expiration_hours as i64);

        let permissions = Self::get_user_permissions(user.role());

        // Create token claims
        let claims = PasetoTokenClaims {
            sub: user.id().to_string(),
            email: user.email().value().to_string(),
            role: format!("{:?}", user.role()),
            session_id: session_id.clone(),
            iat: now.to_rfc3339(),
            exp: exp.to_rfc3339(),
            aud: "terra-siaga".to_string(),
            iss: "terra-siaga-auth".to_string(),
            jti: token_id.clone(),
            scope: permissions.clone(),
        };

        // Store session in cache
        let session = SecureAuthSession {
            user_id: user.id.clone(),
            email: user.email().clone(),
            role: user.role().clone(),
            session_id: session_id.clone(),
            token_id: token_id.clone(),
            created_at: now,
            expires_at: now + ChronoDuration::hours(self.config.session_timeout_hours as i64),
            last_activity: now,
            ip_address,
            user_agent,
            device_fingerprint,
            permissions,
            is_elevated: false,
            mfa_verified: false, // Should be set based on actual MFA verification
        };

        let session_key = format!("session:{}", session_id);
        let session_json = serde_json::to_string(&session)
            .map_err(|e| AppError::InternalServer(format!("Failed to serialize session: {}", e)))?;

        self.cache.set_string(&session_key, session_json, Some(Duration::from_secs(self.config.session_timeout_hours * 3600))).await?;

        // Create PASETO token
        let claims_json = serde_json::to_string(&claims)
            .map_err(|e| AppError::InternalServer(format!("Failed to serialize claims: {}", e)))?;


        let paseto_claims = Claims::from_bytes(claims_json.as_bytes()).map_err(|e| AppError::InternalServer(format!("Failed to create PASETO claims: {}", e)))?;
        let token = if self.config.use_local_tokens {
            // Use encrypted local tokens (v4.local)
            pasetors::local::encrypt(&self.local_key, &paseto_claims, None, None)
                .map_err(|e| AppError::InternalServer(format!("PASETO encryption failed: {}", e)))?
        } else {
            // Use signed public tokens (v4.public)
            if let Some(private_key) = &self.private_key {
                pasetors::public::sign(private_key, &paseto_claims, None, None)
                    .map_err(|e| AppError::InternalServer(format!("PASETO signing failed: {}", e)))?
            } else {
                return Err(AppError::InternalServer("Private key not available for public tokens".to_string()));
            }
        };

        Ok(token)
    }

    /// Validate PASETO token and return session
    pub async fn validate_paseto_token(&self, token: &str) -> AppResult<SecureAuthSession> {
        let trusted_token = if self.config.use_local_tokens {
            // Parse as local token and decrypt
            let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
                .map_err(|e| AppError::Unauthorized(format!("Invalid local token format: {}", e)))?;

            // Decrypt local token with proper parameters
            let validation_rules = ClaimsValidationRules::new();
            pasetors::local::decrypt(&self.local_key, &untrusted_token, &validation_rules, None, None)
                .map_err(|e| AppError::Unauthorized(format!("Token decryption failed: {}", e)))?
        } else {
            // Parse as public token and verify
            let untrusted_token = UntrustedToken::<Public, V4>::try_from(token)
                .map_err(|e| AppError::Unauthorized(format!("Invalid public token format: {}", e)))?;

            if let Some(public_key) = &self.public_key {
                let validation_rules = ClaimsValidationRules::new();
                pasetors::public::verify(public_key, &untrusted_token, &validation_rules, None, None)
                    .map_err(|e| AppError::Unauthorized(format!("Token verification failed: {}", e)))?
            } else {
                return Err(AppError::InternalServer("Public key not available for token verification".to_string()));
            }
        };

        // Parse claims from payload
        let claims_str = String::from_utf8(trusted_token.payload().as_bytes().to_vec())
            .map_err(|e| AppError::Unauthorized(format!("Invalid token payload: {}", e)))?;
        
        let claims: PasetoTokenClaims = serde_json::from_str(&claims_str)
            .map_err(|e| AppError::Unauthorized(format!("Invalid token claims: {}", e)))?;

        // Validate expiration
        let exp_time = DateTime::parse_from_rfc3339(&claims.exp)
            .map_err(|e| AppError::Unauthorized(format!("Invalid expiration time: {}", e)))?
            .with_timezone(&Utc);
        
        if exp_time < Utc::now() {
            return Err(AppError::Unauthorized("Token expired".to_string()));
        }

        // Get session from cache
        let session_key = format!("session:{}", claims.session_id);
        let session_json: String = self.cache.get_string(&session_key).await?
            .ok_or_else(|| AppError::Unauthorized("Session not found".to_string()))?;

        let mut session: SecureAuthSession = serde_json::from_str(&session_json)
            .map_err(|e| AppError::InternalServer(format!("Failed to deserialize session: {}", e)))?;

        // Check session expiration
        if session.expires_at < Utc::now() {
            self.cache.delete(&session_key).await?;
            return Err(AppError::Unauthorized("Session expired".to_string()));
        }

        // Validate token ID matches
        if session.token_id != claims.jti {
            return Err(AppError::Unauthorized("Token ID mismatch".to_string()));
        }

        // Update last activity
        session.last_activity = Utc::now();
        let updated_session_json = serde_json::to_string(&session)
            .map_err(|e| AppError::InternalServer(format!("Failed to serialize session: {}", e)))?;
        self.cache.set_string(&session_key, updated_session_json, Some(Duration::from_secs(self.config.session_timeout_hours * 3600))).await?;

        Ok(session)
    }

    /// Create elevated session for sensitive operations
    pub async fn create_elevated_session(&self, session_id: &str, mfa_token: Option<&str>) -> AppResult<String> {
        let session_key = format!("session:{}", session_id);
        let session_json: String = self.cache.get_string(&session_key).await?
            .ok_or_else(|| AppError::Unauthorized("Session not found".to_string()))?;

        let mut session: SecureAuthSession = serde_json::from_str(&session_json)
            .map_err(|e| AppError::InternalServer(format!("Failed to deserialize session: {}", e)))?;

        // Verify MFA if required for elevation
        if let Some(_mfa_token) = mfa_token {
            // TODO: Implement MFA verification
            session.mfa_verified = true;
        }

        // Mark session as elevated
        session.is_elevated = true;
        session.last_activity = Utc::now();

        // Set shorter expiration for elevated session
        let elevated_exp = Utc::now() + ChronoDuration::minutes(self.config.elevated_session_minutes as i64);
        session.expires_at = elevated_exp;

        // Update session in cache with shorter TTL
        let updated_session_json = serde_json::to_string(&session)
            .map_err(|e| AppError::InternalServer(format!("Failed to serialize session: {}", e)))?;
        self.cache.set_string(&session_key, updated_session_json, Some(Duration::from_secs(self.config.elevated_session_minutes * 60))).await?;

        Ok(session_id.to_string())
    }

    /// Revoke session and invalidate all related tokens
    pub async fn revoke_session(&self, session_id: &str) -> AppResult<()> {
        let session_key = format!("session:{}", session_id);
        
        // Add to revocation list for additional security
        let revocation_key = format!("revoked:{}", session_id);
        self.cache.set_string(&revocation_key, "true".to_string(), Some(Duration::from_secs(self.config.session_timeout_hours * 3600))).await?;

        // Remove session
        self.cache.delete(&session_key).await?;
        
        Ok(())
    }

    /// Check if session is revoked
    pub async fn is_session_revoked(&self, session_id: &str) -> AppResult<bool> {
        let revocation_key = format!("revoked:{}", session_id);
        Ok(self.cache.exists(&revocation_key).await?)
    }
}
