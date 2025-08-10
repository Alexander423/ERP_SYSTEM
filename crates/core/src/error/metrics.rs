use super::{Error, ErrorCategory, ErrorSeverity};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Error metrics collector for monitoring and alerting
#[derive(Debug)]
pub struct ErrorMetrics {
    /// Total error count by category
    error_counts: Arc<RwLock<HashMap<ErrorCategory, AtomicU64>>>,
    /// Error count by severity
    severity_counts: Arc<RwLock<HashMap<ErrorSeverity, AtomicU64>>>,
    /// Error rate tracking (errors per time window)
    error_rate_tracker: Arc<RwLock<ErrorRateTracker>>,
}

#[derive(Debug)]
struct ErrorRateTracker {
    /// Sliding window of error timestamps
    error_timestamps: Vec<chrono::DateTime<chrono::Utc>>,
    /// Window size in seconds
    window_size: u64,
}

impl ErrorMetrics {
    pub fn new() -> Self {
        Self {
            error_counts: Arc::new(RwLock::new(HashMap::new())),
            severity_counts: Arc::new(RwLock::new(HashMap::new())),
            error_rate_tracker: Arc::new(RwLock::new(ErrorRateTracker {
                error_timestamps: Vec::new(),
                window_size: 300, // 5 minutes
            })),
        }
    }

    /// Record an error occurrence
    pub async fn record_error(&self, error: &Error) {
        // Update category counters
        {
            let mut counts = self.error_counts.write().await;
            counts
                .entry(error.category())
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }

        // Update severity counters
        {
            let mut counts = self.severity_counts.write().await;
            counts
                .entry(error.severity)
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }

        // Update error rate tracker
        {
            let mut tracker = self.error_rate_tracker.write().await;
            tracker.record_error();
        }

        // Log error based on severity
        match error.severity {
            ErrorSeverity::Low => {
                tracing::debug!(
                    error_id = %error.context.error_id,
                    error_code = ?error.code,
                    "Low severity error occurred"
                );
            }
            ErrorSeverity::Medium => {
                tracing::info!(
                    error_id = %error.context.error_id,
                    error_code = ?error.code,
                    message = %error.message,
                    "Medium severity error occurred"
                );
            }
            ErrorSeverity::High => {
                tracing::warn!(
                    error_id = %error.context.error_id,
                    error_code = ?error.code,
                    message = %error.message,
                    details = ?error.details,
                    "High severity error occurred"
                );
            }
            ErrorSeverity::Critical => {
                tracing::error!(
                    error_id = %error.context.error_id,
                    error_code = ?error.code,
                    message = %error.message,
                    details = ?error.details,
                    context = ?error.context,
                    "Critical error occurred"
                );
            }
        }
    }

    /// Get error count by category
    pub async fn get_error_count(&self, category: ErrorCategory) -> u64 {
        let counts = self.error_counts.read().await;
        counts
            .get(&category)
            .map(|counter| counter.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get error count by severity
    pub async fn get_severity_count(&self, severity: ErrorSeverity) -> u64 {
        let counts = self.severity_counts.read().await;
        counts
            .get(&severity)
            .map(|counter| counter.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Get current error rate (errors per minute)
    pub async fn get_error_rate(&self) -> f64 {
        let tracker = self.error_rate_tracker.read().await;
        tracker.get_error_rate()
    }

    /// Get all metrics as a structured format
    pub async fn get_all_metrics(&self) -> ErrorMetricsSnapshot {
        let error_counts = self.error_counts.read().await;
        let severity_counts = self.severity_counts.read().await;
        let error_rate = {
            let tracker = self.error_rate_tracker.read().await;
            tracker.get_error_rate()
        };

        let mut category_counts = HashMap::new();
        for (category, counter) in error_counts.iter() {
            category_counts.insert(*category, counter.load(Ordering::Relaxed));
        }

        let mut severity_count_map = HashMap::new();
        for (severity, counter) in severity_counts.iter() {
            severity_count_map.insert(*severity, counter.load(Ordering::Relaxed));
        }

        ErrorMetricsSnapshot {
            category_counts,
            severity_counts: severity_count_map,
            error_rate,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check if system is experiencing high error rates
    pub async fn is_high_error_rate(&self) -> bool {
        self.get_error_rate().await > 10.0 // More than 10 errors per minute
    }

    /// Reset all metrics (useful for testing)
    pub async fn reset(&self) {
        let mut error_counts = self.error_counts.write().await;
        let mut severity_counts = self.severity_counts.write().await;
        let mut tracker = self.error_rate_tracker.write().await;

        error_counts.clear();
        severity_counts.clear();
        tracker.error_timestamps.clear();
    }
}

impl ErrorRateTracker {
    fn record_error(&mut self) {
        let now = chrono::Utc::now();
        self.error_timestamps.push(now);
        
        // Clean old timestamps outside the window
        let cutoff = now - chrono::Duration::seconds(self.window_size as i64);
        self.error_timestamps.retain(|&timestamp| timestamp > cutoff);
    }

    fn get_error_rate(&self) -> f64 {
        let window_minutes = self.window_size as f64 / 60.0;
        self.error_timestamps.len() as f64 / window_minutes
    }
}

impl Default for ErrorMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of error metrics at a point in time
#[derive(Debug, Clone)]
pub struct ErrorMetricsSnapshot {
    pub category_counts: HashMap<ErrorCategory, u64>,
    pub severity_counts: HashMap<ErrorSeverity, u64>,
    pub error_rate: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorMetricsSnapshot {
    /// Convert to JSON for API responses
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "category_counts": self.category_counts,
            "severity_counts": self.severity_counts,
            "error_rate_per_minute": self.error_rate,
            "timestamp": self.timestamp
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCode;

    #[tokio::test]
    async fn test_error_metrics_recording() {
        let metrics = ErrorMetrics::new();
        
        let error = Error::new(ErrorCode::ValidationFailed, "Test error");
        metrics.record_error(&error).await;

        let validation_count = metrics.get_error_count(ErrorCategory::Validation).await;
        assert_eq!(validation_count, 1);

        let low_severity_count = metrics.get_severity_count(ErrorSeverity::Low).await;
        assert_eq!(low_severity_count, 1);
    }

    #[tokio::test]
    async fn test_error_rate_tracking() {
        let metrics = ErrorMetrics::new();
        
        // Record multiple errors
        for _ in 0..5 {
            let error = Error::new(ErrorCode::InternalServerError, "Test error");
            metrics.record_error(&error).await;
        }

        let error_rate = metrics.get_error_rate().await;
        assert!(error_rate > 0.0);
    }
}