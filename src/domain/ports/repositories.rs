/// Repository ports - Data access interfaces
/// These define contracts for data persistence that infrastructure must implement

use async_trait::async_trait;
use crate::shared::{AppResult, UserId, DisasterId, LocationId, NotificationId, PaginationParams, PaginatedResponse};
use crate::domain::entities::notification::{Notification, NotificationStatus, NotificationChannel};
use crate::domain::entities::disaster::Disaster;
use crate::domain::user::{User};
use crate::domain::location::{Location};
// Base repository trait with common CRUD operations
#[async_trait]
pub trait Repository<T, ID>: Send + Sync {
    async fn find_by_id(&self, id: ID) -> AppResult<Option<T>>;
    async fn save(&self, entity: &T) -> AppResult<T>;
    async fn update(&self, entity: &T) -> AppResult<T>;
    async fn delete(&self, id: ID) -> AppResult<bool>;
    async fn find_all(&self) -> AppResult<Vec<T>>;
}

// Disaster repository interface
#[async_trait]
pub trait DisasterRepository: Repository<Disaster, DisasterId> + Send + Sync {
    async fn find_by_status(&self, status: crate::domain::entities::disaster::DisasterStatus) -> AppResult<Vec<Disaster>>;
    async fn find_by_severity(&self, severity: crate::domain::entities::disaster::DisasterSeverity) -> AppResult<Vec<Disaster>>;
    async fn find_by_reporter(&self, reporter_id: UserId) -> AppResult<Vec<Disaster>>;
    async fn find_nearby(&self, lat: f64, lng: f64, radius_km: f64) -> AppResult<Vec<Disaster>>;
    async fn find_active(&self) -> AppResult<Vec<Disaster>>;
    async fn update_status(&self, id: DisasterId, status: crate::domain::entities::disaster::DisasterStatus) -> AppResult<bool>;
    async fn assign_responder(&self, disaster_id: DisasterId, responder_id: UserId) -> AppResult<bool>;
    async fn find_by_location(&self, location_id: LocationId) -> AppResult<Vec<Disaster>>;
}

// User repository interface
#[async_trait]
pub trait UserRepository: Repository<User, UserId> + Send + Sync {
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
    async fn find_by_role(&self, role: crate::domain::entities::user::UserRole) -> AppResult<Vec<User>>;
    async fn find_active_responders(&self) -> AppResult<Vec<User>>;
    async fn update_last_login(&self, id: UserId) -> AppResult<bool>;
    async fn verify_email(&self, id: UserId) -> AppResult<bool>;
    async fn update_password(&self, id: UserId, password_hash: &str) -> AppResult<bool>;
    async fn count_by_role(&self, role: crate::domain::entities::user::UserRole) -> AppResult<u64>;
}

// Location repository interface
#[async_trait]
pub trait LocationRepository: Repository<Location, LocationId> + Send + Sync {
    async fn find_by_coordinates(&self, lat: f64, lng: f64, radius_km: f64) -> AppResult<Vec<crate::domain::location::Location>>;
    async fn find_by_region(&self, region: &str) -> AppResult<Vec<crate::domain::location::Location>>;
    async fn find_by_province(&self, province: &str) -> AppResult<Vec<crate::domain::location::Location>>;
    async fn search_by_name(&self, name: &str) -> AppResult<Vec<crate::domain::location::Location>>;
    async fn find_nearby(&self, lat: f64, lng: f64, radius_km: f64) -> AppResult<Vec<crate::domain::location::Location>>;
    async fn save_location(&self, location: &crate::domain::location::Location) -> AppResult<crate::domain::location::Location>;
    async fn delete_location(&self, id: LocationId) -> AppResult<bool>;
}

// Notification repository interface
#[async_trait]
pub trait NotificationRepository: Repository<Notification, NotificationId> + Send + Sync {
    async fn find_by_recipient(&self, recipient_id: UserId) -> AppResult<Vec<Notification>>;
    async fn find_unread(&self, recipient_id: UserId) -> AppResult<Vec<Notification>>;
    async fn mark_as_read(&self, id: NotificationId) -> AppResult<bool>;
    async fn mark_all_as_read(&self, recipient_id: UserId) -> AppResult<u64>;
    async fn delete_old_notifications(&self, days: u32) -> AppResult<u64>;
    async fn count_unread(&self, recipient_id: UserId) -> AppResult<u64>;

    async fn find_by_status(&self, status: NotificationStatus) -> AppResult<Vec<Notification>>;

    async fn save_notification(&self, notification: &Notification) -> AppResult<Notification>;
    async fn find_unread_by_recipient(&self, recipient_id: UserId) -> AppResult<Vec<Notification>>;

    async fn find_by_channel(&self, channel: NotificationChannel) -> AppResult<Vec<Notification>>;
}

/// Analytics repository interface for complex queries and reporting
#[async_trait]
pub trait AnalyticsRepository: Send + Sync {
    async fn get_disaster_statistics(&self, from_date: chrono::DateTime<chrono::Utc>, to_date: chrono::DateTime<chrono::Utc>) -> AppResult<serde_json::Value>;
    async fn get_response_time_metrics(&self) -> AppResult<serde_json::Value>;
    async fn get_regional_disaster_counts(&self) -> AppResult<serde_json::Value>;
    async fn get_user_activity_stats(&self) -> AppResult<serde_json::Value>;
    async fn get_severity_distribution(&self) -> AppResult<serde_json::Value>;
}
