pub mod audit;
pub mod config;
pub mod database;
pub mod error;
pub mod jobs;
pub mod metrics;
pub mod security;
pub mod session;
pub mod types;
pub mod utils;

pub use audit::{AuditEvent, AuditLogger, AuditRepository};
pub use config::{Config, CorsConfig, EmailConfig};
pub use database::{DatabasePool, TenantPool};
pub use error::{Error, ErrorCode, ErrorContext, ErrorMetrics, Result};
pub use jobs::{JobExecutor, JobQueue, RedisJobQueue, SerializableJob};
pub use metrics::{AuthMetrics, MetricsRegistry, MetricsService};
pub use session::{SessionManager, SessionData, SessionConfig, SessionState, SessionStats};
pub use types::*;

#[cfg(test)]
mod tests;

// Re-export commonly used types from dependencies
pub use uuid::Uuid;
pub use chrono::{DateTime, Utc};