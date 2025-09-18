use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Argon2, Params, Version,
};
use crate::{config::SecurityConfig, error::Result, Error};

#[derive(Clone)]
pub struct PasswordHasher {
    argon2: Argon2<'static>,
}

impl PasswordHasher {
    pub fn new(config: &SecurityConfig) -> Result<Self> {
        let params = Params::new(
            config.argon2_memory_cost,
            config.argon2_time_cost,
            config.argon2_parallelism,
            None,
        )
        .map_err(|e| Error::internal(format!("Invalid Argon2 parameters: {}", e)))?;

        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            Version::V0x13,
            params,
        );

        Ok(Self { argon2 })
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        
        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| Error::internal(format!("Failed to hash password: {}", e)))?;

        Ok(password_hash.to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| Error::internal(format!("Invalid password hash format: {}", e)))?;

        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(Error::internal(format!("Password verification error: {}", e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SecurityConfig;

    #[test]
    fn test_password_hashing() {
        let config = SecurityConfig {
            argon2_memory_cost: 65536,
            argon2_time_cost: 3,
            argon2_parallelism: 4,
            aes_encryption_key: "test-key".to_string(),
        };

        let hasher = PasswordHasher::new(&config).unwrap();
        let password = "SecurePassword123!";
        
        let hash = hasher.hash_password(password).unwrap();
        assert!(!hash.is_empty());
        assert_ne!(hash, password);

        assert!(hasher.verify_password(password, &hash).unwrap());
        assert!(!hasher.verify_password("WrongPassword", &hash).unwrap());
    }
}