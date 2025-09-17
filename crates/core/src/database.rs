//! # Multi-Tenant Database Pool Management
//! 
//! This module provides a sophisticated database connection pool system designed
//! for multi-tenant applications with schema-per-tenant isolation.
//! 
//! ## Architecture Overview
//! 
//! ```text
//! ┌─────────────────┐    ┌──────────────┐    ┌─────────────────┐
//! │   Main Pool     │    │ Tenant Pool  │    │ Tenant Pool     │
//! │   (public)      │    │ (tenant_a)   │    │ (tenant_b)      │
//! │                 │    │              │    │                 │
//! │ - Metadata      │    │ - User Data  │    │ - User Data     │
//! │ - Tenant Info   │    │ - Business   │    │ - Business      │
//! │ - Global Config │    │   Logic      │    │   Logic         │
//! └─────────────────┘    └──────────────┘    └─────────────────┘
//! ```
//! 
//! ## Multi-Tenancy Model
//! 
//! - **Schema-per-tenant**: Each tenant gets an isolated PostgreSQL schema
//! - **Connection pooling**: Efficient connection reuse with tenant-specific pools
//! - **Dynamic schema creation**: Automatic schema setup for new tenants
//! - **Query isolation**: SET search_path ensures proper tenant data isolation
//! 
//! ## Usage Example
//! 
//! ```rust
//! use erp_core::{DatabasePool, TenantContext};
//! use erp_core::config::DatabaseConfig;
//! 
//! // Initialize the database pool
//! let config = DatabaseConfig {
//!     url: "postgresql://user:pass@localhost/db".to_string(),
//!     max_connections: 20,
//!     min_connections: 5,
//! };
//! let db = DatabasePool::new(config).await?;
//! 
//! // Get a tenant-specific connection pool
//! let tenant = TenantContext { 
//!     id: uuid::Uuid::new_v4(),
//!     schema_name: "tenant_12345".to_string(),
//! };
//! let tenant_pool = db.get_tenant_pool(&tenant).await?;
//! 
//! // Execute tenant-specific queries
//! let users = sqlx::query("SELECT * FROM users")
//!     .fetch_all(tenant_pool.get())
//!     .await?;
//! ```

use crate::{config::DatabaseConfig, error::Result, Error, TenantContext};
use dashmap::DashMap;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tracing::{debug, error, info};

/// Main database pool manager for multi-tenant applications.
/// 
/// `DatabasePool` manages both the main PostgreSQL connection pool (for metadata and
/// global operations) and tenant-specific pools (for business data). Each tenant
/// operates in an isolated schema for complete data separation.
/// 
/// ## Thread Safety
/// 
/// This struct is designed to be safely shared across async tasks. The internal
/// `tenant_pools` cache uses `DashMap` for concurrent access without locking.
/// 
/// ## Connection Pool Strategy
/// 
/// - **Main Pool**: Full connection allocation for metadata operations
/// - **Tenant Pools**: 1/4 of main pool size per tenant for efficient resource usage
/// - **Dynamic Creation**: Tenant pools are created on-demand and cached
/// - **Connection Reuse**: Connections are automatically configured with tenant search_path
/// 
/// ## Performance Considerations
/// 
/// - Tenant pools are cached indefinitely (consider implementing eviction for large systems)
/// - Connection establishment includes automatic schema path configuration
/// - Pool size is divided among tenants to prevent resource exhaustion
#[derive(Clone)]
pub struct DatabasePool {
    /// Main PostgreSQL connection pool for global operations.
    /// 
    /// Used for:
    /// - Tenant management operations
    /// - Schema creation and deletion
    /// - Global metadata queries
    /// - Health checks and monitoring
    pub main_pool: PgPool,
    
    /// Cache of tenant-specific connection pools.
    /// 
    /// Each pool is pre-configured with the appropriate schema search_path
    /// for transparent tenant isolation. Pools are created lazily and
    /// cached for performance.
    tenant_pools: Arc<DashMap<String, PgPool>>,
    
    /// Database configuration used for pool creation.
    config: DatabaseConfig,
}

impl DatabasePool {
    /// Creates a new database pool manager with the specified configuration.
    /// 
    /// This method establishes the main PostgreSQL connection pool and initializes
    /// the tenant pool cache. The main pool is used for global operations while
    /// tenant pools will be created on-demand.
    /// 
    /// # Arguments
    /// 
    /// * `config` - Database configuration including connection URL and pool limits
    /// 
    /// # Returns
    /// 
    /// Returns a new `DatabasePool` instance ready for multi-tenant operations.
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Database connection cannot be established
    /// - Invalid connection URL format
    /// - Authentication fails
    /// - Network connectivity issues
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use erp_core::{DatabasePool, config::DatabaseConfig};
    /// 
    /// let config = DatabaseConfig {
    ///     url: "postgresql://user:pass@localhost/db".to_string(),
    ///     max_connections: 20,
    ///     min_connections: 5,
    /// };
    /// 
    /// let db = DatabasePool::new(config).await?;
    /// ```
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        info!("Initializing main database pool");
        
        let main_pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect(&config.url)
            .await?;

        info!("Main database pool initialized successfully");

        Ok(Self {
            main_pool,
            tenant_pools: Arc::new(DashMap::new()),
            config,
        })
    }

    /// Retrieves or creates a tenant-specific database connection pool.
    /// 
    /// This method implements a caching strategy where tenant pools are created
    /// on first access and then reused. Each pool is automatically configured
    /// with the tenant's schema in its search_path for transparent isolation.
    /// 
    /// # Arguments
    /// 
    /// * `tenant` - Tenant context containing schema name and metadata
    /// 
    /// # Returns
    /// 
    /// Returns a `TenantPool` configured for the specific tenant's schema.
    /// 
    /// # Performance Notes
    /// 
    /// - First access: Creates new pool (slower)
    /// - Subsequent access: Returns cached pool (fast)
    /// - Thread-safe concurrent access via DashMap
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use erp_core::{DatabasePool, TenantContext};
    /// 
    /// let tenant = TenantContext {
    ///     id: uuid::Uuid::new_v4(),
    ///     schema_name: "tenant_acme_corp".to_string(),
    /// };
    /// 
    /// let tenant_pool = db.get_tenant_pool(&tenant).await?;
    /// let connection = tenant_pool.get();
    /// ```
    pub async fn get_tenant_pool(&self, tenant: &TenantContext) -> Result<TenantPool> {
        let schema_name = &tenant.schema_name;
        
        if let Some(pool) = self.tenant_pools.get(schema_name) {
            debug!("Using cached pool for tenant schema: {}", schema_name);
            return Ok(TenantPool {
                pool: pool.clone(),
                schema_name: schema_name.clone(),
            });
        }

        debug!("Creating new pool for tenant schema: {}", schema_name);
        
        let pool = self.create_tenant_pool(schema_name).await?;
        self.tenant_pools.insert(schema_name.clone(), pool.clone());
        
        Ok(TenantPool {
            pool,
            schema_name: schema_name.clone(),
        })
    }

    async fn create_tenant_pool(&self, schema_name: &str) -> Result<PgPool> {
        let schema = schema_name.to_string();
        let pool = PgPoolOptions::new()
            .max_connections(std::cmp::max(1, self.config.max_connections / 4))
            .min_connections(1)
            .after_connect(move |conn, _meta| {
                let schema = schema.clone();
                Box::pin(async move {
                    sqlx::query(&format!("SET search_path TO {}, public", schema))
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(&self.config.url)
            .await?;

        Ok(pool)
    }

    /// Validates schema name to prevent SQL injection
    /// Only allows safe characters: a-zA-Z0-9_ and must start with letter/underscore
    fn validate_schema_name(schema_name: &str) -> Result<()> {
        // Check length constraints (PostgreSQL limit: 63 characters)
        if schema_name.is_empty() || schema_name.len() > 63 {
            return Err(Error::validation("Schema name must be 1-63 characters long"));
        }

        // Check first character: must be letter or underscore
        let first_char = schema_name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return Err(Error::validation("Schema name must start with letter or underscore"));
        }

        // Check all characters: only alphanumeric and underscore allowed
        if !schema_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(Error::validation("Schema name can only contain letters, numbers, and underscores"));
        }

        // Reject PostgreSQL reserved words and dangerous patterns
        let reserved_words = ["public", "information_schema", "pg_catalog", "pg_toast"];
        if reserved_words.contains(&schema_name.to_lowercase().as_str()) {
            return Err(Error::validation("Schema name cannot be a PostgreSQL reserved word"));
        }

        Ok(())
    }

    pub async fn create_tenant_schema(&self, schema_name: &str) -> Result<()> {
        // SECURITY: Validate schema name to prevent SQL injection
        Self::validate_schema_name(schema_name)?;
        
        info!("Creating tenant schema: {}", schema_name);

        // Use parameterized query - but PostgreSQL doesn't support parameters for DDL
        // So we validate the schema name strictly and use format as fallback
        let sql = format!("CREATE SCHEMA IF NOT EXISTS \"{}\"", schema_name);
        sqlx::query(&sql)
            .execute(&self.main_pool)
            .await?;

        let setup_sql = include_str!("../sql/tenant_schema.sql");
        let queries: Vec<&str> = setup_sql.split(';').filter(|s| !s.trim().is_empty()).collect();

        for query in queries {
            // SECURITY: Schema name is already validated, but quote it for extra safety
            let formatted_query = query.replace("{{schema}}", &format!("\"{}\"", schema_name));
            sqlx::query(&formatted_query)
                .execute(&self.main_pool)
                .await
                .map_err(|e| {
                    error!("Failed to execute tenant schema setup query: {}", e);
                    e
                })?;
        }

        info!("Tenant schema created successfully: {}", schema_name);
        Ok(())
    }

    pub async fn drop_tenant_schema(&self, schema_name: &str) -> Result<()> {
        // SECURITY: Validate schema name to prevent SQL injection
        Self::validate_schema_name(schema_name)?;
        
        info!("Dropping tenant schema: {}", schema_name);
        
        self.tenant_pools.remove(schema_name);
        
        // Use quoted identifier to prevent SQL injection
        let sql = format!("DROP SCHEMA IF EXISTS \"{}\" CASCADE", schema_name);
        sqlx::query(&sql)
            .execute(&self.main_pool)
            .await?;

        info!("Tenant schema dropped successfully: {}", schema_name);
        Ok(())
    }

    pub async fn check_health(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.main_pool)
            .await?;
        Ok(())
    }
}

/// A tenant-specific database connection pool with pre-configured schema isolation.
/// 
/// `TenantPool` wraps a PostgreSQL connection pool that has been configured
/// to automatically use a specific tenant's schema. All queries executed
/// through this pool will operate within the tenant's isolated data space.
/// 
/// ## Schema Isolation
/// 
/// Each connection in the pool is configured with:
/// - `SET search_path TO {tenant_schema}, public`
/// - This ensures queries default to the tenant's schema
/// - Falls back to `public` schema for shared resources
/// 
/// ## Usage Patterns
/// 
/// ```rust
/// // Get tenant pool
/// let tenant_pool = db.get_tenant_pool(&tenant_context).await?;
/// 
/// // Execute tenant-scoped queries
/// let users = sqlx::query_as::<_, User>("SELECT * FROM users")
///     .fetch_all(tenant_pool.get())
///     .await?;
/// 
/// // All queries automatically operate in tenant's schema
/// let user_count = sqlx::query_scalar("SELECT COUNT(*) FROM users")
///     .fetch_one(tenant_pool.get())
///     .await?;
/// ```
/// 
/// ## Thread Safety
/// 
/// This struct is `Clone` and can be safely shared across async tasks.
/// The underlying `PgPool` handles concurrent access internally.
#[derive(Clone)]
pub struct TenantPool {
    /// The PostgreSQL connection pool configured for this tenant.
    /// 
    /// Each connection in this pool has the search_path set to the
    /// tenant's schema, providing automatic query isolation.
    pub pool: PgPool,
    
    /// The schema name this pool is configured for.
    /// 
    /// Used for debugging, logging, and potential cleanup operations.
    /// This matches the tenant's isolated schema in the database.
    pub schema_name: String,
}

impl TenantPool {
    /// Returns a reference to the underlying PostgreSQL connection pool.
    /// 
    /// Use this method to execute queries within the tenant's schema.
    /// All connections from this pool are pre-configured with the
    /// appropriate search_path for tenant isolation.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let tenant_pool = db.get_tenant_pool(&tenant).await?;
    /// 
    /// // Execute a query in the tenant's schema
    /// let result = sqlx::query("SELECT * FROM users")
    ///     .fetch_all(tenant_pool.get())
    ///     .await?;
    /// 
    /// // Start a transaction
    /// let mut tx = tenant_pool.get().begin().await?;
    /// sqlx::query("INSERT INTO users (name) VALUES ($1)")
    ///     .bind("Alice")
    ///     .execute(&mut *tx)
    ///     .await?;
    /// tx.commit().await?;
    /// ```
    pub fn get(&self) -> &PgPool {
        &self.pool
    }
}