//! System status monitoring commands

use std::collections::HashMap;
use std::process::Command;
use anyhow::Result;
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub overall: HealthStatus,
    pub components: HashMap<String, ComponentStatus>,
    pub timestamp: DateTime<Utc>,
    pub uptime: Option<String>,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
}

/// Execute status command with the expected interface
pub async fn execute(
    detailed: bool,
    format: String,
    component: Option<String>,
) -> Result<()> {
    show_status(detailed, format, component).await
}

pub async fn show_status(
    detailed: bool,
    format: String,
    component: Option<String>,
) -> Result<()> {
    let status = collect_system_status(detailed).await?;

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        "yaml" => {
            println!("{}", serde_yaml::to_string(&status)?);
        }
        _ => {
            display_status_table(&status, component.as_deref())?;
        }
    }

    Ok(())
}

async fn collect_system_status(detailed: bool) -> Result<SystemStatus> {
    let mut components = HashMap::new();
    let start_time = std::time::Instant::now();

    // Check database connectivity
    components.insert("database".to_string(), check_database().await);

    // Check Docker service
    components.insert("docker".to_string(), check_docker().await);

    // Check filesystem
    components.insert("filesystem".to_string(), check_filesystem().await);

    // Check system resources
    if detailed {
        components.insert("memory".to_string(), check_memory().await);
        components.insert("cpu".to_string(), check_cpu().await);
        components.insert("disk".to_string(), check_disk_space().await);
    }

    // Determine overall status
    let overall = determine_overall_status(&components);

    // Calculate total collection time
    let collection_time = start_time.elapsed().as_millis();
    tracing::debug!("System status collection completed in {}ms", collection_time);

    Ok(SystemStatus {
        overall,
        components,
        timestamp: Utc::now(),
        uptime: get_system_uptime().await.ok(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn check_database() -> ComponentStatus {
    let start = std::time::Instant::now();

    match std::env::var("DATABASE_URL") {
        Ok(database_url) => {
            match PgPool::connect(&database_url).await {
                Ok(pool) => {
                    match sqlx::query("SELECT 1").fetch_one(&pool).await {
                        Ok(_) => ComponentStatus {
                            status: HealthStatus::Healthy,
                            message: "Database connection successful".to_string(),
                            details: Some(serde_json::json!({
                                "url": mask_credentials(&database_url)
                            })),
                            last_check: Utc::now(),
                            response_time_ms: Some(start.elapsed().as_millis() as u64),
                        },
                        Err(e) => ComponentStatus {
                            status: HealthStatus::Critical,
                            message: format!("Database query failed: {}", e),
                            details: None,
                            last_check: Utc::now(),
                            response_time_ms: Some(start.elapsed().as_millis() as u64),
                        }
                    }
                }
                Err(e) => ComponentStatus {
                    status: HealthStatus::Critical,
                    message: format!("Database connection failed: {}", e),
                    details: None,
                    last_check: Utc::now(),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                }
            }
        }
        Err(_) => ComponentStatus {
            status: HealthStatus::Warning,
            message: "DATABASE_URL not configured".to_string(),
            details: None,
            last_check: Utc::now(),
            response_time_ms: Some(start.elapsed().as_millis() as u64),
        }
    }
}

async fn check_docker() -> ComponentStatus {
    let start = std::time::Instant::now();

    match Command::new("docker").arg("version").output() {
        Ok(output) => {
            if output.status.success() {
                ComponentStatus {
                    status: HealthStatus::Healthy,
                    message: "Docker is running".to_string(),
                    details: Some(serde_json::json!({
                        "version": String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("unknown")
                    })),
                    last_check: Utc::now(),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                }
            } else {
                ComponentStatus {
                    status: HealthStatus::Critical,
                    message: "Docker command failed".to_string(),
                    details: Some(serde_json::json!({
                        "error": String::from_utf8_lossy(&output.stderr)
                    })),
                    last_check: Utc::now(),
                    response_time_ms: Some(start.elapsed().as_millis() as u64),
                }
            }
        }
        Err(e) => ComponentStatus {
            status: HealthStatus::Critical,
            message: format!("Docker not available: {}", e),
            details: None,
            last_check: Utc::now(),
            response_time_ms: Some(start.elapsed().as_millis() as u64),
        }
    }
}

async fn check_filesystem() -> ComponentStatus {
    let start = std::time::Instant::now();

    match std::fs::metadata(".") {
        Ok(_) => ComponentStatus {
            status: HealthStatus::Healthy,
            message: "Filesystem accessible".to_string(),
            details: None,
            last_check: Utc::now(),
            response_time_ms: Some(start.elapsed().as_millis() as u64),
        },
        Err(e) => ComponentStatus {
            status: HealthStatus::Critical,
            message: format!("Filesystem error: {}", e),
            details: None,
            last_check: Utc::now(),
            response_time_ms: Some(start.elapsed().as_millis() as u64),
        }
    }
}

async fn check_memory() -> ComponentStatus {
    let start = std::time::Instant::now();

    // Platform-specific memory checking
    #[cfg(target_os = "linux")]
    {
        match std::fs::read_to_string("/proc/meminfo") {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let total = parse_meminfo_line(&lines, "MemTotal:");
                let available = parse_meminfo_line(&lines, "MemAvailable:");

                if let (Some(total), Some(available)) = (total, available) {
                    let used_percent = ((total - available) as f64 / total as f64) * 100.0;
                    let status = if used_percent > 90.0 {
                        HealthStatus::Critical
                    } else if used_percent > 80.0 {
                        HealthStatus::Warning
                    } else {
                        HealthStatus::Healthy
                    };

                    ComponentStatus {
                        status,
                        message: format!("Memory usage: {:.1}%", used_percent),
                        details: Some(serde_json::json!({
                            "total_mb": total / 1024,
                            "available_mb": available / 1024,
                            "used_percent": used_percent
                        })),
                        last_check: Utc::now(),
                        response_time_ms: Some(start.elapsed().as_millis() as u64),
                    }
                } else {
                    ComponentStatus {
                        status: HealthStatus::Unknown,
                        message: "Could not parse memory info".to_string(),
                        details: None,
                        last_check: Utc::now(),
                        response_time_ms: Some(start.elapsed().as_millis() as u64),
                    }
                }
            }
            Err(e) => ComponentStatus {
                status: HealthStatus::Unknown,
                message: format!("Memory check failed: {}", e),
                details: None,
                last_check: Utc::now(),
                response_time_ms: Some(start.elapsed().as_millis() as u64),
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        ComponentStatus {
            status: HealthStatus::Unknown,
            message: "Memory monitoring not implemented for this platform".to_string(),
            details: None,
            last_check: Utc::now(),
            response_time_ms: Some(start.elapsed().as_millis() as u64),
        }
    }
}

async fn check_cpu() -> ComponentStatus {
    let start = std::time::Instant::now();

    // Check CPU usage via /proc/loadavg on Linux systems
    #[cfg(target_os = "linux")]
    {
        match std::fs::read_to_string("/proc/loadavg") {
            Ok(content) => {
                let parts: Vec<&str> = content.split_whitespace().collect();
                if let Some(load_1min) = parts.get(0) {
                    if let Ok(load) = load_1min.parse::<f64>() {
                        let num_cpus = num_cpus::get() as f64;
                        let load_percentage = (load / num_cpus) * 100.0;

                        let status = if load_percentage > 90.0 {
                            HealthStatus::Critical
                        } else if load_percentage > 70.0 {
                            HealthStatus::Warning
                        } else {
                            HealthStatus::Healthy
                        };

                        return ComponentStatus {
                            status,
                            message: format!("CPU load: {:.1}% ({:.2} load avg, {} cores)",
                                           load_percentage, load, num_cpus),
                            details: Some(serde_json::json!({
                                "load_1min": load,
                                "load_percentage": load_percentage,
                                "cpu_cores": num_cpus
                            })),
                            last_check: Utc::now(),
                            response_time_ms: Some(start.elapsed().as_millis() as u64),
                        };
                    }
                }
            }
            Err(_) => {}
        }
    }

    // Fallback for non-Linux systems or when /proc/loadavg is not available
    let num_cpus = num_cpus::get();
    ComponentStatus {
        status: HealthStatus::Healthy,
        message: format!("CPU cores available: {}", num_cpus),
        details: Some(serde_json::json!({
            "cpu_cores": num_cpus,
            "note": "Detailed CPU monitoring not available on this platform"
        })),
        last_check: Utc::now(),
        response_time_ms: Some(start.elapsed().as_millis() as u64),
    }
}

async fn check_disk_space() -> ComponentStatus {
    let start = std::time::Instant::now();

    // Platform-specific disk space checking
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::mem;

        let path = CString::new(".").unwrap();
        let mut stat: libc::statvfs = unsafe { mem::zeroed() };

        if unsafe { libc::statvfs(path.as_ptr(), &mut stat) } == 0 {
            let total = stat.f_blocks * stat.f_frsize;
            let free = stat.f_bavail * stat.f_frsize;
            let used_percent = ((total - free) as f64 / total as f64) * 100.0;

            let status = if used_percent > 95.0 {
                HealthStatus::Critical
            } else if used_percent > 85.0 {
                HealthStatus::Warning
            } else {
                HealthStatus::Healthy
            };

            ComponentStatus {
                status,
                message: format!("Disk usage: {:.1}%", used_percent),
                details: Some(serde_json::json!({
                    "total_gb": total / (1024 * 1024 * 1024),
                    "free_gb": free / (1024 * 1024 * 1024),
                    "used_percent": used_percent
                })),
                last_check: Utc::now(),
                response_time_ms: Some(start.elapsed().as_millis() as u64),
            }
        } else {
            ComponentStatus {
                status: HealthStatus::Unknown,
                message: "Could not get disk statistics".to_string(),
                details: None,
                last_check: Utc::now(),
                response_time_ms: Some(start.elapsed().as_millis() as u64),
            }
        }
    }

    #[cfg(not(unix))]
    {
        ComponentStatus {
            status: HealthStatus::Unknown,
            message: "Disk monitoring not implemented for this platform".to_string(),
            details: None,
            last_check: Utc::now(),
            response_time_ms: Some(start.elapsed().as_millis() as u64),
        }
    }
}

async fn get_system_uptime() -> Result<String> {
    #[cfg(target_os = "linux")]
    {
        let uptime = std::fs::read_to_string("/proc/uptime")
            .context("Failed to read uptime")?;
        let seconds: f64 = uptime.split_whitespace()
            .next()
            .context("Invalid uptime format")?
            .parse()
            .context("Failed to parse uptime")?;

        let days = (seconds / 86400.0) as u64;
        let hours = ((seconds % 86400.0) / 3600.0) as u64;
        let minutes = ((seconds % 3600.0) / 60.0) as u64;

        Ok(format!("{}d {}h {}m", days, hours, minutes))
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok("Unknown".to_string())
    }
}

fn determine_overall_status(components: &HashMap<String, ComponentStatus>) -> HealthStatus {
    let mut has_critical = false;
    let mut has_warning = false;

    for status in components.values() {
        match status.status {
            HealthStatus::Critical => has_critical = true,
            HealthStatus::Warning => has_warning = true,
            _ => {}
        }
    }

    if has_critical {
        HealthStatus::Critical
    } else if has_warning {
        HealthStatus::Warning
    } else {
        HealthStatus::Healthy
    }
}

fn display_status_table(status: &SystemStatus, filter_component: Option<&str>) -> Result<()> {
    let overall_color = match status.overall {
        HealthStatus::Healthy => "green",
        HealthStatus::Warning => "yellow",
        HealthStatus::Critical => "red",
        HealthStatus::Unknown => "cyan",
    };

    println!("\n{}", "ERP System Status".bold());
    println!("Overall Status: {}", format!("{:?}", status.overall).color(overall_color).bold());
    println!("Timestamp: {}", status.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    if let Some(uptime) = &status.uptime {
        println!("Uptime: {}", uptime);
    }
    println!("Version: {}", status.version);
    println!();

    println!("{:<15} {:<10} {:<15} {:<40}", "Component", "Status", "Response Time", "Message");
    println!("{}", "-".repeat(80));

    for (name, component) in &status.components {
        if let Some(filter) = filter_component {
            if name != filter {
                continue;
            }
        }

        let status_color = match component.status {
            HealthStatus::Healthy => "green",
            HealthStatus::Warning => "yellow",
            HealthStatus::Critical => "red",
            HealthStatus::Unknown => "cyan",
        };

        let response_time = component.response_time_ms
            .map(|ms| format!("{}ms", ms))
            .unwrap_or_else(|| "-".to_string());

        println!(
            "{:<15} {:<10} {:<15} {:<40}",
            name,
            format!("{:?}", component.status).color(status_color),
            response_time,
            component.message
        );
    }

    println!();
    Ok(())
}

fn mask_credentials(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        let mut masked = parsed.clone();
        if parsed.password().is_some() {
            let _ = masked.set_password(Some("***"));
        }
        masked.to_string()
    } else {
        "***".to_string()
    }
}

#[cfg(target_os = "linux")]
fn parse_meminfo_line(lines: &[&str], prefix: &str) -> Option<u64> {
    lines
        .iter()
        .find(|line| line.starts_with(prefix))
        .and_then(|line| {
            line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
        })
}