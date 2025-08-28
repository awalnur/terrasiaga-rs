/// Disaster domain entity
/// Represents a disaster/emergency event with all business rules and state management

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::shared::{DisasterId, UserId, LocationId, AppResult, AppError, AuditFields, Priority};
use crate::domain::value_objects::Coordinates;
use crate::shared::types::SeverityLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disaster {
    pub id: DisasterId,
    pub title: String,
    pub description: String,
    pub disaster_type: DisasterType,
    pub severity: DisasterSeverity,
    pub status: DisasterStatus,
    pub priority: Priority,
    pub location: Coordinates,
    pub reporter_id: UserId,
    pub assigned_responders: Vec<UserId>,
    pub affected_population: Option<u32>,
    pub estimated_damage: Option<EstimatedDamage>,
    pub resources_needed: Vec<ResourceNeed>,
    pub timeline: DisasterTimeline,
    pub verification: VerificationInfo,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
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
    Critical = 5,
    Catastrophic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DisasterStatus {
    Reported,
    Verified,
    Responded,
    Resolved,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedDamage {
    pub economic_loss: Option<f64>,
    pub casualties: u32,
    pub injuries: u32,
    pub displaced_people: u32,
    pub damaged_buildings: u32,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNeed {
    pub resource_type: String,
    pub quantity: u32,
    pub urgency: Priority,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisasterTimeline {
    pub reported_at: DateTime<Utc>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,
    pub first_response_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationInfo {
    pub is_verified: bool,
    pub verified_by: Option<UserId>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verification_method: Option<String>,
    pub confidence_score: Option<f32>,
}

impl Disaster {
    /// Create a new disaster report
    pub fn new(
        title: String,
        description: String,
        disaster_type: DisasterType,
        severity: DisasterSeverity,
        location: Coordinates,
        reporter_id: UserId,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DisasterId::new(),
            title,
            description,
            disaster_type,
            severity: severity.clone(),
            status: DisasterStatus::Reported,
            priority: Self::calculate_priority(&severity),
            location,
            reporter_id,
            assigned_responders: Vec::new(),
            affected_population: None,
            estimated_damage: None,
            resources_needed: Vec::new(),
            timeline: DisasterTimeline {
                reported_at: now,
                occurred_at: None,
                verified_at: None,
                first_response_at: None,
                resolved_at: None,
                closed_at: None,
            },
            verification: VerificationInfo {
                is_verified: false,
                verified_by: None,
                verified_at: None,
                verification_method: None,
                confidence_score: None,
            },
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    // Getter methods for clean architecture compliance
    pub fn id(&self) -> &DisasterId { &self.id }
    pub fn title(&self) -> &str { &self.title }
    pub fn description(&self) -> &str { &self.description }
    pub fn disaster_type(&self) -> &DisasterType { &self.disaster_type }
    pub fn severity(&self) -> &DisasterSeverity { &self.severity }
    pub fn status(&self) -> &DisasterStatus { &self.status }
    pub fn priority(&self) -> &Priority { &self.priority }
    pub fn location(&self) -> &Coordinates { &self.location }
    pub fn reporter_id(&self) -> &UserId { &self.reporter_id }
    pub fn assigned_responders(&self) -> &[UserId] { &self.assigned_responders }
    pub fn affected_population(&self) -> Option<u32> { self.affected_population }
    pub fn timeline(&self) -> &DisasterTimeline { &self.timeline }
    pub fn verification(&self) -> &VerificationInfo { &self.verification }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
    pub fn version(&self) -> i64 { self.version }

    /// Calculate priority based on severity
    fn calculate_priority(severity: &DisasterSeverity) -> Priority {
        match severity {
            DisasterSeverity::Minor => Priority::Low,
            DisasterSeverity::Moderate => Priority::Normal,
            DisasterSeverity::Major => Priority::High,
            DisasterSeverity::Severe => Priority::Critical,
            DisasterSeverity::Critical => Priority::Emergency,
            DisasterSeverity::Catastrophic => Priority::Emergency,
        }
    }

    /// Update disaster severity with business rules
    pub fn update_severity(&mut self, new_severity: DisasterSeverity, updater_id: UserId) -> AppResult<()> {
        // Business rule: severity can only be escalated, not downgraded
        if new_severity < self.severity {
            return Err(AppError::BusinessRuleViolation(
                "Disaster severity can only be escalated, not downgraded".to_string()
            ));
        }

        self.severity = new_severity;
        self.priority = Self::calculate_priority(&self.severity);
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// Update disaster status
    pub fn update_status(&mut self, new_status: DisasterStatus, updater_id: UserId) -> AppResult<()> {
        // Validate status transition
        self.validate_status_transition(&new_status)?;

        let now = Utc::now();
        
        // Update timeline based on status
        match new_status {
            DisasterStatus::Verified => {
                if !self.verification.is_verified {
                    self.verification.is_verified = true;
                    self.verification.verified_by = Some(updater_id);
                    self.verification.verified_at = Some(now);
                    self.timeline.verified_at = Some(now);
                }
            },
            DisasterStatus::Responded => {
                if self.timeline.first_response_at.is_none() {
                    self.timeline.first_response_at = Some(now);
                }
            },
            DisasterStatus::Resolved => {
                self.timeline.resolved_at = Some(now);
            },
            DisasterStatus::Closed => {
                self.timeline.closed_at = Some(now);
            },
            _ => {}
        }

        self.status = new_status;
        self.updated_at = now;
        self.version += 1;
        Ok(())
    }

    /// Validate status transition business rules
    fn validate_status_transition(&self, new_status: &DisasterStatus) -> AppResult<()> {
        use DisasterStatus::*;
        
        let valid = match (&self.status, new_status) {
            (Reported, Verified) => true,
            (Reported, Responded) => true, // Can skip verification in emergencies
            (Verified, Responded) => true,
            (Responded, Resolved) => true,
            (Resolved, Closed) => true,
            (current, new) if current == new => true, // Same status is ok
            _ => false,
        };

        if !valid {
            return Err(AppError::BusinessRuleViolation(
                format!("Invalid status transition from {:?} to {:?}", self.status, new_status)
            ));
        }

        Ok(())
    }

    /// Assign responder to disaster
    pub fn assign_responder(&mut self, responder_id: UserId) -> AppResult<()> {
        if self.assigned_responders.contains(&responder_id) {
            return Err(AppError::BusinessRuleViolation(
                "Responder is already assigned to this disaster".to_string()
            ));
        }

        self.assigned_responders.push(responder_id);
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// Remove responder from disaster
    pub fn remove_responder(&mut self, responder_id: &UserId) -> AppResult<()> {
        let initial_len = self.assigned_responders.len();
        self.assigned_responders.retain(|id| id != responder_id);
        
        if self.assigned_responders.len() == initial_len {
            return Err(AppError::NotFound(
                "Responder not found in assigned responders".to_string()
            ));
        }

        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// Add resource need
    pub fn add_resource_need(&mut self, resource: ResourceNeed) -> AppResult<()> {
        self.resources_needed.push(resource);
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// Update estimated damage
    pub fn update_estimated_damage(&mut self, damage: EstimatedDamage) -> AppResult<()> {
        self.estimated_damage = Some(damage);
        self.updated_at = Utc::now();
        self.version += 1;
        Ok(())
    }

    /// Check if disaster is active (not resolved or closed)
    pub fn is_active(&self) -> bool {
        !matches!(self.status, DisasterStatus::Resolved | DisasterStatus::Closed)
    }

    /// Check if disaster requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(self.priority, Priority::Critical | Priority::Emergency) ||
        matches!(self.severity, DisasterSeverity::Severe | DisasterSeverity::Critical)
    }

    /// Get age of disaster in hours
    pub fn age_hours(&self) -> f64 {
        let now = Utc::now();
        (now - self.timeline.reported_at).num_seconds() as f64 / 3600.0
    }

    /// Check if disaster response is overdue
    pub fn is_response_overdue(&self) -> bool {
        if !matches!(self.status, DisasterStatus::Reported | DisasterStatus::Verified) {
            return false;
        }

        let response_threshold_hours = match self.severity {
            DisasterSeverity::Critical => 0.5, // 30 minutes
            DisasterSeverity::Severe => 1.0,   // 1 hour
            DisasterSeverity::Major => 4.0,    // 4 hours
            DisasterSeverity::Moderate => 12.0, // 12 hours
            DisasterSeverity::Minor => 24.0,   // 24 hours
            _ => 0.0,                      
        };

        self.age_hours() > response_threshold_hours
    }
}
