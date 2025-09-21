//! Configuration management for the deployment CLI

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: Option<String>,
    pub install_dir: String,
    pub default_environment: String,
    pub docker: DockerConfig,
    pub backup: BackupConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub compose_file: String,
    pub registry: Option<String>,
    pub default_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub default_directory: String,
    pub retention_days: u32,
    pub compression_level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub health_check_url: String,
    pub metrics_url: Option<String>,
    pub log_directory: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: None,
            install_dir: "/opt/erp-system".to_string(),
            default_environment: "production".to_string(),
            docker: DockerConfig {
                compose_file: "docker-compose.yml".to_string(),
                registry: None,
                default_services: vec!["postgres".to_string(), "redis".to_string(), "erp-server".to_string()],
            },
            backup: BackupConfig {
                default_directory: "/opt/erp-system/backups".to_string(),
                retention_days: 30,
                compression_level: 6,
            },
            monitoring: MonitoringConfig {
                health_check_url: "http://localhost:8080/health".to_string(),
                metrics_url: Some("http://localhost:8080/metrics".to_string()),
                log_directory: "/var/log/erp-system".to_string(),
            },
        }
    }
}

pub fn load_config(config_path: Option<&str>) -> Result<Config> {
    let config_paths = vec![
        config_path.map(|s| s.to_string()),
        Some("/etc/erp-system/deploy.toml".to_string()),
        Some("deploy.toml".to_string()),
        dirs::config_dir().map(|d| d.join("erp-deploy").join("config.toml").to_string_lossy().to_string()),
    ];

    for path in config_paths.into_iter().flatten() {
        if Path::new(&path).exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            return Ok(config);
        }
    }

    // No config file found, use defaults
    Ok(Config::default())
}

pub async fn execute_config_command(
    cmd: crate::ConfigCommands,
    config: &Config,
) -> Result<()> {
    match cmd {
        crate::ConfigCommands::Show { section, format } => {
            show_config(config, section.as_deref(), &format)
        }
        crate::ConfigCommands::Set { key, value, scope, tenant } => {
            set_config(&key, &value, scope.as_deref().unwrap_or("global"), tenant.as_deref()).await
        }
        crate::ConfigCommands::Get { key, scope, tenant } => {
            get_config(&key, scope.as_deref().unwrap_or("global"), tenant.as_deref()).await
        }
        crate::ConfigCommands::Validate { file, detailed } => {
            validate_config(file.as_deref(), detailed)
        }
        crate::ConfigCommands::Generate { environment, output } => {
            generate_config(&environment, output.as_deref())
        }
    }
}

fn show_config(config: &Config, section: Option<&str>, format: &str) -> Result<()> {
    match format {
        "json" => {
            if let Some(section) = section {
                match section {
                    "docker" => println!("{}", serde_json::to_string_pretty(&config.docker)?),
                    "backup" => println!("{}", serde_json::to_string_pretty(&config.backup)?),
                    "monitoring" => println!("{}", serde_json::to_string_pretty(&config.monitoring)?),
                    _ => return Err(anyhow!("Unknown section: {}", section)),
                }
            } else {
                println!("{}", serde_json::to_string_pretty(config)?);
            }
        }
        "yaml" => {
            if let Some(section) = section {
                match section {
                    "docker" => println!("{}", serde_yaml::to_string(&config.docker)?),
                    "backup" => println!("{}", serde_yaml::to_string(&config.backup)?),
                    "monitoring" => println!("{}", serde_yaml::to_string(&config.monitoring)?),
                    _ => return Err(anyhow!("Unknown section: {}", section)),
                }
            } else {
                println!("{}", serde_yaml::to_string(config)?);
            }
        }
        _ => {
            // TOML format (default)
            if let Some(section) = section {
                match section {
                    "docker" => println!("{}", toml::to_string_pretty(&config.docker)?),
                    "backup" => println!("{}", toml::to_string_pretty(&config.backup)?),
                    "monitoring" => println!("{}", toml::to_string_pretty(&config.monitoring)?),
                    _ => return Err(anyhow!("Unknown section: {}", section)),
                }
            } else {
                println!("{}", toml::to_string_pretty(config)?);
            }
        }
    }

    Ok(())
}

async fn set_config(key: &str, value: &str, scope: &str, tenant: Option<&str>) -> Result<()> {
    match scope {
        "system" => {
            // Set system-wide configuration
            println!("Setting system config: {} = {}", key, value);
            // Implementation would update the config file
            Ok(())
        }
        "tenant" => {
            if let Some(tenant) = tenant {
                // Set tenant-specific configuration
                println!("Setting tenant config for {}: {} = {}", tenant, key, value);
                // Implementation would update tenant settings in database
                Ok(())
            } else {
                Err(anyhow!("Tenant name required for tenant scope"))
            }
        }
        _ => Err(anyhow!("Invalid scope: {}. Use 'system' or 'tenant'", scope)),
    }
}

async fn get_config(key: &str, scope: &str, tenant: Option<&str>) -> Result<()> {
    match scope {
        "system" => {
            // Get system-wide configuration
            println!("Getting system config: {}", key);
            // Implementation would read from config file
            Ok(())
        }
        "tenant" => {
            if let Some(tenant) = tenant {
                // Get tenant-specific configuration
                println!("Getting tenant config for {}: {}", tenant, key);
                // Implementation would read from database
                Ok(())
            } else {
                Err(anyhow!("Tenant name required for tenant scope"))
            }
        }
        _ => Err(anyhow!("Invalid scope: {}. Use 'system' or 'tenant'", scope)),
    }
}

fn validate_config(file: Option<&str>, detailed: bool) -> Result<()> {
    let config_path = file.unwrap_or("deploy.toml");

    if !Path::new(config_path).exists() {
        return Err(anyhow!("Configuration file not found: {}", config_path));
    }

    let content = std::fs::read_to_string(config_path)?;
    let config: Result<Config, _> = toml::from_str(&content);

    match config {
        Ok(config) => {
            println!("✅ Configuration is valid");

            if detailed {
                println!("\nConfiguration details:");
                println!("  Install directory: {}", config.install_dir);
                println!("  Default environment: {}", config.default_environment);
                println!("  Docker compose file: {}", config.docker.compose_file);
                println!("  Backup directory: {}", config.backup.default_directory);
                println!("  Health check URL: {}", config.monitoring.health_check_url);
            }
        }
        Err(e) => {
            return Err(anyhow!("Configuration validation failed: {}", e));
        }
    }

    Ok(())
}

fn generate_config(environment: &str, output: Option<&str>) -> Result<()> {
    let mut config = Config::default();

    // Customize config based on environment
    match environment {
        "development" => {
            config.install_dir = "./erp-system".to_string();
            config.monitoring.health_check_url = "http://localhost:3000/health".to_string();
            config.backup.retention_days = 7;
        }
        "staging" => {
            config.install_dir = "/opt/erp-system-staging".to_string();
            config.backup.retention_days = 14;
        }
        "production" => {
            // Use defaults
        }
        _ => {
            return Err(anyhow!("Unknown environment: {}. Use development, staging, or production", environment));
        }
    }

    let config_content = toml::to_string_pretty(&config)?;

    match output {
        Some(path) => {
            std::fs::write(path, config_content)?;
            println!("Configuration generated: {}", path);
        }
        None => {
            println!("{}", config_content);
        }
    }

    Ok(())
}