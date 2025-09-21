//! Utility functions for the deployment CLI

use anyhow::Result;
use std::process::Command;

/// Check if a command exists in the system PATH
pub fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Format bytes into human-readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

/// Generate a secure random password
pub fn generate_password(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                             abcdefghijklmnopqrstuvwxyz\
                             0123456789\
                             !@#$%^&*";

    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Validate email format
pub fn is_valid_email(email: &str) -> bool {
    use regex::Regex;
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

/// Convert a name to a valid database schema name
pub fn to_schema_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Check if running as root/administrator
pub fn is_root() -> bool {
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

            let mut elevation: u32 = 0;
            let mut size = std::mem::size_of::<u32>() as u32;

            let result = GetTokenInformation(
                token_handle,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                size,
                &mut size,
            );

            CloseHandle(token_handle);

            result != 0 && elevation != 0
        }
    }
}

/// Get system information
pub fn get_system_info() -> Result<SystemInfo> {
    let os_info = os_info::get();

    Ok(SystemInfo {
        os_type: os_info.os_type().to_string(),
        os_version: os_info.version().to_string(),
        hostname: hostname::get()?.to_string_lossy().to_string(),
        architecture: std::env::consts::ARCH.to_string(),
    })
}

#[derive(Debug)]
pub struct SystemInfo {
    pub os_type: String,
    pub os_version: String,
    pub hostname: String,
    pub architecture: String,
}

/// Progress bar utility
pub struct ProgressBar {
    total: usize,
    current: usize,
    width: usize,
}

impl ProgressBar {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            current: 0,
            width: 50,
        }
    }

    pub fn update(&mut self, current: usize) {
        self.current = current;
        self.display();
    }

    pub fn increment(&mut self) {
        self.current = (self.current + 1).min(self.total);
        self.display();
    }

    pub fn finish(&mut self) {
        self.current = self.total;
        self.display();
        println!();
    }

    fn display(&self) {
        let progress = if self.total > 0 {
            (self.current as f64 / self.total as f64).min(1.0)
        } else {
            0.0
        };

        let filled = (progress * self.width as f64) as usize;
        let empty = self.width - filled;

        let bar = format!(
            "[{}{}] {}/{} ({:.1}%)",
            "=".repeat(filled),
            " ".repeat(empty),
            self.current,
            self.total,
            progress * 100.0
        );

        print!("\r{}", bar);
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512.0 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name+tag@domain.co.uk"));
        assert!(!is_valid_email("invalid.email"));
        assert!(!is_valid_email("@domain.com"));
        assert!(!is_valid_email("user@"));
    }

    #[test]
    fn test_to_schema_name() {
        assert_eq!(to_schema_name("Acme Corp"), "acme_corp");
        assert_eq!(to_schema_name("Test-Company 123"), "test_company_123");
        assert_eq!(to_schema_name("simple"), "simple");
    }

    #[test]
    fn test_generate_password() {
        let password = generate_password(12);
        assert_eq!(password.len(), 12);
        assert!(password.chars().all(|c| c.is_ascii()));
    }
}