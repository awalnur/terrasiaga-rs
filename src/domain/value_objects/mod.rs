/// Domain value objects
/// Immutable objects that represent concepts with no identity, only values

pub mod email;
pub mod username;
pub mod phone_number;
pub mod coordinates;
pub mod address;
pub mod user_role;
pub mod user_status;

pub use email::Email;
pub use username::Username;
pub use phone_number::PhoneNumber;
pub use coordinates::Coordinates;
pub use address::Address;
pub use user_status::UserStatus;

// Backward compatibility: re-export ID types from shared::types
pub use crate::shared::types::{
    UserId,
    DisasterId,
    LocationId,
    ReportId,
    NotificationId,
    VolunteerId,
    EmergencyResponseId,
    ResourceId,
    OrganizationId,
    EvacuationRouteId,
    ShelterLocationId,
    AlertId,
    SessionId,
    UserRole,
};
