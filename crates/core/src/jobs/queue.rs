use super::traits::{JobQueue, QueueStats};
use super::types::{JobId, JobState, JobStatus, QueuedJob};
use crate::error::{Error, ErrorCode, Result};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde_json;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Redis-backed job queue implementation
pub struct RedisJobQueue {
    redis: ConnectionManager,
    queue_name: String,
    processing_set: String,
    job_data_prefix: String,
    stats_key: String,
}

impl RedisJobQueue {
    pub fn new(redis: ConnectionManager, queue_name: impl Into<String>) -> Self {
        let queue_name = queue_name.into();
        Self {
            redis,
            processing_set: format!("{}:processing", queue_name),
            job_data_prefix: format!("{}:job:", queue_name),
            stats_key: format!("{}:stats", queue_name),
            queue_name,
        }
    }

    /// Get Redis key for job data
    fn job_key(&self, job_id: &JobId) -> String {
        format!("{}{}", self.job_data_prefix, job_id.as_str())
    }

    /// Get queue key for specific priority
    fn priority_queue_key(&self, priority: super::types::JobPriority) -> String {
        format!("{}:priority:{}", self.queue_name, priority as u8)
    }

    /// Get delayed jobs key
    fn delayed_jobs_key(&self) -> String {
        format!("{}:delayed", self.queue_name)
    }

    /// Store job data in Redis
    async fn store_job_data(&self, job: &QueuedJob) -> Result<()> {
        let mut conn = self.redis.clone();
        let job_json = serde_json::to_string(job)
            .map_err(|e| Error::new(ErrorCode::SerializationError, e.to_string()))?;

        // Store job data
        conn.set_ex::<_, _, ()>(&self.job_key(&job.id), job_json, 86400 * 7) // Keep for 7 days
            .await?;

        // Update statistics
        conn.hincr::<_, _, _, ()>(&self.stats_key, "total_jobs", 1).await?;
        conn.hincr::<_, _, _, ()>(&self.stats_key, "queued_jobs", 1).await?;

        debug!("Stored job data for {}", job.id);
        Ok(())
    }

    /// Load job data from Redis
    async fn load_job_data(&self, job_id: &JobId) -> Result<Option<QueuedJob>> {
        let mut conn = self.redis.clone();
        let job_json: Option<String> = conn.get(&self.job_key(job_id)).await?;

        match job_json {
            Some(json) => {
                let job: QueuedJob = serde_json::from_str(&json)
                    .map_err(|e| Error::new(ErrorCode::SerializationError, e.to_string()))?;
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }

    /// Move delayed jobs to ready queues
    async fn process_delayed_jobs(&self) -> Result<u32> {
        let mut conn = self.redis.clone();
        let now_timestamp = Utc::now().timestamp();

        // Get jobs that are ready to run
        let ready_jobs: Vec<String> = conn
            .zrangebyscore_limit(
                &self.delayed_jobs_key(),
                0,
                now_timestamp,
                0,
                100, // Process up to 100 delayed jobs at once
            )
            .await?;

        if ready_jobs.is_empty() {
            return Ok(0);
        }

        let mut moved_count = 0;
        for job_id_str in ready_jobs {
            let job_id = JobId::from_string(job_id_str);
            
            if let Some(mut job) = self.load_job_data(&job_id).await? {
                // Remove from delayed set
                let _: u32 = conn.zrem(&self.delayed_jobs_key(), job_id.as_str()).await?;
                
                // Update job status
                job.status.state = JobState::Queued;
                job.status.scheduled_for = None;
                
                // Add to appropriate priority queue
                let queue_key = self.priority_queue_key(job.priority);
                conn.lpush::<_, _, ()>(&queue_key, job_id.as_str()).await?;
                
                // Update job data
                self.store_job_data(&job).await?;
                moved_count += 1;
            }
        }

        if moved_count > 0 {
            info!("Moved {} delayed jobs to ready queues", moved_count);
        }

        Ok(moved_count)
    }

    /// Clean up stale processing jobs
    async fn cleanup_stale_processing_jobs(&self, timeout_seconds: u64) -> Result<u32> {
        let mut conn = self.redis.clone();
        let cutoff_time = Utc::now() - Duration::seconds(timeout_seconds as i64);
        let cutoff_timestamp = cutoff_time.timestamp();

        // Get all processing jobs
        let processing_jobs: Vec<String> = conn.smembers(&self.processing_set).await?;
        let mut cleaned_count = 0;

        for job_id_str in processing_jobs {
            let job_id = JobId::from_string(job_id_str);
            
            if let Some(job) = self.load_job_data(&job_id).await? {
                if let Some(started_at) = job.status.started_at {
                    if started_at.timestamp() < cutoff_timestamp {
                        warn!("Cleaning up stale processing job: {}", job_id);
                        
                        // Remove from processing set
                        let _: u32 = conn.srem(&self.processing_set, job_id.as_str()).await?;
                        
                        // Requeue the job for retry
                        self.requeue_job_for_retry(&job).await?;
                        cleaned_count += 1;
                    }
                }
            }
        }

        if cleaned_count > 0 {
            warn!("Cleaned up {} stale processing jobs", cleaned_count);
        }

        Ok(cleaned_count)
    }

    /// Requeue job for retry
    async fn requeue_job_for_retry(&self, job: &QueuedJob) -> Result<()> {
        let mut job = job.clone();
        
        if job.status.can_retry() {
            job.status.state = JobState::Retrying;
            
            // Calculate retry delay
            let delay_seconds = 2_u64.pow(job.status.attempts).min(300); // Max 5 minutes
            job.status.scheduled_for = Some(Utc::now() + Duration::seconds(delay_seconds as i64));
            
            // Add to delayed jobs
            let mut conn = self.redis.clone();
            let delayed_timestamp = job.status.scheduled_for.unwrap().timestamp();
            conn.zadd::<_, _, _, ()>(
                &self.delayed_jobs_key(),
                job.id.as_str(),
                delayed_timestamp,
            ).await?;
            
            // Update job data
            self.store_job_data(&job).await?;
            
            // Update stats
            conn.hincr::<_, _, _, ()>(&self.stats_key, "retrying_jobs", 1).await?;
            conn.hincr::<_, _, _, ()>(&self.stats_key, "processing_jobs", -1).await?;
        } else {
            // Mark as failed
            job.status.state = JobState::Failed;
            job.status.completed_at = Some(Utc::now());
            
            // Update job data
            self.store_job_data(&job).await?;
            
            // Update stats
            let mut conn = self.redis.clone();
            conn.hincr::<_, _, _, ()>(&self.stats_key, "failed_jobs", 1).await?;
            conn.hincr::<_, _, _, ()>(&self.stats_key, "processing_jobs", -1).await?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl JobQueue for RedisJobQueue {
    async fn enqueue(&self, job: QueuedJob) -> Result<JobId> {
        // Store job data first
        self.store_job_data(&job).await?;
        
        let mut conn = self.redis.clone();
        
        if let Some(scheduled_for) = job.status.scheduled_for {
            // Add to delayed jobs sorted set
            let timestamp = scheduled_for.timestamp();
            conn.zadd::<_, _, _, ()>(&self.delayed_jobs_key(), job.id.as_str(), timestamp)
                .await?;
            debug!("Enqueued delayed job {} for {}", job.id, scheduled_for);
        } else {
            // Add to appropriate priority queue
            let queue_key = self.priority_queue_key(job.priority);
            conn.lpush::<_, _, ()>(&queue_key, job.id.as_str()).await?;
            debug!("Enqueued immediate job {} with priority {:?}", job.id, job.priority);
        }
        
        info!("Enqueued job {} of type {}", job.id, job.job_type);
        Ok(job.id)
    }

    async fn dequeue(&self, worker_id: &str) -> Result<Option<QueuedJob>> {
        // Process delayed jobs first
        self.process_delayed_jobs().await?;
        
        let mut conn = self.redis.clone();
        
        // Try to dequeue from priority queues (highest priority first)
        let priorities = [
            super::types::JobPriority::Critical,
            super::types::JobPriority::High,
            super::types::JobPriority::Normal,
            super::types::JobPriority::Low,
        ];
        
        for priority in &priorities {
            let queue_key = self.priority_queue_key(*priority);
            
            // Atomic right-pop from queue and add to processing set
            let job_id_opt: Option<String> = conn.rpop(&queue_key, None).await?;
            
            if let Some(job_id_str) = job_id_opt {
                let job_id = JobId::from_string(job_id_str);
                
                if let Some(mut job) = self.load_job_data(&job_id).await? {
                    // Verify job is still in correct state
                    if !job.is_ready_to_run() {
                        continue;
                    }
                    
                    // Add to processing set with timestamp
                    conn.sadd::<_, _, ()>(&self.processing_set, job_id.as_str()).await?;
                    
                    // Update job status
                    job.mark_processing();
                    
                    // Store updated job data
                    self.store_job_data(&job).await?;
                    
                    // Update statistics
                    conn.hincr::<_, _, _, ()>(&self.stats_key, "queued_jobs", -1).await?;
                    conn.hincr::<_, _, _, ()>(&self.stats_key, "processing_jobs", 1).await?;
                    
                    debug!("Dequeued job {} for worker {}", job.id, worker_id);
                    return Ok(Some(job));
                }
            }
        }
        
        Ok(None)
    }

    async fn get_status(&self, job_id: &JobId) -> Result<Option<JobStatus>> {
        if let Some(job) = self.load_job_data(job_id).await? {
            Ok(Some(job.status))
        } else {
            Ok(None)
        }
    }

    async fn update_status(&self, job_id: &JobId, status: JobStatus) -> Result<()> {
        if let Some(mut job) = self.load_job_data(job_id).await? {
            let old_state = job.status.state;
            job.status = status;
            
            // Update statistics if state changed
            if old_state != job.status.state {
                let mut conn = self.redis.clone();
                
                match (old_state, job.status.state) {
                    (JobState::Processing, JobState::Completed) => {
                        conn.srem::<_, _, ()>(&self.processing_set, job_id.as_str()).await?;
                        conn.hincr::<_, _, _, ()>(&self.stats_key, "processing_jobs", -1).await?;
                        conn.hincr::<_, _, _, ()>(&self.stats_key, "completed_jobs", 1).await?;
                    }
                    (JobState::Processing, JobState::Failed) => {
                        conn.srem::<_, _, ()>(&self.processing_set, job_id.as_str()).await?;
                        conn.hincr::<_, _, _, ()>(&self.stats_key, "processing_jobs", -1).await?;
                        conn.hincr::<_, _, _, ()>(&self.stats_key, "failed_jobs", 1).await?;
                    }
                    (JobState::Processing, JobState::Retrying) => {
                        conn.srem::<_, _, ()>(&self.processing_set, job_id.as_str()).await?;
                        conn.hincr::<_, _, _, ()>(&self.stats_key, "processing_jobs", -1).await?;
                        conn.hincr::<_, _, _, ()>(&self.stats_key, "retrying_jobs", 1).await?;
                    }
                    _ => {}
                }
            }
            
            self.store_job_data(&job).await?;
            debug!("Updated status for job {}: {:?}", job_id, job.status.state);
        }
        
        Ok(())
    }

    async fn cancel_job(&self, job_id: &JobId) -> Result<bool> {
        if let Some(mut job) = self.load_job_data(job_id).await? {
            if job.status.is_terminal() {
                return Ok(false); // Already completed/failed/cancelled
            }
            
            job.mark_cancelled();
            
            let mut conn = self.redis.clone();
            
            // Remove from all possible locations
            for priority in &[
                super::types::JobPriority::Critical,
                super::types::JobPriority::High,
                super::types::JobPriority::Normal,
                super::types::JobPriority::Low,
            ] {
                let queue_key = self.priority_queue_key(*priority);
                let _: u32 = conn.lrem(&queue_key, 0, job_id.as_str()).await?;
            }
            
            let _: u32 = conn.zrem(&self.delayed_jobs_key(), job_id.as_str()).await?;
            let _: u32 = conn.srem(&self.processing_set, job_id.as_str()).await?;
            
            // Update job data
            self.store_job_data(&job).await?;
            
            // Update statistics
            conn.hincr::<_, _, _, ()>(&self.stats_key, "cancelled_jobs", 1).await?;
            
            info!("Cancelled job {}", job_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn get_stats(&self) -> Result<QueueStats> {
        let mut conn = self.redis.clone();
        
        // Get basic counters
        let stats_map: HashMap<String, i64> = conn.hgetall(&self.stats_key).await?;
        
        let total_jobs = stats_map.get("total_jobs").copied().unwrap_or(0) as u64;
        let queued_jobs = stats_map.get("queued_jobs").copied().unwrap_or(0) as u64;
        let processing_jobs = stats_map.get("processing_jobs").copied().unwrap_or(0) as u64;
        let completed_jobs = stats_map.get("completed_jobs").copied().unwrap_or(0) as u64;
        let failed_jobs = stats_map.get("failed_jobs").copied().unwrap_or(0) as u64;
        let retrying_jobs = stats_map.get("retrying_jobs").copied().unwrap_or(0) as u64;
        let cancelled_jobs = stats_map.get("cancelled_jobs").copied().unwrap_or(0) as u64;
        
        // Calculate error rate
        let error_rate = if total_jobs > 0 {
            Some(failed_jobs as f64 / total_jobs as f64)
        } else {
            None
        };
        
        Ok(QueueStats {
            total_jobs,
            queued_jobs,
            processing_jobs,
            completed_jobs,
            failed_jobs,
            retrying_jobs,
            cancelled_jobs,
            average_processing_time: None, // Could be calculated from job durations
            jobs_per_minute: None,         // Could be calculated from timestamps
            error_rate,
        })
    }

    async fn cleanup_old_jobs(&self, older_than: DateTime<Utc>) -> Result<u64> {
        // This is a simplified implementation
        // In a real system, you'd scan through completed/failed jobs and remove old ones
        let _cutoff_timestamp = older_than.timestamp();
        let mut conn = self.redis.clone();
        
        // Clean up old delayed jobs that are way past their execution time
        let cleaned: u32 = conn
            .zrem(&self.delayed_jobs_key(), "old_jobs")
            .await?;
        
        info!("Cleaned up {} old jobs", cleaned);
        Ok(cleaned as u64)
    }

    async fn get_jobs_by_status(&self, status: JobState, _limit: Option<u32>) -> Result<Vec<QueuedJob>> {
        // This would require scanning through stored job data
        // For now, return empty vec - in a real implementation you'd use a secondary index
        debug!("get_jobs_by_status called for {:?} (not implemented)", status);
        Ok(Vec::new())
    }

    async fn health_check(&self) -> Result<bool> {
        let mut conn = self.redis.clone();
        let _: String = conn.get("ping").await.unwrap_or("PONG".to_string());
        
        // Clean up stale processing jobs
        self.cleanup_stale_processing_jobs(3600).await?; // 1 hour timeout
        
        Ok(true)
    }
}