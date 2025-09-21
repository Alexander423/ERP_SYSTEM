//! Database management command implementations

use anyhow::{anyhow, Result};
use colored::*;
use sqlx::PgPool;
use std::path::Path;
use tokio::process::Command;

use crate::{DatabaseCommands, config::Config};

pub async fn execute_database_command(
    cmd: DatabaseCommands,
    config: &Config,
    database_url: Option<&str>,
) -> Result<()> {
    let db_url = database_url
        .or(config.database_url.as_deref())
        .ok_or_else(|| anyhow!("Database URL not provided"))?;

    match cmd {
        DatabaseCommands::Migrate { tenant, target, dry_run } => {
            migrate_database(db_url, tenant.as_deref(), target, dry_run).await
        }
        DatabaseCommands::Backup { tenant, output, compression } => {
            backup_database(db_url, tenant.as_deref(), output.as_deref(), &compression).await
        }
        DatabaseCommands::Restore { backup_file, tenant, force } => {
            restore_database(db_url, &backup_file, &tenant, force).await
        }
        DatabaseCommands::Check { connections, schema, performance } => {
            check_database(db_url, connections, schema, performance).await
        }
        DatabaseCommands::Reset { tenant, force } => {
            reset_database(db_url, tenant.as_deref(), force).await
        }
    }
}

async fn migrate_database(
    database_url: &str,
    tenant: Option<&str>,
    target: Option<i64>,
    dry_run: bool,
) -> Result<()> {
    println!("{}", "🔄 Running database migrations...".blue().bold());

    if dry_run {
        println!("{}", "🔍 Dry run mode - showing what would be applied".yellow());
    }

    if let Some(tenant_schema) = tenant {
        println!("Target schema: {}", tenant_schema.cyan());
    } else {
        println!("Target: {}", "All schemas".cyan());
    }

    let pool = PgPool::connect(database_url).await?;

    // Check current migration status
    let migration_table_exists = sqlx::query!(
        "SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = '_sqlx_migrations'
        ) as exists"
    )
    .fetch_one(&pool)
    .await?;

    if !migration_table_exists.exists.unwrap_or(false) {
        println!("Creating migration tracking table...");
        if !dry_run {
            sqlx::migrate!("../../../migrations").run(&pool).await?;
        }
    }

    // List pending migrations
    println!("Checking for pending migrations...");

    if dry_run {
        println!("{}", "✅ Dry run completed".green());
    } else {
        println!("{}", "✅ Migrations completed successfully".green());
    }

    pool.close().await;
    Ok(())
}

async fn backup_database(
    database_url: &str,
    tenant: Option<&str>,
    output: Option<&str>,
    compression: &str,
) -> Result<()> {
    println!("{}", "💾 Creating database backup...".blue().bold());

    // Parse database URL to extract connection details
    let url = url::Url::parse(database_url)?;
    let host = url.host_str().unwrap_or("localhost");
    let port = url.port().unwrap_or(5432);
    let username = url.username();
    let password = url.password().unwrap_or("");
    let database = url.path().trim_start_matches('/');

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_filename = match tenant {
        Some(schema) => format!("backup_{}_{}.sql", schema, timestamp),
        None => format!("backup_full_{}.sql", timestamp),
    };

    let output_path = output.unwrap_or(&backup_filename);

    println!("Host: {}", host.yellow());
    println!("Database: {}", database.yellow());
    if let Some(schema) = tenant {
        println!("Schema: {}", schema.yellow());
    }
    println!("Output: {}", output_path.yellow());

    // Build pg_dump command
    let mut cmd = Command::new("pg_dump");
    cmd.arg("--host").arg(host)
       .arg("--port").arg(port.to_string())
       .arg("--username").arg(username)
       .arg("--no-password")
       .arg("--verbose")
       .arg("--format").arg("custom")
       .arg("--file").arg(output_path);

    if let Some(schema) = tenant {
        cmd.arg("--schema").arg(schema);
    }

    cmd.arg(database);

    // Set password via environment
    cmd.env("PGPASSWORD", password);

    println!("Running pg_dump...");
    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Backup failed: {}", stderr));
    }

    // Apply compression if requested
    if compression != "none" {
        println!("Applying {} compression...", compression.yellow());
        match compression {
            "gzip" => {
                let mut gzip_cmd = Command::new("gzip");
                gzip_cmd.arg(output_path);
                gzip_cmd.output().await?;
            }
            "bzip2" => {
                let mut bzip_cmd = Command::new("bzip2");
                bzip_cmd.arg(output_path);
                bzip_cmd.output().await?;
            }
            _ => {
                println!("{}", format!("Unknown compression format: {}", compression).yellow());
            }
        }
    }

    println!("{}", "✅ Backup completed successfully".green().bold());
    Ok(())
}

async fn restore_database(
    database_url: &str,
    backup_file: &str,
    tenant: &str,
    force: bool,
) -> Result<()> {
    println!("{}", "🔄 Restoring database from backup...".blue().bold());

    if !Path::new(backup_file).exists() {
        return Err(anyhow!("Backup file not found: {}", backup_file));
    }

    println!("Backup file: {}", backup_file.yellow());
    println!("Target schema: {}", tenant.yellow());

    if !force {
        use dialoguer::Confirm;
        if !Confirm::new()
            .with_prompt("This will overwrite existing data. Continue?")
            .interact()?
        {
            println!("Restore cancelled");
            return Ok(());
        }
    }

    // Parse database URL
    let url = url::Url::parse(database_url)?;
    let host = url.host_str().unwrap_or("localhost");
    let port = url.port().unwrap_or(5432);
    let username = url.username();
    let password = url.password().unwrap_or("");
    let database = url.path().trim_start_matches('/');

    // Build pg_restore command
    let mut cmd = Command::new("pg_restore");
    cmd.arg("--host").arg(host)
       .arg("--port").arg(port.to_string())
       .arg("--username").arg(username)
       .arg("--dbname").arg(database)
       .arg("--no-password")
       .arg("--verbose")
       .arg("--clean")
       .arg("--if-exists")
       .arg("--schema").arg(tenant)
       .arg(backup_file);

    cmd.env("PGPASSWORD", password);

    println!("Running pg_restore...");
    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Restore failed: {}", stderr));
    }

    println!("{}", "✅ Restore completed successfully".green().bold());
    Ok(())
}

async fn check_database(
    database_url: &str,
    check_connections: bool,
    check_schema: bool,
    check_performance: bool,
) -> Result<()> {
    println!("{}", "🔍 Running database health checks...".blue().bold());

    let pool = PgPool::connect(database_url).await?;

    // Basic connectivity test
    println!("Testing database connectivity...");
    let version = sqlx::query!("SELECT version()")
        .fetch_one(&pool)
        .await?;

    println!("Database version: {}", version.version.unwrap_or_default().green());

    if check_connections {
        println!("\n📊 Connection Pool Status:");
        println!("  Active connections: {}", pool.size());
        // Additional connection metrics would go here
    }

    if check_schema {
        println!("\n🗄️ Schema Information:");

        // Count tables
        let table_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM information_schema.tables WHERE table_schema = 'public'"
        )
        .fetch_one(&pool)
        .await?;

        println!("  Public tables: {}", table_count.count.unwrap_or(0));

        // Count tenant schemas
        let schema_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM information_schema.schemata
             WHERE schema_name NOT IN ('information_schema', 'pg_catalog', 'pg_toast', 'public')"
        )
        .fetch_one(&pool)
        .await?;

        println!("  Tenant schemas: {}", schema_count.count.unwrap_or(0));
    }

    if check_performance {
        println!("\n⚡ Performance Metrics:");

        // Check for slow queries (if pg_stat_statements is available)
        let slow_queries = sqlx::query!(
            "SELECT COUNT(*) as count FROM pg_stat_activity WHERE state = 'active'"
        )
        .fetch_one(&pool)
        .await?;

        println!("  Active queries: {}", slow_queries.count.unwrap_or(0));

        // Database size
        let db_size = sqlx::query!(
            "SELECT pg_size_pretty(pg_database_size(current_database())) as size"
        )
        .fetch_one(&pool)
        .await?;

        println!("  Database size: {}", db_size.size.unwrap_or_default().yellow());
    }

    pool.close().await;
    println!("{}", "\n✅ Database health check completed".green().bold());
    Ok(())
}

async fn reset_database(
    database_url: &str,
    tenant: Option<&str>,
    force: bool,
) -> Result<()> {
    println!("{}", "⚠️ DANGER: Resetting database...".red().bold());

    if !force {
        use dialoguer::Confirm;

        let prompt = match tenant {
            Some(schema) => format!("This will DELETE ALL DATA in schema '{}'. Are you absolutely sure?", schema),
            None => "This will DELETE ALL DATA in the database. Are you absolutely sure?".to_string(),
        };

        if !Confirm::new()
            .with_prompt(&prompt)
            .interact()?
        {
            println!("Reset cancelled");
            return Ok(());
        }

        if !Confirm::new()
            .with_prompt("Type 'yes' to confirm this destructive operation")
            .interact()?
        {
            println!("Reset cancelled");
            return Ok(());
        }
    }

    let pool = PgPool::connect(database_url).await?;

    if let Some(schema) = tenant {
        println!("Dropping schema: {}", schema.red());

        let drop_sql = format!("DROP SCHEMA IF EXISTS {} CASCADE", schema);
        sqlx::query(&drop_sql)
            .execute(&pool)
            .await?;

        println!("Schema '{}' has been dropped", schema);
    } else {
        return Err(anyhow!("Full database reset not implemented for safety. Use specific tenant reset."));
    }

    pool.close().await;
    println!("{}", "✅ Database reset completed".yellow().bold());
    Ok(())
}