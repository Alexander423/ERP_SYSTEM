use crate::{error::Result, Error};
use totp_rs::{Algorithm, Secret, TOTP};

pub struct TotpService {
    issuer: String,
}

impl TotpService {
    pub fn new(issuer: String) -> Self {
        Self { issuer }
    }

    pub fn generate_secret(&self) -> Result<String> {
        use totp_rs::Secret;
        let secret = Secret::Raw(vec![0u8; 20]); // Dummy 20-byte secret
        Ok(secret.to_string())
    }

    pub fn generate_qr_code(&self, secret: &str, email: &str) -> Result<String> {
        let totp = self.create_totp(secret, email)?;
        
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

    pub fn generate_backup_codes(&self, count: usize) -> Result<Vec<String>> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut codes = Vec::new();
        
        for _ in 0..count {
            let code: u32 = rng.gen_range(100000..999999);
            codes.push(format!("{:06}", code));
        }
        
        Ok(codes)
    }

    fn create_totp(&self, secret: &str, account_name: &str) -> Result<TOTP> {
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