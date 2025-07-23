/// Domain layer - Core business logic and entities
/// Clean Architecture implementation with proper separation of concerns

// ✅ Core Clean Architecture components
pub mod entities;      // Business entities with rich behavior
pub mod value_objects; // Immutable value types
pub mod ports;         // Interfaces for external dependencies
pub mod events;        // Domain events
pub mod services;      // Domain services for complex business logic

// Re-export core domain types for convenience
pub use entities::*;
pub use value_objects::*;
pub use events::*;

// ✅ MIGRATION COMPLETED
// All legacy modules have been successfully migrated to Clean Architecture:
// ✅ analytics/     -> Complex queries moved to ports/repositories.rs
// ✅ auth/          -> Moved to application layer (not domain concern)
// ✅ emergency/     -> Migrated to entities/disaster.rs
// ✅ geography/     -> Migrated to entities/location.rs
// ✅ map/           -> Moved to application/services (not domain concern)
// ✅ notification/  -> Migrated to entities/notification.rs
// ✅ report/        -> Covered by entities/disaster.rs
// ✅ tracking/      -> Moved to application layer
// ✅ user/          -> Migrated to entities/user.rs
// ✅ repositories/  -> Migrated to ports/repositories.rs
