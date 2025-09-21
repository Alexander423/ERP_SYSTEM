//! Tenant management command implementations

use anyhow::{anyhow, Result};
use colored::*;
use dialoguer::{Input, Password, Confirm};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{TenantCommands, config::Config};

pub async fn execute_tenant_command(
    cmd: TenantCommands,
    config: &Config,
    database_url: Option<&str>,
) -> Result<()> {
    let db_url = database_url
        .or(config.database_url.as_deref())
        .ok_or_else(|| anyhow!("Database URL not provided"))?;

    let pool = PgPool::connect(db_url).await?;

    match cmd {
        TenantCommands::Create { name, email, password, domain, schema } => {
            create_tenant(&pool, name, email, password, domain, schema).await
        }
        TenantCommands::List { format, include_inactive } => {
            list_tenants(&pool, &format, include_inactive).await
        }
        TenantCommands::Show { tenant, format } => {
            show_tenant(&pool, &tenant, &format).await
        }
        TenantCommands::Update { tenant, name, status } => {
            update_tenant(&pool, &tenant, name, status).await
        }
        TenantCommands::Delete { tenant, force, keep_schema } => {
            delete_tenant(&pool, &tenant, force, keep_schema).await
        }
    }
}

async fn create_tenant(
    pool: &PgPool,
    name: String,
    email: String,
    password: Option<String>,
    domain: Option<String>,
    schema: Option<String>,
) -> Result<()> {
    println!("{}", "🏢 Creating new tenant...".blue().bold());

    // Validate inputs
    if name.trim().is_empty() {
        return Err(anyhow!("Tenant name cannot be empty"));
    }

    if !is_valid_email(&email) {
        return Err(anyhow!("Invalid email format"));
    }

    // Check if tenant already exists
    let existing = sqlx::query!(
        "SELECT COUNT(*) as count FROM public.tenants WHERE name = $1",
        name
    )
    .fetch_one(pool)
    .await?;

    if existing.count.unwrap_or(0) > 0 {
        return Err(anyhow!("Tenant '{}' already exists", name));
    }

    // Generate or validate schema name
    let schema_name = schema.unwrap_or_else(|| generate_schema_name(&name));

    // Check if schema already exists
    let schema_exists = sqlx::query!(
        "SELECT COUNT(*) as count FROM information_schema.schemata WHERE schema_name = $1",
        schema_name
    )
    .fetch_one(pool)
    .await?;

    if schema_exists.count.unwrap_or(0) > 0 {
        return Err(anyhow!("Schema '{}' already exists", schema_name));
    }

    // Get or prompt for password
    let admin_password = match password {
        Some(pwd) => pwd,
        None => {
            Password::new()
                .with_prompt("Admin password")
                .with_confirmation("Confirm password", "Passwords don't match")
                .interact()?
        }
    };

    // Generate IDs
    let tenant_id = Uuid::new_v4();
    let admin_user_id = Uuid::new_v4();

    // Hash password
    let password_hash = bcrypt::hash(&admin_password, 12)?;

    println!("Tenant ID: {}", tenant_id.to_string().yellow());
    println!("Schema: {}", schema_name.yellow());
    println!("Admin Email: {}", email.yellow());

    if !Confirm::new()
        .with_prompt("Create tenant with these settings?")
        .interact()?
    {
        println!("Tenant creation cancelled");
        return Ok(());
    }

    // Start transaction
    let mut tx = pool.begin().await?;

    // Create tenant record
    sqlx::query!(
        "INSERT INTO public.tenants (id, name, schema_name, status) VALUES ($1, $2, $3, 'active')",
        tenant_id,
        name,
        schema_name
    )
    .execute(&mut *tx)
    .await?;

    // Create schema
    // Use relative paths from project root
    let schema_sql = include_str!("../../../migrations/002_tenant_schema_template.sql");
    let processed_sql = schema_sql.replace("{TENANT_SCHEMA}", &schema_name);

    sqlx::raw_sql(&processed_sql)
        .execute(&mut *tx)
        .await?;

    // Seed default roles
    let roles_sql = include_str!("../../../migrations/seeds/001_default_roles.sql");
    let processed_roles = roles_sql.replace("{TENANT_SCHEMA}", &schema_name);

    sqlx::raw_sql(&processed_roles)
        .execute(&mut *tx)
        .await?;

    // Seed reference data
    let ref_data_sql = include_str!("../../../migrations/seeds/002_reference_data.sql");
    let processed_ref_data = ref_data_sql
        .replace("{TENANT_SCHEMA}", &schema_name)
        .replace("{TENANT_NAME}", &name)
        .replace("{TENANT_DOMAIN}", &domain.unwrap_or_else(|| format!("{}.erp-system.com", schema_name)));

    sqlx::raw_sql(&processed_ref_data)
        .execute(&mut *tx)
        .await?;

    // Create admin user
    let admin_sql = include_str!("../../../migrations/seeds/003_admin_user.sql");
    let processed_admin = admin_sql
        .replace("{TENANT_SCHEMA}", &schema_name)
        .replace("{TENANT_ID}", &tenant_id.to_string())
        .replace("{ADMIN_EMAIL}", &email)
        .replace("{ADMIN_PASSWORD_HASH}", &password_hash)
        .replace("{ADMIN_USER_ID}", &admin_user_id.to_string());

    sqlx::raw_sql(&processed_admin)
        .execute(&mut *tx)
        .await?;

    // Commit transaction
    tx.commit().await?;

    println!("{}", "✅ Tenant created successfully!".green().bold());

    // Display summary
    println!("\n{}", "📊 Tenant Summary:".blue().bold());
    println!("  Tenant ID: {}", tenant_id);
    println!("  Name: {}", name);
    println!("  Schema: {}", schema_name);
    println!("  Admin Email: {}", email);
    println!("  Admin Password: {}", "*".repeat(admin_password.len()));
    println!("  Status: Active");

    Ok(())
}

async fn list_tenants(pool: &PgPool, format: &str, include_inactive: bool) -> Result<()> {
    let status_filter = if include_inactive {
        ""
    } else {
        "WHERE status = 'active'"
    };

    let query = format!(
        "SELECT id, name, schema_name, status, created_at FROM public.tenants {} ORDER BY created_at DESC",
        status_filter
    );

    let tenants = sqlx::query(&query)
        .fetch_all(pool)
        .await?;

    match format {
        "json" => {
            let tenant_list: Vec<serde_json::Value> = tenants
                .iter()
                .map(|row| {
                    json!({
                        "id": row.get::<Uuid, _>("id"),
                        "name": row.get::<String, _>("name"),
                        "schema_name": row.get::<String, _>("schema_name"),
                        "status": row.get::<String, _>("status"),
                        "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                    })
                })
                .collect();

            println!("{}", serde_json::to_string_pretty(&tenant_list)?);
        }
        "yaml" => {
            let tenant_list: Vec<serde_json::Value> = tenants
                .iter()
                .map(|row| {
                    json!({
                        "id": row.get::<Uuid, _>("id"),
                        "name": row.get::<String, _>("name"),
                        "schema_name": row.get::<String, _>("schema_name"),
                        "status": row.get::<String, _>("status"),
                        "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                    })
                })
                .collect();

            println!("{}", serde_yaml::to_string(&tenant_list)?);
        }
        _ => {
            // Table format
            println!("{}", "📋 Tenants:".blue().bold());
            println!("{:<36} {:<30} {:<20} {:<10} {:<20}",
                "ID", "Name", "Schema", "Status", "Created");
            println!("{}", "-".repeat(120));

            for row in tenants {
                let id: Uuid = row.get("id");
                let name: String = row.get("name");
                let schema: String = row.get("schema_name");
                let status: String = row.get("status");
                let created: chrono::DateTime<chrono::Utc> = row.get("created_at");

                let status_colored = match status.as_str() {
                    "active" => status.green(),
                    "suspended" => status.yellow(),
                    "deleted" => status.red(),
                    _ => status.normal(),
                };

                println!("{:<36} {:<30} {:<20} {:<10} {:<20}",
                    id.to_string().bright_black(),
                    name.white().bold(),
                    schema.cyan(),
                    status_colored,
                    created.format("%Y-%m-%d %H:%M:%S").to_string().bright_black()
                );
            }
        }
    }

    Ok(())
}

async fn show_tenant(pool: &PgPool, tenant: &str, format: &str) -> Result<()> {
    // Try to find tenant by ID or schema name
    let tenant_data = sqlx::query!(
        "SELECT id, name, schema_name, status, created_at, updated_at
         FROM public.tenants
         WHERE id::text = $1 OR schema_name = $1 OR name = $1",
        tenant
    )
    .fetch_optional(pool)
    .await?;

    let tenant_data = tenant_data.ok_or_else(|| anyhow!("Tenant not found: {}", tenant))?;

    match format {
        "json" => {
            let tenant_info = json!({
                "id": tenant_data.id,
                "name": tenant_data.name,
                "schema_name": tenant_data.schema_name,
                "status": tenant_data.status,
                "created_at": tenant_data.created_at,
                "updated_at": tenant_data.updated_at
            });
            println!("{}", serde_json::to_string_pretty(&tenant_info)?);
        }
        "yaml" => {
            let tenant_info = json!({
                "id": tenant_data.id,
                "name": tenant_data.name,
                "schema_name": tenant_data.schema_name,
                "status": tenant_data.status,
                "created_at": tenant_data.created_at,
                "updated_at": tenant_data.updated_at
            });
            println!("{}", serde_yaml::to_string(&tenant_info)?);
        }
        _ => {
            println!("{}", "📊 Tenant Details:".blue().bold());
            println!("  ID: {}", tenant_data.id.to_string().yellow());
            println!("  Name: {}", tenant_data.name.white().bold());
            println!("  Schema: {}", tenant_data.schema_name.cyan());
            println!("  Status: {}",
                match tenant_data.status.as_str() {
                    "active" => tenant_data.status.green(),
                    "suspended" => tenant_data.status.yellow(),
                    "deleted" => tenant_data.status.red(),
                    _ => tenant_data.status.normal(),
                }
            );
            println!("  Created: {}", tenant_data.created_at.format("%Y-%m-%d %H:%M:%S").to_string().bright_black());
            println!("  Updated: {}", tenant_data.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().bright_black());
        }
    }

    Ok(())
}

async fn update_tenant(
    pool: &PgPool,
    tenant: &str,
    name: Option<String>,
    status: Option<String>,
) -> Result<()> {
    // Validate status if provided
    if let Some(ref status) = status {
        if !["active", "suspended", "inactive", "deleted"].contains(&status.as_str()) {
            return Err(anyhow!("Invalid status. Must be one of: active, suspended, inactive, deleted"));
        }
    }

    // Find tenant
    let tenant_data = sqlx::query!(
        "SELECT id FROM public.tenants WHERE id::text = $1 OR schema_name = $1 OR name = $1",
        tenant
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| anyhow!("Tenant not found: {}", tenant))?;

    // Build update query
    let mut updates = Vec::new();
    let mut params: Vec<&(dyn sqlx::Encode<sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Sync)> = Vec::new();
    let mut param_count = 1;

    if let Some(ref new_name) = name {
        updates.push(format!("name = ${}", param_count));
        params.push(new_name);
        param_count += 1;
    }

    if let Some(ref new_status) = status {
        updates.push(format!("status = ${}", param_count));
        params.push(new_status);
        param_count += 1;
    }

    if updates.is_empty() {
        return Err(anyhow!("No updates specified"));
    }

    updates.push("updated_at = NOW()".to_string());

    let query = format!(
        "UPDATE public.tenants SET {} WHERE id = ${}",
        updates.join(", "),
        param_count
    );

    params.push(&tenant_data.id);

    // Execute update
    let mut query_builder = sqlx::query(&query);
    for param in params {
        query_builder = query_builder.bind(param);
    }

    let result = query_builder.execute(pool).await?;

    if result.rows_affected() == 0 {
        return Err(anyhow!("Failed to update tenant"));
    }

    println!("{}", "✅ Tenant updated successfully!".green().bold());
    Ok(())
}

async fn delete_tenant(
    pool: &PgPool,
    tenant: &str,
    force: bool,
    keep_schema: bool,
) -> Result<()> {
    // Find tenant
    let tenant_data = sqlx::query!(
        "SELECT id, name, schema_name FROM public.tenants WHERE id::text = $1 OR schema_name = $1 OR name = $1",
        tenant
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| anyhow!("Tenant not found: {}", tenant))?;

    println!("{}", "⚠️ WARNING: This will delete the tenant and all associated data!".red().bold());
    println!("Tenant: {} ({})", tenant_data.name.yellow(), tenant_data.schema_name.cyan());

    if !force {
        if !Confirm::new()
            .with_prompt("Are you sure you want to delete this tenant?")
            .interact()?
        {
            println!("Tenant deletion cancelled");
            return Ok(());
        }

        if !keep_schema {
            if !Confirm::new()
                .with_prompt("This will also delete the database schema. Continue?")
                .interact()?
            {
                println!("Tenant deletion cancelled");
                return Ok(());
            }
        }
    }

    let mut tx = pool.begin().await?;

    if !keep_schema {
        // Drop schema and all its contents
        let drop_schema_sql = format!("DROP SCHEMA IF EXISTS {} CASCADE", tenant_data.schema_name);
        sqlx::query(&drop_schema_sql)
            .execute(&mut *tx)
            .await?;

        println!("✅ Database schema dropped");
    } else {
        // Just mark tenant as deleted
        sqlx::query!(
            "UPDATE public.tenants SET status = 'deleted', updated_at = NOW() WHERE id = $1",
            tenant_data.id
        )
        .execute(&mut *tx)
        .await?;
    }

    // Remove tenant record
    sqlx::query!(
        "DELETE FROM public.tenants WHERE id = $1",
        tenant_data.id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    println!("{}", "✅ Tenant deleted successfully!".green().bold());
    Ok(())
}

fn generate_schema_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

fn is_valid_email(email: &str) -> bool {
    use regex::Regex;
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}