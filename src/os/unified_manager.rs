//! Unified OS Manager (GOD-LEVEL INTEGRATION)
//!
//! Brings together all Phase 5 enhancements into a single, cohesive API.
//! This is the main entry point for all OS integration features.

use crate::error::Result;
use crate::events::EventBus;
use crate::metrics::Metrics;
use std::sync::Arc;

use super::{
    automation::AutomationEngine,
    clipboard_manager::ClipboardManager,
    desktop_environment::DesktopEnvironment,
    health_monitor::HealthMonitor,
    multi_monitor::MultiMonitorManager,
    performance_optimizer::PerformanceOptimizer,
    power_manager::PowerManager,
    process_manager::ProcessManager,
    process_priority::ProcessOptimizer,
    resource_monitor::ResourceMonitor,
    smart_app_index::SmartAppIndex,
    virtual_desktop::VirtualDesktopManager,
    window_manager::WindowManager,
};

/// Unified OS Manager - Single entry point for all OS integration
pub struct UnifiedOsManager {
    // P0: Critical Features
    pub process_manager: Arc<ProcessManager>,
    pub multi_monitor: Arc<MultiMonitorManager>,
    pub resource_monitor: Arc<ResourceMonitor>,

    // P1: High-Impact Features
    pub window_manager: Arc<WindowManager>,
    pub app_index: Arc<SmartAppIndex>,
    pub automation: Arc<AutomationEngine>,
    pub health_monitor: Arc<HealthMonitor>,

    // P2: Important Features
    pub power_manager: Arc<PowerManager>,
    pub virtual_desktop: Arc<VirtualDesktopManager>,
    pub process_optimizer: Arc<ProcessOptimizer>,
    pub clipboard_manager: Arc<ClipboardManager>,

    // P3: Supporting Features
    pub desktop_environment: Option<DesktopEnvironment>,
    pub performance_optimizer: Arc<PerformanceOptimizer>,

    // Event bus and metrics
    event_bus: Option<Arc<EventBus>>,
    metrics: Option<Arc<Metrics>>,
}

impl UnifiedOsManager {
    /// Create a new unified OS manager
    pub fn new() -> Self {
        Self {
            process_manager: Arc::new(ProcessManager::new()),
            multi_monitor: Arc::new(MultiMonitorManager::new()),
            resource_monitor: Arc::new(ResourceMonitor::new()),
            window_manager: Arc::new(WindowManager::new()),
            app_index: Arc::new(SmartAppIndex::new()),
            automation: Arc::new(AutomationEngine::new()),
            health_monitor: Arc::new(HealthMonitor::new()),
            power_manager: Arc::new(PowerManager::new()),
            virtual_desktop: Arc::new(VirtualDesktopManager::new()),
            process_optimizer: Arc::new(ProcessOptimizer::new()),
            clipboard_manager: Arc::new(ClipboardManager::new(100)),
            desktop_environment: None,
            performance_optimizer: Arc::new(PerformanceOptimizer::new()),
            event_bus: None,
            metrics: None,
        }
    }

    /// Add event bus to all components
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus.clone());
        
        // Configure components with event bus
        self.process_manager = Arc::new(
            ProcessManager::new().with_event_bus(event_bus.clone())
        );
        self.multi_monitor = Arc::new(
            MultiMonitorManager::new().with_event_bus(event_bus.clone())
        );
        self.resource_monitor = Arc::new(
            ResourceMonitor::new().with_event_bus(event_bus.clone())
        );
        self.health_monitor = Arc::new(
            HealthMonitor::new().with_event_bus(event_bus.clone())
        );

        self
    }

    /// Add metrics to all components
    pub fn with_metrics(mut self, metrics: Arc<Metrics>) -> Self {
        self.metrics = Some(metrics.clone());
        
        // Configure components with metrics
        self.process_manager = Arc::new(
            ProcessManager::new().with_metrics(metrics.clone())
        );
        self.resource_monitor = Arc::new(
            ResourceMonitor::new().with_metrics(metrics.clone())
        );

        self
    }

    /// Initialize all components
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing Unified OS Manager with all god-level enhancements");

        // Detect desktop environment
        self.desktop_environment = Some(DesktopEnvironment::detect().await?);
        tracing::info!("Desktop environment detected: {:?}", self.desktop_environment.as_ref().unwrap().de_type);

        // Detect displays
        self.multi_monitor.detect_displays().await?;
        
        // Start monitoring services
        self.process_manager.start_monitoring().await?;
        self.resource_monitor.start_monitoring().await?;
        self.health_monitor.start_monitoring().await?;

        tracing::info!("✅ All Phase 5 God-Level enhancements initialized successfully");
        
        Ok(())
    }

    /// Shutdown all components gracefully
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down Unified OS Manager");

        self.process_manager.stop_monitoring().await;
        self.resource_monitor.stop_monitoring().await;
        self.health_monitor.stop_monitoring().await;

        tracing::info!("✅ Shutdown complete");
        Ok(())
    }

    /// Get system health status
    pub async fn get_system_health(&self) -> SystemHealth {
        let metrics = self.resource_monitor.get_current_metrics().await;
        let issues = self.health_monitor.detect_issues().await;
        let (total_apps, total_launches) = self.app_index.get_stats().await;

        SystemHealth {
            cpu_usage: metrics.cpu_total,
            memory_used_mb: metrics.memory_used_mb,
            memory_total_mb: metrics.memory_total_mb,
            active_processes: self.process_manager.list_processes().len(),
            health_issues: issues.len(),
            indexed_apps: total_apps,
            total_launches,
        }
    }
}

impl Default for UnifiedOsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// System health summary
#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub cpu_usage: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub active_processes: usize,
    pub health_issues: usize,
    pub indexed_apps: usize,
    pub total_launches: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_manager_creation() {
        let manager = UnifiedOsManager::new();
        assert!(manager.event_bus.is_none());
    }

    #[tokio::test]
    async fn test_initialization() {
        let mut manager = UnifiedOsManager::new();
        let result = manager.initialize().await;
        assert!(result.is_ok());
        
        manager.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_system_health() {
        let mut manager = UnifiedOsManager::new();
        manager.initialize().await.unwrap();
        
        let health = manager.get_system_health().await;
        assert!(health.cpu_usage >= 0.0);
        
        manager.shutdown().await.unwrap();
    }
}
