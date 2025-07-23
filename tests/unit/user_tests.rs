/// Unit tests for User domain entity
/// Tests business logic and validation rules

use terra_siaga::domain::{
    entities::user::{User, UserRole},
    value_objects::Email,
};
use terra_siaga::shared::types::UserId;
use uuid::Uuid;
use chrono::Utc;

#[cfg(test)]
mod user_entity_tests {
    use super::*;

    fn create_test_user(role: UserRole) -> User {
        User {
            id: UserId(Uuid::new_v4()),
            email: Email::new("test@example.com".to_string()).unwrap(),
            username: "testuser".to_string(),
            full_name: "Test User".to_string(),
            password_hash: "$2b$12$test_hash".to_string(),
            role,
            is_active: true,
            is_verified: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_user_creation() {
        let user = create_test_user(UserRole::Citizen);

        assert_eq!(user.email.as_str(), "test@example.com");
        assert_eq!(user.username, "testuser");
        assert_eq!(user.role, UserRole::Citizen);
        assert!(user.is_active);
        assert!(user.is_verified);
    }

    #[test]
    fn test_admin_can_approve_disaster() {
        let admin = create_test_user(UserRole::Admin);
        assert!(admin.can_approve_disaster());
    }

    #[test]
    fn test_responder_can_approve_disaster() {
        let responder = create_test_user(UserRole::Responder);
        assert!(responder.can_approve_disaster());
    }

    #[test]
    fn test_citizen_cannot_approve_disaster() {
        let citizen = create_test_user(UserRole::Citizen);
        assert!(!citizen.can_approve_disaster());
    }

    #[test]
    fn test_user_roles_equality() {
        assert_eq!(UserRole::Admin, UserRole::Admin);
        assert_ne!(UserRole::Admin, UserRole::Citizen);
        assert_ne!(UserRole::Responder, UserRole::Citizen);
    }

    #[test]
    fn test_user_activation() {
        let mut user = create_test_user(UserRole::Citizen);
        user.is_active = false;

        assert!(!user.is_active);

        user.activate();
        assert!(user.is_active);
    }

    #[test]
    fn test_user_suspension() {
        let mut user = create_test_user(UserRole::Citizen);

        let result = user.suspend("Violation of terms");
        assert!(result.is_ok());
        assert!(!user.is_active);
    }

    #[test]
    fn test_user_email_update() {
        let mut user = create_test_user(UserRole::Citizen);
        let new_email = Email::new("newemail@example.com".to_string()).unwrap();

        user.update_email(new_email.clone());
        assert_eq!(user.email.as_str(), "newemail@example.com");
    }
}

#[cfg(test)]
mod email_value_object_tests {
    use super::*;

    #[test]
    fn test_valid_email_creation() {
        let email = Email::new("valid@example.com".to_string());
        assert!(email.is_ok());
        assert_eq!(email.unwrap().as_str(), "valid@example.com");
    }

    #[test]
    fn test_invalid_email_creation() {
        let email = Email::new("invalid-email".to_string());
        assert!(email.is_err());
    }

    #[test]
    fn test_empty_email_creation() {
        let email = Email::new("".to_string());
        assert!(email.is_err());
    }

    #[test]
    fn test_email_domain_extraction() {
        let email = Email::new("user@example.com".to_string()).unwrap();
        assert_eq!(email.domain(), "example.com");
    }

    #[test]
    fn test_email_local_part() {
        let email = Email::new("user@example.com".to_string()).unwrap();
        assert_eq!(email.local_part(), "user");
    }
}
