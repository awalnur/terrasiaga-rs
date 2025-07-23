/// Logging middleware configuration
/// Handles request/response logging and tracing

use actix_web::middleware::Logger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logger() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "terra_siaga=debug,actix_web=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub fn configure_logger() -> Logger {
    Logger::default()
        .exclude("/health")
        .exclude("/metrics")
}
