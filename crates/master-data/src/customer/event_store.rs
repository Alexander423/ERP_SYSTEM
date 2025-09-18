//! Event Store for Customer Domain
//!
//! This module provides persistence for customer domain events using PostgreSQL.
//! Events are stored in append-only fashion with optimistic concurrency control.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

use crate::customer::events::{CustomerEvent, CustomerEventWithMetadata, EventMetadata};
use crate::error::{MasterDataError, Result};
use erp_core::TenantContext;

/// Event store operations for customer domain
#[async_trait]
pub trait CustomerEventStore: Send + Sync {
    /// Append events to the store with optimistic concurrency control
    async fn append_events(
        &self,
        aggregate_id: Uuid,
        events: Vec<CustomerEvent>,
        expected_version: Option<i64>,
        user_id: Option<Uuid>,
    ) -> Result<i64>;

    /// Load all events for a customer aggregate
    async fn load_events(&self, aggregate_id: Uuid) -> Result<Vec<CustomerEventWithMetadata>>;

    /// Load events from a specific version onwards
    async fn load_events_from_version(
        &self,
        aggregate_id: Uuid,
        from_version: i64,
    ) -> Result<Vec<CustomerEventWithMetadata>>;

    /// Load events by type within a date range
    async fn load_events_by_type(
        &self,
        event_types: Vec<String>,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
        limit: Option<i32>,
    ) -> Result<Vec<CustomerEventWithMetadata>>;

    /// Load events for multiple customers (for bulk processing)
    async fn load_events_for_customers(
        &self,
        customer_ids: Vec<Uuid>,
    ) -> Result<HashMap<Uuid, Vec<CustomerEventWithMetadata>>>;

    /// Get the current version (sequence number) for an aggregate
    async fn get_current_version(&self, aggregate_id: Uuid) -> Result<Option<i64>>;

    /// Create a snapshot of the current state
    async fn create_snapshot(
        &self,
        aggregate_id: Uuid,
        version: i64,
        snapshot_data: serde_json::Value,
    ) -> Result<()>;

    /// Load the latest snapshot for an aggregate
    async fn load_snapshot(&self, aggregate_id: Uuid) -> Result<Option<(i64, serde_json::Value)>>;

    /// Get event statistics for monitoring
    async fn get_event_statistics(&self) -> Result<EventStatistics>;
}

/// PostgreSQL implementation of the customer event store
pub struct PostgresCustomerEventStore {
    pool: PgPool,
    tenant_context: TenantContext,
}

/// Event statistics for monitoring and analytics
#[derive(Debug, Clone)]
pub struct EventStatistics {
    pub total_events: i64,
    pub events_by_type: HashMap<String, i64>,
    pub events_last_24h: i64,
    pub unique_aggregates: i64,
    pub storage_size_mb: f64,
}

impl PostgresCustomerEventStore {
    pub fn new(pool: PgPool, tenant_context: TenantContext) -> Self {
        Self {
            pool,
            tenant_context,
        }
    }
}

#[async_trait]
impl CustomerEventStore for PostgresCustomerEventStore {
    async fn append_events(
        &self,
        aggregate_id: Uuid,
        events: Vec<CustomerEvent>,
        expected_version: Option<i64>,
        user_id: Option<Uuid>,
    ) -> Result<i64> {
        if events.is_empty() {
            return Ok(0);
        }

        let mut tx = self.pool.begin().await?;

        // Get current version with row lock (first lock existing records, then get max)
        sqlx::query(
            "SELECT event_id FROM customer_events WHERE aggregate_id = $1 AND tenant_id = $2 FOR UPDATE",
        )
        .bind(aggregate_id)
        .bind(self.tenant_context.tenant_id.0)
        .fetch_all(&mut *tx)
        .await?;

        let current_version: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(MAX(sequence_number), 0) FROM customer_events
             WHERE aggregate_id = $1 AND tenant_id = $2",
        )
        .bind(aggregate_id)
        .bind(self.tenant_context.tenant_id.0)
        .fetch_one(&mut *tx)
        .await?;

        // Check optimistic concurrency control
        if let Some(expected) = expected_version {
            if current_version != expected {
                return Err(MasterDataError::SynchronizationConflict {
                    entity_type: "customer_event".to_string(),
                    entity_id: aggregate_id.to_string(),
                    local_version: current_version as i32,
                    remote_version: expected as i32,
                });
            }
        }

        let mut next_version = current_version;
        let mut event_records = Vec::new();

        // Prepare event records
        for event in events {
            next_version += 1;
            let metadata = EventMetadata::new(
                aggregate_id,
                self.tenant_context.tenant_id.0,
                next_version,
                user_id,
            );

            let event_data = serde_json::to_value(&event)?;
            let metadata_json = serde_json::to_value(&metadata)?;

            event_records.push((
                metadata.event_id,
                aggregate_id,
                self.tenant_context.tenant_id.0,
                next_version,
                event.event_type().to_string(),
                event_data,
                metadata_json,
                metadata.occurred_at,
                metadata.recorded_at,
                user_id,
            ));
        }

        // Insert events in batch
        for (
            event_id,
            agg_id,
            tenant_id,
            seq_num,
            event_type,
            event_data,
            metadata,
            occurred_at,
            recorded_at,
            uid,
        ) in event_records
        {
            sqlx::query(
                r#"
                INSERT INTO customer_events
                (event_id, aggregate_id, tenant_id, sequence_number, event_type,
                 event_data, metadata, occurred_at, recorded_at, user_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(event_id)
            .bind(agg_id)
            .bind(tenant_id)
            .bind(seq_num)
            .bind(event_type)
            .bind(event_data)
            .bind(metadata)
            .bind(occurred_at)
            .bind(recorded_at)
            .bind(uid)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(next_version)
    }

    async fn load_events(&self, aggregate_id: Uuid) -> Result<Vec<CustomerEventWithMetadata>> {
        let records = sqlx::query(
            r#"
            SELECT event_id, aggregate_id, tenant_id, sequence_number, event_type,
                   event_data, metadata, occurred_at, recorded_at, user_id
            FROM customer_events
            WHERE aggregate_id = $1 AND tenant_id = $2
            ORDER BY sequence_number ASC
            "#,
        )
        .bind(aggregate_id)
        .bind(self.tenant_context.tenant_id.0)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for record in records {
            let event: CustomerEvent = serde_json::from_value(record.try_get("event_data")?)?;
            let metadata: EventMetadata = serde_json::from_value(record.try_get("metadata")?)?;

            events.push(CustomerEventWithMetadata { metadata, event });
        }

        Ok(events)
    }

    async fn load_events_from_version(
        &self,
        aggregate_id: Uuid,
        from_version: i64,
    ) -> Result<Vec<CustomerEventWithMetadata>> {
        let records = sqlx::query(
            r#"
            SELECT event_id, aggregate_id, tenant_id, sequence_number, event_type,
                   event_data, metadata, occurred_at, recorded_at, user_id
            FROM customer_events
            WHERE aggregate_id = $1 AND tenant_id = $2 AND sequence_number > $3
            ORDER BY sequence_number ASC
            "#,
        )
        .bind(aggregate_id)
        .bind(self.tenant_context.tenant_id.0)
        .bind(from_version)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for record in records {
            let event: CustomerEvent = serde_json::from_value(record.try_get("event_data")?)?;
            let metadata: EventMetadata = serde_json::from_value(record.try_get("metadata")?)?;

            events.push(CustomerEventWithMetadata { metadata, event });
        }

        Ok(events)
    }

    async fn load_events_by_type(
        &self,
        event_types: Vec<String>,
        from_date: Option<DateTime<Utc>>,
        to_date: Option<DateTime<Utc>>,
        limit: Option<i32>,
    ) -> Result<Vec<CustomerEventWithMetadata>> {
        let mut query = String::from(
            r#"
            SELECT event_id, aggregate_id, tenant_id, sequence_number, event_type,
                   event_data, metadata, occurred_at, recorded_at, user_id
            FROM customer_events
            WHERE tenant_id = $1 AND event_type = ANY($2)
            "#,
        );

        let mut param_count = 2;

        if from_date.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND occurred_at >= ${}", param_count));
        }

        if to_date.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND occurred_at <= ${}", param_count));
        }

        query.push_str(" ORDER BY occurred_at DESC");

        if let Some(_limit_val) = limit {
            param_count += 1;
            query.push_str(&format!(" LIMIT ${}", param_count));
        }

        let mut query_builder = sqlx::query(&query)
            .bind(self.tenant_context.tenant_id.0)
            .bind(&event_types);

        if let Some(from) = from_date {
            query_builder = query_builder.bind(from);
        }

        if let Some(to) = to_date {
            query_builder = query_builder.bind(to);
        }

        if let Some(limit_val) = limit {
            query_builder = query_builder.bind(limit_val);
        }

        let records = query_builder.fetch_all(&self.pool).await?;

        let mut events = Vec::new();
        for record in records {
            use sqlx::Row;
            let event_data: serde_json::Value = record.try_get("event_data").unwrap_or_default();
            let metadata_json: serde_json::Value = record.try_get("metadata").unwrap_or_default();

            let event: CustomerEvent = serde_json::from_value(event_data)?;
            let metadata: EventMetadata = serde_json::from_value(metadata_json)?;

            events.push(CustomerEventWithMetadata { metadata, event });
        }

        Ok(events)
    }

    async fn load_events_for_customers(
        &self,
        customer_ids: Vec<Uuid>,
    ) -> Result<HashMap<Uuid, Vec<CustomerEventWithMetadata>>> {
        if customer_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let records = sqlx::query(
            r#"
            SELECT event_id, aggregate_id, tenant_id, sequence_number, event_type,
                   event_data, metadata, occurred_at, recorded_at, user_id
            FROM customer_events
            WHERE aggregate_id = ANY($1) AND tenant_id = $2
            ORDER BY aggregate_id, sequence_number ASC
            "#,
        )
        .bind(&customer_ids)
        .bind(self.tenant_context.tenant_id.0)
        .fetch_all(&self.pool)
        .await?;

        let mut result: HashMap<Uuid, Vec<CustomerEventWithMetadata>> = HashMap::new();

        for record in records {
            let event: CustomerEvent = serde_json::from_value(record.try_get("event_data")?)?;
            let metadata: EventMetadata = serde_json::from_value(record.try_get("metadata")?)?;

            let event_with_metadata = CustomerEventWithMetadata { metadata, event };

            result
                .entry(record.try_get("aggregate_id")?)
                .or_insert_with(Vec::new)
                .push(event_with_metadata);
        }

        Ok(result)
    }

    async fn get_current_version(&self, aggregate_id: Uuid) -> Result<Option<i64>> {
        let version = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT MAX(sequence_number) FROM customer_events WHERE aggregate_id = $1 AND tenant_id = $2",
        )
        .bind(aggregate_id)
        .bind(self.tenant_context.tenant_id.0)
        .fetch_one(&self.pool)
        .await?;

        Ok(version)
    }

    async fn create_snapshot(
        &self,
        aggregate_id: Uuid,
        version: i64,
        snapshot_data: serde_json::Value,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO customer_snapshots
            (aggregate_id, tenant_id, version, snapshot_data, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (aggregate_id, tenant_id)
            DO UPDATE SET
                version = EXCLUDED.version,
                snapshot_data = EXCLUDED.snapshot_data,
                created_at = EXCLUDED.created_at
            "#,
        )
        .bind(aggregate_id)
        .bind(self.tenant_context.tenant_id.0)
        .bind(version)
        .bind(snapshot_data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn load_snapshot(&self, aggregate_id: Uuid) -> Result<Option<(i64, serde_json::Value)>> {
        let snapshot = sqlx::query(
            "SELECT version, snapshot_data FROM customer_snapshots WHERE aggregate_id = $1 AND tenant_id = $2",
        )
        .bind(aggregate_id)
        .bind(self.tenant_context.tenant_id.0)
        .fetch_optional(&self.pool)
        .await?;

        Ok(snapshot.map(|s| (s.try_get("version").unwrap(), s.try_get("snapshot_data").unwrap())))
    }

    async fn get_event_statistics(&self) -> Result<EventStatistics> {
        let total_events = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM customer_events WHERE tenant_id = $1",
        )
        .bind(self.tenant_context.tenant_id.0)
        .fetch_one(&self.pool)
        .await?;

        let events_last_24h = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM customer_events WHERE tenant_id = $1 AND recorded_at > NOW() - INTERVAL '24 hours'",
        )
        .bind(self.tenant_context.tenant_id.0)
        .fetch_one(&self.pool)
        .await?;

        let unique_aggregates = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(DISTINCT aggregate_id) FROM customer_events WHERE tenant_id = $1",
        )
        .bind(self.tenant_context.tenant_id.0)
        .fetch_one(&self.pool)
        .await?;

        let type_counts = sqlx::query(
            "SELECT event_type, COUNT(*) as count FROM customer_events WHERE tenant_id = $1 GROUP BY event_type",
        )
        .bind(self.tenant_context.tenant_id.0)
        .fetch_all(&self.pool)
        .await?;

        let mut events_by_type = HashMap::new();
        for record in type_counts {
            events_by_type.insert(record.try_get("event_type")?, record.try_get::<Option<i64>, _>("count")?.unwrap_or(0));
        }

        // Estimate storage size (simplified)
        let storage_size_mb = (total_events as f64 * 2.0) / 1024.0; // Rough estimate: 2KB per event

        Ok(EventStatistics {
            total_events,
            events_by_type,
            events_last_24h,
            unique_aggregates,
            storage_size_mb,
        })
    }
}

#[cfg(test)]
mod tests {

    // Note: These would be integration tests requiring a test database
    // For now, they serve as documentation of the expected interface

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_append_and_load_events() {
        // This test would verify that events can be appended and loaded correctly
        // with proper ordering and metadata
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_optimistic_concurrency_control() {
        // This test would verify that concurrent modifications are handled correctly
        // and that version conflicts are detected
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_event_filtering_and_pagination() {
        // This test would verify that events can be filtered by type, date range,
        // and pagination works correctly
    }
}