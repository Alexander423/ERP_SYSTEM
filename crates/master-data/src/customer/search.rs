use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use sqlx::Row;
use rust_decimal::Decimal;

use crate::customer::model::*;
use crate::types::{IndustryClassification, RiskRating, FinancialInfo};
use crate::error::Result;

/// Advanced search capabilities for customers
#[async_trait]
pub trait CustomerSearchEngine: Send + Sync {
    /// Perform full-text search across customer data
    async fn full_text_search(&self, query: &str, options: &SearchOptions) -> Result<SearchResults>;

    /// Semantic search using AI/ML for intelligent matching
    async fn semantic_search(&self, query: &str, options: &SearchOptions) -> Result<SearchResults>;

    /// Find similar customers based on characteristics
    async fn find_similar_customers(&self, customer_id: Uuid, similarity_threshold: f64) -> Result<Vec<CustomerSimilarity>>;

    /// Advanced filtering with multiple criteria
    async fn advanced_filter(&self, filters: &AdvancedSearchFilters) -> Result<SearchResults>;

    /// Search suggestions and auto-completion
    async fn search_suggestions(&self, partial_query: &str, limit: u32) -> Result<Vec<SearchSuggestion>>;

    /// Fuzzy search for handling typos and variations
    async fn fuzzy_search(&self, query: &str, options: &FuzzySearchOptions) -> Result<SearchResults>;

    /// Geographic search for location-based queries
    async fn geographic_search(&self, location: &GeographicQuery) -> Result<SearchResults>;
}

/// Search options for configuring search behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct SearchOptions {
    /// Maximum number of results to return
    pub limit: u32,

    /// Offset for pagination
    pub offset: u32,

    /// Minimum relevance score (0.0 to 1.0)
    pub min_score: f64,

    /// Fields to search in
    pub search_fields: Option<Vec<String>>,

    /// Fields to include in results
    pub include_fields: Option<Vec<String>>,

    /// Whether to include snippets/highlights
    pub include_highlights: bool,

    /// Boost factors for different fields
    pub field_boosts: Option<HashMap<String, f64>>,

    /// Filters to apply
    pub filters: Option<AdvancedSearchFilters>,
}

/// Advanced search filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSearchFilters {
    /// Text-based filters
    pub text_filters: Option<HashMap<String, TextFilter>>,

    /// Numeric range filters
    pub numeric_filters: Option<HashMap<String, NumericFilter>>,

    /// Date range filters
    pub date_filters: Option<HashMap<String, DateFilter>>,

    /// Boolean filters
    pub boolean_filters: Option<HashMap<String, bool>>,

    /// Multi-select filters
    pub multi_select_filters: Option<HashMap<String, Vec<String>>>,

    /// Geographic filters
    pub geo_filters: Option<Vec<GeographicFilter>>,

    /// Custom business logic filters
    pub business_filters: Option<Vec<BusinessFilter>>,
}

/// Text filter options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFilter {
    pub value: String,
    pub operator: TextOperator,
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextOperator {
    Equals,
    Contains,
    StartsWith,
    EndsWith,
    Regex,
    Fuzzy { threshold: f64 },
}

/// Numeric filter for ranges and comparisons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericFilter {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub exact: Option<f64>,
}

/// Date filter for time-based queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateFilter {
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    pub relative: Option<RelativeDateFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelativeDateFilter {
    LastDays(u32),
    LastWeeks(u32),
    LastMonths(u32),
    LastYears(u32),
    ThisWeek,
    ThisMonth,
    ThisYear,
}

/// Geographic filter for location-based searches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicFilter {
    pub center: GeoPoint,
    pub radius_km: f64,
    pub bounds: Option<GeoBounds>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoBounds {
    pub north_east: GeoPoint,
    pub south_west: GeoPoint,
}

/// Business logic filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessFilter {
    pub filter_type: BusinessFilterType,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BusinessFilterType {
    HighValueCustomers,
    AtRiskCustomers,
    RecentlyActive,
    DormantCustomers,
    PaymentIssues,
    ComplianceIssues,
    SimilarToCustomer { customer_id: Uuid },
    CustomRule { rule_name: String },
}

/// Search results with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub customers: Vec<CustomerSearchResult>,
    pub total_count: u64,
    pub max_score: f64,
    pub search_time_ms: u64,
    pub facets: Option<SearchFacets>,
    pub suggestions: Option<Vec<SearchSuggestion>>,
}

/// Individual search result with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSearchResult {
    pub customer: Customer,
    pub score: f64,
    pub highlights: Option<HashMap<String, Vec<String>>>,
    pub explanation: Option<ScoreExplanation>,
}

/// Explanation of how the relevance score was calculated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreExplanation {
    pub total_score: f64,
    pub field_scores: HashMap<String, f64>,
    pub boost_factors: HashMap<String, f64>,
    pub description: String,
}

/// Search facets for result aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacets {
    pub customer_types: HashMap<String, u64>,
    pub lifecycle_stages: HashMap<String, u64>,
    pub industries: HashMap<String, u64>,
    pub countries: HashMap<String, u64>,
    pub credit_ratings: HashMap<String, u64>,
    pub revenue_ranges: HashMap<String, u64>,
}

/// Customer similarity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSimilarity {
    pub customer_id: Uuid,
    pub customer_name: String,
    pub similarity_score: f64,
    pub matching_attributes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityFactor {
    pub factor_type: SimilarityFactorType,
    pub weight: f64,
    pub contribution: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimilarityFactorType {
    Industry,
    BusinessSize,
    Geography,
    PurchaseBehavior,
    Revenue,
    PaymentTerms,
    ProductPreferences,
    ContactPatterns,
}

/// Search suggestions for auto-completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestion {
    pub suggestion: String,
    pub suggestion_type: SuggestionType,
    pub frequency: u32,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    CustomerName,
    CompanyName,
    Industry,
    Location,
    Product,
    Tag,
}

/// Fuzzy search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzySearchOptions {
    pub max_edits: u32,
    pub prefix_length: u32,
    pub max_expansions: u32,
    pub transpositions: bool,
}

/// Geographic query types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeographicQuery {
    NearPoint { point: GeoPoint, radius_km: f64 },
    WithinBounds { bounds: GeoBounds },
    NearAddress { address: String, radius_km: f64 },
    InRegion { country: Option<String>, state: Option<String>, city: Option<String> },
}

/// Advanced search engine implementation using PostgreSQL
pub struct AdvancedSearchEngine {
    pool: sqlx::PgPool,
    tenant_context: erp_core::TenantContext,
}

impl AdvancedSearchEngine {
    pub fn new(pool: sqlx::PgPool, tenant_context: erp_core::TenantContext) -> Self {
        Self {
            pool,
            tenant_context,
        }
    }

    pub async fn initialize_search_index(&self) -> Result<()> {
        // For PostgreSQL-only implementation, we refresh the materialized view
        sqlx::query("SELECT refresh_customer_search_cache()")
            .execute(&self.pool)
            .await?;

        // Use postgres_full_text_search for enhanced search capabilities
        let _ = self.postgres_full_text_search("test_initialization", &SearchOptions::default()).await?;

        Ok(())
    }

    async fn load_customer_by_id(&self, customer_id: uuid::Uuid) -> Result<Option<Customer>> {
        // Load customer with basic information for search results
        let row = sqlx::query(
            r#"
            SELECT c.*
            FROM customers c
            WHERE c.id = $1 AND c.tenant_id = $2 AND c.is_deleted = false
            "#,
        )
        .bind(customer_id)
        .bind(self.tenant_context.tenant_id.0)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let customer = Customer {
                id: customer_id,
                customer_number: row.try_get("customer_number")?,
                external_ids: row.try_get::<Option<serde_json::Value>, _>("external_ids")?
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
                legal_name: row.try_get("legal_name")?,
                trade_names: row.try_get::<Option<serde_json::Value>, _>("trade_names")?
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default(),
                customer_type: row.try_get("customer_type")?,
                industry_classification: row.try_get::<Option<IndustryClassification>, _>("industry_classification").ok().flatten().unwrap_or(IndustryClassification::Other),
                business_size: row.try_get::<Option<crate::types::BusinessSize>, _>("business_size").ok().flatten().unwrap_or(crate::types::BusinessSize::Small),
                parent_customer_id: row.try_get("parent_customer_id")?,
                corporate_group_id: row.try_get("corporate_group_id")?,
                customer_hierarchy_level: row.try_get::<Option<i16>, _>("customer_hierarchy_level")?.unwrap_or(0) as u8,
                consolidation_group: row.try_get::<Option<String>, _>("consolidation_group").ok().flatten(),
                lifecycle_stage: row.try_get::<CustomerLifecycleStage, _>("lifecycle_stage").ok().unwrap_or(CustomerLifecycleStage::Lead),
                status: row.try_get::<crate::types::EntityStatus, _>("status").ok().unwrap_or(crate::types::EntityStatus::Active),
                credit_status: row.try_get::<Option<CreditStatus>, _>("credit_status").ok().flatten().unwrap_or(CreditStatus::Good),
                primary_address_id: row.try_get::<Option<uuid::Uuid>, _>("primary_address_id").ok().flatten(),
                billing_address_id: row.try_get::<Option<uuid::Uuid>, _>("billing_address_id").ok().flatten(),
                shipping_address_ids: row.try_get::<Option<Vec<uuid::Uuid>>, _>("shipping_address_ids").ok().flatten().unwrap_or_default(),
                addresses: Vec::new(), // Load separately if needed
                primary_contact_id: row.try_get::<Option<uuid::Uuid>, _>("primary_contact_id").ok().flatten(),
                contacts: Vec::new(), // Load separately if needed
                tax_jurisdictions: HashMap::new(),
                tax_numbers: HashMap::new(),
                regulatory_classifications: HashMap::new(),
                compliance_status: row.try_get::<Option<ComplianceStatus>, _>("compliance_status").ok().flatten().unwrap_or(ComplianceStatus::Unknown),
                kyc_status: row.try_get::<Option<KycStatus>, _>("kyc_status").ok().flatten().unwrap_or(KycStatus::NotStarted),
                aml_risk_rating: row.try_get::<Option<RiskRating>, _>("aml_risk_rating").ok().flatten().unwrap_or(RiskRating::Low),
                financial_info: FinancialInfo {
                    currency_code: row.try_get::<Option<String>, _>("currency_code").ok().flatten().unwrap_or_else(|| "USD".to_string()),
                    credit_limit: row.try_get::<Option<rust_decimal::Decimal>, _>("credit_limit").ok().flatten(),
                    payment_terms: None,
                    tax_exempt: row.try_get::<bool, _>("tax_exempt").ok().unwrap_or(false),
                    tax_numbers: HashMap::new(),
                },
                price_group_id: row.try_get::<Option<uuid::Uuid>, _>("price_group_id").ok().flatten(),
                discount_group_id: row.try_get::<Option<uuid::Uuid>, _>("discount_group_id").ok().flatten(),
                sales_representative_id: row.try_get::<Option<uuid::Uuid>, _>("sales_representative_id").ok().flatten(),
                account_manager_id: row.try_get::<Option<uuid::Uuid>, _>("account_manager_id").ok().flatten(),
                customer_segments: vec![],
                acquisition_channel: row.try_get::<Option<AcquisitionChannel>, _>("acquisition_channel").ok().flatten(),
                customer_lifetime_value: row.try_get::<Option<rust_decimal::Decimal>, _>("customer_lifetime_value").ok().flatten(),
                churn_probability: row.try_get::<Option<rust_decimal::Decimal>, _>("churn_probability").ok().flatten().map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
                performance_metrics: CustomerPerformanceMetrics::default(),
                behavioral_data: CustomerBehavioralData::default(),
                sync_info: None,
                created_at: row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at").unwrap_or_else(|_| chrono::Utc::now()),
                modified_at: row.try_get::<chrono::DateTime<chrono::Utc>, _>("modified_at").unwrap_or_else(|_| chrono::Utc::now()),
                created_by: row.try_get::<uuid::Uuid, _>("created_by").unwrap_or_else(|_| uuid::Uuid::new_v4()),
                modified_by: row.try_get::<Option<uuid::Uuid>, _>("modified_by").ok().flatten(),
                version: row.try_get::<i32, _>("version").unwrap_or(1),
            };
            Ok(Some(customer))
        } else {
            Ok(None)
        }
    }

    async fn postgres_full_text_search(&self, query: &str, options: &SearchOptions) -> Result<SearchResults> {
        let start_time = std::time::Instant::now();

        // Use PostgreSQL's advanced full-text search with ranking
        let customers = sqlx::query(
            r#"
            SELECT c.id,
                   ts_rank(
                       to_tsvector('english',
                           COALESCE(c.legal_name, '') || ' ' ||
                           COALESCE(c.customer_number, '') || ' ' ||
                           COALESCE(c.notes, '')
                       ),
                       plainto_tsquery('english', $2)
                   ) as search_rank
            FROM customers c
            WHERE c.tenant_id = $1 AND NOT c.is_deleted
              AND to_tsvector('english',
                  COALESCE(c.legal_name, '') || ' ' ||
                  COALESCE(c.customer_number, '') || ' ' ||
                  COALESCE(c.notes, '')
              ) @@ plainto_tsquery('english', $2)
            ORDER BY search_rank DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(self.tenant_context.tenant_id.0)
        .bind(query)
        .bind(options.limit as i64)
        .bind(options.offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut customer_results = Vec::new();
        let mut max_score = 0.0;

        for customer_row in customers {
            let customer_id: Uuid = customer_row.try_get("id")?;
            let customer = self.load_customer_by_id(customer_id).await?;
            if let Some(customer_data) = customer {
                let score = customer_row.try_get::<Option<f64>, _>("search_rank")?.unwrap_or(0.0);
                max_score = f64::max(max_score, score);

                customer_results.push(CustomerSearchResult {
                    customer: customer_data,
                    score,
                    highlights: None,
                    explanation: None,
                });
            }
        }

        let search_time_ms = start_time.elapsed().as_millis() as u64;
        let total_count = customer_results.len() as u64;

        Ok(SearchResults {
            customers: customer_results,
            total_count,
            max_score,
            search_time_ms,
            facets: None,
            suggestions: None,
        })
    }

    // Helper methods for search functionality
    fn enhance_query_with_synonyms(&self, query: &str) -> String {
        // Simple synonym expansion - in production would use a proper thesaurus
        let synonyms = [
            ("company", "business organization firm corporation enterprise"),
            ("tech", "technology software IT computer digital"),
            ("bank", "financial banking finance institution"),
            ("store", "retail shop outlet merchant"),
            ("manufacturer", "factory production industrial maker"),
        ];

        let mut enhanced = query.to_lowercase();
        for (word, expansions) in &synonyms {
            if enhanced.contains(word) {
                enhanced = enhanced.replace(word, &format!("{} {}", word, expansions));
            }
        }
        enhanced
    }

    fn calculate_semantic_relevance(&self, customer: &Customer, query: &str) -> f64 {
        let mut relevance: f64 = 1.0;
        let query_lower = query.to_lowercase();

        // Boost relevance based on business context
        if query_lower.contains("enterprise") && customer.customer_type == CustomerType::B2b {
            relevance *= 1.5;
        }

        if query_lower.contains("large") && customer.customer_lifetime_value.unwrap_or_default() > rust_decimal::Decimal::from(100000) {
            relevance *= 1.3;
        }

        if query_lower.contains("active") && customer.lifecycle_stage == CustomerLifecycleStage::ActiveCustomer {
            relevance *= 1.2;
        }

        relevance.min(2.0f64) // Cap at 2x boost
    }

    fn calculate_customer_similarity(&self, reference: &Customer, candidate: &sqlx::postgres::PgRow) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Industry similarity (high weight)
        if let Ok(industry) = candidate.try_get::<IndustryClassification, _>("industry_classification") {
            if reference.industry_classification == industry {
                score += 0.3;
            }
        }
        factors += 1;

        // Customer type similarity (high weight)
        if let Ok(ctype) = candidate.try_get::<CustomerType, _>("customer_type") {
            if reference.customer_type == ctype {
                score += 0.25;
            }
        }
        factors += 1;

        // Lifecycle stage similarity
        if let Ok(stage) = candidate.try_get::<CustomerLifecycleStage, _>("lifecycle_stage") {
            if reference.lifecycle_stage == stage {
                score += 0.2;
            }
        }
        factors += 1;

        // Revenue similarity (normalize to 0-1 scale)
        let ref_clv = reference.customer_lifetime_value.unwrap_or_default();
        let candidate_clv = candidate.try_get::<Option<Decimal>, _>("customer_lifetime_value")
            .unwrap_or_default()
            .unwrap_or_default();

        let ref_clv_f64 = ref_clv.to_string().parse::<f64>().unwrap_or(0.0);
        let candidate_clv_f64 = candidate_clv.to_string().parse::<f64>().unwrap_or(0.0);

        if ref_clv_f64 > 0.0 && candidate_clv_f64 > 0.0 {
            let ratio = (ref_clv_f64 / candidate_clv_f64).min(candidate_clv_f64 / ref_clv_f64);
            score += ratio * 0.25;
        }
        factors += 1;

        score / factors as f64
    }

    fn get_matching_attributes(&self, reference: &Customer, candidate: &sqlx::postgres::PgRow) -> Vec<String> {
        let mut matches = Vec::new();

        if let Ok(ctype) = candidate.try_get::<CustomerType, _>("customer_type") {
            if reference.customer_type == ctype {
                matches.push("Customer Type".to_string());
            }
        }

        if let Ok(industry) = candidate.try_get::<IndustryClassification, _>("industry_classification") {
            if reference.industry_classification == industry {
                matches.push("Industry".to_string());
            }
        }

        if let Ok(stage) = candidate.try_get::<CustomerLifecycleStage, _>("lifecycle_stage") {
            if reference.lifecycle_stage == stage {
                matches.push("Lifecycle Stage".to_string());
            }
        }

        // Revenue range matching
        let ref_clv = reference.customer_lifetime_value.unwrap_or_default();
        let candidate_clv = candidate.try_get::<Option<Decimal>, _>("customer_lifetime_value")
            .unwrap_or_default()
            .unwrap_or_default();

        let ref_clv_f64 = ref_clv.to_string().parse::<f64>().unwrap_or(0.0);
        let candidate_clv_f64 = candidate_clv.to_string().parse::<f64>().unwrap_or(0.0);

        if (ref_clv_f64 - candidate_clv_f64).abs() / ref_clv_f64.max(1.0_f64) < 0.3 {
            matches.push("Revenue Range".to_string());
        }

        matches
    }
}

#[async_trait]
impl CustomerSearchEngine for AdvancedSearchEngine {
    async fn full_text_search(&self, query: &str, options: &SearchOptions) -> Result<SearchResults> {
        let start_time = std::time::Instant::now();

        // Use PostgreSQL's advanced full-text search with ranking
        let _ts_query = query
            .split_whitespace()
            .map(|word| format!("{}:*", word)) // Add prefix matching
            .collect::<Vec<_>>()
            .join(" & ");

        let customers = sqlx::query(
            r#"
            SELECT c.id, c.customer_number, c.legal_name, c.customer_type,
                   c.lifecycle_stage,
                   c.industry_classification, c.customer_lifetime_value, c.credit_limit,
                   c.created_at, c.modified_at,
                   ts_rank(
                       to_tsvector('english',
                           COALESCE(c.legal_name, '') || ' ' ||
                           COALESCE(c.customer_number, '') || ' ' ||
                           COALESCE(c.notes, '')
                       ),
                       plainto_tsquery('english', $2)
                   ) as search_rank
            FROM customers c
            WHERE c.tenant_id = $1 AND NOT c.is_deleted
              AND to_tsvector('english',
                  COALESCE(c.legal_name, '') || ' ' ||
                  COALESCE(c.customer_number, '') || ' ' ||
                  COALESCE(c.notes, '')
              ) @@ plainto_tsquery('english', $2)
            ORDER BY search_rank DESC, c.legal_name ASC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(self.tenant_context.tenant_id.0)
        .bind(query)
        .bind(options.limit as i64)
        .bind(options.offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut customer_results = Vec::new();
        let mut max_score = 0.0;

        for (_index, customer_row) in customers.iter().enumerate() {
            let customer_id: Uuid = customer_row.try_get("id")?;
            let customer = self.load_customer_by_id(customer_id).await?;
            if let Some(customer_data) = customer {
                let score = customer_row.try_get::<Option<f64>, _>("search_rank")?.unwrap_or(0.0);
                max_score = f64::max(max_score, score as f64);

                customer_results.push(CustomerSearchResult {
                    customer: customer_data,
                    score,
                    highlights: None,
                    explanation: None,
                });
            }
        }

        let search_time_ms = start_time.elapsed().as_millis() as u64;
        let total_count = customer_results.len() as u64;

        Ok(SearchResults {
            customers: customer_results,
            total_count,
            max_score,
            search_time_ms,
            facets: None,
            suggestions: None,
        })
    }

    async fn semantic_search(&self, query: &str, options: &SearchOptions) -> Result<SearchResults> {
        // For now, implement enhanced keyword matching with synonyms and context
        // In production, this would integrate with AI/ML services for true semantic search

        let enhanced_query = self.enhance_query_with_synonyms(query);

        // Use the enhanced query with full-text search
        let mut results = self.full_text_search(&enhanced_query, options).await?;

        // Apply semantic scoring boost based on business context
        for result in &mut results.customers {
            result.score *= self.calculate_semantic_relevance(&result.customer, query);
        }

        // Re-sort by updated scores
        results.customers.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.max_score = results.customers.first().map(|r| r.score).unwrap_or(0.0);

        Ok(results)
    }

    async fn find_similar_customers(&self, customer_id: Uuid, similarity_threshold: f64) -> Result<Vec<CustomerSimilarity>> {
        // Load the reference customer
        let reference_customer = self.load_customer_by_id(customer_id).await?
            .ok_or_else(|| crate::error::MasterDataError::NotFound)?;

        // Find similar customers based on multiple criteria
        let similar_customers = sqlx::query(
            r#"
            SELECT id, legal_name, customer_type,
                   industry_classification, customer_lifetime_value, credit_limit,
                   lifecycle_stage
            FROM customers
            WHERE tenant_id = $1 AND NOT is_deleted AND id != $2
              AND (
                customer_type = $3 OR
                industry_classification = $4 OR
                (customer_lifetime_value BETWEEN $5 - $5 * 0.3 AND $5 + $5 * 0.3) OR
                lifecycle_stage = $6
              )
            ORDER BY
              (CASE WHEN customer_type = $3 THEN 1 ELSE 0 END +
               CASE WHEN industry_classification = $4 THEN 1 ELSE 0 END +
               CASE WHEN lifecycle_stage = $6 THEN 1 ELSE 0 END) DESC,
              ABS(COALESCE(customer_lifetime_value, 0) - $5) ASC
            LIMIT 20
            "#,
        )
        .bind(self.tenant_context.tenant_id.0)
        .bind(customer_id)
        .bind(reference_customer.customer_type.clone() as CustomerType)
        .bind(reference_customer.industry_classification.clone())
        .bind(reference_customer.customer_lifetime_value.unwrap_or(rust_decimal::Decimal::ZERO))
        .bind(reference_customer.lifecycle_stage.clone() as CustomerLifecycleStage)
        .fetch_all(&self.pool)
        .await?;

        let mut similarities = Vec::new();
        for similar in similar_customers {
            let similarity_score = self.calculate_customer_similarity(&reference_customer, &similar);

            if similarity_score >= similarity_threshold {
                let similar_id: Uuid = similar.try_get("id")?;
                let customer_name: String = similar.try_get("legal_name")?;
                similarities.push(CustomerSimilarity {
                    customer_id: similar_id,
                    customer_name,
                    similarity_score,
                    matching_attributes: self.get_matching_attributes(&reference_customer, &similar),
                });
            }
        }

        similarities.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(similarities)
    }

    async fn advanced_filter(&self, _filters: &AdvancedSearchFilters) -> Result<SearchResults> {
        // Implementation would build complex SQL queries with JOINs and subqueries
        // Handle all the different filter types

        // Placeholder implementation
        Ok(SearchResults {
            customers: vec![],
            total_count: 0,
            max_score: 0.0,
            search_time_ms: 25,
            facets: Some(SearchFacets {
                customer_types: HashMap::new(),
                lifecycle_stages: HashMap::new(),
                industries: HashMap::new(),
                countries: HashMap::new(),
                credit_ratings: HashMap::new(),
                revenue_ranges: HashMap::new(),
            }),
            suggestions: None,
        })
    }

    async fn search_suggestions(&self, _partial_query: &str, _limit: u32) -> Result<Vec<SearchSuggestion>> {
        // Implementation would use trigram similarity and frequency analysis
        // SELECT word, similarity(word, 'query') FROM customer_search_index

        // Placeholder implementation
        Ok(vec![])
    }

    async fn fuzzy_search(&self, _query: &str, _options: &FuzzySearchOptions) -> Result<SearchResults> {
        // Implementation would use PostgreSQL's fuzzy matching capabilities
        // levenshtein, soundex, metaphone functions

        // Placeholder implementation
        Ok(SearchResults {
            customers: vec![],
            total_count: 0,
            max_score: 0.0,
            search_time_ms: 15,
            facets: None,
            suggestions: None,
        })
    }

    async fn geographic_search(&self, _location: &GeographicQuery) -> Result<SearchResults> {
        // Implementation would use PostGIS for geographic queries
        // ST_DWithin, ST_Contains, ST_Distance functions

        // Placeholder implementation
        Ok(SearchResults {
            customers: vec![],
            total_count: 0,
            max_score: 0.0,
            search_time_ms: 30,
            facets: None,
            suggestions: None,
        })
    }
}

/// Search query builder for constructing complex searches
pub struct CustomerSearchBuilder {
    query: String,
    options: SearchOptions,
    filters: AdvancedSearchFilters,
}

impl CustomerSearchBuilder {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            options: SearchOptions {
                limit: 50,
                offset: 0,
                min_score: 0.0,
                search_fields: None,
                include_fields: None,
                include_highlights: false,
                field_boosts: None,
                filters: None,
            },
            filters: AdvancedSearchFilters {
                text_filters: None,
                numeric_filters: None,
                date_filters: None,
                boolean_filters: None,
                multi_select_filters: None,
                geo_filters: None,
                business_filters: None,
            },
        }
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.options.limit = limit;
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.options.offset = offset;
        self
    }

    pub fn min_score(mut self, score: f64) -> Self {
        self.options.min_score = score;
        self
    }

    pub fn search_fields(mut self, fields: Vec<String>) -> Self {
        self.options.search_fields = Some(fields);
        self
    }

    pub fn include_highlights(mut self) -> Self {
        self.options.include_highlights = true;
        self
    }

    pub fn boost_field(mut self, field: String, boost: f64) -> Self {
        if self.options.field_boosts.is_none() {
            self.options.field_boosts = Some(HashMap::new());
        }
        self.options.field_boosts.as_mut().unwrap().insert(field, boost);
        self
    }

    pub fn filter_customer_type(mut self, customer_types: Vec<String>) -> Self {
        if self.filters.multi_select_filters.is_none() {
            self.filters.multi_select_filters = Some(HashMap::new());
        }
        self.filters.multi_select_filters.as_mut().unwrap()
            .insert("customer_type".to_string(), customer_types);
        self
    }

    pub fn filter_revenue_range(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        if self.filters.numeric_filters.is_none() {
            self.filters.numeric_filters = Some(HashMap::new());
        }
        self.filters.numeric_filters.as_mut().unwrap()
            .insert("annual_revenue".to_string(), NumericFilter { min, max, exact: None });
        self
    }

    pub fn filter_location(mut self, center: GeoPoint, radius_km: f64) -> Self {
        if self.filters.geo_filters.is_none() {
            self.filters.geo_filters = Some(vec![]);
        }
        self.filters.geo_filters.as_mut().unwrap()
            .push(GeographicFilter { center, radius_km, bounds: None });
        self
    }

    pub fn filter_high_value_customers(mut self) -> Self {
        if self.filters.business_filters.is_none() {
            self.filters.business_filters = Some(vec![]);
        }
        self.filters.business_filters.as_mut().unwrap()
            .push(BusinessFilter {
                filter_type: BusinessFilterType::HighValueCustomers,
                parameters: HashMap::new(),
            });
        self
    }

    pub fn build(mut self) -> (String, SearchOptions) {
        self.options.filters = Some(self.filters);
        (self.query, self.options)
    }
}

impl Default for CustomerSearchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_builder() {
        let (query, options) = CustomerSearchBuilder::new()
            .query("technology company")
            .limit(25)
            .min_score(0.5)
            .search_fields(vec!["legal_name".to_string(), "industry".to_string()])
            .include_highlights()
            .boost_field("legal_name".to_string(), 2.0)
            .filter_customer_type(vec!["b2b".to_string()])
            .filter_revenue_range(Some(1000000.0), Some(10000000.0))
            .filter_high_value_customers()
            .build();

        assert_eq!(query, "technology company");
        assert_eq!(options.limit, 25);
        assert_eq!(options.min_score, 0.5);
        assert!(options.include_highlights);
        assert!(options.filters.is_some());
    }

    #[tokio::test]
    async fn test_advanced_search_engine() {
        // This test is disabled until we have a proper test database setup
        // let engine = AdvancedSearchEngine::new(pool, tenant_context);
        /*
        let options = SearchOptions {
            limit: 10,
            offset: 0,
            min_score: 0.0,
            search_fields: None,
            include_fields: None,
            include_highlights: false,
            field_boosts: None,
            filters: None,
        };

        let result = engine.full_text_search("test query", &options).await;
        assert!(result.is_ok());
        */
    }
}