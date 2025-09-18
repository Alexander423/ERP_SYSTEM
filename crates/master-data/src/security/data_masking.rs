//! Data masking and privacy controls for GDPR and other privacy regulations
//!
//! This module provides comprehensive data masking capabilities to protect
//! sensitive information while maintaining data utility for analytics and testing.

use async_trait::async_trait;
use rand::Rng;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::Result;

/// Data masking service for privacy protection
#[async_trait]
pub trait DataMasking: Send + Sync {
    /// Apply masking to a single field value
    async fn mask_field(
        &self,
        value: &str,
        policy: &MaskingPolicy,
        context: &MaskingContext,
    ) -> Result<String>;

    /// Apply masking to multiple fields
    async fn mask_fields(
        &self,
        data: &HashMap<String, String>,
        policies: &HashMap<String, MaskingPolicy>,
        context: &MaskingContext,
    ) -> Result<HashMap<String, String>>;

    /// Mask structured data (JSON)
    async fn mask_structured_data(
        &self,
        data: &serde_json::Value,
        policies: &HashMap<String, MaskingPolicy>,
        context: &MaskingContext,
    ) -> Result<serde_json::Value>;

    /// Check if user has permission to view unmasked data
    async fn can_view_unmasked(
        &self,
        user_id: Uuid,
        field: &str,
        context: &MaskingContext,
    ) -> Result<bool>;

    /// Create masking policy
    async fn create_policy(&self, policy: &MaskingPolicy) -> Result<Uuid>;

    /// Update masking policy
    async fn update_policy(&self, policy: &MaskingPolicy) -> Result<()>;

    /// Get masking policies for table
    async fn get_table_policies(&self, table_name: &str, tenant_id: Uuid) -> Result<Vec<MaskingPolicy>>;
}

/// Masking policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub table_name: String,
    pub column_name: String,
    pub masking_type: MaskingType,
    pub masking_config: MaskingConfig,
    pub conditions: Option<Vec<MaskingCondition>>,
    pub exemptions: Option<Vec<MaskingExemption>>,
    pub is_active: bool,
    pub tenant_id: Uuid,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_by: Uuid,
    pub modified_at: chrono::DateTime<chrono::Utc>,
}

/// Types of data masking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaskingType {
    /// Replace with static value
    StaticReplacement,
    /// Replace with random characters
    RandomReplacement,
    /// Partial masking (show first/last N characters)
    PartialMasking,
    /// Format preserving encryption
    FormatPreservingEncryption,
    /// Shuffle/randomize within dataset
    Shuffling,
    /// Replace with fake but realistic data
    Substitution,
    /// Redact completely
    Redaction,
    /// Hash the value
    Hashing,
    /// Tokenize the value
    Tokenization,
    /// Date shifting
    DateShifting,
    /// Numeric variance
    NumericVariance,
    /// Custom masking function
    Custom(String),
}

/// Masking configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingConfig {
    /// Static replacement value
    pub replacement_value: Option<String>,
    /// Replacement character for random masking
    pub replacement_char: Option<char>,
    /// Number of characters to preserve at start
    pub preserve_start: Option<usize>,
    /// Number of characters to preserve at end
    pub preserve_end: Option<usize>,
    /// Pattern to maintain (for format preserving)
    pub format_pattern: Option<String>,
    /// Substitution source (dictionary, algorithm)
    pub substitution_source: Option<SubstitutionSource>,
    /// Hash algorithm
    pub hash_algorithm: Option<HashAlgorithm>,
    /// Date shift range in days
    pub date_shift_range: Option<(i32, i32)>,
    /// Numeric variance percentage
    pub numeric_variance_percent: Option<f64>,
    /// Custom function parameters
    pub custom_params: Option<HashMap<String, serde_json::Value>>,
    /// Deterministic masking (same input always produces same output)
    pub deterministic: bool,
    /// Seed for deterministic masking
    pub seed: Option<u64>,
}

/// Substitution sources for realistic fake data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubstitutionSource {
    /// Use predefined dictionary
    Dictionary(String),
    /// Generate using algorithm
    Algorithm(String),
    /// External service
    ExternalService { url: String, params: HashMap<String, String> },
}

/// Hash algorithms for masking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Sha256,
    Sha512,
    Blake3,
    Argon2,
}

/// Conditions for when masking should apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingCondition {
    pub field: String,
    pub operator: ConditionOperator,
    pub value: String,
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    Regex,
    GreaterThan,
    LessThan,
    In,
    NotIn,
}

/// Exemptions from masking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingExemption {
    pub exemption_type: ExemptionType,
    pub exemption_value: String,
    pub reason: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Types of masking exemptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExemptionType {
    /// Specific user ID
    User,
    /// User role
    Role,
    /// IP address range
    IpRange,
    /// Purpose of processing
    Purpose,
    /// Legal basis for processing
    LegalBasis,
    /// Time-based exemption
    TimeWindow,
}

/// Context for masking operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskingContext {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub purpose: Option<String>,
    pub legal_basis: Option<String>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub attributes: HashMap<String, String>,
}

/// Privacy controls for GDPR compliance
pub struct PrivacyControls {
    masking_service: Box<dyn DataMasking>,
    consent_tracker: ConsentTracker,
    data_inventory: DataInventory,
}

/// Consent tracking for data processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentTracker {
    pub user_consents: HashMap<Uuid, Vec<ConsentRecord>>,
}

/// Individual consent record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub purpose: String,
    pub legal_basis: LegalBasis,
    pub consent_given: bool,
    pub consent_date: chrono::DateTime<chrono::Utc>,
    pub expiry_date: Option<chrono::DateTime<chrono::Utc>>,
    pub withdrawal_date: Option<chrono::DateTime<chrono::Utc>>,
    pub data_categories: Vec<String>,
    pub processing_activities: Vec<String>,
    pub third_parties: Vec<String>,
    pub retention_period: Option<chrono::Duration>,
}

/// Legal basis for data processing (GDPR Article 6)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegalBasis {
    Consent,
    Contract,
    LegalObligation,
    VitalInterests,
    PublicTask,
    LegitimateInterests,
}

/// Data inventory for privacy impact assessments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataInventory {
    pub data_categories: HashMap<String, DataCategory>,
    pub processing_activities: HashMap<String, ProcessingActivity>,
}

/// Data category definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sensitivity_level: SensitivityLevel,
    pub regulatory_classification: Vec<String>,
    pub retention_period: Option<chrono::Duration>,
    pub geographic_restrictions: Vec<String>,
}

/// Data sensitivity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensitivityLevel {
    Public,
    Internal,
    Confidential,
    Restricted,
    SpecialCategory, // GDPR special category personal data
}

/// Processing activity record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingActivity {
    pub id: String,
    pub name: String,
    pub purpose: String,
    pub legal_basis: LegalBasis,
    pub data_categories: Vec<String>,
    pub data_subjects: Vec<String>,
    pub recipients: Vec<String>,
    pub international_transfers: Vec<InternationalTransfer>,
    pub retention_period: Option<chrono::Duration>,
    pub security_measures: Vec<String>,
}

/// International data transfer record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternationalTransfer {
    pub recipient_country: String,
    pub adequacy_decision: bool,
    pub appropriate_safeguards: Vec<String>,
    pub derogations: Vec<String>,
}

/// Data masking service implementation
pub struct DataMaskingService {
    pool: sqlx::PgPool,
    policy_cache: std::sync::Arc<std::sync::RwLock<HashMap<String, Vec<MaskingPolicy>>>>,
    substitution_dictionaries: HashMap<String, Vec<String>>,
}

impl DataMaskingService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        let mut service = Self {
            pool,
            policy_cache: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
            substitution_dictionaries: HashMap::new(),
        };

        service.load_substitution_dictionaries();
        service
    }

    /// Load predefined substitution dictionaries
    fn load_substitution_dictionaries(&mut self) {
        // First names
        self.substitution_dictionaries.insert(
            "first_names".to_string(),
            vec![
                "Alex".to_string(), "Jordan".to_string(), "Taylor".to_string(), "Casey".to_string(),
                "Morgan".to_string(), "Riley".to_string(), "Quinn".to_string(), "Blake".to_string(),
                "Avery".to_string(), "Cameron".to_string(), "Dakota".to_string(), "Drew".to_string(),
                "Emery".to_string(), "Finley".to_string(), "Grey".to_string(), "Harper".to_string(),
            ],
        );

        // Last names
        self.substitution_dictionaries.insert(
            "last_names".to_string(),
            vec![
                "Smith".to_string(), "Johnson".to_string(), "Williams".to_string(), "Brown".to_string(),
                "Jones".to_string(), "Garcia".to_string(), "Miller".to_string(), "Davis".to_string(),
                "Rodriguez".to_string(), "Martinez".to_string(), "Hernandez".to_string(), "Lopez".to_string(),
                "Gonzalez".to_string(), "Wilson".to_string(), "Anderson".to_string(), "Thomas".to_string(),
            ],
        );

        // Company names
        self.substitution_dictionaries.insert(
            "company_names".to_string(),
            vec![
                "Acme Corp".to_string(), "Global Industries".to_string(), "Tech Solutions".to_string(),
                "Business Systems".to_string(), "Enterprise Services".to_string(), "Innovation Labs".to_string(),
                "Digital Dynamics".to_string(), "Future Systems".to_string(), "Advanced Analytics".to_string(),
                "Smart Solutions".to_string(), "Data Insights".to_string(), "Cloud Computing".to_string(),
            ],
        );

        // Cities
        self.substitution_dictionaries.insert(
            "cities".to_string(),
            vec![
                "Springfield".to_string(), "Franklin".to_string(), "Riverside".to_string(),
                "Madison".to_string(), "Georgetown".to_string(), "Salem".to_string(),
                "Fairview".to_string(), "Bristol".to_string(), "Clinton".to_string(),
                "Manchester".to_string(), "Ashland".to_string(), "Burlington".to_string(),
            ],
        );
    }

    /// Apply specific masking type to value
    fn apply_masking_type(
        &self,
        value: &str,
        masking_type: &MaskingType,
        config: &MaskingConfig,
    ) -> Result<String> {
        match masking_type {
            MaskingType::StaticReplacement => {
                Ok(config.replacement_value.clone().unwrap_or_else(|| "[REDACTED]".to_string()))
            }
            MaskingType::RandomReplacement => {
                let replacement_char = config.replacement_char.unwrap_or('*');
                Ok(replacement_char.to_string().repeat(value.len()))
            }
            MaskingType::PartialMasking => {
                self.apply_partial_masking(value, config)
            }
            MaskingType::FormatPreservingEncryption => {
                self.apply_format_preserving_encryption(value, config)
            }
            MaskingType::Shuffling => {
                self.apply_shuffling(value, config)
            }
            MaskingType::Substitution => {
                self.apply_substitution(value, config)
            }
            MaskingType::Redaction => {
                Ok("[REDACTED]".to_string())
            }
            MaskingType::Hashing => {
                self.apply_hashing(value, config)
            }
            MaskingType::Tokenization => {
                self.apply_tokenization(value, config)
            }
            MaskingType::DateShifting => {
                self.apply_date_shifting(value, config)
            }
            MaskingType::NumericVariance => {
                self.apply_numeric_variance(value, config)
            }
            MaskingType::Custom(function_name) => {
                self.apply_custom_masking(value, function_name, config)
            }
        }
    }

    /// Apply partial masking (show first/last N characters)
    fn apply_partial_masking(&self, value: &str, config: &MaskingConfig) -> Result<String> {
        let preserve_start = config.preserve_start.unwrap_or(0);
        let preserve_end = config.preserve_end.unwrap_or(0);
        let replacement_char = config.replacement_char.unwrap_or('*');

        if value.len() <= preserve_start + preserve_end {
            return Ok(replacement_char.to_string().repeat(value.len()));
        }

        let start_part = &value[..preserve_start];
        let end_part = &value[value.len() - preserve_end..];
        let middle_length = value.len() - preserve_start - preserve_end;
        let middle_part = replacement_char.to_string().repeat(middle_length);

        Ok(format!("{}{}{}", start_part, middle_part, end_part))
    }

    /// Apply format preserving encryption
    fn apply_format_preserving_encryption(&self, value: &str, _config: &MaskingConfig) -> Result<String> {
        // Simplified FPE - maintains character types but changes values
        let mut result = String::new();
        let mut rng = rand::thread_rng();

        for char in value.chars() {
            let masked_char = if char.is_alphabetic() {
                if char.is_uppercase() {
                    char::from(rng.gen_range(b'A'..=b'Z'))
                } else {
                    char::from(rng.gen_range(b'a'..=b'z'))
                }
            } else if char.is_numeric() {
                char::from(rng.gen_range(b'0'..=b'9'))
            } else {
                char // Keep special characters as-is
            };
            result.push(masked_char);
        }

        Ok(result)
    }

    /// Apply shuffling within the value
    fn apply_shuffling(&self, value: &str, _config: &MaskingConfig) -> Result<String> {
        let mut chars: Vec<char> = value.chars().collect();
        let mut rng = rand::thread_rng();

        // Simple shuffle
        for i in (1..chars.len()).rev() {
            let j = rng.gen_range(0..=i);
            chars.swap(i, j);
        }

        Ok(chars.into_iter().collect())
    }

    /// Apply substitution with realistic fake data
    fn apply_substitution(&self, value: &str, config: &MaskingConfig) -> Result<String> {
        if let Some(SubstitutionSource::Dictionary(dict_name)) = &config.substitution_source {
            if let Some(dictionary) = self.substitution_dictionaries.get(dict_name) {
                let mut rng = rand::thread_rng();
                let random_value = &dictionary[rng.gen_range(0..dictionary.len())];
                return Ok(random_value.clone());
            }
        }

        // Fallback to generic substitution
        match value.len() {
            1..=5 => Ok("XXXXX".to_string()),
            6..=10 => Ok("XXXXXXXXXX".to_string()),
            _ => Ok("X".repeat(value.len())),
        }
    }

    /// Apply hashing
    fn apply_hashing(&self, value: &str, config: &MaskingConfig) -> Result<String> {
        use sha2::{Sha256, Digest};

        let algorithm = config.hash_algorithm.as_ref().unwrap_or(&HashAlgorithm::Sha256);

        match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(value.as_bytes());
                Ok(format!("{:x}", hasher.finalize()))
            }
            _ => {
                // Fallback to SHA256 for now
                let mut hasher = Sha256::new();
                hasher.update(value.as_bytes());
                Ok(format!("{:x}", hasher.finalize()))
            }
        }
    }

    /// Apply tokenization
    fn apply_tokenization(&self, value: &str, config: &MaskingConfig) -> Result<String> {
        // Generate a deterministic token if deterministic mode is enabled
        if config.deterministic {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(value.as_bytes());
            if let Some(seed) = config.seed {
                hasher.update(seed.to_le_bytes());
            }
            let hash = hasher.finalize();
            Ok(format!("TOKEN_{:x}", &hash[..8].iter().fold(0u64, |acc, &b| (acc << 8) | b as u64)))
        } else {
            Ok(format!("TOKEN_{}", Uuid::new_v4().to_string().replace('-', "")[..16].to_uppercase()))
        }
    }

    /// Apply date shifting
    fn apply_date_shifting(&self, value: &str, config: &MaskingConfig) -> Result<String> {
        if let Ok(date) = chrono::DateTime::parse_from_rfc3339(value) {
            let (min_shift, max_shift) = config.date_shift_range.unwrap_or((-30, 30));
            let mut rng = rand::thread_rng();
            let shift_days = rng.gen_range(min_shift..=max_shift);
            let shifted_date = date + chrono::Duration::days(shift_days as i64);
            Ok(shifted_date.to_rfc3339())
        } else {
            // Try parsing as date only
            if let Ok(date) = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                let (min_shift, max_shift) = config.date_shift_range.unwrap_or((-30, 30));
                let mut rng = rand::thread_rng();
                let shift_days = rng.gen_range(min_shift..=max_shift);
                let shifted_date = date + chrono::Duration::days(shift_days as i64);
                Ok(shifted_date.format("%Y-%m-%d").to_string())
            } else {
                Ok(value.to_string()) // Return unchanged if not a valid date
            }
        }
    }

    /// Apply numeric variance
    fn apply_numeric_variance(&self, value: &str, config: &MaskingConfig) -> Result<String> {
        if let Ok(num) = value.parse::<f64>() {
            let variance_percent = config.numeric_variance_percent.unwrap_or(10.0);
            let mut rng = rand::thread_rng();
            let variance_factor = 1.0 + (rng.gen::<f64>() - 0.5) * 2.0 * (variance_percent / 100.0);
            let varied_num = num * variance_factor;

            // Preserve integer format if original was integer
            if value.contains('.') {
                Ok(format!("{:.2}", varied_num))
            } else {
                Ok(format!("{}", varied_num.round() as i64))
            }
        } else {
            Ok(value.to_string()) // Return unchanged if not a valid number
        }
    }

    /// Apply custom masking function
    fn apply_custom_masking(&self, value: &str, function_name: &str, _config: &MaskingConfig) -> Result<String> {
        match function_name {
            "email_domain_mask" => {
                if let Some(at_pos) = value.find('@') {
                    let (local, _domain) = value.split_at(at_pos);
                    Ok(format!("{}@example.com", local))
                } else {
                    Ok(value.to_string())
                }
            }
            "phone_partial_mask" => {
                let cleaned = value.chars().filter(|c| c.is_numeric()).collect::<String>();
                if cleaned.len() >= 10 {
                    Ok(format!("***-***-{}", &cleaned[cleaned.len()-4..]))
                } else {
                    Ok("***-***-****".to_string())
                }
            }
            "credit_card_mask" => {
                let cleaned = value.chars().filter(|c| c.is_numeric()).collect::<String>();
                if cleaned.len() >= 12 {
                    Ok(format!("****-****-****-{}", &cleaned[cleaned.len()-4..]))
                } else {
                    Ok("****-****-****-****".to_string())
                }
            }
            _ => Ok(format!("[CUSTOM_MASKED:{}]", function_name)),
        }
    }

    /// Check if masking conditions are met
    fn evaluate_conditions(&self, conditions: &[MaskingCondition], data: &HashMap<String, String>) -> bool {
        for condition in conditions {
            if let Some(field_value) = data.get(&condition.field) {
                let matches = match &condition.operator {
                    ConditionOperator::Equals => field_value == &condition.value,
                    ConditionOperator::NotEquals => field_value != &condition.value,
                    ConditionOperator::Contains => field_value.contains(&condition.value),
                    ConditionOperator::StartsWith => field_value.starts_with(&condition.value),
                    ConditionOperator::EndsWith => field_value.ends_with(&condition.value),
                    ConditionOperator::Regex => {
                        if let Ok(regex) = Regex::new(&condition.value) {
                            regex.is_match(field_value)
                        } else {
                            false
                        }
                    }
                    ConditionOperator::GreaterThan => {
                        if let (Ok(field_num), Ok(condition_num)) = (field_value.parse::<f64>(), condition.value.parse::<f64>()) {
                            field_num > condition_num
                        } else {
                            false
                        }
                    }
                    ConditionOperator::LessThan => {
                        if let (Ok(field_num), Ok(condition_num)) = (field_value.parse::<f64>(), condition.value.parse::<f64>()) {
                            field_num < condition_num
                        } else {
                            false
                        }
                    }
                    ConditionOperator::In => {
                        let values: Vec<&str> = condition.value.split(',').map(|s| s.trim()).collect();
                        values.contains(&field_value.as_str())
                    }
                    ConditionOperator::NotIn => {
                        let values: Vec<&str> = condition.value.split(',').map(|s| s.trim()).collect();
                        !values.contains(&field_value.as_str())
                    }
                };

                if !matches {
                    return false;
                }
            } else {
                return false; // Field not found
            }
        }

        true
    }

    /// Check if user has exemption from masking
    fn check_exemptions(&self, exemptions: &[MaskingExemption], context: &MaskingContext) -> bool {
        for exemption in exemptions {
            // Check if exemption has expired
            if let Some(expires_at) = exemption.expires_at {
                if context.timestamp > expires_at {
                    continue;
                }
            }

            let matches = match exemption.exemption_type {
                ExemptionType::User => context.user_id.to_string() == exemption.exemption_value,
                ExemptionType::Role => {
                    // Would check user roles here
                    false // Placeholder
                }
                ExemptionType::IpRange => {
                    // Would check IP address range here
                    false // Placeholder
                }
                ExemptionType::Purpose => {
                    context.purpose.as_ref().map_or(false, |p| p == &exemption.exemption_value)
                }
                ExemptionType::LegalBasis => {
                    context.legal_basis.as_ref().map_or(false, |l| l == &exemption.exemption_value)
                }
                ExemptionType::TimeWindow => {
                    // Would check if current time is within allowed window
                    false // Placeholder
                }
            };

            if matches {
                return true;
            }
        }

        false
    }
}

#[async_trait]
impl DataMasking for DataMaskingService {
    async fn mask_field(
        &self,
        value: &str,
        policy: &MaskingPolicy,
        context: &MaskingContext,
    ) -> Result<String> {
        // Check if user has exemption
        if let Some(exemptions) = &policy.exemptions {
            if self.check_exemptions(exemptions, context) {
                return Ok(value.to_string()); // Return unmasked
            }
        }

        // Apply masking
        self.apply_masking_type(value, &policy.masking_type, &policy.masking_config)
    }

    async fn mask_fields(
        &self,
        data: &HashMap<String, String>,
        policies: &HashMap<String, MaskingPolicy>,
        context: &MaskingContext,
    ) -> Result<HashMap<String, String>> {
        let mut masked_data = data.clone();

        for (field_name, policy) in policies {
            if let Some(value) = data.get(field_name) {
                // Check conditions
                if let Some(conditions) = &policy.conditions {
                    if !self.evaluate_conditions(conditions, data) {
                        continue; // Skip masking if conditions not met
                    }
                }

                let masked_value = self.mask_field(value, policy, context).await?;
                masked_data.insert(field_name.clone(), masked_value);
            }
        }

        Ok(masked_data)
    }

    async fn mask_structured_data(
        &self,
        data: &serde_json::Value,
        policies: &HashMap<String, MaskingPolicy>,
        context: &MaskingContext,
    ) -> Result<serde_json::Value> {
        let mut masked_data = data.clone();

        if let serde_json::Value::Object(ref mut obj) = masked_data {
            for (field_name, policy) in policies {
                if let Some(field_value) = obj.get(field_name) {
                    if let Some(string_value) = field_value.as_str() {
                        let masked_value = self.mask_field(string_value, policy, context).await?;
                        obj.insert(field_name.clone(), serde_json::Value::String(masked_value));
                    }
                }
            }
        }

        Ok(masked_data)
    }

    async fn can_view_unmasked(
        &self,
        _user_id: Uuid,
        field: &str,
        context: &MaskingContext,
    ) -> Result<bool> {
        // Load policies for the field
        let policies = sqlx::query(
            r#"
            SELECT exemptions FROM data_masking_policies
            WHERE column_name = $1 AND tenant_id = $2 AND is_active = true
            "#
        )
        .bind(field)
        .bind(context.tenant_id)
        .fetch_all(&self.pool)
        .await?;

        for policy_record in policies {
            if let Ok(exemptions_json) = policy_record.try_get::<serde_json::Value, _>("exemptions") {
                if let Ok(exemptions) = serde_json::from_value::<Vec<MaskingExemption>>(exemptions_json) {
                    if self.check_exemptions(&exemptions, context) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    async fn create_policy(&self, _policy: &MaskingPolicy) -> Result<Uuid> {
        let policy_id = Uuid::new_v4();

        // TODO: Re-enable once sqlx query cache is fixed
        /*
        sqlx::query!(
            r#"
            INSERT INTO data_masking_policies (
                id, name, description, table_name, column_name, masking_type, masking_config,
                conditions, is_active, tenant_id, created_by, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
            "#,
            policy_id,
            policy.name,
            policy.description,
            policy.table_name,
            policy.column_name,
            serde_json::to_string(&policy.masking_type).unwrap(),
            serde_json::to_value(&policy.masking_config).unwrap(),
            policy.conditions.as_ref().map(|c| serde_json::to_value(c).unwrap()),
            policy.is_active,
            policy.tenant_id,
            policy.created_by
        )
        .execute(&self.pool)
        .await?;
        */

        Ok(policy_id)
    }

    async fn update_policy(&self, _policy: &MaskingPolicy) -> Result<()> {
        // TODO: Re-enable once sqlx query cache is fixed
        /*
        sqlx::query!(
            r#"
            UPDATE data_masking_policies SET
                name = $2,
                description = $3,
                masking_type = $4,
                masking_config = $5,
                conditions = $6,
                is_active = $7,
                modified_by = $8,
                modified_at = NOW()
            WHERE id = $1
            "#,
            policy.id,
            policy.name,
            policy.description,
            serde_json::to_string(&policy.masking_type).unwrap(),
            serde_json::to_value(&policy.masking_config).unwrap(),
            policy.conditions.as_ref().map(|c| serde_json::to_value(c).unwrap()),
            policy.is_active,
            policy.modified_by
        )
        .execute(&self.pool)
        .await?;
        */

        // Clear cache
        {
            let mut cache = self.policy_cache.write().unwrap();
            // TODO: Fix when policy parameter is re-enabled
            // cache.remove(&policy.table_name);
            cache.clear(); // Temporary: clear all cache entries
        }

        Ok(())
    }

    async fn get_table_policies(&self, table_name: &str, tenant_id: Uuid) -> Result<Vec<MaskingPolicy>> {
        // Check cache first
        {
            let cache = self.policy_cache.read().unwrap();
            if let Some(cached_policies) = cache.get(table_name) {
                return Ok(cached_policies.clone());
            }
        }

        // TODO: Re-enable once sqlx query cache is fixed
        /*
        let policy_records = sqlx::query!(
            r#"
            SELECT id, name, description, column_name, masking_type, masking_config,
                   conditions, is_active, created_by, created_at, modified_by, modified_at
            FROM data_masking_policies
            WHERE table_name = $1 AND tenant_id = $2 AND is_active = true
            "#,
            table_name,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;
        */
        #[derive(Debug)]
        struct PolicyRecord {
            id: Uuid,
            name: String,
            description: Option<String>,
            column_name: String,
            masking_type: String,
            masking_config: serde_json::Value,
            conditions: Option<serde_json::Value>,
            is_active: bool,
            created_by: Uuid,
            created_at: chrono::DateTime<chrono::Utc>,
            modified_by: Option<Uuid>,
            modified_at: Option<chrono::DateTime<chrono::Utc>>,
        }
        let policy_records: Vec<PolicyRecord> = vec![]; // Temporary placeholder

        let mut policies = Vec::new();
        for record in policy_records {
            let policy = MaskingPolicy {
                id: record.id,
                name: record.name,
                description: record.description.unwrap_or_default(),
                table_name: table_name.to_string(),
                column_name: record.column_name,
                masking_type: serde_json::from_str(&record.masking_type).unwrap_or(MaskingType::Redaction),
                masking_config: serde_json::from_value(record.masking_config).unwrap_or_default(),
                conditions: record.conditions.and_then(|c| serde_json::from_value(c).ok()),
                exemptions: None,
                is_active: record.is_active,
                tenant_id,
                created_by: record.created_by,
                created_at: record.created_at,
                modified_by: record.modified_by.unwrap_or(record.created_by),
                modified_at: record.modified_at.unwrap_or(record.created_at),
            };
            policies.push(policy);
        }

        // Cache the policies
        {
            let mut cache = self.policy_cache.write().unwrap();
            cache.insert(table_name.to_string(), policies.clone());
        }

        Ok(policies)
    }
}

impl Default for MaskingConfig {
    fn default() -> Self {
        Self {
            replacement_value: None,
            replacement_char: Some('*'),
            preserve_start: None,
            preserve_end: None,
            format_pattern: None,
            substitution_source: None,
            hash_algorithm: None,
            date_shift_range: None,
            numeric_variance_percent: None,
            custom_params: None,
            deterministic: false,
            seed: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_partial_masking() {
        let service = DataMaskingService::new(sqlx::Pool::connect("").await.unwrap());

        let config = MaskingConfig {
            preserve_start: Some(2),
            preserve_end: Some(2),
            replacement_char: Some('*'),
            ..Default::default()
        };

        let result = service.apply_partial_masking("test@example.com", &config).unwrap();
        assert_eq!(result, "te***********om");
    }

    #[tokio::test]
    async fn test_email_custom_masking() {
        let service = DataMaskingService::new(sqlx::Pool::connect("").await.unwrap());

        let config = MaskingConfig::default();

        let result = service.apply_custom_masking("user@company.com", "email_domain_mask", &config).unwrap();
        assert_eq!(result, "user@example.com");
    }

    #[tokio::test]
    async fn test_condition_evaluation() {
        let service = DataMaskingService::new(sqlx::Pool::connect("").await.unwrap());

        let conditions = vec![
            MaskingCondition {
                field: "user_role".to_string(),
                operator: ConditionOperator::Equals,
                value: "customer_service".to_string(),
            }
        ];

        let mut data = HashMap::new();
        data.insert("user_role".to_string(), "customer_service".to_string());

        let result = service.evaluate_conditions(&conditions, &data);
        assert!(result);

        data.insert("user_role".to_string(), "admin".to_string());
        let result = service.evaluate_conditions(&conditions, &data);
        assert!(!result);
    }
}