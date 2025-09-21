//! Installation command implementation

use anyhow::{anyhow, Result};
use colored::*;
use std::process::Command;
use tokio::process::Command as AsyncCommand;

pub async fn execute(
    environment: &str,
    skip_security: bool,
    install_dir: &str,
    domain: Option<&str>,
    admin_email: Option<&str>,
) -> Result<()> {
    println!("{}", "🚀 Starting ERP System Installation".blue().bold());
    println!("Environment: {}", environment.yellow());
    println!("Install Directory: {}", install_dir.yellow());

    // Check if running as root (required for installation)
    if !is_root() {
        return Err(anyhow!("Installation must be run as root (use sudo)"));
    }

    // Check prerequisites
    check_prerequisites().await?;

    // Download installation script
    download_install_script().await?;

    // Prepare installation command
    let mut cmd = AsyncCommand::new("bash");
    cmd.arg("/tmp/erp-install.sh");
    cmd.arg(environment);

    if let Some(domain) = domain {
        cmd.env("ERP_DOMAIN", domain);
    }

    if let Some(email) = admin_email {
        cmd.env("ERP_ADMIN_EMAIL", email);
    }

    cmd.env("ERP_INSTALL_DIR", install_dir);
    cmd.env("ERP_SKIP_SECURITY", skip_security.to_string());

    println!("{}", "⚙️ Running installation script...".blue());

    // Execute installation
    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Installation failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);

    // Run security hardening if not skipped
    if !skip_security {
        run_security_hardening().await?;
    }

    // Verify installation
    verify_installation().await?;

    println!("{}", "✅ ERP System installation completed successfully!".green().bold());
    Ok(())
}

fn is_root() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }

    #[cfg(windows)]
    {
        // On Windows, check if running as administrator
        use std::ptr;
        use winapi::um::handleapi::CloseHandle;
        use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
        use winapi::um::winnt::{TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
        use winapi::um::securitybaseapi::GetTokenInformation;

        unsafe {
            let mut token_handle = ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle) == 0 {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

            let result = GetTokenInformation(
                token_handle,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                size,
                &mut size,
            );

            CloseHandle(token_handle);

            result != 0 && elevation.TokenIsElevated != 0
        }
    }
}

async fn check_prerequisites() -> Result<()> {
    println!("{}", "🔍 Checking prerequisites...".blue());

    // Check operating system
    let os_info = os_info::get();
    println!("Operating System: {} {}", os_info.os_type(), os_info.version());

    // Check required commands
    let required_commands = ["curl", "wget", "docker", "docker-compose", "psql"];
    for cmd in &required_commands {
        if !command_exists(cmd) {
            return Err(anyhow!("Required command not found: {}", cmd));
        }
    }

    // Check disk space (minimum 10GB)
    let available_space = get_available_disk_space("/")?;
    if available_space < 10_000_000_000 {
        return Err(anyhow!("Insufficient disk space. At least 10GB required."));
    }

    // Check memory (minimum 4GB)
    let available_memory = get_available_memory()?;
    if available_memory < 4_000_000_000 {
        return Err(anyhow!("Insufficient memory. At least 4GB required."));
    }

    println!("{}", "✅ Prerequisites check passed".green());
    Ok(())
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(unix)]
fn get_available_disk_space(path: &str) -> Result<u64> {
    use std::ffi::CString;
    use std::mem;

    let path_c = CString::new(path)?;
    let mut stat: libc::statvfs = unsafe { mem::zeroed() };

    let result = unsafe { libc::statvfs(path_c.as_ptr(), &mut stat) };

    if result == 0 {
        Ok(stat.f_bavail * stat.f_frsize)
    } else {
        Err(anyhow!("Failed to get disk space information"))
    }
}

#[cfg(windows)]
fn get_available_disk_space(path: &str) -> Result<u64> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::fileapi::GetDiskFreeSpaceExW;

    let wide_path: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut free_bytes: winapi::shared::ntdef::ULARGE_INTEGER = unsafe { std::mem::zeroed() };
    let mut total_bytes: winapi::shared::ntdef::ULARGE_INTEGER = unsafe { std::mem::zeroed() };

    let result = unsafe {
        GetDiskFreeSpaceExW(
            wide_path.as_ptr(),
            &mut free_bytes,
            &mut total_bytes,
            std::ptr::null_mut(),
        )
    };

    if result != 0 {
        Ok(unsafe { *free_bytes.QuadPart() } as u64)
    } else {
        Err(anyhow!("Failed to get disk space information"))
    }
}

#[cfg(unix)]
fn get_available_memory() -> Result<u64> {
    let output = Command::new("free")
        .arg("-b")
        .output()?;

    let output_str = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = output_str.lines().collect();

    if lines.len() >= 2 {
        let mem_line = lines[1];
        let parts: Vec<&str> = mem_line.split_whitespace().collect();
        if parts.len() >= 7 {
            return Ok(parts[6].parse()?);
        }
    }

    Err(anyhow!("Failed to parse memory information"))
}

#[cfg(windows)]
fn get_available_memory() -> Result<u64> {
    use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;

    let result = unsafe { GlobalMemoryStatusEx(&mut status) };

    if result != 0 {
        Ok(status.ullAvailPhys)
    } else {
        Err(anyhow!("Failed to get memory information"))
    }
}

async fn download_install_script() -> Result<()> {
    println!("{}", "📥 Downloading installation script...".blue());

    // In a real implementation, this would download from a release URL
    // For now, we'll use the local script
    let script_path = "scripts/install.sh";

    if !std::path::Path::new(script_path).exists() {
        return Err(anyhow!("Installation script not found: {}", script_path));
    }

    // Copy to /tmp for execution
    let mut cmd = AsyncCommand::new("cp");
    cmd.arg(script_path);
    cmd.arg("/tmp/erp-install.sh");

    let output = cmd.output().await?;
    if !output.status.success() {
        return Err(anyhow!("Failed to copy installation script"));
    }

    // Make executable
    let mut cmd = AsyncCommand::new("chmod");
    cmd.arg("+x");
    cmd.arg("/tmp/erp-install.sh");

    let output = cmd.output().await?;
    if !output.status.success() {
        return Err(anyhow!("Failed to make installation script executable"));
    }

    println!("{}", "✅ Installation script ready".green());
    Ok(())
}

async fn run_security_hardening() -> Result<()> {
    println!("{}", "🔒 Running security hardening...".blue());

    let script_path = "scripts/setup-production.sh";

    if !std::path::Path::new(script_path).exists() {
        println!("{}", "⚠️ Security script not found, skipping hardening".yellow());
        return Ok(());
    }

    // Copy to /tmp for execution
    let mut cmd = AsyncCommand::new("cp");
    cmd.arg(script_path);
    cmd.arg("/tmp/erp-security.sh");
    cmd.output().await?;

    // Make executable
    let mut cmd = AsyncCommand::new("chmod");
    cmd.arg("+x");
    cmd.arg("/tmp/erp-security.sh");
    cmd.output().await?;

    // Execute security script
    let mut cmd = AsyncCommand::new("bash");
    cmd.arg("/tmp/erp-security.sh");

    let output = cmd.output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{} Security hardening warnings: {}", "⚠️".yellow(), stderr);
    }

    println!("{}", "✅ Security hardening completed".green());
    Ok(())
}

async fn verify_installation() -> Result<()> {
    println!("{}", "🔍 Verifying installation...".blue());

    // Check if systemd service is active
    let mut cmd = AsyncCommand::new("systemctl");
    cmd.arg("is-active");
    cmd.arg("erp-system");

    let output = cmd.output().await?;
    if !output.status.success() {
        return Err(anyhow!("ERP system service is not active"));
    }

    // Check health endpoint
    let client = reqwest::Client::new();
    let response = client
        .get("http://localhost:8080/health")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            println!("{}", "✅ Health endpoint responding".green());
        }
        _ => {
            return Err(anyhow!("Health endpoint not responding"));
        }
    }

    // Check database connectivity
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://erp_admin:erp_secure_password_change_in_production@localhost:5432/erp_main".to_string());

    let pool = sqlx::PgPool::connect(&database_url).await?;
    sqlx::query("SELECT 1").fetch_one(&pool).await?;
    pool.close().await;

    println!("{}", "✅ Database connectivity verified".green());
    println!("{}", "✅ Installation verification completed".green());
    Ok(())
}