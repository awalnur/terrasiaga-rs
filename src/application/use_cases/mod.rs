/// Use cases - Application business logic
/// Each use case represents a single business operation

pub mod auth;
pub mod disaster;
pub mod notification;
pub mod user_management;
pub mod emergency_response;

// Re-export use cases
pub use auth::*;
pub use disaster::*;
pub use notification::*;
pub use user_management::*;
pub use emergency_response::*;

// Common use case traits and types
use async_trait::async_trait;
use crate::shared::AppResult;

/// Base trait for all use cases
#[async_trait]
pub trait UseCase<Request, Response> {
    async fn execute(&self, request: Request) -> AppResult<Response>;
}

/// Use case with validation
#[async_trait]
pub trait ValidatedUseCase<Request: Send + 'static, Response>: UseCase<Request, Response> {
    async fn validate(&self, request: &Request) -> AppResult<()>;
    
    async fn execute_validated(&self, request: Request) -> AppResult<Response> {
        self.validate(&request).await?;
        self.execute(request).await
    }
}
