/// User management use cases
/// Handles user registration, profile management, and role-based operations

use async_trait::async_trait;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::use_cases::{UseCase, ValidatedUseCase};
use crate::domain::entities::User;
use crate::domain::value_objects::*;
use crate::domain::ports::repositories::UserRepository;
use crate::domain::ports::services::{AuthService, NotificationService};
use crate::domain::events::{UserRegisteredEvent, UserActivatedEvent, EventPublisher};
use crate::shared::{AppResult, AppError};
use crate::shared::types::Permission;

/// Request to register a new user
#[derive(Debug, Clone)]
pub struct RegisterUserRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub phone_number: Option<String>,
    pub role: UserRole,
    pub address: Option<Address>,
    pub emergency_contact: Option<String>,
}

/// Response after user registration
#[derive(Debug, Clone)]
pub struct UserResponse {
    pub id: UserId,
    pub email: Email,
    pub full_name: String,
    pub phone_number: Option<PhoneNumber>,
    pub role: UserRole,
    pub status: String,
    pub address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

/// Use case for user registration
pub struct RegisterUserUseCase {
    user_repository: Arc<dyn UserRepository>,
    auth_service: Arc<dyn AuthService>,
    notification_service: Arc<dyn NotificationService>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl RegisterUserUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        auth_service: Arc<dyn AuthService>,
        notification_service: Arc<dyn NotificationService>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            user_repository,
            auth_service,
            notification_service,
            event_publisher,
        }
    }
}

#[async_trait]
impl ValidatedUseCase<RegisterUserRequest, UserResponse> for RegisterUserUseCase {
    async fn validate(&self, request: &RegisterUserRequest) -> AppResult<()> {
        // Validate email format
        let _email = Email::new(request.email.clone())?;

        // Validate phone number if provided
        if let Some(phone) = &request.phone_number {
            PhoneNumber::new(phone.clone())?;
        }

        // Check if email already exists
        let email_obj = Email::new(request.email.clone())?;
        if self.user_repository.find_by_email(&email_obj).await?.is_some() {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        // Validate password strength
        if request.password.len() < 8 {
            return Err(AppError::Validation("Password must be at least 8 characters".to_string()));
        }

        // Validate full name
        if request.full_name.trim().is_empty() {
            return Err(AppError::Validation("Full name cannot be empty".to_string()));
        }

        if request.full_name.len() < 2 {
            return Err(AppError::Validation("Full name must be at least 2 characters".to_string()));
        }

        // Validate role permissions (only admins can create other admins)
        if matches!(request.role, UserRole::Admin | UserRole::SuperAdmin) {
            // In a real implementation, you'd check the requesting user's permissions
            // For now, we'll allow it but in production this should be restricted
        }

        Ok(())
    }
}

#[async_trait]
impl UseCase<RegisterUserRequest, UserResponse> for RegisterUserUseCase {
    async fn execute(&self, request: RegisterUserRequest) -> AppResult<UserResponse> {
        // Create value objects
        let email = Email::new(request.email)?;
        let phone_number = if let Some(phone) = request.phone_number {
            Some(PhoneNumber::new(phone)?)
        } else {
            None
        };

        // Derive a username from email local-part (fallback to full_name slug)
        let username = {
            let local = email.value().split('@').next().unwrap_or("");
            let candidate = if !local.is_empty() { local.to_string() } else { request.full_name.replace(' ', ".") };
            Username::new(candidate)?
        };

        // Hash password
        let password_hash = self.auth_service.hash_password(&request.password).await?;

        // Create user entity
        let mut user = User::new(
            email.clone(),
            username,
            request.full_name.clone(),
            password_hash,
            request.role.clone(),
        )?;

        // Optional fields
        user.phone_number = phone_number;
        if let Some(addr) = request.address.clone() {
            let line = format!(
                "{}, {}, {}, {}{}",
                addr.street,
                addr.city,
                addr.province,
                addr.country.unwrap_or_else(|| "".to_string()),
                addr.postal_code.map(|p| format!("-{}", p)).unwrap_or_default()
            ).trim().to_string();
            user.address = if line.trim_matches([',', ' ']).is_empty() { None } else { Some(line) };
        }
        if let Some(ec) = request.emergency_contact.clone() {
            user.profile.emergency_contact = Some(ec);
        }

        // Save user
        let saved_user = self.user_repository.save(&user).await?;

        // Publish domain event
        let event = UserRegisteredEvent {
            event_id: Uuid::new_v4(),
            user_id: *saved_user.id(),
            email: saved_user.email().clone(),
            role: saved_user.role().clone(),
            occurred_at: Utc::now(),
            version: saved_user.version as u64,
        };
        self.event_publisher.publish(&event).await?;

        // Send welcome email notification
        let _ = self.notification_service
            .send_email(saved_user.email().value(), "Welcome to Terra Siaga", "Your account has been created successfully.")
            .await;

        Ok(UserResponse {
            id: *saved_user.id(),
            email: saved_user.email().clone(),
            full_name: saved_user.full_name().to_string(),
            phone_number: saved_user.phone_number().cloned(),
            role: saved_user.role().clone(),
            status: saved_user.status().to_string(),
            address: saved_user.address.clone(),
            created_at: saved_user.created_at,
            last_login: saved_user.last_login,
        })
    }
}

/// Request to update user profile
#[derive(Debug, Clone)]
pub struct UpdateUserProfileRequest {
    pub user_id: UserId,
    pub full_name: Option<String>,
    pub phone_number: Option<String>,
    pub address: Option<Address>,
    pub emergency_contact: Option<String>,
    pub updated_by: UserId, // For audit trail
}

/// Use case for updating user profile
pub struct UpdateUserProfileUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl UpdateUserProfileUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }
}

#[async_trait]
impl ValidatedUseCase<UpdateUserProfileRequest, UserResponse> for UpdateUserProfileUseCase {
    async fn validate(&self, request: &UpdateUserProfileRequest) -> AppResult<()> {
        // Validate phone number if provided
        if let Some(phone) = &request.phone_number {
            if !phone.is_empty() {
                PhoneNumber::new(phone.clone())?;
            }
        }

        // Validate full name if provided
        if let Some(name) = &request.full_name {
            if name.trim().is_empty() {
                return Err(AppError::Validation("Full name cannot be empty".to_string()));
            }
            if name.len() < 2 {
                return Err(AppError::Validation("Full name must be at least 2 characters".to_string()));
            }
        }

        // Check if user exists
        self.user_repository
            .find_by_id(&request.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl UseCase<UpdateUserProfileRequest, UserResponse> for UpdateUserProfileUseCase {
    async fn execute(&self, request: UpdateUserProfileRequest) -> AppResult<UserResponse> {
        // Get existing user
        let mut user = self.user_repository
            .find_by_id(&request.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Update fields if provided
        if let Some(full_name) = request.full_name {
            let name = full_name.trim().to_string();
            if name.is_empty() { return Err(AppError::Validation("Full name cannot be empty".to_string())); }
            user.full_name = name;
        }

        if let Some(phone) = request.phone_number {
            user.phone_number = if phone.is_empty() { None } else { Some(PhoneNumber::new(phone)?) };
        }

        if let Some(address) = request.address {
            let line = format!(
                "{}, {}, {}, {}{}",
                address.street,
                address.city,
                address.province,
                address.country.unwrap_or_else(|| "".to_string()),
                address.postal_code.map(|p| format!("-{}", p)).unwrap_or_default()
            ).trim().to_string();
            user.address = if line.trim_matches([',', ' ']).is_empty() { None } else { Some(line) };
        }

        if let Some(ec) = request.emergency_contact {
            user.profile.emergency_contact = if ec.trim().is_empty() { None } else { Some(ec) };
        }

        // Update audit fields
        user.updated_at = Utc::now();
        user.version += 1;

        // Save updated user
        let saved_user = self.user_repository.update(&user).await?;

        Ok(UserResponse {
            id: *saved_user.id(),
            email: saved_user.email().clone(),
            full_name: saved_user.full_name().to_string(),
            phone_number: saved_user.phone_number().cloned(),
            role: saved_user.role().clone(),
            status: saved_user.status().to_string(),
            address: saved_user.address.clone(),
            created_at: saved_user.created_at,
            last_login: saved_user.last_login,
        })
    }
}

/// Request to activate/deactivate user
#[derive(Debug, Clone)]
pub struct ChangeUserStatusRequest {
    pub user_id: UserId,
    pub new_status: String, // "active", "inactive", "suspended", "banned"
    pub reason: Option<String>,
    pub changed_by: UserId,
}

/// Use case for changing user status
pub struct ChangeUserStatusUseCase {
    user_repository: Arc<dyn UserRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl ChangeUserStatusUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            user_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl ValidatedUseCase<ChangeUserStatusRequest, UserResponse> for ChangeUserStatusUseCase {
    async fn validate(&self, request: &ChangeUserStatusRequest) -> AppResult<()> {
        let valid_statuses = ["active", "inactive", "suspended", "banned"];
        if !valid_statuses.contains(&request.new_status.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid status. Must be one of: {}",
                valid_statuses.join(", ")
            )));
        }

        // Check if user exists
        self.user_repository
            .find_by_id(&request.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Check if requester has permission to change status
        let requester = self.user_repository
            .find_by_id(&request.changed_by)
            .await?
            .ok_or_else(|| AppError::NotFound("Requester not found".to_string()))?;

        if !requester.role().has_permission(&Permission::ManageUsers) {
            return Err(AppError::Forbidden("Insufficient permissions to change user status".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl UseCase<ChangeUserStatusRequest, UserResponse> for ChangeUserStatusUseCase {
    async fn execute(&self, request: ChangeUserStatusRequest) -> AppResult<UserResponse> {
        // Get existing user
        let mut user = self.user_repository
            .find_by_id(&request.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Update status
        match request.new_status.as_str() {
            "active" => { user.activate()?; }
            "inactive" => {
                user.status = UserStatus::Inactive;
                user.updated_at = Utc::now();
                user.version += 1;
            }
            "suspended" => { user.suspend(request.reason.clone())?; }
            "banned" => {
                user.status = UserStatus::Banned;
                user.updated_at = Utc::now();
                user.version += 1;
            }
            _ => {}
        }

        // Save updated user
        let saved_user = self.user_repository.update(&user).await?;

        // Publish appropriate domain event
        match request.new_status.as_str() {
            "active" => {
                let event = UserActivatedEvent {
                    event_id: Uuid::new_v4(),
                    user_id: request.user_id,
                    occurred_at: Utc::now(),
                    version: saved_user.version as u64,
                };
                self.event_publisher.publish(&event).await?;
            }
            "inactive" | "suspended" | "banned" => {
                let event = crate::domain::events::UserDeactivatedEvent {
                    event_id: Uuid::new_v4(),
                    user_id: request.user_id,
                    reason: request.reason.unwrap_or_else(|| format!("Status changed to {}", request.new_status)),
                    occurred_at: Utc::now(),
                    version: saved_user.version as u64,
                };
                self.event_publisher.publish(&event).await?;
            }
            _ => {}
        }

        Ok(UserResponse {
            id: *saved_user.id(),
            email: saved_user.email().clone(),
            full_name: saved_user.full_name().to_string(),
            phone_number: saved_user.phone_number().cloned(),
            role: saved_user.role().clone(),
            status: saved_user.status().to_string(),
            address: saved_user.address.clone(),
            created_at: saved_user.created_at,
            last_login: saved_user.last_login,
        })
    }
}
