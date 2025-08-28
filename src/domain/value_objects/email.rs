// filepath: /Users/development/RUST/terra-siaga/src/domain/value_objects/email.rs
use serde::{Deserialize, Serialize};
use crate::shared::error::{AppError, AppResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Email(String);

impl Email {
    pub fn new<S: Into<String>>(s: S) -> AppResult<Self> {
        let email = s.into().trim().to_lowercase();
        if email.is_empty() {
            return Err(AppError::Validation("Email cannot be empty".to_string()));
        }
        // Basic validation; more robust checks can live in shared::validation
        if !email.contains('@') || email.len() > 254 {
            return Err(AppError::Validation("Invalid email format".to_string()));
        }


        Ok(Self(email))
    }

    pub fn value(&self) -> &str { &self.0 }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
