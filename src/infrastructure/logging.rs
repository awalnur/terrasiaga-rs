/// Infrastructure logging configuration
/// Handles structured logging and monitoring

use tracing::{info, warn, error};
use serde_json::Value;

pub struct LoggingService {
    service_name: String,
}

impl LoggingService {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }

    pub fn log_request(&self, method: &str, path: &str, status: u16, duration_ms: u64) {
        info!(
            service = %self.service_name,
            method = %method,
            path = %path,
            status = %status,
            duration_ms = %duration_ms,
            "HTTP request processed"
        );
    }

    pub fn log_error(&self, error: &str, context: Option<Value>) {
        error!(
            service = %self.service_name,
            error = %error,
            context = ?context,
            "Application error occurred"
        );
    }

    pub fn log_warning(&self, message: &str, context: Option<Value>) {
        warn!(
            service = %self.service_name,
            message = %message,
            context = ?context,
            "Warning condition detected"
        );
    }
}
