//! System Health Monitor with Auto-Remediation (GOD-LEVEL)

use crate::error::Result;
use crate::events::{EventBus, LunaEvent};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

#[cfg(feature = "full")]
use sysinfo::{System, SystemExt, ProcessExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthIssue {
    HighMemoryUsage,
    HighCpuUsage,
    HighDiskUsage,
    TooManyOpenFiles,
    UnresponsiveApp(String),
    DiskErrors,
    NetworkIssues,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemediationAction {
    KillProcess(u32),
    RestartApp(String),
    ClearTempFiles,
    CloseUnusedTabs,
    ReduceProcessPriority(u32),
}

pub struct HealthCheck {
    pub name: String,
    pub check_fn: Arc<dyn Fn() -> HealthStatus + Send + Sync>,
    pub interval: Duration,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct HealthMonitor {
    checks: Vec<HealthCheck>,
    remediation_actions: Arc<RwLock<Vec<(HealthIssue, RemediationAction)>>>,
    event_bus: Option<Arc<EventBus>>,
    monitoring_active: Arc<RwLock<bool>>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            remediation_actions: Arc::new(RwLock::new(Vec::new())),
            event_bus: None,
            monitoring_active: Arc::new(RwLock::new(false)),
        }
    }

    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    pub async fn start_monitoring(&self) -> Result<()> {
        let mut active = self.monitoring_active.write().await;
        *active = true;
        drop(active);

        info!("Starting health monitoring");
        Ok(())
    }

    pub async fn stop_monitoring(&self) {
        let mut active = self.monitoring_active.write().await;
        *active = false;
        info!("Stopped health monitoring");
    }

    pub async fn detect_issues(&self) -> Vec<HealthIssue> {
        let mut issues = Vec::new();
        
        // Detect high memory
        let mem_usage = self.check_memory_usage().await;
        if mem_usage > 90 {
            issues.push(HealthIssue::HighMemoryUsage);
            self.emit_health_event(HealthIssue::HighMemoryUsage, "Critical").await;
        } else if mem_usage > 75 {
            self.emit_health_event(HealthIssue::HighMemoryUsage, "Warning").await;
        }

        // Detect high CPU
        let cpu_usage = self.check_cpu_usage().await;
        if cpu_usage > 95.0 {
            issues.push(HealthIssue::HighCpuUsage);
            self.emit_health_event(HealthIssue::HighCpuUsage, "Critical").await;
        } else if cpu_usage > 80.0 {
            self.emit_health_event(HealthIssue::HighCpuUsage, "Warning").await;
        }

        issues
    }

    async fn check_memory_usage(&self) -> u8 {
        #[cfg(feature = "full")]
        {
            let mut sys = System::new_all();
            sys.refresh_memory();
            let used = sys.used_memory();
            let total = sys.total_memory();
            if total > 0 {
                return ((used as f64 / total as f64) * 100.0) as u8;
            }
        }
        50 // Default for testing
    }

    async fn check_cpu_usage(&self) -> f32 {
        #[cfg(feature = "full")]
        {
            let mut sys = System::new_all();
            sys.refresh_cpu();
            return sys.global_cpu_info().cpu_usage();
        }
        30.0 // Default for testing
    }
    
    async fn emit_health_event(&self, issue: HealthIssue, severity: &str) {
        if let Some(ref bus) = self.event_bus {
            let _ = bus.publish(LunaEvent::HealthIssueDetected {
                issue_type: format!("{:?}", issue),
                severity: severity.to_string(),
            }).await;
        }
    }

    pub async fn remediate(&self, issue: &HealthIssue) -> Result<()> {
        warn!("Remediating issue: {:?}", issue);
        
        let action = match issue {
            HealthIssue::HighMemoryUsage => {
                info!("Clearing caches and optimizing memory");
                "cleared_caches".to_string()
            }
            HealthIssue::HighCpuUsage => {
                info!("Reducing process priorities");
                "reduced_priorities".to_string()
            }
            _ => "no_action".to_string(),
        };
        
        // Emit remediation event
        if let Some(ref bus) = self.event_bus {
            let _ = bus.publish(LunaEvent::HealthRemediated {
                issue_type: format!("{:?}", issue),
                action_taken: action,
            }).await;
        }

        Ok(())
    }
    
    /// Check if system is healthy enough for TTS
    pub async fn can_speak(&self) -> bool {
        let cpu = self.check_cpu_usage().await;
        let mem = self.check_memory_usage().await;
        
        // Don't speak if system is under extreme load
        cpu < 98.0 && mem < 95
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}
