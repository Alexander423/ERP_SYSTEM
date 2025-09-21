//! Docker management command implementations

use anyhow::{anyhow, Result};
use colored::*;
use serde_json::Value;
use tokio::process::Command;

use crate::DockerCommands;

pub async fn execute_docker_command(cmd: DockerCommands) -> Result<()> {
    match cmd {
        DockerCommands::Start { service, services, detach } => {
            let mut all_services = services;
            if let Some(s) = service {
                all_services.push(s);
            }
            start_services(all_services, detach).await
        }
        DockerCommands::Stop { service, services, force } => {
            let mut all_services = services;
            if let Some(s) = service {
                all_services.push(s);
            }
            stop_services(all_services, force).await
        }
        DockerCommands::Restart { services } => {
            restart_services(services).await
        }
        DockerCommands::Status { format } => {
            show_status(&format).await
        }
        DockerCommands::Logs { service, follow } => {
            show_logs(&service, follow).await
        }
        DockerCommands::Update { force } => {
            update_services(force).await
        }
    }
}

async fn start_services(services: Vec<String>, detach: bool) -> Result<()> {
    println!("{}", "🚀 Starting ERP system services...".blue().bold());

    // Check if Docker is running
    check_docker_running().await?;

    let services_to_start = if services.is_empty() {
        vec!["postgres".to_string(), "redis".to_string(), "erp-server".to_string()]
    } else {
        services
    };

    println!("Services to start: {}", services_to_start.join(", ").yellow());

    let mut cmd = Command::new("docker-compose");
    cmd.arg("up");

    if detach {
        cmd.arg("-d");
    }

    for service in &services_to_start {
        cmd.arg(service);
    }

    println!("Running: docker-compose up{}{}",
        if detach { " -d" } else { "" },
        if !services_to_start.is_empty() {
            format!(" {}", services_to_start.join(" "))
        } else {
            String::new()
        }
    );

    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to start services: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        println!("{}", stdout);
    }

    println!("{}", "✅ Services started successfully".green().bold());
    Ok(())
}

async fn stop_services(services: Vec<String>, force: bool) -> Result<()> {
    println!("{}", "🛑 Stopping ERP system services...".blue().bold());

    check_docker_running().await?;

    let services_to_stop = if services.is_empty() {
        vec!["erp-server".to_string(), "redis".to_string(), "postgres".to_string()]
    } else {
        services
    };

    println!("Services to stop: {}", services_to_stop.join(", ").yellow());

    let mut cmd = Command::new("docker-compose");
    if force {
        cmd.arg("kill");
    } else {
        cmd.arg("stop");
    }

    for service in &services_to_stop {
        cmd.arg(service);
    }

    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to stop services: {}", stderr));
    }

    println!("{}", "✅ Services stopped successfully".green().bold());
    Ok(())
}

async fn restart_services(services: Vec<String>) -> Result<()> {
    println!("{}", "🔄 Restarting ERP system services...".blue().bold());

    check_docker_running().await?;

    let services_to_restart = if services.is_empty() {
        vec!["erp-server".to_string()]
    } else {
        services
    };

    println!("Services to restart: {}", services_to_restart.join(", ").yellow());

    let mut cmd = Command::new("docker-compose");
    cmd.arg("restart");

    for service in &services_to_restart {
        cmd.arg(service);
    }

    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to restart services: {}", stderr));
    }

    println!("{}", "✅ Services restarted successfully".green().bold());
    Ok(())
}

async fn show_status(format: &str) -> Result<()> {
    println!("{}", "📊 Docker container status...".blue().bold());

    check_docker_running().await?;

    let output = Command::new("docker-compose")
        .arg("ps")
        .arg("--format")
        .arg("json")
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to get container status: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    match format {
        "json" => {
            println!("{}", stdout);
        }
        "yaml" => {
            // Parse JSON and convert to YAML
            let containers: Vec<Value> = stdout
                .lines()
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect();
            println!("{}", serde_yaml::to_string(&containers)?);
        }
        _ => {
            // Table format
            if stdout.trim().is_empty() {
                println!("No containers running");
                return Ok(());
            }

            println!("{:<20} {:<15} {:<10} {:<20} {:<30}",
                "Name", "Service", "State", "Status", "Ports");
            println!("{}", "-".repeat(95));

            for line in stdout.lines() {
                if let Ok(container) = serde_json::from_str::<Value>(line) {
                    let name = container["Name"].as_str().unwrap_or("N/A");
                    let service = container["Service"].as_str().unwrap_or("N/A");
                    let state = container["State"].as_str().unwrap_or("N/A");
                    let status = container["Status"].as_str().unwrap_or("N/A");
                    let ports = container["Publishers"].as_array()
                        .map(|p| p.iter()
                            .filter_map(|port| port["PublishedPort"].as_u64())
                            .map(|p| p.to_string())
                            .collect::<Vec<_>>()
                            .join(","))
                        .unwrap_or_default();

                    let state_colored = match state {
                        "running" => state.green(),
                        "exited" => state.red(),
                        "paused" => state.yellow(),
                        _ => state.normal(),
                    };

                    println!("{:<20} {:<15} {:<10} {:<20} {:<30}",
                        name.cyan(),
                        service.white(),
                        state_colored,
                        status.bright_black(),
                        ports.blue()
                    );
                }
            }
        }
    }

    Ok(())
}

async fn show_logs(service: &str, follow: bool) -> Result<()> {
    println!("{}", format!("📋 Showing logs for service: {}", service).blue().bold());

    check_docker_running().await?;

    let lines = 100; // Default number of lines to show

    let mut cmd = Command::new("docker-compose");
    cmd.arg("logs");

    if follow {
        cmd.arg("--follow");
    }

    cmd.arg("--tail").arg(lines.to_string());
    cmd.arg(service);

    println!("Running: docker-compose logs{} --tail {} {}",
        if follow { " --follow" } else { "" },
        lines,
        service
    );

    let output = if follow {
        // For follow mode, stream the output
        let mut child = cmd.spawn()?;
        child.wait().await?;
        return Ok(());
    } else {
        cmd.output().await?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to get logs: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);

    Ok(())
}

async fn update_services(force: bool) -> Result<()> {
    println!("{}", "📦 Updating container images...".blue().bold());

    check_docker_running().await?;

    let services_to_update = vec!["postgres".to_string(), "redis".to_string(), "erp-server".to_string()];

    if !force {
        use dialoguer::Confirm;
        if !Confirm::new()
            .with_prompt("This will pull latest images and restart services. Continue?")
            .interact()?
        {
            println!("Update cancelled");
            return Ok(());
        }
    }

    println!("Services to update: {}", services_to_update.join(", ").yellow());

    // Pull latest images
    println!("Pulling latest images...");
    for service in &services_to_update {
        let mut cmd = Command::new("docker-compose");
        cmd.arg("pull").arg(service);

        let output = cmd.output().await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("{}", format!("Warning: Failed to pull {}: {}", service, stderr).yellow());
        } else {
            println!("✅ Pulled latest image for {}", service.green());
        }
    }

    // Restart services with new images
    println!("Restarting services with updated images...");
    let mut cmd = Command::new("docker-compose");
    cmd.arg("up")
       .arg("-d")
       .arg("--force-recreate");

    for service in &services_to_update {
        cmd.arg(service);
    }

    let output = cmd.output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to restart services: {}", stderr));
    }

    println!("{}", "✅ Services updated successfully".green().bold());
    Ok(())
}

async fn check_docker_running() -> Result<()> {
    let output = Command::new("docker")
        .arg("info")
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => Ok(()),
        Ok(_) => Err(anyhow!("Docker is not running or not accessible")),
        Err(_) => Err(anyhow!("Docker command not found. Please install Docker.")),
    }
}