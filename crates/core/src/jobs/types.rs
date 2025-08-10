use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a job
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub String);

impl JobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Job priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for JobPriority {
    fn default() -> Self {
        JobPriority::Normal
    }
}

/// Current state of a job in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    /// Job is waiting to be picked up by a worker
    Queued,
    /// Job is currently being processed
    Processing,
    /// Job completed successfully
    Completed,
    /// Job failed and will not be retried
    Failed,
    /// Job failed but will be retried
    Retrying,
    /// Job was cancelled before completion
    Cancelled,
}

/// Detailed status information about a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    pub id: JobId,
    pub job_type: String,
    pub state: JobState,
    pub priority: JobPriority,
    pub created_at: DateTime<Utc>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub attempts: u32,
    pub max_attempts: u32,
    pub last_error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub progress: Option<f64>, // 0.0 to 1.0
    pub result: Option<serde_json::Value>,
}

impl JobStatus {
    pub fn new(id: JobId, job_type: impl Into<String>, priority: JobPriority) -> Self {
        Self {
            id,
            job_type: job_type.into(),
            state: JobState::Queued,
            priority,
            created_at: Utc::now(),
            scheduled_for: None,
            started_at: None,
            completed_at: None,
            attempts: 0,
            max_attempts: 3,
            last_error: None,
            metadata: HashMap::new(),
            progress: None,
            result: None,
        }
    }

    pub fn with_scheduled_time(mut self, scheduled_for: DateTime<Utc>) -> Self {
        self.scheduled_for = Some(scheduled_for);
        self
    }

    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.state,
            JobState::Completed | JobState::Failed | JobState::Cancelled
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.state,
            JobState::Queued | JobState::Processing | JobState::Retrying
        )
    }

    pub fn can_retry(&self) -> bool {
        self.attempts < self.max_attempts && !self.is_terminal()
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        if let (Some(started), Some(completed)) = (self.started_at, self.completed_at) {
            Some(completed - started)
        } else if let Some(started) = self.started_at {
            Some(Utc::now() - started)
        } else {
            None
        }
    }
}

/// Trait for jobs that can be serialized and stored in the queue
pub trait SerializableJob: Send + Sync {
    /// Get the job type identifier
    fn job_type(&self) -> &'static str;
    
    /// Serialize the job data to JSON
    fn serialize(&self) -> Result<serde_json::Value, serde_json::Error>;
    
    /// Deserialize job data from JSON
    fn deserialize(data: &serde_json::Value) -> Result<Box<dyn SerializableJob>, serde_json::Error>
    where
        Self: Sized;
    
    /// Get job priority
    fn priority(&self) -> JobPriority {
        JobPriority::Normal
    }
    
    /// Get maximum retry attempts
    fn max_attempts(&self) -> u32 {
        3
    }
    
    /// Get delay before retry (in seconds)
    fn retry_delay(&self, attempt: u32) -> u64 {
        // Exponential backoff: 2^attempt seconds (max 300 seconds = 5 minutes)
        2_u64.pow(attempt).min(300)
    }
    
    /// Get scheduled execution time (for delayed jobs)
    fn scheduled_for(&self) -> Option<DateTime<Utc>> {
        None
    }
    
    /// Get job timeout (in seconds)
    fn timeout(&self) -> Option<u64> {
        Some(300) // 5 minutes default
    }
    
    /// Get additional metadata for the job
    fn metadata(&self) -> HashMap<String, serde_json::Value> {
        HashMap::new()
    }
}

/// Wrapper for storing jobs in the queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedJob {
    pub id: JobId,
    pub job_type: String,
    pub priority: JobPriority,
    pub data: serde_json::Value,
    pub status: JobStatus,
}

impl QueuedJob {
    pub fn new(job: &dyn SerializableJob) -> Result<Self, serde_json::Error> {
        let id = JobId::new();
        let job_type = job.job_type().to_string();
        let priority = job.priority();
        let data = job.serialize()?;
        
        let mut status = JobStatus::new(id.clone(), &job_type, priority)
            .with_max_attempts(job.max_attempts());
        
        if let Some(scheduled_for) = job.scheduled_for() {
            status = status.with_scheduled_time(scheduled_for);
        }
        
        // Add job metadata to status
        for (key, value) in job.metadata() {
            status = status.with_metadata(key, value);
        }
        
        Ok(Self {
            id,
            job_type,
            priority,
            data,
            status,
        })
    }

    pub fn is_ready_to_run(&self) -> bool {
        match self.status.state {
            JobState::Queued | JobState::Retrying => {
                if let Some(scheduled_for) = self.status.scheduled_for {
                    Utc::now() >= scheduled_for
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    pub fn mark_processing(&mut self) {
        self.status.state = JobState::Processing;
        self.status.started_at = Some(Utc::now());
        self.status.attempts += 1;
    }

    pub fn mark_completed(&mut self, result: Option<serde_json::Value>) {
        self.status.state = JobState::Completed;
        self.status.completed_at = Some(Utc::now());
        self.status.result = result;
        self.status.progress = Some(1.0);
    }

    pub fn mark_failed(&mut self, error: String) {
        if self.status.can_retry() {
            self.status.state = JobState::Retrying;
            // Schedule retry with exponential backoff
            let delay_seconds = 2_u64.pow(self.status.attempts).min(300);
            self.status.scheduled_for = Some(Utc::now() + chrono::Duration::seconds(delay_seconds as i64));
        } else {
            self.status.state = JobState::Failed;
            self.status.completed_at = Some(Utc::now());
        }
        self.status.last_error = Some(error);
    }

    pub fn mark_cancelled(&mut self) {
        self.status.state = JobState::Cancelled;
        self.status.completed_at = Some(Utc::now());
    }

    pub fn update_progress(&mut self, progress: f64) {
        self.status.progress = Some(progress.max(0.0).min(1.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_id_generation() {
        let id1 = JobId::new();
        let id2 = JobId::new();
        
        assert_ne!(id1, id2);
        assert!(!id1.as_str().is_empty());
    }

    #[test]
    fn test_job_priority_ordering() {
        assert!(JobPriority::Critical > JobPriority::High);
        assert!(JobPriority::High > JobPriority::Normal);
        assert!(JobPriority::Normal > JobPriority::Low);
    }

    #[test]
    fn test_job_status_states() {
        let id = JobId::new();
        let status = JobStatus::new(id, "test_job", JobPriority::Normal);
        
        assert_eq!(status.state, JobState::Queued);
        assert!(status.is_active());
        assert!(!status.is_terminal());
        assert!(status.can_retry());
    }

    #[test]
    fn test_queued_job_ready_to_run() {
        // Test immediate execution
        struct TestJob;
        impl SerializableJob for TestJob {
            fn job_type(&self) -> &'static str { "test" }
            fn serialize(&self) -> Result<serde_json::Value, serde_json::Error> {
                Ok(serde_json::json!({}))
            }
            fn deserialize(_data: &serde_json::Value) -> Result<Box<dyn SerializableJob>, serde_json::Error> {
                Ok(Box::new(TestJob))
            }
        }
        
        let job = TestJob;
        let queued_job = QueuedJob::new(&job).unwrap();
        assert!(queued_job.is_ready_to_run());
    }
}