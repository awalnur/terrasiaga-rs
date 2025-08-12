/// Event handling and messaging system for Terra Siaga
/// Implements Event Sourcing and CQRS patterns for better scalability

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::shared::error::{AppError, AppResult};
use crate::shared::types::*;

/// Define Status enum for domain events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Active,
    Inactive,
    Pending,
    Completed,
    Cancelled,
    InProgress,
    Failed,
}

/// Base trait for all domain events
#[async_trait]
pub trait DomainEvent: Send + Sync + std::fmt::Debug {
    fn event_type(&self) -> &'static str;
    fn aggregate_id(&self) -> String;
    fn event_version(&self) -> u32;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn correlation_id(&self) -> Option<String>;
    fn causation_id(&self) -> Option<String>;
}

/// Event metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub event_type: String,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event_version: u32,
    pub occurred_at: DateTime<Utc>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub user_id: Option<UserId>,
    pub session_id: Option<String>,
}

/// Stored event with metadata and payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub metadata: EventMetadata,
    pub payload: serde_json::Value,
}

/// Disaster-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisasterEvent {
    DisasterReported {
        report_id: ReportId,
        disaster_id: DisasterId,
        disaster_type: DisasterType,
        severity: SeverityLevel,
        location: LocationInfo,
        reporter_id: UserId,
        description: String,
        reported_at: DateTime<Utc>,
    },
    DisasterVerified {
        disaster_id: DisasterId,
        verified_by: UserId,
        verification_notes: Option<String>,
        verified_at: DateTime<Utc>,
    },
    DisasterStatusChanged {
        disaster_id: DisasterId,
        old_status: Status,
        new_status: Status,
        changed_by: UserId,
        reason: Option<String>,
        changed_at: DateTime<Utc>,
    },
    DisasterSeverityUpdated {
        disaster_id: DisasterId,
        old_severity: SeverityLevel,
        new_severity: SeverityLevel,
        updated_by: UserId,
        reason: String,
        updated_at: DateTime<Utc>,
    },
}

#[async_trait]
impl DomainEvent for DisasterEvent {
    fn event_type(&self) -> &'static str {
        match self {
            DisasterEvent::DisasterReported { .. } => "disaster.reported",
            DisasterEvent::DisasterVerified { .. } => "disaster.verified",
            DisasterEvent::DisasterStatusChanged { .. } => "disaster.status_changed",
            DisasterEvent::DisasterSeverityUpdated { .. } => "disaster.severity_updated",
        }
    }

    fn aggregate_id(&self) -> String {
        match self {
            DisasterEvent::DisasterReported { disaster_id, .. } => disaster_id.to_string(),
            DisasterEvent::DisasterVerified { disaster_id, .. } => disaster_id.to_string(),
            DisasterEvent::DisasterStatusChanged { disaster_id, .. } => disaster_id.to_string(),
            DisasterEvent::DisasterSeverityUpdated { disaster_id, .. } => disaster_id.to_string(),
        }
    }

    fn event_version(&self) -> u32 {
        1
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            DisasterEvent::DisasterReported { reported_at, .. } => *reported_at,
            DisasterEvent::DisasterVerified { verified_at, .. } => *verified_at,
            DisasterEvent::DisasterStatusChanged { changed_at, .. } => *changed_at,
            DisasterEvent::DisasterSeverityUpdated { updated_at, .. } => *updated_at,
        }
    }

    fn correlation_id(&self) -> Option<String> {
        None // Can be set during event publishing
    }

    fn causation_id(&self) -> Option<String> {
        None // Can be set during event publishing
    }
}

/// Emergency response events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencyResponseEvent {
    ResponseInitiated {
        response_id: EmergencyResponseId,
        disaster_id: DisasterId,
        coordinator_id: UserId,
        response_type: String,
        initiated_at: DateTime<Utc>,
    },
    VolunteerAssigned {
        response_id: EmergencyResponseId,
        volunteer_id: VolunteerId,
        role: String,
        assigned_by: UserId,
        assigned_at: DateTime<Utc>,
    },
    ResourceAllocated {
        response_id: EmergencyResponseId,
        resource_id: ResourceId,
        quantity: u32,
        allocated_by: UserId,
        allocated_at: DateTime<Utc>,
    },
    ResponseStatusChanged {
        response_id: EmergencyResponseId,
        old_status: Status,
        new_status: Status,
        changed_by: UserId,
        changed_at: DateTime<Utc>,
    },
}

#[async_trait]
impl DomainEvent for EmergencyResponseEvent {
    fn event_type(&self) -> &'static str {
        match self {
            EmergencyResponseEvent::ResponseInitiated { .. } => "emergency_response.initiated",
            EmergencyResponseEvent::VolunteerAssigned { .. } => "emergency_response.volunteer_assigned",
            EmergencyResponseEvent::ResourceAllocated { .. } => "emergency_response.resource_allocated",
            EmergencyResponseEvent::ResponseStatusChanged { .. } => "emergency_response.status_changed",
        }
    }

    fn aggregate_id(&self) -> String {
        match self {
            EmergencyResponseEvent::ResponseInitiated { response_id, .. } => response_id.to_string(),
            EmergencyResponseEvent::VolunteerAssigned { response_id, .. } => response_id.to_string(),
            EmergencyResponseEvent::ResourceAllocated { response_id, .. } => response_id.to_string(),
            EmergencyResponseEvent::ResponseStatusChanged { response_id, .. } => response_id.to_string(),
        }
    }

    fn event_version(&self) -> u32 {
        1
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            EmergencyResponseEvent::ResponseInitiated { initiated_at, .. } => *initiated_at,
            EmergencyResponseEvent::VolunteerAssigned { assigned_at, .. } => *assigned_at,
            EmergencyResponseEvent::ResourceAllocated { allocated_at, .. } => *allocated_at,
            EmergencyResponseEvent::ResponseStatusChanged { changed_at, .. } => *changed_at,
        }
    }

    fn correlation_id(&self) -> Option<String> {
        None
    }

    fn causation_id(&self) -> Option<String> {
        None
    }
}

/// User and authentication events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserEvent {
    UserRegistered {
        user_id: UserId,
        email: String,
        role: UserRole,
        registered_at: DateTime<Utc>,
    },
    UserLoggedIn {
        user_id: UserId,
        session_id: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        logged_in_at: DateTime<Utc>,
    },
    UserLoggedOut {
        user_id: UserId,
        session_id: String,
        logged_out_at: DateTime<Utc>,
    },
    UserRoleChanged {
        user_id: UserId,
        old_role: UserRole,
        new_role: UserRole,
        changed_by: UserId,
        changed_at: DateTime<Utc>,
    },
    UserPermissionsUpdated {
        user_id: UserId,
        added_permissions: Vec<Permission>,
        removed_permissions: Vec<Permission>,
        updated_by: UserId,
        updated_at: DateTime<Utc>,
    },
}

#[async_trait]
impl DomainEvent for UserEvent {
    fn event_type(&self) -> &'static str {
        match self {
            UserEvent::UserRegistered { .. } => "user.registered",
            UserEvent::UserLoggedIn { .. } => "user.logged_in",
            UserEvent::UserLoggedOut { .. } => "user.logged_out",
            UserEvent::UserRoleChanged { .. } => "user.role_changed",
            UserEvent::UserPermissionsUpdated { .. } => "user.permissions_updated",
        }
    }

    fn aggregate_id(&self) -> String {
        match self {
            UserEvent::UserRegistered { user_id, .. } => user_id.to_string(),
            UserEvent::UserLoggedIn { user_id, .. } => user_id.to_string(),
            UserEvent::UserLoggedOut { user_id, .. } => user_id.to_string(),
            UserEvent::UserRoleChanged { user_id, .. } => user_id.to_string(),
            UserEvent::UserPermissionsUpdated { user_id, .. } => user_id.to_string(),
        }
    }

    fn event_version(&self) -> u32 {
        1
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            UserEvent::UserRegistered { registered_at, .. } => *registered_at,
            UserEvent::UserLoggedIn { logged_in_at, .. } => *logged_in_at,
            UserEvent::UserLoggedOut { logged_out_at, .. } => *logged_out_at,
            UserEvent::UserRoleChanged { changed_at, .. } => *changed_at,
            UserEvent::UserPermissionsUpdated { updated_at, .. } => *updated_at,
        }
    }

    fn correlation_id(&self) -> Option<String> {
        None
    }

    fn causation_id(&self) -> Option<String> {
        None
    }
}

/// Event handler trait
#[async_trait]
pub trait EventHandler<T: DomainEvent>: Send + Sync {
    async fn handle(&self, event: &T, metadata: &EventMetadata) -> AppResult<()>;
}

/// Event store trait for persisting events
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append_event(
        &self,
        aggregate_id: &str,
        expected_version: u32,
        event: Box<dyn DomainEvent>,
        metadata: Option<EventMetadata>,
    ) -> AppResult<()>;

    async fn get_events_for_aggregate(
        &self,
        aggregate_id: &str,
        from_version: Option<u32>,
    ) -> AppResult<Vec<StoredEvent>>;

    async fn get_events_by_type(
        &self,
        event_type: &str,
        from_timestamp: Option<DateTime<Utc>>,
        limit: Option<u32>,
    ) -> AppResult<Vec<StoredEvent>>;
}

/// Event bus for publishing and subscribing to events
pub struct EventBus {
    handlers: Arc<RwLock<HashMap<String, Vec<Box<dyn EventHandler<dyn DomainEvent>>>>>>,
    event_store: Arc<dyn EventStore>,
    publisher: mpsc::UnboundedSender<StoredEvent>,
}

impl EventBus {
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        let (publisher, mut receiver) = mpsc::unbounded_channel::<StoredEvent>();
        let handlers = Arc::new(RwLock::new(HashMap::new()));

        // Start background event processing
        let handlers_clone = handlers.clone();
        tokio::spawn(async move {
            while let Some(stored_event) = receiver.recv().await {
                Self::process_event(handlers_clone.clone(), stored_event).await;
            }
        });

        Self {
            handlers,
            event_store,
            publisher,
        }
    }

    /// Publish an event
    pub async fn publish(
        &self,
        event: Box<dyn DomainEvent>,
        user_id: Option<UserId>,
        session_id: Option<String>,
        correlation_id: Option<String>,
        causation_id: Option<String>,
    ) -> AppResult<()> {
        let metadata = EventMetadata {
            event_id: Uuid::new_v4(),
            event_type: event.event_type().to_string(),
            aggregate_id: event.aggregate_id(),
            aggregate_type: "disaster".to_string(), // This should be determined by the event
            event_version: event.event_version(),
            occurred_at: event.occurred_at(),
            correlation_id,
            causation_id,
            user_id,
            session_id,
        };

        // Store the event
        self.event_store
            .append_event(
                &event.aggregate_id(),
                event.event_version(),
                event,
                Some(metadata.clone()),
            )
            .await?;

        // Create stored event for processing
        let stored_event = StoredEvent {
            metadata,
            payload: serde_json::Value::Null, // Would contain serialized event data
        };

        // Publish for async processing
        if let Err(_) = self.publisher.send(stored_event) {
            tracing::error!("Failed to send event to processing queue");
        }

        Ok(())
    }

    /// Subscribe to events of a specific type
    pub async fn subscribe<T: DomainEvent + 'static>(
        &self,
        event_type: &str,
        handler: Box<dyn EventHandler<T>>,
    ) {
        let mut handlers = self.handlers.write().await;
        let event_handlers = handlers.entry(event_type.to_string()).or_insert_with(Vec::new);

        // This would need proper type erasure handling in a real implementation
        // event_handlers.push(Box::new(handler));
    }

    /// Process events asynchronously
    async fn process_event(
        handlers: Arc<RwLock<HashMap<String, Vec<Box<dyn EventHandler<dyn DomainEvent>>>>>>,
        stored_event: StoredEvent,
    ) {
        let handlers_read = handlers.read().await;
        if let Some(event_handlers) = handlers_read.get(&stored_event.metadata.event_type) {
            for handler in event_handlers {
                // Process each handler
                // This would need proper event deserialization
                // if let Err(e) = handler.handle(&event, &stored_event.metadata).await {
                //     tracing::error!("Event handler failed: {}", e);
                // }
            }
        }
    }
}

/// Command trait for CQRS implementation
#[async_trait]
pub trait Command: Send + Sync {
    type Result: Send;
    async fn execute(&self) -> AppResult<Self::Result>;
}

/// Query trait for CQRS implementation
#[async_trait]
pub trait Query: Send + Sync {
    type Result: Send;
    async fn execute(&self) -> AppResult<Self::Result>;
}

/// Command handler trait
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    async fn handle(&self, command: C) -> AppResult<C::Result>;
}

/// Query handler trait
#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    async fn handle(&self, query: Q) -> AppResult<Q::Result>;
}

/// Command bus for CQRS
pub struct CommandBus {
    handlers: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl CommandBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler<C: Command + 'static>(
        &mut self,
        handler: Box<dyn CommandHandler<C>>,
    ) {
        let command_type = std::any::type_name::<C>();
        self.handlers.insert(command_type.to_string(), Box::new(handler));
    }

    pub async fn execute<C: Command + 'static>(&self, command: C) -> AppResult<C::Result> {
        let command_type = std::any::type_name::<C>();

        // This would need proper type handling in a real implementation
        command.execute().await
    }
}

/// Query bus for CQRS
pub struct QueryBus {
    handlers: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl QueryBus {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler<Q: Query + 'static>(
        &mut self,
        handler: Box<dyn QueryHandler<Q>>,
    ) {
        let query_type = std::any::type_name::<Q>();
        self.handlers.insert(query_type.to_string(), Box::new(handler));
    }

    pub async fn execute<Q: Query + 'static>(&self, query: Q) -> AppResult<Q::Result> {
        let query_type = std::any::type_name::<Q>();

        // This would need proper type handling in a real implementation
        query.execute().await
    }
}

/// Event projection for read models
#[async_trait]
pub trait Projection: Send + Sync {
    async fn project(&self, event: &StoredEvent) -> AppResult<()>;
    async fn reset(&self) -> AppResult<()>;
}

/// Projection manager
pub struct ProjectionManager {
    projections: Vec<Arc<dyn Projection>>,
    event_store: Arc<dyn EventStore>,
}

impl ProjectionManager {
    pub fn new(event_store: Arc<dyn EventStore>) -> Self {
        Self {
            projections: Vec::new(),
            event_store,
        }
    }

    pub fn add_projection(&mut self, projection: Arc<dyn Projection>) {
        self.projections.push(projection);
    }

    /// Rebuild all projections from event store
    pub async fn rebuild_projections(&self) -> AppResult<()> {
        for projection in &self.projections {
            projection.reset().await?;
        }

        // This would typically stream all events and replay them
        // let events = self.event_store.get_all_events().await?;
        // for event in events {
        //     for projection in &self.projections {
        //         projection.project(&event).await?;
        //     }
        // }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_disaster_event_creation() {
        let disaster_id = DisasterId::new();
        let report_id = ReportId::new();
        let user_id = UserId::new();

        let event = DisasterEvent::DisasterReported {
            report_id,
            disaster_id,
            disaster_type: DisasterType::Earthquake,
            severity: SeverityLevel::High,
            location: LocationInfo {
                coordinates: Coordinates::new(-6.2088, 106.8456).unwrap(),
                address: Some("Jakarta, Indonesia".to_string()),
                administrative: None,
                landmark: None,
                accuracy_radius: Some(50.0),
            },
            reporter_id: user_id,
            description: "Major earthquake detected".to_string(),
            reported_at: Utc::now(),
        };

        assert_eq!(event.event_type(), "disaster.reported");
        assert_eq!(event.aggregate_id(), disaster_id.to_string());
    }
}
