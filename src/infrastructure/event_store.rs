/// Event Store implementation for Terra Siaga
/// Provides event sourcing capabilities with PostgreSQL storage

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use serde_json;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::events::{DomainEvent, EventStore};
use crate::shared::{AppError, AppResult};

/// Event store record for database persistence
#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = event_store)]
pub struct EventStoreRecord {
    pub event_id: Uuid,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub event_version: i64,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

table! {
    event_store (event_id) {
        event_id -> Uuid,
        aggregate_id -> Uuid,
        event_type -> Varchar,
        event_data -> Jsonb,
        event_version -> BigInt,
        occurred_at -> Timestamptz,
        created_at -> Timestamptz,
    }
}

/// PostgreSQL-based event store implementation
pub struct PostgresEventStore {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl PostgresEventStore {
    pub fn new(pool: Arc<Pool<ConnectionManager<PgConnection>>>) -> Self {
        Self { pool }
    }

    /// Serialize domain event to JSON
    fn serialize_event(&self, event: &dyn DomainEvent) -> AppResult<serde_json::Value> {
        // This is a simplified serialization - in production you'd want proper event serialization
        let event_data = serde_json::json!({
            "event_id": event.event_id(),
            "event_type": event.event_type(),
            "occurred_at": event.occurred_at(),
            "aggregate_id": event.aggregate_id(),
            "version": event.version(),
            // Additional event-specific data would be serialized here based on event type
        });
        
        Ok(event_data)
    }
}

#[async_trait]
impl EventStore for PostgresEventStore {
    async fn save_events(
        &self,
        aggregate_id: Uuid,
        events: &[Box<dyn DomainEvent>],
        expected_version: u64,
    ) -> AppResult<()> {
        use self::event_store::dsl::*;
        
        let mut conn = self.pool.get()
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Start transaction
        conn.transaction(|conn| -> Result<(), AppError> {
            // Check current version
            let current_version: Option<i64> = event_store
                .filter(aggregate_id.eq(aggregate_id))
                .select(diesel::dsl::max(event_version))
                .first(conn)
                .optional()
                .map_err(|e| AppError::Database(e.to_string()))?;

            let current_version = current_version.unwrap_or(0) as u64;
            
            if current_version != expected_version {
                return Err(AppError::Conflict(format!(
                    "Expected version {}, but current version is {}",
                    expected_version, current_version
                )));
            }

            // Insert events
            for (index, event) in events.iter().enumerate() {
                let event_data = self.serialize_event(event.as_ref())?;
                
                let record = EventStoreRecord {
                    event_id: event.event_id(),
                    aggregate_id: event.aggregate_id(),
                    event_type: event.event_type().to_string(),
                    event_data,
                    event_version: (expected_version + index as u64 + 1) as i64,
                    occurred_at: event.occurred_at(),
                    created_at: Utc::now(),
                };

                diesel::insert_into(event_store)
                    .values(&record)
                    .execute(conn)
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }

            Ok(())
        })?;

        Ok(())
    }

    async fn get_events(
        &self,
        aggregate_id: Uuid,
        from_version: Option<u64>,
    ) -> AppResult<Vec<Box<dyn DomainEvent>>> {
        use self::event_store::dsl::*;
        
        let mut conn = self.pool.get()
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut query = event_store
            .filter(aggregate_id.eq(aggregate_id))
            .order(event_version.asc())
            .into_boxed();

        if let Some(version) = from_version {
            query = query.filter(event_version.gt(version as i64));
        }

        let records: Vec<EventStoreRecord> = query
            .load(&mut conn)
            .map_err(|e| AppError::Database(e.to_string()))?;

        // In a real implementation, you'd deserialize these back to proper event types
        // For now, returning empty vector as this requires event type registry
        Ok(vec![])
    }
}

/// In-memory event store for testing
pub struct InMemoryEventStore {
    events: std::sync::Mutex<std::collections::HashMap<Uuid, Vec<Box<dyn DomainEvent>>>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn save_events(
        &self,
        aggregate_id: Uuid,
        events: &[Box<dyn DomainEvent>],
        expected_version: u64,
    ) -> AppResult<()> {
        let mut store = self.events.lock().unwrap();
        
        let current_events = store.entry(aggregate_id).or_insert_with(Vec::new);
        let current_version = current_events.len() as u64;
        
        if current_version != expected_version {
            return Err(AppError::Conflict(format!(
                "Expected version {}, but current version is {}",
                expected_version, current_version
            )));
        }

        // Clone events for storage (this is simplified - in real implementation you'd serialize)
        for event in events {
            // Note: This is a simplified approach - proper cloning of trait objects requires more work
            current_events.push(event.clone());
        }

        Ok(())
    }

    async fn get_events(
        &self,
        aggregate_id: Uuid,
        from_version: Option<u64>,
    ) -> AppResult<Vec<Box<dyn DomainEvent>>> {
        let store = self.events.lock().unwrap();
        
        if let Some(events) = store.get(&aggregate_id) {
            let start_index = from_version.unwrap_or(0) as usize;
            let result = events[start_index..].to_vec();
            Ok(result)
        } else {
            Ok(vec![])
        }
    }
}
