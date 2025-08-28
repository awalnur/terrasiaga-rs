/// Emergency response use cases
/// Handles emergency team dispatch, resource allocation, and response coordination

use async_trait::async_trait;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::use_cases::{UseCase, ValidatedUseCase};
use crate::domain::entities::disaster::DisasterSeverity;
use crate::shared::types::Coordinates as SCoordinates;
use crate::shared::Permission;
use crate::domain::value_objects::*;
use crate::domain::ports::repositories::{DisasterRepository, UserRepository};
use crate::domain::ports::services::{NotificationService, GeolocationService};
use crate::domain::events::{EmergencyResponseDispatchedEvent, EventPublisher};
use crate::shared::{AppResult, AppError};

/// Request to dispatch emergency response team
#[derive(Debug, Clone)]
pub struct DispatchEmergencyResponseRequest {
    pub disaster_id: DisasterId,
    pub dispatched_by: UserId,
    pub response_team_type: String, // "medical", "fire", "rescue", "police"
    pub priority_level: u8, // 1-5 scale
    pub estimated_personnel_needed: u32,
    pub special_equipment_needed: Vec<String>,
    pub notes: Option<String>,
}

/// Response after dispatching emergency team
#[derive(Debug, Clone)]
pub struct EmergencyResponseDispatchResponse {
    pub response_id: Uuid,
    pub disaster_id: DisasterId,
    pub team_type: String,
    pub estimated_arrival: DateTime<Utc>,
    pub personnel_count: u32,
    pub equipment_allocated: Vec<String>,
    pub status: String,
    pub dispatched_at: DateTime<Utc>,
}

/// Use case for dispatching emergency response teams
pub struct DispatchEmergencyResponseUseCase {
    disaster_repository: Arc<dyn DisasterRepository>,
    user_repository: Arc<dyn UserRepository>,
    notification_service: Arc<dyn NotificationService>,
    geo_service: Arc<dyn GeolocationService>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl DispatchEmergencyResponseUseCase {
    pub fn new(
        disaster_repository: Arc<dyn DisasterRepository>,
        user_repository: Arc<dyn UserRepository>,
        notification_service: Arc<dyn NotificationService>,
        geo_service: Arc<dyn GeolocationService>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            disaster_repository,
            user_repository,
            notification_service,
            geo_service,
            event_publisher,
        }
    }

    /// Calculate estimated arrival time based on distance and traffic
    async fn calculate_arrival_time(
        &self,
        from_location: &SCoordinates,
        to_location: &SCoordinates,
        team_type: &str,
    ) -> AppResult<DateTime<Utc>> {
        let distance_km = from_location.distance_to(to_location);

        // Base speed in km/h based on team type
        let base_speed = match team_type {
            "medical" => 80.0, // Ambulance with emergency lights
            "fire" => 70.0,    // Fire truck
            "rescue" => 60.0,  // Rescue vehicle
            "police" => 90.0,  // Police patrol
            _ => 50.0,         // Default emergency vehicle
        };

        // Add traffic factor (simplified - in real implementation use traffic API)
        let traffic_factor = 1.3; // 30% slower due to traffic
        let effective_speed = base_speed / traffic_factor; // km/h

        // distance_km is already in kilometers
        let travel_time_hours = distance_km / effective_speed;
        let travel_time_minutes = (travel_time_hours * 60.0) as i64;
        
        // Add preparation time (getting ready, equipment check)
        let preparation_time = match team_type {
            "medical" => 5,  // 5 minutes
            "fire" => 8,     // 8 minutes
            "rescue" => 12,  // 12 minutes
            "police" => 3,   // 3 minutes
            _ => 10,
        };

        let total_minutes = travel_time_minutes + preparation_time;
        Ok(Utc::now() + chrono::Duration::minutes(total_minutes))
    }

    /// Find nearest available emergency station
    async fn find_nearest_station(&self, _location: &SCoordinates, _team_type: &str) -> AppResult<SCoordinates> {
        // This would query a database of emergency stations
        // For now, returning a mock coordinate for Jakarta Emergency Center
        SCoordinates::new(-6.2088, 106.8456).map_err(|e| AppError::Validation(e.to_string()))
    }

    /// Allocate resources based on disaster severity and type
    fn allocate_resources(&self, disaster_severity: &DisasterSeverity, team_type: &str, requested_equipment: &[String]) -> Vec<String> {
        let mut equipment = Vec::new();

        match team_type {
            "medical" => {
                equipment.extend_from_slice(&[
                    "Ambulance".to_string(),
                    "Medical Kit".to_string(),
                    "Stretcher".to_string(),
                ]);
                
                if matches!(disaster_severity, DisasterSeverity::Severe | DisasterSeverity::Critical) {
                    equipment.extend_from_slice(&[
                        "Advanced Life Support".to_string(),
                        "Trauma Kit".to_string(),
                        "Oxygen Tank".to_string(),
                    ]);
                }
            },
            "fire" => {
                equipment.extend_from_slice(&[
                    "Fire Truck".to_string(),
                    "Water Tank".to_string(),
                    "Fire Hose".to_string(),
                    "Protective Gear".to_string(),
                ]);

                if matches!(disaster_severity, DisasterSeverity::Critical) {
                    equipment.extend_from_slice(&[
                        "Ladder Truck".to_string(),
                        "Foam Tank".to_string(),
                        "Breathing Apparatus".to_string(),
                    ]);
                }
            },
            "rescue" => {
                equipment.extend_from_slice(&[
                    "Rescue Vehicle".to_string(),
                    "Search Equipment".to_string(),
                    "Rope and Harness".to_string(),
                    "First Aid Kit".to_string(),
                ]);

                if matches!(disaster_severity, DisasterSeverity::Severe | DisasterSeverity::Critical) {
                    equipment.extend_from_slice(&[
                        "Heavy Lifting Equipment".to_string(),
                        "Cutting Tools".to_string(),
                        "Communication Radio".to_string(),
                    ]);
                }
            },
            "police" => {
                equipment.extend_from_slice(&[
                    "Patrol Vehicle".to_string(),
                    "Traffic Cones".to_string(),
                    "Communication Radio".to_string(),
                ]);
            },
            _ => {
                equipment.push("Standard Emergency Kit".to_string());
            }
        }

        // Add any specifically requested equipment
        for item in requested_equipment {
            if !equipment.contains(item) {
                equipment.push(item.clone());
            }
        }

        equipment
    }
}

#[async_trait]
impl ValidatedUseCase<DispatchEmergencyResponseRequest, EmergencyResponseDispatchResponse> for DispatchEmergencyResponseUseCase {
    async fn validate(&self, request: &DispatchEmergencyResponseRequest) -> AppResult<()> {
        // Validate disaster exists
        self.disaster_repository
            .find_by_id(&request.disaster_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Disaster not found".to_string()))?;

        // Validate dispatcher has authority
        let dispatcher = self.user_repository
            .find_by_id(&request.dispatched_by)
            .await?
            .ok_or_else(|| AppError::NotFound("Dispatcher not found".to_string()))?;

        if !dispatcher.role().has_permission(&Permission::ManageEmergencyResponse) {
            return Err(AppError::Forbidden("Insufficient permissions to dispatch emergency response".to_string()));
        }

        // Validate team type (case-insensitive, trim spaces)
        let valid_team_types = ["medical", "fire", "rescue", "police", "search_and_rescue"];
        let team_type_norm = request.response_team_type.trim().to_lowercase();
        if !valid_team_types.contains(&team_type_norm.as_str()) {
            return Err(AppError::Validation(format!(
                "Invalid team type. Must be one of: {}",
                valid_team_types.join(", ")
            )));
        }

        // Validate priority level
        if request.priority_level == 0 || request.priority_level > 5 {
            return Err(AppError::Validation("Priority level must be between 1 and 5".to_string()));
        }

        // Validate personnel count
        if request.estimated_personnel_needed == 0 || request.estimated_personnel_needed > 100 {
            return Err(AppError::Validation("Personnel needed must be between 1 and 100".to_string()));
        }

        Ok(())
    }
}

#[async_trait]
impl UseCase<DispatchEmergencyResponseRequest, EmergencyResponseDispatchResponse> for DispatchEmergencyResponseUseCase {
    async fn execute(&self, request: DispatchEmergencyResponseRequest) -> AppResult<EmergencyResponseDispatchResponse> {
        // Get disaster details
        let mut disaster = self.disaster_repository
            .find_by_id(&request.disaster_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Disaster not found".to_string()))?;

        // Normalize team type and map aliases
        let mut team_type_norm = request.response_team_type.trim().to_lowercase();
        if team_type_norm == "search_and_rescue" { team_type_norm = "rescue".to_string(); }

        // Find nearest station and estimate arrival
        // Convert disaster location to shared Coordinates
        let disaster_loc = {
            let loc = disaster.location();
            SCoordinates { latitude: loc.latitude, longitude: loc.longitude, altitude: loc.altitude }
        };
        let station_coords = self.find_nearest_station(&disaster_loc, &team_type_norm).await?;
        let estimated_arrival = self
            .calculate_arrival_time(&station_coords, &disaster_loc, &team_type_norm)
            .await?;

        // Resolve a human-readable station name (best-effort)
        let assigned_station_name = match self.geo_service.reverse_geocode(&station_coords).await {
            Ok(name) => Some(name),
            Err(_) => None,
        };

        // Allocate resources
        let resources = self.allocate_resources(disaster.severity(), &team_type_norm, &request.special_equipment_needed);

        // Update disaster status to Responded if currently Reported/Verified
        use crate::domain::entities::disaster::DisasterStatus;
        if matches!(disaster.status(), DisasterStatus::Reported | DisasterStatus::Verified) {
            // Ignore status update failure quietly only if invalid transition; otherwise bubble up
            if let Err(e) = disaster.update_status(DisasterStatus::Responded, request.dispatched_by.clone()) {
                // Only allow no-op if already responded/resolved/closed; else return error
                if !matches!(disaster.status(), DisasterStatus::Responded | DisasterStatus::Resolved | DisasterStatus::Closed) {
                    return Err(e);
                }
            }
            let _ = self.disaster_repository.update(&disaster).await?;
        }

        // Create response payload
        let response_id = Uuid::new_v4();
        let personnel = request.estimated_personnel_needed;

        // Publish domain event
        let event = EmergencyResponseDispatchedEvent {
            event_id: Uuid::new_v4(),
            disaster_id: request.disaster_id.clone(),
            response_team_id: response_id,
            dispatched_by: request.dispatched_by.clone(),
            estimated_arrival,
            resources_allocated: resources.clone(),
            occurred_at: Utc::now(),
            version: disaster.version() as u64,
        };
        self.event_publisher.publish(&event).await?;

        // Send notifications (best-effort)
        let notify_payload = crate::domain::ports::services::EmergencyResponse {
            id: crate::shared::types::EmergencyResponseId::new(),
            disaster_id: request.disaster_id.clone(),
            response_team_type: team_type_norm.clone(),
            assigned_station: assigned_station_name.clone(),
            estimated_arrival: Some(estimated_arrival),
            status: "Dispatched".to_string(),
            resources_deployed: resources.clone(),
        };
        let _ = self.notification_service.notify_emergency_dispatch(&disaster, &notify_payload).await;

        Ok(EmergencyResponseDispatchResponse {
            response_id,
            disaster_id: request.disaster_id,
            team_type: team_type_norm,
            estimated_arrival,
            personnel_count: personnel,
            equipment_allocated: resources,
            status: "Dispatched".to_string(),
            dispatched_at: Utc::now(),
        })
    }
}
