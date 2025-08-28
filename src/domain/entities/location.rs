/// Location domain entity
/// Represents a geographic location with business rules and validation

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::shared::{LocationId, AppResult, AppError, AuditFields};
use crate::domain::value_objects::{Coordinates, Address};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub coordinates: Coordinates,
    pub address: Address,
    pub location_type: LocationType,
    pub administrative_level: AdministrativeLevel,
    pub population: Option<u32>,
    pub area_km2: Option<f64>,
    pub elevation_meters: Option<f64>,
    pub is_disaster_prone: bool,
    pub risk_factors: Vec<RiskFactor>,
    pub emergency_contacts: Vec<EmergencyContact>,
    pub audit: AuditFields,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LocationType {
    City,
    Village,
    District,
    Province,
    EvacuationCenter,
    Hospital,
    School,
    FireStation,
    PoliceStation,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdministrativeLevel {
    Province,      // Provinsi
    Regency,       // Kabupaten/Kota
    District,      // Kecamatan
    Village,       // Desa/Kelurahan
    Hamlet,        // Dusun/RW
    Neighborhood,  // RT
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub risk_type: String,
    pub severity_level: u8, // 1-5
    pub description: String,
    pub last_assessed: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyContact {
    pub contact_type: String, // Police, Fire, Medical, etc.
    pub name: String,
    pub phone: String,
    pub is_primary: bool,
}

impl Location {
    /// Create a new location with validation
    pub fn new(
        name: String,
        coordinates: Coordinates,
        address: Address,
        location_type: LocationType,
        administrative_level: AdministrativeLevel,
    ) -> AppResult<Self> {
        // Business rules validation
        if name.trim().is_empty() {
            return Err(AppError::Validation("Location name cannot be empty".to_string()));
        }

        // Validate coordinates are within Indonesia bounds
        if !coordinates.is_within_indonesia() {
            return Err(AppError::Validation("Coordinates must be within Indonesia".to_string()));
        }

        let now = Utc::now();

        Ok(Self {
            id: LocationId(uuid::Uuid::new_v4()),
            name: name.trim().to_string(),
            coordinates,
            address,
            location_type,
            administrative_level,
            population: None,
            area_km2: None,
            elevation_meters: None,
            is_disaster_prone: false,
            risk_factors: Vec::new(),
            emergency_contacts: Vec::new(),
            audit: AuditFields {
                created_at: now,
                updated_at: now,
                created_by: None,
                updated_by: None,
                version: 0,
            },
        })
    }

    /// Add a risk factor to the location
    pub fn add_risk_factor(&mut self, risk_factor: RiskFactor) -> AppResult<()> {
        // Business rule: Severity level must be 1-5
        if risk_factor.severity_level < 1 || risk_factor.severity_level > 5 {
            return Err(AppError::Validation("Risk severity must be between 1 and 5".to_string()));
        }

        self.risk_factors.push(risk_factor);
        
        // Update disaster prone status based on high-severity risks
        self.is_disaster_prone = self.risk_factors.iter()
            .any(|rf| rf.severity_level >= 4);
        
        self.audit.updated_at = Utc::now();
        Ok(())
    }

    /// Add emergency contact
    pub fn add_emergency_contact(&mut self, contact: EmergencyContact) -> AppResult<()> {
        // Business rule: Only one primary contact per type
        if contact.is_primary {
            if let Some(existing) = self.emergency_contacts.iter_mut()
                .find(|c| c.contact_type == contact.contact_type && c.is_primary) {
                existing.is_primary = false;
            }
        }

        self.emergency_contacts.push(contact);
        self.audit.updated_at = Utc::now();
        Ok(())
    }

    /// Calculate distance to another location
    pub fn distance_to(&self, other: &Location) -> f64 {
        self.coordinates.distance_to(&other.coordinates)
    }

    /// Get the highest risk level for this location
    pub fn max_risk_level(&self) -> Option<u8> {
        self.risk_factors.iter()
            .map(|rf| rf.severity_level)
            .max()
    }

    /// Check if location is suitable for evacuation center
    pub fn is_suitable_for_evacuation(&self) -> bool {
        matches!(self.location_type, 
            LocationType::School | 
            LocationType::EvacuationCenter
        ) && self.max_risk_level().unwrap_or(0) <= 2
    }

    /// Get emergency contacts by type
    pub fn get_emergency_contacts_by_type(&self, contact_type: &str) -> Vec<&EmergencyContact> {
        self.emergency_contacts.iter()
            .filter(|c| c.contact_type == contact_type)
            .collect()
    }
}
