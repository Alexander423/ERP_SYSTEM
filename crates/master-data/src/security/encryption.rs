//! Field-level encryption for sensitive customer data
//!
//! This module provides enterprise-grade field-level encryption capabilities
//! to protect sensitive customer information at rest and in transit.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{MasterDataError, Result};

/// Field-level encryption service for sensitive data
#[async_trait::async_trait]
pub trait FieldEncryption: Send + Sync {
    /// Encrypt a field value with the specified encryption context
    async fn encrypt_field(
        &self,
        value: &str,
        field_name: &str,
        context: &EncryptionContext,
    ) -> Result<EncryptedField>;

    /// Decrypt a field value
    async fn decrypt_field(
        &self,
        encrypted_field: &EncryptedField,
        context: &EncryptionContext,
    ) -> Result<String>;

    /// Encrypt multiple fields in batch
    async fn encrypt_fields(
        &self,
        fields: &HashMap<String, String>,
        context: &EncryptionContext,
    ) -> Result<HashMap<String, EncryptedField>>;

    /// Decrypt multiple fields in batch
    async fn decrypt_fields(
        &self,
        encrypted_fields: &HashMap<String, EncryptedField>,
        context: &EncryptionContext,
    ) -> Result<HashMap<String, String>>;

    /// Rotate encryption keys for enhanced security
    async fn rotate_encryption_keys(&self, context: &EncryptionContext) -> Result<()>;

    /// Validate field encryption integrity
    async fn validate_encryption(&self, encrypted_field: &EncryptedField) -> Result<bool>;
}

/// Encryption context containing tenant and user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionContext {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub operation: EncryptionOperation,
    pub compliance_level: ComplianceLevel,
    pub data_classification: DataClassification,
}

/// Types of encryption operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionOperation {
    Create,
    Read,
    Update,
    Delete,
    Export,
    Backup,
}

/// Compliance levels requiring different encryption strengths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceLevel {
    Standard,    // AES-256
    High,        // AES-256 with key rotation
    Critical,    // AES-256 with per-field keys and HSM
}

/// Data classification levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
    TopSecret,
}

/// Encrypted field with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedField {
    /// Base64-encoded encrypted data
    pub encrypted_data: String,
    /// Base64-encoded nonce/IV
    pub nonce: String,
    /// Encryption algorithm used
    pub algorithm: EncryptionAlgorithm,
    /// Key ID for key rotation
    pub key_id: String,
    /// Field name for audit purposes
    pub field_name: String,
    /// Encryption timestamp
    pub encrypted_at: chrono::DateTime<chrono::Utc>,
    /// Data integrity hash
    pub integrity_hash: String,
    /// Encryption context hash for validation
    pub context_hash: String,
}

/// Supported encryption algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    Aes256Gcm128, // With 128-bit authentication tag
    ChaCha20Poly1305,
}

/// Enterprise encryption service implementation
pub struct EncryptionService {
    master_key: Vec<u8>,
    key_derivation_cache: std::sync::Arc<std::sync::RwLock<HashMap<String, Vec<u8>>>>,
    compliance_policies: std::sync::Arc<std::sync::RwLock<HashMap<DataClassification, EncryptionPolicy>>>,
}

/// Encryption policy defining how different data types should be encrypted
#[derive(Debug, Clone)]
pub struct EncryptionPolicy {
    pub algorithm: EncryptionAlgorithm,
    pub key_rotation_interval: chrono::Duration,
    pub requires_hsm: bool,
    pub requires_per_field_keys: bool,
    pub audit_all_operations: bool,
}

impl EncryptionService {
    /// Create a new encryption service with master key
    pub fn new(master_key: &[u8]) -> Result<Self> {
        if master_key.len() != 32 {
            return Err(MasterDataError::ValidationError {
                field: "master_key".to_string(),
                message: "Master key must be exactly 32 bytes (256 bits)".to_string(),
            });
        }

        let mut compliance_policies = HashMap::new();

        // Standard compliance policy
        compliance_policies.insert(DataClassification::Public, EncryptionPolicy {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_rotation_interval: chrono::Duration::days(365),
            requires_hsm: false,
            requires_per_field_keys: false,
            audit_all_operations: false,
        });

        // High security policy for confidential data
        compliance_policies.insert(DataClassification::Confidential, EncryptionPolicy {
            algorithm: EncryptionAlgorithm::Aes256Gcm128,
            key_rotation_interval: chrono::Duration::days(90),
            requires_hsm: false,
            requires_per_field_keys: true,
            audit_all_operations: true,
        });

        // Maximum security for restricted data
        compliance_policies.insert(DataClassification::Restricted, EncryptionPolicy {
            algorithm: EncryptionAlgorithm::Aes256Gcm128,
            key_rotation_interval: chrono::Duration::days(30),
            requires_hsm: true,
            requires_per_field_keys: true,
            audit_all_operations: true,
        });

        Ok(Self {
            master_key: master_key.to_vec(),
            key_derivation_cache: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
            compliance_policies: std::sync::Arc::new(std::sync::RwLock::new(compliance_policies)),
        })
    }

    /// Derive field-specific encryption key
    fn derive_field_key(&self, context: &EncryptionContext, field_name: &str) -> Result<Vec<u8>> {
        use sha2::{Sha256, Digest};

        let cache_key = format!("{}:{}:{}", context.tenant_id, field_name, context.user_id);

        // Check cache first
        {
            let cache = self.key_derivation_cache.read().unwrap();
            if let Some(cached_key) = cache.get(&cache_key) {
                return Ok(cached_key.clone());
            }
        }

        // Derive new key using HKDF-like approach
        let mut hasher = Sha256::new();
        hasher.update(&self.master_key);
        hasher.update(context.tenant_id.as_bytes());
        hasher.update(field_name.as_bytes());
        hasher.update(context.user_id.as_bytes());

        // Add data classification salt
        match context.data_classification {
            DataClassification::Restricted => hasher.update(b"RESTRICTED_SALT_2024"),
            DataClassification::Confidential => hasher.update(b"CONFIDENTIAL_SALT_2024"),
            _ => hasher.update(b"STANDARD_SALT_2024"),
        }

        let derived_key = hasher.finalize().to_vec();

        // Cache the derived key
        {
            let mut cache = self.key_derivation_cache.write().unwrap();
            cache.insert(cache_key, derived_key.clone());
        }

        Ok(derived_key)
    }

    /// Generate context hash for integrity verification
    fn generate_context_hash(&self, context: &EncryptionContext) -> String {
        use sha2::{Sha256, Digest};

        let context_str = format!(
            "{}:{}:{:?}:{:?}:{:?}",
            context.tenant_id,
            context.user_id,
            context.operation,
            context.compliance_level,
            context.data_classification
        );

        let mut hasher = Sha256::new();
        hasher.update(context_str.as_bytes());
        general_purpose::STANDARD.encode(hasher.finalize())
    }

    /// Generate integrity hash for encrypted data
    fn generate_integrity_hash(&self, encrypted_data: &[u8], nonce: &[u8]) -> String {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();
        hasher.update(encrypted_data);
        hasher.update(nonce);
        hasher.update(&self.master_key);
        general_purpose::STANDARD.encode(hasher.finalize())
    }
}

#[async_trait::async_trait]
impl FieldEncryption for EncryptionService {
    async fn encrypt_field(
        &self,
        value: &str,
        field_name: &str,
        context: &EncryptionContext,
    ) -> Result<EncryptedField> {
        // Check compliance policies for this data classification
        let policies = self.compliance_policies.read().unwrap();
        let _policy = policies.get(&context.data_classification)
            .ok_or_else(|| MasterDataError::ValidationError {
                field: field_name.to_string(),
                message: "No encryption policy found for data classification".to_string(),
            })?;

        let field_key = self.derive_field_key(context, field_name)?;
        let key = Key::<Aes256Gcm>::from_slice(&field_key);
        let cipher = Aes256Gcm::new(key);

        // Generate random nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Encrypt the data
        let encrypted_data = cipher
            .encrypt(&nonce, value.as_bytes())
            .map_err(|e| MasterDataError::ValidationError {
                field: "encryption".to_string(),
                message: format!("Encryption failed: {}", e),
            })?;

        let encrypted_field = EncryptedField {
            encrypted_data: general_purpose::STANDARD.encode(&encrypted_data),
            nonce: general_purpose::STANDARD.encode(&nonce),
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_id: format!("{}:{}", context.tenant_id, chrono::Utc::now().timestamp()),
            field_name: field_name.to_string(),
            encrypted_at: chrono::Utc::now(),
            integrity_hash: self.generate_integrity_hash(&encrypted_data, &nonce),
            context_hash: self.generate_context_hash(context),
        };

        Ok(encrypted_field)
    }

    async fn decrypt_field(
        &self,
        encrypted_field: &EncryptedField,
        context: &EncryptionContext,
    ) -> Result<String> {
        // Validate context
        let current_context_hash = self.generate_context_hash(context);
        if current_context_hash != encrypted_field.context_hash {
            return Err(MasterDataError::ValidationError {
                field: "context".to_string(),
                message: "Encryption context mismatch".to_string(),
            });
        }

        let field_key = self.derive_field_key(context, &encrypted_field.field_name)?;
        let key = Key::<Aes256Gcm>::from_slice(&field_key);
        let cipher = Aes256Gcm::new(key);

        // Decode from base64
        let encrypted_data = general_purpose::STANDARD
            .decode(&encrypted_field.encrypted_data)
            .map_err(|e| MasterDataError::ValidationError {
                field: "encrypted_data".to_string(),
                message: format!("Invalid base64 encoding: {}", e),
            })?;

        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted_field.nonce)
            .map_err(|e| MasterDataError::ValidationError {
                field: "nonce".to_string(),
                message: format!("Invalid nonce encoding: {}", e),
            })?;

        // Validate integrity
        let expected_hash = self.generate_integrity_hash(&encrypted_data, &nonce_bytes);
        if expected_hash != encrypted_field.integrity_hash {
            return Err(MasterDataError::ValidationError {
                field: "integrity".to_string(),
                message: "Data integrity check failed".to_string(),
            });
        }

        let nonce = Nonce::from_slice(&nonce_bytes);

        // Decrypt the data
        let decrypted_data = cipher
            .decrypt(nonce, encrypted_data.as_ref())
            .map_err(|e| MasterDataError::ValidationError {
                field: "decryption".to_string(),
                message: format!("Decryption failed: {}", e),
            })?;

        String::from_utf8(decrypted_data).map_err(|e| MasterDataError::ValidationError {
            field: "utf8".to_string(),
            message: format!("Invalid UTF-8 data: {}", e),
        })
    }

    async fn encrypt_fields(
        &self,
        fields: &HashMap<String, String>,
        context: &EncryptionContext,
    ) -> Result<HashMap<String, EncryptedField>> {
        let mut encrypted_fields = HashMap::new();

        for (field_name, value) in fields {
            let encrypted_field = self.encrypt_field(value, field_name, context).await?;
            encrypted_fields.insert(field_name.clone(), encrypted_field);
        }

        Ok(encrypted_fields)
    }

    async fn decrypt_fields(
        &self,
        encrypted_fields: &HashMap<String, EncryptedField>,
        context: &EncryptionContext,
    ) -> Result<HashMap<String, String>> {
        let mut decrypted_fields = HashMap::new();

        for (field_name, encrypted_field) in encrypted_fields {
            let decrypted_value = self.decrypt_field(encrypted_field, context).await?;
            decrypted_fields.insert(field_name.clone(), decrypted_value);
        }

        Ok(decrypted_fields)
    }

    async fn rotate_encryption_keys(&self, _context: &EncryptionContext) -> Result<()> {
        // Clear the key derivation cache to force new key generation
        {
            let mut cache = self.key_derivation_cache.write().unwrap();
            cache.clear();
        }

        // In a production system, this would:
        // 1. Generate new master keys
        // 2. Re-encrypt all data with new keys
        // 3. Update key management system
        // 4. Archive old keys securely

        Ok(())
    }

    async fn validate_encryption(&self, encrypted_field: &EncryptedField) -> Result<bool> {
        // Validate that all required fields are present
        if encrypted_field.encrypted_data.is_empty()
            || encrypted_field.nonce.is_empty()
            || encrypted_field.integrity_hash.is_empty() {
            return Ok(false);
        }

        // Validate base64 encoding
        if general_purpose::STANDARD.decode(&encrypted_field.encrypted_data).is_err()
            || general_purpose::STANDARD.decode(&encrypted_field.nonce).is_err() {
            return Ok(false);
        }

        // Additional validation would include:
        // - Key ID validity
        // - Encryption algorithm support
        // - Timestamp validation
        // - Compliance policy adherence

        Ok(true)
    }
}

/// Helper functions for sensitive field identification
pub struct SensitiveFieldIdentifier;

impl SensitiveFieldIdentifier {
    /// Identify which fields require encryption based on data type and regulations
    pub fn identify_sensitive_fields(data: &serde_json::Value) -> Vec<(String, DataClassification)> {
        let mut sensitive_fields = Vec::new();

        // Common PII fields that require encryption
        let pii_patterns = [
            ("email", DataClassification::Confidential),
            ("phone", DataClassification::Confidential),
            ("ssn", DataClassification::Restricted),
            ("tax_id", DataClassification::Restricted),
            ("credit_card", DataClassification::Restricted),
            ("bank_account", DataClassification::Restricted),
            ("passport", DataClassification::Restricted),
            ("driver_license", DataClassification::Restricted),
            ("medical_record", DataClassification::Restricted),
            ("financial_info", DataClassification::Confidential),
            ("salary", DataClassification::Confidential),
            ("notes", DataClassification::Internal),
        ];

        if let serde_json::Value::Object(obj) = data {
            for (key, _value) in obj {
                let key_lower = key.to_lowercase();
                for (pattern, classification) in &pii_patterns {
                    if key_lower.contains(pattern) {
                        sensitive_fields.push((key.clone(), classification.clone()));
                        break;
                    }
                }
            }
        }

        sensitive_fields
    }

    /// Determine encryption requirements based on compliance frameworks
    pub fn get_encryption_requirements(
        _field_name: &str,
        data_classification: &DataClassification,
    ) -> EncryptionRequirement {
        match data_classification {
            DataClassification::Restricted => EncryptionRequirement {
                required: true,
                algorithm: EncryptionAlgorithm::Aes256Gcm128,
                key_rotation_days: 30,
                audit_required: true,
                hsm_required: true,
            },
            DataClassification::Confidential => EncryptionRequirement {
                required: true,
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_rotation_days: 90,
                audit_required: true,
                hsm_required: false,
            },
            DataClassification::Internal => EncryptionRequirement {
                required: true,
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_rotation_days: 365,
                audit_required: false,
                hsm_required: false,
            },
            _ => EncryptionRequirement {
                required: false,
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                key_rotation_days: 365,
                audit_required: false,
                hsm_required: false,
            },
        }
    }
}

/// Encryption requirements for specific fields
#[derive(Debug, Clone)]
pub struct EncryptionRequirement {
    pub required: bool,
    pub algorithm: EncryptionAlgorithm,
    pub key_rotation_days: i64,
    pub audit_required: bool,
    pub hsm_required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_field_encryption_decryption() {
        let master_key = b"test_master_key_32_bytes_exactly";
        let encryption_service = EncryptionService::new(master_key).unwrap();

        let context = EncryptionContext {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            operation: EncryptionOperation::Create,
            compliance_level: ComplianceLevel::Standard,
            data_classification: DataClassification::Confidential,
        };

        let original_value = "sensitive_customer_data@example.com";
        let field_name = "email";

        // Encrypt
        let encrypted_field = encryption_service
            .encrypt_field(original_value, field_name, &context)
            .await
            .unwrap();

        // Verify encryption worked
        assert_ne!(encrypted_field.encrypted_data, original_value);
        assert!(!encrypted_field.encrypted_data.is_empty());
        assert!(!encrypted_field.nonce.is_empty());

        // Decrypt
        let decrypted_value = encryption_service
            .decrypt_field(&encrypted_field, &context)
            .await
            .unwrap();

        // Verify decryption worked
        assert_eq!(decrypted_value, original_value);
    }

    #[tokio::test]
    async fn test_encryption_validation() {
        let master_key = b"test_master_key_32_bytes_exactly";
        let encryption_service = EncryptionService::new(master_key).unwrap();

        let context = EncryptionContext {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            operation: EncryptionOperation::Create,
            compliance_level: ComplianceLevel::Standard,
            data_classification: DataClassification::Confidential,
        };

        let encrypted_field = encryption_service
            .encrypt_field("test@example.com", "email", &context)
            .await
            .unwrap();

        // Valid encryption should pass validation
        let is_valid = encryption_service
            .validate_encryption(&encrypted_field)
            .await
            .unwrap();
        assert!(is_valid);

        // Invalid encryption should fail validation
        let mut invalid_field = encrypted_field.clone();
        invalid_field.encrypted_data = "invalid_base64!@#".to_string();

        let is_valid = encryption_service
            .validate_encryption(&invalid_field)
            .await
            .unwrap();
        assert!(!is_valid);
    }
}