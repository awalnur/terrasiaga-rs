/// API Version 1 endpoints
/// Contains all v1 API routes organized by domain

use actix_web::web;

pub mod auth;
pub mod users;
pub mod disasters;
pub mod locations;
pub mod notifications;
pub mod analytics;
pub mod emergency;

/// Configure all v1 API routes
pub fn configure_v1_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/status", web::get().to(|| async { "API v1 is running" }))

        // Authentication & Authorization routes
        .service(
            web::scope("/auth")
                .configure(auth::configure_auth_routes)
        )

        // User management routes
        .service(
            web::scope("/users")
                .configure(users::configure_user_routes)
        )

        // Disaster management routes
        .service(
            web::scope("/disasters")
                .configure(disasters::configure_disaster_routes)
        )

        // Location & mapping routes
        .service(
            web::scope("/locations")
                .configure(locations::configure_location_routes)
        )

        // Notification routes
        .service(
            web::scope("/notifications")
                .configure(notifications::configure_notification_routes)
        )

        // Analytics & reporting routes
        .service(
            web::scope("/analytics")
                .configure(analytics::configure_analytics_routes)
        )

        // Emergency response routes
        .service(
            web::scope("/emergency")
                .configure(emergency::configure_emergency_routes)
        );
}
