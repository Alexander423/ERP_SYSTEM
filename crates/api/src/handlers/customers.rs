//! Customer management handlers
//!
//! HTTP handlers for customer CRUD operations

use axum::{
    extract::{State, Path, Query, Extension},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete, Router},
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::state::AppState;
use erp_core::TenantContext;
use erp_master_data::customer::model::{
    CreateCustomerRequest as DomainCreateCustomerRequest,
    UpdateCustomerRequest as DomainUpdateCustomerRequest,
    CustomerSearchCriteria,
    CustomerType,
    CustomerLifecycleStage,
    CreditStatus,
    AcquisitionChannel
};
use erp_master_data::types::{IndustryClassification, BusinessSize, EntityStatus};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 { 1 }
fn default_limit() -> u32 { 20 }

#[derive(Debug, Deserialize)]
pub struct CreateCustomerRequest {
    pub customer_number: Option<String>,
    pub legal_name: String,
    pub trade_names: Option<Vec<String>>,
    pub customer_type: CustomerType,
    pub industry_classification: Option<IndustryClassification>,
    pub business_size: Option<BusinessSize>,
    pub parent_customer_id: Option<Uuid>,
    pub corporate_group_id: Option<Uuid>,
    pub lifecycle_stage: Option<CustomerLifecycleStage>,
    pub status: Option<EntityStatus>,
    pub credit_status: Option<CreditStatus>,
    pub acquisition_channel: Option<AcquisitionChannel>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomerRequest {
    pub legal_name: Option<String>,
    pub trade_names: Option<Vec<String>>,
    pub industry_classification: Option<IndustryClassification>,
    pub business_size: Option<BusinessSize>,
    pub lifecycle_stage: Option<CustomerLifecycleStage>,
    pub status: Option<EntityStatus>,
    pub credit_status: Option<CreditStatus>,
}

#[derive(Debug, Deserialize)]
pub struct CustomerSearchParams {
    pub legal_name: Option<String>,
    pub customer_number: Option<String>,
    pub customer_type: Option<CustomerType>,
    pub status: Option<EntityStatus>,
    pub lifecycle_stage: Option<CustomerLifecycleStage>,
}


/// Create customer management routes
pub fn customer_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_customers))
        .route("/", post(create_customer))
        .route("/:id", get(get_customer))
        .route("/:id", put(update_customer))
        .route("/:id", delete(delete_customer))
        .route("/:id/hierarchy", get(get_customer_hierarchy))
}

/// List all customers
async fn list_customers(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(search): Query<CustomerSearchParams>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    // Use tenant context from middleware

    // Create service instance with business logic
    let service = state.customer_service(tenant_context.clone());

    // Build search criteria
    let criteria = CustomerSearchCriteria {
        search_term: search.legal_name,
        customer_numbers: search.customer_number.map(|cn| vec![cn]),
        customer_types: search.customer_type.map(|ct| vec![ct]),
        statuses: search.status.map(|s| vec![s]),
        lifecycle_stages: search.lifecycle_stage.map(|ls| vec![ls]),
        page: Some(pagination.page),
        page_size: Some(pagination.limit),
        ..Default::default()
    };

    // Call service with business rules applied
    match service.search_customers(criteria).await {
        Ok(search_response) => {
            Ok(Json(json!({
                "success": true,
                "customers": search_response.customers,
                "pagination": {
                    "page": search_response.page,
                    "limit": search_response.page_size,
                    "total": search_response.total_count,
                    "total_pages": search_response.total_pages
                },
                "tenant_id": tenant_context.tenant_id.0
            })))
        },
        Err(e) => {
            tracing::error!("Failed to list customers: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to retrieve customers",
                "message": e.to_string()
            })))
        }
    }
}

/// Create a new customer
async fn create_customer(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(payload): Json<CreateCustomerRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Use tenant context from middleware

    // Basic validation
    if payload.legal_name.trim().is_empty() {
        return Ok(Json(json!({
            "success": false,
            "error": "Legal name is required"
        })));
    }

    // Create service instance with business logic
    let service = state.customer_service(tenant_context.clone());

    // Map API request to domain CreateCustomerRequest
    let domain_request = DomainCreateCustomerRequest {
        customer_number: payload.customer_number,
        legal_name: payload.legal_name,
        trade_names: payload.trade_names,
        customer_type: payload.customer_type,
        industry_classification: payload.industry_classification,
        business_size: payload.business_size,
        parent_customer_id: payload.parent_customer_id,
        corporate_group_id: payload.corporate_group_id,
        lifecycle_stage: payload.lifecycle_stage,
        status: payload.status,
        credit_status: payload.credit_status,
        acquisition_channel: payload.acquisition_channel,
        customer_hierarchy_level: None,
        consolidation_group: None,
        addresses: None,
        contacts: None,
        tax_jurisdictions: None,
        tax_numbers: None,
        financial_info: None,
        sales_representative_id: None,
        account_manager_id: None,
        external_ids: None,
        sync_info: None,
    };

    // Use a default user ID for created_by (this would come from JWT in production)
    let created_by = uuid::Uuid::new_v4();

    // Call service with business rules applied
    match service.create_customer(domain_request, created_by).await {
        Ok(customer) => {
            Ok(Json(json!({
                "success": true,
                "customer": customer,
                "message": "Customer created successfully"
            })))
        },
        Err(e) => {
            tracing::error!("Failed to create customer: {}", e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to create customer",
                "message": e.to_string()
            })))
        }
    }
}

/// Get customer by ID
async fn get_customer(
    State(state): State<AppState>,
    Path(customer_id): Path<Uuid>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    // Use tenant context from middleware

    // Create service instance with business logic
    let service = state.customer_service(tenant_context.clone());

    // Call service with business rules applied
    match service.get_customer(customer_id).await {
        Ok(Some(customer)) => {
            Ok(Json(json!({
                "success": true,
                "customer": customer
            })))
        },
        Ok(None) => {
            Ok(Json(json!({
                "success": false,
                "error": "Customer not found",
                "message": format!("Customer with ID {} not found", customer_id)
            })))
        },
        Err(e) => {
            tracing::error!("Failed to get customer {}: {}", customer_id, e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to retrieve customer",
                "message": e.to_string()
            })))
        }
    }
}

/// Update customer
async fn update_customer(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(customer_id): Path<Uuid>,
    Json(payload): Json<UpdateCustomerRequest>,
) -> Result<Json<Value>, StatusCode> {
    // Use tenant context from middleware

    // Create service instance with business logic
    let service = state.customer_service(tenant_context.clone());

    // Map API request to domain UpdateCustomerRequest
    let domain_update = DomainUpdateCustomerRequest {
        customer_number: None,
        legal_name: payload.legal_name,
        trade_names: payload.trade_names,
        customer_type: None,
        industry_classification: payload.industry_classification,
        business_size: payload.business_size,
        parent_customer_id: None,
        corporate_group_id: None,
        lifecycle_stage: payload.lifecycle_stage,
        status: payload.status,
        credit_status: payload.credit_status,
        tax_numbers: None,
        financial_info: None,
        sales_representative_id: None,
        account_manager_id: None,
        external_ids: None,
        sync_info: None,
        version: 1, // Version for optimistic locking - in production this would come from the request
    };

    // Use a default user ID for modified_by (this would come from JWT in production)
    let modified_by = uuid::Uuid::new_v4();

    // Call service with business rules applied
    match service.update_customer(customer_id, domain_update, modified_by).await {
        Ok(customer) => {
            Ok(Json(json!({
                "success": true,
                "customer": customer,
                "message": "Customer updated successfully"
            })))
        },
        Err(e) => {
            tracing::error!("Failed to update customer {}: {}", customer_id, e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to update customer",
                "message": e.to_string()
            })))
        }
    }
}

/// Delete customer
async fn delete_customer(
    State(state): State<AppState>,
    Path(customer_id): Path<Uuid>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    // Use tenant context from middleware

    // Create service instance with business logic
    let service = state.customer_service(tenant_context.clone());

    // Use a default user ID for deleted_by (this would come from JWT in production)
    let deleted_by = uuid::Uuid::new_v4();

    // Call service with business rules applied (soft delete)
    match service.delete_customer(customer_id, deleted_by).await {
        Ok(()) => {
            Ok(Json(json!({
                "success": true,
                "message": format!("Customer {} deleted successfully", customer_id)
            })))
        },
        Err(e) => {
            tracing::error!("Failed to delete customer {}: {}", customer_id, e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to delete customer",
                "message": e.to_string()
            })))
        }
    }
}

/// Get customer hierarchy
async fn get_customer_hierarchy(
    State(state): State<AppState>,
    Path(customer_id): Path<Uuid>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    // Use tenant context from middleware

    // Create repository instance (hierarchy not yet in service layer)
    let repository = state.customer_repository(tenant_context.clone());

    // Call repository to get customer hierarchy
    match repository.get_customer_hierarchy(customer_id).await {
        Ok(hierarchy) => {
            Ok(Json(json!({
                "success": true,
                "hierarchy": hierarchy
            })))
        },
        Err(e) => {
            tracing::error!("Failed to get customer hierarchy for {}: {}", customer_id, e);
            Ok(Json(json!({
                "success": false,
                "error": "Failed to retrieve customer hierarchy",
                "message": e.to_string()
            })))
        }
    }
}