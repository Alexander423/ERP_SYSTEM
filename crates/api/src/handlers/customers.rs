//! Customer management handlers
//!
//! HTTP handlers for customer CRUD operations

use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete, Router},
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::state::AppState;

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
    pub legal_name: String,
    pub customer_type: String, // TODO: Use proper enum
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomerRequest {
    pub legal_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
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
    State(_state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Value>, StatusCode> {
    // For now, use a default tenant ID since middleware integration needs more work
    let tenant_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
        .unwrap_or_else(|_| uuid::Uuid::new_v4());

    // For now, return a mock response since we need to wire up the repository
    // In a complete implementation, we'd:
    // 1. Create CustomerRepository instance
    // 2. Call repository.list_customers with search criteria
    // 3. Return paginated results

    Ok(Json(json!({
        "success": true,
        "customers": [],
        "pagination": {
            "page": params.page,
            "limit": params.limit,
            "total": 0,
            "total_pages": 0
        },
        "tenant_id": tenant_id
    })))
}

/// Create a new customer
async fn create_customer(
    State(_state): State<AppState>,
    Json(payload): Json<CreateCustomerRequest>,
) -> Result<Json<Value>, StatusCode> {
    // For now, use a default tenant ID since middleware integration needs more work
    let tenant_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
        .unwrap_or_else(|_| uuid::Uuid::new_v4());

    // Basic validation
    if payload.legal_name.trim().is_empty() {
        return Ok(Json(json!({
            "success": false,
            "error": "Legal name is required"
        })));
    }

    // For now, return a mock response
    // In a complete implementation, we'd:
    // 1. Create CustomerRepository instance with tenant_context
    // 2. Map API request to domain CreateCustomerRequest
    // 3. Call repository.create_customer
    // 4. Return created customer data

    let customer_id = Uuid::new_v4();

    Ok(Json(json!({
        "success": true,
        "customer": {
            "id": customer_id,
            "legal_name": payload.legal_name,
            "customer_type": payload.customer_type,
            "email": payload.email,
            "phone": payload.phone,
            "website": payload.website,
            "tenant_id": tenant_id
        },
        "message": "Customer created successfully (mock response)"
    })))
}

/// Get customer by ID
async fn get_customer(
    State(_state): State<AppState>,
    Path(customer_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    // For now, use a default tenant ID since middleware integration needs more work
    let tenant_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
        .unwrap_or_else(|_| uuid::Uuid::new_v4());

    // For now, return a mock response
    // In a complete implementation, we'd:
    // 1. Create CustomerRepository instance
    // 2. Call repository.get_customer_by_id
    // 3. Return customer data or 404

    Ok(Json(json!({
        "success": true,
        "customer": {
            "id": customer_id,
            "legal_name": "Sample Customer",
            "customer_type": "b2b",
            "tenant_id": tenant_id
        }
    })))
}

/// Update customer
async fn update_customer(
    State(_state): State<AppState>,
    Path(customer_id): Path<Uuid>,
    Json(payload): Json<UpdateCustomerRequest>,
) -> Result<Json<Value>, StatusCode> {
    // For now, use a default tenant ID since middleware integration needs more work
    let tenant_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
        .unwrap_or_else(|_| uuid::Uuid::new_v4());

    // For now, return a mock response
    Ok(Json(json!({
        "success": true,
        "customer": {
            "id": customer_id,
            "legal_name": payload.legal_name.unwrap_or("Updated Customer".to_string()),
            "email": payload.email,
            "phone": payload.phone,
            "website": payload.website,
            "tenant_id": tenant_id
        },
        "message": "Customer updated successfully (mock response)"
    })))
}

/// Delete customer
async fn delete_customer(
    State(_state): State<AppState>,
    Path(customer_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    // For now, use a default tenant ID since middleware integration needs more work
    let tenant_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
        .unwrap_or_else(|_| uuid::Uuid::new_v4());

    // For now, return a mock success response
    Ok(Json(json!({
        "success": true,
        "message": format!("Customer {} deleted successfully (mock response)", customer_id),
        "tenant_id": tenant_id
    })))
}

/// Get customer hierarchy
async fn get_customer_hierarchy(
    State(_state): State<AppState>,
    Path(customer_id): Path<Uuid>,
) -> Result<Json<Value>, StatusCode> {
    // For now, use a default tenant ID since middleware integration needs more work
    let tenant_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
        .unwrap_or_else(|_| uuid::Uuid::new_v4());

    // For now, return a mock hierarchy
    Ok(Json(json!({
        "success": true,
        "hierarchy": [
            {
                "id": customer_id,
                "legal_name": "Parent Customer",
                "level": 0
            }
        ],
        "tenant_id": tenant_id
    })))
}