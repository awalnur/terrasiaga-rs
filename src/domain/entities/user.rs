/// User domain entity
/// Represents a user in the system with all business rules and invariants

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::shared::{UserId, AppResult, AppError, AuditFields, UserRole};
use crate::domain::value_objects::{Email, Username, PhoneNumber, UserStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: Email,
    pub username: Username,
    pub full_name: String,
    pub phone_number: Option<PhoneNumber>,
    pub password_hash: String,
    pub role: UserRole,
    pub profile: UserProfile,
    pub status: UserStatus,
    pub address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
    pub expertise: Vec<String>,
    pub languages: Vec<String>,
    pub emergency_contact: Option<String>,
}

impl User {
    /// Create a new user with proper validation
    pub fn new(
        email: Email,
        username: Username,
        full_name: String,
        password_hash: String,
        role: UserRole,
    ) -> AppResult<Self> {
        if full_name.trim().is_empty() {
            return Err(AppError::Validation("Full name cannot be empty".to_string()));
        }

        let now = Utc::now();

        Ok(User {
            id: UserId::new(),
            email,
            username,
            full_name: full_name.trim().to_string(),
            phone_number: None,
            password_hash,
            role,
            profile: UserProfile {
                bio: None,
                avatar_url: None,
                location: None,
                expertise: vec![],
                languages: vec!["id".to_string()], // Default to Indonesian
                emergency_contact: None,
            },
            status: UserStatus::Pending,
            address: None,
            created_at: now,
            updated_at: now,
            last_login: None,
            version: 1,
        })
    }

    // Getter methods
    pub fn id(&self) -> &UserId {
        &self.id
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn username(&self) -> &Username {
        &self.username
    }

    pub fn full_name(&self) -> &str {
        &self.full_name
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn role(&self) -> &UserRole {
        &self.role
    }

    pub fn status(&self) -> &UserStatus {
        &self.status
    }

    pub fn phone_number(&self) -> Option<&PhoneNumber> {
        self.phone_number.as_ref()
    }

    // Business methods
    pub fn can_login(&self) -> bool {
        self.status.can_login()
    }

    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    pub fn activate(&mut self) -> AppResult<()> {
        if matches!(self.status, UserStatus::Banned) {
            return Err(AppError::Authorization("Cannot activate banned user".to_string()));
        }

        self.status = UserStatus::Active;
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    pub fn suspend(&mut self, reason: Option<String>) -> AppResult<()> {
        if matches!(self.status, UserStatus::SysAdmin) {
            return Err(AppError::Authorization("Cannot suspend super admin".to_string()));
        }

        self.status = UserStatus::Suspended;
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
        self.updated_at = Utc::now();
        self.version += 1;
    }

    pub fn update_profile(&mut self, profile: UserProfile) {
        self.profile = profile;
        self.updated_at = Utc::now();
        self.version += 1;
    }

}
