/// Repository implementations for the infrastructure layer

use diesel::prelude::*;
pub mod user_repository;
pub mod disaster_repository;
pub mod location_repository;
pub mod notification_repository;

// Re-export repository implementations
pub use user_repository::PostgresUserRepository;
pub use disaster_repository::PostgresDisasterRepository;
pub use location_repository::PostgresLocationRepository;
pub use notification_repository::PostgresNotificationRepository;
