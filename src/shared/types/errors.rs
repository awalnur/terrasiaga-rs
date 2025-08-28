// filepath: /Users/development/RUST/terra-siaga/src/shared/types/errors.rs
//! Re-exports for error types to provide structured access under shared::types

pub use crate::shared::error::{
    AppError,
    DomainError,
    ErrorSeverity,
    ErrorContext,
    ContextualError,
    ErrorRecovery,
};
