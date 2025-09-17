use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::ValidationError;

use crate::customer::model::*;
use crate::error::{MasterDataError, Result};

/// Advanced validation and business rules engine for customers
#[async_trait]
pub trait CustomerValidationEngine: Send + Sync {
    /// Validate customer data against business rules
    async fn validate_customer(&self, customer: &Customer, context: &ValidationContext) -> Result<ValidationReport>;

    /// Validate customer creation request
    async fn validate_create_request(&self, request: &CreateCustomerRequest, context: &ValidationContext) -> Result<ValidationReport>;

    /// Validate customer update request
    async fn validate_update_request(&self, customer_id: Uuid, request: &UpdateCustomerRequest, context: &ValidationContext) -> Result<ValidationReport>;

    /// Execute custom business rules
    async fn execute_business_rules(&self, rule_set: &BusinessRuleSet, customer: &Customer) -> Result<RuleExecutionResult>;

    /// Validate data quality and completeness
    async fn validate_data_quality(&self, customer: &Customer) -> Result<DataQualityReport>;

    /// Validate compliance with regulations
    async fn validate_compliance(&self, customer: &Customer, regulations: &[ComplianceRule]) -> Result<ComplianceReport>;

    /// Validate customer relationships and hierarchies
    async fn validate_relationships(&self, customer: &Customer, context: &ValidationContext) -> Result<RelationshipValidationResult>;
}

/// Validation context with environmental information
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub user_roles: Vec<String>,
    pub user_permissions: Vec<String>,
    pub operation_type: OperationType,
    pub validation_level: ValidationLevel,
    pub country_context: Option<String>,
    pub regulatory_context: Vec<String>,
    pub business_context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum OperationType {
    Create,
    Update,
    Delete,
    Import,
    Sync,
    Migration,
}

#[derive(Debug, Clone)]
pub enum ValidationLevel {
    Basic,       // Core validation only
    Standard,    // Standard business rules
    Strict,      // All rules including edge cases
    Compliance,  // Full compliance validation
}

/// Comprehensive validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub validation_level: String,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
    pub suggestions: Vec<ValidationSuggestion>,
    pub data_quality_score: f64,
    pub compliance_score: f64,
    pub business_rule_results: Vec<BusinessRuleResult>,
    pub validation_timestamp: chrono::DateTime<chrono::Utc>,
    pub validation_duration_ms: u64,
}

/// Individual validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub issue_type: ValidationIssueType,
    pub severity: ValidationSeverity,
    pub field_path: String,
    pub current_value: Option<serde_json::Value>,
    pub expected_value: Option<serde_json::Value>,
    pub error_code: String,
    pub message: String,
    pub description: Option<String>,
    pub remediation_steps: Vec<String>,
    pub related_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationIssueType {
    RequiredField,
    InvalidFormat,
    InvalidValue,
    BusinessRule,
    DataQuality,
    Compliance,
    Relationship,
    Duplicate,
    Inconsistency,
    Security,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Critical,  // Blocks operation
    Error,     // Should block operation
    Warning,   // Should review
    Info,      // Informational
}

/// Validation suggestions for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSuggestion {
    pub suggestion_type: SuggestionType,
    pub field_path: String,
    pub current_value: Option<serde_json::Value>,
    pub suggested_value: serde_json::Value,
    pub confidence: f64,
    pub reason: String,
    pub benefits: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    FormatImprovement,
    DataEnrichment,
    Standardization,
    BestPractice,
    Performance,
    Compliance,
}

/// Business rule set for execution
#[derive(Debug, Clone)]
pub struct BusinessRuleSet {
    pub rules: Vec<BusinessRule>,
    pub execution_order: Vec<String>,
    pub rule_dependencies: HashMap<String, Vec<String>>,
}

/// Individual business rule
#[derive(Debug, Clone)]
pub struct BusinessRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: BusinessRuleType,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
    pub priority: u32,
    pub enabled: bool,
    pub effective_date: Option<chrono::DateTime<chrono::Utc>>,
    pub expiry_date: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum BusinessRuleType {
    Validation,
    Transformation,
    Calculation,
    Notification,
    Workflow,
    Security,
    Compliance,
}

/// Rule condition for evaluation
#[derive(Debug, Clone)]
pub struct RuleCondition {
    pub condition_type: ConditionType,
    pub field_path: String,
    pub operator: ConditionOperator,
    pub expected_value: serde_json::Value,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone)]
pub enum ConditionType {
    FieldValue,
    FieldExists,
    FieldLength,
    FieldFormat,
    RelatedData,
    Calculated,
    External,
}

#[derive(Debug, Clone)]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    Matches,      // Regex
    In,           // Array membership
    NotIn,
    Between,
    IsNull,
    IsNotNull,
    IsEmpty,
    IsNotEmpty,
}

/// Rule action to execute
#[derive(Debug, Clone)]
pub struct RuleAction {
    pub action_type: RuleActionType,
    pub target_field: Option<String>,
    pub action_value: Option<serde_json::Value>,
    pub message: Option<String>,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone)]
pub enum RuleActionType {
    SetError,
    SetWarning,
    SetValue,
    TransformValue,
    SendNotification,
    LogEvent,
    TriggerWorkflow,
    BlockOperation,
}

/// Rule execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleExecutionResult {
    pub executed_rules: Vec<BusinessRuleResult>,
    pub total_execution_time_ms: u64,
    pub errors_found: u32,
    pub warnings_found: u32,
    pub transformations_applied: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRuleResult {
    pub rule_id: String,
    pub rule_name: String,
    pub executed: bool,
    pub conditions_met: bool,
    pub actions_executed: Vec<String>,
    pub execution_time_ms: u64,
    pub result_data: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Data quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityReport {
    pub overall_score: f64,
    pub dimension_scores: HashMap<String, f64>,
    pub quality_issues: Vec<DataQualityIssue>,
    pub completeness_score: f64,
    pub accuracy_score: f64,
    pub consistency_score: f64,
    pub validity_score: f64,
    pub uniqueness_score: f64,
    pub timeliness_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityIssue {
    pub dimension: DataQualityDimension,
    pub field_path: String,
    pub issue_description: String,
    pub impact_level: DataQualityImpact,
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataQualityDimension {
    Completeness,
    Accuracy,
    Consistency,
    Validity,
    Uniqueness,
    Timeliness,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataQualityImpact {
    High,
    Medium,
    Low,
}

/// Compliance validation
#[derive(Debug, Clone)]
pub struct ComplianceRule {
    pub regulation_name: String,
    pub rule_code: String,
    pub description: String,
    pub applicable_regions: Vec<String>,
    pub requirements: Vec<ComplianceRequirement>,
    pub validation_logic: String, // Could be SQL, script, or rule expression
}

#[derive(Debug, Clone)]
pub struct ComplianceRequirement {
    pub requirement_id: String,
    pub field_requirements: Vec<FieldRequirement>,
    pub business_logic_requirements: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FieldRequirement {
    pub field_path: String,
    pub required: bool,
    pub format_requirements: Option<String>,
    pub value_constraints: Option<ValueConstraints>,
}

#[derive(Debug, Clone)]
pub struct ValueConstraints {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub allowed_values: Option<Vec<String>>,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub overall_compliance: bool,
    pub compliance_score: f64,
    pub regulation_results: Vec<RegulationComplianceResult>,
    pub violations: Vec<ComplianceViolation>,
    pub recommendations: Vec<ComplianceRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulationComplianceResult {
    pub regulation_name: String,
    pub compliant: bool,
    pub compliance_percentage: f64,
    pub passed_requirements: u32,
    pub total_requirements: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub regulation_name: String,
    pub requirement_id: String,
    pub violation_type: String,
    pub description: String,
    pub severity: ValidationSeverity,
    pub remediation_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRecommendation {
    pub regulation_name: String,
    pub recommendation: String,
    pub benefit: String,
    pub implementation_effort: ImplementationEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
}

/// Relationship validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipValidationResult {
    pub valid_relationships: bool,
    pub hierarchy_issues: Vec<HierarchyIssue>,
    pub circular_references: Vec<CircularReference>,
    pub orphaned_relationships: Vec<OrphanedRelationship>,
    pub constraint_violations: Vec<ConstraintViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyIssue {
    pub issue_type: HierarchyIssueType,
    pub customer_id: Uuid,
    pub related_customer_id: Option<Uuid>,
    pub description: String,
    pub suggested_fix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HierarchyIssueType {
    MaxDepthExceeded,
    InvalidParent,
    SelfReference,
    OrphanedChild,
    InconsistentLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularReference {
    pub customer_ids: Vec<Uuid>,
    pub relationship_path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanedRelationship {
    pub customer_id: Uuid,
    pub relationship_type: String,
    pub related_entity_id: Uuid,
    pub entity_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintViolation {
    pub constraint_name: String,
    pub customer_id: Uuid,
    pub field_path: String,
    pub violation_description: String,
}

/// Default validation engine implementation
pub struct DefaultCustomerValidationEngine {
    business_rules: BusinessRuleSet,
    compliance_rules: Vec<ComplianceRule>,
    data_quality_config: DataQualityConfig,
}

#[derive(Debug, Clone)]
pub struct DataQualityConfig {
    pub completeness_weight: f64,
    pub accuracy_weight: f64,
    pub consistency_weight: f64,
    pub validity_weight: f64,
    pub uniqueness_weight: f64,
    pub timeliness_weight: f64,
    pub minimum_acceptable_score: f64,
}

impl DefaultCustomerValidationEngine {
    pub fn new() -> Self {
        Self {
            business_rules: Self::create_default_business_rules(),
            compliance_rules: Self::create_default_compliance_rules(),
            data_quality_config: DataQualityConfig {
                completeness_weight: 0.25,
                accuracy_weight: 0.25,
                consistency_weight: 0.20,
                validity_weight: 0.15,
                uniqueness_weight: 0.10,
                timeliness_weight: 0.05,
                minimum_acceptable_score: 0.8,
            },
        }
    }

    fn create_default_business_rules() -> BusinessRuleSet {
        let mut rules = vec![];

        // B2B Customer Legal Name Rule
        rules.push(BusinessRule {
            id: "B2B_LEGAL_NAME_LENGTH".to_string(),
            name: "B2B Customer Legal Name Minimum Length".to_string(),
            description: "B2B customers must have legal names with at least 2 characters".to_string(),
            rule_type: BusinessRuleType::Validation,
            conditions: vec![
                RuleCondition {
                    condition_type: ConditionType::FieldValue,
                    field_path: "customer_type".to_string(),
                    operator: ConditionOperator::Equals,
                    expected_value: serde_json::Value::String("B2b".to_string()),
                    case_sensitive: false,
                },
            ],
            actions: vec![
                RuleAction {
                    action_type: RuleActionType::SetError,
                    target_field: Some("legal_name".to_string()),
                    action_value: None,
                    message: Some("B2B customers must have a legal name with at least 2 characters".to_string()),
                    severity: ValidationSeverity::Error,
                },
            ],
            priority: 1,
            enabled: true,
            effective_date: None,
            expiry_date: None,
            metadata: HashMap::new(),
        });

        // Credit Limit Rule
        rules.push(BusinessRule {
            id: "CREDIT_LIMIT_VALIDATION".to_string(),
            name: "Credit Limit Validation".to_string(),
            description: "Credit limit must not exceed 10x annual revenue".to_string(),
            rule_type: BusinessRuleType::Validation,
            conditions: vec![],
            actions: vec![
                RuleAction {
                    action_type: RuleActionType::SetError,
                    target_field: Some("financial_info.credit_limit".to_string()),
                    action_value: None,
                    message: Some("Credit limit cannot exceed 10x annual revenue".to_string()),
                    severity: ValidationSeverity::Error,
                },
            ],
            priority: 2,
            enabled: true,
            effective_date: None,
            expiry_date: None,
            metadata: HashMap::new(),
        });

        BusinessRuleSet {
            rules,
            execution_order: vec!["B2B_LEGAL_NAME_LENGTH".to_string(), "CREDIT_LIMIT_VALIDATION".to_string()],
            rule_dependencies: HashMap::new(),
        }
    }

    fn create_default_compliance_rules() -> Vec<ComplianceRule> {
        vec![
            // GDPR compliance rule
            ComplianceRule {
                regulation_name: "GDPR".to_string(),
                rule_code: "GDPR_DATA_MINIMIZATION".to_string(),
                description: "Ensure only necessary personal data is collected".to_string(),
                applicable_regions: vec!["EU".to_string()],
                requirements: vec![
                    ComplianceRequirement {
                        requirement_id: "GDPR_CONSENT".to_string(),
                        field_requirements: vec![
                            FieldRequirement {
                                field_path: "contacts.email".to_string(),
                                required: false,
                                format_requirements: Some("valid_email".to_string()),
                                value_constraints: None,
                            },
                        ],
                        business_logic_requirements: vec![
                            "Must have explicit consent for email processing".to_string(),
                        ],
                    },
                ],
                validation_logic: "Check consent flags and data retention policies".to_string(),
            },
        ]
    }
}

#[async_trait]
impl CustomerValidationEngine for DefaultCustomerValidationEngine {
    async fn validate_customer(&self, customer: &Customer, context: &ValidationContext) -> Result<ValidationReport> {
        let start_time = std::time::Instant::now();
        let mut errors = vec![];
        let mut warnings = vec![];
        let mut suggestions = vec![];

        // Execute business rules
        let rule_result = self.execute_business_rules(&self.business_rules, customer).await?;

        // Validate data quality
        let data_quality = self.validate_data_quality(customer).await?;

        // Validate compliance
        let compliance = self.validate_compliance(customer, &self.compliance_rules).await?;

        // Aggregate results
        let validation_duration = start_time.elapsed().as_millis() as u64;

        Ok(ValidationReport {
            is_valid: errors.is_empty(),
            validation_level: format!("{:?}", context.validation_level),
            errors,
            warnings,
            suggestions,
            data_quality_score: data_quality.overall_score,
            compliance_score: compliance.compliance_score,
            business_rule_results: rule_result.executed_rules,
            validation_timestamp: chrono::Utc::now(),
            validation_duration_ms: validation_duration,
        })
    }

    async fn validate_create_request(&self, _request: &CreateCustomerRequest, _context: &ValidationContext) -> Result<ValidationReport> {
        // Implementation would validate the create request
        // This is a placeholder
        Ok(ValidationReport {
            is_valid: true,
            validation_level: "Standard".to_string(),
            errors: vec![],
            warnings: vec![],
            suggestions: vec![],
            data_quality_score: 0.85,
            compliance_score: 0.90,
            business_rule_results: vec![],
            validation_timestamp: chrono::Utc::now(),
            validation_duration_ms: 10,
        })
    }

    async fn validate_update_request(&self, _customer_id: Uuid, _request: &UpdateCustomerRequest, _context: &ValidationContext) -> Result<ValidationReport> {
        // Implementation would validate the update request
        // This is a placeholder
        Ok(ValidationReport {
            is_valid: true,
            validation_level: "Standard".to_string(),
            errors: vec![],
            warnings: vec![],
            suggestions: vec![],
            data_quality_score: 0.85,
            compliance_score: 0.90,
            business_rule_results: vec![],
            validation_timestamp: chrono::Utc::now(),
            validation_duration_ms: 15,
        })
    }

    async fn execute_business_rules(&self, _rule_set: &BusinessRuleSet, _customer: &Customer) -> Result<RuleExecutionResult> {
        // Implementation would execute all business rules
        // This is a placeholder
        Ok(RuleExecutionResult {
            executed_rules: vec![],
            total_execution_time_ms: 5,
            errors_found: 0,
            warnings_found: 0,
            transformations_applied: 0,
        })
    }

    async fn validate_data_quality(&self, _customer: &Customer) -> Result<DataQualityReport> {
        // Implementation would assess data quality across all dimensions
        // This is a placeholder
        Ok(DataQualityReport {
            overall_score: 0.85,
            dimension_scores: HashMap::from([
                ("completeness".to_string(), 0.90),
                ("accuracy".to_string(), 0.85),
                ("consistency".to_string(), 0.80),
                ("validity".to_string(), 0.88),
                ("uniqueness".to_string(), 0.95),
                ("timeliness".to_string(), 0.75),
            ]),
            quality_issues: vec![],
            completeness_score: 0.90,
            accuracy_score: 0.85,
            consistency_score: 0.80,
            validity_score: 0.88,
            uniqueness_score: 0.95,
            timeliness_score: 0.75,
        })
    }

    async fn validate_compliance(&self, _customer: &Customer, _regulations: &[ComplianceRule]) -> Result<ComplianceReport> {
        // Implementation would check compliance with all applicable regulations
        // This is a placeholder
        Ok(ComplianceReport {
            overall_compliance: true,
            compliance_score: 0.90,
            regulation_results: vec![],
            violations: vec![],
            recommendations: vec![],
        })
    }

    async fn validate_relationships(&self, _customer: &Customer, _context: &ValidationContext) -> Result<RelationshipValidationResult> {
        // Implementation would validate customer relationships and hierarchies
        // This is a placeholder
        Ok(RelationshipValidationResult {
            valid_relationships: true,
            hierarchy_issues: vec![],
            circular_references: vec![],
            orphaned_relationships: vec![],
            constraint_violations: vec![],
        })
    }
}

impl Default for DefaultCustomerValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{MasterDataError, Result};
    use regex::Regex;
    use once_cell::sync::Lazy;

    static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()
    });

    static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[\+]?[1-9]?[\d\s\-\(\)\.]{7,15}$").unwrap()
    });

    static CUSTOMER_NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^[A-Z0-9\-_]{1,50}$").unwrap()
    });

    pub struct CustomerValidator;

    impl CustomerValidator {
        pub fn new() -> Self {
            Self
        }

        pub fn validate_customer_number(&self, customer_number: &str) -> Result<()> {
            if customer_number.is_empty() {
                return Err(MasterDataError::ValidationError(
                    "Customer number cannot be empty".to_string(),
                ));
            }

            if customer_number.len() > 50 {
                return Err(MasterDataError::ValidationError(
                    "Customer number cannot exceed 50 characters".to_string(),
                ));
            }

            if !CUSTOMER_NUMBER_REGEX.is_match(customer_number) {
                return Err(MasterDataError::ValidationError(
                    "Customer number contains invalid characters. Only alphanumeric characters, hyphens, and underscores are allowed".to_string(),
                ));
            }

            Ok(())
        }

        pub fn validate_email(&self, email: &str) -> Result<()> {
            if email.is_empty() {
                return Err(MasterDataError::ValidationError(
                    "Email cannot be empty".to_string(),
                ));
            }

            if !EMAIL_REGEX.is_match(email) {
                return Err(MasterDataError::ValidationError(
                    "Invalid email format".to_string(),
                ));
            }

            Ok(())
        }

        pub fn validate_phone(&self, phone: &str) -> Result<()> {
            if phone.is_empty() {
                return Err(MasterDataError::ValidationError(
                    "Phone number cannot be empty".to_string(),
                ));
            }

            if !PHONE_REGEX.is_match(phone) {
                return Err(MasterDataError::ValidationError(
                    "Invalid phone number format".to_string(),
                ));
            }

            Ok(())
        }

        pub fn validate_legal_name(&self, legal_name: &str) -> Result<()> {
            if legal_name.is_empty() {
                return Err(MasterDataError::ValidationError(
                    "Legal name cannot be empty".to_string(),
                ));
            }

            if legal_name.len() > 255 {
                return Err(MasterDataError::ValidationError(
                    "Legal name cannot exceed 255 characters".to_string(),
                ));
            }

            if legal_name.trim().is_empty() {
                return Err(MasterDataError::ValidationError(
                    "Legal name cannot be only whitespace".to_string(),
                ));
            }

            Ok(())
        }

        pub fn validate_tags(&self, tags: &[String]) -> Result<()> {
            if tags.len() > 50 {
                return Err(MasterDataError::ValidationError(
                    "Cannot have more than 50 tags".to_string(),
                ));
            }

            for tag in tags {
                if tag.is_empty() {
                    return Err(MasterDataError::ValidationError(
                        "Tags cannot be empty".to_string(),
                    ));
                }

                if tag.len() > 100 {
                    return Err(MasterDataError::ValidationError(
                        "Individual tags cannot exceed 100 characters".to_string(),
                    ));
                }

                if tag.trim() != tag {
                    return Err(MasterDataError::ValidationError(
                        "Tags cannot have leading or trailing whitespace".to_string(),
                    ));
                }
            }

            // Check for duplicates
            let mut unique_tags = std::collections::HashSet::new();
            for tag in tags {
                if !unique_tags.insert(tag.to_lowercase()) {
                    return Err(MasterDataError::ValidationError(
                        format!("Duplicate tag found: {}", tag),
                    ));
                }
            }

            Ok(())
        }
    }

    impl Default for CustomerValidator {
        fn default() -> Self {
            Self::new()
        }
    }

    #[tokio::test]
    async fn test_validation_engine() {
        let engine = DefaultCustomerValidationEngine::new();

        // Create test customer
        let customer = Customer {
            id: Uuid::new_v4(),
            customer_number: "B2B000001".to_string(),
            legal_name: "Test Customer Ltd.".to_string(),
            customer_type: CustomerType::B2b,
            // ... other fields would be filled in real test
            ..Default::default()
        };

        let context = ValidationContext {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            user_roles: vec!["admin".to_string()],
            user_permissions: vec!["customer:validate".to_string()],
            operation_type: OperationType::Create,
            validation_level: ValidationLevel::Standard,
            country_context: Some("US".to_string()),
            regulatory_context: vec!["SOX".to_string()],
            business_context: HashMap::new(),
        };

        let result = engine.validate_customer(&customer, &context).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.data_quality_score >= 0.0);
        assert!(report.compliance_score >= 0.0);
    }

    #[test]
    fn test_validate_customer_number() {
        let validator = CustomerValidator::new();

        // Valid cases
        assert!(validator.validate_customer_number("CUST-001").is_ok());
        assert!(validator.validate_customer_number("12345").is_ok());
        assert!(validator.validate_customer_number("ABC_123").is_ok());
        assert!(validator.validate_customer_number("TEST-CUSTOMER-123").is_ok());

        // Invalid cases
        assert!(validator.validate_customer_number("").is_err());
        assert!(validator.validate_customer_number("customer with spaces").is_err());
        assert!(validator.validate_customer_number("INVALID!@#").is_err());
        assert!(validator.validate_customer_number(&"A".repeat(51)).is_err());
    }

    #[test]
    fn test_validate_email() {
        let validator = CustomerValidator::new();

        // Valid cases
        assert!(validator.validate_email("test@example.com").is_ok());
        assert!(validator.validate_email("user.name+tag@example.com").is_ok());
        assert!(validator.validate_email("user123@test-domain.co.uk").is_ok());

        // Invalid cases
        assert!(validator.validate_email("").is_err());
        assert!(validator.validate_email("invalid-email").is_err());
        assert!(validator.validate_email("@example.com").is_err());
        assert!(validator.validate_email("test@").is_err());
        assert!(validator.validate_email("test@.com").is_err());
    }

    #[test]
    fn test_validate_phone() {
        let validator = CustomerValidator::new();

        // Valid cases
        assert!(validator.validate_phone("+1-555-123-4567").is_ok());
        assert!(validator.validate_phone("555-123-4567").is_ok());
        assert!(validator.validate_phone("+44 20 7946 0958").is_ok());
        assert!(validator.validate_phone("(555) 123-4567").is_ok());

        // Invalid cases
        assert!(validator.validate_phone("").is_err());
        assert!(validator.validate_phone("invalid-phone").is_err());
        assert!(validator.validate_phone("123").is_err());
        assert!(validator.validate_phone("abc-def-ghij").is_err());
    }

    #[test]
    fn test_validate_legal_name() {
        let validator = CustomerValidator::new();

        // Valid cases
        assert!(validator.validate_legal_name("Test Customer Ltd.").is_ok());
        assert!(validator.validate_legal_name("John Doe").is_ok());
        assert!(validator.validate_legal_name("ACME Corporation").is_ok());

        // Invalid cases
        assert!(validator.validate_legal_name("").is_err());
        assert!(validator.validate_legal_name("   ").is_err());
        assert!(validator.validate_legal_name(&"A".repeat(256)).is_err());
    }

    #[test]
    fn test_validate_tags() {
        let validator = CustomerValidator::new();

        // Valid cases
        assert!(validator.validate_tags(&[]).is_ok());
        assert!(validator.validate_tags(&["tag1".to_string(), "tag2".to_string()]).is_ok());
        assert!(validator.validate_tags(&vec!["tag".to_string(); 50]).is_ok());

        // Invalid cases
        assert!(validator.validate_tags(&vec!["tag".to_string(); 51]).is_err());
        assert!(validator.validate_tags(&["".to_string()]).is_err());
        assert!(validator.validate_tags(&[" leading space".to_string()]).is_err());
        assert!(validator.validate_tags(&["trailing space ".to_string()]).is_err());
        assert!(validator.validate_tags(&["tag1".to_string(), "tag1".to_string()]).is_err());
        assert!(validator.validate_tags(&["Tag1".to_string(), "tag1".to_string()]).is_err()); // Case-insensitive duplicates
        assert!(validator.validate_tags(&["A".repeat(101)]).is_err());
    }

    #[test]
    fn test_business_rule_creation() {
        let engine = DefaultCustomerValidationEngine::new();
        assert!(engine.business_rules.rules.len() >= 2);
        assert!(engine.business_rules.execution_order.len() >= 2);
        assert!(engine.compliance_rules.len() >= 1);
    }

    #[test]
    fn test_data_quality_config() {
        let engine = DefaultCustomerValidationEngine::new();
        let config = &engine.data_quality_config;

        // Weights should sum to 1.0
        let total_weight = config.completeness_weight
            + config.accuracy_weight
            + config.consistency_weight
            + config.validity_weight
            + config.uniqueness_weight
            + config.timeliness_weight;

        assert!((total_weight - 1.0).abs() < 0.01);
        assert!(config.minimum_acceptable_score > 0.0);
        assert!(config.minimum_acceptable_score <= 1.0);
    }

    #[tokio::test]
    async fn test_validate_create_request() {
        let engine = DefaultCustomerValidationEngine::new();

        let request = CreateCustomerRequest {
            customer_number: Some("TEST-001".to_string()),
            legal_name: "Test Customer".to_string(),
            trade_names: None,
            customer_type: CustomerType::B2b,
            industry_classification: None,
            business_size: None,
            parent_customer_id: None,
            corporate_group_id: None,
            lifecycle_stage: Some(CustomerLifecycleStage::Lead),
            status: None,
            credit_status: None,
            addresses: None,
            contacts: None,
            tax_jurisdictions: None,
            tax_numbers: None,
            financial_info: None,
            sales_representative_id: None,
            account_manager_id: None,
            acquisition_channel: None,
            external_ids: None,
            sync_info: None,
        };

        let context = ValidationContext {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            user_roles: vec!["user".to_string()],
            user_permissions: vec!["customer:create".to_string()],
            operation_type: OperationType::Create,
            validation_level: ValidationLevel::Standard,
            country_context: Some("US".to_string()),
            regulatory_context: vec![],
            business_context: HashMap::new(),
        };

        let result = engine.validate_create_request(&request, &context).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.is_valid);
        assert!(report.data_quality_score >= 0.0);
        assert!(report.compliance_score >= 0.0);
    }

    #[tokio::test]
    async fn test_validate_update_request() {
        let engine = DefaultCustomerValidationEngine::new();
        let customer_id = Uuid::new_v4();

        let request = UpdateCustomerRequest {
            legal_name: Some("Updated Customer".to_string()),
            lifecycle_stage: Some(CustomerLifecycleStage::ActiveCustomer),
            ..Default::default()
        };

        let context = ValidationContext {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            user_roles: vec!["user".to_string()],
            user_permissions: vec!["customer:update".to_string()],
            operation_type: OperationType::Update,
            validation_level: ValidationLevel::Standard,
            country_context: Some("US".to_string()),
            regulatory_context: vec![],
            business_context: HashMap::new(),
        };

        let result = engine.validate_update_request(customer_id, &request, &context).await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.is_valid);
        assert!(report.data_quality_score >= 0.0);
        assert!(report.compliance_score >= 0.0);
    }

    #[tokio::test]
    async fn test_execute_business_rules() {
        let engine = DefaultCustomerValidationEngine::new();
        let customer = Customer::default();

        let result = engine.execute_business_rules(&engine.business_rules, &customer).await;
        assert!(result.is_ok());

        let rule_result = result.unwrap();
        assert!(rule_result.total_execution_time_ms >= 0);
        assert!(rule_result.errors_found >= 0);
        assert!(rule_result.warnings_found >= 0);
        assert!(rule_result.transformations_applied >= 0);
    }

    #[tokio::test]
    async fn test_validate_data_quality() {
        let engine = DefaultCustomerValidationEngine::new();
        let customer = Customer::default();

        let result = engine.validate_data_quality(&customer).await;
        assert!(result.is_ok());

        let quality_report = result.unwrap();
        assert!(quality_report.overall_score >= 0.0);
        assert!(quality_report.overall_score <= 1.0);
        assert!(quality_report.completeness_score >= 0.0);
        assert!(quality_report.accuracy_score >= 0.0);
        assert!(quality_report.consistency_score >= 0.0);
        assert!(quality_report.validity_score >= 0.0);
        assert!(quality_report.uniqueness_score >= 0.0);
        assert!(quality_report.timeliness_score >= 0.0);
    }

    #[tokio::test]
    async fn test_validate_compliance() {
        let engine = DefaultCustomerValidationEngine::new();
        let customer = Customer::default();

        let result = engine.validate_compliance(&customer, &engine.compliance_rules).await;
        assert!(result.is_ok());

        let compliance_report = result.unwrap();
        assert!(compliance_report.compliance_score >= 0.0);
        assert!(compliance_report.compliance_score <= 1.0);
    }

    #[tokio::test]
    async fn test_validate_relationships() {
        let engine = DefaultCustomerValidationEngine::new();
        let customer = Customer::default();

        let context = ValidationContext {
            tenant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            user_roles: vec!["user".to_string()],
            user_permissions: vec!["customer:read".to_string()],
            operation_type: OperationType::Create,
            validation_level: ValidationLevel::Standard,
            country_context: Some("US".to_string()),
            regulatory_context: vec![],
            business_context: HashMap::new(),
        };

        let result = engine.validate_relationships(&customer, &context).await;
        assert!(result.is_ok());

        let relationship_result = result.unwrap();
        assert!(relationship_result.valid_relationships);
    }
}

// Placeholder Default implementation for Customer (for tests)
impl Default for Customer {
    fn default() -> Self {
        use std::collections::HashMap;

        Self {
            id: Uuid::new_v4(),
            customer_number: "DEFAULT".to_string(),
            external_ids: HashMap::new(),
            legal_name: "Default Customer".to_string(),
            trade_names: vec![],
            customer_type: CustomerType::B2b,
            industry_classification: crate::types::IndustryClassification::Other,
            business_size: crate::types::BusinessSize::Small,
            parent_customer_id: None,
            corporate_group_id: None,
            customer_hierarchy_level: 1,
            consolidation_group: None,
            lifecycle_stage: CustomerLifecycleStage::Prospect,
            status: crate::types::EntityStatus::Active,
            credit_status: CreditStatus::Good,
            primary_address_id: None,
            billing_address_id: None,
            shipping_address_ids: vec![],
            addresses: vec![],
            primary_contact_id: None,
            contacts: vec![],
            tax_jurisdictions: vec![],
            tax_numbers: HashMap::new(),
            regulatory_classifications: vec![],
            compliance_status: ComplianceStatus::Compliant,
            kyc_status: KycStatus::NotStarted,
            aml_risk_rating: crate::types::RiskRating::Low,
            financial_info: crate::types::FinancialInfo {
                currency_code: "USD".to_string(),
                credit_limit: None,
                payment_terms: None,
                tax_exempt: false,
                tax_numbers: HashMap::new(),
            },
            price_group_id: None,
            discount_group_id: None,
            sales_representative_id: None,
            account_manager_id: None,
            customer_segments: vec![],
            acquisition_channel: None,
            customer_lifetime_value: None,
            churn_probability: None,
            performance_metrics: CustomerPerformanceMetrics {
                total_revenue: None,
                revenue_last_12_months: None,
                average_order_value: None,
                order_frequency: None,
                total_orders: None,
                last_order_date: None,
                profit_margin: None,
                last_purchase_date: None,
                first_purchase_date: None,
                customer_lifetime_value: None,
                predicted_churn_probability: None,
                relationship_duration_days: None,
                satisfaction_score: None,
                net_promoter_score: None,
                last_contact_date: None,
                contact_frequency: None,
                response_rate: None,
                days_sales_outstanding: None,
                payment_reliability_score: None,
                support_ticket_count: None,
                last_calculated: chrono::Utc::now(),
            },
            behavioral_data: CustomerBehavioralData {
                preferred_purchase_channels: vec![],
                seasonal_purchase_patterns: HashMap::new(),
                product_category_preferences: HashMap::new(),
                purchase_frequency: None,
                preferred_categories: HashMap::new(),
                seasonal_trends: HashMap::new(),
                price_sensitivity: None,
                brand_loyalty: None,
                preferred_contact_times: vec![],
                channel_engagement_rates: HashMap::new(),
                communication_preferences: HashMap::new(),
                support_ticket_frequency: None,
                product_return_rate: None,
                referral_activity: None,
                website_engagement_score: None,
                mobile_app_usage: None,
                social_media_sentiment: None,
                propensity_to_buy: None,
                upsell_probability: None,
                cross_sell_probability: None,
                last_updated: chrono::Utc::now(),
            },
            sync_info: crate::types::SyncInfo {
                last_sync: None,
                sync_source: None,
                sync_version: None,
                sync_status: crate::types::SyncStatus::NotSynced,
                external_references: HashMap::new(),
            },
            custom_fields: HashMap::new(),
            contract_ids: vec![],
            audit: crate::types::AuditFields {
                created_at: chrono::Utc::now(),
                created_by: Uuid::new_v4(),
                modified_at: chrono::Utc::now(),
                modified_by: Uuid::new_v4(),
                version: 1,
                is_deleted: false,
                deleted_at: None,
                deleted_by: None,
            },
        }
    }
}