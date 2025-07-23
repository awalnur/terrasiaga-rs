// Command handlers for write operations (CQRS pattern)
// Commands represent state-changing operations

use crate::shared::error::AppResult;
use async_trait::async_trait;

/// Base trait for all command handlers
#[async_trait]
pub trait CommandHandler<T> {
    type Output;
    
    async fn handle(&self, command: T) -> AppResult<Self::Output>;
}

/// Marker trait for commands
pub trait Command {}

// Emergency/Report Commands
pub mod emergency;
pub mod user;
pub mod notification;
pub mod tracking;
