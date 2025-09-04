/// Main application entry point
/// Sets up the complete application with Clean Architecture and dependency injection

use actix_web::{middleware, web, App, HttpServer, ResponseError};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use terra_siaga::{
    config::AppConfig,
    infrastructure::AppContainer,
    presentation::api,
    middleware::{cors, errors as error_middleware},
};
use terra_siaga::infrastructure::{HealthService, PasetoSecurityService};
use terra_siaga::infrastructure::monitoring::{DatabaseHealthChecker, CacheHealthChecker};
use terra_siaga::infrastructure::database::DbPool;
use terra_siaga::middleware::ErrorHandler;
// Add imports for JSON error handling
use actix_web::error::JsonPayloadError;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize enhanced logging with structured output
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "terra_siaga=debug,actix_web=debug,actix_server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(true))
        .init();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    info!("🚀 Starting Terra Siaga Emergency Response System...");

    // Load and validate configuration
    let config = AppConfig::from_env().map_err(|e| {
        error!("❌ Failed to load configuration: {}", e);
        e
    })?;

    info!("✅ Configuration loaded successfully");
    info!("🌍 Environment: {}", config.environment());
    info!("🖥️  Server: {}:{}", config.server.host, config.server.port);


    // Build application container with all dependencies
    let container = AppContainer::build(&config).await.map_err(|e| {
        error!("❌ Failed to build application container: {}", e);
        e
    })?;

    info!("📦 Application container built successfully");

    // Initialize health monitoring service
    let mut health_service = HealthService::new(
        env!("CARGO_PKG_VERSION").to_string(),
        config.environment().to_string()
    );

    // Add health checks for critical components
    if let Some(db_service) = container.database_pool() {
        let db_pool_arc: Arc<DbPool> = Arc::new(db_service.pool().clone());
        health_service.add_checker(Arc::new(
            DatabaseHealthChecker::new(
                "primary_database".to_string(),
                db_pool_arc,
            )
        ));
    }

    let cache_service = container.cache_service().clone();
    health_service.add_checker(Arc::new(
        CacheHealthChecker::new(
            "redis_cache".to_string(),
            cache_service,
        )
    ));

    info!("💊 Health monitoring service configured");

    // Prepare shared app data
    let app_data = web::Data::new(container);
    let health_data = web::Data::new(Arc::new(health_service));

    // Extract CORS origins to avoid lifetime issues (not required by current CORS config)
    // let cors_origins = config.server.cors_origins.clone();
    let server_config = config.server.clone();

    // Initialize metrics collection
    let _metrics_exporter = metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], 9090))
        .build()
        .map_err(|e| {
            warn!("⚠️  Failed to initialize Prometheus metrics: {}", e);
            e
        })?;

    info!("📊 Metrics collection initialized on port 9090");

    // Start HTTP server with enhanced configuration
    let server = HttpServer::new(move || {
        App::new()
            // Inject dependencies
            .app_data(app_data.clone())
            .app_data(health_data.clone())
            // Configure JSON extractor to return consistent JSON errors
            .app_data(
                web::JsonConfig::default()
                    .limit(1 << 20) // 1 MiB
                    .error_handler(|err, _req| {
                        // Map JSON payload errors to our AppError with 400 status
                        let message = match &err {
                            JsonPayloadError::Deserialize(e) => format!("Invalid JSON payload: {}", e),
                            JsonPayloadError::ContentType => "Unsupported Content-Type. Expecting application/json".to_string(),
                            JsonPayloadError::OverflowKnownLength { .. } | JsonPayloadError::Overflow { .. } =>
                                "JSON payload too large".to_string(),
                            _ => err.to_string(),
                        };
                        let app_err = terra_siaga::shared::error::AppError::BadRequest(message);
                        app_err.into()
                        // actix_web::error::InternalError::from_response(err, app_err.error_response()).into()
                    })
            )

            // Global middleware stack (order matters!)
            .wrap(middleware::Logger::new(
                r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#
            ))

            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("X-Version", env!("CARGO_PKG_VERSION")))
                    .add(("X-Service", "terra-siaga"))
            )
            .wrap(cors::configure_cors())
            .wrap(ErrorHandler::new())

            // Security headers
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("X-Content-Type-Options", "nosniff"))
                    .add(("X-Frame-Options", "DENY"))
                    .add(("X-XSS-Protection", "1; mode=block"))
                    .add(("Strict-Transport-Security", "max-age=31536000; includeSubDomains"))
                    .add(("Referrer-Policy", "strict-origin-when-cross-origin"))
            )

            // Configure API routes
            .configure(api::configure_routes)
    })
    .bind(format!("{}:{}", server_config.host, server_config.port))
    .map_err(|e| {
        error!("❌ Failed to bind server to {}:{}: {}", server_config.host, server_config.port, e);
        e
    })?
    .workers(server_config.workers.unwrap_or_else(|| {
        let cpu_count = num_cpus::get();
        info!("🔧 Auto-detected {} CPU cores, using {} workers", cpu_count, cpu_count);
        cpu_count
    }))
    .keep_alive(Duration::from_secs(server_config.keep_alive))
    .client_request_timeout(Duration::from_millis(
        server_config.client_timeout.num_milliseconds() as u64,
    ))
    .client_disconnect_timeout(Duration::from_secs(5))
    .shutdown_timeout(30);

    info!("🎯 Terra Siaga server starting on {}:{}", server_config.host, server_config.port);
    info!("📚 API documentation: http://{}:{}/docs", server_config.host, server_config.port);
    info!("💊 Health check: http://{}:{}/health", server_config.host, server_config.port);
    info!("📊 Metrics: http://{}:9090/metrics", server_config.host);

    // Graceful shutdown handler
    let server_handle = server.run();

    // Setup signal handlers for graceful shutdown
    tokio::select! {
        result = server_handle => {
            match result {
                Ok(_) => info!("✅ Server shut down gracefully"),
                Err(e) => error!("❌ Server error: {}", e),
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("🛑 Received Ctrl+C, initiating graceful shutdown...");
        }
    }

    info!("👋 Terra Siaga Emergency Response System shutdown complete");
    Ok(())
}
