/// Shared domain events module
/// Re-exports all domain events for easy access

pub use crate::domain::ports::events::*;

/// Domain events for Terra Siaga disaster management system
/// Events represent significant domain changes that other parts of the system should react to

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::domain::value_objects::*;

/// Base trait for all domain events
pub trait DomainEvent: Send + Sync + std::fmt::Debug {
    fn event_id(&self) -> Uuid;
    fn event_type(&self) -> &'static str;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn aggregate_id(&self) -> Uuid;
    fn version(&self) -> u64;
}

/// User-related domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisteredEvent {
    pub event_id: Uuid,
    pub user_id: UserId,
    pub email: Email,
    pub role: UserRole,
    pub occurred_at: DateTime<Utc>,
    pub version: u64,
}

impl DomainEvent for UserRegisteredEvent {
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "UserRegistered" }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> Uuid { self.user_id.value() }
    fn version(&self) -> u64 { self.version }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivatedEvent {
    pub event_id: Uuid,
    pub user_id: UserId,
    pub occurred_at: DateTime<Utc>,
    pub version: u64,
}

impl DomainEvent for UserActivatedEvent {
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "UserActivated" }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> Uuid { self.user_id.value() }
    fn version(&self) -> u64 { self.version }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeactivatedEvent {
    pub event_id: Uuid,
    pub user_id: UserId,
    pub reason: String,
    pub occurred_at: DateTime<Utc>,
    pub version: u64,
}

impl DomainEvent for UserDeactivatedEvent {
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "UserDeactivated" }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> Uuid { self.user_id.value() }
    fn version(&self) -> u64 { self.version }
}

/// Disaster-related domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisasterReportedEvent {
    pub event_id: Uuid,
    pub disaster_id: DisasterId,
    pub reported_by: UserId,
    pub disaster_type: String,
    pub severity: DisasterSeverity,
    pub location: Coordinates,
    pub description: String,
    pub occurred_at: DateTime<Utc>,
    pub version: u64,
}

impl DomainEvent for DisasterReportedEvent {
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "DisasterReported" }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> Uuid { self.disaster_id.value() }
    fn version(&self) -> u64 { self.version }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisasterStatusUpdatedEvent {
    pub event_id: Uuid,
    pub disaster_id: DisasterId,
    pub updated_by: UserId,
    pub old_status: String,
    pub new_status: String,
    pub occurred_at: DateTime<Utc>,
    pub version: u64,
}

impl DomainEvent for DisasterStatusUpdatedEvent {
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "DisasterStatusUpdated" }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> Uuid { self.disaster_id.value() }
    fn version(&self) -> u64 { self.version }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyResponseDispatchedEvent {
    pub event_id: Uuid,
    pub disaster_id: DisasterId,
    pub response_team_id: Uuid,
    pub dispatched_by: UserId,
    pub estimated_arrival: DateTime<Utc>,
    pub resources_allocated: Vec<String>,
    pub occurred_at: DateTime<Utc>,
    pub version: u64,
}

impl DomainEvent for EmergencyResponseDispatchedEvent {
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "EmergencyResponseDispatched" }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> Uuid { self.disaster_id.value() }
    fn version(&self) -> u64 { self.version }
}

/// Notification-related domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSentEvent {
    pub event_id: Uuid,
    pub notification_id: NotificationId,
    pub recipient_id: UserId,
    pub notification_type: String,
    pub channel: String, // SMS, Email, Push, WhatsApp
    pub content: String,
    pub occurred_at: DateTime<Utc>,
    pub version: u64,
}

impl DomainEvent for NotificationSentEvent {
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "NotificationSent" }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> Uuid { self.notification_id.value() }
    fn version(&self) -> u64 { self.version }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassNotificationTriggeredEvent {
    pub event_id: Uuid,
    pub disaster_id: DisasterId,
    pub triggered_by: UserId,
    pub affected_area_radius_km: f64,
    pub notification_type: String,
    pub estimated_recipients: u32,
    pub occurred_at: DateTime<Utc>,
    pub version: u64,
}

impl DomainEvent for MassNotificationTriggeredEvent {
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "MassNotificationTriggered" }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> Uuid { self.disaster_id.value() }
    fn version(&self) -> u64 { self.version }
}

/// Location-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationUpdatedEvent {
    pub event_id: Uuid,
    pub user_id: UserId,
    pub old_location: Option<Coordinates>,
    pub new_location: Coordinates,
    pub accuracy_meters: Option<f64>,
    pub occurred_at: DateTime<Utc>,
    pub version: u64,
}

impl DomainEvent for LocationUpdatedEvent {
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "LocationUpdated" }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn aggregate_id(&self) -> Uuid { self.user_id.value() }
    fn version(&self) -> u64 { self.version }
}

/// Aggregate root trait for entities that can publish events
pub trait AggregateRoot {
    fn uncommitted_events(&self) -> &Vec<Box<dyn DomainEvent>>;
    fn mark_events_as_committed(&mut self);
    fn add_event(&mut self, event: Box<dyn DomainEvent>);
    fn version(&self) -> u64;
    fn increment_version(&mut self);
}

/// Event store interface
#[async_trait::async_trait]
pub trait EventStore: Send + Sync {
    async fn save_events(&self, aggregate_id: Uuid, events: &[Box<dyn DomainEvent>], expected_version: u64) -> crate::shared::AppResult<()>;
    async fn get_events(&self, aggregate_id: Uuid, from_version: Option<u64>) -> crate::shared::AppResult<Vec<Box<dyn DomainEvent>>>;
}

/// Event publisher interface
#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &dyn DomainEvent) -> crate::shared::AppResult<()>;
    async fn publish_batch(&self, events: &[&dyn DomainEvent]) -> crate::shared::AppResult<()>;
}

/// Event handler trait
#[async_trait::async_trait]
pub trait EventHandler<T: DomainEvent>: Send + Sync {
    async fn handle(&self, event: &T) -> crate::shared::AppResult<()>;
}

// Re-export all events and traits
pub use self::{
    UserRegisteredEvent, UserActivatedEvent, UserDeactivatedEvent,
    DisasterReportedEvent, DisasterStatusUpdatedEvent, EmergencyResponseDispatchedEvent,
    NotificationSentEvent, MassNotificationTriggeredEvent, LocationUpdatedEvent,
    DomainEvent, AggregateRoot, EventStore, EventPublisher, EventHandler
};
