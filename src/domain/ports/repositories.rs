/// Repository ports - Data access interfaces
/// These define contracts for data persistence that infrastructure must implement

use async_trait::async_trait;
use crate::AppError;
use crate::shared::{AppResult, UserId, DisasterId, LocationId, NotificationId, PaginationParams, PaginatedResponse};
use crate::domain::entities::notification::{Notification, NotificationStatus, NotificationChannel};
use crate::domain::entities::disaster::Disaster;
use crate::domain::entities::user::User;
use crate::domain::entities::location::Location;
use crate::domain::value_objects::{Coordinates, Email};

// Base repository trait with common CRUD operations
#[async_trait]
pub trait Repository<T, ID>: Send + Sync {
    async fn find_by_id(&self, id: &ID) -> AppResult<Option<T>>;
    async fn save(&self, entity: &T) -> AppResult<T>;
    async fn update(&self, entity: &T) -> AppResult<T>;
    async fn delete(&self, id: &ID) -> AppResult<bool>;
    async fn find_all(&self) -> AppResult<Vec<T>>;
}

// Disaster repository interface
#[async_trait]
pub trait DisasterRepository: Send + Sync {
    async fn find_by_id(&self, id: &DisasterId) -> AppResult<Option<Disaster>>;
    async fn save(&self, entity: &Disaster) -> AppResult<Disaster>;
    async fn update(&self, entity: &Disaster) -> AppResult<Disaster>;
    async fn delete(&self, id: &DisasterId) -> AppResult<bool>;
    async fn find_all(&self) -> AppResult<Vec<Disaster>>;
    
    async fn find_by_status(&self, status: crate::domain::entities::disaster::DisasterStatus) -> AppResult<Vec<Disaster>>;
    async fn find_by_severity(&self, severity: crate::domain::entities::disaster::DisasterSeverity) -> AppResult<Vec<Disaster>>;
    async fn find_by_reporter(&self, reporter_id: &UserId) -> AppResult<Vec<Disaster>>;
    async fn find_nearby(&self, lat: f64, lng: f64, radius_km: f64) -> AppResult<Vec<Disaster>>;
    async fn find_active(&self) -> AppResult<Vec<Disaster>>;
    async fn update_status(&self, id: &DisasterId, status: crate::domain::entities::disaster::DisasterStatus) -> AppResult<bool>;
    async fn assign_responder(&self, disaster_id: &DisasterId, responder_id: &UserId) -> AppResult<bool>;
    async fn find_by_location(&self, location_id: &LocationId) -> AppResult<Vec<Disaster>>;
}

// User repository interface
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, uid: &UserId) -> AppResult<Option<User>>;
    async fn save(&self, entity: &User) -> AppResult<User>;
    async fn update(&self, entity: &User) -> AppResult<User>;
    async fn delete(&self, uid: &UserId) -> AppResult<bool>;
    async fn find_all(&self) -> AppResult<Vec<User>>;
    
    async fn find_by_email(&self, email_val: &Email) -> AppResult<Option<User>>;
    async fn find_by_username(&self, username_val: &str) -> AppResult<Option<User>>;
    async fn find_by_role(&self, role: &crate::domain::value_objects::UserRole) -> AppResult<Vec<User>>;
    async fn find_active_responders(&self) -> AppResult<Vec<User>>;
    async fn update_last_login(&self, uid: &UserId) -> AppResult<bool>;
    async fn verify_email(&self, uid: &UserId) -> AppResult<bool>;
    async fn update_password(&self, uid: &UserId, password_hash: &str) -> AppResult<bool>;
    async fn count_by_role(&self, role: &crate::domain::value_objects::UserRole) -> AppResult<u64>;
    async fn find_users_in_radius(&self, center: &Coordinates, radius_km: f64) -> AppResult<Vec<User>>;
    async fn count_by_status(&self, status: &str) -> AppResult<u64>;
}

// Notification repository interface  
#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn find_by_id(&self, id: &NotificationId) -> AppResult<Option<Notification>>;
    async fn save(&self, entity: &Notification) -> AppResult<Notification>;
    async fn update(&self, entity: &Notification) -> AppResult<Notification>;
    async fn delete(&self, id: &NotificationId) -> AppResult<bool>;
    async fn find_all(&self) -> AppResult<Vec<Notification>>;
    
    async fn find_by_recipient(&self, recipient_id: &UserId) -> AppResult<Vec<Notification>>;
    async fn find_unread(&self, recipient_id: &UserId) -> AppResult<Vec<Notification>>;
    async fn mark_as_read(&self, id: &NotificationId) -> AppResult<bool>;
    async fn mark_all_as_read(&self, recipient_id: &UserId) -> AppResult<u64>;
    async fn delete_old_notifications(&self, days: u32) -> AppResult<u64>;
    async fn count_unread(&self, recipient_id: &UserId) -> AppResult<u64>;
    async fn find_by_status(&self, status: NotificationStatus) -> AppResult<Vec<Notification>>;
    async fn save_notification(&self, notification: &Notification) -> AppResult<Notification>;
    async fn find_by_user(&self, user_id: &UserId, limit: Option<u32>) -> AppResult<Vec<Notification>>;
    async fn find_unread_by_recipient(&self, recipient_id: UserId) -> AppResult<Vec<Notification>>;
    async fn find_by_channel(&self, channel: NotificationChannel) -> AppResult<Vec<Notification>>;
}

#[async_trait]
pub trait LocationRepository: Send + Sync {
    async fn find_by_id(&self, id: &LocationId) -> AppResult<Option<crate::domain::entities::location::Location>>;
    async fn save(&self, entity: &crate::domain::entities::location::Location) -> AppResult<crate::domain::entities::location::Location>;
    async fn update(&self, entity: &crate::domain::entities::location::Location) -> AppResult<crate::domain::entities::location::Location>;
    async fn delete(&self, id: &LocationId) -> AppResult<bool>;
    async fn find_all(&self) -> AppResult<Vec<crate::domain::entities::location::Location>>;
    async fn find_by_name(&self, name: &str) -> AppResult<Option<crate::domain::entities::location::Location>>;

    async fn find_by_region(&self, region: &str) -> AppResult<Vec<Location>>;
    async fn find_by_province(&self, province: &str) -> AppResult<Vec<Location>>;
    async fn find_by_coordinates(&self, lat: f64, lng: f64, radius_km: f64) -> AppResult<Vec<Location>>;
    async fn search_by_name(&self, name: &str) -> AppResult<Vec<Location>>;
    async fn find_nearby(&self, lat: f64, lng: f64, radius_km: f64) -> AppResult<Vec<Location>>;
    async fn save_location(&self, location: &Location) -> AppResult<Location>;
    async fn delete_location(&self, id: LocationId) -> AppResult<bool>;
}