// Query handlers for read operations (CQRS pattern)
// Queries represent data retrieval operations without side effects

use crate::shared::error::AppResult;
use async_trait::async_trait;

/// Base trait for all query handlers
#[async_trait]
pub trait QueryHandler<T> {
    type Output;
    
    async fn handle(&self, query: T) -> AppResult<Self::Output>;
}

/// Marker trait for queries
pub trait Query {}

// Query modules for different domains
pub mod emergency;
pub mod user;
pub mod analytics;
pub mod map;
pub mod notification;
