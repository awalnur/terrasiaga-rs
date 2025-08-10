/// Emergency response use cases
/// Handles emergency team dispatch, resource allocation, and response coordination

use async_trait::async_trait;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::use_cases::{UseCase, ValidatedUseCase};
use crate::domain::value_objects::*;
use crate::domain::ports::repositories::{DisasterRepository, UserRepository};
use crate::domain::ports::services::{NotificationService, GeoService};
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
    geo_service: Arc<dyn GeoService>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl DispatchEmergencyResponseUseCase {
    pub fn new(
        disaster_repository: Arc<dyn DisasterRepository>,
        user_repository: Arc<dyn UserRepository>,
        notification_service: Arc<dyn NotificationService>,
        geo_service: Arc<dyn GeoService>,
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
        from_location: &Coordinates,
        to_location: &Coordinates,
        team_type: &str,
    ) -> AppResult<DateTime<Utc>> {
        let distance = from_location.distance_to(to_location);
        
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
        let effective_speed = base_speed / traffic_factor;
        
        let travel_time_hours = distance / 1000.0 / effective_speed;
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
    async fn find_nearest_station(&self, location: &Coordinates, team_type: &str) -> AppResult<Coordinates> {
        // This would query a database of emergency stations
        // For now, returning a mock coordinate for Jakarta Emergency Center
        Ok(Coordinates::new(-6.2088, 106.8456)?) // Mock Jakarta coordinate
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
                
                if matches!(disaster_severity, DisasterSeverity::High | DisasterSeverity::Critical) {
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

                if matches!(disaster_severity, DisasterSeverity::High | DisasterSeverity::Critical) {
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

        if !dispatcher.role().can_perform("dispatch_emergency_response") {
            return Err(AppError::Forbidden("Insufficient permissions to dispatch emergency response".to_string()));
        }

        // Validate team type
        let valid_team_types = ["medical", "fire", "rescue", "police", "search_and_rescue"];
        if !valid_team_types.contains(&request.response_team_type.as_str()) {
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
        let disaster = self.disaster_repository
            .find_by_id(&request.disaster_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Disaster not found".to_string()))?;

        // Find nearest emergency station
        let station_location = self.find_nearest_station(disaster.location(), &request.response_team_type).await?;

        // Calculate estimated arrival time
        let estimated_arrival = self.calculate_arrival_time(
            &station_location,
            disaster.location(),
            &request.response_team_type,
        ).await?;

        // Allocate resources
        let equipment_allocated = self.allocate_resources(
            disaster.severity(),
            &request.response_team_type,
            &request.special_equipment_needed,
        );

        // Generate response ID
        let response_id = Uuid::new_v4();
        let dispatched_at = Utc::now();

        // Create response record (in real implementation, save to database)
        let response = EmergencyResponseDispatchResponse {
            response_id,
            disaster_id: request.disaster_id.clone(),
            team_type: request.response_team_type.clone(),
            estimated_arrival,
            personnel_count: request.estimated_personnel_needed,
            equipment_allocated: equipment_allocated.clone(),
            status: "dispatched".to_string(),
            dispatched_at,
        };

        // Publish domain event
        let event = EmergencyResponseDispatchedEvent {
            event_id: Uuid::new_v4(),
            disaster_id: request.disaster_id.clone(),
            response_team_id: response_id,
            dispatched_by: request.dispatched_by.clone(),
            estimated_arrival,
            resources_allocated: equipment_allocated,
            occurred_at: dispatched_at,
            version: 1,
        };

        self.event_publisher.publish(&event).await?;

        // Send notifications to relevant parties
        self.notification_service
            .notify_emergency_dispatch(&disaster, &response)
            .await?;

        Ok(response)
    }
}

/// Request to get active emergency responses
#[derive(Debug, Clone)]
pub struct GetActiveResponsesRequest {
    pub disaster_id: Option<DisasterId>,
    pub team_type: Option<String>,
    pub status_filter: Option<Vec<String>>,
    pub radius_km: Option<f64>,
    pub center_location: Option<Coordinates>,
}

/// Use case for getting active emergency responses
pub struct GetActiveResponsesUseCase {
    // Implementation would query response database
}

impl GetActiveResponsesUseCase {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl UseCase<GetActiveResponsesRequest, Vec<EmergencyResponseDispatchResponse>> for GetActiveResponsesUseCase {
    async fn execute(&self, _request: GetActiveResponsesRequest) -> AppResult<Vec<EmergencyResponseDispatchResponse>> {
        // In real implementation, this would query the emergency response database
        // For now, returning empty vector
        Ok(vec![])
    }
}
