use std::env;
// filepath: /Users/development/RUST/terra-siaga/src/shared/types/mod.rs
/// Shared types used across the entire application
/// Common types, constants, and utilities following Clean Architecture principles

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fmt::{Display, Formatter};
use garde::Validate;

// ---------------------------------------------------------------------------
// Submodules (new structured layout)
// ---------------------------------------------------------------------------
pub mod errors;
pub mod results;
pub mod pagination;
pub mod dto;
pub mod common;

// ============================================================================
// CORE DOMAIN TYPES
// ============================================================================

/// Strong-typed ID wrapper for type safety
macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn as_uuid(&self) -> Uuid {
                self.0
            }

            // Backward-compat: allow calling id.value()
            pub fn value(&self) -> Uuid {
                self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

// Re-export standard Duration type for consistency
pub use std::time::Duration;

// Define all ID types
define_id!(UserId);
define_id!(DisasterId);
define_id!(LocationId);
define_id!(ReportId);
define_id!(NotificationId);
define_id!(VolunteerId);
define_id!(EmergencyResponseId);
define_id!(ResourceId);
define_id!(OrganizationId);
define_id!(EvacuationRouteId);
define_id!(ShelterLocationId);
define_id!(AlertId);
define_id!(SessionId);

// ============================================================================
// USER AND AUTHENTICATION TYPES
// ============================================================================


// TODO update this to use the new user roles
/// User role hierarchy with specific permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub enum UserRole {
    /// Regular user who can report disasters and receive alerts
    Reporter,
    /// Volunteer who can respond to reports and assist in emergency response
    Volunteer,
    /// Emergency coordinator who can manage responses and volunteers
    Coordinator,
    /// Organization administrator with elevated permissions
    OrgAdmin,
    /// System administrator with full access
    SystemAdmin,
    /// General admin role (alias for OrgAdmin for backwards compatibility)
    Admin,

    /// General Guest role with minimal permissions (not authenticated)
    Citizen,

    Responder,
    SuperAdmin,
}

impl UserRole {
    /// Get default permissions for this role
    pub fn default_permissions(&self) -> Vec<Permission> {
        match self {
            UserRole::Citizen => vec![
                Permission::ReadReports,
                Permission::ReadPublicData,
            ],
            UserRole::Reporter => vec![
                Permission::ReadReports,
                Permission::WriteReports,
                Permission::ReadAlerts,
                Permission::ReadPublicData,
            ],
            UserRole::Volunteer => vec![
                Permission::ReadReports,
                Permission::WriteReports,
                Permission::ReadAlerts,
                Permission::ReadPublicData,
                Permission::UpdateReports,
                Permission::ManageVolunteerResponse,
            ],
            UserRole::Coordinator => vec![
                Permission::ReadReports,
                Permission::WriteReports,
                Permission::ReadAlerts,
                Permission::ReadPublicData,
                Permission::UpdateReports,
                Permission::ManageVolunteerResponse,
                Permission::ManageEmergencyResponse,
                Permission::WriteArea,
            ],
            UserRole::OrgAdmin | UserRole::Admin => vec![
                Permission::ReadReports,
                Permission::WriteReports,
                Permission::ReadAlerts,
                Permission::ReadPublicData,
                Permission::UpdateReports,
                Permission::ManageVolunteerResponse,
                Permission::ManageEmergencyResponse,
                Permission::WriteAlerts,
                Permission::ManageUsers,
                Permission::ReadAnalytics,
            ],
            UserRole::SystemAdmin => Permission::all(),
            UserRole::Responder => vec![
                Permission::ReadReports,
                Permission::WriteReports,
                Permission::ReadAlerts,
                Permission::ReadPublicData,
                Permission::UpdateReports,
                Permission::ManageVolunteerResponse,
                Permission::ManageEmergencyResponse,
                Permission::WriteArea,
            ],
            UserRole::SuperAdmin => Permission::all(),

        }
    }


    pub fn has_permission(&self, permission: &Permission) -> bool {
        let default_permissions = self.default_permissions();
        default_permissions.contains(permission)
    }

    /// Check if this role has at least the given minimum role level
    pub fn has_minimum_level(&self, min_role: &UserRole) -> bool {
        let current_level = self.hierarchy_level();
        let min_level = min_role.hierarchy_level();
        current_level >= min_level
    }


    /// Get the hierarchy level of the role (higher number = more permissions)
    fn hierarchy_level(&self) -> u8 {
        match self {
            UserRole::Reporter => 1,
            UserRole::Volunteer => 2,
            UserRole::Coordinator => 3,
            UserRole::OrgAdmin | UserRole::Admin => 4,
            UserRole::SystemAdmin => 5,
            UserRole::Citizen => 6,
            UserRole::Responder => 7,
            UserRole::SuperAdmin => 8,
        }
    }
}

/// Granular permission system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Permission {
    // Report permissions
    ReadReports,
    WriteReports,
    DeleteReports,
    ReadOwnReports,
    UpdateReports,
    // Alert permissions
    ReadAlerts,
    WriteAlerts,
    DeleteAlerts,
    ManageAlerts,

    // Emergency response permissions
    ManageEmergencyResponse,
    UpdateEmergencyStatus,
    AssignEmergencyResponse,

    // Volunteer permissions
    ManageVolunteers,
    WriteVolunteerResponse,
    ReadVolunteerData,
    ManageVolunteerResponse,
    // User management permissions
    ManageUsers,
    ReadUserProfiles,
    UpdateUserProfiles,
    DeleteUsers,

    // Analytics permissions
    ReadAnalytics,
    WriteAnalytics,
    ReadAdvancedAnalytics,

    // Area permissions
    WriteArea,

    // Organization permissions
    ManageOrganization,
    ReadOrgData,
    WriteOrgData,

    // System permissions
    ManageSystem,
    ReadSystemData,
    WriteSystemData,
    ManageSystemConfig,

    // Data access levels
    ReadPublicData,
    ReadCoordinatorData,

    // Profile permissions
    ReadOwnProfile,
    UpdateOwnProfile,
    ReadAllProfiles,

    // Location permissions
    ManageLocations,
    ReadLocationData,
    UpdateLocationData,

    // Resource permissions
    ManageResources,
    ReadResourceData,
    UpdateResourceData,

    // Notification permissions
    SendNotifications,
    ManageNotificationTemplates,
    ReadNotificationHistory,
}

impl Permission {
    /// Convert permission to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::ReadReports => "reports:read",
            Permission::WriteReports => "reports:write",
            Permission::DeleteReports => "reports:delete",
            Permission::UpdateReports => "reports:update",
            Permission::ReadOwnReports => "reports:read_own",
            Permission::ReadAlerts => "alerts:read",
            Permission::WriteAlerts => "alerts:write",
            Permission::DeleteAlerts => "alerts:delete",
            Permission::ManageAlerts => "alerts:manage",
            Permission::ManageEmergencyResponse => "emergency:manage",
            Permission::UpdateEmergencyStatus => "emergency:update_status",
            Permission::AssignEmergencyResponse => "emergency:assign",
            Permission::ManageVolunteers => "volunteers:manage",
            Permission::WriteVolunteerResponse => "volunteers:respond",
            Permission::ReadVolunteerData => "volunteers:read",
            Permission::ManageUsers => "users:manage",
            Permission::ReadUserProfiles => "users:read_profiles",
            Permission::UpdateUserProfiles => "users:update_profiles",
            Permission::DeleteUsers => "users:delete",
            Permission::ReadAnalytics => "analytics:read",
            Permission::WriteAnalytics => "analytics:write",
            Permission::ReadAdvancedAnalytics => "analytics:read_advanced",
            Permission::ManageOrganization => "organization:manage",
            Permission::ReadOrgData => "organization:read",
            Permission::WriteOrgData => "organization:write",
            Permission::ManageSystem => "system:manage",
            Permission::ReadSystemData => "system:read",
            Permission::WriteSystemData => "system:write",
            Permission::ManageSystemConfig => "system:config",
            Permission::ReadPublicData => "data:read_public",
            Permission::ReadCoordinatorData => "data:read_coordinator",
            Permission::ReadOwnProfile => "profile:read_own",
            Permission::UpdateOwnProfile => "profile:update_own",
            Permission::ReadAllProfiles => "profile:read_all",
            Permission::ManageLocations => "locations:manage",
            Permission::ReadLocationData => "locations:read",
            Permission::UpdateLocationData => "locations:update",
            Permission::ManageResources => "resources:manage",
            Permission::ReadResourceData => "resources:read",
            Permission::UpdateResourceData => "resources:update",
            Permission::SendNotifications => "notifications:send",
            Permission::ManageNotificationTemplates => "notifications:manage_templates",
            Permission::ReadNotificationHistory => "notifications:read_history",
            Permission::WriteArea => "area:write",
            Permission::ManageVolunteerResponse => "volunteer:manage",
        }
    }

    /// Parse permission from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "reports:read" => Some(Permission::ReadReports),
            "reports:write" => Some(Permission::WriteReports),
            "reports:delete" => Some(Permission::DeleteReports),
            "reports:read_own" => Some(Permission::ReadOwnReports),
            "alerts:read" => Some(Permission::ReadAlerts),
            "alerts:write" => Some(Permission::WriteAlerts),
            "alerts:delete" => Some(Permission::DeleteAlerts),
            "alerts:manage" => Some(Permission::ManageAlerts),
            "emergency:manage" => Some(Permission::ManageEmergencyResponse),
            "emergency:update_status" => Some(Permission::UpdateEmergencyStatus),
            "emergency:assign" => Some(Permission::AssignEmergencyResponse),
            "volunteers:manage" => Some(Permission::ManageVolunteers),
            "volunteers:respond" => Some(Permission::WriteVolunteerResponse),
            "volunteers:read" => Some(Permission::ReadVolunteerData),
            "users:manage" => Some(Permission::ManageUsers),
            "users:read_profiles" => Some(Permission::ReadUserProfiles),
            "users:update_profiles" => Some(Permission::UpdateUserProfiles),
            "users:delete" => Some(Permission::DeleteUsers),
            "analytics:read" => Some(Permission::ReadAnalytics),
            "analytics:write" => Some(Permission::WriteAnalytics),
            "analytics:read_advanced" => Some(Permission::ReadAdvancedAnalytics),
            "organization:manage" => Some(Permission::ManageOrganization),
            "organization:read" => Some(Permission::ReadOrgData),
            "organization:write" => Some(Permission::WriteOrgData),
            "system:manage" => Some(Permission::ManageSystem),
            "system:read" => Some(Permission::ReadSystemData),
            "system:write" => Some(Permission::WriteSystemData),
            "system:config" => Some(Permission::ManageSystemConfig),
            "data:read_public" => Some(Permission::ReadPublicData),
            "data:read_coordinator" => Some(Permission::ReadCoordinatorData),
            "profile:read_own" => Some(Permission::ReadOwnProfile),
            "profile:update_own" => Some(Permission::UpdateOwnProfile),
            "profile:read_all" => Some(Permission::ReadAllProfiles),
            "locations:manage" => Some(Permission::ManageLocations),
            "locations:read" => Some(Permission::ReadLocationData),
            "locations:update" => Some(Permission::UpdateLocationData),
            "resources:manage" => Some(Permission::ManageResources),
            "resources:read" => Some(Permission::ReadResourceData),
            "resources:update" => Some(Permission::UpdateResourceData),
            "notifications:send" => Some(Permission::SendNotifications),
            "notifications:manage_templates" => Some(Permission::ManageNotificationTemplates),
            "notifications:read_history" => Some(Permission::ReadNotificationHistory),
            _ => None,
        }
    }

    /// Get all available permissions
    pub fn all() -> Vec<Self> {
        vec![
            Permission::ReadReports,
            Permission::WriteReports,
            Permission::DeleteReports,
            Permission::ReadOwnReports,
            Permission::ReadAlerts,
            Permission::WriteAlerts,
            Permission::DeleteAlerts,
            Permission::ManageAlerts,
            Permission::ManageEmergencyResponse,
            Permission::UpdateEmergencyStatus,
            Permission::AssignEmergencyResponse,
            Permission::ManageVolunteers,
            Permission::WriteVolunteerResponse,
            Permission::ReadVolunteerData,
            Permission::ManageUsers,
            Permission::ReadUserProfiles,
            Permission::UpdateUserProfiles,
            Permission::DeleteUsers,
            Permission::ReadAnalytics,
            Permission::WriteAnalytics,
            Permission::ReadAdvancedAnalytics,
            Permission::ManageOrganization,
            Permission::ReadOrgData,
            Permission::WriteOrgData,
            Permission::ManageSystem,
            Permission::ReadSystemData,
            Permission::WriteSystemData,
            Permission::ManageSystemConfig,
            Permission::ReadPublicData,
            Permission::ReadCoordinatorData,
            Permission::ReadOwnProfile,
            Permission::UpdateOwnProfile,
            Permission::ReadAllProfiles,
            Permission::ManageLocations,
            Permission::ReadLocationData,
            Permission::UpdateLocationData,
            Permission::ManageResources,
            Permission::ReadResourceData,
            Permission::UpdateResourceData,
            Permission::SendNotifications,
            Permission::ManageNotificationTemplates,
            Permission::ReadNotificationHistory,
        ]
    }
}

// ============================================================================
// GEOGRAPHIC AND LOCATION TYPES
// ============================================================================

/// Geographic bounds for defining rectangular areas
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeoBounds {
    pub north_east: Coordinates,
    pub south_west: Coordinates,
}

impl GeoBounds {
    pub fn new(north_east: Coordinates, south_west: Coordinates) -> Result<Self, &'static str> {
        if north_east.latitude < south_west.latitude || north_east.longitude < south_west.longitude {
            return Err("Invalid bounds: north_east must be greater than south_west");
        }
        Ok(Self { north_east, south_west })
    }

    pub fn contains(&self, point: &Coordinates) -> bool {
        point.latitude >= self.south_west.latitude
            && point.latitude <= self.north_east.latitude
            && point.longitude >= self.south_west.longitude
            && point.longitude <= self.north_east.longitude
    }

    pub fn center(&self) -> Coordinates {
        Coordinates {
            latitude: (self.north_east.latitude + self.south_west.latitude) / 2.0,
            longitude: (self.north_east.longitude + self.south_west.longitude) / 2.0,
            altitude: None,
        }
    }
}

/// Location information from geocoding
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationInfo {
    pub coordinates: Coordinates,
    pub address: Option<String>,
    pub administrative: Option<String>,
    pub landmark: Option<String>,
    pub accuracy_radius: Option<f64>,
}

/// Priority levels for various operations
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
    Emergency = 5,
}

impl Display for Priority {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "Low"),
            Priority::Normal => write!(f, "Normal"),
            Priority::High => write!(f, "High"),
            Priority::Critical => write!(f, "Critical"),
            Priority::Emergency => write!(f, "Emergency"),
        }
    }
}

/// Audit fields for tracking entity changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFields {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<UserId>,
    pub updated_by: Option<UserId>,
    pub version: u64,
}

impl AuditFields {
    pub fn new(created_by: Option<UserId>) -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
            version: 1,
        }
    }

    pub fn update(&mut self, updated_by: Option<UserId>) {
        self.updated_at = Utc::now();
        self.updated_by = updated_by;
        self.version += 1;
    }
}

/// Pagination parameters (alias for backward compatibility)
pub type PaginationParams = Pagination;

/// Paginated response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: u64, limit: u32, offset: u32) -> Self {
        Self {
            data,
            pagination: PaginationMeta {
                total,
                limit,
                offset,
                page: offset / limit + 1,
                total_pages: (total as f64 / limit as f64).ceil() as u32,
            },
        }
    }
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
    pub page: u32,
    pub total_pages: u32,
}

/// Geographic coordinates with validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct Coordinates {
    #[garde(range(min = -90.0, max = 90.0))]
    pub latitude: f64,
    #[garde(range(min = -180.0, max = 180.0))]
    pub longitude: f64,
    #[garde(skip)]
    pub altitude: Option<f64>,
}

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, &'static str> {
        if latitude < -90.0 || latitude > 90.0 {
            return Err("Latitude must be between -90 and 90 degrees");
        }
        if longitude < -180.0 || longitude > 180.0 {
            return Err("Longitude must be between -180 and 180 degrees");
        }
        Ok(Self {
            latitude,
            longitude,
            altitude: None,
        })
    }

    pub fn with_altitude(mut self, altitude: f64) -> Self {
        self.altitude = Some(altitude);
        self
    }

    /// Calculate distance to another coordinate (in kilometers)
    pub fn distance_to(&self, other: &Coordinates) -> f64 {
        use std::f64::consts::PI;

        let lat1_rad = self.latitude * PI / 180.0;
        let lat2_rad = other.latitude * PI / 180.0;
        let delta_lat = (other.latitude - self.latitude) * PI / 180.0;
        let delta_lon = (other.longitude - self.longitude) * PI / 180.0;

        let a = (delta_lat / 2.0).sin().powi(2) +
                lat1_rad.cos() * lat2_rad.cos() *
                (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        6371.0 * c // Earth's radius in kilometers
    }
}

/// Address information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct Address {
    #[garde(length(min = 1, max = 255))]
    pub street: String,
    #[garde(length(min = 1, max = 100))]
    pub city: String,
    #[garde(length(min = 1, max = 100))]
    pub province: String,
    #[garde(length(min = 1, max = 100))]
    pub country: String,
    #[garde(length(min = 1, max = 20))]
    pub postal_code: String,
}

/// Complete location information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct Location {
    #[garde(skip)]
    pub coordinates: Coordinates,
    #[garde(skip)]
    pub address: Option<Address>,
    #[garde(length(min = 1, max = 255))]
    pub description: Option<String>,
}

// ============================================================================
// DISASTER AND EMERGENCY TYPES
// ============================================================================

/// Types of disasters that can be reported
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisasterType {
    Earthquake,
    Flood,
    Fire,
    Landslide,
    Tsunami,
    Volcano,
    Storm,
    Drought,
    Epidemic,
    TechnologicalDisaster,
    Other(String),
}




impl Display for DisasterType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DisasterType::Earthquake => write!(f, "Earthquake"),
            DisasterType::Flood => write!(f, "Flood"),
            DisasterType::Fire => write!(f, "Fire"),
            DisasterType::Landslide => write!(f, "Landslide"),
            DisasterType::Tsunami => write!(f, "Tsunami"),
            DisasterType::Volcano => write!(f, "Volcano"),
            DisasterType::Storm => write!(f, "Storm"),
            DisasterType::Drought => write!(f, "Drought"),
            DisasterType::Epidemic => write!(f, "Epidemic"),
            DisasterType::TechnologicalDisaster => write!(f, "Technological Disaster"),
            DisasterType::Other(desc) => write!(f, "Other: {}", desc),
        }
    }
}



/// Severity levels for disasters
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SeverityLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
    Extreme = 5,
}

impl Display for SeverityLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SeverityLevel::Low => write!(f, "Low"),
            SeverityLevel::Medium => write!(f, "Medium"),
            SeverityLevel::High => write!(f, "High"),
            SeverityLevel::Critical => write!(f, "Critical"),
            SeverityLevel::Extreme => write!(f, "Extreme"),
        }
    }
}

/// Status of emergency response
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmergencyStatus {
    Reported,
    Verified,
    Responding,
    Resolved,
    Closed,
}

impl Display for EmergencyStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EmergencyStatus::Reported => write!(f, "Reported"),
            EmergencyStatus::Verified => write!(f, "Verified"),
            EmergencyStatus::Responding => write!(f, "Responding"),
            EmergencyStatus::Resolved => write!(f, "Resolved"),
            EmergencyStatus::Closed => write!(f, "Closed"),
        }
    }
}

impl DisasterType {
    /// Baseline expected severity for each disaster type
    pub fn default_severity(&self) -> SeverityLevel {
        match self {
            DisasterType::Earthquake => SeverityLevel::High,
            DisasterType::Flood => SeverityLevel::High,
            DisasterType::Fire => SeverityLevel::High,
            DisasterType::Landslide => SeverityLevel::High,
            DisasterType::Tsunami => SeverityLevel::Critical,
            DisasterType::Volcano => SeverityLevel::High,
            DisasterType::Storm => SeverityLevel::Medium,
            DisasterType::Drought => SeverityLevel::Medium,
            DisasterType::Epidemic => SeverityLevel::High,
            DisasterType::TechnologicalDisaster => SeverityLevel::High,
            DisasterType::Other(_) => SeverityLevel::Medium,
        }
    }
}

// ============================================================================
// COMMON UTILITY TYPES
// ============================================================================

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Pagination {
    #[garde(range(min = 1, max = 1000))]
    pub limit: u32,
    #[garde(range(min = 0))]
    pub offset: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 20,
            offset: 0,
        }
    }
}

/// Sorting parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sorting {
    pub field: String,
    pub order: SortOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Time range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, &'static str> {
        if start >= end {
            return Err("Start time must be before end time");
        }
        Ok(Self { start, end })
    }

    pub fn contains(&self, timestamp: &DateTime<Utc>) -> bool {
        timestamp >= &self.start && timestamp <= &self.end
    }

    pub fn duration(&self) -> chrono::Duration {
        self.end - self.start
    }
}

// ============================================================================
// API RESPONSE TYPES
// ============================================================================

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub metadata: Option<ResponseMetadata>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: ResponseMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Response metadata for pagination and additional info
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub total_count: Option<u64>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub request_id: Option<String>,
    pub execution_time_ms: Option<u64>,
}

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

/// Email validation
pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.len() > 3 && email.len() < 255
}

/// Phone number validation (basic)
pub fn is_valid_phone(phone: &str) -> bool {
    phone.chars().all(|c| c.is_ascii_digit() || c == '+' || c == '-' || c == ' ')
        && phone.len() >= 8
        && phone.len() <= 20
}

// ============================================================================
// CONSTANTS
// ============================================================================

/// Application constants
pub mod constants {
    /// Maximum file upload size (10MB)
    pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

    /// Maximum number of items per page
    pub const MAX_PAGE_SIZE: u32 = 1000;

    /// Default pagination limit
    pub const DEFAULT_PAGE_SIZE: u32 = 20;

    /// Token expiration times
    pub const ACCESS_TOKEN_DURATION_MINUTES: i64 = 15;
    pub const REFRESH_TOKEN_DURATION_DAYS: i64 = 7;

    /// Rate limiting
    pub const DEFAULT_RATE_LIMIT_PER_MINUTE: u32 = 60;
    pub const AUTH_RATE_LIMIT_PER_MINUTE: u32 = 5;

    /// Geographic constants
    pub const EARTH_RADIUS_KM: f64 = 6371.0;
    pub const DEFAULT_SEARCH_RADIUS_KM: f64 = 10.0;

    /// Cache TTL defaults
    pub const DEFAULT_CACHE_TTL_SECONDS: u64 = 3600; // 1 hour
    pub const SHORT_CACHE_TTL_SECONDS: u64 = 300;    // 5 minutes
    pub const LONG_CACHE_TTL_SECONDS: u64 = 86400;   // 24 hours
}

pub fn parse_duration_seconds(key: &str, default: u64) -> std::time::Duration {
    Duration::from_secs(
        env::var(key)
            .unwrap_or_else(|_| default.to_string())
            .parse::<u64>()
            .unwrap_or(default)
    )
}