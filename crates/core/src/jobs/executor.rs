use super::{
    traits::{JobContext, JobHandler, JobQueue, JobResult},
    types::{JobId, JobState, QueuedJob},
};
use crate::error::{Error, ErrorCode, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// Configuration for the job executor
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    pub worker_id: String,
    pub max_concurrent_jobs: usize,
    pub poll_interval: Duration,
    pub job_timeout: Duration,
    pub shutdown_timeout: Duration,
    pub enable_metrics: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            worker_id: format!("worker-{}", uuid::Uuid::new_v4()),
            max_concurrent_jobs: 10,
            poll_interval: Duration::from_secs(1),
            job_timeout: Duration::from_secs(300), // 5 minutes
            shutdown_timeout: Duration::from_secs(30),
            enable_metrics: true,
        }
    }
}

/// Job executor that processes jobs from a queue
pub struct JobExecutor {
    queue: Arc<dyn JobQueue>,
    handlers: Arc<RwLock<HashMap<String, Arc<dyn JobHandler>>>>,
    config: ExecutorConfig,
    shutdown_tx: Option<mpsc::Sender<()>>,
    semaphore: Arc<Semaphore>,
    metrics: Arc<RwLock<ExecutorMetrics>>,
}

#[derive(Debug, Default)]
struct ExecutorMetrics {
    jobs_processed: u64,
    jobs_succeeded: u64,
    jobs_failed: u64,
    jobs_retried: u64,
    total_processing_time: Duration,
    active_jobs: u64,
}

impl ExecutorMetrics {
    fn success_rate(&self) -> f64 {
        if self.jobs_processed == 0 {
            0.0
        } else {
            self.jobs_succeeded as f64 / self.jobs_processed as f64
        }
    }

    fn average_processing_time(&self) -> Duration {
        if self.jobs_processed == 0 {
            Duration::ZERO
        } else {
            self.total_processing_time / self.jobs_processed as u32
        }
    }
}

impl JobExecutor {
    /// Create a new job executor
    pub fn new(queue: Arc<dyn JobQueue>, config: ExecutorConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_jobs));
        
        Self {
            queue,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            config,
            shutdown_tx: None,
            semaphore,
            metrics: Arc::new(RwLock::new(ExecutorMetrics::default())),
        }
    }

    /// Register a job handler for a specific job type
    pub async fn register_handler(&self, handler: Arc<dyn JobHandler>) {
        let job_type = handler.job_type().to_string();
        let mut handlers = self.handlers.write().await;
        handlers.insert(job_type.clone(), handler);
        info!("Registered handler for job type: {}", job_type);
    }

    /// Start the executor (non-blocking)
    pub async fn start(&mut self) -> Result<()> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let queue = Arc::clone(&self.queue);
        let handlers = Arc::clone(&self.handlers);
        let config = self.config.clone();
        let semaphore = Arc::clone(&self.semaphore);
        let metrics = Arc::clone(&self.metrics);

        tokio::spawn(async move {
            Self::worker_loop(queue, handlers, config, semaphore, metrics, shutdown_rx).await;
        });

        info!("Job executor started with worker ID: {}", self.config.worker_id);
        Ok(())
    }

    /// Stop the executor gracefully
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
            info!("Job executor shutdown signal sent");
        }
        Ok(())
    }

    /// Main worker loop
    async fn worker_loop(
        queue: Arc<dyn JobQueue>,
        handlers: Arc<RwLock<HashMap<String, Arc<dyn JobHandler>>>>,
        config: ExecutorConfig,
        semaphore: Arc<Semaphore>,
        metrics: Arc<RwLock<ExecutorMetrics>>,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) {
        info!("Worker loop started: {}", config.worker_id);
        
        let mut poll_interval = tokio::time::interval(config.poll_interval);
        poll_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received, stopping worker loop");
                    break;
                }
                _ = poll_interval.tick() => {
                    if let Err(e) = Self::process_next_job(
                        Arc::clone(&queue),
                        Arc::clone(&handlers),
                        config.clone(),
                        Arc::clone(&semaphore),
                        Arc::clone(&metrics),
                    ).await {
                        error!("Error processing job: {}", e);
                    }
                }
            }
        }

        // Wait for active jobs to complete
        info!("Waiting for active jobs to complete...");
        let shutdown_timeout = timeout(
            config.shutdown_timeout,
            Self::wait_for_active_jobs(&semaphore, config.max_concurrent_jobs),
        );

        match shutdown_timeout.await {
            Ok(_) => info!("All jobs completed successfully"),
            Err(_) => warn!("Shutdown timeout reached, some jobs may have been interrupted"),
        }

        info!("Worker loop stopped: {}", config.worker_id);
    }

    /// Process the next available job
    async fn process_next_job(
        queue: Arc<dyn JobQueue>,
        handlers: Arc<RwLock<HashMap<String, Arc<dyn JobHandler>>>>,
        config: ExecutorConfig,
        semaphore: Arc<Semaphore>,
        metrics: Arc<RwLock<ExecutorMetrics>>,
    ) -> Result<()> {
        // Try to dequeue a job
        match queue.dequeue(&config.worker_id).await {
            Ok(Some(job)) => {
                let job_id = job.id.clone();
                let handlers_clone = Arc::clone(&handlers);
                let queue_clone = Arc::clone(&queue);
                let config_clone = config.clone();
                let metrics_clone = Arc::clone(&metrics);
                let semaphore_clone = Arc::clone(&semaphore);

                // Process job in background task
                tokio::spawn(async move {
                    // Try to acquire a permit (non-blocking) - if we can't, skip this job
                    let _permit = match semaphore_clone.try_acquire() {
                        Ok(permit) => permit,
                        Err(_) => {
                            debug!("All worker slots busy, skipping job {}", job_id);
                            // TODO: Could implement proper requeueing here
                            return;
                        }
                    };
                    
                    let start_time = std::time::Instant::now();
                    let result = Self::execute_job(job, &handlers_clone, &config_clone).await;
                    let duration = start_time.elapsed();

                    // Update metrics
                    {
                        let mut m = metrics_clone.write().await;
                        m.jobs_processed += 1;
                        m.total_processing_time += duration;
                        m.active_jobs = m.active_jobs.saturating_sub(1);

                        match &result {
                            JobResult::Success { .. } => m.jobs_succeeded += 1,
                            JobResult::Failed { .. } => m.jobs_failed += 1,
                            JobResult::Retry { .. } => m.jobs_retried += 1,
                            JobResult::Cancelled { .. } => {} // Don't count as success or failure
                        }
                    }

                    // Update job status in queue
                    if let Err(e) = Self::handle_job_result(&queue_clone, &job_id, result).await {
                        error!("Failed to update job status for {}: {}", job_id, e);
                    }
                });

                // Update active jobs count
                {
                    let mut m = metrics.write().await;
                    m.active_jobs += 1;
                }
            }
            Ok(None) => {
                debug!("No jobs available in queue");
            }
            Err(e) => {
                error!("Failed to dequeue job: {}", e);
            }
        }

        Ok(())
    }

    /// Execute a specific job
    async fn execute_job(
        job: QueuedJob,
        handlers: &Arc<RwLock<HashMap<String, Arc<dyn JobHandler>>>>,
        config: &ExecutorConfig,
    ) -> JobResult {
        let job_id = job.id.clone();
        debug!("Executing job: {} (type: {})", job_id, job.job_type);

        // Get handler for this job type
        let handler = {
            let handlers_guard = handlers.read().await;
            match handlers_guard.get(&job.job_type) {
                Some(handler) => Arc::clone(handler),
                None => {
                    error!("No handler registered for job type: {}", job.job_type);
                    return JobResult::failed(format!("No handler for job type: {}", job.job_type));
                }
            }
        };

        // Validate job data
        if let Err(e) = handler.validate_job_data(&job.data) {
            error!("Job data validation failed for {}: {}", job_id, e);
            return JobResult::failed(format!("Job data validation failed: {}", e));
        }

        // Create job context
        let context = JobContext::new(
            job.id.clone(),
            job.status.attempts,
            job.status.max_attempts,
        ).with_metadata(job.status.metadata.clone());

        // Execute job with timeout
        let job_timeout = Duration::from_secs(
            handler.config().default_timeout.unwrap_or(config.job_timeout.as_secs())
        );

        let execution_future = handler.handle(&job.data, &context);
        
        match timeout(job_timeout, execution_future).await {
            Ok(result) => {
                info!("Job {} completed with result: {:?}", job_id, 
                      if result.is_success() { "Success" } else { "Failed" });
                result
            }
            Err(_) => {
                error!("Job {} timed out after {:?}", job_id, job_timeout);
                JobResult::failed("Job execution timed out".to_string())
            }
        }
    }

    /// Handle job execution result
    async fn handle_job_result(
        queue: &Arc<dyn JobQueue>,
        job_id: &JobId,
        result: JobResult,
    ) -> Result<()> {
        // Get current job status
        let mut status = queue
            .get_status(job_id)
            .await?
            .ok_or_else(|| Error::new(ErrorCode::ResourceNotFound, "Job not found"))?;

        match result {
            JobResult::Success { result, message } => {
                status.state = JobState::Completed;
                status.completed_at = Some(chrono::Utc::now());
                status.result = result;
                status.progress = Some(1.0);
                
                if let Some(msg) = message {
                    status.metadata.insert(
                        "completion_message".to_string(),
                        serde_json::Value::String(msg),
                    );
                }
            }
            
            JobResult::Retry { error, delay_seconds } => {
                if status.can_retry() {
                    status.state = JobState::Retrying;
                    status.last_error = Some(error);
                    
                    let delay = delay_seconds.unwrap_or_else(|| {
                        2_u64.pow(status.attempts).min(300) // Exponential backoff, max 5 minutes
                    });
                    
                    status.scheduled_for = Some(
                        chrono::Utc::now() + chrono::Duration::seconds(delay as i64)
                    );
                } else {
                    status.state = JobState::Failed;
                    status.completed_at = Some(chrono::Utc::now());
                    status.last_error = Some(error);
                }
            }
            
            JobResult::Failed { error } => {
                status.state = JobState::Failed;
                status.completed_at = Some(chrono::Utc::now());
                status.last_error = Some(error);
            }
            
            JobResult::Cancelled { reason } => {
                status.state = JobState::Cancelled;
                status.completed_at = Some(chrono::Utc::now());
                status.metadata.insert(
                    "cancellation_reason".to_string(),
                    serde_json::Value::String(reason),
                );
            }
        }

        queue.update_status(job_id, status).await
    }

    /// Wait for all active jobs to complete
    async fn wait_for_active_jobs(semaphore: &Arc<Semaphore>, max_permits: usize) {
        // Try to acquire all permits, which means all jobs are done
        let _permits = semaphore.acquire_many(max_permits as u32).await.unwrap();
        debug!("All active jobs completed");
    }

    /// Get executor metrics
    pub async fn get_metrics(&self) -> ExecutorMetricsSnapshot {
        let metrics = self.metrics.read().await;
        ExecutorMetricsSnapshot {
            worker_id: self.config.worker_id.clone(),
            jobs_processed: metrics.jobs_processed,
            jobs_succeeded: metrics.jobs_succeeded,
            jobs_failed: metrics.jobs_failed,
            jobs_retried: metrics.jobs_retried,
            active_jobs: metrics.active_jobs,
            success_rate: metrics.success_rate(),
            average_processing_time: metrics.average_processing_time(),
        }
    }

    /// Get registered job types
    pub async fn get_registered_job_types(&self) -> Vec<String> {
        let handlers = self.handlers.read().await;
        handlers.keys().cloned().collect()
    }
}

/// Snapshot of executor metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecutorMetricsSnapshot {
    pub worker_id: String,
    pub jobs_processed: u64,
    pub jobs_succeeded: u64,
    pub jobs_failed: u64,
    pub jobs_retried: u64,
    pub active_jobs: u64,
    pub success_rate: f64,
    pub average_processing_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::traits::JobHandler;
    use async_trait::async_trait;

    struct TestJobHandler;

    #[async_trait]
    impl JobHandler for TestJobHandler {
        fn job_type(&self) -> &'static str {
            "test_job"
        }

        async fn handle(&self, _job_data: &serde_json::Value, _context: &JobContext) -> JobResult {
            JobResult::success()
        }

        fn validate_job_data(&self, _job_data: &serde_json::Value) -> Result<()> {
            Ok(())
        }
    }

    // Note: Full integration tests would require a real Redis instance
    // These are simplified unit tests for the configuration and setup
    
    #[tokio::test]
    async fn test_executor_creation() {
        // This test would need a mock queue implementation
        // For now, we just test the configuration
        let config = ExecutorConfig::default();
        assert!(!config.worker_id.is_empty());
        assert_eq!(config.max_concurrent_jobs, 10);
    }

    #[test]
    fn test_executor_metrics() {
        let metrics = ExecutorMetrics {
            jobs_processed: 100,
            jobs_succeeded: 95,
            jobs_failed: 5,
            jobs_retried: 3,
            total_processing_time: Duration::from_secs(1000),
            active_jobs: 2,
        };

        assert_eq!(metrics.success_rate(), 0.95);
        assert_eq!(metrics.average_processing_time(), Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_job_handler_functionality() {
        let handler = TestJobHandler;

        // Test job type
        assert_eq!(handler.job_type(), "test_job");

        // Test job data validation
        let job_data = serde_json::json!({"test": "data"});
        assert!(handler.validate_job_data(&job_data).is_ok());

        // Test job handling
        let context = JobContext {
            job_id: JobId::new(),
            attempt: 1,
            max_attempts: 3,
            metadata: HashMap::new(),
            started_at: chrono::Utc::now(),
        };

        let result = handler.handle(&job_data, &context).await;
        assert!(result.is_success());
    }

    #[test]
    fn test_job_handler_properties() {
        let handler = TestJobHandler;

        // Verify the handler implements the required traits
        assert_eq!(handler.job_type(), "test_job");

        // Test that validation passes for empty data
        let empty_data = serde_json::json!({});
        assert!(handler.validate_job_data(&empty_data).is_ok());

        // Test that validation passes for complex data
        let complex_data = serde_json::json!({
            "user_id": 123,
            "action": "test",
            "metadata": {
                "timestamp": "2023-01-01T00:00:00Z"
            }
        });
        assert!(handler.validate_job_data(&complex_data).is_ok());
    }
}