//! ERP System Deployment CLI Tool
//!
//! A comprehensive command-line tool for deploying and managing ERP system instances.
//! Supports installation, tenant management, database operations, and monitoring.

use clap::{Parser, Subcommand};
use colored::*;
use std::process;

mod commands;
mod config;
mod utils;

use commands::*;
use erp_deploy::{DatabaseCommands, TenantCommands, DockerCommands, BackupCommands, ConfigCommands};

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

// TenantCommands moved to lib.rs - using import above

// DatabaseCommands moved to lib.rs - using import above

// DockerCommands moved to lib.rs - using import above

// ConfigCommands and BackupCommands moved to lib.rs - using import above

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
            status::execute(detailed, format, None).await
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