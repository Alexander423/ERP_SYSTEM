//! Customer Domain Events for Event Sourcing
//!
//! This module defines all domain events that can occur within the customer aggregate.
//! Events represent facts about what has happened and are immutable once created.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::customer::model::*;
use crate::types::*;

/// All possible events in the customer domain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type", content = "data")]
pub enum CustomerEvent {
    /// Customer was created
    CustomerCreated {
        customer_id: Uuid,
        tenant_id: Uuid,
        customer_number: String,
        legal_name: String,
        customer_type: CustomerType,
        created_by: Uuid,
        created_at: DateTime<Utc>,
    },

    /// Customer basic information was updated
    CustomerInformationUpdated {
        customer_id: Uuid,
        previous_legal_name: Option<String>,
        new_legal_name: Option<String>,
        previous_customer_type: Option<CustomerType>,
        new_customer_type: Option<CustomerType>,
        updated_by: Uuid,
        updated_at: DateTime<Utc>,
    },

    /// Customer lifecycle stage changed
    LifecycleStageChanged {
        customer_id: Uuid,
        previous_stage: CustomerLifecycleStage,
        new_stage: CustomerLifecycleStage,
        reason: Option<String>,
        changed_by: Uuid,
        changed_at: DateTime<Utc>,
    },

    /// Customer credit status changed
    CreditStatusChanged {
        customer_id: Uuid,
        previous_status: CreditStatus,
        new_status: CreditStatus,
        previous_limit: Option<rust_decimal::Decimal>,
        new_limit: Option<rust_decimal::Decimal>,
        reason: String,
        approved_by: Uuid,
        changed_at: DateTime<Utc>,
    },

    /// Address was added to customer
    AddressAdded {
        customer_id: Uuid,
        address_id: Uuid,
        address_type: AddressType,
        street_line_1: String,
        city: String,
        country_code: String,
        is_primary: bool,
        added_by: Uuid,
        added_at: DateTime<Utc>,
    },

    /// Address was updated
    AddressUpdated {
        customer_id: Uuid,
        address_id: Uuid,
        address_type: AddressType,
        updated_fields: Vec<String>,
        updated_by: Uuid,
        updated_at: DateTime<Utc>,
    },

    /// Address was removed
    AddressRemoved {
        customer_id: Uuid,
        address_id: Uuid,
        address_type: AddressType,
        reason: Option<String>,
        removed_by: Uuid,
        removed_at: DateTime<Utc>,
    },

    /// Contact was added to customer
    ContactAdded {
        customer_id: Uuid,
        contact_id: Uuid,
        contact_type: ContactType,
        first_name: String,
        last_name: String,
        email: Option<String>,
        phone: Option<String>,
        is_primary: bool,
        added_by: Uuid,
        added_at: DateTime<Utc>,
    },

    /// Contact information was updated
    ContactUpdated {
        customer_id: Uuid,
        contact_id: Uuid,
        updated_fields: Vec<String>,
        updated_by: Uuid,
        updated_at: DateTime<Utc>,
    },

    /// Contact was removed
    ContactRemoved {
        customer_id: Uuid,
        contact_id: Uuid,
        contact_type: ContactType,
        reason: Option<String>,
        removed_by: Uuid,
        removed_at: DateTime<Utc>,
    },

    /// Customer was assigned to a sales representative
    SalesRepresentativeAssigned {
        customer_id: Uuid,
        previous_rep_id: Option<Uuid>,
        new_rep_id: Uuid,
        effective_date: DateTime<Utc>,
        assigned_by: Uuid,
        assigned_at: DateTime<Utc>,
    },

    /// Customer performance metrics were calculated
    PerformanceMetricsCalculated {
        customer_id: Uuid,
        total_revenue: Option<rust_decimal::Decimal>,
        total_orders: Option<i64>,
        last_order_date: Option<DateTime<Utc>>,
        customer_lifetime_value: Option<rust_decimal::Decimal>,
        calculated_at: DateTime<Utc>,
        calculation_method: String,
    },

    /// Customer behavioral data was updated
    BehavioralDataUpdated {
        customer_id: Uuid,
        propensity_to_buy: Option<rust_decimal::Decimal>,
        churn_probability: Option<rust_decimal::Decimal>,
        preferred_channels: Vec<String>,
        data_sources: Vec<String>,
        confidence_score: Option<rust_decimal::Decimal>,
        updated_at: DateTime<Utc>,
    },

    /// Customer compliance status changed
    ComplianceStatusChanged {
        customer_id: Uuid,
        previous_status: ComplianceStatus,
        new_status: ComplianceStatus,
        kyc_status: KycStatus,
        documents_provided: Vec<String>,
        reviewed_by: Uuid,
        changed_at: DateTime<Utc>,
    },

    /// Customer was soft deleted
    CustomerSoftDeleted {
        customer_id: Uuid,
        reason: String,
        deleted_by: Uuid,
        deleted_at: DateTime<Utc>,
    },

    /// Customer was restored from soft delete
    CustomerRestored {
        customer_id: Uuid,
        reason: String,
        restored_by: Uuid,
        restored_at: DateTime<Utc>,
    },

    /// Customer hierarchy was changed
    HierarchyChanged {
        customer_id: Uuid,
        previous_parent_id: Option<Uuid>,
        new_parent_id: Option<Uuid>,
        hierarchy_level: i16,
        changed_by: Uuid,
        changed_at: DateTime<Utc>,
    },

    /// Customer segmentation was updated
    SegmentationUpdated {
        customer_id: Uuid,
        previous_segments: Vec<String>,
        new_segments: Vec<String>,
        segmentation_algorithm: String,
        confidence_score: rust_decimal::Decimal,
        updated_at: DateTime<Utc>,
    },

    /// Customer risk rating was updated
    RiskRatingUpdated {
        customer_id: Uuid,
        previous_rating: RiskRating,
        new_rating: RiskRating,
        assessment_factors: Vec<String>,
        assessed_by: Uuid,
        assessed_at: DateTime<Utc>,
    },
}

/// Event metadata for audit and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub event_version: u32,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub sequence_number: i64,
    pub occurred_at: DateTime<Utc>,
    pub recorded_at: DateTime<Utc>,
    pub causation_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub tenant_id: Uuid,
}

/// Complete event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerEventWithMetadata {
    pub metadata: EventMetadata,
    pub event: CustomerEvent,
}

impl CustomerEvent {
    /// Get the customer ID from any event
    pub fn customer_id(&self) -> Uuid {
        match self {
            CustomerEvent::CustomerCreated { customer_id, .. } => *customer_id,
            CustomerEvent::CustomerInformationUpdated { customer_id, .. } => *customer_id,
            CustomerEvent::LifecycleStageChanged { customer_id, .. } => *customer_id,
            CustomerEvent::CreditStatusChanged { customer_id, .. } => *customer_id,
            CustomerEvent::AddressAdded { customer_id, .. } => *customer_id,
            CustomerEvent::AddressUpdated { customer_id, .. } => *customer_id,
            CustomerEvent::AddressRemoved { customer_id, .. } => *customer_id,
            CustomerEvent::ContactAdded { customer_id, .. } => *customer_id,
            CustomerEvent::ContactUpdated { customer_id, .. } => *customer_id,
            CustomerEvent::ContactRemoved { customer_id, .. } => *customer_id,
            CustomerEvent::SalesRepresentativeAssigned { customer_id, .. } => *customer_id,
            CustomerEvent::PerformanceMetricsCalculated { customer_id, .. } => *customer_id,
            CustomerEvent::BehavioralDataUpdated { customer_id, .. } => *customer_id,
            CustomerEvent::ComplianceStatusChanged { customer_id, .. } => *customer_id,
            CustomerEvent::CustomerSoftDeleted { customer_id, .. } => *customer_id,
            CustomerEvent::CustomerRestored { customer_id, .. } => *customer_id,
            CustomerEvent::HierarchyChanged { customer_id, .. } => *customer_id,
            CustomerEvent::SegmentationUpdated { customer_id, .. } => *customer_id,
            CustomerEvent::RiskRatingUpdated { customer_id, .. } => *customer_id,
        }
    }

    /// Get the timestamp when the event occurred
    pub fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            CustomerEvent::CustomerCreated { created_at, .. } => *created_at,
            CustomerEvent::CustomerInformationUpdated { updated_at, .. } => *updated_at,
            CustomerEvent::LifecycleStageChanged { changed_at, .. } => *changed_at,
            CustomerEvent::CreditStatusChanged { changed_at, .. } => *changed_at,
            CustomerEvent::AddressAdded { added_at, .. } => *added_at,
            CustomerEvent::AddressUpdated { updated_at, .. } => *updated_at,
            CustomerEvent::AddressRemoved { removed_at, .. } => *removed_at,
            CustomerEvent::ContactAdded { added_at, .. } => *added_at,
            CustomerEvent::ContactUpdated { updated_at, .. } => *updated_at,
            CustomerEvent::ContactRemoved { removed_at, .. } => *removed_at,
            CustomerEvent::SalesRepresentativeAssigned { assigned_at, .. } => *assigned_at,
            CustomerEvent::PerformanceMetricsCalculated { calculated_at, .. } => *calculated_at,
            CustomerEvent::BehavioralDataUpdated { updated_at, .. } => *updated_at,
            CustomerEvent::ComplianceStatusChanged { changed_at, .. } => *changed_at,
            CustomerEvent::CustomerSoftDeleted { deleted_at, .. } => *deleted_at,
            CustomerEvent::CustomerRestored { restored_at, .. } => *restored_at,
            CustomerEvent::HierarchyChanged { changed_at, .. } => *changed_at,
            CustomerEvent::SegmentationUpdated { updated_at, .. } => *updated_at,
            CustomerEvent::RiskRatingUpdated { assessed_at, .. } => *assessed_at,
        }
    }

    /// Get the event type as a string for categorization
    pub fn event_type(&self) -> &'static str {
        match self {
            CustomerEvent::CustomerCreated { .. } => "customer_created",
            CustomerEvent::CustomerInformationUpdated { .. } => "customer_information_updated",
            CustomerEvent::LifecycleStageChanged { .. } => "lifecycle_stage_changed",
            CustomerEvent::CreditStatusChanged { .. } => "credit_status_changed",
            CustomerEvent::AddressAdded { .. } => "address_added",
            CustomerEvent::AddressUpdated { .. } => "address_updated",
            CustomerEvent::AddressRemoved { .. } => "address_removed",
            CustomerEvent::ContactAdded { .. } => "contact_added",
            CustomerEvent::ContactUpdated { .. } => "contact_updated",
            CustomerEvent::ContactRemoved { .. } => "contact_removed",
            CustomerEvent::SalesRepresentativeAssigned { .. } => "sales_representative_assigned",
            CustomerEvent::PerformanceMetricsCalculated { .. } => "performance_metrics_calculated",
            CustomerEvent::BehavioralDataUpdated { .. } => "behavioral_data_updated",
            CustomerEvent::ComplianceStatusChanged { .. } => "compliance_status_changed",
            CustomerEvent::CustomerSoftDeleted { .. } => "customer_soft_deleted",
            CustomerEvent::CustomerRestored { .. } => "customer_restored",
            CustomerEvent::HierarchyChanged { .. } => "hierarchy_changed",
            CustomerEvent::SegmentationUpdated { .. } => "segmentation_updated",
            CustomerEvent::RiskRatingUpdated { .. } => "risk_rating_updated",
        }
    }

    /// Check if this is a high-impact business event
    pub fn is_high_impact(&self) -> bool {
        matches!(
            self,
            CustomerEvent::CustomerCreated { .. }
                | CustomerEvent::CreditStatusChanged { .. }
                | CustomerEvent::LifecycleStageChanged { .. }
                | CustomerEvent::ComplianceStatusChanged { .. }
                | CustomerEvent::CustomerSoftDeleted { .. }
                | CustomerEvent::RiskRatingUpdated { .. }
        )
    }
}

impl EventMetadata {
    pub fn new(
        aggregate_id: Uuid,
        tenant_id: Uuid,
        sequence_number: i64,
        user_id: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            event_id: Uuid::new_v4(),
            event_version: 1,
            aggregate_id,
            aggregate_type: "customer".to_string(),
            sequence_number,
            occurred_at: now,
            recorded_at: now,
            causation_id: None,
            correlation_id: None,
            user_id,
            tenant_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_event_serialization() {
        let event = CustomerEvent::CustomerCreated {
            customer_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            customer_number: "CUST001".to_string(),
            legal_name: "ACME Corp".to_string(),
            customer_type: CustomerType::B2b,
            created_by: Uuid::new_v4(),
            created_at: Utc::now(),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: CustomerEvent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_event_customer_id_extraction() {
        let customer_id = Uuid::new_v4();
        let event = CustomerEvent::LifecycleStageChanged {
            customer_id,
            previous_stage: CustomerLifecycleStage::Prospect,
            new_stage: CustomerLifecycleStage::ActiveCustomer,
            reason: Some("Completed first purchase".to_string()),
            changed_by: Uuid::new_v4(),
            changed_at: Utc::now(),
        };

        assert_eq!(event.customer_id(), customer_id);
        assert_eq!(event.event_type(), "lifecycle_stage_changed");
        assert!(event.is_high_impact());
    }
}