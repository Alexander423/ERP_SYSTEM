use super::{
    event::{AuditEvent, EventType},
    traits::AuditBackend,
};
use crate::error::{Error, ErrorCode, ErrorMetrics, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// High-level audit logger that provides convenient methods for logging business events
#[derive(Clone)]
pub struct AuditLogger {
    backend: Arc<dyn AuditBackend>,
    error_metrics: Arc<ErrorMetrics>,
    context: Arc<RwLock<AuditContext>>,
}

/// Context that persists across audit operations in a session/request
#[derive(Debug, Clone, Default)]
pub struct AuditContext {
    pub actor_id: Option<String>,
    pub tenant_id: Option<String>,
    pub request_id: Option<String>,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub impersonator_id: Option<String>,
}

impl AuditLogger {
    pub fn new(backend: Arc<dyn AuditBackend>, error_metrics: Arc<ErrorMetrics>) -> Self {
        Self {
            backend,
            error_metrics,
            context: Arc::new(RwLock::new(AuditContext::default())),
        }
    }

    /// Set the audit context for subsequent operations
    pub async fn set_context(&self, context: AuditContext) {
        let mut ctx = self.context.write().await;
        *ctx = context;
    }

    /// Update specific context fields
    pub async fn update_context<F>(&self, updater: F)
    where
        F: FnOnce(&mut AuditContext),
    {
        let mut ctx = self.context.write().await;
        updater(&mut *ctx);
    }

    /// Log a generic audit event
    pub async fn log_event(&self, mut event: AuditEvent) -> Result<()> {
        // Apply context if fields are not already set
        {
            let ctx = self.context.read().await;
            
            if event.actor_id.is_none() {
                event.actor_id = ctx.actor_id.clone();
            }
            if event.tenant_id.is_none() {
                event.tenant_id = ctx.tenant_id.clone();
            }
            if event.request_id.is_none() {
                event.request_id = ctx.request_id.clone();
            }
            if event.source_ip.is_none() {
                event.source_ip = ctx.source_ip.clone();
            }
            if event.user_agent.is_none() {
                event.user_agent = ctx.user_agent.clone();
            }
            if event.impersonator_id.is_none() {
                event.impersonator_id = ctx.impersonator_id.clone();
            }
        }

        // Log to structured logging as well
        match event.severity {
            crate::audit::event::EventSeverity::Info => {
                info!(
                    event_id = %event.id,
                    event_type = %event.event_type,
                    actor_id = ?event.actor_id,
                    resource = ?event.resource_type,
                    description = %event.description,
                    "Audit event"
                );
            }
            crate::audit::event::EventSeverity::Warning => {
                warn!(
                    event_id = %event.id,
                    event_type = %event.event_type,
                    actor_id = ?event.actor_id,
                    resource = ?event.resource_type,
                    description = %event.description,
                    "Audit event (warning)"
                );
            }
            crate::audit::event::EventSeverity::Critical => {
                error!(
                    event_id = %event.id,
                    event_type = %event.event_type,
                    actor_id = ?event.actor_id,
                    resource = ?event.resource_type,
                    description = %event.description,
                    metadata = ?event.metadata,
                    "Critical audit event"
                );
            }
        }

        // Store to backend
        match self.backend.store_event(&event).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // Record the audit failure as an error metric
                let audit_error = Error::new(
                    ErrorCode::StorageError,
                    format!("Failed to store audit event: {}", e)
                );
                self.error_metrics.record_error(&audit_error).await;
                
                // Critical: if we can't audit, this is a serious security concern
                error!(
                    event_id = %event.id,
                    error = %e,
                    "Failed to store audit event - this is a critical security issue"
                );
                
                Err(e)
            }
        }
    }

    // Convenience methods for common audit events

    /// Log authentication attempt
    pub async fn log_authentication_attempt(
        &self,
        email: &str,
        success: bool,
        failure_reason: Option<&str>,
    ) -> Result<()> {
        let (event_type, description) = if success {
            (EventType::AuthenticationSuccess, format!("User {} authenticated successfully", email))
        } else {
            (
                EventType::AuthenticationFailure,
                format!(
                    "Authentication failed for user {}: {}",
                    email,
                    failure_reason.unwrap_or("Unknown reason")
                ),
            )
        };

        let mut event = AuditEvent::builder(event_type, description)
            .metadata("email".to_string(), serde_json::Value::String(email.to_string()));

        if let Some(reason) = failure_reason {
            event = event.metadata("failure_reason".to_string(), serde_json::Value::String(reason.to_string()));
        }

        self.log_event(event.build()).await
    }

    /// Log resource access
    pub async fn log_resource_access(
        &self,
        action: &str, // "create", "read", "update", "delete"
        resource_type: &str,
        resource_id: &str,
        previous_values: Option<serde_json::Value>,
        new_values: Option<serde_json::Value>,
    ) -> Result<()> {
        let event_type = match action {
            "create" => EventType::ResourceCreated,
            "read" => EventType::ResourceRead,
            "update" => EventType::ResourceUpdated,
            "delete" => EventType::ResourceDeleted,
            _ => EventType::Custom(format!("RESOURCE_{}", action.to_uppercase())),
        };

        let description = format!("{} {} {}", action, resource_type, resource_id);

        let mut event = AuditEvent::builder(event_type, description)
            .resource(resource_type, resource_id)
            .metadata("action".to_string(), serde_json::Value::String(action.to_string()));

        if let Some(prev) = previous_values {
            event = event.previous_values(prev);
        }

        if let Some(new) = new_values {
            event = event.new_values(new);
        }

        self.log_event(event.build()).await
    }

    /// Log security policy violation
    pub async fn log_security_violation(
        &self,
        violation_type: &str,
        description: &str,
        additional_context: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut event = AuditEvent::builder(EventType::SecurityPolicyViolation, description)
            .severity(crate::audit::event::EventSeverity::Critical)
            .metadata("violation_type".to_string(), serde_json::Value::String(violation_type.to_string()));

        if let Some(context) = additional_context {
            event = event.metadata("additional_context".to_string(), context);
        }

        self.log_event(event.build()).await
    }

    /// Log permission changes
    pub async fn log_permission_change(
        &self,
        target_user_id: &str,
        permission_type: &str, // "role_assigned", "role_revoked", "permission_granted", etc.
        permission_details: &str,
    ) -> Result<()> {
        let event_type = match permission_type {
            "role_assigned" => EventType::RoleAssigned,
            "role_revoked" => EventType::RoleRevoked,
            "permission_granted" => EventType::PermissionGranted,
            "permission_revoked" => EventType::PermissionRevoked,
            _ => EventType::Custom(format!("PERMISSION_{}", permission_type.to_uppercase())),
        };

        let description = format!(
            "{} for user {}: {}",
            permission_type.replace('_', " "),
            target_user_id,
            permission_details
        );

        let event = AuditEvent::builder(event_type, description)
            .resource("user", target_user_id)
            .metadata("permission_type".to_string(), serde_json::Value::String(permission_type.to_string()))
            .metadata("permission_details".to_string(), serde_json::Value::String(permission_details.to_string()))
            .build();

        self.log_event(event).await
    }

    /// Log data export/import
    pub async fn log_data_operation(
        &self,
        operation: &str, // "export" or "import"
        data_type: &str,
        record_count: Option<u64>,
        file_path: Option<&str>,
    ) -> Result<()> {
        let event_type = match operation {
            "export" => EventType::DataExport,
            "import" => EventType::DataImport,
            _ => EventType::Custom(format!("DATA_{}", operation.to_uppercase())),
        };

        let description = format!("{} operation for {}", operation, data_type);

        let mut event = AuditEvent::builder(event_type, description)
            .severity(crate::audit::event::EventSeverity::Warning) // Data operations are sensitive
            .metadata("operation".to_string(), serde_json::Value::String(operation.to_string()))
            .metadata("data_type".to_string(), serde_json::Value::String(data_type.to_string()));

        if let Some(count) = record_count {
            event = event.metadata("record_count".to_string(), serde_json::Value::Number(serde_json::Number::from(count)));
        }

        if let Some(path) = file_path {
            event = event.metadata("file_path".to_string(), serde_json::Value::String(path.to_string()));
        }

        self.log_event(event.build()).await
    }

    /// Log system events
    pub async fn log_system_event(
        &self,
        event_type: EventType,
        description: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let mut event = AuditEvent::builder(event_type, description);

        if let Some(meta) = metadata {
            event = event.metadata("system_metadata".to_string(), meta);
        }

        self.log_event(event.build()).await
    }
}

impl AuditContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_actor_id(mut self, actor_id: impl Into<String>) -> Self {
        self.actor_id = Some(actor_id.into());
        self
    }

    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_source_ip(mut self, source_ip: impl Into<String>) -> Self {
        self.source_ip = Some(source_ip.into());
        self
    }

    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn with_impersonator_id(mut self, impersonator_id: impl Into<String>) -> Self {
        self.impersonator_id = Some(impersonator_id.into());
        self
    }
}