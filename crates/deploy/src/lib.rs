//! ERP System Deployment CLI Library
//!
//! This library provides the core functionality for the ERP deployment CLI tool.

use clap::Subcommand;

pub mod commands;
pub mod config;
pub mod utils;

// Re-export commonly used types
pub use config::Config;

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show {
        /// Configuration section
        section: Option<String>,
        /// Output format (table, json, yaml, toml)
        format: String,
    },
    /// Set configuration value
    Set {
        /// Configuration key (e.g., server.port)
        key: String,
        /// Configuration value
        value: String,
        /// Configuration scope (global, user, local)
        scope: Option<String>,
        /// Tenant ID for tenant-specific config
        tenant: Option<String>,
    },
    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
        /// Configuration scope
        scope: Option<String>,
        /// Tenant ID
        tenant: Option<String>,
    },
    /// Validate configuration file
    Validate {
        /// Configuration file path
        file: Option<String>,
        /// Show detailed validation output
        detailed: bool,
    },
    /// Generate configuration template
    Generate {
        /// Target environment (development, staging, production)
        environment: String,
        /// Output file path
        output: Option<String>,
    },
}

// Error types
pub type Result<T> = anyhow::Result<T>;

// Command enums
#[derive(Subcommand)]
pub enum TenantCommands {
    /// Create a new tenant
    Create {
        /// Tenant name
        name: String,
        /// Admin email
        email: String,
        /// Admin password (will prompt if not provided)
        password: Option<String>,
        /// Tenant domain
        domain: Option<String>,
        /// Database schema name
        schema: Option<String>,
    },
    /// List all tenants
    List {
        /// Output format (table, json, yaml)
        format: String,
        /// Include inactive tenants
        include_inactive: bool,
    },
    /// Show tenant details
    Show {
        /// Tenant ID or name
        tenant: String,
    },
    /// Update tenant settings
    Update {
        /// Tenant ID or name
        tenant: String,
        /// New tenant name
        name: Option<String>,
        /// New admin email
        email: Option<String>,
    },
    /// Delete a tenant
    Delete {
        /// Tenant ID or name
        tenant: String,
        /// Force deletion without confirmation
        force: bool,
        /// Keep database schema
        keep_schema: bool,
    },
}

#[derive(Subcommand)]
pub enum DatabaseCommands {
    /// Run database migrations
    Migrate {
        /// Dry run only
        dry_run: bool,
        /// Target tenant
        tenant: Option<String>,
        /// Migration target
        target: Option<String>,
    },
    /// Create database backup
    Backup {
        /// Backup name
        name: String,
        /// Output directory
        output: Option<String>,
    },
    /// Restore from backup
    Restore {
        /// Backup name
        backup: String,
        /// Force restore
        force: bool,
    },
    /// Check database health
    Check {
        /// Detailed check
        detailed: bool,
    },
    /// Show migration status
    Status,
    /// Reset database
    Reset {
        /// Force reset without confirmation
        force: bool,
        /// Target tenant
        tenant: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum DockerCommands {
    /// Start services
    Start {
        /// Service name (optional)
        service: Option<String>,
        /// Services to start
        services: Vec<String>,
        /// Run in detached mode
        detach: bool,
    },
    /// Stop services
    Stop {
        /// Service name (optional)
        service: Option<String>,
        /// Services to stop
        services: Vec<String>,
        /// Force stop
        force: bool,
    },
    /// Restart services
    Restart {
        /// Services to restart
        services: Vec<String>,
    },
    /// Show service status
    Status {
        /// Output format
        format: String,
    },
    /// Show service logs
    Logs {
        /// Service name
        service: String,
        /// Follow logs
        follow: bool,
    },
    /// Update containers
    Update {
        /// Force update
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum BackupCommands {
    /// Create backup
    Create {
        /// Backup name
        name: String,
        /// Output directory
        output: Option<String>,
        /// Include patterns
        include: Vec<String>,
        /// Exclude patterns
        exclude: Vec<String>,
        /// Compression type
        compression: String,
    },
    /// List backups
    List {
        /// Backup directory
        directory: Option<String>,
        /// Output format
        format: String,
    },
    /// Restore backup
    Restore {
        /// Backup name
        name: String,
        /// Backup file path
        backup: String,
        /// Force restore
        force: bool,
        /// Components to restore
        components: Vec<String>,
    },
    /// Verify backup integrity
    Verify {
        /// Backup name
        name: String,
        /// Detailed verification
        detailed: bool,
    },
    /// Cleanup old backups
    Cleanup {
        /// Keep last N backups
        keep: usize,
        /// Dry run
        dry_run: bool,
    },
}