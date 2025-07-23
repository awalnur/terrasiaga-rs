/// CORS middleware configuration
/// Handles Cross-Origin Resource Sharing for the API

use actix_cors::Cors;
use actix_web::http;

pub fn configure_cors() -> Cors {
    Cors::default()
        .allowed_origin_fn(|origin, _req_head| {
            // Allow all origins in development, restrict in production
            origin.as_bytes().starts_with(b"http://localhost") || 
            origin.as_bytes().starts_with(b"https://")
        })
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::header::CONTENT_TYPE,
        ])
        .supports_credentials()
        .max_age(3600)
}
