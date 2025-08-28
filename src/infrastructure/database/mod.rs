// Improved database infrastructure with better connection management and health monitoring
// Provides robust database connectivity with proper error handling and monitoring

pub mod schemas;

use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::config::DatabaseConfig;
use crate::shared::error::{AppResult, AppError, DatabaseError};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Database configuration with environment-specific settings

impl DatabaseConfig {
    /// Create database config from environment
    pub fn from_env() -> AppResult<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| AppError::Configuration("DATABASE_URL is required".to_string()))?;

        let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        // Environment-specific defaults
        let (max_connections, min_connections, enable_logging) = match env.as_str() {
            "production" => (20, 5, false),
            "testing" => (5, 1, false),
            _ => (10, 2, true), // development
        };

        Ok(Self {
            url: database_url,
            max_connections,
            min_connections,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
            enable_logging,
        })
    }
}

/// Database service with improved connection management
pub struct DatabaseService {
    pool: DbPool,
    config: DatabaseConfig,
}

impl DatabaseService {
    /// Create new database service with configuration
    pub async fn new(config: DatabaseConfig) -> AppResult<Self> {
        let manager = ConnectionManager::<PgConnection>::new(&config.url);

        let pool = Pool::builder()
            .max_size(config.max_connections)
            .min_idle(Some(config.min_connections))
            .connection_timeout(config.connection_timeout)
            .idle_timeout(Some(config.idle_timeout))
            .max_lifetime(Some(config.max_lifetime))
            .test_on_check_out(true)
            .build(manager)
            .map_err(|e| AppError::Database(DatabaseError::ConnectionPool(e)))?;

        // Test connection
        {
            let mut conn = pool.get()
                .map_err(|e| AppError::Database(DatabaseError::ConnectionPool(e)))?;

            // Simple health check query
            diesel::sql_query("SELECT 1")
                .execute(&mut conn)
                .map_err(|e| AppError::Database(DatabaseError::Diesel(e)))?;
        }

        Ok(Self { pool, config })
    }

    /// Get database connection from pool
    pub fn get_connection(&self) -> AppResult<DbConnection> {
        self.pool.get()
            .map_err(|e| AppError::Database(DatabaseError::ConnectionPool(e)))
    }

    /// Run database migrations
    pub fn run_migrations(&self) -> AppResult<()> {
        let mut conn = self.get_connection()?;

        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| AppError::Database(DatabaseError::Other(e.to_string())))?;

        Ok(())
    }

    /// Check database health
    pub async fn health_check(&self) -> AppResult<DatabaseHealth> {
        let start = std::time::Instant::now();

        let mut conn = self.get_connection()?;

        // Test basic connectivity
        let result = diesel::sql_query("SELECT version(), current_database(), current_user")
            .get_result::<DatabaseInfo>(&mut conn);

        let latency = start.elapsed();

        match result {
            Ok(info) => Ok(DatabaseHealth {
                status: HealthStatus::Healthy,
                latency_ms: latency.as_millis() as u64,
                active_connections: self.pool.state().connections,
                idle_connections: self.pool.state().idle_connections,
                database_info: Some(info),
                error: None,
            }),
            Err(e) => Ok(DatabaseHealth {
                status: HealthStatus::Unhealthy,
                latency_ms: latency.as_millis() as u64,
                active_connections: self.pool.state().connections,
                idle_connections: self.pool.state().idle_connections,
                database_info: None,
                error: Some(e.to_string()),
            }),
        }
    }

    /// Execute in transaction with automatic rollback on error
    pub async fn with_transaction<T, F>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&mut DbConnection) -> AppResult<T>,
    {
        let mut conn = self.get_connection()?;

        conn.transaction::<T, AppError, _>(|conn| {
            f(conn)
        })

    }

    /// Get pool statistics
    pub fn pool_stats(&self) -> PoolStats {
        let state = self.pool.state();
        PoolStats {
            connections: state.connections,
            idle_connections: state.idle_connections,
            max_connections: self.config.max_connections,
            min_connections: self.config.min_connections,
        }
    }

    /// Get configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Get database connection pool
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}

/// Database health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Database health information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseHealth {
    pub status: HealthStatus,
    pub latency_ms: u64,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub database_info: Option<DatabaseInfo>,
    pub error: Option<String>,
}

/// Database information from health check
#[derive(Debug, Clone, QueryableByName, Serialize, Deserialize)]
pub struct DatabaseInfo {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub version: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub current_database: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub current_user: String,
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub connections: u32,
    pub idle_connections: u32,
    pub max_connections: u32,
    pub min_connections: u32,
}

/// Database utilities and helpers
pub mod utils {
    use super::*;

    /// Check if database is reachable
    pub async fn ping_database(database_url: &str) -> AppResult<bool> {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(1)
            .connection_timeout(Duration::from_secs(5))
            .build(manager)
            .map_err(|e| AppError::Database(DatabaseError::ConnectionPool(e)))?;

        let mut conn = pool.get()
            .map_err(|e| AppError::Database(DatabaseError::ConnectionPool(e)))?;

        diesel::sql_query("SELECT 1")
            .execute(&mut conn)
            .map_err(|e| AppError::Database(DatabaseError::Diesel(e)))?;

        Ok(true)
    }
}
