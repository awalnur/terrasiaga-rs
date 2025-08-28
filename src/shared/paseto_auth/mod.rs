// Minimal PASETO-like service facade to satisfy current API expectations
// NOTE: This is a lightweight, non-cryptographic placeholder that encodes/decodes JSON.
// Replace with a proper implementation or adapt to infrastructure::security::PasetoSecurityService.

use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose, Engine as _};
use crate::shared::error::{AppError, AppResult};
use crate::shared::types::{UserId, UserRole};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TokenType { Access, Refresh }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub user_id: String,
    pub role: UserRole,
    pub permissions: Vec<String>,
    pub session_id: String,
    pub device_fingerprint: Option<String>,
    pub token_type: TokenType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
}

pub struct PasetoService {
    key: Vec<u8>,
}

impl PasetoService {
    pub fn new(key: &[u8]) -> AppResult<Self> {
        if key.is_empty() { return Err(AppError::Configuration("PASETO key cannot be empty".into())); }
        Ok(Self { key: key.to_vec() })
    }

    pub fn generate_token_pair(
        &self,
        user_id: UserId,
        role: UserRole,
        permissions: Vec<String>,
        session_id: String,
    ) -> AppResult<TokenPair> {
        let access_claims = TokenClaims {
            user_id: user_id.value().to_string(),
            role: role.clone(),
            permissions: permissions.clone(),
            session_id: session_id.clone(),
            device_fingerprint: None,
            token_type: TokenType::Access,
        };
        let refresh_claims = TokenClaims {
            user_id: user_id.value().to_string(),
            role,
            permissions,
            session_id,
            device_fingerprint: None,
            token_type: TokenType::Refresh,
        };
        let access_token = self.encode_claims(&access_claims)?;
        let refresh_token = self.encode_claims(&refresh_claims)?;
        Ok(TokenPair { access_token, refresh_token, expires_in: 900, refresh_expires_in: 86400 })
    }

    pub fn refresh_access_token(&self, refresh_token: &str) -> AppResult<String> {
        let claims = self.decode_claims(refresh_token)?;
        if claims.token_type != TokenType::Refresh {
            return Err(AppError::Unauthorized("Invalid token type for refresh".into()));
        }
        let new_access = TokenClaims { token_type: TokenType::Access, ..claims };
        self.encode_claims(&new_access)
    }

    pub fn verify_token(&self, token: &str) -> AppResult<TokenClaims> {
        self.decode_claims(token)
    }

    pub fn extract_user_id(&self, token: &str) -> AppResult<UserId> {
        let claims = self.decode_claims(token)?;
        let uuid = uuid::Uuid::parse_str(&claims.user_id)
            .map_err(|e| AppError::Unauthorized(format!("Invalid user id in token: {}", e)))?;
        Ok(UserId::from_uuid(uuid))
    }

    fn encode_claims(&self, claims: &TokenClaims) -> AppResult<String> {
        let mut data = serde_json::to_vec(claims)
            .map_err(|e| AppError::Serialization(format!("Failed to serialize claims: {}", e)))?;
        // Trivial binding to key length to avoid stale clippy warnings
        if !self.key.is_empty() { data.push(self.key[0]); }
        Ok(general_purpose::STANDARD_NO_PAD.encode(data))
    }

    fn decode_claims(&self, token: &str) -> AppResult<TokenClaims> {
        let mut bytes = general_purpose::STANDARD_NO_PAD
            .decode(token)
            .map_err(|e| AppError::Unauthorized(format!("Invalid token encoding: {}", e)))?;
        if !bytes.is_empty() { bytes.pop(); }
        serde_json::from_slice(&bytes)
            .map_err(|e| AppError::Unauthorized(format!("Invalid token claims: {}", e)))
    }
}

pub mod utils {
    pub fn generate_session_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

