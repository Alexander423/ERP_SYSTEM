use super::AuditEvent;
use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Trait for audit backends (database, file, remote, etc.)
#[async_trait]
pub trait AuditBackend: Send + Sync {
    /// Store an audit event
    async fn store_event(&self, event: &AuditEvent) -> Result<()>;
    
    /// Retrieve audit events with filtering
    async fn retrieve_events(
        &self,
        filter: &AuditFilter,
    ) -> Result<Vec<AuditEvent>>;
    
    /// Count audit events matching filter
    async fn count_events(&self, filter: &AuditFilter) -> Result<u64>;
    
    /// Health check for the backend
    async fn health_check(&self) -> Result<BackendHealth>;
    
    /// Clean up old events based on retention policy
    async fn cleanup_old_events(&self, older_than: DateTime<Utc>) -> Result<u64>;
}

/// Health status of audit backend
#[derive(Debug, Clone)]
pub struct BackendHealth {
    pub is_healthy: bool,
    pub message: Option<String>,
    pub last_write: Option<DateTime<Utc>>,
    pub events_stored_today: Option<u64>,
}

/// Filter for querying audit events
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    /// Start time range
    pub start_time: Option<DateTime<Utc>>,
    /// End time range
    pub end_time: Option<DateTime<Utc>>,
    /// Filter by actor ID
    pub actor_id: Option<String>,
    /// Filter by tenant ID
    pub tenant_id: Option<String>,
    /// Filter by event types
    pub event_types: Option<Vec<String>>,
    /// Filter by severity
    pub severities: Option<Vec<String>>,
    /// Filter by resource type
    pub resource_type: Option<String>,
    /// Filter by resource ID
    pub resource_id: Option<String>,
    /// Filter by outcome
    pub outcomes: Option<Vec<String>>,
    /// Filter by tags
    pub tags: Option<Vec<String>>,
    /// Search in description
    pub description_contains: Option<String>,
    /// Pagination limit
    pub limit: Option<u32>,
    /// Pagination offset
    pub offset: Option<u32>,
    /// Sort order
    pub sort_order: SortOrder,
}

/// Sort order for audit events
#[derive(Debug, Clone)]
pub enum SortOrder {
    TimestampAsc,
    TimestampDesc,
    SeverityDesc,
}

impl Default for SortOrder {
    fn default() -> Self {
        SortOrder::TimestampDesc
    }
}

/// Trait for types that can be audited
pub trait Auditable {
    /// Get the resource type for auditing
    fn resource_type() -> &'static str;
    
    /// Get the resource ID
    fn resource_id(&self) -> String;
    
    /// Convert to JSON for audit logging
    fn to_audit_json(&self) -> Value;
}

/// Builder for audit filters
pub struct AuditFilterBuilder {
    filter: AuditFilter,
}

impl AuditFilterBuilder {
    pub fn new() -> Self {
        Self {
            filter: AuditFilter::default(),
        }
    }

    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.filter.start_time = Some(start);
        self.filter.end_time = Some(end);
        self
    }

    pub fn actor_id(mut self, actor_id: impl Into<String>) -> Self {
        self.filter.actor_id = Some(actor_id.into());
        self
    }

    pub fn tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.filter.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn event_types(mut self, event_types: Vec<String>) -> Self {
        self.filter.event_types = Some(event_types);
        self
    }

    pub fn severities(mut self, severities: Vec<String>) -> Self {
        self.filter.severities = Some(severities);
        self
    }

    pub fn resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.filter.resource_type = Some(resource_type.into());
        self
    }

    pub fn resource_id(mut self, resource_id: impl Into<String>) -> Self {
        self.filter.resource_id = Some(resource_id.into());
        self
    }

    pub fn outcomes(mut self, outcomes: Vec<String>) -> Self {
        self.filter.outcomes = Some(outcomes);
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.filter.tags = Some(tags);
        self
    }

    pub fn description_contains(mut self, text: impl Into<String>) -> Self {
        self.filter.description_contains = Some(text.into());
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.filter.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.filter.offset = Some(offset);
        self
    }

    pub fn sort_order(mut self, sort_order: SortOrder) -> Self {
        self.filter.sort_order = sort_order;
        self
    }

    pub fn build(self) -> AuditFilter {
        self.filter
    }
}

impl Default for AuditFilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditFilter {
    pub fn builder() -> AuditFilterBuilder {
        AuditFilterBuilder::new()
    }
}