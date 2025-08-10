use crate::{error::Result, Error};
use totp_rs::{Algorithm, Secret, TOTP};
use rand::rngs::OsRng;
use rand::RngCore;

#[derive(Debug, Clone)]
pub struct TotpService {
    issuer: String,
}

impl TotpService {
    pub fn new(issuer: String) -> Self {
        Self { issuer }
    }

    pub fn generate_secret(&self) -> Result<String> {
        use totp_rs::Secret;
        
        // Generate cryptographically secure random 32-byte secret
        let mut secret_bytes = vec![0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        
        let secret = Secret::Raw(secret_bytes);
        Ok(secret.to_string())
    }

    pub fn generate_qr_code(&self, secret: &str, email: &str) -> Result<String> {
        let _totp = self.create_totp(secret, email)?;
        
        // Generate QR URL manually
        let qr_url = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}",
            self.issuer, email, secret, self.issuer
        );
        
        Ok(qr_url)
    }

    pub fn verify_code(&self, secret: &str, code: &str) -> Result<bool> {
        let totp = self.create_totp(secret, "")?;
        
        let parsed_code: u32 = code.parse()
            .map_err(|_| Error::validation("Invalid TOTP code format"))?;
        
        Ok(totp.check_current(&parsed_code.to_string())
            .map_err(|e| Error::internal(format!("TOTP verification error: {}", e)))?)
    }

    pub fn generate_code(&self, secret: &str) -> Result<String> {
        let totp = self.create_totp(secret, "")?;
        
        totp.generate_current()
            .map_err(|e| Error::internal(format!("TOTP code generation error: {}", e)))
    }

    pub fn generate_backup_codes(&self, count: usize) -> Result<Vec<String>> {
        use rand::Rng;
        let mut codes = Vec::new();
        
        for _ in 0..count {
            // Use cryptographically secure RNG for backup codes
            let code: u32 = OsRng.gen_range(100000..999999);
            codes.push(format!("{:06}", code));
        }
        
        Ok(codes)
    }

    fn create_totp(&self, secret: &str, _account_name: &str) -> Result<TOTP> {
        let secret = Secret::Encoded(secret.to_string())
            .to_bytes()
            .map_err(|e| Error::internal(format!("Invalid TOTP secret: {}", e)))?;
        
        TOTP::new(
            Algorithm::SHA256,
            6,
            1,
            30,
            secret,
        ).map_err(|e| Error::internal(format!("Failed to create TOTP: {}", e)))
    }
}