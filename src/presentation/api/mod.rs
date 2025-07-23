/// API layer with versioning support
/// Handles HTTP endpoints and request/response mapping

pub mod v1;
pub mod health;

use actix_web::web;

/// Configure all API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(health::health_check)
        .service(health::readiness_check)
        .service(
            web::scope("/api")
                .service(
                    web::scope("/v1")
                        .configure(v1::configure_v1_routes)
                )
        );
}
