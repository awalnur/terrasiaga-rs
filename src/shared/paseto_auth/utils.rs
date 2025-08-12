/// Utility functions for PASETO authentication
/// Provides secure key generation, validation, and session management

use rand::Rng;
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use sha2::{Sha256, Digest};
use std::str::FromStr;
use crate::shared::error::{AppError, AppResult};

/// Generate a cryptographically secure 32-byte key for PASETO
pub fn generate_secure_key() -> [u8; 32] {
    let mut rng = ChaCha20Rng::from_entropy();
    let mut key = [0u8; 32];
    rng.fill(&mut key);
    key
}

/// Validate key strength to ensure it's suitable for cryptographic use
pub fn validate_key_strength(key: &[u8]) -> AppResult<()> {
    if key.len() < 32 {
        return Err(AppError::Validation(
            "Key must be at least 32 bytes long".to_string()
        ));
    }

    // Check for weak keys (all zeros, all ones, repeating patterns)
    if key.iter().all(|&b| b == 0) {
        return Err(AppError::Validation(
            "Key cannot be all zeros".to_string()
        ));
    }

    if key.iter().all(|&b| b == 255) {
        return Err(AppError::Validation(
            "Key cannot be all ones".to_string()
        ));
    }

    // Check for repeating byte patterns
    if key.len() >= 4 {
        let pattern = &key[0..4];
        let mut is_repeating = true;
        for chunk in key.chunks(4) {
            if chunk != pattern {
                is_repeating = false;
                break;
            }
        }
        if is_repeating {
            return Err(AppError::Validation(
                "Key cannot have repeating patterns".to_string()
            ));
        }
    }

    // Calculate entropy (simplified check)
    let unique_bytes = key.iter().collect::<std::collections::HashSet<_>>().len();
    if unique_bytes < 16 {
        return Err(AppError::Validation(
            "Key has insufficient entropy".to_string()
        ));
    }

    Ok(())
}

/// Generate a secure session ID
pub fn generate_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Generate a device fingerprint from user agent and other client info
pub fn generate_device_fingerprint(
    user_agent: Option<&str>,
    accept_language: Option<&str>,
    screen_resolution: Option<&str>,
    timezone: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();

    hasher.update(user_agent.unwrap_or("unknown"));
    hasher.update(accept_language.unwrap_or("unknown"));
    hasher.update(screen_resolution.unwrap_or("unknown"));
    hasher.update(timezone.unwrap_or("unknown"));

    format!("{:x}", hasher.finalize())
}

/// Derive a key from a password using PBKDF2
pub fn derive_key_from_password(password: &str, salt: &[u8]) -> AppResult<[u8; 32]> {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;

    if password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters".to_string()
        ));
    }

    if salt.len() < 16 {
        return Err(AppError::Validation(
            "Salt must be at least 16 bytes".to_string()
        ));
    }

    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 100_000, &mut key);

    Ok(key)
}

/// Generate a random salt for key derivation
pub fn generate_salt() -> [u8; 16] {
    let mut rng = ChaCha20Rng::from_entropy();
    let mut salt = [0u8; 16];
    rng.fill(&mut salt);
    salt
}

/// Secure string comparison to prevent timing attacks
pub fn secure_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
        result |= byte_a ^ byte_b;
    }

    result == 0
}

/// Extract IP address from various headers
pub fn extract_real_ip(
    x_forwarded_for: Option<&str>,
    x_real_ip: Option<&str>,
    cf_connecting_ip: Option<&str>,
    remote_addr: Option<&str>,
) -> Option<String> {
    // Priority order: CF-Connecting-IP, X-Real-IP, X-Forwarded-For, Remote-Addr
    if let Some(cf_ip) = cf_connecting_ip {
        if !cf_ip.is_empty() {
            return Some(cf_ip.to_string());
        }
    }

    if let Some(real_ip) = x_real_ip {
        if !real_ip.is_empty() {
            return Some(real_ip.to_string());
        }
    }

    if let Some(forwarded) = x_forwarded_for {
        if let Some(first_ip) = forwarded.split(',').next() {
            let clean_ip = first_ip.trim();
            if !clean_ip.is_empty() {
                return Some(clean_ip.to_string());
            }
        }
    }

    remote_addr.map(|addr| addr.to_string())
}

/// Validate IP address format
pub fn is_valid_ip(ip: &str) -> bool {
    std::net::IpAddr::from_str(ip).is_ok()
}

/// Hash a token for blacklisting (without revealing the original token)
pub fn hash_token_for_blacklist(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate a secure random string of specified length
pub fn generate_random_string(length: usize) -> String {
    use rand::distributions::{Alphanumeric, DistString};
    let mut rng = ChaCha20Rng::from_entropy();
    Alphanumeric.sample_string(&mut rng, length)
}

/// Calculate token expiry in seconds from now
pub fn calculate_expiry_seconds(duration: time::Duration) -> i64 {
    duration.whole_seconds()
}

/// Check if a timestamp is within acceptable clock skew
pub fn is_within_clock_skew(timestamp: time::OffsetDateTime, skew_seconds: i64) -> bool {
    let now = time::OffsetDateTime::now_utc();
    let diff = (now - timestamp).whole_seconds().abs();
    diff <= skew_seconds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secure_key() {
        let key1 = generate_secure_key();
        let key2 = generate_secure_key();

        // Keys should be different
        assert_ne!(key1, key2);

        // Keys should pass validation
        assert!(validate_key_strength(&key1).is_ok());
        assert!(validate_key_strength(&key2).is_ok());
    }

    #[test]
    fn test_key_validation() {
        // Test valid key
        let valid_key = generate_secure_key();
        assert!(validate_key_strength(&valid_key).is_ok());

        // Test short key
        let short_key = [1u8; 16];
        assert!(validate_key_strength(&short_key).is_err());

        // Test all-zero key
        let zero_key = [0u8; 32];
        assert!(validate_key_strength(&zero_key).is_err());

        // Test all-ones key
        let ones_key = [255u8; 32];
        assert!(validate_key_strength(&ones_key).is_err());
    }

    #[test]
    fn test_device_fingerprint() {
        let fp1 = generate_device_fingerprint(
            Some("Mozilla/5.0"),
            Some("en-US"),
            Some("1920x1080"),
            Some("America/New_York"),
        );

        let fp2 = generate_device_fingerprint(
            Some("Mozilla/5.0"),
            Some("en-US"),
            Some("1920x1080"),
            Some("America/New_York"),
        );

        let fp3 = generate_device_fingerprint(
            Some("Chrome/91.0"),
            Some("en-US"),
            Some("1920x1080"),
            Some("America/New_York"),
        );

        // Same inputs should produce same fingerprint
        assert_eq!(fp1, fp2);

        // Different inputs should produce different fingerprint
        assert_ne!(fp1, fp3);
    }

    #[test]
    fn test_secure_compare() {
        assert!(secure_compare("hello", "hello"));
        assert!(!secure_compare("hello", "world"));
        assert!(!secure_compare("hello", "hello2"));
        assert!(!secure_compare("", "hello"));
    }

    #[test]
    fn test_ip_extraction() {
        // Test CF-Connecting-IP priority
        let ip = extract_real_ip(
            Some("192.168.1.1, 10.0.0.1"),
            Some("172.16.0.1"),
            Some("203.0.113.1"),
            Some("198.51.100.1"),
        );
        assert_eq!(ip, Some("203.0.113.1".to_string()));

        // Test X-Forwarded-For parsing
        let ip = extract_real_ip(
            Some("192.168.1.1, 10.0.0.1"),
            None,
            None,
            Some("198.51.100.1"),
        );
        assert_eq!(ip, Some("192.168.1.1".to_string()));

        // Test fallback to remote addr
        let ip = extract_real_ip(None, None, None, Some("198.51.100.1"));
        assert_eq!(ip, Some("198.51.100.1".to_string()));
    }

    #[test]
    fn test_ip_validation() {
        assert!(is_valid_ip("192.168.1.1"));
        assert!(is_valid_ip("::1"));
        assert!(is_valid_ip("2001:db8::1"));
        assert!(!is_valid_ip("not.an.ip"));
        assert!(!is_valid_ip("999.999.999.999"));
    }

    #[test]
    fn test_random_string_generation() {
        let str1 = generate_random_string(32);
        let str2 = generate_random_string(32);

        assert_eq!(str1.len(), 32);
        assert_eq!(str2.len(), 32);
        assert_ne!(str1, str2);

        // Should only contain alphanumeric characters
        assert!(str1.chars().all(|c| c.is_alphanumeric()));
    }
}
