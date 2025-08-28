/// Authentication use cases
/// Handles user authentication workflows

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;
use crate::shared::{AppResult, UserId};
use crate::application::use_cases::{UseCase, ValidatedUseCase};
use crate::domain::{Email, User, Username};
use crate::domain::ports::repositories::UserRepository;
use crate::domain::ports::services::AuthService;
use crate::shared::UserRole;

// DTOs for authentication use cases
#[derive(Debug, serde::Deserialize, validator::Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, serde::Serialize)]
pub struct LoginResponse {
    pub user_id: UserId,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, serde::Deserialize, validator::Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 30))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
    pub full_name: String,
    pub phone: Option<String>,
}

// Use case implementations
pub struct LoginUseCase {
    user_repository: Arc<dyn UserRepository + Send + Sync>,
    auth_service: Arc<dyn AuthService + Send + Sync>,
}

impl LoginUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository + Send + Sync>,
        auth_service: Arc<dyn AuthService + Send + Sync>,
    ) -> Self {
        Self {
            user_repository,
            auth_service,
        }
    }
}

#[async_trait]
impl UseCase<LoginRequest, LoginResponse> for LoginUseCase {
    async fn execute(&self, request: LoginRequest) -> AppResult<LoginResponse> {
        // Find user by email
        let email = Email::new(request.email.clone())?;
        let user = self.user_repository
            .find_by_email(&email)
            .await?
            .ok_or_else(|| crate::shared::AppError::Authentication("Invalid credentials".to_string()))?;

        // Verify password
        if !self.auth_service.verify_password(&request.password, &user.password_hash).await? {
            return Err(crate::shared::AppError::Authentication("Invalid credentials".to_string()));
        }

        // Generate tokens
        let tokens = self.auth_service.generate_tokens(user.id).await?;

        Ok(LoginResponse {
            user_id: user.id,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
        })
    }
}

#[async_trait]
impl ValidatedUseCase<LoginRequest, LoginResponse> for LoginUseCase {
    async fn validate(&self, request: &LoginRequest) -> AppResult<()> {
        use validator::Validate;
        request.validate()
            .map_err(|e| crate::shared::AppError::Validation(format!("Validation failed: {:?}", e)))?;
        Ok(())
    }
}

pub struct RegisterUseCase {
    user_repository: Arc<dyn UserRepository + Send + Sync>,
    auth_service: Arc<dyn AuthService + Send + Sync>,
}

impl RegisterUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository + Send + Sync>,
        auth_service: Arc<dyn AuthService + Send + Sync>,
    ) -> Self {
        Self {
            user_repository,
            auth_service,
        }
    }
}

#[async_trait]
impl ValidatedUseCase<RegisterRequest, LoginResponse> for RegisterUseCase {
    async fn validate(&self, request: &RegisterRequest) -> AppResult<()> {
        let email = Email::new(request.email.clone())?;
        use validator::Validate;
        request.validate()
            .map_err(|e| crate::shared::AppError::Validation(format!("Validation failed: {:?}", e)))?;

        // Check if email already exists
        if self.user_repository.find_by_email(&email).await?.is_some() {
            return Err(crate::shared::AppError::Conflict("Email already exists".to_string()));
        }

        // Check if username already exists
        if self.user_repository.find_by_username(&request.username).await?.is_some() {
            return Err(crate::shared::AppError::Conflict("Username already exists".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl UseCase<RegisterRequest, LoginResponse> for RegisterUseCase {
    async fn execute(&self, request: RegisterRequest) -> AppResult<LoginResponse> {
        // Hash password
        let password_hash = self.auth_service.hash_password(&request.password).await?;

        // Create user using value objects
        let email = Email::new(request.email)?;
        let username =Username::new(request.username)?;
        let role = UserRole::Volunteer;

        let user = User::new(
            email,
            username,
            request.full_name,
            password_hash,
            role,
        )?;

        // Save user
        let saved_user = self.user_repository.save(&user).await?;

        // Generate tokens
        let tokens = self.auth_service.generate_tokens(saved_user.id).await?;

        Ok(LoginResponse {
            user_id: saved_user.id,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            expires_in: tokens.expires_in,
        })
    }
}
