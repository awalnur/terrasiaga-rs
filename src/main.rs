/// Main application entry point
/// Sets up the complete application with Clean Architecture and dependency injection
use actix_web::{middleware, web, App, HttpServer};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};


use terra_siaga::{
    config::AppConfig, infrastructure::AppContainer, presentation::api,
    middleware::cors,
    middleware::errors  as error_middleware,
};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    // This assumes the .env file is in the root of the project directory
    // and contains necessary configuration like database URLs, server ports, etc.
    dotenv::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = AppConfig::from_env().map_err(|e| {
        error!("Failed to load configuration: {}", e);
        e
    })?;

    info!("Starting Terra Siaga application...");
    info!("Environment: {}", config.environment());
    info!("Server: {}:{}", config.server.host, config.server.port);

    // Build application container with all dependencies
    let container = AppContainer::build(&config).await.map_err(|e| {
        error!("Failed to build application container: {}", e);
        e
    })?;

    let app_data = web::Data::new(container);

    // Extract CORS origins to avoid lifetime issues
    let cors_origins = config.server.cors_origins.clone();

    // Start HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(
                cors::configure_cors( )
            )
            .wrap(error_middleware::ErrorHandler::new())
            .configure(api::configure_routes)
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .workers(config.server.workers.unwrap_or_else(|| num_cpus::get()))
    .keep_alive(Duration::from_secs(config.server.keep_alive))
    .client_request_timeout(Duration::from_millis(
        config.server.client_timeout.num_milliseconds() as u64,
    ))
    .run();

    info!("Terra Siaga server started successfully!");

    server.await?;
    Ok(())
}
