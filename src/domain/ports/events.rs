/// Domain events - Business events that occur within the system
/// These represent significant business occurrences that other parts of the system may need to react to

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::shared::{AppResult, UserId, DisasterId, LocationId};

/// Base trait for all domain events
pub trait DomainEvent: Send + Sync {
    fn event_id(&self) -> Uuid;
    fn event_type(&self) -> &'static str;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn aggregate_id(&self) -> String;
    fn version(&self) -> u32;
}

/// Event publisher interface
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: Box<dyn DomainEvent>) -> AppResult<()>;
    async fn publish_batch(&self, events: Vec<Box<dyn DomainEvent>>) -> AppResult<()>;
}

/// Event subscriber interface
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    async fn handle(&self, event: Box<dyn DomainEvent>) -> AppResult<()>;
    fn supported_events(&self) -> Vec<&'static str>;
}

// Specific domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisteredEvent {
    pub event_id: Uuid,
    pub user_id: UserId,
    pub email: String,
    pub username: String,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

impl DomainEvent for UserRegisteredEvent {
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "user.registered" }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> String { self.user_id.0.to_string() }
    fn version(&self) -> u32 { self.version }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisasterReportedEvent {
    pub event_id: Uuid,
    pub disaster_id: DisasterId,
    pub reporter_id: UserId,
    pub location_id: Option<LocationId>,
    pub disaster_type: String,
    pub severity: u8,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

impl DomainEvent for DisasterReportedEvent {
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "disaster.reported" }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> String { self.disaster_id.0.to_string() }
    fn version(&self) -> u32 { self.version }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisasterStatusChangedEvent {
    pub event_id: Uuid,
    pub disaster_id: DisasterId,
    pub old_status: String,
    pub new_status: String,
    pub changed_by: UserId,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

impl DomainEvent for DisasterStatusChangedEvent {
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "disaster.status_changed" }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> String { self.disaster_id.0.to_string() }
    fn version(&self) -> u32 { self.version }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyAlertTriggeredEvent {
    pub event_id: Uuid,
    pub disaster_id: DisasterId,
    pub alert_level: String,
    pub affected_area: String,
    pub message: String,
    pub occurred_at: DateTime<Utc>,
    pub version: u32,
}

impl DomainEvent for EmergencyAlertTriggeredEvent {
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "emergency.alert_triggered" }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> String { self.disaster_id.0.to_string() }
    fn version(&self) -> u32 { self.version }
}
