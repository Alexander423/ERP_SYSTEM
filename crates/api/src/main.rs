//! # ERP System API Server
//! 
//! This is the main HTTP server for the ERP system, built with Axum for high-performance
//! async request handling. The server provides:
//! 
//! ## Core Features
//! 
//! - **RESTful API**: Complete REST endpoints for all ERP modules
//! - **Multi-tenant support**: Tenant isolation via headers and database schemas
//! - **Interactive API docs**: Swagger UI with complete OpenAPI 3.0 specification
//! - **Security middleware**: CORS, security headers, request ID tracking
//! - **Performance optimization**: Response compression and efficient connection pooling
//! - **Health monitoring**: Health check endpoints for load balancers and monitoring
//! 
//! ## Architecture
//! 
//! ```text
//! ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐
//! │   Client    │    │ API Server   │    │ Business Logic  │
//! │             │────│              │────│                 │
//! │ - Web UI    │    │ - Axum HTTP  │    │ - Auth Service  │
//! │ - Mobile    │    │ - Middleware │    │ - Core Services │
//! │ - API Calls │    │ - Validation │    │ - Repositories  │
//! └─────────────┘    └──────────────┘    └─────────────────┘
//! ```
//! 
//! ## Middleware Stack
//! 
//! Requests flow through middleware in this order:
//! 1. **Security Headers**: HSTS, CSP, X-Frame-Options
//! 2. **Request ID**: Unique tracking for request tracing
//! 3. **Tracing**: Structured logging with correlation IDs
//! 4. **Compression**: Gzip/Brotli response compression
//! 5. **CORS**: Cross-origin resource sharing policies
//! 6. **Authentication**: JWT token validation (route-specific)
//! 
//! ## Usage
//! 
//! Start the server:
//! ```bash
//! cargo run --bin erp-api
//! ```
//! 
//! The server will be available at:
//! - **API**: http://localhost:3000/api/v1/
//! - **Health**: http://localhost:3000/health
//! - **Docs**: http://localhost:3000/swagger-ui

use axum::{
    Router,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use erp_auth::AuthService;
use erp_core::{Config, CorsConfig, DatabasePool};
use redis::aio::ConnectionManager;
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{CorsLayer, Any},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use axum::http::{Method, HeaderName, HeaderValue};
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod error;
mod handlers;
mod health;
mod api_middleware;
mod state;

use crate::{
    handlers::{auth, users, roles, customers},
    state::AppState
};

/// Builds a CORS layer from configuration settings.
/// 
/// This function creates a tower-http CORS layer based on the application's
/// CORS configuration. It supports both permissive development settings
/// and restrictive production policies.
/// 
/// # Configuration Options
/// 
/// - **Origins**: Specific domains or "*" wildcard (development only)
/// - **Methods**: HTTP methods allowed for cross-origin requests
/// - **Headers**: Request headers permitted in CORS requests
/// - **Credentials**: Whether to allow cookies and authorization headers
/// - **Max Age**: How long browsers cache preflight responses
/// 
/// # Security Notes
/// 
/// - Production should never use "*" for allowed origins
/// - Credentials should only be enabled with specific origins
/// - Headers should be limited to necessary values only
/// 
/// # Examples
/// 
/// ```rust
/// let cors_config = CorsConfig {
///     allowed_origins: vec!["https://myapp.com".to_string()],
///     allowed_methods: vec!["GET".to_string(), "POST".to_string()],
///     allowed_headers: vec!["authorization".to_string()],
///     expose_headers: vec!["x-request-id".to_string()],
///     allow_credentials: true,
///     max_age: Some(3600),
/// };
/// 
/// let cors_layer = build_cors_layer(&cors_config)?;
/// ```
fn build_cors_layer(cors_config: &CorsConfig) -> Result<CorsLayer, Box<dyn std::error::Error>> {
    let mut cors = CorsLayer::new();
    
    // Configure allowed origins
    if cors_config.allowed_origins.contains(&"*".to_string()) {
        cors = cors.allow_origin(Any);
    } else {
        let origins: Result<Vec<HeaderValue>, _> = cors_config.allowed_origins
            .iter()
            .map(|origin| origin.parse())
            .collect();
        cors = cors.allow_origin(origins?);
    }
    
    // Configure allowed methods
    if cors_config.allowed_methods.contains(&"*".to_string()) {
        cors = cors.allow_methods(Any);
    } else {
        let methods: Result<Vec<Method>, _> = cors_config.allowed_methods
            .iter()
            .map(|method| method.parse())
            .collect();
        cors = cors.allow_methods(methods?);
    }
    
    // Configure allowed headers
    if cors_config.allowed_headers.contains(&"*".to_string()) {
        cors = cors.allow_headers(Any);
    } else {
        let headers: Result<Vec<HeaderName>, _> = cors_config.allowed_headers
            .iter()
            .map(|header| header.parse())
            .collect();
        cors = cors.allow_headers(headers?);
    }
    
    // Configure exposed headers
    if !cors_config.expose_headers.is_empty() {
        let expose_headers: Result<Vec<HeaderName>, _> = cors_config.expose_headers
            .iter()
            .map(|header| header.parse())
            .collect();
        cors = cors.expose_headers(expose_headers?);
    }
    
    // Configure credentials
    cors = cors.allow_credentials(cors_config.allow_credentials);
    
    // Configure max age
    if let Some(max_age) = cors_config.max_age {
        cors = cors.max_age(std::time::Duration::from_secs(max_age));
    }
    
    Ok(cors)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    init_tracing();

    info!("Starting ERP Server...");

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded successfully");

    // Validate configuration security
    validate_configuration(&config)?;
    info!("Configuration validation passed");

    // Initialize database
    let db = DatabasePool::new(config.database.clone()).await?;
    info!("Database pool initialized");

    // Run migrations
    run_migrations(&db).await?;
    info!("Database migrations completed");

    // Initialize Redis
    let redis = init_redis(&config.redis.url).await?;
    info!("Redis connection established");

    // Initialize services
    let auth_service = Arc::new(
        AuthService::new(db.clone(), redis.clone(), config.clone()).await?
    );
    info!("Auth service initialized");

    // Create app state
    let app_state = AppState {
        config: config.clone(),
        db,
        redis,
        auth_service: auth_service.clone(),
    };

    // Build the application
    let app = create_app(app_state, auth_service)?;

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}

fn create_app(state: AppState, _auth_service: Arc<AuthService>) -> Result<Router, Box<dyn std::error::Error>> {
    // OpenAPI documentation
    #[derive(OpenApi)]
    #[openapi(
        paths(
            health::health_check,
            health::readiness_check,
        ),
        components(schemas()),
        tags(
            (name = "health", description = "Health check endpoints"),
            (name = "auth", description = "Authentication and authorization"),
            (name = "users", description = "User management"),
            (name = "roles", description = "Role and permission management"),
        )
    )]
    struct ApiDoc;

    // Build the router
    let router = Router::new()
        // API routes
        .nest("/api/v1", create_api_routes())
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Health checks
        .route("/health", axum::routing::get(health::health_check))
        .route("/ready", axum::routing::get(health::readiness_check))
        // Global middleware
        .layer(
            ServiceBuilder::new()
                // Security headers (applied first, to all responses)
                .layer(axum::middleware::from_fn(api_middleware::security_headers::security_headers_middleware))
                // Request ID middleware
                .layer(axum::middleware::from_fn(api_middleware::request_id::request_id_middleware))
                // Tenant context extraction
                .layer(axum::middleware::from_fn(api_middleware::tenant_context::tenant_context_middleware))
                // Logging and tracing
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                // Response compression
                .layer(CompressionLayer::new())
                // CORS (should be after security headers)
                .layer(build_cors_layer(&state.config.cors)?),
        )
        .with_state(state)
        // Fallback
        .fallback(handler_404);
    
    Ok(router)
}

/// Create the API routes
fn create_api_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::auth_routes())
        .nest("/users", users::user_routes())
        .nest("/roles", roles::role_routes())
        .nest("/customers", customers::customer_routes())
}

async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Resource not found"
        })),
    )
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "erp_api=debug,erp_auth=debug,erp_core=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn init_redis(url: &str) -> Result<ConnectionManager, redis::RedisError> {
    let client = redis::Client::open(url)?;
    ConnectionManager::new(client).await
}

async fn run_migrations(db: &DatabasePool) -> Result<(), sqlx::Error> {
    info!("Running database migrations...");

    // Use sqlx migrator to run all migrations in proper order
    let migrator = sqlx::migrate!("../../migrations");
    migrator.run(&db.main_pool).await?;

    info!("Migrations completed successfully");
    Ok(())
}

/// Validate configuration to ensure secure defaults are not used in production
fn validate_configuration(config: &Config) -> Result<(), Box<dyn std::error::Error>> {

    const DEFAULT_SECRETS: &[&str] = &[
        "your-super-secret-jwt-key-change-in-production-min-32-chars",
        "your-32-char-encryption-key-here!",
        "change_me_in_production",
        "placeholder",
        "default",
        "secret",
    ];

    const DEFAULT_PASSWORDS: &[&str] = &[
        "erp_secure_password_change_in_production",
        "redis_secure_password_change_in_production",
        "your-app-password",
        "password123",
        "admin",
    ];

    let mut errors = Vec::new();

    // Check environment
    let is_production = std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string())
        .to_lowercase() == "production";

    // JWT Secret validation
    if config.jwt.secret.len() < 32 {
        errors.push("JWT secret must be at least 32 characters long".to_string());
    }

    if DEFAULT_SECRETS.iter().any(|&s| config.jwt.secret.contains(s)) {
        errors.push("JWT secret contains default/insecure value".to_string());
    }

    // AES Encryption Key validation
    if config.security.aes_encryption_key.len() != 32 {
        errors.push("AES encryption key must be exactly 32 characters long".to_string());
    }

    if DEFAULT_SECRETS.iter().any(|&s| config.security.aes_encryption_key.contains(s)) {
        errors.push("AES encryption key contains default/insecure value".to_string());
    }

    // Database password validation (if extractable from URL)
    if let Some(password_start) = config.database.url.find(':') {
        if let Some(password_section) = config.database.url.get(password_start + 1..) {
            if let Some(at_sign) = password_section.find('@') {
                let password_part = &password_section[..at_sign];
                if let Some(colon) = password_part.rfind(':') {
                    let password = &password_part[colon + 1..];
                    if DEFAULT_PASSWORDS.iter().any(|&p| password == p) {
                        errors.push("Database password contains default/insecure value".to_string());
                    }
                }
            }
        }
    }

    // Redis password validation (if extractable from URL)
    if config.redis.url.contains("redis_secure_password_change_in_production") {
        errors.push("Redis password contains default/insecure value".to_string());
    }

    // Production-specific validations
    if is_production {
        // CORS validation
        if config.cors.allowed_origins.contains(&"*".to_string()) {
            errors.push("CORS allowed origins contains wildcard (*) in production".to_string());
        }

        // Email provider validation
        if config.email.provider == "mock" {
            errors.push("Email provider is set to 'mock' in production".to_string());
        }

        // Debug mode validation
        if std::env::var("DEBUG_MODE").unwrap_or_default() == "true" {
            errors.push("DEBUG_MODE is enabled in production".to_string());
        }

        // Token expiry validation
        if config.jwt.access_token_expiry > 3600 {
            errors.push("JWT access token expiry is too long for production (should be ≤ 1 hour)".to_string());
        }
    }

    // If there are errors, report them
    if !errors.is_empty() {
        eprintln!("\n⚠️  CONFIGURATION SECURITY ISSUES DETECTED ⚠️");
        eprintln!("================================================");
        for (i, error) in errors.iter().enumerate() {
            eprintln!("{}. {}", i + 1, error);
        }
        eprintln!("================================================");

        if is_production {
            eprintln!("\n🛑 REFUSING TO START IN PRODUCTION WITH SECURITY ISSUES");
            eprintln!("Please fix the above issues before deploying to production.\n");
            return Err("Configuration validation failed: security issues detected".into());
        } else {
            eprintln!("\n⚠️  WARNING: Running with insecure configuration in development mode");
            eprintln!("These issues MUST be fixed before deploying to production.\n");
        }
    } else {
        info!("✅ Configuration security validation passed");
    }

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received terminate signal");
        },
    }
}