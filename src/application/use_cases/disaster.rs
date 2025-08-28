/// Disaster management use cases
/// Handles disaster reporting, status updates, and response coordination

use async_trait::async_trait;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::use_cases::{UseCase, ValidatedUseCase};
use crate::domain::entities::disaster::{Disaster, DisasterSeverity, DisasterStatus, DisasterType};
use crate::domain::value_objects::*;
use crate::domain::ports::repositories::DisasterRepository;
use crate::domain::ports::services::NotificationService;
use crate::domain::events::{DisasterReportedEvent, DisasterStatusUpdatedEvent, EventPublisher};
use crate::shared::{AppResult, AppError};

/// Request to report a new disaster
#[derive(Debug, Clone)]
pub struct ReportDisasterRequest {
    pub title: String,
    pub description: String,
    pub disaster_type: String,
    pub severity: DisasterSeverity,
    pub location: Coordinates,
    pub reported_by: UserId,
    pub contact_info: Option<PhoneNumber>,
    pub images: Vec<String>, // URLs or base64 encoded images
}

/// Response after reporting a disaster
#[derive(Debug, Clone)]
pub struct DisasterResponse {
    pub id: DisasterId,
    pub title: String,
    pub disaster_type: String,
    pub severity: DisasterSeverity,
    pub status: String,
    pub location: Coordinates,
    pub description: String,
    pub reported_by: UserId,
    pub reported_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Use case for reporting new disasters
pub struct ReportDisasterUseCase {
    disaster_repository: Arc<dyn DisasterRepository>,
    notification_service: Arc<dyn NotificationService>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl ReportDisasterUseCase {
    pub fn new(
        disaster_repository: Arc<dyn DisasterRepository>,
        notification_service: Arc<dyn NotificationService>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            disaster_repository,
            notification_service,
            event_publisher,
        }
    }
}

#[async_trait]
impl ValidatedUseCase<ReportDisasterRequest, DisasterResponse> for ReportDisasterUseCase {
    async fn validate(&self, request: &ReportDisasterRequest) -> AppResult<()> {
        if request.disaster_type.trim().is_empty() {
            return Err(AppError::Validation("Disaster type cannot be empty".to_string()));
        }

        if request.title.trim().is_empty() {
            return Err(AppError::Validation("Title cannot be empty".to_string()));
        }

        if request.description.trim().is_empty() {
            return Err(AppError::Validation("Description cannot be empty".to_string()));
        }

        if request.description.len() < 10 {
            return Err(AppError::Validation("Description must be at least 10 characters".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl UseCase<ReportDisasterRequest, DisasterResponse> for ReportDisasterUseCase {
    async fn execute(&self, request: ReportDisasterRequest) -> AppResult<DisasterResponse> {
        // Map type string to enum
        let dtype = match request.disaster_type.to_lowercase().as_str() {
            "earthquake" => DisasterType::Earthquake,
            "flood" => DisasterType::Flood,
            "tsunami" => DisasterType::Tsunami,
            "landslide" => DisasterType::Landslide,
            "volcanic_eruption" | "volcaniceruption" | "volcano" => DisasterType::VolcanicEruption,
            "fire" => DisasterType::Fire,
            "storm" => DisasterType::Storm,
            "drought" => DisasterType::Drought,
            "epidemic" => DisasterType::Epidemic,
            "technological" | "technological_disaster" => DisasterType::TechnologicalDisaster,
            other => DisasterType::Other(other.to_string()),
        };

        // Create disaster entity
        let disaster = Disaster::new(
            request.title.clone(),
            request.description.clone(),
            dtype,
            request.severity.clone(),
            request.location.clone(),
            request.reported_by.clone(),
        );

        // Save to repository
        let saved_disaster = self.disaster_repository.save(&disaster).await?;

        // Publish domain event
        let event = DisasterReportedEvent {
            event_id: Uuid::new_v4(),
            disaster_id: saved_disaster.id().clone(),
            reported_by: request.reported_by.clone(),
            disaster_type: request.disaster_type.clone(),
            severity: request.severity.clone(),
            location: request.location.clone(),
            description: request.description.clone(),
            occurred_at: saved_disaster.created_at(),
            version: saved_disaster.version() as u64,
        };

        self.event_publisher.publish(&event).await?;

        // Optional notification: notify about new disaster (use existing method)
        let _ = self.notification_service.notify_disaster_update(&saved_disaster).await;

        Ok(DisasterResponse {
            id: saved_disaster.id().clone(),
            title: saved_disaster.title().to_string(),
            disaster_type: format!("{:?}", saved_disaster.disaster_type()),
            severity: saved_disaster.severity().clone(),
            status: format!("{:?}", saved_disaster.status()),
            location: saved_disaster.location().clone(),
            description: saved_disaster.description().to_string(),
            reported_by: saved_disaster.reporter_id().clone(),
            reported_at: saved_disaster.created_at(),
            updated_at: saved_disaster.updated_at(),
        })
    }
}

/// Request to update disaster status
#[derive(Debug, Clone)]
pub struct UpdateDisasterStatusRequest {
    pub disaster_id: DisasterId,
    pub new_status: String,
    pub updated_by: UserId,
    pub update_notes: Option<String>,
}

/// Use case for updating disaster status
pub struct UpdateDisasterStatusUseCase {
    disaster_repository: Arc<dyn DisasterRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl UpdateDisasterStatusUseCase {
    pub fn new(
        disaster_repository: Arc<dyn DisasterRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            disaster_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl ValidatedUseCase<UpdateDisasterStatusRequest, DisasterResponse> for UpdateDisasterStatusUseCase {
    async fn validate(&self, request: &UpdateDisasterStatusRequest) -> AppResult<()> {
        let valid_statuses = ["reported", "verified", "responded", "resolved", "closed"];
        
        if !valid_statuses.contains(&request.new_status.to_lowercase().as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid status. Must be one of: {}",
                valid_statuses.join(", ")
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl UseCase<UpdateDisasterStatusRequest, DisasterResponse> for UpdateDisasterStatusUseCase {
    async fn execute(&self, request: UpdateDisasterStatusRequest) -> AppResult<DisasterResponse> {
        // Get existing disaster
        let mut disaster = self.disaster_repository
            .find_by_id(&request.disaster_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Disaster not found".to_string()))?;

        let old_status = format!("{:?}", disaster.status());

        // Parse new status
        let new_status_enum = match request.new_status.to_lowercase().as_str() {
            "reported" => DisasterStatus::Reported,
            "verified" => DisasterStatus::Verified,
            "responded" | "responding" => DisasterStatus::Responded,
            "resolved" => DisasterStatus::Resolved,
            "closed" => DisasterStatus::Closed,
            _ => return Err(AppError::Validation("Invalid status".to_string())),
        };

        // Update status
        disaster.update_status(new_status_enum, request.updated_by.clone())?;

        // Save updated disaster
        let saved_disaster = self.disaster_repository.update(&disaster).await?;

        // Publish domain event
        let event = DisasterStatusUpdatedEvent {
            event_id: Uuid::new_v4(),
            disaster_id: request.disaster_id.clone(),
            updated_by: request.updated_by.clone(),
            old_status,
            new_status: format!("{:?}", saved_disaster.status()),
            occurred_at: Utc::now(),
            version: saved_disaster.version() as u64,
        };

        self.event_publisher.publish(&event).await?;

        Ok(DisasterResponse {
            id: saved_disaster.id().clone(),
            title: saved_disaster.title().to_string(),
            disaster_type: format!("{:?}", saved_disaster.disaster_type()),
            severity: saved_disaster.severity().clone(),
            status: format!("{:?}", saved_disaster.status()),
            location: saved_disaster.location().clone(),
            description: saved_disaster.description().to_string(),
            reported_by: saved_disaster.reporter_id().clone(),
            reported_at: saved_disaster.created_at(),
            updated_at: saved_disaster.updated_at(),
        })
    }
}

/// Request to get disasters near a location
#[derive(Debug, Clone)]
pub struct GetNearbyDisastersRequest {
    pub location: Coordinates,
    pub radius_km: f64,
    pub status_filter: Option<Vec<String>>,
    pub severity_filter: Option<Vec<DisasterSeverity>>,
    pub limit: Option<u32>,
}

/// Use case for getting nearby disasters
pub struct GetNearbyDisastersUseCase {
    disaster_repository: Arc<dyn DisasterRepository>,
}

impl GetNearbyDisastersUseCase {
    pub fn new(disaster_repository: Arc<dyn DisasterRepository>) -> Self {
        Self { disaster_repository }
    }
}

#[async_trait]
impl ValidatedUseCase<GetNearbyDisastersRequest, Vec<DisasterResponse>> for GetNearbyDisastersUseCase {
    async fn validate(&self, request: &GetNearbyDisastersRequest) -> AppResult<()> {
        if request.radius_km <= 0.0 {
            return Err(AppError::Validation("Radius must be positive".to_string()));
        }

        if request.radius_km > 100.0 {
            return Err(AppError::Validation("Radius cannot exceed 100km".to_string()));
        }

        if let Some(limit) = request.limit {
            if limit == 0 || limit > 1000 {
                return Err(AppError::Validation("Limit must be between 1 and 1000".to_string()));
            }
        }

        Ok(())
    }
}

#[async_trait]
impl UseCase<GetNearbyDisastersRequest, Vec<DisasterResponse>> for GetNearbyDisastersUseCase {
    async fn execute(&self, request: GetNearbyDisastersRequest) -> AppResult<Vec<DisasterResponse>> {
        let disasters = self.disaster_repository
            .find_nearby(
                request.location.latitude,
                request.location.longitude,
                request.radius_km,
            )
            .await?;

        // Apply optional filters in application layer
        let mut filtered: Vec<Disaster> = disasters.into_iter().filter(|d| {
            let status_ok = if let Some(ref statuses) = request.status_filter {
                let s = format!("{:?}", d.status()).to_lowercase();
                statuses.iter().any(|x| x.to_lowercase() == s)
            } else { true };

            let severity_ok = if let Some(ref severities) = request.severity_filter {
                severities.contains(d.severity())
            } else { true };

            status_ok && severity_ok
        }).collect();

        if let Some(limit) = request.limit { filtered.truncate(limit as usize); }

        let responses = filtered
            .into_iter()
            .map(|disaster| DisasterResponse {
                id: disaster.id().clone(),
                title: disaster.title().to_string(),
                disaster_type: format!("{:?}", disaster.disaster_type()),
                severity: disaster.severity().clone(),
                status: format!("{:?}", disaster.status()),
                location: disaster.location().clone(),
                description: disaster.description().to_string(),
                reported_by: disaster.reporter_id().clone(),
                reported_at: disaster.created_at(),
                updated_at: disaster.updated_at(),
            })
            .collect();

        Ok(responses)
    }
}
