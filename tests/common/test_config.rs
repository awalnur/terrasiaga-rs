/// Test configuration and setup for Terra Siaga
/// Handles test database setup, environment configuration, and test utilities

use std::sync::Once;
use tokio_postgres::{NoTls, Client};

static INIT: Once = Once::new();

/// Initialize test environment
pub async fn init_test_env() {
    INIT.call_once(|| {
        // Load test environment variables
        dotenv::from_filename(".env.test").ok();

        // Set test-specific environment variables
        std::env::set_var("ENVIRONMENT", "test");
        std::env::set_var("RUST_LOG", "debug");

        // Initialize test logging
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .try_init();
    });
}

/// Setup test database
pub async fn setup_test_database() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/postgres".to_string());

    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;

    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    // Create test database if it doesn't exist
    let _ = client.execute("DROP DATABASE IF EXISTS terrasiaga_test", &[]).await;
    client.execute("CREATE DATABASE terrasiaga_test", &[]).await?;

    // Update environment to use test database
    std::env::set_var("DATABASE_URL", "postgresql://postgres:postgres@localhost:5432/terrasiaga_test");

    Ok(())
}

/// Clean up test database
pub async fn cleanup_test_database() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = "postgresql://postgres:postgres@localhost:5432/postgres";
    let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });

    let _ = client.execute("DROP DATABASE IF EXISTS terrasiaga_test", &[]).await;

    Ok(())
}

/// Test data fixtures
pub struct TestData;

impl TestData {
    /// Load test data from SQL files
    pub async fn load_fixtures() -> Result<(), Box<dyn std::error::Error>> {
        // This would load test data from fixtures
        Ok(())
    }

    /// Clean all test data
    pub async fn clean_fixtures() -> Result<(), Box<dyn std::error::Error>> {
        // This would clean test data
        Ok(())
    }
}

/// Test timing utilities
pub struct TestTimer {
    start: std::time::Instant,
}

impl TestTimer {
    pub fn start() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    pub fn assert_max_duration(&self, max_duration: std::time::Duration, operation: &str) {
        let elapsed = self.elapsed();
        assert!(
            elapsed <= max_duration,
            "{} took {:?}, expected <= {:?}",
            operation,
            elapsed,
            max_duration
        );
    }
}
