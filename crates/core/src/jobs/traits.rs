use super::types::{JobId, JobPriority, JobStatus, QueuedJob};
use crate::error::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Result of a job execution
#[derive(Debug, Clone)]
pub enum JobResult {
    /// Job completed successfully
    Success {
        result: Option<serde_json::Value>,
        message: Option<String>,
    },
    /// Job failed but can be retried
    Retry {
        error: String,
        delay_seconds: Option<u64>,
    },
    /// Job failed permanently
    Failed {
        error: String,
    },
    /// Job was cancelled
    Cancelled {
        reason: String,
    },
}

impl JobResult {
    pub fn success() -> Self {
        Self::Success {
            result: None,
            message: None,
        }
    }

    pub fn success_with_result(result: serde_json::Value) -> Self {
        Self::Success {
            result: Some(result),
            message: None,
        }
    }

    pub fn success_with_message(message: impl Into<String>) -> Self {
        Self::Success {
            result: None,
            message: Some(message.into()),
        }
    }

    pub fn retry(error: impl Into<String>) -> Self {
        Self::Retry {
            error: error.into(),
            delay_seconds: None,
        }
    }

    pub fn retry_with_delay(error: impl Into<String>, delay_seconds: u64) -> Self {
        Self::Retry {
            error: error.into(),
            delay_seconds: Some(delay_seconds),
        }
    }

    pub fn failed(error: impl Into<String>) -> Self {
        Self::Failed {
            error: error.into(),
        }
    }

    pub fn cancelled(reason: impl Into<String>) -> Self {
        Self::Cancelled {
            reason: reason.into(),
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, JobResult::Success { .. })
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, JobResult::Failed { .. })
    }

    pub fn should_retry(&self) -> bool {
        matches!(self, JobResult::Retry { .. })
    }
}

/// Context provided to job handlers during execution
#[derive(Debug)]
pub struct JobContext {
    pub job_id: JobId,
    pub attempt: u32,
    pub max_attempts: u32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub started_at: DateTime<Utc>,
}

impl JobContext {
    pub fn new(job_id: JobId, attempt: u32, max_attempts: u32) -> Self {
        Self {
            job_id,
            attempt,
            max_attempts,
            metadata: HashMap::new(),
            started_at: Utc::now(),
        }
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn is_last_attempt(&self) -> bool {
        self.attempt >= self.max_attempts
    }

    pub fn elapsed_time(&self) -> chrono::Duration {
        Utc::now() - self.started_at
    }

}

/// Trait for executable jobs
#[async_trait]
pub trait Job: Send + Sync {
    /// Execute the job with the given context
    async fn execute(&self, context: &JobContext) -> JobResult;

    /// Get the job type identifier
    fn job_type(&self) -> &'static str;

    /// Get job priority
    fn priority(&self) -> JobPriority {
        JobPriority::Normal
    }

    /// Get maximum retry attempts
    fn max_attempts(&self) -> u32 {
        3
    }

    /// Get job timeout in seconds
    fn timeout(&self) -> Option<u64> {
        Some(300) // 5 minutes default
    }

    /// Get job metadata
    fn metadata(&self) -> HashMap<String, serde_json::Value> {
        HashMap::new()
    }

    /// Called before job execution (for setup/validation)
    async fn before_execute(&self, _context: &JobContext) -> Result<()> {
        Ok(())
    }

    /// Called after job execution (for cleanup)
    async fn after_execute(&self, _context: &JobContext, _result: &JobResult) -> Result<()> {
        Ok(())
    }

    /// Estimate job execution time (for scheduling)
    fn estimated_duration(&self) -> Option<chrono::Duration> {
        None
    }

    /// Get job dependencies (jobs that must complete before this one)
    fn dependencies(&self) -> Vec<JobId> {
        Vec::new()
    }

    /// Check if job should be executed (dynamic validation)
    async fn should_execute(&self, _context: &JobContext) -> bool {
        true
    }
}

/// Trait for handling specific job types
#[async_trait]
pub trait JobHandler: Send + Sync {
    /// Get the job type this handler can process
    fn job_type(&self) -> &'static str;

    /// Handle a job execution
    async fn handle(&self, job_data: &serde_json::Value, context: &JobContext) -> JobResult;

    /// Validate job data before execution
    fn validate_job_data(&self, job_data: &serde_json::Value) -> Result<()>;

    /// Get handler configuration
    fn config(&self) -> JobHandlerConfig {
        JobHandlerConfig::default()
    }
}

/// Configuration for job handlers
#[derive(Debug)]
pub struct JobHandlerConfig {
    pub max_concurrent_jobs: Option<u32>,
    pub default_timeout: Option<u64>,
    pub default_max_attempts: Option<u32>,
}

impl Default for JobHandlerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: Some(10),
            default_timeout: Some(300), // 5 minutes
            default_max_attempts: Some(3),
        }
    }
}

/// Trait for job queue implementations
#[async_trait]
pub trait JobQueue: Send + Sync {
    /// Enqueue a job for processing
    async fn enqueue(&self, job: QueuedJob) -> Result<JobId>;

    /// Dequeue the next available job
    async fn dequeue(&self, worker_id: &str) -> Result<Option<QueuedJob>>;

    /// Get job status
    async fn get_status(&self, job_id: &JobId) -> Result<Option<JobStatus>>;

    /// Update job status
    async fn update_status(&self, job_id: &JobId, status: JobStatus) -> Result<()>;

    /// Cancel a job
    async fn cancel_job(&self, job_id: &JobId) -> Result<bool>;

    /// Get job statistics
    async fn get_stats(&self) -> Result<QueueStats>;

    /// Clean up completed/failed jobs older than the specified time
    async fn cleanup_old_jobs(&self, older_than: DateTime<Utc>) -> Result<u64>;

    /// Get jobs by status
    async fn get_jobs_by_status(&self, status: super::types::JobState, limit: Option<u32>) -> Result<Vec<QueuedJob>>;

    /// Health check
    async fn health_check(&self) -> Result<bool>;
}

/// Statistics about the job queue
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueueStats {
    pub total_jobs: u64,
    pub queued_jobs: u64,
    pub processing_jobs: u64,
    pub completed_jobs: u64,
    pub failed_jobs: u64,
    pub retrying_jobs: u64,
    pub cancelled_jobs: u64,
    pub average_processing_time: Option<chrono::Duration>,
    pub jobs_per_minute: Option<f64>,
    pub error_rate: Option<f64>, // Percentage of failed jobs
}

impl Default for QueueStats {
    fn default() -> Self {
        Self {
            total_jobs: 0,
            queued_jobs: 0,
            processing_jobs: 0,
            completed_jobs: 0,
            failed_jobs: 0,
            retrying_jobs: 0,
            cancelled_jobs: 0,
            average_processing_time: None,
            jobs_per_minute: None,
            error_rate: None,
        }
    }
}

impl QueueStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_jobs == 0 {
            0.0
        } else {
            self.completed_jobs as f64 / self.total_jobs as f64
        }
    }

    pub fn is_healthy(&self) -> bool {
        // Consider queue healthy if:
        // - Error rate is below 10%
        // - Not too many jobs stuck in processing (might indicate crashed workers)
        let error_rate = self.error_rate.unwrap_or(0.0);
        let processing_ratio = if self.total_jobs > 0 {
            self.processing_jobs as f64 / self.total_jobs as f64
        } else {
            0.0
        };

        error_rate < 0.10 && processing_ratio < 0.50
    }
}

/// Progress callback for long-running jobs
pub type ProgressCallback = Box<dyn Fn(f64) + Send + Sync>;

/// Factory trait for creating jobs from serialized data
#[async_trait]
pub trait JobFactory: Send + Sync {
    /// Create a job instance from serialized data
    async fn create_job(
        &self,
        job_type: &str,
        job_data: &serde_json::Value,
    ) -> Result<Box<dyn Job>>;

    /// Get list of supported job types
    fn supported_job_types(&self) -> Vec<&'static str>;

    /// Validate job data for a given job type
    fn validate_job_data(&self, job_type: &str, job_data: &serde_json::Value) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::types::JobId;

    #[tokio::test]
    async fn test_job_result_variants() {
        let success = JobResult::success();
        assert!(success.is_success());
        assert!(!success.is_failure());
        assert!(!success.should_retry());

        let retry = JobResult::retry("Temporary error");
        assert!(!retry.is_success());
        assert!(!retry.is_failure());
        assert!(retry.should_retry());

        let failed = JobResult::failed("Permanent error");
        assert!(!failed.is_success());
        assert!(failed.is_failure());
        assert!(!failed.should_retry());
    }

    #[test]
    fn test_job_context() {
        let job_id = JobId::new();
        let context = JobContext::new(job_id.clone(), 1, 3);
        
        assert_eq!(context.job_id, job_id);
        assert_eq!(context.attempt, 1);
        assert_eq!(context.max_attempts, 3);
        assert!(!context.is_last_attempt());

        let last_attempt_context = JobContext::new(job_id, 3, 3);
        assert!(last_attempt_context.is_last_attempt());
    }

    #[test]
    fn test_queue_stats() {
        let stats = QueueStats {
            total_jobs: 100,
            queued_jobs: 10,
            processing_jobs: 5,
            completed_jobs: 80,
            failed_jobs: 3,
            retrying_jobs: 2,
            cancelled_jobs: 0,
            average_processing_time: None,
            jobs_per_minute: None,
            error_rate: Some(0.05), // 5%
        };

        assert_eq!(stats.success_rate(), 0.80);
        assert!(stats.is_healthy()); // 5% error rate is acceptable
    }
}