/// Unit tests for Disaster domain entity
/// Tests business logic, status transitions, and validation rules

use terra_siaga::domain::entities::{
    disaster::{Disaster, DisasterStatus, DisasterSeverity},
    user::UserRole,
};
use terra_siaga::shared::types::{DisasterId, UserId};
use uuid::Uuid;
use chrono::Utc;

#[cfg(test)]
mod disaster_entity_tests {
    use super::*;

    fn create_test_disaster() -> Disaster {
        Disaster {
            id: DisasterId(Uuid::new_v4()),
            title: "Test Earthquake".to_string(),
            description: "A magnitude 6.0 earthquake".to_string(),
            disaster_type: "earthquake".to_string(),
            severity: DisasterSeverity::High,
            status: DisasterStatus::Reported,
            latitude: -6.2088,
            longitude: 106.8456,
            address: Some("Jakarta, Indonesia".to_string()),
            reporter_id: UserId(Uuid::new_v4()),
            assigned_responders: Vec::new(),
            affected_population: Some(10000),
            images: vec!["image1.jpg".to_string(), "image2.jpg".to_string()],
            contact_info: Some("+6281234567890".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_disaster_creation() {
        let disaster = create_test_disaster();

        assert_eq!(disaster.title, "Test Earthquake");
        assert_eq!(disaster.disaster_type, "earthquake");
        assert_eq!(disaster.severity, DisasterSeverity::High);
        assert_eq!(disaster.status, DisasterStatus::Reported);
        assert_eq!(disaster.latitude, -6.2088);
        assert_eq!(disaster.longitude, 106.8456);
    }

    #[test]
    fn test_disaster_status_transitions() {
        let mut disaster = create_test_disaster();

        // Reported -> Verified
        assert!(disaster.can_transition_to(DisasterStatus::Verified));
        disaster.update_status(DisasterStatus::Verified);
        assert_eq!(disaster.status, DisasterStatus::Verified);

        // Verified -> Responding
        assert!(disaster.can_transition_to(DisasterStatus::Responding));
        disaster.update_status(DisasterStatus::Responding);
        assert_eq!(disaster.status, DisasterStatus::Responding);

        // Responding -> Resolved
        assert!(disaster.can_transition_to(DisasterStatus::Resolved));
        disaster.update_status(DisasterStatus::Resolved);
        assert_eq!(disaster.status, DisasterStatus::Resolved);
    }

    #[test]
    fn test_invalid_status_transitions() {
        let mut disaster = create_test_disaster();

        // Cannot go directly from Reported to Resolved
        assert!(!disaster.can_transition_to(DisasterStatus::Resolved));

        // Cannot go backwards from Verified to Reported
        disaster.update_status(DisasterStatus::Verified);
        assert!(!disaster.can_transition_to(DisasterStatus::Reported));
    }

    #[test]
    fn test_assign_responder() {
        let mut disaster = create_test_disaster();
        let responder_id = UserId(Uuid::new_v4());

        assert!(disaster.assigned_responders.is_empty());

        disaster.assign_responder(responder_id);
        assert_eq!(disaster.assigned_responders.len(), 1);
        assert!(disaster.assigned_responders.contains(&responder_id));
    }

    #[test]
    fn test_cannot_assign_duplicate_responder() {
        let mut disaster = create_test_disaster();
        let responder_id = UserId(Uuid::new_v4());

        disaster.assign_responder(responder_id);
        disaster.assign_responder(responder_id); // Try to assign again

        assert_eq!(disaster.assigned_responders.len(), 1);
    }

    #[test]
    fn test_remove_responder() {
        let mut disaster = create_test_disaster();
        let responder_id = UserId(Uuid::new_v4());

        disaster.assign_responder(responder_id);
        assert_eq!(disaster.assigned_responders.len(), 1);

        disaster.remove_responder(responder_id);
        assert!(disaster.assigned_responders.is_empty());
    }

    #[test]
    fn test_disaster_severity_levels() {
        assert!(DisasterSeverity::Critical > DisasterSeverity::High);
        assert!(DisasterSeverity::High > DisasterSeverity::Medium);
        assert!(DisasterSeverity::Medium > DisasterSeverity::Low);
    }

    #[test]
    fn test_disaster_is_active() {
        let mut disaster = create_test_disaster();

        disaster.status = DisasterStatus::Reported;
        assert!(disaster.is_active());

        disaster.status = DisasterStatus::Verified;
        assert!(disaster.is_active());

        disaster.status = DisasterStatus::Responding;
        assert!(disaster.is_active());

        disaster.status = DisasterStatus::Resolved;
        assert!(!disaster.is_active());
    }

    #[test]
    fn test_disaster_requires_immediate_response() {
        let mut disaster = create_test_disaster();

        disaster.severity = DisasterSeverity::Critical;
        assert!(disaster.requires_immediate_response());

        disaster.severity = DisasterSeverity::High;
        assert!(disaster.requires_immediate_response());

        disaster.severity = DisasterSeverity::Medium;
        assert!(!disaster.requires_immediate_response());

        disaster.severity = DisasterSeverity::Low;
        assert!(!disaster.requires_immediate_response());
    }

    #[test]
    fn test_disaster_location_validation() {
        let disaster = create_test_disaster();

        // Valid coordinates for Jakarta
        assert!(disaster.has_valid_coordinates());

        let mut invalid_disaster = disaster.clone();
        invalid_disaster.latitude = 91.0; // Invalid latitude
        assert!(!invalid_disaster.has_valid_coordinates());

        invalid_disaster.latitude = -6.2088;
        invalid_disaster.longitude = 181.0; // Invalid longitude
        assert!(!invalid_disaster.has_valid_coordinates());
    }

    #[test]
    fn test_disaster_update_affected_population() {
        let mut disaster = create_test_disaster();

        disaster.update_affected_population(Some(15000));
        assert_eq!(disaster.affected_population, Some(15000));

        disaster.update_affected_population(None);
        assert_eq!(disaster.affected_population, None);
    }
}
