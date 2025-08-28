pub mod service;

pub use service::*;

/// Cache service implementation for Terra Siaga
/// Provides high-performance Redis-based caching with fallback mechanisms

use redis::{AsyncCommands};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Cache keys for different entities
pub struct CacheKeys;

impl CacheKeys {
    pub fn user(user_id: &Uuid) -> String {
        format!("user:{}", user_id)
    }
    
    pub fn user_by_email(email: &str) -> String {
        format!("user:email:{}", email)
    }

    pub fn user_session(session_id: &str) -> String {
        format!("session:{}", session_id)
    }
    
    pub fn disaster(disaster_id: &Uuid) -> String {
        format!("disaster:{}", disaster_id)
    }
    
    pub fn location(location_id: &Uuid) -> String {
        format!("location:{}", location_id)
    }
    
    pub fn notification(notification_id: &Uuid) -> String {
        format!("notification:{}", notification_id)
    }
    
    pub fn rate_limit(identifier: &str) -> String {
        format!("rate_limit:{}", identifier)
    }
}
