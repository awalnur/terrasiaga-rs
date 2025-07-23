/// Domain ports - Interfaces for external dependencies
/// These define contracts that infrastructure must implement

pub mod repositories;
pub mod services;
pub mod events;

// Re-export port traits
pub use repositories::*;
pub use services::*;
pub use events::*;
