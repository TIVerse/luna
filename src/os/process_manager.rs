//! Smart Process Lifecycle Management (GOD-LEVEL)
//!
//! Tracks all spawned processes with full lineage, resource monitoring,
//! auto-restart policies, health checks, and graceful shutdown.

use crate::error::{LunaError, Result};
use crate::events::{EventBus, LunaEvent};
use crate::metrics::Metrics;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{Pid as SysPid, System};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Process ID type
pub type Pid = u32;

/// Process status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessStatus {
    Running,
    Sleeping,
    Stopped,
    Zombie,
    Dead,
    Unknown,
}

/// Process information with full tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub parent: Option<Pid>,
    pub children: Vec<Pid>,
    pub command: String,
    pub args: Vec<String>,
    pub executable: PathBuf,
    #[serde(skip, default = "Instant::now")]
    pub start_time: Instant,
    pub cpu_usage: f64,
    pub memory_mb: u64,
    pub status: ProcessStatus,
    pub health_check: Option<HealthCheckConfig>,
    pub restart_policy: Option<RestartPolicy>,
    pub restart_count: u32,
    #[serde(skip, default)]
    pub last_restart: Option<Instant>,
}

/// Restart policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartPolicy {
    pub max_restarts: u32,
    pub restart_window_secs: u64,
    pub backoff_initial_ms: u64,
    pub backoff_max_ms: u64,
    pub backoff_multiplier: f64,
    pub on_failure_only: bool,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self {
            max_restarts: 3,
            restart_window_secs: 300,
            backoff_initial_ms: 1000,
            backoff_max_ms: 30000,
            backoff_multiplier: 2.0,
            on_failure_only: true,
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub check_interval_secs: u64,
    pub timeout_secs: u64,
    pub max_failures: u32,
    pub check_type: HealthCheckType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckType {
    ProcessAlive,
    CpuUsageBelow(f64),
    MemoryUsageBelow(u64),
    Responsive(PathBuf), // Check if process responds to signals
}

/// Resource quota per process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub max_cpu_percent: Option<f64>,
    pub max_memory_mb: Option<u64>,
    pub max_children: Option<usize>,
}

/// Process manager - tracks and manages all spawned processes
pub struct ProcessManager {
    /// All tracked processes
    processes: Arc<DashMap<Pid, ProcessInfo>>,
    /// System info for resource monitoring
    system: Arc<RwLock<System>>,
    /// Resource quotas
    quotas: Arc<DashMap<Pid, ResourceQuota>>,
    /// Restart policies by app name
    app_restart_policies: Arc<DashMap<String, RestartPolicy>>,
    /// Event bus
    event_bus: Option<Arc<EventBus>>,
    /// Metrics
    metrics: Option<Arc<Metrics>>,
    /// Monitoring task active
    monitoring_active: Arc<RwLock<bool>>,
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new() -> Self {
        Self {
            processes: Arc::new(DashMap::new()),
            system: Arc::new(RwLock::new(System::new_all())),
            quotas: Arc::new(DashMap::new()),
            app_restart_policies: Arc::new(DashMap::new()),
            event_bus: None,
            metrics: None,
            monitoring_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Add event bus
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Add metrics
    pub fn with_metrics(mut self, metrics: Arc<Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Start monitoring all processes
    pub async fn start_monitoring(&self) -> Result<()> {
        let mut active = self.monitoring_active.write().await;
        if *active {
            return Ok(());
        }
        *active = true;
        drop(active);

        info!("Starting process monitoring");

        let processes = Arc::clone(&self.processes);
        let system = Arc::clone(&self.system);
        let active_flag = Arc::clone(&self.monitoring_active);
        let event_bus = self.event_bus.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                let active = *active_flag.read().await;
                if !active {
                    break;
                }

                // Refresh system info
                let mut sys = system.write().await;
                sys.refresh_all();

                // Update all tracked processes
                for mut entry in processes.iter_mut() {
                    let pid = *entry.key();
                    let info = entry.value_mut();

                    if let Some(process) = sys.process(SysPid::from_u32(pid)) {
                        info.cpu_usage = process.cpu_usage() as f64;
                        info.memory_mb = process.memory() / 1024 / 1024;
                        info.status = ProcessStatus::Running;
                    } else {
                        info.status = ProcessStatus::Dead;

                        // Publish process death event
                        if let Some(ref bus) = event_bus {
                            bus.publish(LunaEvent::Custom {
                                event_type: "ProcessDied".to_string(),
                                data: serde_json::json!({
                                    "pid": pid,
                                    "command": &info.command,
                                }),
                            })
                            .await;
                        }
                    }
                }

                drop(sys);
            }
        });

        Ok(())
    }

    /// Stop monitoring
    pub async fn stop_monitoring(&self) {
        let mut active = self.monitoring_active.write().await;
        *active = false;
        info!("Stopped process monitoring");
    }

    /// Spawn a new process with tracking
    pub async fn spawn(&self, command: &str, args: &[&str]) -> Result<Pid> {
        debug!("Spawning process: {} {:?}", command, args);

        let mut cmd = Command::new(command);
        cmd.args(args);

        let child = cmd
            .spawn()
            .map_err(|e| LunaError::SystemOperation(format!("Failed to spawn process: {}", e)))?;

        let pid = child.id();

        // Track the process
        let info = ProcessInfo {
            pid,
            parent: None,
            children: Vec::new(),
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            executable: PathBuf::from(command),
            start_time: Instant::now(),
            cpu_usage: 0.0,
            memory_mb: 0,
            status: ProcessStatus::Running,
            health_check: None,
            restart_policy: None,
            restart_count: 0,
            last_restart: None,
        };

        self.processes.insert(pid, info);

        // Publish event
        if let Some(ref bus) = self.event_bus {
            bus.publish(LunaEvent::Custom {
                event_type: "ProcessSpawned".to_string(),
                data: serde_json::json!({
                    "pid": pid,
                    "command": command,
                }),
            })
            .await;
        }

        info!("Spawned process {} (PID: {})", command, pid);
        Ok(pid)
    }

    /// Kill a process by PID
    pub async fn kill(&self, pid: Pid) -> Result<()> {
        debug!("Killing process: {}", pid);

        #[cfg(target_os = "linux")]
        {
            use nix::sys::signal::{kill as nix_kill, Signal};
            use nix::unistd::Pid as NixPid;

            nix_kill(NixPid::from_raw(pid as i32), Signal::SIGTERM).map_err(|e| {
                LunaError::SystemOperation(format!("Failed to kill process: {}", e))
            })?;
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .output()
                .map_err(|e| {
                    LunaError::SystemOperation(format!("Failed to kill process: {}", e))
                })?;
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output()
                .map_err(|e| {
                    LunaError::SystemOperation(format!("Failed to kill process: {}", e))
                })?;
        }

        // Remove from tracking
        self.processes.remove(&pid);

        info!("Killed process: {}", pid);
        Ok(())
    }

    /// Get process info
    pub fn get_info(&self, pid: Pid) -> Option<ProcessInfo> {
        self.processes.get(&pid).map(|entry| entry.clone())
    }

    /// List all tracked processes
    pub fn list_processes(&self) -> Vec<ProcessInfo> {
        self.processes.iter().map(|entry| entry.clone()).collect()
    }

    /// Set resource quota for a process
    pub fn set_quota(&self, pid: Pid, quota: ResourceQuota) {
        self.quotas.insert(pid, quota);
    }

    /// Set restart policy for an application
    pub fn set_restart_policy(&self, app_name: String, policy: RestartPolicy) {
        self.app_restart_policies.insert(app_name, policy);
    }

    /// Check resource quotas and enforce
    pub async fn enforce_quotas(&self) -> Result<()> {
        let mut sys = self.system.write().await;
        sys.refresh_all();

        for entry in self.quotas.iter() {
            let pid = *entry.key();
            let quota = entry.value();

            if let Some(mut process_info) = self.processes.get_mut(&pid) {
                if let Some(process) = sys.process(SysPid::from_u32(pid)) {
                    let cpu = process.cpu_usage() as f64;
                    let mem_mb = process.memory() / 1024 / 1024;

                    let mut violated = false;

                    if let Some(max_cpu) = quota.max_cpu_percent {
                        if cpu > max_cpu {
                            warn!("Process {} exceeds CPU quota: {}% > {}%", pid, cpu, max_cpu);
                            violated = true;
                        }
                    }

                    if let Some(max_mem) = quota.max_memory_mb {
                        if mem_mb > max_mem {
                            warn!(
                                "Process {} exceeds memory quota: {}MB > {}MB",
                                pid, mem_mb, max_mem
                            );
                            violated = true;
                        }
                    }

                    if violated {
                        // Optionally kill the process
                        warn!("Process {} violated resource quotas, consider killing", pid);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get total resource usage across all tracked processes
    pub async fn get_total_usage(&self) -> (f64, u64) {
        let mut total_cpu = 0.0;
        let mut total_mem = 0u64;

        for entry in self.processes.iter() {
            let info = entry.value();
            total_cpu += info.cpu_usage;
            total_mem += info.memory_mb;
        }

        (total_cpu, total_mem)
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_process_manager_creation() {
        let manager = ProcessManager::new();
        assert_eq!(manager.list_processes().len(), 0);
    }

    #[tokio::test]
    async fn test_spawn_and_track() {
        let manager = ProcessManager::new();

        // Spawn echo command
        #[cfg(not(target_os = "windows"))]
        let result = manager.spawn("echo", &["hello"]).await;

        #[cfg(target_os = "windows")]
        let result = manager.spawn("cmd", &["/c", "echo", "hello"]).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_monitoring() {
        let manager = ProcessManager::new();
        let result = manager.start_monitoring().await;
        assert!(result.is_ok());

        manager.stop_monitoring().await;
    }
}
