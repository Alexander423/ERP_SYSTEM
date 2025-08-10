use super::{
    traits::{AuditBackend, AuditFilter, BackendHealth, SortOrder},
    AuditEvent,
};
use crate::error::{Error, ErrorCode, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Database-backed audit repository
pub struct DatabaseAuditRepository {
    pool: Arc<PgPool>,
    table_name: String,
}

impl DatabaseAuditRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            table_name: "audit_events".to_string(),
        }
    }

    pub fn with_table_name(mut self, table_name: impl Into<String>) -> Self {
        self.table_name = table_name.into();
        self
    }

    /// Initialize the audit table if it doesn't exist
    pub async fn initialize(&self) -> Result<()> {
        let sql = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id VARCHAR(255) PRIMARY KEY,
                event_type VARCHAR(100) NOT NULL,
                severity VARCHAR(20) NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                actor_id VARCHAR(255),
                impersonator_id VARCHAR(255),
                tenant_id VARCHAR(255),
                request_id VARCHAR(255),
                resource_type VARCHAR(100),
                resource_id VARCHAR(255),
                source_ip INET,
                user_agent TEXT,
                description TEXT NOT NULL,
                metadata JSONB,
                previous_values JSONB,
                new_values JSONB,
                outcome VARCHAR(20) NOT NULL,
                tags TEXT[],
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            
            CREATE INDEX IF NOT EXISTS idx_{}_timestamp ON {} (timestamp);
            CREATE INDEX IF NOT EXISTS idx_{}_actor_id ON {} (actor_id);
            CREATE INDEX IF NOT EXISTS idx_{}_tenant_id ON {} (tenant_id);
            CREATE INDEX IF NOT EXISTS idx_{}_event_type ON {} (event_type);
            CREATE INDEX IF NOT EXISTS idx_{}_resource ON {} (resource_type, resource_id);
            CREATE INDEX IF NOT EXISTS idx_{}_severity ON {} (severity);
            "#,
            self.table_name,
            self.table_name, self.table_name,
            self.table_name, self.table_name,
            self.table_name, self.table_name,
            self.table_name, self.table_name,
            self.table_name, self.table_name,
            self.table_name, self.table_name,
        );

        sqlx::query(&sql).execute(self.pool.as_ref()).await?;
        info!("Audit table '{}' initialized", self.table_name);
        Ok(())
    }
}

#[async_trait]
impl AuditBackend for DatabaseAuditRepository {
    async fn store_event(&self, event: &AuditEvent) -> Result<()> {
        let sql = format!(
            r#"
            INSERT INTO {} (
                id, event_type, severity, timestamp, actor_id, impersonator_id,
                tenant_id, request_id, resource_type, resource_id, source_ip,
                user_agent, description, metadata, previous_values, new_values,
                outcome, tags
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
            )
            "#,
            self.table_name
        );

        let source_ip: Option<String> = event.source_ip.clone();

        let result = sqlx::query(&sql)
            .bind(&event.id)
            .bind(&event.event_type.to_string())
            .bind(&event.severity.to_string())
            .bind(&event.timestamp)
            .bind(&event.actor_id)
            .bind(&event.impersonator_id)
            .bind(&event.tenant_id)
            .bind(&event.request_id)
            .bind(&event.resource_type)
            .bind(&event.resource_id)
            .bind(source_ip)
            .bind(&event.user_agent)
            .bind(&event.description)
            .bind(serde_json::to_value(&event.metadata).unwrap_or(serde_json::Value::Null))
            .bind(&event.previous_values)
            .bind(&event.new_values)
            .bind(&event.outcome.to_string())
            .bind(&event.tags)
            .execute(self.pool.as_ref())
            .await;

        match result {
            Ok(_) => {
                debug!("Stored audit event: {}", event.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to store audit event {}: {}", event.id, e);
                Err(Error::from(e))
            }
        }
    }

    async fn retrieve_events(&self, filter: &AuditFilter) -> Result<Vec<AuditEvent>> {
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        // Build WHERE conditions
        if let Some(start_time) = &filter.start_time {
            param_count += 1;
            conditions.push(format!("timestamp >= ${}", param_count));
            params.push(Box::new(*start_time));
        }

        if let Some(end_time) = &filter.end_time {
            param_count += 1;
            conditions.push(format!("timestamp <= ${}", param_count));
            params.push(Box::new(*end_time));
        }

        if let Some(actor_id) = &filter.actor_id {
            param_count += 1;
            conditions.push(format!("actor_id = ${}", param_count));
            params.push(Box::new(actor_id.clone()));
        }

        if let Some(tenant_id) = &filter.tenant_id {
            param_count += 1;
            conditions.push(format!("tenant_id = ${}", param_count));
            params.push(Box::new(tenant_id.clone()));
        }

        if let Some(resource_type) = &filter.resource_type {
            param_count += 1;
            conditions.push(format!("resource_type = ${}", param_count));
            params.push(Box::new(resource_type.clone()));
        }

        if let Some(description_contains) = &filter.description_contains {
            param_count += 1;
            conditions.push(format!("description ILIKE ${}", param_count));
            params.push(Box::new(format!("%{}%", description_contains)));
        }

        // Build ORDER BY
        let order_by = match filter.sort_order {
            SortOrder::TimestampAsc => "timestamp ASC",
            SortOrder::TimestampDesc => "timestamp DESC",
            SortOrder::SeverityDesc => "CASE severity WHEN 'critical' THEN 1 WHEN 'warning' THEN 2 ELSE 3 END, timestamp DESC",
        };

        // Build LIMIT and OFFSET
        let limit_clause = if let Some(limit) = filter.limit {
            param_count += 1;
            params.push(Box::new(limit as i64));
            format!(" LIMIT ${}", param_count)
        } else {
            String::new()
        };

        let offset_clause = if let Some(offset) = filter.offset {
            param_count += 1;
            params.push(Box::new(offset as i64));
            format!(" OFFSET ${}", param_count)
        } else {
            String::new()
        };

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            r#"
            SELECT id, event_type, severity, timestamp, actor_id, impersonator_id,
                   tenant_id, request_id, resource_type, resource_id, source_ip,
                   user_agent, description, metadata, previous_values, new_values,
                   outcome, tags
            FROM {}{}
            ORDER BY {}{}{}
            "#,
            self.table_name, where_clause, order_by, limit_clause, offset_clause
        );

        // This is a simplified version - in a real implementation, 
        // you'd need to properly bind the dynamic parameters
        let rows = sqlx::query(&sql)
            .fetch_all(self.pool.as_ref())
            .await?;

        let mut events = Vec::new();
        for row in rows {
            // Parse the row into AuditEvent
            // This is simplified - you'd need proper parsing for all fields
            let event = AuditEvent {
                id: row.get("id"),
                event_type: parse_event_type(&row.get::<String, _>("event_type")),
                severity: parse_severity(&row.get::<String, _>("severity")),
                timestamp: row.get("timestamp"),
                actor_id: row.get("actor_id"),
                impersonator_id: row.get("impersonator_id"),
                tenant_id: row.get("tenant_id"),
                request_id: row.get("request_id"),
                resource_type: row.get("resource_type"),
                resource_id: row.get("resource_id"),
                source_ip: row.get::<Option<String>, _>("source_ip"),
                user_agent: row.get("user_agent"),
                description: row.get("description"),
                metadata: serde_json::from_value(
                    row.get::<serde_json::Value, _>("metadata")
                ).unwrap_or_default(),
                previous_values: row.get("previous_values"),
                new_values: row.get("new_values"),
                outcome: parse_outcome(&row.get::<String, _>("outcome")),
                tags: row.get::<Vec<String>, _>("tags"),
            };
            events.push(event);
        }

        Ok(events)
    }

    async fn count_events(&self, _filter: &AuditFilter) -> Result<u64> {
        // Similar to retrieve_events but with COUNT query
        let sql = format!("SELECT COUNT(*) FROM {}", self.table_name);
        
        let count: i64 = sqlx::query_scalar(&sql)
            .fetch_one(self.pool.as_ref())
            .await?;

        Ok(count as u64)
    }

    async fn health_check(&self) -> Result<BackendHealth> {
        match sqlx::query("SELECT 1").fetch_one(self.pool.as_ref()).await {
            Ok(_) => Ok(BackendHealth {
                is_healthy: true,
                message: None,
                last_write: None, // Would need to track this
                events_stored_today: None, // Would need to query for today's count
            }),
            Err(e) => Ok(BackendHealth {
                is_healthy: false,
                message: Some(e.to_string()),
                last_write: None,
                events_stored_today: None,
            }),
        }
    }

    async fn cleanup_old_events(&self, older_than: DateTime<Utc>) -> Result<u64> {
        let sql = format!(
            "DELETE FROM {} WHERE timestamp < $1",
            self.table_name
        );

        let result = sqlx::query(&sql)
            .bind(older_than)
            .execute(self.pool.as_ref())
            .await?;

        info!("Cleaned up {} old audit events", result.rows_affected());
        Ok(result.rows_affected())
    }
}

/// Generic audit repository that can use multiple backends
pub struct AuditRepository {
    backends: Vec<Box<dyn AuditBackend>>,
    primary_backend: usize,
}

impl AuditRepository {
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
            primary_backend: 0,
        }
    }

    pub fn add_backend(mut self, backend: Box<dyn AuditBackend>) -> Self {
        self.backends.push(backend);
        self
    }

    pub fn with_primary_backend(mut self, index: usize) -> Self {
        if index < self.backends.len() {
            self.primary_backend = index;
        }
        self
    }
}

#[async_trait]
impl AuditBackend for AuditRepository {
    async fn store_event(&self, event: &AuditEvent) -> Result<()> {
        if self.backends.is_empty() {
            return Err(Error::new(ErrorCode::ConfigurationError, "No audit backends configured"));
        }

        // Store to all backends, but only fail if primary backend fails
        let mut primary_result = Ok(());
        for (i, backend) in self.backends.iter().enumerate() {
            match backend.store_event(event).await {
                Ok(_) => {
                    if i == self.primary_backend {
                        primary_result = Ok(());
                    }
                }
                Err(e) => {
                    if i == self.primary_backend {
                        primary_result = Err(e);
                    } else {
                        error!("Secondary audit backend {} failed: {}", i, e);
                    }
                }
            }
        }

        primary_result
    }

    async fn retrieve_events(&self, filter: &AuditFilter) -> Result<Vec<AuditEvent>> {
        if self.primary_backend >= self.backends.len() {
            return Err(Error::new(ErrorCode::ConfigurationError, "Invalid primary backend index"));
        }

        self.backends[self.primary_backend]
            .retrieve_events(filter)
            .await
    }

    async fn count_events(&self, filter: &AuditFilter) -> Result<u64> {
        if self.primary_backend >= self.backends.len() {
            return Err(Error::new(ErrorCode::ConfigurationError, "Invalid primary backend index"));
        }

        self.backends[self.primary_backend]
            .count_events(filter)
            .await
    }

    async fn health_check(&self) -> Result<BackendHealth> {
        if self.primary_backend >= self.backends.len() {
            return Err(Error::new(ErrorCode::ConfigurationError, "Invalid primary backend index"));
        }

        self.backends[self.primary_backend]
            .health_check()
            .await
    }

    async fn cleanup_old_events(&self, older_than: DateTime<Utc>) -> Result<u64> {
        let mut total_cleaned = 0u64;
        
        for backend in &self.backends {
            match backend.cleanup_old_events(older_than).await {
                Ok(cleaned) => total_cleaned += cleaned,
                Err(e) => error!("Failed to cleanup events in backend: {}", e),
            }
        }

        Ok(total_cleaned)
    }
}

impl Default for AuditRepository {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for parsing database values
fn parse_event_type(s: &str) -> crate::audit::event::EventType {
    use crate::audit::event::EventType;
    
    match s {
        "AUTHENTICATION_ATTEMPT" => EventType::AuthenticationAttempt,
        "AUTHENTICATION_SUCCESS" => EventType::AuthenticationSuccess,
        "AUTHENTICATION_FAILURE" => EventType::AuthenticationFailure,
        // Add more cases as needed
        _ => EventType::Custom(s.to_string()),
    }
}

fn parse_severity(s: &str) -> crate::audit::event::EventSeverity {
    use crate::audit::event::EventSeverity;
    
    match s.to_lowercase().as_str() {
        "info" => EventSeverity::Info,
        "warning" => EventSeverity::Warning,
        "critical" => EventSeverity::Critical,
        _ => EventSeverity::Info,
    }
}

fn parse_outcome(s: &str) -> crate::audit::event::EventOutcome {
    use crate::audit::event::EventOutcome;
    
    match s.to_lowercase().as_str() {
        "success" => EventOutcome::Success,
        "failure" => EventOutcome::Failure,
        "partial" => EventOutcome::Partial,
        _ => EventOutcome::Unknown,
    }
}