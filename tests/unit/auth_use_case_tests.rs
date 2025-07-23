/// Unit tests for Authentication use cases
/// Tests login, registration, and token management business logic

use std::sync::Arc;
use tokio;
use terra_siaga::{
    application::use_cases::auth::{LoginUseCase, RegisterUseCase, LoginRequest, RegisterRequest},
    domain::entities::user::{User, UserRole},
    shared::AppResult,
};
use crate::common::{TestFixtures, mocks::MockUserRepository};

#[cfg(test)]
mod auth_use_case_tests {
    use super::*;

    #[tokio::test]
    async fn test_successful_login() {
        // Arrange
        let user_repo = Arc::new(MockUserRepository::new());
        let auth_service = Arc::new(MockAuthService::new());

        let test_user = TestFixtures::create_citizen_user();
        user_repo.add_user(test_user.clone()).await;

        let login_use_case = LoginUseCase::new(user_repo, auth_service);

        let request = LoginRequest {
            email: test_user.email.as_str().to_string(),
            password: "correct_password".to_string(),
        };

        // Act
        let result = login_use_case.execute(request).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.user_id, test_user.id);
        assert!(!response.access_token.is_empty());
        assert!(!response.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_login_with_invalid_email() {
        // Arrange
        let user_repo = Arc::new(MockUserRepository::new());
        let auth_service = Arc::new(MockAuthService::new());
        let login_use_case = LoginUseCase::new(user_repo, auth_service);

        let request = LoginRequest {
            email: "nonexistent@example.com".to_string(),
            password: "password".to_string(),
        };

        // Act
        let result = login_use_case.execute(request).await;

        // Assert
        assert!(result.is_err());
        // Should return authentication error
    }

    #[tokio::test]
    async fn test_login_with_wrong_password() {
        // Arrange
        let user_repo = Arc::new(MockUserRepository::new());
        let auth_service = Arc::new(MockAuthService::new());

        let test_user = TestFixtures::create_citizen_user();
        user_repo.add_user(test_user.clone()).await;

        let login_use_case = LoginUseCase::new(user_repo, auth_service);

        let request = LoginRequest {
            email: test_user.email.as_str().to_string(),
            password: "wrong_password".to_string(),
        };

        // Act
        let result = login_use_case.execute(request).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_successful_registration() {
        // Arrange
        let user_repo = Arc::new(MockUserRepository::new());
        let auth_service = Arc::new(MockAuthService::new());
        let register_use_case = RegisterUseCase::new(user_repo.clone(), auth_service);

        let request = RegisterRequest {
            email: "newuser@example.com".to_string(),
            username: "newuser".to_string(),
            password: "securepassword123".to_string(),
            full_name: "New User".to_string(),
            phone: Some("+6281234567890".to_string()),
        };

        // Act
        let result = register_use_case.execute(request).await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.access_token.is_empty());

        // Verify user was saved
        let saved_user = user_repo.find_by_email("newuser@example.com").await.unwrap();
        assert!(saved_user.is_some());
        assert_eq!(saved_user.unwrap().username, "newuser");
    }

    #[tokio::test]
    async fn test_registration_with_existing_email() {
        // Arrange
        let user_repo = Arc::new(MockUserRepository::new());
        let auth_service = Arc::new(MockAuthService::new());

        let existing_user = TestFixtures::create_citizen_user();
        user_repo.add_user(existing_user.clone()).await;

        let register_use_case = RegisterUseCase::new(user_repo, auth_service);

        let request = RegisterRequest {
            email: existing_user.email.as_str().to_string(), // Same email
            username: "differentuser".to_string(),
            password: "securepassword123".to_string(),
            full_name: "Different User".to_string(),
            phone: None,
        };

        // Act
        let result = register_use_case.execute(request).await;

        // Assert
        assert!(result.is_err());
        // Should return conflict error
    }

    #[tokio::test]
    async fn test_registration_with_existing_username() {
        // Arrange
        let user_repo = Arc::new(MockUserRepository::new());
        let auth_service = Arc::new(MockAuthService::new());

        let existing_user = TestFixtures::create_citizen_user();
        user_repo.add_user(existing_user.clone()).await;

        let register_use_case = RegisterUseCase::new(user_repo, auth_service);

        let request = RegisterRequest {
            email: "different@example.com".to_string(),
            username: existing_user.username.clone(), // Same username
            password: "securepassword123".to_string(),
            full_name: "Different User".to_string(),
            phone: None,
        };

        // Act
        let result = register_use_case.execute(request).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_registration_validation_weak_password() {
        // Arrange
        let user_repo = Arc::new(MockUserRepository::new());
        let auth_service = Arc::new(MockAuthService::new());
        let register_use_case = RegisterUseCase::new(user_repo, auth_service);

        let request = RegisterRequest {
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            password: "123".to_string(), // Too weak
            full_name: "Test User".to_string(),
            phone: None,
        };

        // Act
        let result = register_use_case.execute(request).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_registration_validation_invalid_email() {
        // Arrange
        let user_repo = Arc::new(MockUserRepository::new());
        let auth_service = Arc::new(MockAuthService::new());
        let register_use_case = RegisterUseCase::new(user_repo, auth_service);

        let request = RegisterRequest {
            email: "invalid-email".to_string(), // Invalid format
            username: "testuser".to_string(),
            password: "securepassword123".to_string(),
            full_name: "Test User".to_string(),
            phone: None,
        };

        // Act
        let result = register_use_case.execute(request).await;

        // Assert
        assert!(result.is_err());
    }
}

/// Mock authentication service for testing
struct MockAuthService;

impl MockAuthService {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl terra_siaga::domain::ports::services::AuthService for MockAuthService {
    async fn hash_password(&self, password: &str) -> AppResult<String> {
        if password.len() < 8 {
            return Err(terra_siaga::shared::AppError::Validation(
                "Password too short".to_string()
            ));
        }
        Ok(format!("$2b$12$hashed_{}", password))
    }

    async fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
        if password == "correct_password" {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn generate_tokens(&self, user_id: terra_siaga::shared::types::UserId) -> AppResult<terra_siaga::domain::ports::services::TokenPair> {
        Ok(terra_siaga::domain::ports::services::TokenPair {
            access_token: format!("access_token_for_{}", user_id.0),
            refresh_token: format!("refresh_token_for_{}", user_id.0),
            expires_in: 3600,
        })
    }

    async fn verify_token(&self, token: &str) -> AppResult<terra_siaga::shared::types::UserId> {
        if token.starts_with("access_token_for_") {
            Ok(terra_siaga::shared::types::UserId(uuid::Uuid::new_v4()))
        } else {
            Err(terra_siaga::shared::AppError::Authentication("Invalid token".to_string()))
        }
    }

    async fn refresh_token(&self, refresh_token: &str) -> AppResult<terra_siaga::domain::ports::services::TokenPair> {
        if refresh_token.starts_with("refresh_token_for_") {
            Ok(terra_siaga::domain::ports::services::TokenPair {
                access_token: "new_access_token".to_string(),
                refresh_token: "new_refresh_token".to_string(),
                expires_in: 3600,
            })
        } else {
            Err(terra_siaga::shared::AppError::Authentication("Invalid refresh token".to_string()))
        }
    }
}
