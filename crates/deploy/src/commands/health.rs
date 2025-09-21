//! Health check command implementation

use anyhow::Result;
use colored::*;
use serde_json::json;
use std::time::Duration;
use crate::config::Config;

pub async fn execute(
    all: bool,
    component: Option<&str>,
    format: &str,
    config: &Config,
) -> Result<()> {
    if all {
        check_all_components(format, config).await
    } else if let Some(comp) = component {
        check_component(comp, format, config).await
    } else {
        check_basic_health(format, config).await
    }
}

async fn check_all_components(format: &str, config: &Config) -> Result<()> {
    println!("{}", "🔍 Checking all system components...".blue().bold());

    let mut results = vec![];

    // Check API health
    let api_result = check_api_health(&config.monitoring.health_check_url).await;
    results.push(("API", api_result));

    // Check database
    let db_result = check_database().await;
    results.push(("Database", db_result));

    // Check Redis
    let redis_result = check_redis().await;
    results.push(("Redis", redis_result));

    // Check Docker containers
    let docker_result = check_docker().await;
    results.push(("Docker", docker_result));

    // Check disk space
    let disk_result = check_disk_space(&config.install_dir).await;
    results.push(("Disk Space", disk_result));

    // Check memory
    let memory_result = check_memory().await;
    results.push(("Memory", memory_result));

    display_health_results(results, format)?;
    Ok(())
}

async fn check_component(component: &str, format: &str, config: &Config) -> Result<()> {
    let result = match component.to_lowercase().as_str() {
        "api" => check_api_health(&config.monitoring.health_check_url).await,
        "database" | "db" => check_database().await,
        "redis" => check_redis().await,
        "docker" => check_docker().await,
        "disk" => check_disk_space(&config.install_dir).await,
        "memory" | "mem" => check_memory().await,
        _ => {
            return Err(anyhow::anyhow!("Unknown component: {}", component));
        }
    };

    display_health_results(vec![(component, result)], format)?;
    Ok(())
}

async fn check_basic_health(format: &str, config: &Config) -> Result<()> {
    let api_result = check_api_health(&config.monitoring.health_check_url).await;
    display_health_results(vec![("API", api_result)], format)?;
    Ok(())
}

async fn check_api_health(url: &str) -> HealthResult {
    let client = reqwest::Client::new();

    match client
        .get(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                HealthResult {
                    status: HealthStatus::Healthy,
                    message: "API responding".to_string(),
                    details: Some(json!({
                        "status_code": response.status().as_u16(),
                        "response_time": "< 10s"
                    })),
                }
            } else {
                HealthResult {
                    status: HealthStatus::Unhealthy,
                    message: format!("API returned status: {}", response.status()),
                    details: Some(json!({
                        "status_code": response.status().as_u16()
                    })),
                }
            }
        }
        Err(e) => HealthResult {
            status: HealthStatus::Unhealthy,
            message: format!("API unreachable: {}", e),
            details: None,
        },
    }
}

async fn check_database() -> HealthResult {
    // Try to connect to database
    if let Ok(database_url) = std::env::var("DATABASE_URL") {
        match sqlx::PgPool::connect(&database_url).await {
            Ok(pool) => {
                match sqlx::query("SELECT 1").fetch_one(&pool).await {
                    Ok(_) => {
                        pool.close().await;
                        HealthResult {
                            status: HealthStatus::Healthy,
                            message: "Database connected".to_string(),
                            details: Some(json!({
                                "connection": "successful",
                                "query_test": "passed"
                            })),
                        }
                    }
                    Err(e) => HealthResult {
                        status: HealthStatus::Unhealthy,
                        message: format!("Database query failed: {}", e),
                        details: None,
                    },
                }
            }
            Err(e) => HealthResult {
                status: HealthStatus::Unhealthy,
                message: format!("Database connection failed: {}", e),
                details: None,
            },
        }
    } else {
        HealthResult {
            status: HealthStatus::Unknown,
            message: "DATABASE_URL not configured".to_string(),
            details: None,
        }
    }
}

async fn check_redis() -> HealthResult {
    // Try to connect to Redis via Docker
    match tokio::process::Command::new("docker")
        .args(&["exec", "erp-redis", "redis-cli", "ping"])
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout);
                if response.trim() == "PONG" {
                    HealthResult {
                        status: HealthStatus::Healthy,
                        message: "Redis responding".to_string(),
                        details: Some(json!({
                            "ping": "PONG"
                        })),
                    }
                } else {
                    HealthResult {
                        status: HealthStatus::Unhealthy,
                        message: format!("Redis unexpected response: {}", response),
                        details: None,
                    }
                }
            } else {
                HealthResult {
                    status: HealthStatus::Unhealthy,
                    message: "Redis ping failed".to_string(),
                    details: None,
                }
            }
        }
        Err(e) => HealthResult {
            status: HealthStatus::Unhealthy,
            message: format!("Redis check error: {}", e),
            details: None,
        },
    }
}

async fn check_docker() -> HealthResult {
    match tokio::process::Command::new("docker")
        .args(&["ps", "--format", "json"])
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                let containers_output = String::from_utf8_lossy(&output.stdout);
                let container_count = containers_output.lines().count();

                HealthResult {
                    status: HealthStatus::Healthy,
                    message: format!("{} containers running", container_count),
                    details: Some(json!({
                        "running_containers": container_count
                    })),
                }
            } else {
                HealthResult {
                    status: HealthStatus::Unhealthy,
                    message: "Docker command failed".to_string(),
                    details: None,
                }
            }
        }
        Err(e) => HealthResult {
            status: HealthStatus::Unhealthy,
            message: format!("Docker check error: {}", e),
            details: None,
        },
    }
}

async fn check_disk_space(install_dir: &str) -> HealthResult {
    match tokio::process::Command::new("df")
        .args(&["-h", install_dir])
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                let df_output = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = df_output.lines().nth(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        let used_percent = parts[4];
                        let available = parts[3];

                        let percent_num: u32 = used_percent
                            .trim_end_matches('%')
                            .parse()
                            .unwrap_or(0);

                        let status = if percent_num > 90 {
                            HealthStatus::Unhealthy
                        } else if percent_num > 80 {
                            HealthStatus::Warning
                        } else {
                            HealthStatus::Healthy
                        };

                        HealthResult {
                            status,
                            message: format!("Disk usage: {}, Available: {}", used_percent, available),
                            details: Some(json!({
                                "used_percent": percent_num,
                                "available": available
                            })),
                        }
                    } else {
                        HealthResult {
                            status: HealthStatus::Unknown,
                            message: "Could not parse disk usage".to_string(),
                            details: None,
                        }
                    }
                } else {
                    HealthResult {
                        status: HealthStatus::Unknown,
                        message: "Could not read disk usage".to_string(),
                        details: None,
                    }
                }
            } else {
                HealthResult {
                    status: HealthStatus::Unhealthy,
                    message: "Disk check command failed".to_string(),
                    details: None,
                }
            }
        }
        Err(e) => HealthResult {
            status: HealthStatus::Unhealthy,
            message: format!("Disk check error: {}", e),
            details: None,
        },
    }
}

async fn check_memory() -> HealthResult {
    match tokio::process::Command::new("free")
        .args(&["-m"])
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                let free_output = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = free_output.lines().nth(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let total: u32 = parts[1].parse().unwrap_or(0);
                        let used: u32 = parts[2].parse().unwrap_or(0);
                        let available: u32 = total - used;

                        let used_percent = if total > 0 { (used * 100) / total } else { 0 };

                        let status = if used_percent > 90 {
                            HealthStatus::Unhealthy
                        } else if used_percent > 80 {
                            HealthStatus::Warning
                        } else {
                            HealthStatus::Healthy
                        };

                        HealthResult {
                            status,
                            message: format!("Memory usage: {}%, Available: {}MB", used_percent, available),
                            details: Some(json!({
                                "total_mb": total,
                                "used_mb": used,
                                "available_mb": available,
                                "used_percent": used_percent
                            })),
                        }
                    } else {
                        HealthResult {
                            status: HealthStatus::Unknown,
                            message: "Could not parse memory usage".to_string(),
                            details: None,
                        }
                    }
                } else {
                    HealthResult {
                        status: HealthStatus::Unknown,
                        message: "Could not read memory usage".to_string(),
                        details: None,
                    }
                }
            } else {
                HealthResult {
                    status: HealthStatus::Unhealthy,
                    message: "Memory check command failed".to_string(),
                    details: None,
                }
            }
        }
        Err(e) => HealthResult {
            status: HealthStatus::Unhealthy,
            message: format!("Memory check error: {}", e),
            details: None,
        },
    }
}

fn display_health_results(results: Vec<(&str, HealthResult)>, format: &str) -> Result<()> {
    match format {
        "json" => {
            let json_results: Vec<serde_json::Value> = results
                .iter()
                .map(|(component, result)| {
                    json!({
                        "component": component,
                        "status": result.status.to_string(),
                        "message": result.message,
                        "details": result.details
                    })
                })
                .collect();

            println!("{}", serde_json::to_string_pretty(&json_results)?);
        }
        "yaml" => {
            let yaml_results: Vec<serde_json::Value> = results
                .iter()
                .map(|(component, result)| {
                    json!({
                        "component": component,
                        "status": result.status.to_string(),
                        "message": result.message,
                        "details": result.details
                    })
                })
                .collect();

            println!("{}", serde_yaml::to_string(&yaml_results)?);
        }
        _ => {
            // Table format
            println!("{}", "🏥 System Health Status:".blue().bold());
            println!("{:<15} {:<10} {:<50}",
                "Component", "Status", "Message");
            println!("{}", "-".repeat(75));

            for (component, result) in results {
                let status_colored = match result.status {
                    HealthStatus::Healthy => "Healthy".green(),
                    HealthStatus::Warning => "Warning".yellow(),
                    HealthStatus::Unhealthy => "Unhealthy".red(),
                    HealthStatus::Unknown => "Unknown".bright_black(),
                };

                println!("{:<15} {:<10} {:<50}",
                    component.cyan(),
                    status_colored,
                    result.message.white()
                );
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct HealthResult {
    status: HealthStatus,
    message: String,
    details: Option<serde_json::Value>,
}

#[derive(Debug)]
enum HealthStatus {
    Healthy,
    Warning,
    Unhealthy,
    Unknown,
}

impl ToString for HealthStatus {
    fn to_string(&self) -> String {
        match self {
            HealthStatus::Healthy => "healthy".to_string(),
            HealthStatus::Warning => "warning".to_string(),
            HealthStatus::Unhealthy => "unhealthy".to_string(),
            HealthStatus::Unknown => "unknown".to_string(),
        }
    }
}