//! Customer Aggregate for CQRS/Event Sourcing
//!
//! This module implements the Customer aggregate root that encapsulates
//! business logic and maintains consistency through event sourcing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::customer::events::CustomerEvent;
use crate::customer::model::*;
use crate::error::{MasterDataError, Result};
use crate::types::*;

/// Customer aggregate root implementing event sourcing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerAggregate {
    // Identity
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub version: i64,
    pub is_deleted: bool,

    // Core customer data
    pub customer_number: String,
    pub legal_name: String,
    pub customer_type: CustomerType,
    pub lifecycle_stage: CustomerLifecycleStage,
    pub status: EntityStatus,
    pub credit_status: CreditStatus,

    // Financial information
    pub currency_code: String,
    pub credit_limit: Option<rust_decimal::Decimal>,
    pub payment_terms: Option<PaymentTerms>,
    pub tax_exempt: bool,
    pub tax_numbers: HashMap<String, String>,

    // Hierarchy and relationships
    pub parent_customer_id: Option<Uuid>,
    pub corporate_group_id: Option<Uuid>,
    pub hierarchy_level: i16,

    // Commercial information
    pub sales_representative_id: Option<Uuid>,
    pub account_manager_id: Option<Uuid>,
    pub acquisition_channel: Option<AcquisitionChannel>,

    // Compliance and risk
    pub compliance_status: ComplianceStatus,
    pub kyc_status: KycStatus,
    pub aml_risk_rating: RiskRating,

    // Addresses (reference IDs)
    pub primary_address_id: Option<Uuid>,
    pub billing_address_id: Option<Uuid>,
    pub shipping_address_ids: Vec<Uuid>,

    // Contacts (reference IDs)
    pub primary_contact_id: Option<Uuid>,
    pub contact_ids: Vec<Uuid>,

    // Performance metrics
    pub customer_lifetime_value: Option<rust_decimal::Decimal>,
    pub total_revenue: Option<rust_decimal::Decimal>,
    pub total_orders: Option<i64>,
    pub last_order_date: Option<DateTime<Utc>>,
    pub churn_probability: Option<rust_decimal::Decimal>,

    // Behavioral insights
    pub preferred_channels: Vec<String>,
    pub customer_segments: Vec<String>,
    pub propensity_to_buy: Option<rust_decimal::Decimal>,

    // Audit information
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub modified_by: Uuid,
    pub modified_at: DateTime<Utc>,

    // Uncommitted events for persistence
    #[serde(skip)]
    uncommitted_events: Vec<CustomerEvent>,
}

impl CustomerAggregate {
    /// Create a new customer aggregate
    pub fn create(
        tenant_id: Uuid,
        customer_number: String,
        legal_name: String,
        customer_type: CustomerType,
        created_by: Uuid,
    ) -> Result<Self> {
        let customer_id = Uuid::new_v4();
        let now = Utc::now();

        // Business rule validation
        if legal_name.trim().is_empty() {
            return Err(MasterDataError::ValidationError {
                field: "legal_name".to_string(),
                message: "Legal name cannot be empty".to_string(),
            });
        }

        if customer_number.trim().is_empty() {
            return Err(MasterDataError::ValidationError {
                field: "customer_number".to_string(),
                message: "Customer number cannot be empty".to_string(),
            });
        }

        let event = CustomerEvent::CustomerCreated {
            customer_id,
            tenant_id,
            customer_number: customer_number.clone(),
            legal_name: legal_name.clone(),
            customer_type,
            created_by,
            created_at: now,
        };

        let mut aggregate = Self::default_with_id(customer_id, tenant_id);
        aggregate.apply_event(&event);
        aggregate.uncommitted_events.push(event);

        Ok(aggregate)
    }

    /// Load aggregate from historical events
    pub fn from_events(events: Vec<CustomerEvent>) -> Result<Self> {
        if events.is_empty() {
            return Err(MasterDataError::ValidationError {
                field: "events".to_string(),
                message: "Cannot create aggregate from empty event stream".to_string(),
            });
        }

        let first_event = &events[0];
        let customer_id = first_event.customer_id();

        // Extract tenant_id from first event
        let tenant_id = match first_event {
            CustomerEvent::CustomerCreated { tenant_id, .. } => *tenant_id,
            _ => {
                return Err(MasterDataError::ValidationError {
                    field: "events".to_string(),
                    message: "First event must be CustomerCreated".to_string(),
                });
            }
        };

        let mut aggregate = Self::default_with_id(customer_id, tenant_id);

        for event in events {
            aggregate.apply_event(&event);
            aggregate.version += 1;
        }

        Ok(aggregate)
    }

    /// Update customer basic information
    pub fn update_information(
        &mut self,
        new_legal_name: Option<String>,
        new_customer_type: Option<CustomerType>,
        updated_by: Uuid,
    ) -> Result<()> {
        if self.is_deleted {
            return Err(MasterDataError::ValidationError {
                field: "customer".to_string(),
                message: "Cannot update deleted customer".to_string(),
            });
        }

        let mut has_changes = false;
        let mut previous_legal_name = None;
        let mut previous_customer_type = None;

        if let Some(ref name) = new_legal_name {
            if name.trim().is_empty() {
                return Err(MasterDataError::ValidationError {
                    field: "legal_name".to_string(),
                    message: "Legal name cannot be empty".to_string(),
                });
            }
            if name != &self.legal_name {
                previous_legal_name = Some(self.legal_name.clone());
                has_changes = true;
            }
        }

        if let Some(ref customer_type) = new_customer_type {
            if *customer_type != self.customer_type {
                // Business rule: Cannot change type if customer has orders
                if self.total_orders.unwrap_or(0) > 0 {
                    return Err(MasterDataError::ValidationError {
                        field: "customer_type".to_string(),
                        message: "Cannot change customer type for customers with existing orders"
                            .to_string(),
                    });
                }
                previous_customer_type = Some(self.customer_type.clone());
                has_changes = true;
            }
        }

        if has_changes {
            let event = CustomerEvent::CustomerInformationUpdated {
                customer_id: self.id,
                previous_legal_name,
                new_legal_name,
                previous_customer_type,
                new_customer_type,
                updated_by,
                updated_at: Utc::now(),
            };

            self.apply_event(&event);
            self.uncommitted_events.push(event);
        }

        Ok(())
    }

    /// Change lifecycle stage with business rules
    pub fn change_lifecycle_stage(
        &mut self,
        new_stage: CustomerLifecycleStage,
        reason: Option<String>,
        changed_by: Uuid,
    ) -> Result<()> {
        if self.is_deleted {
            return Err(MasterDataError::ValidationError {
                field: "customer".to_string(),
                message: "Cannot change lifecycle stage of deleted customer".to_string(),
            });
        }

        // Validate transition rules
        self.validate_lifecycle_transition(&self.lifecycle_stage, &new_stage)?;

        if new_stage != self.lifecycle_stage {
            let event = CustomerEvent::LifecycleStageChanged {
                customer_id: self.id,
                previous_stage: self.lifecycle_stage.clone(),
                new_stage,
                reason,
                changed_by,
                changed_at: Utc::now(),
            };

            self.apply_event(&event);
            self.uncommitted_events.push(event);
        }

        Ok(())
    }

    /// Update credit status and limit
    pub fn update_credit_status(
        &mut self,
        new_status: CreditStatus,
        new_limit: Option<rust_decimal::Decimal>,
        reason: String,
        approved_by: Uuid,
    ) -> Result<()> {
        if self.is_deleted {
            return Err(MasterDataError::ValidationError {
                field: "customer".to_string(),
                message: "Cannot update credit status of deleted customer".to_string(),
            });
        }

        // Business rule: Credit limit increases > 50% require special approval
        if let (Some(current), Some(new)) = (self.credit_limit, new_limit) {
            if new > current {
                let increase_percentage = if current > rust_decimal::Decimal::ZERO {
                    ((new - current) / current) * rust_decimal::Decimal::from(100)
                } else {
                    rust_decimal::Decimal::from(100)
                };

                if increase_percentage > rust_decimal::Decimal::from(50) {
                    return Err(MasterDataError::ValidationError {
                        field: "credit_limit".to_string(),
                        message: format!(
                            "Credit limit increase of {:.2}% exceeds 50% threshold",
                            increase_percentage
                        ),
                    });
                }
            }
        }

        let event = CustomerEvent::CreditStatusChanged {
            customer_id: self.id,
            previous_status: self.credit_status.clone(),
            new_status,
            previous_limit: self.credit_limit,
            new_limit,
            reason,
            approved_by,
            changed_at: Utc::now(),
        };

        self.apply_event(&event);
        self.uncommitted_events.push(event);

        Ok(())
    }

    /// Update performance metrics
    pub fn update_performance_metrics(
        &mut self,
        total_revenue: Option<rust_decimal::Decimal>,
        total_orders: Option<i64>,
        last_order_date: Option<DateTime<Utc>>,
        customer_lifetime_value: Option<rust_decimal::Decimal>,
        calculation_method: String,
    ) -> Result<()> {
        let event = CustomerEvent::PerformanceMetricsCalculated {
            customer_id: self.id,
            total_revenue,
            total_orders,
            last_order_date,
            customer_lifetime_value,
            calculated_at: Utc::now(),
            calculation_method,
        };

        self.apply_event(&event);
        self.uncommitted_events.push(event);

        Ok(())
    }

    /// Soft delete the customer
    pub fn soft_delete(&mut self, reason: String, deleted_by: Uuid) -> Result<()> {
        if self.is_deleted {
            return Err(MasterDataError::ValidationError {
                field: "customer".to_string(),
                message: "Customer is already deleted".to_string(),
            });
        }

        // Business rule: Cannot delete VIP customers without approval
        if self.lifecycle_stage == CustomerLifecycleStage::VipCustomer {
            return Err(MasterDataError::ValidationError {
                field: "lifecycle_stage".to_string(),
                message: "VIP customers require special approval for deletion".to_string(),
            });
        }

        let event = CustomerEvent::CustomerSoftDeleted {
            customer_id: self.id,
            reason,
            deleted_by,
            deleted_at: Utc::now(),
        };

        self.apply_event(&event);
        self.uncommitted_events.push(event);

        Ok(())
    }

    /// Get uncommitted events for persistence
    pub fn uncommitted_events(&self) -> &[CustomerEvent] {
        &self.uncommitted_events
    }

    /// Mark events as committed
    pub fn mark_events_committed(&mut self) {
        self.uncommitted_events.clear();
    }

    /// Apply an event to update the aggregate state
    fn apply_event(&mut self, event: &CustomerEvent) {
        match event {
            CustomerEvent::CustomerCreated {
                customer_id,
                tenant_id,
                customer_number,
                legal_name,
                customer_type,
                created_by,
                created_at,
            } => {
                self.id = *customer_id;
                self.tenant_id = *tenant_id;
                self.customer_number = customer_number.clone();
                self.legal_name = legal_name.clone();
                self.customer_type = customer_type.clone();
                self.created_by = *created_by;
                self.created_at = *created_at;
                self.modified_by = *created_by;
                self.modified_at = *created_at;
            }

            CustomerEvent::CustomerInformationUpdated {
                new_legal_name,
                new_customer_type,
                updated_by,
                updated_at,
                ..
            } => {
                if let Some(name) = new_legal_name {
                    self.legal_name = name.clone();
                }
                if let Some(customer_type) = new_customer_type {
                    self.customer_type = customer_type.clone();
                }
                self.modified_by = *updated_by;
                self.modified_at = *updated_at;
            }

            CustomerEvent::LifecycleStageChanged {
                new_stage,
                changed_by,
                changed_at,
                ..
            } => {
                self.lifecycle_stage = new_stage.clone();
                self.modified_by = *changed_by;
                self.modified_at = *changed_at;
            }

            CustomerEvent::CreditStatusChanged {
                new_status,
                new_limit,
                approved_by,
                changed_at,
                ..
            } => {
                self.credit_status = new_status.clone();
                self.credit_limit = *new_limit;
                self.modified_by = *approved_by;
                self.modified_at = *changed_at;
            }

            CustomerEvent::PerformanceMetricsCalculated {
                total_revenue,
                total_orders,
                last_order_date,
                customer_lifetime_value,
                calculated_at,
                ..
            } => {
                self.total_revenue = *total_revenue;
                self.total_orders = *total_orders;
                self.last_order_date = *last_order_date;
                self.customer_lifetime_value = *customer_lifetime_value;
                self.modified_at = *calculated_at;
            }

            CustomerEvent::CustomerSoftDeleted { deleted_at, .. } => {
                self.is_deleted = true;
                self.modified_at = *deleted_at;
            }

            CustomerEvent::CustomerRestored { restored_at, .. } => {
                self.is_deleted = false;
                self.modified_at = *restored_at;
            }

            // Add other event handlers as needed
            _ => {
                // For events that don't directly modify the core aggregate state
                // (like address/contact changes), we just update the modified timestamp
                self.modified_at = event.occurred_at();
            }
        }
    }

    /// Validate lifecycle stage transitions
    fn validate_lifecycle_transition(
        &self,
        current: &CustomerLifecycleStage,
        new: &CustomerLifecycleStage,
    ) -> Result<()> {
        use CustomerLifecycleStage::*;

        let valid_transitions = match current {
            Lead => vec![Prospect, ProspectCustomer, FormerCustomer],
            Prospect => vec![NewCustomer, ProspectCustomer, FormerCustomer],
            ProspectCustomer => vec![NewCustomer, ActiveCustomer, Active, FormerCustomer],
            NewCustomer => vec![ActiveCustomer, Active, InactiveCustomer, FormerCustomer],
            Active => vec![ActiveCustomer, VipCustomer, AtRiskCustomer, InactiveCustomer, Churned, FormerCustomer],
            ActiveCustomer => vec![Active, VipCustomer, AtRiskCustomer, InactiveCustomer, Churned, FormerCustomer],
            VipCustomer => vec![ActiveCustomer, Active, AtRiskCustomer, InactiveCustomer, Churned, FormerCustomer],
            AtRiskCustomer => {
                vec![ActiveCustomer, Active, WonBackCustomer, InactiveCustomer, Churned, FormerCustomer]
            }
            InactiveCustomer => vec![WonBackCustomer, Churned, FormerCustomer],
            Churned => vec![WonBackCustomer, FormerCustomer],
            WonBackCustomer => vec![
                ActiveCustomer,
                Active,
                VipCustomer,
                AtRiskCustomer,
                InactiveCustomer,
                FormerCustomer,
            ],
            FormerCustomer => vec![WonBackCustomer],
        };

        if !valid_transitions.contains(new) {
            return Err(MasterDataError::ValidationError {
                field: "lifecycle_stage".to_string(),
                message: format!("Invalid lifecycle stage transition from {:?} to {:?}", current, new),
            });
        }

        Ok(())
    }

    /// Create a default aggregate with the given ID
    fn default_with_id(id: Uuid, tenant_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id,
            tenant_id,
            version: 0,
            is_deleted: false,
            customer_number: String::new(),
            legal_name: String::new(),
            customer_type: CustomerType::B2b,
            lifecycle_stage: CustomerLifecycleStage::Prospect,
            status: EntityStatus::Active,
            credit_status: CreditStatus::Good,
            currency_code: "USD".to_string(),
            credit_limit: None,
            payment_terms: None,
            tax_exempt: false,
            tax_numbers: HashMap::new(),
            parent_customer_id: None,
            corporate_group_id: None,
            hierarchy_level: 1,
            sales_representative_id: None,
            account_manager_id: None,
            acquisition_channel: None,
            compliance_status: ComplianceStatus::Unknown,
            kyc_status: KycStatus::NotStarted,
            aml_risk_rating: RiskRating::Medium,
            primary_address_id: None,
            billing_address_id: None,
            shipping_address_ids: Vec::new(),
            primary_contact_id: None,
            contact_ids: Vec::new(),
            customer_lifetime_value: None,
            total_revenue: None,
            total_orders: None,
            last_order_date: None,
            churn_probability: None,
            preferred_channels: Vec::new(),
            customer_segments: Vec::new(),
            propensity_to_buy: None,
            created_by: Uuid::nil(),
            created_at: now,
            modified_by: Uuid::nil(),
            modified_at: now,
            uncommitted_events: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_customer_aggregate() {
        let tenant_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();

        let aggregate = CustomerAggregate::create(
            tenant_id,
            "CUST001".to_string(),
            "ACME Corp".to_string(),
            CustomerType::B2b,
            created_by,
        )
        .unwrap();

        assert_eq!(aggregate.tenant_id, tenant_id);
        assert_eq!(aggregate.customer_number, "CUST001");
        assert_eq!(aggregate.legal_name, "ACME Corp");
        assert_eq!(aggregate.customer_type, CustomerType::B2b);
        assert_eq!(aggregate.created_by, created_by);
        assert_eq!(aggregate.uncommitted_events.len(), 1);
        assert!(!aggregate.is_deleted);
    }

    #[test]
    fn test_lifecycle_stage_transitions() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let mut aggregate = CustomerAggregate::create(
            tenant_id,
            "CUST001".to_string(),
            "ACME Corp".to_string(),
            CustomerType::B2b,
            user_id,
        )
        .unwrap();

        // Valid transition: Prospect -> NewCustomer
        aggregate
            .change_lifecycle_stage(
                CustomerLifecycleStage::NewCustomer,
                Some("First purchase completed".to_string()),
                user_id,
            )
            .unwrap();

        assert_eq!(aggregate.lifecycle_stage, CustomerLifecycleStage::NewCustomer);

        // Invalid transition: NewCustomer -> Lead (should fail)
        let result = aggregate.change_lifecycle_stage(
            CustomerLifecycleStage::Lead,
            Some("Invalid transition".to_string()),
            user_id,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_credit_limit_validation() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let mut aggregate = CustomerAggregate::create(
            tenant_id,
            "CUST001".to_string(),
            "ACME Corp".to_string(),
            CustomerType::B2b,
            user_id,
        )
        .unwrap();

        // Set initial credit limit
        aggregate
            .update_credit_status(
                CreditStatus::Good,
                Some(rust_decimal::Decimal::from(10000)),
                "Initial credit assessment".to_string(),
                user_id,
            )
            .unwrap();

        // Try to increase by more than 50% (should fail)
        let result = aggregate.update_credit_status(
            CreditStatus::Excellent,
            Some(rust_decimal::Decimal::from(20000)), // 100% increase
            "Credit limit increase".to_string(),
            user_id,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_soft_delete_vip_customer() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let mut aggregate = CustomerAggregate::create(
            tenant_id,
            "CUST001".to_string(),
            "ACME Corp".to_string(),
            CustomerType::B2b,
            user_id,
        )
        .unwrap();

        // Change to VIP customer
        aggregate
            .change_lifecycle_stage(
                CustomerLifecycleStage::VipCustomer,
                Some("High value customer".to_string()),
                user_id,
            )
            .unwrap();

        // Try to delete VIP customer (should fail)
        let result = aggregate.soft_delete("Business closure".to_string(), user_id);

        assert!(result.is_err());
        assert!(!aggregate.is_deleted);
    }
}