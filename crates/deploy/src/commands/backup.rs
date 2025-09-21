//! Backup and restore command implementations

use anyhow::{anyhow, Result};
use colored::*;
use serde_json::json;
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use walkdir::WalkDir;

use crate::{BackupCommands, config::Config};

pub async fn execute_backup_command(
    cmd: BackupCommands,
    _config: &Config,
) -> Result<()> {
    match cmd {
        BackupCommands::Create { name, output, include, exclude, compression } => {
            create_backup(&name, output.as_deref(), include, exclude, &compression).await
        }
        BackupCommands::List { directory, format } => {
            list_backups(directory.as_deref(), &format).await
        }
        BackupCommands::Restore { name: _name, backup, force, components } => {
            restore_backup(&backup, force, components).await
        }
        BackupCommands::Verify { name, detailed } => {
            verify_backup(&name, detailed).await
        }
        BackupCommands::Cleanup { keep, dry_run } => {
            cleanup_backups("./backups", keep as u32, dry_run).await
        }
    }
}

async fn create_backup(
    name: &str,
    output_dir: Option<&str>,
    include: Vec<String>,
    exclude: Vec<String>,
    compression: &str,
) -> Result<()> {
    println!("{}", "💾 Creating system backup...".blue().bold());

    let default_output = "./backups".to_string();
    let output_dir = output_dir.unwrap_or(&default_output);
    let output_path = Path::new(output_dir);
    if !output_path.exists() {
        fs::create_dir_all(output_path)?;
        println!("Created backup directory: {}", output_dir.yellow());
    }

    println!("Creating backup: {}", name.yellow());

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("erp_backup_{}", timestamp);
    let backup_path = output_path.join(&backup_name);

    fs::create_dir_all(&backup_path)?;

    // Default components to backup
    let components = if include.is_empty() {
        vec!["database".to_string(), "config".to_string(), "logs".to_string()]
    } else {
        include
    };

    println!("Backup components: {}", components.join(", ").yellow());
    if !exclude.is_empty() {
        println!("Excluding: {}", exclude.join(", ").red());
    }

    // Create backup manifest
    let manifest = json!({
        "created_at": Utc::now().to_rfc3339(),
        "version": "1.0.0",
        "components": components,
        "excluded": exclude,
        "compression": compression,
        "backup_type": "full"
    });

    fs::write(
        backup_path.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?
    )?;

    for component in &components {
        if exclude.contains(component) {
            continue;
        }

        match component.as_str() {
            "database" => backup_database(&backup_path).await?,
            "config" => backup_config(&backup_path).await?,
            "logs" => backup_logs(&backup_path).await?,
            "containers" => backup_containers(&backup_path).await?,
            _ => {
                println!("{}", format!("Unknown component: {}", component).yellow());
            }
        }
    }

    // Compress if requested
    if compression != "none" {
        println!("Compressing backup with {} compression...", compression);
        let compression_level = match compression {
            "fast" => 1,
            "medium" => 5,
            "max" => 9,
            _ => 5, // default
        };
        compress_backup(&backup_path, compression_level).await?;
    }

    println!("{}", format!("✅ Backup created: {}", backup_path.display()).green().bold());
    Ok(())
}

async fn backup_database(backup_path: &Path) -> Result<()> {
    println!("📊 Backing up database...");

    let db_backup_path = backup_path.join("database");
    fs::create_dir_all(&db_backup_path)?;

    // Use pg_dump to backup database
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let dump_file = db_backup_path.join(format!("database_dump_{}.sql", timestamp));

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main".to_string());

    // Parse URL for pg_dump parameters
    let url = url::Url::parse(&database_url)?;
    let host = url.host_str().unwrap_or("localhost");
    let port = url.port().unwrap_or(5432);
    let username = url.username();
    let password = url.password().unwrap_or("");
    let database = url.path().trim_start_matches('/');

    let output = tokio::process::Command::new("pg_dump")
        .arg("--host").arg(host)
        .arg("--port").arg(port.to_string())
        .arg("--username").arg(username)
        .arg("--no-password")
        .arg("--format").arg("custom")
        .arg("--file").arg(&dump_file)
        .arg(database)
        .env("PGPASSWORD", password)
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => {
            println!("✅ Database backup completed");
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(anyhow!("Database backup failed: {}", stderr));
        }
        Err(_) => {
            println!("{}", "⚠️ pg_dump not found, skipping database backup".yellow());
        }
    }

    Ok(())
}

async fn backup_config(backup_path: &Path) -> Result<()> {
    println!("⚙️ Backing up configuration...");

    let config_backup_path = backup_path.join("config");
    fs::create_dir_all(&config_backup_path)?;

    // Configuration paths to backup
    let config_paths = vec![
        "/etc/erp-system",
        "/opt/erp-system/config",
        "./config",
        "docker-compose.yml",
        "Cargo.toml",
    ];

    for config_path in config_paths {
        let source_path = Path::new(config_path);
        if source_path.exists() {
            let dest_name = source_path.file_name().unwrap_or_default();
            let dest_path = config_backup_path.join(dest_name);

            if source_path.is_dir() {
                copy_dir_recursive(source_path, &dest_path)?;
            } else if source_path.is_file() {
                fs::copy(source_path, &dest_path)?;
            }

            println!("📋 Copied: {}", config_path);
        }
    }

    println!("✅ Configuration backup completed");
    Ok(())
}

async fn backup_logs(backup_path: &Path) -> Result<()> {
    println!("📋 Backing up logs...");

    let logs_backup_path = backup_path.join("logs");
    fs::create_dir_all(&logs_backup_path)?;

    // Log paths to backup
    let log_paths = vec![
        "/var/log/erp-system",
        "/opt/erp-system/logs",
        "./logs",
    ];

    for log_path in log_paths {
        let source_path = Path::new(log_path);
        if source_path.exists() {
            let dest_name = source_path.file_name().unwrap_or_default();
            let dest_path = logs_backup_path.join(dest_name);

            copy_dir_recursive(source_path, &dest_path)?;
            println!("📋 Copied logs: {}", log_path);
        }
    }

    println!("✅ Logs backup completed");
    Ok(())
}

async fn backup_containers(backup_path: &Path) -> Result<()> {
    println!("🐳 Backing up container information...");

    let containers_backup_path = backup_path.join("containers");
    fs::create_dir_all(&containers_backup_path)?;

    // Export docker-compose state
    let output = tokio::process::Command::new("docker-compose")
        .arg("config")
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => {
            let compose_config = String::from_utf8_lossy(&result.stdout);
            fs::write(
                containers_backup_path.join("docker-compose.yml"),
                compose_config.as_bytes()
            )?;
        }
        _ => {
            println!("{}", "⚠️ Docker not available, skipping container backup".yellow());
        }
    }

    println!("✅ Container backup completed");
    Ok(())
}

async fn compress_backup(backup_path: &Path, compression: u8) -> Result<()> {
    let compressed_file = format!("{}.tar.gz", backup_path.display());

    // Select compression method based on level
    let compression_arg = match compression {
        0..=3 => "czf",  // gzip fast
        4..=6 => "cJf",  // xz medium
        7..=9 => "cjf",  // bzip2 max
        _ => "czf",      // default to gzip
    };

    let output = tokio::process::Command::new("tar")
        .arg(compression_arg)
        .arg(&compressed_file)
        .arg("-C")
        .arg(backup_path.parent().unwrap())
        .arg(backup_path.file_name().unwrap())
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => {
            // Remove uncompressed directory
            fs::remove_dir_all(backup_path)?;
            println!("✅ Backup compressed: {}", compressed_file);
        }
        _ => {
            println!("{}", "⚠️ Compression failed, keeping uncompressed backup".yellow());
        }
    }

    Ok(())
}

async fn list_backups(directory: Option<&str>, format: &str) -> Result<()> {
    let directory = directory.unwrap_or("./backups");
    println!("{}", "📋 Listing available backups...".blue().bold());

    let backup_dir = Path::new(directory);
    if !backup_dir.exists() {
        return Err(anyhow!("Backup directory not found: {}", directory));
    }

    let mut backups = Vec::new();

    for entry in fs::read_dir(backup_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() || path.extension().map_or(false, |ext| ext == "tar" || ext == "gz") {
            let metadata = entry.metadata()?;
            let created = metadata.created()
                .map(|time| DateTime::<Utc>::from(time))
                .unwrap_or_else(|_| Utc::now());

            let size = if path.is_dir() {
                calculate_dir_size(&path)?
            } else {
                metadata.len()
            };

            backups.push(json!({
                "name": path.file_name().unwrap().to_string_lossy(),
                "path": path.display().to_string(),
                "created": created.to_rfc3339(),
                "size": size,
                "size_human": format_bytes(size),
                "type": if path.is_dir() { "directory" } else { "archive" }
            }));
        }
    }

    // Sort by creation time (newest first)
    backups.sort_by(|a, b| {
        b["created"].as_str().unwrap_or_default()
            .cmp(a["created"].as_str().unwrap_or_default())
    });

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&backups)?);
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(&backups)?);
        }
        _ => {
            // Table format
            if backups.is_empty() {
                println!("No backups found in {}", directory);
                return Ok(());
            }

            println!("{:<30} {:<20} {:<10} {:<15}",
                "Name", "Created", "Type", "Size");
            println!("{}", "-".repeat(75));

            for backup in backups {
                let name = backup["name"].as_str().unwrap_or("N/A");
                let created = backup["created"].as_str().unwrap_or("N/A");
                let backup_type = backup["type"].as_str().unwrap_or("N/A");
                let size_human = backup["size_human"].as_str().unwrap_or("N/A");

                // Parse and format date
                let formatted_date = if let Ok(dt) = DateTime::parse_from_rfc3339(created) {
                    dt.format("%Y-%m-%d %H:%M").to_string()
                } else {
                    created.to_string()
                };

                let type_colored = match backup_type {
                    "directory" => backup_type.blue(),
                    "archive" => backup_type.green(),
                    _ => backup_type.normal(),
                };

                println!("{:<30} {:<20} {:<10} {:<15}",
                    name.cyan(),
                    formatted_date.bright_black(),
                    type_colored,
                    size_human.yellow()
                );
            }
        }
    }

    Ok(())
}

async fn restore_backup(backup: &str, force: bool, components: Vec<String>) -> Result<()> {
    println!("{}", "🔄 Restoring from backup...".blue().bold());

    let backup_path = Path::new(backup);
    if !backup_path.exists() {
        return Err(anyhow!("Backup not found: {}", backup));
    }

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

    // Filter components if specified
    if !components.is_empty() {
        println!("Restoring specific components: {:?}", components);
        for component in &components {
            println!("  - Restoring component: {}", component);
        }
    } else {
        println!("Restoring all components from backup");
    }

    // Check if backup is compressed
    let working_backup_path = if backup_path.extension().map_or(false, |ext| ext == "gz" || ext == "tar") {
        println!("📂 Extracting compressed backup...");
        extract_backup(backup_path).await?
    } else {
        backup_path.to_path_buf()
    };

    // Read manifest
    let manifest_path = working_backup_path.join("manifest.json");
    if !manifest_path.exists() {
        return Err(anyhow!("Invalid backup: manifest.json not found"));
    }

    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;

    let backup_components = manifest["components"].as_array()
        .ok_or_else(|| anyhow!("Invalid manifest: components not found"))?;

    // Filter components to restore
    let restore_components: Vec<String> = if components.is_empty() {
        backup_components.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    } else {
        components
    };

    println!("Restoring components: {:?}", restore_components);

    // Restore each component
    for component in &restore_components {
        println!("🔄 Restoring component: {}", component.yellow());

        match component.as_str() {
            "database" => restore_database(&working_backup_path).await?,
            "config" => restore_config(&working_backup_path).await?,
            "logs" => restore_logs(&working_backup_path).await?,
            "containers" => restore_containers(&working_backup_path).await?,
            _ => {
                println!("{}", format!("⚠️ Unknown component: {}", component).yellow());
            }
        }
    }

    // Clean up extracted backup if it was compressed
    if backup_path != working_backup_path {
        fs::remove_dir_all(&working_backup_path)?;
    }

    println!("{}", "✅ Restore completed successfully".green().bold());
    Ok(())
}

async fn verify_backup(backup: &str, detailed: bool) -> Result<()> {
    println!("{}", "🔍 Verifying backup integrity...".blue().bold());

    let backup_path = Path::new(backup);
    if !backup_path.exists() {
        return Err(anyhow!("Backup not found: {}", backup));
    }

    let mut issues_found = 0;

    // Check if backup is compressed and extract if needed
    let working_backup_path = if backup_path.extension().map_or(false, |ext| ext == "gz" || ext == "tar") {
        println!("📂 Extracting backup for verification...");
        extract_backup(backup_path).await?
    } else {
        backup_path.to_path_buf()
    };

    // Verify manifest exists and is valid
    let manifest_path = working_backup_path.join("manifest.json");
    if !manifest_path.exists() {
        println!("{}", "❌ Missing manifest.json".red());
        issues_found += 1;
        return Err(anyhow!("Backup verification failed: missing manifest"));
    }

    let manifest_content = fs::read_to_string(&manifest_path).map_err(|e| {
        println!("{}", "❌ Cannot read manifest.json".red());
        anyhow!("Failed to read manifest: {}", e)
    })?;

    let manifest: serde_json::Value = serde_json::from_str(&manifest_content).map_err(|e| {
        println!("{}", "❌ Invalid manifest.json format".red());
        anyhow!("Invalid manifest JSON: {}", e)
    })?;

    println!("✅ Manifest file is valid");

    // Verify backup components
    if let Some(components) = manifest["components"].as_array() {
        println!("🔍 Verifying backup components...");

        for component in components {
            if let Some(component_name) = component.as_str() {
                let component_path = working_backup_path.join(component_name);

                if component_path.exists() {
                    println!("✅ Component '{}' found", component_name.green());

                    if detailed {
                        match verify_component(&component_path, component_name).await {
                            Ok(()) => println!("  ✅ Component integrity verified"),
                            Err(e) => {
                                println!("  ❌ Component integrity check failed: {}", e.to_string().red());
                                issues_found += 1;
                            }
                        }
                    }
                } else {
                    println!("❌ Component '{}' missing", component_name.red());
                    issues_found += 1;
                }
            }
        }
    }

    // Additional detailed checks
    if detailed {
        println!("🔍 Performing detailed verification...");

        // Check backup creation timestamp
        if let Some(created_at) = manifest["created_at"].as_str() {
            match DateTime::parse_from_rfc3339(created_at) {
                Ok(timestamp) => {
                    let age = Utc::now().signed_duration_since(timestamp.with_timezone(&Utc));
                    println!("✅ Backup age: {} days", age.num_days());

                    if age.num_days() > 30 {
                        println!("⚠️ Backup is older than 30 days");
                    }
                }
                Err(_) => {
                    println!("❌ Invalid creation timestamp in manifest");
                    issues_found += 1;
                }
            }
        }

        // Verify backup size
        let backup_size = calculate_dir_size(&working_backup_path)?;
        println!("✅ Total backup size: {}", format_bytes(backup_size));

        if backup_size < 1024 {
            println!("⚠️ Backup seems unusually small (< 1KB)");
        }
    }

    // Clean up extracted backup if it was compressed
    if backup_path != working_backup_path {
        fs::remove_dir_all(&working_backup_path)?;
    }

    // Final result
    if issues_found == 0 {
        println!("{}", "✅ Backup verification completed successfully".green().bold());
    } else {
        println!("{}", format!("❌ Backup verification completed with {} issues", issues_found).red().bold());
        return Err(anyhow!("Backup verification failed with {} issues", issues_found));
    }

    Ok(())
}

async fn cleanup_backups(directory: &str, keep_days: u32, dry_run: bool) -> Result<()> {
    let action = if dry_run { "Would delete" } else { "Deleting" };
    println!("{}", format!("🧹 {} old backups (older than {} days)...", action, keep_days).blue().bold());

    let backup_dir = Path::new(directory);
    if !backup_dir.exists() {
        return Err(anyhow!("Backup directory not found: {}", directory));
    }

    let cutoff_time = Utc::now() - chrono::Duration::days(keep_days as i64);
    let mut deleted_count = 0;
    let mut freed_space = 0u64;

    for entry in fs::read_dir(backup_dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        if let Ok(created) = metadata.created() {
            let created_time = DateTime::<Utc>::from(created);

            if created_time < cutoff_time {
                let size = if path.is_dir() {
                    calculate_dir_size(&path)?
                } else {
                    metadata.len()
                };

                println!("{} {} ({})",
                    action.yellow(),
                    path.file_name().unwrap().to_string_lossy().red(),
                    format_bytes(size).cyan()
                );

                if !dry_run {
                    if path.is_dir() {
                        fs::remove_dir_all(&path)?;
                    } else {
                        fs::remove_file(&path)?;
                    }
                }

                deleted_count += 1;
                freed_space += size;
            }
        }
    }

    if deleted_count == 0 {
        println!("No old backups found to delete");
    } else {
        println!("{} {} old backup(s), freed {}",
            if dry_run { "Would delete" } else { "Deleted" },
            deleted_count.to_string().green(),
            format_bytes(freed_space).cyan()
        );
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(src)?;
        let dest_path = dst.join(relative);

        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &dest_path)?;
        }
    }

    Ok(())
}

fn calculate_dir_size(path: &Path) -> Result<u64> {
    let mut size = 0u64;

    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.path().is_file() {
            size += entry.metadata()?.len();
        }
    }

    Ok(size)
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

async fn extract_backup(backup_path: &Path) -> Result<std::path::PathBuf> {
    let temp_dir = std::env::temp_dir().join(format!("erp_backup_extract_{}", Utc::now().timestamp()));
    fs::create_dir_all(&temp_dir)?;

    println!("📂 Extracting to: {}", temp_dir.display());

    let output = tokio::process::Command::new("tar")
        .arg("-xf")
        .arg(backup_path)
        .arg("-C")
        .arg(&temp_dir)
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => {
            // Find the extracted directory (should be the only subdirectory)
            for entry in fs::read_dir(&temp_dir)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    return Ok(entry.path());
                }
            }
            Ok(temp_dir)
        }
        _ => Err(anyhow!("Failed to extract backup archive"))
    }
}

async fn restore_database(backup_path: &Path) -> Result<()> {
    println!("📊 Restoring database from backup...");

    let db_backup_path = backup_path.join("database");
    if !db_backup_path.exists() {
        return Err(anyhow!("Database backup not found"));
    }

    // Find the database dump file
    let mut dump_file = None;
    for entry in fs::read_dir(&db_backup_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "sql") {
            dump_file = Some(path);
            break;
        }
    }

    let dump_file = dump_file.ok_or_else(|| anyhow!("Database dump file not found"))?;

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main".to_string());

    // Parse URL for restore parameters
    let url = url::Url::parse(&database_url)?;
    let host = url.host_str().unwrap_or("localhost");
    let port = url.port().unwrap_or(5432);
    let username = url.username();
    let password = url.password().unwrap_or("");
    let database = url.path().trim_start_matches('/');

    let output = tokio::process::Command::new("pg_restore")
        .arg("--host").arg(host)
        .arg("--port").arg(port.to_string())
        .arg("--username").arg(username)
        .arg("--no-password")
        .arg("--dbname").arg(database)
        .arg("--clean")
        .arg("--if-exists")
        .arg(&dump_file)
        .env("PGPASSWORD", password)
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => {
            println!("✅ Database restoration completed");
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(anyhow!("Database restoration failed: {}", stderr));
        }
        Err(_) => {
            return Err(anyhow!("pg_restore not found"));
        }
    }

    Ok(())
}

async fn restore_config(backup_path: &Path) -> Result<()> {
    println!("⚙️ Restoring configuration from backup...");

    let config_backup_path = backup_path.join("config");
    if !config_backup_path.exists() {
        return Err(anyhow!("Configuration backup not found"));
    }

    // Restore configuration files
    for entry in fs::read_dir(&config_backup_path)? {
        let entry = entry?;
        let src_path = entry.path();
        let filename = src_path.file_name().unwrap();

        // Determine destination based on filename
        let dest_path = match filename.to_str() {
            Some("docker-compose.yml") => Path::new("./docker-compose.yml").to_path_buf(),
            Some("Cargo.toml") => Path::new("./Cargo.toml").to_path_buf(),
            _ => Path::new("./config").join(filename),
        };

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }

        println!("📋 Restored: {}", dest_path.display());
    }

    println!("✅ Configuration restoration completed");
    Ok(())
}

async fn restore_logs(backup_path: &Path) -> Result<()> {
    println!("📋 Restoring logs from backup...");

    let logs_backup_path = backup_path.join("logs");
    if !logs_backup_path.exists() {
        return Err(anyhow!("Logs backup not found"));
    }

    // Create logs directory if it doesn't exist
    let logs_dir = Path::new("./logs");
    fs::create_dir_all(logs_dir)?;

    // Copy log files
    copy_dir_recursive(&logs_backup_path, logs_dir)?;

    println!("✅ Logs restoration completed");
    Ok(())
}

async fn restore_containers(backup_path: &Path) -> Result<()> {
    println!("🐳 Restoring container configuration from backup...");

    let containers_backup_path = backup_path.join("containers");
    if !containers_backup_path.exists() {
        return Err(anyhow!("Container backup not found"));
    }

    let compose_file = containers_backup_path.join("docker-compose.yml");
    if compose_file.exists() {
        fs::copy(&compose_file, "./docker-compose.yml")?;
        println!("📋 Restored: docker-compose.yml");
    }

    println!("✅ Container restoration completed");
    Ok(())
}

async fn verify_component(component_path: &Path, component_name: &str) -> Result<()> {
    match component_name {
        "database" => {
            // Verify database dump exists and is readable
            let mut found_dump = false;
            for entry in fs::read_dir(component_path)? {
                let entry = entry?;
                if entry.path().extension().map_or(false, |ext| ext == "sql") {
                    found_dump = true;
                    break;
                }
            }
            if !found_dump {
                return Err(anyhow!("No database dump file found"));
            }
        }
        "config" => {
            // Verify essential config files exist
            let essential_files = ["docker-compose.yml", "Cargo.toml"];
            for file in &essential_files {
                let file_path = component_path.join(file);
                if file_path.exists() {
                    return Ok(()); // At least one essential file found
                }
            }
            return Err(anyhow!("No essential configuration files found"));
        }
        "logs" => {
            // Verify logs directory is not empty and check file types
            let mut has_files = false;
            let mut file_count = 0;
            let mut total_size = 0u64;

            for entry in fs::read_dir(component_path)? {
                let entry = entry?;
                if entry.path().is_file() {
                    file_count += 1;
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                }
                has_files = true;
            }

            if !has_files {
                return Err(anyhow!("Logs directory is empty"));
            }

            // Log verification details
            tracing::debug!(
                "Verified logs component: {} files, {} bytes total",
                file_count,
                total_size
            );
        }
        "containers" => {
            // Verify container configuration exists
            let compose_file = component_path.join("docker-compose.yml");
            if !compose_file.exists() {
                return Err(anyhow!("docker-compose.yml not found"));
            }
        }
        _ => {
            // Generic verification: just check if directory exists and is readable
            if !component_path.exists() {
                return Err(anyhow!("Component directory does not exist"));
            }
        }
    }
    Ok(())
}