/// External API integrations module
/// Organizes all third-party service integrations

pub mod email;
pub mod geo_service;
pub mod weather_api;
pub mod whatsapp;

// Re-export for convenience
pub use email::*;
pub use geo_service::*;
pub use weather_api::*;
pub use whatsapp::*;
