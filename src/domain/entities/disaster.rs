/// Disaster domain entity
/// Represents a disaster/emergency event with all business rules and state management

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::shared::{DisasterId, UserId, LocationId, AppResult, AppError, AuditFields, Priority};
use crate::domain::value_objects::Coordinates;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disaster {
    pub id: DisasterId,
    pub title: String,
    pub description: String,
    pub disaster_type: DisasterType,
    pub severity: DisasterSeverity,
    pub status: DisasterStatus,
    pub priority: Priority,
    pub location_id: Option<LocationId>,
    pub coordinates: Option<Coordinates>,
    pub reporter_id: UserId,
    pub assigned_responders: Vec<UserId>,
    pub affected_population: Option<u32>,
    pub estimated_damage: Option<EstimatedDamage>,
    pub resources_needed: Vec<ResourceNeed>,
    pub timeline: DisasterTimeline,
    pub verification: VerificationInfo,
    pub audit: AuditFields,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisasterType {
    Earthquake,
    Flood,
    Tsunami,
    Landslide,
    VolcanicEruption,
    Fire,
    Storm,
    Drought,
    Epidemic,
    TechnologicalDisaster,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum DisasterSeverity {
    Minor = 1,
    Moderate = 2,
    Major = 3,
    Severe = 4,
    Catastrophic = 5,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisasterStatus {
    Reported,
    Verified,
    InProgress,
    Contained,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedDamage {
    pub economic_loss: Option<u64>, // In Indonesian Rupiah
    pub infrastructure_damage: Option<String>,
    pub environmental_impact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNeed {
    pub resource_type: String,
    pub quantity: u32,
    pub priority: Priority,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisasterTimeline {
    pub reported_at: DateTime<Utc>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,
    pub response_started_at: Option<DateTime<Utc>>,
    pub contained_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationInfo {
    pub is_verified: bool,
    pub verified_by: Option<UserId>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verification_notes: Option<String>,
    pub confidence_score: Option<f32>, // 0.0 to 1.0
}

impl Disaster {
    /// Create a new disaster report
    pub fn new(
        title: String,
        description: String,
        disaster_type: DisasterType,
        severity: DisasterSeverity,
        reporter_id: UserId,
        coordinates: Option<Coordinates>,
    ) -> AppResult<Self> {
        // Business rules validation
        if title.trim().is_empty() {
            return Err(AppError::Validation("Disaster title cannot be empty".to_string()));
        }

        if description.trim().is_empty() {
            return Err(AppError::Validation("Disaster description cannot be empty".to_string()));
        }

        let now = Utc::now();
        let priority = Self::calculate_initial_priority(&disaster_type, &severity);

        Ok(Self {
            id: DisasterId(Uuid::new_v4()),
            title: title.trim().to_string(),
            description: description.trim().to_string(),
            disaster_type,
            severity,
            status: DisasterStatus::Reported,
            priority,
            location_id: None,
            coordinates,
            reporter_id,
            assigned_responders: Vec::new(),
            affected_population: None,
            estimated_damage: None,
            resources_needed: Vec::new(),
            timeline: DisasterTimeline {
                reported_at: now,
                occurred_at: None,
                verified_at: None,
                response_started_at: None,
                contained_at: None,
                resolved_at: None,
            },
            verification: VerificationInfo {
                is_verified: false,
                verified_by: None,
                verified_at: None,
                verification_notes: None,
                confidence_score: None,
            },
            audit: AuditFields {
                created_at: now,
                updated_at: now,
                created_by: Some(reporter_id),
                updated_by: Some(reporter_id),
            },
        })
    }

    /// Verify the disaster report
    pub fn verify(&mut self, verifier_id: UserId, notes: Option<String>) -> AppResult<()> {
        match self.status {
            DisasterStatus::Reported => {
                self.status = DisasterStatus::Verified;
                self.verification.is_verified = true;
                self.verification.verified_by = Some(verifier_id);
                self.verification.verified_at = Some(Utc::now());
                self.verification.verification_notes = notes;
                self.timeline.verified_at = Some(Utc::now());
                self.audit.updated_at = Utc::now();
                self.audit.updated_by = Some(verifier_id);
                Ok(())
            }
            _ => Err(AppError::BusinessRuleViolation(
                "Only reported disasters can be verified".to_string()
            )),
        }
    }

    /// Start response to the disaster
    pub fn start_response(&mut self, responder_ids: Vec<UserId>) -> AppResult<()> {
        match self.status {
            DisasterStatus::Verified => {
                self.status = DisasterStatus::InProgress;
                self.assigned_responders = responder_ids;
                self.timeline.response_started_at = Some(Utc::now());
                self.audit.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(AppError::BusinessRuleViolation(
                "Only verified disasters can have response started".to_string()
            )),
        }
    }

    /// Mark disaster as contained
    pub fn contain(&mut self, updater_id: UserId) -> AppResult<()> {
        match self.status {
            DisasterStatus::InProgress => {
                self.status = DisasterStatus::Contained;
                self.timeline.contained_at = Some(Utc::now());
                self.audit.updated_at = Utc::now();
                self.audit.updated_by = Some(updater_id);
                Ok(())
            }
            _ => Err(AppError::BusinessRuleViolation(
                "Only in-progress disasters can be contained".to_string()
            )),
        }
    }

    /// Mark disaster as resolved
    pub fn resolve(&mut self, updater_id: UserId) -> AppResult<()> {
        match self.status {
            DisasterStatus::Contained => {
                self.status = DisasterStatus::Resolved;
                self.timeline.resolved_at = Some(Utc::now());
                self.audit.updated_at = Utc::now();
                self.audit.updated_by = Some(updater_id);
                Ok(())
            }
            _ => Err(AppError::BusinessRuleViolation(
                "Only contained disasters can be resolved".to_string()
            )),
        }
    }

    /// Add resource needs
    pub fn add_resource_need(&mut self, resource_need: ResourceNeed) -> AppResult<()> {
        // Business rule: Can only add resources for active disasters
        match self.status {
            DisasterStatus::Verified | DisasterStatus::InProgress | DisasterStatus::Contained => {
                self.resources_needed.push(resource_need);
                self.audit.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(AppError::BusinessRuleViolation(
                "Cannot add resources to inactive disasters".to_string()
            )),
        }
    }

    /// Update severity and recalculate priority
    pub fn update_severity(&mut self, new_severity: DisasterSeverity, updater_id: UserId) -> AppResult<()> {
        self.severity = new_severity;
        self.priority = Self::calculate_initial_priority(&self.disaster_type, &self.severity);
        self.audit.updated_at = Utc::now();
        self.audit.updated_by = Some(updater_id);
        Ok(())
    }

    /// Check if disaster is active (can be responded to)
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            DisasterStatus::Verified | DisasterStatus::InProgress | DisasterStatus::Contained
        )
    }

    /// Calculate response time in minutes
    pub fn response_time_minutes(&self) -> Option<i64> {
        if let (Some(response_started), Some(verified_at)) = (self.timeline.response_started_at, self.timeline.verified_at) {
            Some((response_started - verified_at).num_minutes())
        } else {
            None
        }
    }

    /// Calculate initial priority based on type and severity
    fn calculate_initial_priority(disaster_type: &DisasterType, severity: &DisasterSeverity) -> Priority {
        let base_priority = match severity {
            DisasterSeverity::Minor => Priority::Low,
            DisasterSeverity::Moderate => Priority::Medium,
            DisasterSeverity::Major => Priority::High,
            DisasterSeverity::Severe => Priority::Critical,
            DisasterSeverity::Catastrophic => Priority::Emergency,
        };

        // Adjust priority based on disaster type
        match disaster_type {
            DisasterType::Tsunami | DisasterType::VolcanicEruption | DisasterType::Earthquake => {
                // These types are inherently more critical
                match base_priority {
                    Priority::Low => Priority::Medium,
                    Priority::Medium => Priority::High,
                    Priority::High => Priority::Critical,
                    priority => priority,
                }
            }
            _ => base_priority,
        }
    }
}
