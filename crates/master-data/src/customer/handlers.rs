use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::customer::model::*;
use crate::customer::service::CustomerService;
use crate::error::{MasterDataError, Result};
use crate::types::*;
use erp_core::RequestContext;

/// HTTP handlers for customer operations
pub struct CustomerHandlers {
    service: Arc<dyn CustomerService>,
}

impl CustomerHandlers {
    pub fn new(service: Arc<dyn CustomerService>) -> Self {
        Self { service }
    }

    /// Configure customer routes
    pub fn routes() -> Router<Arc<dyn CustomerService>> {
        Router::new()
            .route("/customers", post(create_customer))
            .route("/customers", get(search_customers))
            .route("/customers/:id", get(get_customer))
            .route("/customers/:id", put(update_customer))
            .route("/customers/:id", delete(delete_customer))
            .route("/customers/:id/lifecycle", put(update_lifecycle_stage))
            .route("/customers/:id/credit-limit", post(validate_credit_limit))
            .route("/customers/:id/hierarchy", get(get_customer_hierarchy))
            .route("/customers/:id/performance", get(get_performance_metrics))
            .route("/customers/generate-number/:customer_type", post(generate_customer_number))
    }
}

/// Request/Response DTOs for API

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerResponse {
    pub customer: Customer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomersResponse {
    pub customers: Vec<Customer>,
    pub total_count: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateLifecycleStageRequest {
    #[validate(required)]
    pub new_stage: CustomerLifecycleStage,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ValidateCreditLimitRequest {
    #[validate(range(min = 0.0))]
    pub new_limit: rust_decimal::Decimal,
}

#[derive(Debug, Serialize)]
pub struct CreditLimitValidationResponse {
    pub valid: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CustomerNumberResponse {
    pub customer_number: String,
}

#[derive(Debug, Serialize)]
pub struct PerformanceMetricsResponse {
    pub metrics: CustomerPerformanceMetrics,
}

/// API Handler Functions

/// Create a new customer
/// POST /customers
async fn create_customer(
    State(service): State<Arc<dyn CustomerService>>,
    ctx: RequestContext,
    Json(request): Json<CreateCustomerRequest>,
) -> Result<Json<CustomerResponse>, MasterDataError> {
    // Get user ID from context
    let created_by = ctx.user_id
        .ok_or_else(|| MasterDataError::ValidationError {
            field: "user".to_string(),
            message: "User ID required".to_string(),
        })?;

    let customer = service.create_customer(request, created_by).await?;

    Ok(Json(CustomerResponse { customer }))
}

/// Get customer by ID
/// GET /customers/:id
async fn get_customer(
    State(service): State<Arc<dyn CustomerService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CustomerResponse>, MasterDataError> {
    let customer = service.get_customer(id).await?
        .ok_or(MasterDataError::CustomerNotFound { id: id.to_string() })?;

    Ok(Json(CustomerResponse { customer }))
}

/// Update customer
/// PUT /customers/:id
async fn update_customer(
    State(service): State<Arc<dyn CustomerService>>,
    Path(id): Path<Uuid>,
    ctx: RequestContext,
    Json(request): Json<UpdateCustomerRequest>,
) -> Result<Json<CustomerResponse>, MasterDataError> {
    // Get user ID from context
    let modified_by = ctx.user_id
        .ok_or_else(|| MasterDataError::ValidationError {
            field: "user".to_string(),
            message: "User ID required".to_string(),
        })?;

    let customer = service.update_customer(id, request, modified_by).await?;

    Ok(Json(CustomerResponse { customer }))
}

/// Search customers with advanced filtering
/// GET /customers?search_term=...&customer_type=...&page=1&page_size=50
async fn search_customers(
    State(service): State<Arc<dyn CustomerService>>,
    Query(params): Query<CustomerSearchQueryParams>,
) -> Result<Json<CustomersResponse>, MasterDataError> {
    let criteria = params.into_search_criteria();
    let response = service.search_customers(criteria).await?;

    Ok(Json(CustomersResponse {
        customers: response.customers,
        total_count: response.total_count,
        page: response.page,
        page_size: response.page_size,
        total_pages: response.total_pages,
    }))
}

/// Delete customer (soft delete)
/// DELETE /customers/:id
async fn delete_customer(
    State(service): State<Arc<dyn CustomerService>>,
    Path(id): Path<Uuid>,
    ctx: RequestContext,
) -> Result<StatusCode, MasterDataError> {
    // Get user ID from context
    let deleted_by = ctx.user_id
        .ok_or_else(|| MasterDataError::ValidationError {
            field: "user".to_string(),
            message: "User ID required".to_string(),
        })?;

    service.delete_customer(id, deleted_by).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Update customer lifecycle stage
/// PUT /customers/:id/lifecycle
async fn update_lifecycle_stage(
    State(service): State<Arc<dyn CustomerService>>,
    Path(id): Path<Uuid>,
    ctx: RequestContext,
    Json(request): Json<UpdateLifecycleStageRequest>,
) -> Result<StatusCode, MasterDataError> {
    // Validate request
    request.validate()
        .map_err(|e| MasterDataError::ValidationError {
            field: "request".to_string(),
            message: e.to_string(),
        })?;

    // Get user ID from context
    let updated_by = ctx.user_id
        .ok_or_else(|| MasterDataError::ValidationError {
            field: "user".to_string(),
            message: "User ID required".to_string(),
        })?;

    service.update_lifecycle_stage(id, request.new_stage, updated_by).await?;

    Ok(StatusCode::OK)
}

/// Validate credit limit increase
/// POST /customers/:id/credit-limit
async fn validate_credit_limit(
    State(service): State<Arc<dyn CustomerService>>,
    Path(id): Path<Uuid>,
    Json(request): Json<ValidateCreditLimitRequest>,
) -> Result<Json<CreditLimitValidationResponse>, MasterDataError> {
    // Validate request
    request.validate()
        .map_err(|e| MasterDataError::ValidationError {
            field: "request".to_string(),
            message: e.to_string(),
        })?;

    match service.validate_credit_limit_increase(id, request.new_limit).await {
        Ok(()) => Ok(Json(CreditLimitValidationResponse {
            valid: true,
            message: None,
        })),
        Err(e) => Ok(Json(CreditLimitValidationResponse {
            valid: false,
            message: Some(e.to_string()),
        })),
    }
}

/// Get customer hierarchy
/// GET /customers/:id/hierarchy
async fn get_customer_hierarchy(
    State(service): State<Arc<dyn CustomerService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Customer>>, MasterDataError> {
    // Note: This is a simplified implementation
    // In a full implementation, we would call a repository method directly
    // or add a hierarchy method to the service
    let customer = service.get_customer(id).await?
        .ok_or(MasterDataError::CustomerNotFound { id: id.to_string() })?;

    // For now, just return the single customer
    // In a full implementation, this would return the complete hierarchy
    Ok(Json(vec![customer]))
}

/// Get customer performance metrics
/// GET /customers/:id/performance
async fn get_performance_metrics(
    State(service): State<Arc<dyn CustomerService>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PerformanceMetricsResponse>, MasterDataError> {
    let metrics = service.calculate_performance_metrics(id).await?;

    Ok(Json(PerformanceMetricsResponse { metrics }))
}

/// Generate customer number
/// POST /customers/generate-number/:customer_type
async fn generate_customer_number(
    State(service): State<Arc<dyn CustomerService>>,
    Path(customer_type): Path<CustomerType>,
) -> Result<Json<CustomerNumberResponse>, MasterDataError> {
    let customer_number = service.generate_customer_number(customer_type).await?;

    Ok(Json(CustomerNumberResponse { customer_number }))
}

/// Query parameters for customer search
#[derive(Debug, Deserialize)]
pub struct CustomerSearchQueryParams {
    // Search criteria
    pub search_term: Option<String>,
    pub customer_type: Option<CustomerType>,
    pub lifecycle_stage: Option<CustomerLifecycleStage>,
    pub credit_status: Option<CreditStatus>,
    pub industry_classification: Option<IndustryClassification>,
    pub business_size: Option<BusinessSize>,
    pub status: Option<EntityStatus>,

    // Geographic filters
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,

    // Hierarchy filters
    pub parent_customer_id: Option<Uuid>,
    pub corporate_group_id: Option<Uuid>,
    pub hierarchy_level: Option<u8>,

    // Date filters (ISO 8601 format)
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub modified_after: Option<String>,
    pub modified_before: Option<String>,

    // Financial filters
    pub min_credit_limit: Option<rust_decimal::Decimal>,
    pub max_credit_limit: Option<rust_decimal::Decimal>,
    pub min_annual_revenue: Option<rust_decimal::Decimal>,
    pub max_annual_revenue: Option<rust_decimal::Decimal>,

    // Pagination
    pub page: Option<u32>,
    pub page_size: Option<u32>,

    // Sorting
    pub sort_by: Option<String>,
    pub sort_order: Option<String>, // "asc" or "desc"
}

impl CustomerSearchQueryParams {
    pub fn into_search_criteria(self) -> CustomerSearchCriteria {
        CustomerSearchCriteria {
            search_term: self.search_term,
            customer_types: self.customer_type.map(|t| vec![t]),
            lifecycle_stages: self.lifecycle_stage.map(|s| vec![s]),
            credit_statuses: self.credit_status.map(|s| vec![s]),
            industry_classifications: self.industry_classification.map(|i| vec![i]),
            business_sizes: self.business_size.map(|s| vec![s]),
            statuses: self.status.map(|s| vec![s]),
            countries: self.country.map(|c| vec![c]),
            regions: self.region.map(|r| vec![r]),
            cities: self.city.map(|c| vec![c]),
            parent_customer_id: self.parent_customer_id,
            corporate_group_id: self.corporate_group_id,
            hierarchy_level: self.hierarchy_level,
            created_after: self.created_after.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
            created_before: self.created_before.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
            modified_after: self.modified_after.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
            modified_before: self.modified_before.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Utc))),
            min_credit_limit: self.min_credit_limit,
            max_credit_limit: self.max_credit_limit,
            min_annual_revenue: self.min_annual_revenue,
            max_annual_revenue: self.max_annual_revenue,
            page: self.page.unwrap_or(1),
            page_size: std::cmp::min(self.page_size.unwrap_or(50), 1000), // Cap at 1000
            sort_by: self.sort_by,
            sort_order: self.sort_order,
            include_deleted: Some(false), // Default to excluding deleted
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use std::sync::Arc;

    // Mock service for testing
    struct MockCustomerService;

    #[async_trait]
    impl CustomerService for MockCustomerService {
        async fn create_customer(&self, _request: CreateCustomerRequest, _created_by: Uuid) -> Result<Customer> {
            todo!("Mock implementation needed")
        }

        async fn update_customer(&self, _id: Uuid, _request: UpdateCustomerRequest, _modified_by: Uuid) -> Result<Customer> {
            todo!("Mock implementation needed")
        }

        async fn get_customer(&self, _id: Uuid) -> Result<Option<Customer>> {
            todo!("Mock implementation needed")
        }

        async fn search_customers(&self, _criteria: CustomerSearchCriteria) -> Result<CustomerSearchResponse> {
            todo!("Mock implementation needed")
        }

        async fn delete_customer(&self, _id: Uuid, _deleted_by: Uuid) -> Result<()> {
            todo!("Mock implementation needed")
        }

        async fn validate_credit_limit_increase(&self, _customer_id: Uuid, _new_limit: rust_decimal::Decimal) -> Result<()> {
            todo!("Mock implementation needed")
        }

        async fn update_lifecycle_stage(&self, _customer_id: Uuid, _new_stage: CustomerLifecycleStage, _updated_by: Uuid) -> Result<()> {
            todo!("Mock implementation needed")
        }

        async fn calculate_performance_metrics(&self, _customer_id: Uuid) -> Result<CustomerPerformanceMetrics> {
            todo!("Mock implementation needed")
        }

        async fn generate_customer_number(&self, _customer_type: CustomerType) -> Result<String> {
            Ok("B2B000001".to_string())
        }

        async fn validate_hierarchy(&self, _customer_id: Option<Uuid>, _parent_id: Option<Uuid>) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_generate_customer_number_endpoint() {
        let service = Arc::new(MockCustomerService) as Arc<dyn CustomerService>;
        let app = CustomerHandlers::routes().with_state(service);
        let server = TestServer::new(app).unwrap();

        let response = server
            .post("/customers/generate-number/b2b")
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
    }
}