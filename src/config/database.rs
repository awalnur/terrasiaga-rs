/// Database configuration for Terra Siaga
/// Handles database connection settings and options
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub enable_logging: bool,
}

impl DatabaseConfig {
    /// Create a new database configuration from a connection URL
    pub fn new(url: String) -> Self {
        Self {
            url,
            max_connections: 10,
            enable_logging: false,
        }
    }
    
    /// Create a database configuration with default development settings
    pub fn development() -> Self {
        Self {
            url: String::from("postgresql://localhost/terra_siaga"),
            max_connections: 5,
            enable_logging: true,
        }
    }
    
    /// Create a database configuration with production settings
    pub fn production(url: String) -> Self {
        Self {
            url,
            max_connections: 20,
            enable_logging: false,
        }
    }
}