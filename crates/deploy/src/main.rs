//! ERP System Deployment CLI Tool
//!
//! A comprehensive command-line tool for deploying and managing ERP system instances.
//! Supports installation, tenant management, database operations, and monitoring.

use clap::{Parser, Subcommand};
use colored::*;
use std::process;

mod commands;
mod config;
mod database;
mod docker;
mod tenant;
mod utils;

use commands::*;

#[derive(Parser)]
#[command(name = "erp-deploy")]
#[command(version = "1.0.0")]
#[command(about = "ERP System Deployment and Management CLI")]
#[command(long_about = "
ERP System Deployment CLI - Complete deployment and management tool

This tool provides comprehensive functionality for deploying, configuring,
and managing ERP system instances. It supports:

• Fresh installations with automatic setup
• Multi-tenant management and provisioning
• Database migration and maintenance
• Docker container orchestration
• Health monitoring and diagnostics
• Backup and recovery operations

Examples:
  erp-deploy install --environment production
  erp-deploy tenant create --name \"Acme Corp\" --email admin@acme.com
  erp-deploy database migrate --tenant acme_corp
  erp-deploy health check --all
")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Database URL
    #[arg(long, env = "DATABASE_URL", global = true)]
    database_url: Option<String>,

    /// Skip confirmation prompts
    #[arg(short, long, global = true)]
    yes: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Install ERP system
    #[command(about = "Install ERP system on a fresh server")]
    Install {
        /// Installation environment
        #[arg(short, long, default_value = "production")]
        environment: String,

        /// Skip security hardening
        #[arg(long)]
        skip_security: bool,

        /// Custom installation directory
        #[arg(long, default_value = "/opt/erp-system")]
        install_dir: String,

        /// Domain name for SSL certificates
        #[arg(long)]
        domain: Option<String>,

        /// Admin email for notifications
        #[arg(long)]
        admin_email: Option<String>,
    },

    /// Tenant management commands
    #[command(subcommand)]
    #[command(about = "Manage tenants (create, list, delete)")]
    Tenant(TenantCommands),

    /// Database management commands
    #[command(subcommand)]
    #[command(about = "Database operations (migrate, backup, restore)")]
    Database(DatabaseCommands),

    /// Docker management commands
    #[command(subcommand)]
    #[command(about = "Docker container management")]
    Docker(DockerCommands),

    /// Health check and monitoring
    #[command(about = "Check system health and status")]
    Health {
        /// Check all components
        #[arg(short, long)]
        all: bool,

        /// Check specific component
        #[arg(short, long)]
        component: Option<String>,

        /// Output format (table, json, yaml)
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Configuration management
    #[command(subcommand)]
    #[command(about = "Manage system configuration")]
    Config(ConfigCommands),

    /// Backup and restore operations
    #[command(subcommand)]
    #[command(about = "Backup and restore operations")]
    Backup(BackupCommands),

    /// Log management and analysis
    #[command(about = "View and analyze system logs")]
    Logs {
        /// Component to view logs for
        #[arg(short, long)]
        component: Option<String>,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show
        #[arg(short, long, default_value = "100")]
        lines: usize,

        /// Show logs since timestamp
        #[arg(long)]
        since: Option<String>,
    },

    /// System status and information
    #[command(about = "Show detailed system information")]
    Status {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,

        /// Output format (table, json, yaml)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}

#[derive(Subcommand)]
enum TenantCommands {
    /// Create a new tenant
    Create {
        /// Tenant name
        #[arg(short, long)]
        name: String,

        /// Admin email
        #[arg(short, long)]
        email: String,

        /// Admin password (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,

        /// Tenant domain
        #[arg(short, long)]
        domain: Option<String>,

        /// Database schema name
        #[arg(short, long)]
        schema: Option<String>,
    },

    /// List all tenants
    List {
        /// Output format (table, json, yaml)
        #[arg(short, long, default_value = "table")]
        format: String,

        /// Show inactive tenants
        #[arg(long)]
        include_inactive: bool,
    },

    /// Show tenant details
    Show {
        /// Tenant ID or schema name
        tenant: String,

        /// Output format (table, json, yaml)
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Update tenant settings
    Update {
        /// Tenant ID or schema name
        tenant: String,

        /// New tenant name
        #[arg(long)]
        name: Option<String>,

        /// New status (active, suspended, inactive)
        #[arg(long)]
        status: Option<String>,
    },

    /// Delete a tenant
    Delete {
        /// Tenant ID or schema name
        tenant: String,

        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,

        /// Keep database schema (soft delete)
        #[arg(long)]
        keep_schema: bool,
    },
}

#[derive(Subcommand)]
enum DatabaseCommands {
    /// Run database migrations
    Migrate {
        /// Specific tenant schema
        #[arg(short, long)]
        tenant: Option<String>,

        /// Target migration version
        #[arg(long)]
        target: Option<i64>,

        /// Dry run (show what would be applied)
        #[arg(long)]
        dry_run: bool,
    },

    /// Create database backup
    Backup {
        /// Specific tenant to backup
        #[arg(short, long)]
        tenant: Option<String>,

        /// Backup destination path
        #[arg(short, long)]
        output: Option<String>,

        /// Compression format (gzip, bzip2, none)
        #[arg(short, long, default_value = "gzip")]
        compression: String,
    },

    /// Restore database from backup
    Restore {
        /// Backup file path
        backup_file: String,

        /// Target tenant schema
        #[arg(short, long)]
        tenant: String,

        /// Force restore without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Database health check
    Check {
        /// Check connections
        #[arg(long)]
        connections: bool,

        /// Check schema integrity
        #[arg(long)]
        schema: bool,

        /// Check performance
        #[arg(long)]
        performance: bool,
    },

    /// Reset database (DANGEROUS)
    Reset {
        /// Specific tenant schema to reset
        #[arg(short, long)]
        tenant: Option<String>,

        /// Force reset without confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum DockerCommands {
    /// Start ERP system containers
    Start {
        /// Service names to start
        services: Vec<String>,

        /// Start in detached mode
        #[arg(short, long)]
        detach: bool,
    },

    /// Stop ERP system containers
    Stop {
        /// Service names to stop
        services: Vec<String>,

        /// Force stop (SIGKILL)
        #[arg(short, long)]
        force: bool,
    },

    /// Restart ERP system containers
    Restart {
        /// Service names to restart
        services: Vec<String>,
    },

    /// Show container status
    Status {
        /// Output format (table, json, yaml)
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// View container logs
    Logs {
        /// Service name
        service: String,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to show
        #[arg(short, long, default_value = "100")]
        lines: usize,
    },

    /// Update container images
    Update {
        /// Force update without confirmation
        #[arg(short, long)]
        force: bool,

        /// Specific services to update
        services: Vec<String>,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show {
        /// Configuration section
        #[arg(short, long)]
        section: Option<String>,

        /// Output format (table, json, yaml, toml)
        #[arg(short, long, default_value = "toml")]
        format: String,
    },

    /// Set configuration value
    Set {
        /// Configuration key (e.g., server.port)
        key: String,

        /// Configuration value
        value: String,

        /// Configuration scope (system, tenant)
        #[arg(short, long, default_value = "system")]
        scope: String,

        /// Tenant for tenant-scoped settings
        #[arg(short, long)]
        tenant: Option<String>,
    },

    /// Get configuration value
    Get {
        /// Configuration key
        key: String,

        /// Configuration scope (system, tenant)
        #[arg(short, long, default_value = "system")]
        scope: String,

        /// Tenant for tenant-scoped settings
        #[arg(short, long)]
        tenant: Option<String>,
    },

    /// Validate configuration
    Validate {
        /// Configuration file path
        #[arg(short, long)]
        file: Option<String>,

        /// Show detailed validation results
        #[arg(short, long)]
        detailed: bool,
    },

    /// Generate configuration template
    Generate {
        /// Environment (development, staging, production)
        #[arg(short, long, default_value = "production")]
        environment: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
enum BackupCommands {
    /// Create full system backup
    Create {
        /// Backup destination directory
        #[arg(short, long, default_value = "/opt/erp-system/backups")]
        output: String,

        /// Include specific components
        #[arg(short, long)]
        include: Vec<String>,

        /// Exclude specific components
        #[arg(long)]
        exclude: Vec<String>,

        /// Compression level (0-9)
        #[arg(short, long, default_value = "6")]
        compression: u8,
    },

    /// List available backups
    List {
        /// Backup directory
        #[arg(short, long, default_value = "/opt/erp-system/backups")]
        directory: String,

        /// Output format (table, json, yaml)
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Restore from backup
    Restore {
        /// Backup file or directory
        backup: String,

        /// Force restore without confirmation
        #[arg(short, long)]
        force: bool,

        /// Restore specific components only
        #[arg(short, long)]
        components: Vec<String>,
    },

    /// Verify backup integrity
    Verify {
        /// Backup file or directory
        backup: String,

        /// Detailed verification
        #[arg(short, long)]
        detailed: bool,
    },

    /// Cleanup old backups
    Cleanup {
        /// Backup directory
        #[arg(short, long, default_value = "/opt/erp-system/backups")]
        directory: String,

        /// Keep backups newer than days
        #[arg(short, long, default_value = "30")]
        keep_days: u32,

        /// Dry run (show what would be deleted)
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    init_logging(cli.verbose);

    // Load configuration
    let config = match config::load_config(cli.config.as_deref()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{} Failed to load configuration: {}", "Error:".red().bold(), e);
            process::exit(1);
        }
    };

    // Execute command
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(execute_command(cli, config));

    match result {
        Ok(_) => {
            println!("{}", "✅ Command completed successfully".green().bold());
        }
        Err(e) => {
            eprintln!("{} {}", "❌ Error:".red().bold(), e);
            process::exit(1);
        }
    }
}

async fn execute_command(cli: Cli, config: config::Config) -> anyhow::Result<()> {
    match cli.command {
        Commands::Install {
            environment,
            skip_security,
            install_dir,
            domain,
            admin_email
        } => {
            install::execute(
                &environment,
                skip_security,
                &install_dir,
                domain.as_deref(),
                admin_email.as_deref()
            ).await
        }

        Commands::Tenant(cmd) => {
            tenant::execute_tenant_command(cmd, &config, cli.database_url.as_deref()).await
        }

        Commands::Database(cmd) => {
            database::execute_database_command(cmd, &config, cli.database_url.as_deref()).await
        }

        Commands::Docker(cmd) => {
            docker::execute_docker_command(cmd).await
        }

        Commands::Health { all, component, format } => {
            health::execute(all, component.as_deref(), &format, &config).await
        }

        Commands::Config(cmd) => {
            config::execute_config_command(cmd, &config).await
        }

        Commands::Backup(cmd) => {
            backup::execute_backup_command(cmd, &config).await
        }

        Commands::Logs { component, follow, lines, since } => {
            logs::execute(component.as_deref(), follow, lines, since.as_deref()).await
        }

        Commands::Status { detailed, format } => {
            status::execute(detailed, &format, &config).await
        }
    }
}

fn init_logging(verbosity: u8) {
    let level = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    std::env::set_var("RUST_LOG", level);
    env_logger::init();
}