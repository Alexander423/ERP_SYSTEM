use crate::config::MetricsConfig;
use prometheus::{Encoder, Registry, TextEncoder};
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

/// Centralized metrics registry for the ERP system
#[derive(Debug, Clone)]
pub struct MetricsRegistry {
    registry: Arc<Mutex<Registry>>,
    config: MetricsConfig,
}

impl MetricsRegistry {
    pub fn new(config: MetricsConfig) -> Self {
        let registry = Registry::new_custom(
            Some(config.namespace.clone()),
            None,
        ).unwrap_or_else(|_| {
            warn!("Failed to create custom registry, using default");
            Registry::new()
        });

        Self {
            registry: Arc::new(Mutex::new(registry)),
            config,
        }
    }

    pub fn register<T>(&self, collector: T) -> Result<(), prometheus::Error>
    where
        T: prometheus::core::Collector + 'static,
    {
        if let Ok(registry) = self.registry.lock() {
            registry.register(Box::new(collector))
        } else {
            Err(prometheus::Error::Msg("Failed to acquire registry lock".to_string()))
        }
    }

    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        if let Ok(registry) = self.registry.lock() {
            registry.gather()
        } else {
            Vec::new()
        }
    }

    pub fn metrics_text(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.gather();
        
        let mut buffer = Vec::new();
        if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
            warn!("Failed to encode metrics: {}", e);
            return String::new();
        }

        String::from_utf8(buffer).unwrap_or_else(|e| {
            warn!("Failed to convert metrics to UTF-8: {}", e);
            String::new()
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Metrics service for HTTP endpoints
pub struct MetricsService {
    registry: MetricsRegistry,
}

impl MetricsService {
    pub fn new(registry: MetricsRegistry) -> Self {
        if registry.is_enabled() {
            info!(
                "Metrics service initialized on port {}, path: {}", 
                registry.config.port, 
                registry.config.path
            );
        }
        
        Self { registry }
    }

    pub fn get_metrics(&self) -> String {
        if !self.registry.is_enabled() {
            return "# Metrics disabled\n".to_string();
        }

        let mut response = format!(
            "# HELP erp_metrics_info ERP System metrics information\n# TYPE erp_metrics_info gauge\nerp_metrics_info{{version=\"{}\"}} 1\n",
            env!("CARGO_PKG_VERSION")
        );
        
        response.push_str(&self.registry.metrics_text());
        response
    }

    pub fn registry(&self) -> &MetricsRegistry {
        &self.registry
    }
}