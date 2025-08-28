// filepath: /Users/development/RUST/terra-siaga/src/domain/value_objects/phone_number.rs
use serde::{Deserialize, Serialize};
use crate::shared::error::{AppError, AppResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    pub fn new<S: Into<String>>(s: S) -> AppResult<Self> {
        let phone = s.into().trim().to_string();
        if phone.is_empty() {
            return Err(AppError::Validation("Phone number cannot be empty".to_string()));
        }
        // Basic E.164-like validation
        if !phone.starts_with('+') || !phone[1..].chars().all(|c| c.is_ascii_digit()) || phone.len() < 8 || phone.len() > 20 {
            return Err(AppError::Validation("Invalid phone number format".to_string()));
        }
        Ok(Self(phone))
    }

    pub fn value(&self) -> &str { &self.0 }
}

impl std::fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
