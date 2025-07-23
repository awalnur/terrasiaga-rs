/// User domain entity
/// Represents a user in the system with all business rules and invariants

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::shared::{UserId, AppResult, AppError, AuditFields};
use crate::domain::value_objects::{Email, Username, PhoneNumber};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: Email,
    pub username: Username,
    pub full_name: String,
    pub phone: Option<PhoneNumber>,
    pub password_hash: String,
    pub role: UserRole,
    pub profile: UserProfile,
    pub status: UserStatus,
    pub audit: AuditFields,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Reporter,    // Regular citizen who can report disasters
    Volunteer,   // Volunteer responder
    Official,    // Government official
    Admin,       // System administrator
    Analyst,     // Data analyst
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    PendingVerification,
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
    ) -> AppResult<Self> {
        // Business rule: Full name cannot be empty
        if full_name.trim().is_empty() {
            return Err(AppError::Validation("Full name cannot be empty".to_string()));
        }

        // Business rule: Password hash must be provided
        if password_hash.is_empty() {
            return Err(AppError::Validation("Password hash is required".to_string()));
        }

        let now = Utc::now();
        
        Ok(Self {
            id: UserId(uuid::Uuid::new_v4()),
            email,
            username,
            full_name,
            phone: None,
            password_hash,
            role: UserRole::Reporter, // Default role
            profile: UserProfile::default(),
            status: UserStatus::PendingVerification,
            audit: AuditFields {
                created_at: now,
                updated_at: now,
                created_by: None,
                updated_by: None,
            },
        })
    }

    /// Verify the user account
    pub fn verify(&mut self) -> AppResult<()> {
        match self.status {
            UserStatus::PendingVerification => {
                self.status = UserStatus::Active;
                self.audit.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(AppError::BusinessRuleViolation(
                "User is not in pending verification status".to_string()
            )),
        }
    }

    /// Suspend the user
    pub fn suspend(&mut self, reason: &str) -> AppResult<()> {
        match self.status {
            UserStatus::Active => {
                self.status = UserStatus::Suspended;
                self.audit.updated_at = Utc::now();
                // Log suspension reason (this would be handled by domain service)
                Ok(())
            }
            _ => Err(AppError::BusinessRuleViolation(
                "Can only suspend active users".to_string()
            )),
        }
    }

    /// Update user profile
    pub fn update_profile(&mut self, profile: UserProfile) -> AppResult<()> {
        self.profile = profile;
        self.audit.updated_at = Utc::now();
        Ok(())
    }

    /// Change user role (admin operation)
    pub fn change_role(&mut self, new_role: UserRole) -> AppResult<()> {
        // Business rule: Cannot change role of suspended users
        if self.status == UserStatus::Suspended {
            return Err(AppError::BusinessRuleViolation(
                "Cannot change role of suspended user".to_string()
            ));
        }

        self.role = new_role;
        self.audit.updated_at = Utc::now();
        Ok(())
    }

    /// Check if user can perform action based on role
    pub fn can_perform_action(&self, action: &UserAction) -> bool {
        if self.status != UserStatus::Active {
            return false;
        }

        match (&self.role, action) {
            (UserRole::Admin, _) => true,
            (UserRole::Official, UserAction::ValidateReport | UserAction::ManageResources) => true,
            (UserRole::Volunteer, UserAction::RespondToEmergency | UserAction::ValidateReport) => true,
            (UserRole::Analyst, UserAction::ViewAnalytics | UserAction::GenerateReports) => true,
            (UserRole::Reporter, UserAction::ReportDisaster | UserAction::ViewPublicData) => true,
            _ => false,
        }
    }

    /// Check if user is verified and active
    pub fn is_verified_and_active(&self) -> bool {
        self.status == UserStatus::Active
    }
}

#[derive(Debug, Clone)]
pub enum UserAction {
    ReportDisaster,
    ValidateReport,
    RespondToEmergency,
    ManageResources,
    ViewAnalytics,
    GenerateReports,
    ViewPublicData,
    AdminAction,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            bio: None,
            avatar_url: None,
            location: None,
            expertise: Vec::new(),
            languages: vec!["Indonesian".to_string()], // Default language
            emergency_contact: None,
        }
    }
}
