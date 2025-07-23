/// Integration tests for Terra Siaga API endpoints
/// Tests complete request-response flow including authentication, validation, and database operations

use actix_web::{test, web, App, http::StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use terra_siaga::{
    config::AppConfig,
    infrastructure::AppContainer,
    presentation::api,
};

mod common;
use common::*;

#[cfg(test)]
mod api_integration_tests {
    use super::*;

    async fn create_test_app() -> impl actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    > {
        dotenv::dotenv().ok();

        // Setup test environment
        std::env::set_var("DATABASE_URL", "postgresql://postgres:postgres@localhost:5432/terrasiaga_test");
        std::env::set_var("REDIS_URL", "redis://localhost:6379/1");
        std::env::set_var("JWT_SECRET", "test_secret_key_minimum_32_characters_long");
        std::env::set_var("ENVIRONMENT", "test");

        let config = AppConfig::from_env().unwrap();
        let container = AppContainer::build(&config).await.unwrap();
        let container = Arc::new(container);

        test::init_service(
            App::new()
                .app_data(web::Data::new(container))
                .configure(api::configure_routes)
        ).await
    }

    #[actix_web::test]
    async fn test_health_endpoint() {
        let app = create_test_app().await;

        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "healthy");
        assert_eq!(body["service"], "terra-siaga");
    }

    #[actix_web::test]
    async fn test_user_registration_success() {
        let app = create_test_app().await;

        let registration_data = json!({
            "email": "test@example.com",
            "username": "testuser",
            "password": "securepassword123",
            "full_name": "Test User",
            "phone": "+6281234567890"
        });

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(&registration_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED);

        let body: Value = test::read_body_json(resp).await;
        assert!(body["access_token"].is_string());
        assert!(body["refresh_token"].is_string());
    }

    #[actix_web::test]
    async fn test_user_registration_validation_error() {
        let app = create_test_app().await;

        let invalid_data = json!({
            "email": "invalid-email", // Invalid email format
            "username": "test",
            "password": "123", // Too short
            "full_name": ""  // Empty name
        });

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(&invalid_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body: Value = test::read_body_json(resp).await;
        assert!(body["error"].is_object());
    }

    #[actix_web::test]
    async fn test_user_login_success() {
        let app = create_test_app().await;

        // First register a user
        let registration_data = json!({
            "email": "login_test@example.com",
            "username": "loginuser",
            "password": "securepassword123",
            "full_name": "Login Test User"
        });

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(&registration_data)
            .to_request();

        test::call_service(&app, req).await;

        // Now try to login
        let login_data = json!({
            "email": "login_test@example.com",
            "password": "securepassword123"
        });

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(&login_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: Value = test::read_body_json(resp).await;
        assert!(body["access_token"].is_string());
        assert!(body["user_id"].is_string());
    }

    #[actix_web::test]
    async fn test_disaster_creation_authenticated() {
        let app = create_test_app().await;

        // Register and login to get token
        let token = register_and_login(&app).await;

        let disaster_data = json!({
            "title": "Test Earthquake",
            "description": "A test earthquake for integration testing",
            "disaster_type": "earthquake",
            "severity": "medium",
            "latitude": -6.2088,
            "longitude": 106.8456,
            "address": "Jakarta, Indonesia",
            "affected_population": 1000
        });

        let req = test::TestRequest::post()
            .uri("/api/v1/disasters")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&disaster_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED);

        let body: Value = test::read_body_json(resp).await;
        assert!(body["id"].is_string());
        assert_eq!(body["title"], "Test Earthquake");
    }

    #[actix_web::test]
    async fn test_disaster_creation_unauthorized() {
        let app = create_test_app().await;

        let disaster_data = json!({
            "title": "Test Earthquake",
            "description": "A test earthquake",
            "disaster_type": "earthquake",
            "severity": "medium",
            "latitude": -6.2088,
            "longitude": 106.8456
        });

        let req = test::TestRequest::post()
            .uri("/api/v1/disasters")
            // No Authorization header
            .set_json(&disaster_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_get_disasters_list() {
        let app = create_test_app().await;
        let token = register_and_login(&app).await;

        // Create a test disaster first
        create_test_disaster(&app, &token).await;

        let req = test::TestRequest::get()
            .uri("/api/v1/disasters")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: Value = test::read_body_json(resp).await;
        assert!(body["disasters"].is_array());
    }

    #[actix_web::test]
    async fn test_get_disasters_with_filters() {
        let app = create_test_app().await;
        let token = register_and_login(&app).await;

        let req = test::TestRequest::get()
            .uri("/api/v1/disasters?status=active&severity=high&page=1&limit=10")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: Value = test::read_body_json(resp).await;
        assert!(body["disasters"].is_array());
        assert!(body["pagination"].is_object());
    }

    #[actix_web::test]
    async fn test_emergency_response_initiation() {
        let app = create_test_app().await;
        let token = register_and_login(&app).await;

        // Create a disaster first
        let disaster_id = create_test_disaster(&app, &token).await;

        let emergency_data = json!({
            "disaster_id": disaster_id,
            "response_type": "rescue",
            "priority": "high",
            "estimated_duration": 120,
            "required_resources": ["ambulance", "rescue_team"],
            "team_size": 8
        });

        let req = test::TestRequest::post()
            .uri("/api/v1/emergency/response")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&emergency_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED);

        let body: Value = test::read_body_json(resp).await;
        assert!(body["response_id"].is_string());
        assert_eq!(body["disaster_id"], disaster_id);
    }

    #[actix_web::test]
    async fn test_notification_creation() {
        let app = create_test_app().await;
        let token = register_and_login(&app).await;

        let notification_data = json!({
            "title": "Emergency Alert",
            "message": "Tsunami warning in coastal areas",
            "notification_type": "alert",
            "priority": "critical",
            "target_audience": "location_based",
            "location_filter": {
                "latitude": -8.3405,
                "longitude": 115.0920,
                "radius": 50
            },
            "channels": ["push", "sms", "whatsapp"]
        });

        let req = test::TestRequest::post()
            .uri("/api/v1/notifications")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&notification_data)
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED);

        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["title"], "Emergency Alert");
        assert_eq!(body["priority"], "critical");
    }

    #[actix_web::test]
    async fn test_analytics_dashboard() {
        let app = create_test_app().await;
        let token = register_and_login(&app).await;

        let req = test::TestRequest::get()
            .uri("/api/v1/analytics/dashboard")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: Value = test::read_body_json(resp).await;
        assert!(body["summary"].is_object());
        assert!(body["trends"].is_object());
    }

    #[actix_web::test]
    async fn test_location_geocoding() {
        let app = create_test_app().await;
        let token = register_and_login(&app).await;

        let req = test::TestRequest::get()
            .uri("/api/v1/locations/geocode?address=Monas,%20Jakarta")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: Value = test::read_body_json(resp).await;
        assert!(body["coordinates"].is_object());
        assert!(body["coordinates"]["latitude"].is_number());
        assert!(body["coordinates"]["longitude"].is_number());
    }

    // Helper functions
    async fn register_and_login(app: &impl actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >) -> String {
        // Register
        let registration_data = json!({
            "email": "integration_test@example.com",
            "username": "integrationuser",
            "password": "securepassword123",
            "full_name": "Integration Test User"
        });

        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(&registration_data)
            .to_request();

        let resp = test::call_service(app, req).await;
        let body: Value = test::read_body_json(resp).await;

        body["access_token"].as_str().unwrap().to_string()
    }

    async fn create_test_disaster(app: &impl actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >, token: &str) -> String {
        let disaster_data = json!({
            "title": "Integration Test Earthquake",
            "description": "Test earthquake for integration testing",
            "disaster_type": "earthquake",
            "severity": "medium",
            "latitude": -6.2088,
            "longitude": 106.8456,
            "address": "Jakarta, Indonesia"
        });

        let req = test::TestRequest::post()
            .uri("/api/v1/disasters")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&disaster_data)
            .to_request();

        let resp = test::call_service(app, req).await;
        let body: Value = test::read_body_json(resp).await;

        body["id"].as_str().unwrap().to_string()
    }
}
