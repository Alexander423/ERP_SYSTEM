//! Log management and viewing command implementations

use anyhow::{anyhow, Result};
use colored::*;
use std::path::Path;
use tokio::process::Command;

pub async fn execute(
    component: Option<&str>,
    follow: bool,
    lines: usize,
    since: Option<&str>,
) -> Result<()> {
    println!("{}", "📋 Viewing system logs...".blue().bold());

    match component {
        Some("docker") | Some("containers") => show_docker_logs(follow, lines).await,
        Some("erp-api") | Some("api") => show_api_logs(follow, lines, since).await,
        Some("postgres") | Some("database") => show_postgres_logs(follow, lines).await,
        Some("redis") => show_redis_logs(follow, lines).await,
        Some("nginx") => show_nginx_logs(follow, lines).await,
        Some("system") => show_system_logs(follow, lines, since).await,
        None => show_all_logs(follow, lines).await,
        Some(comp) => {
            return Err(anyhow!("Unknown component: {}. Available: docker, erp-api, postgres, redis, nginx, system", comp));
        }
    }
}

async fn show_docker_logs(follow: bool, lines: usize) -> Result<()> {
    println!("🐳 Docker container logs:");

    // Get list of running containers
    let output = Command::new("docker-compose")
        .arg("ps")
        .arg("--services")
        .output()
        .await;

    let services = match output {
        Ok(result) if result.status.success() => {
            String::from_utf8_lossy(&result.stdout)
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        }
        _ => {
            println!("{}", "Docker Compose not available or no services running".yellow());
            return Ok(());
        }
    };

    if services.is_empty() {
        println!("No Docker services found");
        return Ok(());
    }

    for service in &services {
        println!("\n{}", format!("=== {} ===", service).cyan().bold());

        let mut cmd = Command::new("docker-compose");
        cmd.arg("logs")
           .arg("--tail")
           .arg(lines.to_string())
           .arg(service);

        if follow && services.len() == 1 {
            cmd.arg("--follow");
        }

        let output = cmd.output().await?;

        if output.status.success() {
            let logs = String::from_utf8_lossy(&output.stdout);
            print!("{}", logs);
        } else {
            println!("{}", format!("Failed to get logs for {}", service).red());
        }
    }

    if follow && services.len() > 1 {
        println!("\n{}", "Note: Follow mode only works with a single service".yellow());
    }

    Ok(())
}

async fn show_api_logs(follow: bool, lines: usize, since: Option<&str>) -> Result<()> {
    println!("🚀 ERP API logs:");

    // Try different log locations
    let log_paths = vec![
        "/var/log/erp-system/app.log",
        "/opt/erp-system/logs/app.log",
        "./logs/app.log",
    ];

    let mut found_logs = false;

    for log_path in &log_paths {
        if Path::new(log_path).exists() {
            found_logs = true;
            show_file_logs(log_path, follow, lines, since).await?;
            break;
        }
    }

    if !found_logs {
        // Try Docker logs
        println!("Log files not found, trying Docker logs...");
        let mut cmd = Command::new("docker-compose");
        cmd.arg("logs")
           .arg("--tail")
           .arg(lines.to_string());

        if follow {
            cmd.arg("--follow");
        }

        cmd.arg("erp-server");

        let output = cmd.output().await;
        match output {
            Ok(result) if result.status.success() => {
                let logs = String::from_utf8_lossy(&result.stdout);
                print!("{}", logs);
            }
            _ => {
                println!("{}", "No ERP API logs found".yellow());
            }
        }
    }

    Ok(())
}

async fn show_postgres_logs(follow: bool, lines: usize) -> Result<()> {
    println!("🗄️ PostgreSQL logs:");

    // Try Docker logs first
    let output = Command::new("docker-compose")
        .arg("logs")
        .arg("--tail")
        .arg(lines.to_string())
        .arg("postgres")
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => {
            let logs = String::from_utf8_lossy(&result.stdout);
            print!("{}", logs);
            return Ok(());
        }
        _ => {}
    }

    // Try system PostgreSQL logs
    let pg_log_paths = vec![
        "/var/log/postgresql/postgresql-*.log",
        "/usr/local/var/log/postgresql*.log",
        "/var/lib/pgsql/data/log/postgresql-*.log",
    ];

    for log_pattern in &pg_log_paths {
        let mut cmd = Command::new("tail");
        cmd.arg("-n").arg(lines.to_string());

        if follow {
            cmd.arg("-f");
        }

        cmd.arg(log_pattern);

        let output = cmd.output().await;
        if let Ok(result) = output {
            if result.status.success() {
                let logs = String::from_utf8_lossy(&result.stdout);
                if !logs.trim().is_empty() {
                    print!("{}", logs);
                    return Ok(());
                }
            }
        }
    }

    println!("{}", "No PostgreSQL logs found".yellow());
    Ok(())
}

async fn show_redis_logs(follow: bool, lines: usize) -> Result<()> {
    println!("🔴 Redis logs:");

    // Try Docker logs first
    let output = Command::new("docker-compose")
        .arg("logs")
        .arg("--tail")
        .arg(lines.to_string())
        .arg("redis")
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => {
            let logs = String::from_utf8_lossy(&result.stdout);
            print!("{}", logs);
            return Ok(());
        }
        _ => {}
    }

    // Try system Redis logs
    let redis_log_paths = vec![
        "/var/log/redis/redis-server.log",
        "/usr/local/var/log/redis.log",
        "/var/log/redis.log",
    ];

    for log_path in &redis_log_paths {
        if Path::new(log_path).exists() {
            show_file_logs(log_path, follow, lines, None).await?;
            return Ok(());
        }
    }

    println!("{}", "No Redis logs found".yellow());
    Ok(())
}

async fn show_nginx_logs(follow: bool, lines: usize) -> Result<()> {
    println!("🌐 Nginx logs:");

    let nginx_log_paths = vec![
        "/var/log/nginx/access.log",
        "/var/log/nginx/error.log",
        "/usr/local/var/log/nginx/access.log",
        "/usr/local/var/log/nginx/error.log",
    ];

    let mut found_logs = false;

    for log_path in &nginx_log_paths {
        if Path::new(log_path).exists() {
            found_logs = true;
            println!("\n{}", format!("=== {} ===", log_path).cyan());
            show_file_logs(log_path, false, lines / 2, None).await?;
        }
    }

    if !found_logs {
        println!("{}", "No Nginx logs found".yellow());
    }

    Ok(())
}

async fn show_system_logs(follow: bool, lines: usize, since: Option<&str>) -> Result<()> {
    println!("🖥️ System logs:");

    // Try journalctl first (systemd systems)
    let mut cmd = Command::new("journalctl");
    cmd.arg("-u").arg("erp-system")
       .arg("-n").arg(lines.to_string())
       .arg("--no-pager");

    if follow {
        cmd.arg("-f");
    }

    if let Some(since_time) = since {
        cmd.arg("--since").arg(since_time);
    }

    let output = cmd.output().await;
    match output {
        Ok(result) if result.status.success() => {
            let logs = String::from_utf8_lossy(&result.stdout);
            if !logs.trim().is_empty() {
                print!("{}", logs);
                return Ok(());
            }
        }
        _ => {}
    }

    // Fallback to syslog
    let syslog_paths = vec![
        "/var/log/syslog",
        "/var/log/messages",
        "/var/log/system.log",
    ];

    for log_path in &syslog_paths {
        if Path::new(log_path).exists() {
            println!("Reading from: {}", log_path.cyan());

            let mut cmd = Command::new("tail");
            cmd.arg("-n").arg(lines.to_string());

            if follow {
                cmd.arg("-f");
            }

            cmd.arg(log_path);

            let output = cmd.output().await?;
            if output.status.success() {
                let logs = String::from_utf8_lossy(&output.stdout);

                // Filter for ERP-related entries
                for line in logs.lines() {
                    if line.to_lowercase().contains("erp") {
                        println!("{}", line);
                    }
                }
            }
            return Ok(());
        }
    }

    println!("{}", "No system logs found".yellow());
    Ok(())
}

async fn show_all_logs(follow: bool, lines: usize) -> Result<()> {
    println!("📋 All system logs (summary):");

    let per_component = lines / 4;

    // Show logs from each component
    println!("\n{}", "=== ERP API ===".green().bold());
    show_api_logs(false, per_component, None).await?;

    println!("\n{}", "=== Database ===".blue().bold());
    show_postgres_logs(false, per_component).await?;

    println!("\n{}", "=== Redis ===".red().bold());
    show_redis_logs(false, per_component).await?;

    println!("\n{}", "=== System ===".yellow().bold());
    show_system_logs(false, per_component, None).await?;

    if follow {
        println!("\n{}", "Note: Use --component to follow logs for a specific service".cyan());
    }

    Ok(())
}

async fn show_file_logs(
    file_path: &str,
    follow: bool,
    lines: usize,
    since: Option<&str>,
) -> Result<()> {
    let mut cmd = Command::new("tail");
    cmd.arg("-n").arg(lines.to_string());

    if follow {
        cmd.arg("-f");
    }

    cmd.arg(file_path);

    if follow {
        // For follow mode, spawn and wait
        let mut child = cmd.spawn()?;
        child.wait().await?;
    } else {
        let output = cmd.output().await?;
        if output.status.success() {
            let logs = String::from_utf8_lossy(&output.stdout);

            // Apply since filter if specified
            if let Some(since_time) = since {
                // Simple time filtering (could be enhanced)
                for line in logs.lines() {
                    // This is a basic implementation - could be improved
                    // to parse actual timestamps and compare with since_time
                    if line.contains(since_time) {
                        println!("{}", line);
                    }
                }
            } else {
                print!("{}", logs);
            }
        } else {
            return Err(anyhow!("Failed to read log file: {}", file_path));
        }
    }

    Ok(())
}