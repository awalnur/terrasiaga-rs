/// Comprehensive Password Security Module
/// Provides secure password hashing, verification, and security utilities using Argon2
/// with additional security features like rate limiting, password strength validation,
/// and secure random generation.

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{rand_core::OsRng, SaltString, Error as Argon2Error}
};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Security configuration for password hashing
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Memory cost in KB (default: 65536 = 64MB)
    pub memory_cost: u32,
    /// Time cost (number of iterations, default: 3)
    pub time_cost: u32,
    /// Parallelism (number of threads, default: 4)
    pub parallelism: u32,
    /// Output length in bytes (default: 32)
    pub output_length: Option<usize>,
    /// Maximum password length allowed
    pub max_password_length: usize,
    /// Minimum password length required
    pub min_password_length: usize,
    /// Rate limiting: max attempts per time window
    pub max_attempts: u32,
    /// Rate limiting: time window in seconds
    pub time_window_seconds: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            memory_cost: 65536,    // 64MB
            time_cost: 3,          // 3 iterations
            parallelism: 4,        // 4 threads
            output_length: Some(32), // 32 bytes output
            max_password_length: 128,
            min_password_length: 8,
            max_attempts: 5,
            time_window_seconds: 900, // 15 minutes
        }
    }
}

/// Password service trait for dependency injection
pub trait PasswordService: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String, SecurityError>;
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, SecurityError>;
    fn validate_password_strength(&self, password: &str) -> Result<PasswordStrength, SecurityError>;
    fn generate_secure_password(&self, length: usize) -> String;
}

/// Security service trait for general security operations
pub trait SecurityService: Send + Sync {
    fn generate_secure_token(&self, length: usize) -> String;
    fn generate_uuid(&self) -> Uuid;
    fn constant_time_compare(&self, a: &[u8], b: &[u8]) -> bool;
    fn hash_data(&self, data: &[u8]) -> Vec<u8>;
}

/// Security errors
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Password hashing failed: {0}")]
    HashingFailed(String),
    #[error("Password verification failed: {0}")]
    VerificationFailed(String),
    #[error("Password too weak: {0}")]
    WeakPassword(String),
    #[error("Password too long (max {max} characters)")]
    PasswordTooLong { max: usize },
    #[error("Password too short (min {min} characters)")]
    PasswordTooShort { min: usize },
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Result type for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Password strength assessment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PasswordStrengthLevel {
    VeryWeak,
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

impl PasswordStrengthLevel {
    pub fn score(&self) -> u8 {
        match self {
            PasswordStrengthLevel::VeryWeak => 1,
            PasswordStrengthLevel::Weak => 2,
            PasswordStrengthLevel::Medium => 3,
            PasswordStrengthLevel::Strong => 4,
            PasswordStrengthLevel::VeryStrong => 5,
        }
    }
}

/// Password strength requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordStrength {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_special_chars: bool,
    pub max_consecutive_chars: usize,
    pub forbidden_patterns: Vec<String>,
}

impl Default for PasswordStrength {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_special_chars: true,
            max_consecutive_chars: 3,
            forbidden_patterns: vec![
                "password".to_string(),
                "123456".to_string(),
                "qwerty".to_string(),
                "admin".to_string(),
            ],
        }
    }
}

/// Password validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordValidation {
    pub is_valid: bool,
    pub score: u8, // 0-100
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Rate limiting attempt tracking
#[derive(Debug, Clone)]
struct AttemptRecord {
    count: u32,
    first_attempt: Instant,
    last_attempt: Instant,
}

/// Main password security service
#[derive(Clone,Debug)]
pub struct PasswordSecurity {
    config: SecurityConfig,
    strength_config: PasswordStrength,
    argon2: Argon2<'static>,
    rate_limiter: Arc<Mutex<HashMap<String, AttemptRecord>>>,
}

impl PasswordSecurity {
    /// Create new password security instance with default configuration
    pub fn new() -> Self {
        let config = SecurityConfig::default();
        Self::with_config(config, PasswordStrength::default())
    }

    /// Create new password security instance with custom configuration
    pub fn with_config(config: SecurityConfig, strength_config: PasswordStrength) -> Self {
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                config.memory_cost,
                config.time_cost,
                config.parallelism,
                config.output_length,
            ).expect("Invalid Argon2 parameters"),
        );

        Self {
            config,
            strength_config,
            argon2,
            rate_limiter: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Hash a password securely using Argon2
    pub fn hash_password(&self, password: &str) -> SecurityResult<String> {
        // Validate password length
        if password.len() > self.config.max_password_length {
            return Err(SecurityError::PasswordTooLong {
                max: self.config.max_password_length,
            });
        }

        if password.len() < self.config.min_password_length {
            return Err(SecurityError::PasswordTooShort {
                min: self.config.min_password_length,
            });
        }

        // Validate password strength
        let validation = self.validate_password_strength(password);
        if !validation.is_valid {
            return Err(SecurityError::WeakPassword {
                score: validation.score,
            });
        }

        // Generate salt
        let salt = SaltString::generate(&mut OsRng);

        // Hash password
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| SecurityError::HashingFailed {
                message: format!("Argon2 hashing failed: {}", e),
            })?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against its hash
    pub fn verify_password(&self, password: &str, hash: &str) -> SecurityResult<bool> {
        self.verify_password_with_rate_limit(password, hash, None)
    }

    /// Verify a password with rate limiting
    pub fn verify_password_with_rate_limit(
        &self,
        password: &str,
        hash: &str,
        identifier: Option<&str>
    ) -> SecurityResult<bool> {
        // Check rate limiting if identifier provided
        if let Some(id) = identifier {
            self.check_rate_limit(id)?;
        }

        // Parse stored hash
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| SecurityError::VerificationFailed {
                message: format!("Invalid hash format: {}", e),
            })?;

        // Verify password
        let is_valid = self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();

        // Record attempt if identifier provided
        if let Some(id) = identifier {
            self.record_attempt(id, is_valid);
        }

        Ok(is_valid)
    }

    /// Validate password strength
    pub fn validate_password_strength(&self, password: &str) -> PasswordValidation {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut score = 0u8;

        // Length check
        if password.len() < self.strength_config.min_length {
            issues.push(format!("Password must be at least {} characters long", self.strength_config.min_length));
            suggestions.push("Use a longer password".to_string());
        } else {
            score += 20;
        }

        // Character type checks
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_numbers = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        if self.strength_config.require_uppercase && !has_uppercase {
            issues.push("Password must contain at least one uppercase letter".to_string());
            suggestions.push("Add uppercase letters (A-Z)".to_string());
        } else if has_uppercase {
            score += 15;
        }

        if self.strength_config.require_lowercase && !has_lowercase {
            issues.push("Password must contain at least one lowercase letter".to_string());
            suggestions.push("Add lowercase letters (a-z)".to_string());
        } else if has_lowercase {
            score += 15;
        }

        if self.strength_config.require_numbers && !has_numbers {
            issues.push("Password must contain at least one number".to_string());
            suggestions.push("Add numbers (0-9)".to_string());
        } else if has_numbers {
            score += 15;
        }

        if self.strength_config.require_special_chars && !has_special {
            issues.push("Password must contain at least one special character".to_string());
            suggestions.push("Add special characters (!@#$%^&*)".to_string());
        } else if has_special {
            score += 15;
        }

        // Check for consecutive characters
        if self.has_consecutive_chars(password, self.strength_config.max_consecutive_chars) {
            issues.push(format!("Password contains more than {} consecutive identical characters", self.strength_config.max_consecutive_chars));
            suggestions.push("Avoid repeating characters".to_string());
        } else {
            score += 10;
        }

        // Check forbidden patterns
        let password_lower = password.to_lowercase();
        for pattern in &self.strength_config.forbidden_patterns {
            if password_lower.contains(&pattern.to_lowercase()) {
                issues.push(format!("Password contains forbidden pattern: {}", pattern));
                suggestions.push("Avoid common words and patterns".to_string());
                score = score.saturating_sub(20);
                break;
            }
        }

        // Bonus for length
        if password.len() > 12 {
            score = std::cmp::min(100, score + 10);
        }

        PasswordValidation {
            is_valid: issues.is_empty(),
            score,
            issues,
            suggestions,
        }
    }

    /// Generate a secure random password
    pub fn generate_secure_password(&self, length: usize) -> String {
        use rand::Rng;

        let uppercase = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let lowercase = "abcdefghijklmnopqrstuvwxyz";
        let numbers = "0123456789";
        let special = "!@#$%^&*()_+-=[]{}|;:,.<>?";

        let all_chars = format!("{}{}{}{}", uppercase, lowercase, numbers, special);
        let mut rng = rand::thread_rng();

        // Ensure at least one character from each required category
        let mut password = String::new();

        if self.strength_config.require_uppercase {
            password.push(uppercase.chars().nth(rng.gen_range(0..uppercase.len())).unwrap());
        }
        if self.strength_config.require_lowercase {
            password.push(lowercase.chars().nth(rng.gen_range(0..lowercase.len())).unwrap());
        }
        if self.strength_config.require_numbers {
            password.push(numbers.chars().nth(rng.gen_range(0..numbers.len())).unwrap());
        }
        if self.strength_config.require_special_chars {
            password.push(special.chars().nth(rng.gen_range(0..special.len())).unwrap());
        }

        // Fill remaining length with random characters
        while password.len() < length {
            let char_index = rng.gen_range(0..all_chars.len());
            password.push(all_chars.chars().nth(char_index).unwrap());
        }

        // Shuffle the password
        let mut chars: Vec<char> = password.chars().collect();
        use rand::seq::SliceRandom;
        chars.shuffle(&mut rng);

        chars.into_iter().collect()
    }

    /// Check if password needs rehashing (for security upgrades)
    // pub fn needs_rehash(&self, hash: &str) -> bool {
    //     match PasswordHash::new(hash) {
    //         Ok(parsed) => {
    //             // Check if parameters match current configuration
    //             if let Some(params) = parsed.params {
    //                 params.m_cost() != Some(self.config.memory_cost) ||
    //                 params.t_cost() != Some(self.config.time_cost) ||
    //                 params.p_cost() != Some(self.config.parallelism)
    //             } else {
    //                 true // If we can't parse params, assume rehash needed
    //             }
    //         }
    //         Err(_) => true // Invalid hash format, needs rehash
    //     }
    // }

    /// Generate secure session token
    pub fn generate_session_token(&self) -> String {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        base64::encode(bytes)
    }

    /// Generate CSRF token
    pub fn generate_csrf_token(&self) -> String {
        Uuid::new_v4().to_string()
    }

    // Private helper methods

    fn has_consecutive_chars(&self, password: &str, max_consecutive: usize) -> bool {
        let chars: Vec<char> = password.chars().collect();
        let mut count = 1;

        for i in 1..chars.len() {
            if chars[i] == chars[i - 1] {
                count += 1;
                if count > max_consecutive {
                    return true;
                }
            } else {
                count = 1;
            }
        }

        false
    }

    fn check_rate_limit(&self, identifier: &str) -> SecurityResult<()> {
        let mut limiter = self.rate_limiter.lock().unwrap();
        let now = Instant::now();

        if let Some(record) = limiter.get(identifier) {
            // Check if time window has expired
            if now.duration_since(record.first_attempt).as_secs() > self.config.time_window_seconds {
                // Reset the record
                limiter.insert(identifier.to_string(), AttemptRecord {
                    count: 0,
                    first_attempt: now,
                    last_attempt: now,
                });
            } else if record.count >= self.config.max_attempts {
                return Err(SecurityError::RateLimitExceeded);
            }
        }

        Ok(())
    }

    fn record_attempt(&self, identifier: &str, success: bool) {
        let mut limiter = self.rate_limiter.lock().unwrap();
        let now = Instant::now();

        let record = limiter.entry(identifier.to_string()).or_insert(AttemptRecord {
            count: 0,
            first_attempt: now,
            last_attempt: now,
        });

        if !success {
            record.count += 1;
        } else {
            // Reset on successful login
            record.count = 0;
            record.first_attempt = now;
        }

        record.last_attempt = now;
    }
}

impl Default for PasswordSecurity {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience functions
/// Hash password with default configuration
pub fn hash_password(password: &str) -> SecurityResult<String> {
    let security = PasswordSecurity::new();
    security.hash_password(password)
}

/// Verify password with default configuration
pub fn verify_password(password: &str, hash: &str) -> SecurityResult<bool> {
    let security = PasswordSecurity::new();
    security.verify_password(password, hash)
}

/// Validate password strength with default configuration
pub fn validate_password_strength(password: &str) -> PasswordValidation {
    let security = PasswordSecurity::new();
    security.validate_password_strength(password)
}

/// Generate secure password with default configuration
pub fn generate_secure_password(length: usize) -> String {
    let security = PasswordSecurity::new();
    security.generate_secure_password(length)
}
