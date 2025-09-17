use async_trait::async_trait;
use uuid::Uuid;
use validator::Validate;

use crate::customer::model::*;
use crate::customer::repository::CustomerRepository;
use crate::error::{MasterDataError, Result};
use erp_core::TenantContext;

/// Business rules and validation for customer operations
#[async_trait]
pub trait CustomerService: Send + Sync {
    /// Create a new customer with full business validation
    async fn create_customer(&self, request: CreateCustomerRequest, created_by: Uuid) -> Result<Customer>;

    /// Update an existing customer with business rule validation
    async fn update_customer(&self, id: Uuid, request: UpdateCustomerRequest, modified_by: Uuid) -> Result<Customer>;

    /// Get customer by ID with business context
    async fn get_customer(&self, id: Uuid) -> Result<Option<Customer>>;

    /// Search customers with business rule filtering
    async fn search_customers(&self, criteria: CustomerSearchCriteria) -> Result<CustomerSearchResponse>;

    /// Soft delete customer with dependency validation
    async fn delete_customer(&self, id: Uuid, deleted_by: Uuid) -> Result<()>;

    /// Validate customer credit limit increase
    async fn validate_credit_limit_increase(&self, customer_id: Uuid, new_limit: rust_decimal::Decimal) -> Result<()>;

    /// Update customer lifecycle stage with business rules
    async fn update_lifecycle_stage(&self, customer_id: Uuid, new_stage: CustomerLifecycleStage, updated_by: Uuid) -> Result<()>;

    /// Calculate customer performance metrics
    async fn calculate_performance_metrics(&self, customer_id: Uuid) -> Result<CustomerPerformanceMetrics>;

    /// Generate customer number based on business rules
    async fn generate_customer_number(&self, customer_type: CustomerType) -> Result<String>;

    /// Validate customer hierarchy constraints
    async fn validate_hierarchy(&self, customer_id: Option<Uuid>, parent_id: Option<Uuid>) -> Result<()>;
}

/// Default implementation of customer service with comprehensive business logic
pub struct DefaultCustomerService {
    repository: Box<dyn CustomerRepository>,
    tenant_context: TenantContext,
}

impl DefaultCustomerService {
    pub fn new(repository: Box<dyn CustomerRepository>, tenant_context: TenantContext) -> Self {
        Self {
            repository,
            tenant_context,
        }
    }
}

#[async_trait]
impl CustomerService for DefaultCustomerService {
    async fn create_customer(&self, request: CreateCustomerRequest, created_by: Uuid) -> Result<Customer> {
        // 1. Input validation
        request.validate()
            .map_err(|e| MasterDataError::ValidationError {
                field: "request".to_string(),
                message: e.to_string(),
            })?;

        // 2. Business rule validation
        self.validate_create_business_rules(&request).await?;

        // Clone request early to avoid partial move issues
        let mut request_with_number = request.clone();

        // 3. Generate customer number if not provided
        let customer_number = if request.customer_number.is_none() {
            self.generate_customer_number(request.customer_type).await?
        } else {
            request.customer_number.unwrap()
        };

        // 4. Validate customer number uniqueness
        if !self.repository.is_customer_number_available(&customer_number).await? {
            return Err(MasterDataError::DuplicateCustomerNumber {
                number: customer_number,
            });
        }

        // 5. Validate hierarchy constraints
        self.validate_hierarchy(None, request.parent_customer_id).await?;

        // 6. Set customer number
        request_with_number.customer_number = Some(customer_number);
        let customer = self.repository.create_customer(&request_with_number, created_by).await?;

        // 7. Post-creation business logic
        self.handle_post_creation_logic(&customer).await?;

        Ok(customer)
    }

    async fn update_customer(&self, id: Uuid, mut request: UpdateCustomerRequest, modified_by: Uuid) -> Result<Customer> {
        // 1. Input validation
        request.validate()
            .map_err(|e| MasterDataError::ValidationError {
                field: "request".to_string(),
                message: e.to_string(),
            })?;

        // 2. Get existing customer
        let existing = self.repository.get_customer_by_id(id).await?
            .ok_or(MasterDataError::CustomerNotFound { id: id.to_string() })?;

        // 3. Business rule validation for updates
        self.validate_update_business_rules(&existing, &request).await?;

        // 4. Validate hierarchy changes
        let new_parent_id = request.parent_customer_id.unwrap_or(existing.parent_customer_id);
        if new_parent_id != existing.parent_customer_id {
            self.validate_hierarchy(Some(id), new_parent_id).await?;
        }

        // 5. Validate customer number changes
        if let Some(ref new_number) = request.customer_number {
            if new_number != &existing.customer_number {
                if !self.repository.is_customer_number_available(new_number).await? {
                    return Err(MasterDataError::DuplicateCustomerNumber {
                        number: new_number.clone(),
                    });
                }
            }
        }

        // 6. Update customer
        let updated_customer = self.repository.update_customer(id, &request, modified_by).await?;

        // 7. Post-update business logic
        self.handle_post_update_logic(&existing, &updated_customer).await?;

        Ok(updated_customer)
    }

    async fn get_customer(&self, id: Uuid) -> Result<Option<Customer>> {
        self.repository.get_customer_by_id(id).await
    }

    async fn search_customers(&self, criteria: CustomerSearchCriteria) -> Result<CustomerSearchResponse> {
        // Apply business rule filters
        let filtered_criteria = self.apply_business_rule_filters(criteria).await?;

        let customers = self.repository.search_customers(&filtered_criteria).await?;

        // Convert to CustomerSearchResponse with basic pagination info
        Ok(CustomerSearchResponse {
            customers,
            total_count: 0, // Would need separate count query
            page: 1,
            page_size: 50,
            total_pages: 1,
        })
    }

    async fn delete_customer(&self, id: Uuid, deleted_by: Uuid) -> Result<()> {
        // 1. Get existing customer
        let customer = self.repository.get_customer_by_id(id).await?
            .ok_or(MasterDataError::CustomerNotFound { id: id.to_string() })?;

        // 2. Validate deletion constraints
        self.validate_deletion_constraints(&customer).await?;

        // 3. Check for dependent records
        if self.has_active_orders(&customer).await? {
            return Err(MasterDataError::CustomerHasActiveOrders);
        }

        // 4. Soft delete
        self.repository.delete_customer(id, deleted_by).await
    }

    async fn validate_credit_limit_increase(&self, customer_id: Uuid, new_limit: rust_decimal::Decimal) -> Result<()> {
        let customer = self.repository.get_customer_by_id(customer_id).await?
            .ok_or(MasterDataError::CustomerNotFound { id: customer_id.to_string() })?;

        // Business rules for credit limit increases
        let current_limit = customer.financial_info.credit_limit.unwrap_or_default();
        let increase_percentage = if current_limit > rust_decimal::Decimal::ZERO {
            ((new_limit - current_limit) / current_limit) * rust_decimal::Decimal::from(100)
        } else {
            rust_decimal::Decimal::from(100) // 100% increase from zero
        };

        // Rule: Credit limit increases > 50% require additional approval
        if increase_percentage > rust_decimal::Decimal::from(50) {
            return Err(MasterDataError::ValidationError {
                field: "credit_limit".to_string(),
                message: format!("Credit limit increase of {:.2}% exceeds 50% threshold and requires additional approval", increase_percentage),
            });
        }

        // Rule: Credit limit cannot exceed 10x annual revenue
        if let Some(annual_revenue) = customer.performance_metrics.total_revenue {
            let max_limit = annual_revenue * rust_decimal::Decimal::from(10);
            if new_limit > max_limit {
                return Err(MasterDataError::CreditLimitExceeded {
                    requested: new_limit.to_string(),
                    limit: max_limit.to_string(),
                });
            }
        }

        Ok(())
    }

    async fn update_lifecycle_stage(&self, customer_id: Uuid, new_stage: CustomerLifecycleStage, updated_by: Uuid) -> Result<()> {
        let customer = self.repository.get_customer_by_id(customer_id).await?
            .ok_or(MasterDataError::CustomerNotFound { id: customer_id.to_string() })?;

        // Validate lifecycle stage transitions
        self.validate_lifecycle_stage_transition(&customer.lifecycle_stage, &new_stage)?;

        // Update customer with new lifecycle stage
        let update_request = UpdateCustomerRequest {
            lifecycle_stage: Some(new_stage),
            version: customer.audit.version,
            ..Default::default()
        };

        self.repository.update_customer(customer_id, &update_request, updated_by).await?;

        Ok(())
    }

    async fn calculate_performance_metrics(&self, _customer_id: Uuid) -> Result<CustomerPerformanceMetrics> {
        // This would typically integrate with order management, payment systems, etc.
        // For now, return basic metrics structure
        Ok(CustomerPerformanceMetrics {
            total_revenue: None,
            revenue_last_12_months: None,
            average_order_value: None,
            order_frequency: None,
            total_orders: None,
            last_order_date: None,
            profit_margin: None,
            last_purchase_date: None,
            first_purchase_date: None,
            customer_lifetime_value: None,
            predicted_churn_probability: None,
            relationship_duration_days: None,
            satisfaction_score: None,
            net_promoter_score: None,
            last_contact_date: None,
            contact_frequency: None,
            response_rate: None,
            days_sales_outstanding: None,
            payment_reliability_score: None,
            support_ticket_count: None,
            last_calculated: chrono::Utc::now(),
        })
    }

    async fn generate_customer_number(&self, customer_type: CustomerType) -> Result<String> {
        // Business rules for customer number generation
        let prefix = match customer_type {
            CustomerType::B2b => "B2B",
            CustomerType::B2c => "B2C",
            CustomerType::B2g => "B2G",
            CustomerType::Internal => "INT",
            CustomerType::Reseller => "RSL",
            CustomerType::Distributor => "DST",
            CustomerType::EndUser => "END",
            CustomerType::Prospect => "PRS",
        };

        // Generate sequence number (this would typically use a database sequence)
        let sequence = self.get_next_sequence_number(&customer_type).await?;

        Ok(format!("{}{:06}", prefix, sequence))
    }

    async fn validate_hierarchy(&self, customer_id: Option<Uuid>, parent_id: Option<Uuid>) -> Result<()> {
        if let Some(parent_id) = parent_id {
            // Validate parent exists
            let _parent = self.repository.get_customer_by_id(parent_id).await?
                .ok_or(MasterDataError::CustomerNotFound { id: parent_id.to_string() })?;

            // Prevent circular hierarchy
            if let Some(customer_id) = customer_id {
                if self.would_create_circular_hierarchy(customer_id, parent_id).await? {
                    return Err(MasterDataError::ValidationError {
                        field: "parent_customer_id".to_string(),
                        message: "Circular hierarchy detected".to_string(),
                    });
                }
            }

            // Validate hierarchy depth (max 5 levels)
            let hierarchy_level = self.calculate_hierarchy_level(parent_id).await?;
            if hierarchy_level >= 5 {
                return Err(MasterDataError::ValidationError {
                    field: "parent_customer_id".to_string(),
                    message: "Maximum hierarchy depth of 5 levels exceeded".to_string(),
                });
            }
        }

        Ok(())
    }
}

// Private helper methods
impl DefaultCustomerService {
    async fn validate_create_business_rules(&self, request: &CreateCustomerRequest) -> Result<()> {
        // Rule: B2B customers must have a legal name of at least 2 characters
        if request.customer_type == CustomerType::B2b && request.legal_name.len() < 2 {
            return Err(MasterDataError::ValidationError {
                field: "legal_name".to_string(),
                message: "B2B customers must have a legal name of at least 2 characters".to_string(),
            });
        }

        // Rule: Internal customers cannot have external IDs
        if request.customer_type == CustomerType::Internal {
            if let Some(ref external_ids) = request.external_ids {
                if !external_ids.is_empty() {
                    return Err(MasterDataError::ValidationError {
                        field: "external_ids".to_string(),
                        message: "Internal customers cannot have external IDs".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    async fn validate_update_business_rules(&self, existing: &Customer, request: &UpdateCustomerRequest) -> Result<()> {
        // Rule: Cannot change customer type if customer has orders
        if let Some(new_type) = &request.customer_type {
            if *new_type != existing.customer_type && self.has_orders(existing).await? {
                return Err(MasterDataError::ValidationError {
                    field: "customer_type".to_string(),
                    message: "Cannot change customer type for customers with existing orders".to_string(),
                });
            }
        }

        // Rule: Cannot downgrade lifecycle stage
        if let Some(new_stage) = &request.lifecycle_stage {
            self.validate_lifecycle_stage_transition(&existing.lifecycle_stage, new_stage)?;
        }

        Ok(())
    }

    fn validate_lifecycle_stage_transition(&self, current: &CustomerLifecycleStage, new: &CustomerLifecycleStage) -> Result<()> {
        use CustomerLifecycleStage::*;

        let valid_transitions = match current {
            Lead => vec![Prospect, FormerCustomer],
            Prospect => vec![NewCustomer, FormerCustomer],
            NewCustomer => vec![ActiveCustomer, InactiveCustomer, FormerCustomer],
            ActiveCustomer => vec![VipCustomer, AtRiskCustomer, InactiveCustomer, FormerCustomer],
            VipCustomer => vec![ActiveCustomer, AtRiskCustomer, InactiveCustomer, FormerCustomer],
            AtRiskCustomer => vec![ActiveCustomer, WonBackCustomer, InactiveCustomer, FormerCustomer],
            InactiveCustomer => vec![WonBackCustomer, FormerCustomer],
            WonBackCustomer => vec![ActiveCustomer, VipCustomer, AtRiskCustomer, InactiveCustomer, FormerCustomer],
            FormerCustomer => vec![WonBackCustomer], // Only allow win-back
        };

        if !valid_transitions.contains(new) {
            return Err(MasterDataError::ValidationError {
                field: "lifecycle_stage".to_string(),
                message: format!("Invalid lifecycle stage transition from {:?} to {:?}", current, new),
            });
        }

        Ok(())
    }

    async fn apply_business_rule_filters(&self, criteria: CustomerSearchCriteria) -> Result<CustomerSearchCriteria> {
        // Apply tenant-specific filtering and business rules
        // For now, return the criteria as-is
        Ok(criteria)
    }

    async fn validate_deletion_constraints(&self, customer: &Customer) -> Result<()> {
        // Rule: Cannot delete VIP customers without approval
        if customer.lifecycle_stage == CustomerLifecycleStage::VipCustomer {
            return Err(MasterDataError::ValidationError {
                field: "lifecycle_stage".to_string(),
                message: "VIP customers require special approval for deletion".to_string(),
            });
        }

        // Rule: Cannot delete customers with children
        let children = self.repository.get_customer_hierarchy(customer.id).await?;
        if children.len() > 1 { // More than just the customer itself
            return Err(MasterDataError::ValidationError {
                field: "hierarchy".to_string(),
                message: "Cannot delete customers with child customers".to_string(),
            });
        }

        Ok(())
    }

    async fn has_active_orders(&self, _customer: &Customer) -> Result<bool> {
        // This would integrate with order management system
        // For now, return false
        Ok(false)
    }

    async fn has_orders(&self, _customer: &Customer) -> Result<bool> {
        // This would integrate with order management system
        // For now, return false
        Ok(false)
    }

    async fn handle_post_creation_logic(&self, _customer: &Customer) -> Result<()> {
        // Post-creation business logic (notifications, integrations, etc.)
        Ok(())
    }

    async fn handle_post_update_logic(&self, _old_customer: &Customer, _new_customer: &Customer) -> Result<()> {
        // Post-update business logic (notifications, audit, etc.)
        Ok(())
    }

    async fn get_next_sequence_number(&self, _customer_type: &CustomerType) -> Result<u32> {
        // This would typically use a database sequence or counter
        // For now, return a placeholder
        Ok(1)
    }

    async fn would_create_circular_hierarchy(&self, customer_id: Uuid, parent_id: Uuid) -> Result<bool> {
        // Check if setting parent_id as parent of customer_id would create a cycle
        let hierarchy = self.repository.get_customer_hierarchy(parent_id).await?;
        Ok(hierarchy.iter().any(|c| c.id == customer_id))
    }

    async fn calculate_hierarchy_level(&self, parent_id: Uuid) -> Result<u8> {
        let hierarchy = self.repository.get_customer_hierarchy(parent_id).await?;
        Ok(hierarchy.len() as u8)
    }
}