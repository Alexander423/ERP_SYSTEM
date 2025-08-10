use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Context information for errors, providing additional debugging and tracking data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Unique identifier for this error instance
    pub error_id: String,
    /// Request ID for tracing
    pub request_id: Option<String>,
    /// User ID if available
    pub user_id: Option<String>,
    /// Tenant ID if in multi-tenant context
    pub tenant_id: Option<String>,
    /// Additional structured data
    pub metadata: HashMap<String, serde_json::Value>,
    /// Stack of error origins (for chained errors)
    pub trace: Vec<String>,
    /// Timestamp when error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            error_id: Uuid::new_v4().to_string(),
            request_id: None,
            user_id: None,
            tenant_id: None,
            metadata: HashMap::new(),
            trace: Vec::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    pub fn add_trace(mut self, trace: impl Into<String>) -> Self {
        self.trace.push(trace.into());
        self
    }

    pub fn add_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Request context for carrying information throughout the request lifecycle
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub correlation_id: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            user_id: None,
            tenant_id: None,
            source_ip: None,
            user_agent: None,
            correlation_id: None,
            started_at: chrono::Utc::now(),
        }
    }

    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = request_id.into();
        self
    }

    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
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

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// Convert to ErrorContext for error reporting
    pub fn to_error_context(&self) -> ErrorContext {
        ErrorContext::new()
            .with_request_id(self.request_id.clone())
            .with_user_id(self.user_id.clone().unwrap_or_default())
            .with_tenant_id(self.tenant_id.clone().unwrap_or_default())
            .with_metadata("source_ip".to_string(), 
                serde_json::Value::String(self.source_ip.clone().unwrap_or_default()))
            .with_metadata("user_agent".to_string(), 
                serde_json::Value::String(self.user_agent.clone().unwrap_or_default()))
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new()
    }
}