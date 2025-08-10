use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Severity levels for audit events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventSeverity {
    Info,
    Warning,
    Critical,
}

/// Generic event types for business-agnostic auditing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    // Authentication & Authorization Events
    AuthenticationAttempt,
    AuthenticationSuccess,
    AuthenticationFailure,
    AuthorizationGranted,
    AuthorizationDenied,
    SessionCreated,
    SessionTerminated,
    
    // Resource Management Events  
    ResourceCreated,
    ResourceRead,
    ResourceUpdated,
    ResourceDeleted,
    ResourcePermissionChanged,
    
    // System Events
    SystemStartup,
    SystemShutdown,
    ConfigurationChanged,
    MaintenanceModeEnabled,
    MaintenanceModeDisabled,
    
    // Security Events
    SecurityPolicyViolation,
    SuspiciousActivity,
    DataExport,
    DataImport,
    PasswordChanged,
    AccountLocked,
    AccountUnlocked,
    
    // Administrative Events
    UserCreated,
    UserModified,
    UserDeactivated,
    RoleAssigned,
    RoleRevoked,
    PermissionGranted,
    PermissionRevoked,
    
    // Integration Events
    ExternalApiCall,
    WebhookReceived,
    ExternalSystemIntegration,
    
    // Custom Events (for business-specific extensions)
    Custom(String),
}

/// Core audit event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique identifier for this event
    pub id: String,
    /// Event type
    pub event_type: EventType,
    /// Event severity
    pub severity: EventSeverity,
    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,
    /// User ID who performed the action (if applicable)
    pub actor_id: Option<String>,
    /// User ID being impersonated (if applicable)
    pub impersonator_id: Option<String>,
    /// Tenant/Organization context
    pub tenant_id: Option<String>,
    /// Request ID for correlation
    pub request_id: Option<String>,
    /// Resource being acted upon
    pub resource_type: Option<String>,
    /// ID of the resource
    pub resource_id: Option<String>,
    /// Source IP address
    pub source_ip: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Human-readable description
    pub description: String,
    /// Structured metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Previous values (for update operations)
    pub previous_values: Option<serde_json::Value>,
    /// New values (for create/update operations)
    pub new_values: Option<serde_json::Value>,
    /// Operation outcome
    pub outcome: EventOutcome,
    /// Additional tags for filtering
    pub tags: Vec<String>,
}

/// Event outcome enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventOutcome {
    Success,
    Failure,
    Partial,
    Unknown,
}

impl std::fmt::Display for EventOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventOutcome::Success => write!(f, "success"),
            EventOutcome::Failure => write!(f, "failure"),
            EventOutcome::Partial => write!(f, "partial"),
            EventOutcome::Unknown => write!(f, "unknown"),
        }
    }
}

/// Builder for creating audit events
pub struct AuditEventBuilder {
    event: AuditEvent,
}

impl AuditEventBuilder {
    pub fn new(event_type: EventType, description: impl Into<String>) -> Self {
        Self {
            event: AuditEvent {
                id: Uuid::new_v4().to_string(),
                event_type,
                severity: EventSeverity::Info,
                timestamp: Utc::now(),
                actor_id: None,
                impersonator_id: None,
                tenant_id: None,
                request_id: None,
                resource_type: None,
                resource_id: None,
                source_ip: None,
                user_agent: None,
                description: description.into(),
                metadata: HashMap::new(),
                previous_values: None,
                new_values: None,
                outcome: EventOutcome::Success,
                tags: Vec::new(),
            }
        }
    }

    pub fn severity(mut self, severity: EventSeverity) -> Self {
        self.event.severity = severity;
        self
    }

    pub fn actor_id(mut self, actor_id: impl Into<String>) -> Self {
        self.event.actor_id = Some(actor_id.into());
        self
    }

    pub fn impersonator_id(mut self, impersonator_id: impl Into<String>) -> Self {
        self.event.impersonator_id = Some(impersonator_id.into());
        self
    }

    pub fn tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.event.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn request_id(mut self, request_id: impl Into<String>) -> Self {
        self.event.request_id = Some(request_id.into());
        self
    }

    pub fn resource(mut self, resource_type: impl Into<String>, resource_id: impl Into<String>) -> Self {
        self.event.resource_type = Some(resource_type.into());
        self.event.resource_id = Some(resource_id.into());
        self
    }

    pub fn source_ip(mut self, source_ip: impl Into<String>) -> Self {
        self.event.source_ip = Some(source_ip.into());
        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.event.user_agent = Some(user_agent.into());
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.event.metadata.insert(key.into(), value);
        self
    }

    pub fn previous_values(mut self, values: serde_json::Value) -> Self {
        self.event.previous_values = Some(values);
        self
    }

    pub fn new_values(mut self, values: serde_json::Value) -> Self {
        self.event.new_values = Some(values);
        self
    }

    pub fn outcome(mut self, outcome: EventOutcome) -> Self {
        self.event.outcome = outcome;
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.event.tags.push(tag.into());
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.event.tags.extend(tags);
        self
    }

    pub fn build(self) -> AuditEvent {
        self.event
    }
}

impl AuditEvent {
    /// Create a new builder
    pub fn builder(event_type: EventType, description: impl Into<String>) -> AuditEventBuilder {
        AuditEventBuilder::new(event_type, description)
    }

    /// Check if event should be alerted
    pub fn should_alert(&self) -> bool {
        matches!(self.severity, EventSeverity::Critical)
            || matches!(self.outcome, EventOutcome::Failure)
            || matches!(
                self.event_type,
                EventType::SecurityPolicyViolation
                    | EventType::SuspiciousActivity
                    | EventType::AuthenticationFailure
                    | EventType::AuthorizationDenied
            )
    }

    /// Get event category for grouping
    pub fn category(&self) -> &'static str {
        match &self.event_type {
            EventType::AuthenticationAttempt
            | EventType::AuthenticationSuccess
            | EventType::AuthenticationFailure
            | EventType::AuthorizationGranted
            | EventType::AuthorizationDenied
            | EventType::SessionCreated
            | EventType::SessionTerminated => "authentication",

            EventType::ResourceCreated
            | EventType::ResourceRead
            | EventType::ResourceUpdated
            | EventType::ResourceDeleted
            | EventType::ResourcePermissionChanged => "resource",

            EventType::SystemStartup
            | EventType::SystemShutdown
            | EventType::ConfigurationChanged
            | EventType::MaintenanceModeEnabled
            | EventType::MaintenanceModeDisabled => "system",

            EventType::SecurityPolicyViolation
            | EventType::SuspiciousActivity
            | EventType::DataExport
            | EventType::DataImport
            | EventType::PasswordChanged
            | EventType::AccountLocked
            | EventType::AccountUnlocked => "security",

            EventType::UserCreated
            | EventType::UserModified
            | EventType::UserDeactivated
            | EventType::RoleAssigned
            | EventType::RoleRevoked
            | EventType::PermissionGranted
            | EventType::PermissionRevoked => "administration",

            EventType::ExternalApiCall
            | EventType::WebhookReceived
            | EventType::ExternalSystemIntegration => "integration",

            EventType::Custom(_) => "custom",
        }
    }

    /// Convert to a loggable format
    pub fn to_log_format(&self) -> String {
        format!(
            "[{}] {} by {} on {}: {}",
            self.severity,
            self.event_type,
            self.actor_id.as_deref().unwrap_or("system"),
            self.resource_type.as_deref().unwrap_or("unknown"),
            self.description
        )
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Custom(name) => write!(f, "CUSTOM_{}", name.to_uppercase()),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl std::fmt::Display for EventSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}