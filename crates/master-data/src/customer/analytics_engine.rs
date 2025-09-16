//! Real-Time Customer Analytics Engine
//!
//! This module provides real-time analytics processing for customer events,
//! calculating metrics, generating insights, and updating dashboards.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use tokio::sync::RwLock;

use crate::customer::events::{CustomerEvent, CustomerEventWithMetadata};
use crate::customer::model::*;
use crate::error::{MasterDataError, Result};
use erp_core::TenantContext;

/// Real-time analytics engine for customer insights
#[async_trait]
pub trait CustomerAnalyticsEngine: Send + Sync {
    /// Process a customer event and update analytics
    async fn process_event(&self, event: &CustomerEventWithMetadata) -> Result<()>;

    /// Process multiple events in batch
    async fn process_events_batch(&self, events: Vec<CustomerEventWithMetadata>) -> Result<()>;

    /// Get real-time customer insights
    async fn get_customer_insights(&self, customer_id: Uuid) -> Result<CustomerInsights>;

    /// Get aggregate tenant analytics
    async fn get_tenant_analytics(&self) -> Result<TenantAnalytics>;

    /// Get performance metrics for a specific time range
    async fn get_metrics_for_period(
        &self,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> Result<PeriodMetrics>;

    /// Calculate customer lifetime value
    async fn calculate_clv(&self, customer_id: Uuid) -> Result<f64>;

    /// Calculate churn probability
    async fn calculate_churn_probability(&self, customer_id: Uuid) -> Result<f64>;

    /// Get customer segmentation
    async fn get_customer_segmentation(&self, customer_id: Uuid) -> Result<Vec<String>>;

    /// Get trending metrics
    async fn get_trending_metrics(&self) -> Result<TrendingMetrics>;
}

/// Real-time customer insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerInsights {
    pub customer_id: Uuid,
    pub calculated_at: DateTime<Utc>,

    // Financial metrics
    pub lifetime_value: f64,
    pub total_revenue: f64,
    pub average_order_value: f64,
    pub total_orders: i64,

    // Engagement metrics
    pub engagement_score: f64,
    pub last_activity_date: Option<DateTime<Utc>>,
    pub activity_frequency: f64, // activities per month

    // Risk metrics
    pub churn_probability: f64,
    pub risk_score: f64,
    pub credit_score: f64,

    // Behavioral insights
    pub preferred_channels: Vec<String>,
    pub purchase_patterns: HashMap<String, f64>,
    pub seasonal_trends: HashMap<String, f64>,

    // Predictive insights
    pub propensity_to_buy: f64,
    pub recommended_products: Vec<String>,
    pub upsell_opportunities: Vec<String>,
    pub cross_sell_opportunities: Vec<String>,

    // Segmentation
    pub segments: Vec<String>,
    pub personality_traits: HashMap<String, f64>,

    // Relationship metrics
    pub relationship_health: f64,
    pub satisfaction_score: Option<f64>,
    pub net_promoter_score: Option<i32>,
}

/// Tenant-wide analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantAnalytics {
    pub tenant_id: Uuid,
    pub calculated_at: DateTime<Utc>,

    // Customer counts
    pub total_customers: i64,
    pub active_customers: i64,
    pub new_customers_last_30_days: i64,
    pub churned_customers_last_30_days: i64,

    // Revenue metrics
    pub total_revenue: f64,
    pub revenue_last_30_days: f64,
    pub average_customer_value: f64,
    pub revenue_growth_rate: f64,

    // Lifecycle distribution
    pub lifecycle_distribution: HashMap<CustomerLifecycleStage, i64>,

    // Segmentation insights
    pub segment_distribution: HashMap<String, i64>,
    pub top_performing_segments: Vec<(String, f64)>,

    // Risk analytics
    pub high_risk_customers: i64,
    pub churn_rate: f64,
    pub average_churn_probability: f64,

    // Trends
    pub customer_acquisition_trend: Vec<(DateTime<Utc>, i64)>,
    pub revenue_trend: Vec<(DateTime<Utc>, f64)>,
    pub churn_trend: Vec<(DateTime<Utc>, f64)>,
}

/// Period-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodMetrics {
    pub from_date: DateTime<Utc>,
    pub to_date: DateTime<Utc>,
    pub calculated_at: DateTime<Utc>,

    // Event counts
    pub total_events: i64,
    pub events_by_type: HashMap<String, i64>,

    // Customer activity
    pub active_customers: i64,
    pub new_customers: i64,
    pub reactivated_customers: i64,
    pub churned_customers: i64,

    // Financial metrics
    pub revenue: f64,
    pub average_order_value: f64,
    pub orders_count: i64,

    // Performance indicators
    pub conversion_rate: f64,
    pub retention_rate: f64,
    pub satisfaction_score: f64,
}

/// Trending metrics and alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingMetrics {
    pub calculated_at: DateTime<Utc>,

    // Growth metrics
    pub customer_growth_rate: f64,
    pub revenue_growth_rate: f64,
    pub engagement_growth_rate: f64,

    // Risk alerts
    pub high_churn_risk_customers: Vec<Uuid>,
    pub credit_limit_alerts: Vec<Uuid>,
    pub compliance_alerts: Vec<Uuid>,

    // Opportunities
    pub upsell_opportunities: Vec<Uuid>,
    pub cross_sell_opportunities: Vec<Uuid>,
    pub win_back_opportunities: Vec<Uuid>,

    // Performance alerts
    pub declining_segments: Vec<String>,
    pub top_performing_customers: Vec<Uuid>,
    pub anomaly_alerts: Vec<String>,
}

/// In-memory analytics engine with real-time processing
pub struct InMemoryAnalyticsEngine {
    tenant_context: TenantContext,

    // In-memory state for fast access
    customer_metrics: Arc<RwLock<HashMap<Uuid, CustomerInsights>>>,
    tenant_metrics: Arc<RwLock<TenantAnalytics>>,
    event_counts: Arc<RwLock<HashMap<String, i64>>>,

    // Configuration
    calculation_window_days: i64,
    min_events_for_prediction: i64,
}

impl InMemoryAnalyticsEngine {
    pub fn new(tenant_context: TenantContext) -> Self {
        let tenant_analytics = TenantAnalytics {
            tenant_id: tenant_context.tenant_id.0,
            calculated_at: Utc::now(),
            total_customers: 0,
            active_customers: 0,
            new_customers_last_30_days: 0,
            churned_customers_last_30_days: 0,
            total_revenue: 0.0,
            revenue_last_30_days: 0.0,
            average_customer_value: 0.0,
            revenue_growth_rate: 0.0,
            lifecycle_distribution: HashMap::new(),
            segment_distribution: HashMap::new(),
            top_performing_segments: Vec::new(),
            high_risk_customers: 0,
            churn_rate: 0.0,
            average_churn_probability: 0.0,
            customer_acquisition_trend: Vec::new(),
            revenue_trend: Vec::new(),
            churn_trend: Vec::new(),
        };

        Self {
            tenant_context,
            customer_metrics: Arc::new(RwLock::new(HashMap::new())),
            tenant_metrics: Arc::new(RwLock::new(tenant_analytics)),
            event_counts: Arc::new(RwLock::new(HashMap::new())),
            calculation_window_days: 90, // 3 months
            min_events_for_prediction: 5,
        }
    }
}

#[async_trait]
impl CustomerAnalyticsEngine for InMemoryAnalyticsEngine {
    async fn process_event(&self, event: &CustomerEventWithMetadata) -> Result<()> {
        let customer_id = event.event.customer_id();

        // Update event counts
        {
            let mut counts = self.event_counts.write().await;
            *counts.entry(event.event.event_type().to_string()).or_insert(0) += 1;
        }

        // Process event based on type
        match &event.event {
            CustomerEvent::CustomerCreated { .. } => {
                self.handle_customer_created(event).await?;
            }

            CustomerEvent::PerformanceMetricsCalculated { .. } => {
                self.handle_performance_metrics_updated(event).await?;
            }

            CustomerEvent::LifecycleStageChanged { .. } => {
                self.handle_lifecycle_stage_changed(event).await?;
            }

            CustomerEvent::CreditStatusChanged { .. } => {
                self.handle_credit_status_changed(event).await?;
            }

            CustomerEvent::BehavioralDataUpdated { .. } => {
                self.handle_behavioral_data_updated(event).await?;
            }

            _ => {
                // For other events, just update the basic metrics
                self.update_customer_activity(customer_id, event.metadata.occurred_at).await?;
            }
        }

        // Update tenant-wide metrics
        self.update_tenant_metrics().await?;

        Ok(())
    }

    async fn process_events_batch(&self, events: Vec<CustomerEventWithMetadata>) -> Result<()> {
        for event in events {
            self.process_event(&event).await?;
        }
        Ok(())
    }

    async fn get_customer_insights(&self, customer_id: Uuid) -> Result<CustomerInsights> {
        let metrics = self.customer_metrics.read().await;

        match metrics.get(&customer_id) {
            Some(insights) => Ok(insights.clone()),
            None => {
                // Calculate insights on-demand for new customers
                self.calculate_customer_insights(customer_id).await
            }
        }
    }

    async fn get_tenant_analytics(&self) -> Result<TenantAnalytics> {
        let analytics = self.tenant_metrics.read().await;
        Ok(analytics.clone())
    }

    async fn get_metrics_for_period(
        &self,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> Result<PeriodMetrics> {
        // This would normally query the event store for the specific period
        // For now, return a simplified calculation

        let event_counts = self.event_counts.read().await;
        let total_events: i64 = event_counts.values().sum();

        Ok(PeriodMetrics {
            from_date,
            to_date,
            calculated_at: Utc::now(),
            total_events,
            events_by_type: event_counts.clone(),
            active_customers: 0, // Would calculate from events
            new_customers: 0,
            reactivated_customers: 0,
            churned_customers: 0,
            revenue: 0.0,
            average_order_value: 0.0,
            orders_count: 0,
            conversion_rate: 0.0,
            retention_rate: 0.0,
            satisfaction_score: 0.0,
        })
    }

    async fn calculate_clv(&self, customer_id: Uuid) -> Result<f64> {
        let insights = self.get_customer_insights(customer_id).await?;

        // Simplified CLV calculation: (Average Order Value × Purchase Frequency × Gross Margin × Lifespan)
        let avg_order_value = insights.average_order_value;
        let purchase_frequency = insights.activity_frequency; // purchases per month
        let gross_margin = 0.3; // 30% assumed gross margin
        let lifespan_months = 24.0; // 2 years assumed lifespan

        let clv = avg_order_value * purchase_frequency * gross_margin * lifespan_months;
        Ok(clv)
    }

    async fn calculate_churn_probability(&self, customer_id: Uuid) -> Result<f64> {
        let insights = self.get_customer_insights(customer_id).await?;

        // Simplified churn model based on various factors
        let mut churn_score = 0.0;

        // Factor 1: Time since last activity
        if let Some(last_activity) = insights.last_activity_date {
            let days_since_activity = (Utc::now() - last_activity).num_days() as f64;
            churn_score += (days_since_activity / 30.0) * 0.3; // 30% weight
        }

        // Factor 2: Engagement score (inverse relationship)
        churn_score += (1.0 - insights.engagement_score) * 0.25; // 25% weight

        // Factor 3: Revenue trend (if declining, higher churn risk)
        churn_score += insights.risk_score * 0.25; // 25% weight

        // Factor 4: Support issues (simplified)
        churn_score += 0.2; // 20% base risk

        // Normalize to 0-1 range
        Ok(churn_score.min(1.0).max(0.0))
    }

    async fn get_customer_segmentation(&self, customer_id: Uuid) -> Result<Vec<String>> {
        let insights = self.get_customer_insights(customer_id).await?;
        Ok(insights.segments.clone())
    }

    async fn get_trending_metrics(&self) -> Result<TrendingMetrics> {
        let tenant_analytics = self.get_tenant_analytics().await?;

        // Simplified trending analysis
        Ok(TrendingMetrics {
            calculated_at: Utc::now(),
            customer_growth_rate: tenant_analytics.revenue_growth_rate,
            revenue_growth_rate: tenant_analytics.revenue_growth_rate,
            engagement_growth_rate: 0.0, // Would calculate from historical data
            high_churn_risk_customers: Vec::new(), // Would identify from customer insights
            credit_limit_alerts: Vec::new(),
            compliance_alerts: Vec::new(),
            upsell_opportunities: Vec::new(),
            cross_sell_opportunities: Vec::new(),
            win_back_opportunities: Vec::new(),
            declining_segments: Vec::new(),
            top_performing_customers: Vec::new(),
            anomaly_alerts: Vec::new(),
        })
    }
}

// Private implementation methods
impl InMemoryAnalyticsEngine {
    async fn handle_customer_created(&self, event: &CustomerEventWithMetadata) -> Result<()> {
        let customer_id = event.event.customer_id();

        // Initialize customer insights
        let insights = CustomerInsights {
            customer_id,
            calculated_at: Utc::now(),
            lifetime_value: 0.0,
            total_revenue: 0.0,
            average_order_value: 0.0,
            total_orders: 0,
            engagement_score: 0.5, // Start with neutral score
            last_activity_date: Some(event.metadata.occurred_at),
            activity_frequency: 0.0,
            churn_probability: 0.1, // Low initial churn risk
            risk_score: 0.2, // Low initial risk
            credit_score: 0.7, // Good initial credit score
            preferred_channels: Vec::new(),
            purchase_patterns: HashMap::new(),
            seasonal_trends: HashMap::new(),
            propensity_to_buy: 0.5,
            recommended_products: Vec::new(),
            upsell_opportunities: Vec::new(),
            cross_sell_opportunities: Vec::new(),
            segments: vec!["new_customer".to_string()],
            personality_traits: HashMap::new(),
            relationship_health: 0.8, // Good initial relationship
            satisfaction_score: None,
            net_promoter_score: None,
        };

        let mut metrics = self.customer_metrics.write().await;
        metrics.insert(customer_id, insights);

        Ok(())
    }

    async fn handle_performance_metrics_updated(&self, event: &CustomerEventWithMetadata) -> Result<()> {
        let customer_id = event.event.customer_id();

        if let CustomerEvent::PerformanceMetricsCalculated {
            total_revenue,
            total_orders,
            customer_lifetime_value,
            ..
        } = &event.event {
            let mut metrics = self.customer_metrics.write().await;

            if let Some(insights) = metrics.get_mut(&customer_id) {
                if let Some(revenue) = total_revenue {
                    insights.total_revenue = revenue.to_string().parse().unwrap_or(0.0);
                }

                if let Some(orders) = total_orders {
                    insights.total_orders = *orders;
                    if *orders > 0 {
                        insights.average_order_value = insights.total_revenue / *orders as f64;
                    }
                }

                if let Some(clv) = customer_lifetime_value {
                    insights.lifetime_value = clv.to_string().parse().unwrap_or(0.0);
                }

                insights.calculated_at = Utc::now();
            }
        }

        Ok(())
    }

    async fn handle_lifecycle_stage_changed(&self, event: &CustomerEventWithMetadata) -> Result<()> {
        let customer_id = event.event.customer_id();

        if let CustomerEvent::LifecycleStageChanged { new_stage, .. } = &event.event {
            let mut metrics = self.customer_metrics.write().await;

            if let Some(insights) = metrics.get_mut(&customer_id) {
                // Update segmentation based on lifecycle stage
                insights.segments.clear();
                insights.segments.push(format!("{:?}", new_stage).to_lowercase());

                // Adjust engagement and risk scores based on stage
                match new_stage {
                    CustomerLifecycleStage::VipCustomer => {
                        insights.engagement_score = 0.9;
                        insights.churn_probability = 0.05;
                        insights.risk_score = 0.1;
                    }
                    CustomerLifecycleStage::AtRiskCustomer => {
                        insights.engagement_score = 0.3;
                        insights.churn_probability = 0.7;
                        insights.risk_score = 0.8;
                    }
                    CustomerLifecycleStage::InactiveCustomer => {
                        insights.engagement_score = 0.1;
                        insights.churn_probability = 0.9;
                        insights.risk_score = 0.9;
                    }
                    _ => {
                        // Default adjustments for other stages
                        insights.engagement_score = 0.6;
                        insights.churn_probability = 0.2;
                        insights.risk_score = 0.3;
                    }
                }

                insights.calculated_at = Utc::now();
            }
        }

        Ok(())
    }

    async fn handle_credit_status_changed(&self, event: &CustomerEventWithMetadata) -> Result<()> {
        let customer_id = event.event.customer_id();

        if let CustomerEvent::CreditStatusChanged { new_status, .. } = &event.event {
            let mut metrics = self.customer_metrics.write().await;

            if let Some(insights) = metrics.get_mut(&customer_id) {
                // Update credit score based on status
                insights.credit_score = match new_status {
                    CreditStatus::Excellent => 0.95,
                    CreditStatus::Good => 0.8,
                    CreditStatus::Fair => 0.6,
                    CreditStatus::Poor => 0.3,
                    CreditStatus::OnHold => 0.2,
                    CreditStatus::Blocked => 0.0,
                    CreditStatus::CashOnly => 0.1,
                    CreditStatus::RequiresPrepayment => 0.15,
                };

                insights.calculated_at = Utc::now();
            }
        }

        Ok(())
    }

    async fn handle_behavioral_data_updated(&self, event: &CustomerEventWithMetadata) -> Result<()> {
        let customer_id = event.event.customer_id();

        if let CustomerEvent::BehavioralDataUpdated {
            propensity_to_buy,
            churn_probability,
            preferred_channels,
            ..
        } = &event.event {
            let mut metrics = self.customer_metrics.write().await;

            if let Some(insights) = metrics.get_mut(&customer_id) {
                if let Some(propensity) = propensity_to_buy {
                    insights.propensity_to_buy = propensity.to_string().parse().unwrap_or(0.5);
                }

                if let Some(churn) = churn_probability {
                    insights.churn_probability = churn.to_string().parse().unwrap_or(0.2);
                }

                insights.preferred_channels = preferred_channels.clone();
                insights.calculated_at = Utc::now();
            }
        }

        Ok(())
    }

    async fn update_customer_activity(&self, customer_id: Uuid, activity_time: DateTime<Utc>) -> Result<()> {
        let mut metrics = self.customer_metrics.write().await;

        if let Some(insights) = metrics.get_mut(&customer_id) {
            insights.last_activity_date = Some(activity_time);
            insights.calculated_at = Utc::now();
        }

        Ok(())
    }

    async fn update_tenant_metrics(&self) -> Result<()> {
        let customer_metrics = self.customer_metrics.read().await;
        let mut tenant_metrics = self.tenant_metrics.write().await;

        // Calculate aggregated metrics
        tenant_metrics.total_customers = customer_metrics.len() as i64;
        tenant_metrics.active_customers = customer_metrics
            .values()
            .filter(|c| c.engagement_score > 0.3)
            .count() as i64;

        tenant_metrics.total_revenue = customer_metrics
            .values()
            .map(|c| c.total_revenue)
            .sum();

        if tenant_metrics.total_customers > 0 {
            tenant_metrics.average_customer_value =
                tenant_metrics.total_revenue / tenant_metrics.total_customers as f64;
        }

        tenant_metrics.high_risk_customers = customer_metrics
            .values()
            .filter(|c| c.churn_probability > 0.7)
            .count() as i64;

        tenant_metrics.calculated_at = Utc::now();

        Ok(())
    }

    async fn calculate_customer_insights(&self, customer_id: Uuid) -> Result<CustomerInsights> {
        // Default insights for customers not yet in cache
        Ok(CustomerInsights {
            customer_id,
            calculated_at: Utc::now(),
            lifetime_value: 0.0,
            total_revenue: 0.0,
            average_order_value: 0.0,
            total_orders: 0,
            engagement_score: 0.5,
            last_activity_date: None,
            activity_frequency: 0.0,
            churn_probability: 0.2,
            risk_score: 0.3,
            credit_score: 0.7,
            preferred_channels: Vec::new(),
            purchase_patterns: HashMap::new(),
            seasonal_trends: HashMap::new(),
            propensity_to_buy: 0.5,
            recommended_products: Vec::new(),
            upsell_opportunities: Vec::new(),
            cross_sell_opportunities: Vec::new(),
            segments: vec!["unknown".to_string()],
            personality_traits: HashMap::new(),
            relationship_health: 0.5,
            satisfaction_score: None,
            net_promoter_score: None,
        })
    }
}