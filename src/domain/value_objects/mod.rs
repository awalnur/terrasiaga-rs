/// Domain value objects
/// Immutable objects that represent concepts with no identity, only values

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;
use regex::Regex;
use crate::shared::{AppResult, AppError};

/// Email value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> AppResult<Self> {
        let email = email.trim().to_lowercase();
        
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|_| AppError::Internal("Regex compilation failed".to_string()))?;
        
        if !email_regex.is_match(&email) {
            return Err(AppError::Validation("Invalid email format".to_string()));
        }
        
        if email.len() > 254 {
            return Err(AppError::Validation("Email too long".to_string()));
        }
        
        Ok(Email(email))
    }
    
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Phone number value object with international format support
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    pub fn new(phone: String) -> AppResult<Self> {
        let cleaned = phone.chars()
            .filter(|c| c.is_ascii_digit() || *c == '+')
            .collect::<String>();
        
        if cleaned.is_empty() {
            return Err(AppError::Validation("Phone number cannot be empty".to_string()));
        }
        
        // Indonesian phone number validation
        if cleaned.starts_with("+62") && cleaned.len() >= 10 && cleaned.len() <= 15 {
            return Ok(PhoneNumber(cleaned));
        }
        
        // International format
        if cleaned.starts_with('+') && cleaned.len() >= 8 && cleaned.len() <= 15 {
            return Ok(PhoneNumber(cleaned));
        }
        
        // Local format (assume Indonesian)
        if cleaned.starts_with('0') && cleaned.len() >= 9 && cleaned.len() <= 13 {
            let international = format!("+62{}", &cleaned[1..]);
            return Ok(PhoneNumber(international));
        }
        
        Err(AppError::Validation("Invalid phone number format".to_string()))
    }
    
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Geographic coordinates value object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> AppResult<Self> {
        if latitude < -90.0 || latitude > 90.0 {
            return Err(AppError::Validation("Latitude must be between -90 and 90".to_string()));
        }
        
        if longitude < -180.0 || longitude > 180.0 {
            return Err(AppError::Validation("Longitude must be between -180 and 180".to_string()));
        }
        
        Ok(Coordinates { latitude, longitude })
    }
    
    pub fn distance_to(&self, other: &Coordinates) -> f64 {
        use geo::Point;
        use geo::algorithm::haversine_distance::HaversineDistance;
        
        let point1 = Point::new(self.longitude, self.latitude);
        let point2 = Point::new(other.longitude, other.latitude);
        
        point1.haversine_distance(&point2)
    }
}

/// Address value object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub province: String,
    pub postal_code: Option<String>,
    pub country: String,
}

impl Address {
    pub fn new(
        street: String,
        city: String,
        province: String,
        postal_code: Option<String>,
        country: Option<String>,
    ) -> AppResult<Self> {
        if street.trim().is_empty() {
            return Err(AppError::Validation("Street cannot be empty".to_string()));
        }
        
        if city.trim().is_empty() {
            return Err(AppError::Validation("City cannot be empty".to_string()));
        }
        
        if province.trim().is_empty() {
            return Err(AppError::Validation("Province cannot be empty".to_string()));
        }
        
        Ok(Address {
            street: street.trim().to_string(),
            city: city.trim().to_string(),
            province: province.trim().to_string(),
            postal_code,
            country: country.unwrap_or_else(|| "Indonesia".to_string()),
        })
    }
    
    pub fn full_address(&self) -> String {
        let mut parts = vec![&self.street, &self.city, &self.province];
        
        if let Some(postal) = &self.postal_code {
            parts.push(postal);
        }
        
        parts.push(&self.country);
        parts.join(", ")
    }
}

/// Disaster severity level
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisasterSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl DisasterSeverity {
    pub fn priority_level(&self) -> u8 {
        match self {
            DisasterSeverity::Low => 1,
            DisasterSeverity::Medium => 2,
            DisasterSeverity::High => 3,
            DisasterSeverity::Critical => 4,
        }
    }
}

/// User role value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserRole {
    Citizen,
    Volunteer,
    Responder,
    Admin,
    SuperAdmin,
}

impl UserRole {
    pub fn permissions(&self) -> Vec<&'static str> {
        match self {
            UserRole::Citizen => vec!["report_disaster", "view_disasters", "receive_notifications"],
            UserRole::Volunteer => vec!["report_disaster", "view_disasters", "receive_notifications", "volunteer_response"],
            UserRole::Responder => vec!["report_disaster", "view_disasters", "receive_notifications", "respond_to_disaster", "update_disaster_status"],
            UserRole::Admin => vec!["report_disaster", "view_disasters", "receive_notifications", "respond_to_disaster", "update_disaster_status", "manage_users", "manage_disasters"],
            UserRole::SuperAdmin => vec!["all_permissions"],
        }
    }
    
    pub fn can_perform(&self, action: &str) -> bool {
        match self {
            UserRole::SuperAdmin => true,
            _ => self.permissions().contains(&action),
        }
    }
}

/// Entity ID wrapper for type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DisasterId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocationId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NotificationId(pub Uuid);

impl UserId {
    pub fn new() -> Self {
        UserId(Uuid::new_v4())
    }
    
    pub fn from_uuid(id: Uuid) -> Self {
        UserId(id)
    }
    
    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl DisasterId {
    pub fn new() -> Self {
        DisasterId(Uuid::new_v4())
    }
    
    pub fn from_uuid(id: Uuid) -> Self {
        DisasterId(id)
    }
    
    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl LocationId {
    pub fn new() -> Self {
        LocationId(Uuid::new_v4())
    }
    
    pub fn from_uuid(id: Uuid) -> Self {
        LocationId(id)
    }
    
    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl NotificationId {
    pub fn new() -> Self {
        NotificationId(Uuid::new_v4())
    }
    
    pub fn from_uuid(id: Uuid) -> Self {
        NotificationId(id)
    }
    
    pub fn value(&self) -> Uuid {
        self.0
    }
}

// Re-export commonly used value objects
pub use self::{
    Email, PhoneNumber, Coordinates, Address, DisasterSeverity, UserRole,
    UserId, DisasterId, LocationId, NotificationId
};
