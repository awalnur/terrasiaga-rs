// filepath: /Users/development/RUST/terra-siaga/src/domain/value_objects/username.rs
use serde::{Deserialize, Serialize};
use crate::shared::error::{AppError, AppResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Username(String);

impl Username {
    pub fn new<S: Into<String>>(s: S) -> AppResult<Self> {
        let username = s.into().trim().to_string();
        if username.len() < 3 || username.len() > 32 {
            return Err(AppError::Validation("Username must be 3-32 characters".to_string()));
        }
        if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') {
            return Err(AppError::Validation("Username can contain letters, numbers, '_' and '.' only".to_string()));
        }
        Ok(Self(username))
    }

    pub fn value(&self) -> &str { &self.0 }
}

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
