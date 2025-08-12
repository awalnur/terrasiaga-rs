/// PASETO authentication implementation for Terra Siaga
/// More secure alternative to JWT with built-in encryption and authenticity

pub mod utils;

use pasetors::claims::{Claims, ClaimsValidationRules};
use pasetors::keys::SymmetricKey;
use pasetors::token::UntrustedToken;
use pasetors::{local, Local, version4::V4};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use time::{Duration, OffsetDateTime};
use zeroize::Zeroize;
use tracing::{error, warn, info};

use crate::shared::error::{AppError, AppResult};
use crate::shared::types::{UserId, UserRole};

/// PASETO token claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at timestamp
    pub iat: OffsetDateTime,
    /// Expiration timestamp
    pub exp: OffsetDateTime,
    /// Not before timestamp
    pub nbf: OffsetDateTime,
    /// User role
    pub role: UserRole,
    /// User permissions
    pub permissions: Vec<String>,
    /// Session ID for token revocation
    pub session_id: String,
    /// Token type (access, refresh)
    pub token_type: TokenType,
    /// Device fingerprint for additional security
    pub device_fingerprint: Option<String>,
    /// IP address for geo-based validation
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
    Reset,      // For password reset
    Verify,     // For email verification
}

/// Token pair for authentication
#[derive(Debug, Clone)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
    pub token_type: String,
}

/// Enhanced PASETO service for token management
pub struct PasetoService {
    symmetric_key: SymmetricKey<V4>,
    access_token_duration: Duration,
    refresh_token_duration: Duration,
    reset_token_duration: Duration,
    verify_token_duration: Duration,
}

impl PasetoService {
    /// Create a new PASETO service with the given key
    pub fn new(key_bytes: &[u8]) -> AppResult<Self> {
        if key_bytes.len() < 32 {
            return Err(AppError::Validation(
                "PASETO key must be at least 32 bytes".to_string()
            ));
        }

        let symmetric_key = SymmetricKey::<V4>::from(key_bytes)
            .map_err(|e| {
                error!("Failed to create PASETO key: {}", e);
                AppError::InternalServer(format!("Failed to create PASETO key: {}", e))
            })?;

        Ok(Self {
            symmetric_key,
            access_token_duration: Duration::minutes(15),   // 15 minutes for access tokens
            refresh_token_duration: Duration::days(7),      // 7 days for refresh tokens
            reset_token_duration: Duration::hours(1),       // 1 hour for password reset
            verify_token_duration: Duration::hours(24),     // 24 hours for email verification
        })
    }

    /// Generate token pair (access + refresh) with enhanced security
    pub fn generate_token_pair(
        &self,
        user_id: UserId,
        role: UserRole,
        permissions: Vec<String>,
        session_id: String,
        device_fingerprint: Option<String>,
        ip_address: Option<String>,
    ) -> AppResult<TokenPair> {
        let now = OffsetDateTime::now_utc();

        // Generate access token
        let access_claims = TokenClaims {
            sub: user_id.0.to_string(),
            iat: now,
            exp: now + self.access_token_duration,
            nbf: now,
            role: role.clone(),
            permissions: permissions.clone(),
            session_id: session_id.clone(),
            token_type: TokenType::Access,
            device_fingerprint: device_fingerprint.clone(),
            ip_address: ip_address.clone(),
        };

        // Generate refresh token with longer expiry
        let refresh_claims = TokenClaims {
            sub: user_id.0.to_string(),
            iat: now,
            exp: now + self.refresh_token_duration,
            nbf: now,
            role,
            permissions,
            session_id,
            token_type: TokenType::Refresh,
            device_fingerprint,
            ip_address,
        };

        let access_token = self.create_token(&access_claims)?;
        let refresh_token = self.create_token(&refresh_claims)?;

        info!("Generated token pair for user: {}", user_id.0);

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: self.access_token_duration.whole_seconds(),
            refresh_expires_in: self.refresh_token_duration.whole_seconds(),
            token_type: "Bearer".to_string(),
        })
    }

    /// Generate special purpose tokens (reset, verify)
    pub fn generate_special_token(
        &self,
        user_id: UserId,
        token_type: TokenType,
        additional_claims: Option<BTreeMap<String, serde_json::Value>>,
    ) -> AppResult<String> {
        let now = OffsetDateTime::now_utc();

        let duration = match token_type {
            TokenType::Reset => self.reset_token_duration,
            TokenType::Verify => self.verify_token_duration,
            _ => return Err(AppError::Validation("Invalid special token type".to_string())),
        };

        let mut claims = TokenClaims {
            sub: user_id.0.to_string(),
            iat: now,
            exp: now + duration,
            nbf: now,
            role: UserRole::Guest, // Special tokens have minimal permissions
            permissions: vec![],
            session_id: uuid::Uuid::new_v4().to_string(),
            token_type,
            device_fingerprint: None,
            ip_address: None,
        };

        self.create_token(&claims)
    }

    /// Create a PASETO token from claims with enhanced security
    fn create_token(&self, claims: &TokenClaims) -> AppResult<String> {
        let mut token_claims = Claims::new()
            .map_err(|e| {
                error!("Failed to create claims: {}", e);
                AppError::InternalServer(format!("Failed to create claims: {}", e))
            })?;

        // Set standard claims
        token_claims.subject(&claims.sub)
            .map_err(|e| AppError::InternalServer(format!("Failed to set subject: {}", e)))?;
        token_claims.issued_at(&claims.iat)
            .map_err(|e| AppError::InternalServer(format!("Failed to set issued_at: {}", e)))?;
        token_claims.expiration(&claims.exp)
            .map_err(|e| AppError::InternalServer(format!("Failed to set expiration: {}", e)))?;
        token_claims.not_before(&claims.nbf)
            .map_err(|e| AppError::InternalServer(format!("Failed to set not_before: {}", e)))?;

        // Add custom claims with error handling
        token_claims.add_additional("role", serde_json::to_value(&claims.role)
            .map_err(|e| AppError::InternalServer(format!("Failed to serialize role: {}", e)))?);

        token_claims.add_additional("permissions", serde_json::to_value(&claims.permissions)
            .map_err(|e| AppError::InternalServer(format!("Failed to serialize permissions: {}", e)))?);

        token_claims.add_additional("session_id", serde_json::to_value(&claims.session_id)
            .map_err(|e| AppError::InternalServer(format!("Failed to serialize session_id: {}", e)))?);

        token_claims.add_additional("token_type", serde_json::to_value(&claims.token_type)
            .map_err(|e| AppError::InternalServer(format!("Failed to serialize token_type: {}", e)))?);

        // Add security claims if present
        if let Some(ref fingerprint) = claims.device_fingerprint {
            token_claims.add_additional("device_fingerprint", serde_json::to_value(fingerprint)
                .map_err(|e| AppError::InternalServer(format!("Failed to serialize device_fingerprint: {}", e)))?);
        }

        if let Some(ref ip) = claims.ip_address {
            token_claims.add_additional("ip_address", serde_json::to_value(ip)
                .map_err(|e| AppError::InternalServer(format!("Failed to serialize ip_address: {}", e)))?);
        }

        // Create encrypted token with footer for additional security
        let footer = Some(b"terra-siaga-v1");
        local::encrypt(&self.symmetric_key, &token_claims, footer, None)
            .map_err(|e| {
                error!("Failed to create PASETO token: {}", e);
                AppError::InternalServer(format!("Failed to create PASETO token: {}", e))
            })
    }

    /// Verify and decode a PASETO token with enhanced validation
    pub fn verify_token(&self, token: &str) -> AppResult<TokenClaims> {
        // Parse the untrusted token
        let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)
            .map_err(|e| {
                warn!("Invalid token format: {}", e);
                AppError::Unauthorized(format!("Invalid token format: {}", e))
            })?;

        // Set comprehensive validation rules
        let mut validation_rules = ClaimsValidationRules::new();
        let now = OffsetDateTime::now_utc();
        validation_rules.validate_expiration_claim(now);
        validation_rules.validate_not_before_claim(now);

        // Decrypt and validate token with footer verification
        let footer = Some(b"terra-siaga-v1");
        let trusted_token = local::decrypt(&self.symmetric_key, &untrusted_token, &validation_rules, footer, None)
            .map_err(|e| {
                warn!("Token verification failed: {}", e);
                AppError::Unauthorized(format!("Token verification failed: {}", e))
            })?;

        let claims = trusted_token.payload_claims()
            .ok_or_else(|| AppError::Unauthorized("No payload claims found".to_string()))?;

        // Extract and validate all claims
        let subject = claims.get_claim("sub")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Unauthorized("Missing subject claim".to_string()))?;

        let iat = claims.get_claim("iat")
            .and_then(|v| v.as_str())
            .and_then(|s| OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok())
            .ok_or_else(|| AppError::Unauthorized("Invalid issued at claim".to_string()))?;

        let exp = claims.get_claim("exp")
            .and_then(|v| v.as_str())
            .and_then(|s| OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok())
            .ok_or_else(|| AppError::Unauthorized("Invalid expiration claim".to_string()))?;

        let nbf = claims.get_claim("nbf")
            .and_then(|v| v.as_str())
            .and_then(|s| OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok())
            .ok_or_else(|| AppError::Unauthorized("Invalid not before claim".to_string()))?;

        // Extract custom claims with proper error handling
        let role: UserRole = claims.get_claim("role")
            .ok_or_else(|| AppError::Unauthorized("Missing role claim".to_string()))
            .and_then(|v| serde_json::from_value(v.clone())
                .map_err(|e| AppError::Unauthorized(format!("Invalid role claim: {}", e))))?;

        let permissions: Vec<String> = claims.get_claim("permissions")
            .ok_or_else(|| AppError::Unauthorized("Missing permissions claim".to_string()))
            .and_then(|v| serde_json::from_value(v.clone())
                .map_err(|e| AppError::Unauthorized(format!("Invalid permissions claim: {}", e))))?;

        let session_id: String = claims.get_claim("session_id")
            .ok_or_else(|| AppError::Unauthorized("Missing session_id claim".to_string()))
            .and_then(|v| serde_json::from_value(v.clone())
                .map_err(|e| AppError::Unauthorized(format!("Invalid session_id claim: {}", e))))?;

        let token_type: TokenType = claims.get_claim("token_type")
            .ok_or_else(|| AppError::Unauthorized("Missing token_type claim".to_string()))
            .and_then(|v| serde_json::from_value(v.clone())
                .map_err(|e| AppError::Unauthorized(format!("Invalid token_type claim: {}", e))))?;

        // Extract optional security claims
        let device_fingerprint: Option<String> = claims.get_claim("device_fingerprint")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let ip_address: Option<String> = claims.get_claim("ip_address")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        info!("Successfully verified token for user: {}", subject);

        Ok(TokenClaims {
            sub: subject.to_string(),
            iat,
            exp,
            nbf,
            role,
            permissions,
            session_id,
            token_type,
            device_fingerprint,
            ip_address,
        })
    }

    /// Validate token with specific requirements (used by middleware)
    pub fn validate_token_requirements(
        &self,
        token: &str,
        required_permissions: &[String],
        required_role: Option<UserRole>,
        expected_device: Option<&str>,
        expected_ip: Option<&str>,
    ) -> AppResult<TokenClaims> {
        let claims = self.validate_token_with_context(token, expected_ip, expected_device)?;

        // Check role requirement
        if let Some(ref req_role) = required_role {
            if !claims.role.has_permission_level(req_role) {
                return Err(AppError::Forbidden(format!(
                    "Insufficient role: required {:?}, have {:?}",
                    req_role, claims.role
                )));
            }
        }

        // Check permission requirements
        if !required_permissions.is_empty() {
            let missing_permissions: Vec<&String> = required_permissions
                .iter()
                .filter(|perm| !claims.permissions.contains(perm))
                .collect();

            if !missing_permissions.is_empty() {
                return Err(AppError::Forbidden(format!(
                    "Missing required permissions: {:?}",
                    missing_permissions
                )));
            }
        }

        Ok(claims)
    }

    /// Refresh an access token using a valid refresh token
    pub fn refresh_access_token(
        &self,
        refresh_token: &str,
        device_fingerprint: Option<String>,
        ip_address: Option<String>,
    ) -> AppResult<TokenPair> {
        // Verify the refresh token
        let refresh_claims = self.verify_token(refresh_token)?;

        // Ensure it's actually a refresh token
        if refresh_claims.token_type != TokenType::Refresh {
            return Err(AppError::Unauthorized("Invalid token type for refresh".to_string()));
        }

        // Validate device fingerprint if provided
        if let (Some(ref stored_fingerprint), Some(ref provided_fingerprint)) =
            (&refresh_claims.device_fingerprint, &device_fingerprint) {
            if stored_fingerprint != provided_fingerprint {
                warn!("Device fingerprint mismatch for user: {}", refresh_claims.sub);
                return Err(AppError::Unauthorized("Device fingerprint mismatch".to_string()));
            }
        }

        // Generate new token pair with updated timestamps
        let user_id = UserId(uuid::Uuid::parse_str(&refresh_claims.sub)
            .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))?);

        self.generate_token_pair(
            user_id,
            refresh_claims.role,
            refresh_claims.permissions,
            refresh_claims.session_id,
            device_fingerprint,
            ip_address,
        )
    }

    /// Validate token with additional security checks
    pub fn validate_token_with_context(
        &self,
        token: &str,
        expected_ip: Option<&str>,
        expected_fingerprint: Option<&str>,
    ) -> AppResult<TokenClaims> {
        let claims = self.verify_token(token)?;

        // Validate IP address if provided
        if let (Some(stored_ip), Some(expected_ip)) = (&claims.ip_address, expected_ip) {
            if stored_ip != expected_ip {
                warn!("IP address mismatch for user: {} (stored: {}, provided: {})",
                      claims.sub, stored_ip, expected_ip);
                return Err(AppError::Unauthorized("IP address validation failed".to_string()));
            }
        }

        // Validate device fingerprint if provided
        if let (Some(stored_fingerprint), Some(expected_fingerprint)) =
            (&claims.device_fingerprint, expected_fingerprint) {
            if stored_fingerprint != expected_fingerprint {
                warn!("Device fingerprint mismatch for user: {}", claims.sub);
                return Err(AppError::Unauthorized("Device fingerprint validation failed".to_string()));
            }
        }

        Ok(claims)
    }

    /// Extract user ID from token without full verification (for logout, etc.)
    pub fn extract_user_id(&self, token: &str) -> AppResult<UserId> {
        let claims = self.verify_token(token)?;
        let user_id = UserId(uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))?);
        Ok(user_id)
    }

    /// Check if token is expired without full verification
    pub fn is_token_expired(&self, token: &str) -> bool {
        match UntrustedToken::<Local, V4>::try_from(token) {
            Ok(untrusted_token) => {
                match local::decrypt(&self.symmetric_key, &untrusted_token, &ClaimsValidationRules::new(), Some(b"terra-siaga-v1"), None) {
                    Ok(trusted_token) => {
                        if let Some(claims) = trusted_token.payload_claims() {
                            if let Some(exp_value) = claims.get_claim("exp") {
                                if let Some(exp_str) = exp_value.as_str() {
                                    if let Ok(exp) = OffsetDateTime::parse(exp_str, &time::format_description::well_known::Rfc3339) {
                                        return OffsetDateTime::now_utc() > exp;
                                    }
                                }
                            }
                        }
                        true // If we can't parse expiration, consider it expired
                    }
                    Err(_) => true // If decryption fails, consider it expired
                }
            }
            Err(_) => true // If parsing fails, consider it expired
        }
    }

    /// Get token expiration time
    pub fn get_token_expiration(&self, token: &str) -> AppResult<OffsetDateTime> {
        let claims = self.verify_token(token)?;
        Ok(claims.exp)
    }

    /// Create a blacklist token hash for revocation
    pub fn create_token_hash(token: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl Drop for PasetoService {
    fn drop(&mut self) {
        // Zero out the key material when dropping
        self.symmetric_key.zeroize();
    }
}

/// Enhanced token blacklist for revocation
#[derive(Debug, Clone)]
pub struct TokenBlacklist {
    blacklisted_tokens: std::collections::HashSet<String>,
    blacklisted_sessions: std::collections::HashSet<String>,
}

impl TokenBlacklist {
    pub fn new() -> Self {
        Self {
            blacklisted_tokens: std::collections::HashSet::new(),
            blacklisted_sessions: std::collections::HashSet::new(),
        }
    }

    /// Add token to blacklist
    pub fn blacklist_token(&mut self, token: &str) {
        let token_hash = PasetoService::create_token_hash(token);
        self.blacklisted_tokens.insert(token_hash);
    }

    /// Add session to blacklist (revokes all tokens for session)
    pub fn blacklist_session(&mut self, session_id: &str) {
        self.blacklisted_sessions.insert(session_id.to_string());
    }

    /// Check if token is blacklisted
    pub fn is_token_blacklisted(&self, token: &str) -> bool {
        let token_hash = PasetoService::create_token_hash(token);
        self.blacklisted_tokens.contains(&token_hash)
    }

    /// Check if session is blacklisted
    pub fn is_session_blacklisted(&self, session_id: &str) -> bool {
        self.blacklisted_sessions.contains(session_id)
    }

    /// Remove expired tokens from blacklist (cleanup)
    pub fn cleanup_expired(&mut self, current_time: OffsetDateTime) {
        // In a real implementation, you'd track expiration times
        // For now, this is a placeholder for the cleanup logic
        info!("Cleaning up expired blacklisted tokens at: {}", current_time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_paseto_token_generation_and_verification() {
        let key = b"this-is-a-super-secret-key-for-testing-purposes-32-bytes";
        let paseto_service = PasetoService::new(key).unwrap();

        let user_id = UserId(uuid::Uuid::new_v4());
        let role = UserRole::Admin;
        let permissions = vec!["read".to_string(), "write".to_string()];
        let session_id = uuid::Uuid::new_v4().to_string();

        // Generate token pair
        let token_pair = paseto_service.generate_token_pair(
            user_id,
            role.clone(),
            permissions.clone(),
            session_id.clone(),
            Some("device123".to_string()),
            Some("192.168.1.1".to_string()),
        ).unwrap();

        // Verify access token
        let claims = paseto_service.verify_token(&token_pair.access_token).unwrap();
        assert_eq!(claims.sub, user_id.0.to_string());
        assert_eq!(claims.role, role);
        assert_eq!(claims.permissions, permissions);
        assert_eq!(claims.session_id, session_id);
        assert_eq!(claims.token_type, TokenType::Access);

        // Verify refresh token
        let refresh_claims = paseto_service.verify_token(&token_pair.refresh_token).unwrap();
        assert_eq!(refresh_claims.token_type, TokenType::Refresh);
    }

    #[tokio::test]
    async fn test_token_refresh() {
        let key = b"this-is-a-super-secret-key-for-testing-purposes-32-bytes";
        let paseto_service = PasetoService::new(key).unwrap();

        let user_id = UserId(uuid::Uuid::new_v4());
        let role = UserRole::User;
        let permissions = vec!["read".to_string()];
        let session_id = uuid::Uuid::new_v4().to_string();

        // Generate initial token pair
        let initial_tokens = paseto_service.generate_token_pair(
            user_id,
            role,
            permissions,
            session_id,
            Some("device123".to_string()),
            Some("192.168.1.1".to_string()),
        ).unwrap();

        // Refresh the token
        let refreshed_tokens = paseto_service.refresh_access_token(
            &initial_tokens.refresh_token,
            Some("device123".to_string()),
            Some("192.168.1.1".to_string()),
        ).unwrap();

        // Verify new access token works
        let claims = paseto_service.verify_token(&refreshed_tokens.access_token).unwrap();
        assert_eq!(claims.sub, user_id.0.to_string());
    }

    #[test]
    fn test_key_validation() {
        // Test weak key rejection
        let weak_key = [0u8; 32];
        assert!(utils::validate_key_strength(&weak_key).is_err());

        // Test short key rejection
        let short_key = [1u8; 16];
        assert!(utils::validate_key_strength(&short_key).is_err());

        // Test strong key acceptance
        let strong_key = utils::generate_secure_key();
        assert!(utils::validate_key_strength(&strong_key).is_ok());
    }
}
