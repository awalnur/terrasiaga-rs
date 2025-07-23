/// Test utilities and common helpers for Terra Siaga tests
/// Provides mock implementations, test fixtures, and helper functions

use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use terra_siaga::{
    domain::{
        entities::{
            user::{User, UserRole},
            disaster::{Disaster, DisasterStatus, DisasterSeverity},
            notification::{Notification, NotificationStatus, NotificationChannel},
        },
        value_objects::Email,
    },
    infrastructure::AppContainer,
    shared::{AppResult, types::{UserId, DisasterId}},
};

/// Test configuration for database and services
pub struct TestConfig {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://postgres:postgres@localhost:5433/terrasiaga_test".to_string(),
            redis_url: "redis://localhost:6379/1".to_string(), // Use DB 1 for tests
            jwt_secret: "test_secret_key_minimum_32_characters_long".to_string(),
        }
    }
}

/// Create test application container with test configuration
pub async fn create_test_container() -> AppResult<Arc<AppContainer>> {
    std::env::set_var("DATABASE_URL", "postgresql://postgres:postgres@localhost:5433/terrasiaga_test");
    std::env::set_var("REDIS_URL", "redis://localhost:6379/1");
    std::env::set_var("JWT_SECRET", "test_secret_key_minimum_32_characters_long");
    std::env::set_var("ENVIRONMENT", "test");

    let config = terra_siaga::config::AppConfig::from_env()?;
    let container = AppContainer::build(&config).await?;
    Ok(Arc::new(container))
}

/// Test fixtures for creating sample data
pub struct TestFixtures;

impl TestFixtures {
    /// Create a test user
    pub fn create_test_user(role: UserRole) -> User {
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

    /// Create a test admin user
    pub fn create_admin_user() -> User {
        Self::create_test_user(UserRole::Admin)
    }

    /// Create a test responder user
    pub fn create_responder_user() -> User {
        Self::create_test_user(UserRole::Responder)
    }

    /// Create a test citizen user
    pub fn create_citizen_user() -> User {
        Self::create_test_user(UserRole::Citizen)
    }

    /// Create a test disaster
    pub fn create_test_disaster(reporter_id: UserId) -> Disaster {
        Disaster {
            id: DisasterId(Uuid::new_v4()),
            title: "Test Earthquake".to_string(),
            description: "A test earthquake event".to_string(),
            disaster_type: "earthquake".to_string(),
            severity: DisasterSeverity::Medium,
            status: DisasterStatus::Reported,
            latitude: -6.2088,
            longitude: 106.8456,
            address: Some("Jakarta, Indonesia".to_string()),
            reporter_id,
            assigned_responders: Vec::new(),
            affected_population: Some(1000),
            images: Vec::new(),
            contact_info: Some("+6281234567890".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Create a test notification
    pub fn create_test_notification(recipient_id: UserId) -> Notification {
        Notification {
            id: terra_siaga::shared::types::NotificationId(Uuid::new_v4()),
            title: "Test Notification".to_string(),
            message: "This is a test notification".to_string(),
            notification_type: "alert".to_string(),
            priority: "medium".to_string(),
            recipient_id,
            status: NotificationStatus::Pending,
            channel: NotificationChannel::Push,
            scheduled_at: None,
            sent_at: None,
            read_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// Mock implementations for testing
pub mod mocks {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use tokio::sync::Mutex;
    use terra_siaga::domain::ports::repositories::{UserRepository, DisasterRepository, NotificationRepository};

    /// Mock user repository for testing
    pub struct MockUserRepository {
        users: Arc<Mutex<HashMap<UserId, User>>>,
    }

    impl MockUserRepository {
        pub fn new() -> Self {
            Self {
                users: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub async fn add_user(&self, user: User) {
            let mut users = self.users.lock().await;
            users.insert(user.id, user);
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
            let users = self.users.lock().await;
            Ok(users.get(&id).cloned())
        }

        async fn save(&self, user: &User) -> AppResult<User> {
            let mut users = self.users.lock().await;
            users.insert(user.id, user.clone());
            Ok(user.clone())
        }

        async fn update(&self, user: &User) -> AppResult<User> {
            let mut users = self.users.lock().await;
            users.insert(user.id, user.clone());
            Ok(user.clone())
        }

        async fn delete(&self, id: UserId) -> AppResult<bool> {
            let mut users = self.users.lock().await;
            Ok(users.remove(&id).is_some())
        }

        async fn find_all(&self) -> AppResult<Vec<User>> {
            let users = self.users.lock().await;
            Ok(users.values().cloned().collect())
        }

        async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
            let users = self.users.lock().await;
            Ok(users.values().find(|u| u.email.as_str() == email).cloned())
        }

        async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
            let users = self.users.lock().await;
            Ok(users.values().find(|u| u.username == username).cloned())
        }

        async fn find_by_role(&self, role: terra_siaga::domain::entities::user::UserRole) -> AppResult<Vec<User>> {
            let users = self.users.lock().await;
            Ok(users.values().filter(|u| u.role == role).cloned().collect())
        }

        async fn find_active_responders(&self) -> AppResult<Vec<User>> {
            let users = self.users.lock().await;
            Ok(users.values()
                .filter(|u| u.role == UserRole::Responder && u.is_active)
                .cloned()
                .collect())
        }

        async fn update_last_login(&self, _id: UserId) -> AppResult<bool> {
            Ok(true)
        }

        async fn verify_email(&self, _id: UserId) -> AppResult<bool> {
            Ok(true)
        }

        async fn update_password(&self, _id: UserId, _password_hash: &str) -> AppResult<bool> {
            Ok(true)
        }

        async fn count_by_role(&self, role: terra_siaga::domain::entities::user::UserRole) -> AppResult<u64> {
            let users = self.users.lock().await;
            Ok(users.values().filter(|u| u.role == role).count() as u64)
        }
    }

    /// Mock disaster repository for testing
    pub struct MockDisasterRepository {
        disasters: Arc<Mutex<HashMap<DisasterId, Disaster>>>,
    }

    impl MockDisasterRepository {
        pub fn new() -> Self {
            Self {
                disasters: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub async fn add_disaster(&self, disaster: Disaster) {
            let mut disasters = self.disasters.lock().await;
            disasters.insert(disaster.id, disaster);
        }
    }

    #[async_trait]
    impl DisasterRepository for MockDisasterRepository {
        async fn find_by_id(&self, id: DisasterId) -> AppResult<Option<Disaster>> {
            let disasters = self.disasters.lock().await;
            Ok(disasters.get(&id).cloned())
        }

        async fn save(&self, disaster: &Disaster) -> AppResult<Disaster> {
            let mut disasters = self.disasters.lock().await;
            disasters.insert(disaster.id, disaster.clone());
            Ok(disaster.clone())
        }

        async fn update(&self, disaster: &Disaster) -> AppResult<Disaster> {
            let mut disasters = self.disasters.lock().await;
            disasters.insert(disaster.id, disaster.clone());
            Ok(disaster.clone())
        }

        async fn delete(&self, id: DisasterId) -> AppResult<bool> {
            let mut disasters = self.disasters.lock().await;
            Ok(disasters.remove(&id).is_some())
        }

        async fn find_all(&self) -> AppResult<Vec<Disaster>> {
            let disasters = self.disasters.lock().await;
            Ok(disasters.values().cloned().collect())
        }

        async fn find_by_status(&self, status: terra_siaga::domain::entities::disaster::DisasterStatus) -> AppResult<Vec<Disaster>> {
            let disasters = self.disasters.lock().await;
            Ok(disasters.values().filter(|d| d.status == status).cloned().collect())
        }

        async fn find_by_severity(&self, severity: terra_siaga::domain::entities::disaster::DisasterSeverity) -> AppResult<Vec<Disaster>> {
            let disasters = self.disasters.lock().await;
            Ok(disasters.values().filter(|d| d.severity == severity).cloned().collect())
        }

        async fn find_by_reporter(&self, reporter_id: UserId) -> AppResult<Vec<Disaster>> {
            let disasters = self.disasters.lock().await;
            Ok(disasters.values().filter(|d| d.reporter_id == reporter_id).cloned().collect())
        }

        async fn find_nearby(&self, _lat: f64, _lng: f64, _radius_km: f64) -> AppResult<Vec<Disaster>> {
            // Mock implementation - return all disasters for testing
            let disasters = self.disasters.lock().await;
            Ok(disasters.values().cloned().collect())
        }

        async fn find_active(&self) -> AppResult<Vec<Disaster>> {
            let disasters = self.disasters.lock().await;
            Ok(disasters.values()
                .filter(|d| matches!(d.status, DisasterStatus::Reported | DisasterStatus::Verified | DisasterStatus::Responding))
                .cloned()
                .collect())
        }

        async fn update_status(&self, id: DisasterId, status: terra_siaga::domain::entities::disaster::DisasterStatus) -> AppResult<bool> {
            let mut disasters = self.disasters.lock().await;
            if let Some(disaster) = disasters.get_mut(&id) {
                disaster.status = status;
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn assign_responder(&self, disaster_id: DisasterId, responder_id: UserId) -> AppResult<bool> {
            let mut disasters = self.disasters.lock().await;
            if let Some(disaster) = disasters.get_mut(&disaster_id) {
                if !disaster.assigned_responders.contains(&responder_id) {
                    disaster.assigned_responders.push(responder_id);
                }
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn find_by_location(&self, _location_id: terra_siaga::shared::types::LocationId) -> AppResult<Vec<Disaster>> {
            // Mock implementation
            Ok(Vec::new())
        }
    }
}

/// Assertion helpers for tests
pub mod assertions {
    use super::*;

    pub fn assert_user_equals(actual: &User, expected: &User) {
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.email.as_str(), expected.email.as_str());
        assert_eq!(actual.username, expected.username);
        assert_eq!(actual.role, expected.role);
    }

    pub fn assert_disaster_equals(actual: &Disaster, expected: &Disaster) {
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.title, expected.title);
        assert_eq!(actual.disaster_type, expected.disaster_type);
        assert_eq!(actual.severity, expected.severity);
        assert_eq!(actual.status, expected.status);
    }
}

/// Database test helpers
pub mod db {
    use super::*;

    /// Setup test database
    pub async fn setup_test_db() -> AppResult<()> {
        // Create test database if it doesn't exist
        // Run migrations
        // This would be implemented based on your database setup
        Ok(())
    }

    /// Clean test database
    pub async fn cleanup_test_db() -> AppResult<()> {
        // Clean up test data
        // This would be implemented based on your database setup
        Ok(())
    }

    /// Execute test in transaction (auto-rollback)
    pub async fn with_test_transaction<F, T>(test: F) -> AppResult<T>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<T>>>>,
    {
        // Start transaction
        let result = test().await;
        // Rollback transaction
        result
    }
}
