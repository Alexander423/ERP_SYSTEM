//! Comprehensive audit logging and security monitoring
//!
//! This module provides enterprise-grade audit capabilities for compliance
//! with regulations like SOX, GDPR, HIPAA, and other security frameworks.

use async_trait::async_trait;
use chrono::Timelike;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::Result;

/// Audit logging service for security and compliance
#[async_trait]
pub trait AuditLogger: Send + Sync {
    /// Log a security event
    async fn log_security_event(&self, event: &AuditEvent) -> Result<()>;

    /// Log a data access event
    async fn log_data_access(&self, access: &DataAccessEvent) -> Result<()>;

    /// Log a system event
    async fn log_system_event(&self, event: &SystemEvent) -> Result<()>;

    /// Query audit trail with filters
    async fn query_audit_trail(&self, query: &AuditQuery) -> Result<AuditTrail>;

    /// Generate compliance report
    async fn generate_compliance_report(
        &self,
        framework: &ComplianceFramework,
        period: &AuditPeriod,
    ) -> Result<ComplianceReport>;

    /// Detect anomalous activities
    async fn detect_anomalies(&self, context: &AnomalyDetectionContext) -> Result<Vec<SecurityAnomaly>>;

    /// Archive old audit logs
    async fn archive_logs(&self, retention_policy: &RetentionPolicy) -> Result<u64>;
}

/// Comprehensive audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub event_type: EventType,
    pub event_category: EventCategory,
    pub user_id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub action: String,
    pub outcome: EventOutcome,
    pub risk_level: RiskLevel,
    pub event_data: HashMap<String, serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub correlation_id: Option<Uuid>,
    pub source_system: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub retention_until: Option<chrono::DateTime<chrono::Utc>>,
}

/// Types of audit events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    SystemConfiguration,
    SecurityIncident,
    ComplianceEvent,
    UserManagement,
    BackupRestore,
    SystemMaintenance,
}

/// Event categories for classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventCategory {
    Security,
    Access,
    Data,
    System,
    Compliance,
    Administrative,
}

/// Event outcome
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventOutcome {
    Success,
    Failure,
    Denied,
    Warning,
    Error,
}

/// Risk level assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Data access event with detailed context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAccessEvent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub table_name: String,
    pub record_id: Option<Uuid>,
    pub fields_accessed: Vec<String>,
    pub access_type: DataAccessType,
    pub purpose: Option<String>,
    pub legal_basis: Option<String>, // For GDPR compliance
    pub retention_period: Option<chrono::Duration>,
    pub data_classification: DataClassification,
    pub query_executed: Option<String>,
    pub results_count: Option<i64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub context: HashMap<String, String>,
}

/// Types of data access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataAccessType {
    Read,
    Create,
    Update,
    Delete,
    Export,
    Decrypt,
    Unmask,
    Aggregate,
}

/// Data classification levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
    TopSecret,
}

/// System events for infrastructure monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub id: Uuid,
    pub event_type: SystemEventType,
    pub severity: EventSeverity,
    pub component: String,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
    pub host: Option<String>,
    pub process_id: Option<u32>,
    pub thread_id: Option<u64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: Option<Uuid>,
}

/// System event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEventType {
    Startup,
    Shutdown,
    ConfigurationChange,
    PerformanceAlert,
    ErrorCondition,
    SecurityAlert,
    ResourceExhaustion,
    NetworkEvent,
    DatabaseEvent,
}

/// Event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
    Fatal,
}

/// Audit query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub tenant_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub event_types: Option<Vec<EventType>>,
    pub event_categories: Option<Vec<EventCategory>>,
    pub outcomes: Option<Vec<EventOutcome>>,
    pub risk_levels: Option<Vec<RiskLevel>>,
    pub resource_types: Option<Vec<String>>,
    pub resource_ids: Option<Vec<Uuid>>,
    pub from_date: Option<chrono::DateTime<chrono::Utc>>,
    pub to_date: Option<chrono::DateTime<chrono::Utc>>,
    pub correlation_id: Option<Uuid>,
    pub search_terms: Option<Vec<String>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

/// Sort order for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Audit trail results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    pub events: Vec<AuditEvent>,
    pub total_count: i64,
    pub summary: AuditSummary,
    pub facets: Option<AuditFacets>,
}

/// Summary statistics for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub total_events: i64,
    pub events_by_type: HashMap<EventType, i64>,
    pub events_by_outcome: HashMap<EventOutcome, i64>,
    pub events_by_risk_level: HashMap<RiskLevel, i64>,
    pub unique_users: i64,
    pub unique_resources: i64,
    pub time_range: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
}

/// Faceted search results for audit data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFacets {
    pub event_types: HashMap<EventType, i64>,
    pub event_categories: HashMap<EventCategory, i64>,
    pub outcomes: HashMap<EventOutcome, i64>,
    pub risk_levels: HashMap<RiskLevel, i64>,
    pub users: HashMap<Uuid, i64>,
    pub resources: HashMap<String, i64>,
}

/// Compliance frameworks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComplianceFramework {
    Gdpr,
    Sox,
    Hipaa,
    PciDss,
    Iso27001,
    Nist,
    CisControls,
    Custom(String),
}

/// Audit period for compliance reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditPeriod {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
    pub timezone: String,
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub framework: ComplianceFramework,
    pub period: AuditPeriod,
    pub compliance_status: ComplianceStatus,
    pub findings: Vec<ComplianceFinding>,
    pub metrics: ComplianceMetrics,
    pub recommendations: Vec<String>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub generated_by: Uuid,
}

/// Compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    PartiallyCompliant,
    UnderReview,
    NotAssessed,
}

/// Individual compliance finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub id: Uuid,
    pub requirement: String,
    pub control: String,
    pub status: ComplianceStatus,
    pub evidence: Vec<Uuid>, // References to audit events
    pub gaps: Vec<String>,
    pub risk_rating: RiskLevel,
    pub remediation_actions: Vec<String>,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
}

/// Compliance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceMetrics {
    pub total_controls: i32,
    pub compliant_controls: i32,
    pub non_compliant_controls: i32,
    pub compliance_percentage: f64,
    pub high_risk_findings: i32,
    pub medium_risk_findings: i32,
    pub low_risk_findings: i32,
    pub overdue_actions: i32,
}

/// Anomaly detection context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionContext {
    pub tenant_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub time_window: chrono::Duration,
    pub detection_rules: Vec<AnomalyRule>,
    pub baseline_period: Option<AuditPeriod>,
}

/// Security anomaly detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnomaly {
    pub id: Uuid,
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub description: String,
    pub affected_entities: Vec<Uuid>,
    pub detection_rule: String,
    pub baseline_metric: f64,
    pub observed_metric: f64,
    pub confidence_score: f64,
    pub evidence: Vec<Uuid>, // References to audit events
    pub recommendations: Vec<String>,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub status: AnomalyStatus,
}

/// Types of security anomalies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    UnusualAccessPattern,
    SuspiciousDataAccess,
    PrivilegeEscalation,
    DataExfiltration,
    FailedAuthenticationSpike,
    OffHoursActivity,
    GeographicAnomaly,
    VolumeAnomaly,
    BehavioralAnomaly,
}

/// Anomaly severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Anomaly status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyStatus {
    New,
    Investigating,
    Confirmed,
    FalsePositive,
    Resolved,
}

/// Anomaly detection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyRule {
    pub id: Uuid,
    pub name: String,
    pub rule_type: AnomalyType,
    pub threshold: f64,
    pub time_window: chrono::Duration,
    pub conditions: HashMap<String, serde_json::Value>,
    pub is_active: bool,
}

/// Data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub default_retention_days: i32,
    pub category_retention: HashMap<EventCategory, i32>,
    pub risk_level_retention: HashMap<RiskLevel, i32>,
    pub compliance_retention: HashMap<ComplianceFramework, i32>,
    pub archive_to_cold_storage: bool,
    pub delete_after_archive: bool,
}

/// Security audit service implementation
pub struct SecurityAuditService {
    pool: sqlx::PgPool,
    anomaly_detection_enabled: bool,
    compliance_frameworks: Vec<ComplianceFramework>,
}

impl SecurityAuditService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            anomaly_detection_enabled: true,
            compliance_frameworks: vec![
                ComplianceFramework::Gdpr,
                ComplianceFramework::Sox,
                ComplianceFramework::Iso27001,
            ],
        }
    }

    pub fn with_compliance_frameworks(mut self, frameworks: Vec<ComplianceFramework>) -> Self {
        self.compliance_frameworks = frameworks;
        self
    }

    pub fn enable_anomaly_detection(mut self, enabled: bool) -> Self {
        self.anomaly_detection_enabled = enabled;
        self
    }

    /// Calculate risk score based on event characteristics
    fn calculate_risk_score(&self, event: &AuditEvent) -> f64 {
        let mut score: f64 = 0.0;

        // Base score by event type
        score += match event.event_type {
            EventType::SecurityIncident => 8.0,
            EventType::DataModification => 6.0,
            EventType::Authorization => 4.0,
            EventType::DataAccess => 3.0,
            EventType::Authentication => 2.0,
            _ => 1.0,
        };

        // Outcome modifier
        score *= match event.outcome {
            EventOutcome::Failure | EventOutcome::Denied => 2.0,
            EventOutcome::Error => 1.5,
            EventOutcome::Warning => 1.2,
            EventOutcome::Success => 1.0,
        };

        // Time-based modifier (off-hours activity)
        let hour = event.timestamp.time().hour();
        if hour < 6 || hour > 22 {
            score *= 1.3;
        }

        score.min(10.0_f64)
    }

    /// Determine appropriate retention period
    fn calculate_retention_period(&self, event: &AuditEvent) -> chrono::Duration {
        let base_retention = match event.risk_level {
            RiskLevel::Critical => chrono::Duration::days(2555), // 7 years
            RiskLevel::High => chrono::Duration::days(1825),     // 5 years
            RiskLevel::Medium => chrono::Duration::days(1095),   // 3 years
            RiskLevel::Low => chrono::Duration::days(365),       // 1 year
        };

        // Extend for specific event types
        match event.event_type {
            EventType::SecurityIncident => chrono::Duration::days(2555), // 7 years
            EventType::DataModification => chrono::Duration::days(1825), // 5 years
            _ => base_retention,
        }
    }
}

#[async_trait]
impl AuditLogger for SecurityAuditService {
    async fn log_security_event(&self, event: &AuditEvent) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO security_audit_log (
                id, event_type, event_category, user_id, tenant_id, resource_type, resource_id,
                action, outcome, risk_level, event_data, ip_address, user_agent, session_id,
                correlation_id, source_system, timestamp, retention_until
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::inet, $13, $14, $15, $16, $17, $18)
            "#,
        )
        .bind(event.id)
        .bind(serde_json::to_string(&event.event_type).unwrap())
        .bind(serde_json::to_string(&event.event_category).unwrap())
        .bind(event.user_id)
        .bind(event.tenant_id)
        .bind(&event.resource_type)
        .bind(event.resource_id)
        .bind(&event.action)
        .bind(serde_json::to_string(&event.outcome).unwrap())
        .bind(serde_json::to_string(&event.risk_level).unwrap())
        .bind(serde_json::to_value(&event.event_data).unwrap())
        .bind(&event.ip_address)
        .bind(&event.user_agent)
        .bind(&event.session_id)
        .bind(event.correlation_id)
        .bind(&event.source_system)
        .bind(event.timestamp)
        .bind(event.retention_until)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn log_data_access(&self, access: &DataAccessEvent) -> Result<()> {
        let audit_event = AuditEvent {
            id: access.id,
            event_type: EventType::DataAccess,
            event_category: EventCategory::Data,
            user_id: Some(access.user_id),
            tenant_id: access.tenant_id,
            resource_type: Some(access.table_name.clone()),
            resource_id: access.record_id,
            action: serde_json::to_string(&access.access_type).unwrap(),
            outcome: EventOutcome::Success,
            risk_level: match access.data_classification {
                DataClassification::TopSecret | DataClassification::Restricted => RiskLevel::High,
                DataClassification::Confidential => RiskLevel::Medium,
                _ => RiskLevel::Low,
            },
            event_data: serde_json::to_value(access).unwrap().as_object().unwrap().clone().into_iter().collect(),
            ip_address: access.context.get("ip_address").cloned(),
            user_agent: access.context.get("user_agent").cloned(),
            session_id: access.context.get("session_id").cloned(),
            correlation_id: None,
            source_system: "ERP_SYSTEM".to_string(),
            timestamp: access.timestamp,
            retention_until: access.retention_period.map(|period| access.timestamp + period),
        };

        self.log_security_event(&audit_event).await
    }

    async fn log_system_event(&self, event: &SystemEvent) -> Result<()> {
        let audit_event = AuditEvent {
            id: event.id,
            event_type: EventType::SystemConfiguration,
            event_category: EventCategory::System,
            user_id: None,
            tenant_id: Uuid::nil(), // System events are global
            resource_type: Some(event.component.clone()),
            resource_id: None,
            action: serde_json::to_string(&event.event_type).unwrap(),
            outcome: match event.severity {
                EventSeverity::Error | EventSeverity::Critical | EventSeverity::Fatal => EventOutcome::Error,
                EventSeverity::Warning => EventOutcome::Warning,
                EventSeverity::Info => EventOutcome::Success,
            },
            risk_level: match event.severity {
                EventSeverity::Fatal | EventSeverity::Critical => RiskLevel::Critical,
                EventSeverity::Error => RiskLevel::High,
                EventSeverity::Warning => RiskLevel::Medium,
                EventSeverity::Info => RiskLevel::Low,
            },
            event_data: event.details.clone(),
            ip_address: None,
            user_agent: None,
            session_id: None,
            correlation_id: event.correlation_id,
            source_system: "SYSTEM".to_string(),
            timestamp: event.timestamp,
            retention_until: Some(event.timestamp + chrono::Duration::days(365)),
        };

        self.log_security_event(&audit_event).await
    }

    async fn query_audit_trail(&self, _query: &AuditQuery) -> Result<AuditTrail> {
        // This would implement complex audit trail querying
        // For now, return empty results
        Ok(AuditTrail {
            events: vec![],
            total_count: 0,
            summary: AuditSummary {
                total_events: 0,
                events_by_type: HashMap::new(),
                events_by_outcome: HashMap::new(),
                events_by_risk_level: HashMap::new(),
                unique_users: 0,
                unique_resources: 0,
                time_range: (chrono::Utc::now(), chrono::Utc::now()),
            },
            facets: None,
        })
    }

    async fn generate_compliance_report(
        &self,
        framework: &ComplianceFramework,
        period: &AuditPeriod,
    ) -> Result<ComplianceReport> {
        // This would implement compliance report generation
        // For now, return basic report
        Ok(ComplianceReport {
            framework: framework.clone(),
            period: period.clone(),
            compliance_status: ComplianceStatus::UnderReview,
            findings: vec![],
            metrics: ComplianceMetrics {
                total_controls: 0,
                compliant_controls: 0,
                non_compliant_controls: 0,
                compliance_percentage: 0.0,
                high_risk_findings: 0,
                medium_risk_findings: 0,
                low_risk_findings: 0,
                overdue_actions: 0,
            },
            recommendations: vec![],
            generated_at: chrono::Utc::now(),
            generated_by: Uuid::nil(),
        })
    }

    async fn detect_anomalies(&self, _context: &AnomalyDetectionContext) -> Result<Vec<SecurityAnomaly>> {
        if !self.anomaly_detection_enabled {
            return Ok(vec![]);
        }

        // This would implement ML-based anomaly detection
        // For now, return empty results
        Ok(vec![])
    }

    async fn archive_logs(&self, _retention_policy: &RetentionPolicy) -> Result<u64> {
        let archived_count = sqlx::query(
            "SELECT cleanup_old_audit_logs() as count"
        )
        .fetch_one(&self.pool)
        .await?
        .get::<Option<i64>, _>("count")
        .unwrap_or(0) as u64;

        Ok(archived_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_risk_score_calculation() {
        let service = SecurityAuditService::new(sqlx::Pool::connect("").await.unwrap());

        let event = AuditEvent {
            id: Uuid::new_v4(),
            event_type: EventType::SecurityIncident,
            event_category: EventCategory::Security,
            user_id: Some(Uuid::new_v4()),
            tenant_id: Uuid::new_v4(),
            resource_type: None,
            resource_id: None,
            action: "security_breach".to_string(),
            outcome: EventOutcome::Failure,
            risk_level: RiskLevel::Critical,
            event_data: HashMap::new(),
            ip_address: None,
            user_agent: None,
            session_id: None,
            correlation_id: None,
            source_system: "TEST".to_string(),
            timestamp: chrono::Utc::now(),
            retention_until: None,
        };

        let risk_score = service.calculate_risk_score(&event);
        assert!(risk_score > 8.0); // High risk event
    }

    #[tokio::test]
    async fn test_retention_period_calculation() {
        let service = SecurityAuditService::new(sqlx::Pool::connect("").await.unwrap());

        let critical_event = AuditEvent {
            id: Uuid::new_v4(),
            event_type: EventType::SecurityIncident,
            event_category: EventCategory::Security,
            user_id: None,
            tenant_id: Uuid::new_v4(),
            resource_type: None,
            resource_id: None,
            action: "test".to_string(),
            outcome: EventOutcome::Success,
            risk_level: RiskLevel::Critical,
            event_data: HashMap::new(),
            ip_address: None,
            user_agent: None,
            session_id: None,
            correlation_id: None,
            source_system: "TEST".to_string(),
            timestamp: chrono::Utc::now(),
            retention_until: None,
        };

        let retention = service.calculate_retention_period(&critical_event);
        assert_eq!(retention.num_days(), 2555); // 7 years for critical security incidents
    }
}