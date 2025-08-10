//! # Health Check Endpoints
//! 
//! This module provides health monitoring endpoints for the ERP API server.
//! These endpoints are essential for:
//! 
//! - **Load balancer health checks**: Determine if instances should receive traffic
//! - **Container orchestration**: Kubernetes liveness and readiness probes
//! - **Monitoring systems**: Automated alerting on service degradation
//! - **Deployment validation**: Ensure services start correctly
//! 
//! ## Health Check Types
//! 
//! ### Liveness Check (`/health`)
//! - **Purpose**: Indicates if the service is running and not deadlocked
//! - **Response**: Always returns 200 OK with basic service info
//! - **Use case**: Load balancer health checks, basic monitoring
//! 
//! ### Readiness Check (`/ready`)  
//! - **Purpose**: Indicates if the service can handle requests
//! - **Dependencies**: Tests database and Redis connectivity
//! - **Response**: 200 OK if ready, 503 Service Unavailable if not
//! - **Use case**: Kubernetes readiness probes, deployment validation
//! 
//! ## Integration Examples
//! 
//! ### Docker Health Check
//! ```dockerfile
//! HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
//!   CMD curl -f http://localhost:3000/health || exit 1
//! ```
//! 
//! ### Kubernetes Probes
//! ```yaml
//! livenessProbe:
//!   httpGet:
//!     path: /health
//!     port: 3000
//!   initialDelaySeconds: 30
//!   periodSeconds: 10
//! 
//! readinessProbe:
//!   httpGet:
//!     path: /ready
//!     port: 3000
//!   initialDelaySeconds: 5
//!   periodSeconds: 5
//! ```

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use tracing::error;

use crate::state::AppState;

/// Basic health check endpoint for liveness monitoring.
/// 
/// This endpoint provides a simple health status response that indicates
/// the service is running and responsive. It does not check external
/// dependencies and should always return successfully unless the service
/// is completely non-functional.
/// 
/// # Response Format
/// 
/// ```json
/// {
///   "status": "healthy",
///   "service": "erp-api", 
///   "version": "0.1.0"
/// }
/// ```
/// 
/// # HTTP Status
/// 
/// - **200 OK**: Service is alive and responding
/// 
/// # Usage
/// 
/// ```bash
/// curl http://localhost:3000/health
/// ```
/// 
/// # Monitoring Integration
/// 
/// This endpoint is ideal for:
/// - Load balancer health checks
/// - Basic uptime monitoring
/// - Service discovery health status
/// - Container orchestration liveness probes
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = Object)
    ),
    tag = "health"
)]
pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "erp-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Comprehensive readiness check with dependency validation.
/// 
/// This endpoint performs deep health checks of all critical dependencies
/// before indicating the service is ready to handle requests. It validates:
/// 
/// - **Database connectivity**: PostgreSQL connection and query capability
/// - **Cache connectivity**: Redis connection and command execution
/// 
/// # Response Format
/// 
/// **Ready State (200 OK):**
/// ```json
/// {
///   "ready": true,
///   "checks": {
///     "database": true,
///     "redis": true
///   }
/// }
/// ```
/// 
/// **Not Ready State (503 Service Unavailable):**
/// ```json
/// {
///   "ready": false,
///   "checks": {
///     "database": false,
///     "redis": true
///   }
/// }
/// ```
/// 
/// # HTTP Status Codes
/// 
/// - **200 OK**: All dependencies are healthy and service is ready
/// - **503 Service Unavailable**: One or more dependencies are unhealthy
/// 
/// # Error Handling
/// 
/// Dependency failures are:
/// - Logged as errors for debugging
/// - Included in the response for troubleshooting
/// - Result in 503 status to prevent traffic routing
/// 
/// # Usage
/// 
/// ```bash
/// # Check if service is ready
/// curl http://localhost:3000/ready
/// 
/// # Use in health check scripts
/// if curl -f http://localhost:3000/ready; then
///   echo "Service is ready"
/// else
///   echo "Service is not ready"
///   exit 1
/// fi
/// ```
/// 
/// # Monitoring Integration
/// 
/// This endpoint is ideal for:
/// - Kubernetes readiness probes
/// - Deployment validation scripts
/// - Load balancer backend health checks
/// - Automated testing verification
#[utoipa::path(
    get,
    path = "/ready",
    responses(
        (status = 200, description = "Service is ready", body = Object),
        (status = 503, description = "Service is not ready", body = Object)
    ),
    tag = "health"
)]
pub async fn readiness_check(State(state): State<AppState>) -> impl IntoResponse {
    // Check database connection
    let db_healthy = match state.db.check_health().await {
        Ok(_) => true,
        Err(e) => {
            error!("Database health check failed: {}", e);
            false
        }
    };

    // Check Redis connection
    let redis_healthy = {
        let mut conn = state.redis.clone();
        match redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
        {
            Ok(_) => true,
            Err(e) => {
                error!("Redis health check failed: {}", e);
                false
            }
        }
    };

    let is_ready = db_healthy && redis_healthy;

    let status = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(json!({
            "ready": is_ready,
            "checks": {
                "database": db_healthy,
                "redis": redis_healthy,
            }
        })),
    )
}