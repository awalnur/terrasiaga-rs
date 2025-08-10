/// Security service implementations for Terra Siaga
/// Provides both JWT (legacy) and PASETO (recommended) implementations

pub mod service;
pub mod paseto_service;

// Re-export security services
pub use service::ProductionSecurityService;
pub use paseto_service::{PasetoSecurityService, PasetoConfig, SecureAuthSession};

use crate::shared::AppResult;

/// Security service factory
pub struct SecurityServiceFactory;

impl SecurityServiceFactory {
    /// Create PASETO-based security service (recommended for production)
    pub fn create_paseto_service(
        config: PasetoConfig,
        cache: std::sync::Arc<dyn crate::infrastructure::cache::CacheService>,
    ) -> AppResult<PasetoSecurityService> {
        PasetoSecurityService::new(config, cache)
    }
    
    /// Create JWT-based security service (legacy support)
    pub fn create_jwt_service(
        config: crate::infrastructure::security::service::SecurityConfig,
        cache: std::sync::Arc<dyn crate::infrastructure::cache::CacheService>,
    ) -> AppResult<ProductionSecurityService> {
        ProductionSecurityService::new(config, cache)
    }
}
