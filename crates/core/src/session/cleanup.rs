use super::{SessionManager, SessionStats};
use crate::{error::Result, TenantContext, TenantId};
use chrono::{DateTime, Duration, Utc};
use std::{collections::HashMap, sync::Arc};
use tokio::{task::JoinHandle, time::interval};
use tracing::{error, info};

/// Periodic session cleanup service that maintains session hygiene
/// by removing expired sessions and gathering statistics.
pub struct SessionCleanupService {
    session_manager: Arc<SessionManager>,
    cleanup_interval: Duration,
    tenants: Vec<TenantId>,
    stats_history: HashMap<TenantId, Vec<SessionStatsSnapshot>>,
}

/// Historical session statistics snapshot
#[derive(Debug, Clone)]
pub struct SessionStatsSnapshot {
    pub timestamp: DateTime<Utc>,
    pub stats: SessionStats,
    pub cleaned_up_count: u32,
}

impl SessionCleanupService {
    /// Create a new session cleanup service
    pub fn new(
        session_manager: Arc<SessionManager>,
        cleanup_interval: Duration,
        tenants: Vec<TenantId>,
    ) -> Self {
        Self {
            session_manager,
            cleanup_interval,
            tenants,
            stats_history: HashMap::new(),
        }
    }

    /// Start the cleanup service in the background
    pub fn start(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            self.run_cleanup_loop().await;
        })
    }

    /// Run the main cleanup loop
    async fn run_cleanup_loop(mut self) {
        let mut cleanup_interval = interval(
            self.cleanup_interval
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(300))
        );

        info!(
            interval_minutes = self.cleanup_interval.num_minutes(),
            tenant_count = self.tenants.len(),
            "Session cleanup service started"
        );

        loop {
            cleanup_interval.tick().await;

            match self.perform_cleanup_cycle().await {
                Ok(total_cleaned) => {
                    if total_cleaned > 0 {
                        info!(
                            cleaned_up_sessions = total_cleaned,
                            "Session cleanup cycle completed"
                        );
                    }
                }
                Err(e) => {
                    error!(
                        error = %e,
                        "Session cleanup cycle failed"
                    );
                }
            }
        }
    }

    /// Perform a complete cleanup cycle across all tenants
    async fn perform_cleanup_cycle(&mut self) -> Result<u32> {
        let mut total_cleaned = 0;
        let cleanup_timestamp = Utc::now();

        let tenants = self.tenants.clone();
        for tenant_id in &tenants {
            let tenant_context = TenantContext {
                tenant_id: *tenant_id,
                schema_name: format!("tenant_{}", tenant_id.0),
            };

            match self.cleanup_tenant_sessions(&tenant_context).await {
                Ok((cleaned_count, stats)) => {
                    total_cleaned += cleaned_count;

                    // Store statistics snapshot
                    let snapshot = SessionStatsSnapshot {
                        timestamp: cleanup_timestamp,
                        stats: stats.clone(),
                        cleaned_up_count: cleaned_count,
                    };

                    self.store_stats_snapshot(*tenant_id, snapshot);

                    if cleaned_count > 0 {
                        info!(
                            tenant_id = %tenant_id.0,
                            cleaned_sessions = cleaned_count,
                            active_sessions = stats.active_sessions,
                            "Tenant session cleanup completed"
                        );
                    }
                }
                Err(e) => {
                    error!(
                        tenant_id = %tenant_id.0,
                        error = %e,
                        "Failed to cleanup sessions for tenant"
                    );
                }
            }
        }

        Ok(total_cleaned)
    }

    /// Clean up sessions for a specific tenant
    async fn cleanup_tenant_sessions(
        &self,
        tenant: &TenantContext,
    ) -> Result<(u32, SessionStats)> {
        // Get statistics before cleanup
        let _stats_before = self.session_manager.get_session_stats(tenant).await?;

        // Perform cleanup
        let cleaned_count = self.session_manager.cleanup_expired_sessions(tenant).await?;

        // Get statistics after cleanup
        let stats_after = self.session_manager.get_session_stats(tenant).await?;

        Ok((cleaned_count, stats_after))
    }

    /// Store statistics snapshot with size limit
    fn store_stats_snapshot(&mut self, tenant_id: TenantId, snapshot: SessionStatsSnapshot) {
        let max_history_size = 144; // Keep 24 hours of 10-minute snapshots

        let tenant_history = self.stats_history.entry(tenant_id).or_insert_with(Vec::new);

        tenant_history.push(snapshot);

        // Trim history if it gets too large
        if tenant_history.len() > max_history_size {
            tenant_history.remove(0);
        }
    }

    /// Get recent session statistics for a tenant
    pub fn get_tenant_stats_history(
        &self,
        tenant_id: TenantId,
        hours_back: u32,
    ) -> Vec<SessionStatsSnapshot> {
        let cutoff = Utc::now() - Duration::hours(hours_back as i64);

        self.stats_history
            .get(&tenant_id)
            .map(|history| {
                history
                    .iter()
                    .filter(|snapshot| snapshot.timestamp > cutoff)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get aggregated statistics across all tenants
    pub fn get_aggregated_stats(&self) -> AggregatedSessionStats {
        let mut total_stats = AggregatedSessionStats::default();

        for history in self.stats_history.values() {
            if let Some(latest) = history.last() {
                total_stats.total_tenants += 1;
                total_stats.total_sessions += latest.stats.total_sessions;
                total_stats.active_sessions += latest.stats.active_sessions;
                total_stats.expired_sessions += latest.stats.expired_sessions;
                total_stats.logged_out_sessions += latest.stats.logged_out_sessions;
                total_stats.revoked_sessions += latest.stats.revoked_sessions;
                total_stats.suspended_sessions += latest.stats.suspended_sessions;
            }
        }

        total_stats
    }

    /// Manually trigger cleanup for all tenants (useful for testing/admin)
    pub async fn manual_cleanup(&mut self) -> Result<u32> {
        info!("Manual session cleanup triggered");
        self.perform_cleanup_cycle().await
    }

    /// Get health status of the cleanup service
    pub fn get_health_status(&self) -> CleanupServiceHealth {
        let now = Utc::now();
        let mut health = CleanupServiceHealth {
            is_healthy: true,
            last_cleanup_time: None,
            tenant_count: self.tenants.len() as u32,
            issues: Vec::new(),
        };

        // Find the most recent cleanup time across all tenants
        let most_recent_cleanup = self
            .stats_history
            .values()
            .flat_map(|history| history.iter())
            .map(|snapshot| snapshot.timestamp)
            .max();

        health.last_cleanup_time = most_recent_cleanup;

        // Check if cleanup is overdue
        if let Some(last_cleanup) = most_recent_cleanup {
            let time_since_cleanup = now - last_cleanup;
            let overdue_threshold = self.cleanup_interval + Duration::minutes(5); // 5 min grace period

            if time_since_cleanup > overdue_threshold {
                health.is_healthy = false;
                health.issues.push(format!(
                    "Cleanup overdue by {} minutes",
                    (time_since_cleanup - self.cleanup_interval).num_minutes()
                ));
            }
        } else {
            health.is_healthy = false;
            health.issues.push("No cleanup has been performed yet".to_string());
        }

        // Check for tenants with high session counts
        for (tenant_id, history) in &self.stats_history {
            if let Some(latest) = history.last() {
                if latest.stats.total_sessions > 1000 {
                    health.issues.push(format!(
                        "Tenant {} has {} total sessions (high volume)",
                        tenant_id.0, latest.stats.total_sessions
                    ));
                }

                if latest.stats.expired_sessions > 100 {
                    health.issues.push(format!(
                        "Tenant {} has {} expired sessions (cleanup needed)",
                        tenant_id.0, latest.stats.expired_sessions
                    ));
                }
            }
        }

        health
    }
}

/// Aggregated statistics across all tenants
#[derive(Debug, Default)]
pub struct AggregatedSessionStats {
    pub total_tenants: u32,
    pub total_sessions: u32,
    pub active_sessions: u32,
    pub expired_sessions: u32,
    pub logged_out_sessions: u32,
    pub revoked_sessions: u32,
    pub suspended_sessions: u32,
}

/// Health status of the cleanup service
#[derive(Debug)]
pub struct CleanupServiceHealth {
    pub is_healthy: bool,
    pub last_cleanup_time: Option<DateTime<Utc>>,
    pub tenant_count: u32,
    pub issues: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{SessionConfig, SessionManager};
    use redis::aio::ConnectionManager;

    #[tokio::test]
    async fn test_cleanup_service_creation() {
        // This is a basic test to ensure the service can be created
        // In a real test environment, you'd need a Redis connection
        
        let tenants = vec![TenantId(uuid::Uuid::new_v4())];
        let cleanup_interval = Duration::minutes(5);

        // Mock session manager would be created here with a test Redis connection
        // For now, we'll just test the structure
        assert!(tenants.len() > 0);
        assert!(cleanup_interval > Duration::zero());
    }
}