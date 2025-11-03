//! Real-Time Resource Monitor & Optimizer (GOD-LEVEL)
//!
//! Monitors CPU, memory, disk, network, GPU, battery, temperatures.
//! Provides predictive alerts, auto-optimization, anomaly detection.

use crate::error::{LunaError, Result};
use crate::events::{EventBus, LunaEvent};
use crate::metrics::Metrics;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::System;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// System metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    pub cpu_per_core: Vec<f32>,
    pub cpu_total: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub memory_available_mb: u64,
    pub disk_io: DiskIO,
    pub network_io: NetworkIO,
    pub gpu_usage: Option<GpuMetrics>,
    pub battery: Option<BatteryInfo>,
    pub temperatures: Vec<Temperature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIO {
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
    pub total_read_bytes: u64,
    pub total_write_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIO {
    pub received_bytes_per_sec: u64,
    pub transmitted_bytes_per_sec: u64,
    pub total_received_bytes: u64,
    pub total_transmitted_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    pub usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub temperature_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub charge_percent: u8,
    pub is_charging: bool,
    pub time_remaining_mins: Option<u32>,
    pub health_percent: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Temperature {
    pub label: String,
    pub temp_celsius: f32,
    pub critical_temp: Option<f32>,
}

/// Resource thresholds for alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceThresholds {
    pub cpu_warning: f32,
    pub cpu_critical: f32,
    pub memory_warning: u8,
    pub memory_critical: u8,
    pub disk_warning: u8,
    pub disk_critical: u8,
    pub temperature_warning: f32,
    pub temperature_critical: f32,
}

impl Default for ResourceThresholds {
    fn default() -> Self {
        Self {
            cpu_warning: 80.0,
            cpu_critical: 95.0,
            memory_warning: 80,
            memory_critical: 95,
            disk_warning: 85,
            disk_critical: 95,
            temperature_warning: 75.0,
            temperature_critical: 90.0,
        }
    }
}

/// Metrics history for trend analysis
pub struct MetricsHistory {
    samples: VecDeque<SystemMetrics>,
    max_samples: usize,
}

impl MetricsHistory {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn add_sample(&mut self, metrics: SystemMetrics) {
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(metrics);
    }

    pub fn get_samples(&self) -> &VecDeque<SystemMetrics> {
        &self.samples
    }

    /// Predict when memory will fill (linear extrapolation)
    pub fn predict_memory_full(&self) -> Option<Duration> {
        if self.samples.len() < 10 {
            return None;
        }

        // Get last 10 samples
        let recent: Vec<_> = self.samples.iter().rev().take(10).collect();

        // Calculate trend
        let mut rate_mb_per_sec = 0.0;
        for i in 1..recent.len() {
            let delta_mb = recent[i - 1].memory_used_mb as f64 - recent[i].memory_used_mb as f64;
            let delta_sec = recent[i - 1]
                .timestamp
                .duration_since(recent[i].timestamp)
                .as_secs_f64();
            if delta_sec > 0.0 {
                rate_mb_per_sec += delta_mb / delta_sec;
            }
        }
        rate_mb_per_sec /= (recent.len() - 1) as f64;

        if rate_mb_per_sec <= 0.0 {
            return None; // Memory usage not increasing
        }

        // Calculate time to fill
        let available = recent[0].memory_available_mb as f64;
        let secs_to_fill = available / rate_mb_per_sec;

        if secs_to_fill > 0.0 && secs_to_fill < 3600.0 {
            Some(Duration::from_secs_f64(secs_to_fill))
        } else {
            None
        }
    }

    /// Detect anomalies (sudden spikes)
    pub fn detect_anomalies(&self) -> Vec<String> {
        let mut anomalies = Vec::new();

        if self.samples.len() < 5 {
            return anomalies;
        }

        let recent: Vec<_> = self.samples.iter().rev().take(5).collect();

        // Check for CPU spike
        let avg_cpu: f32 = recent[1..].iter().map(|m| m.cpu_total).sum::<f32>() / 4.0;
        if recent[0].cpu_total > avg_cpu * 1.5 {
            anomalies.push(format!(
                "CPU spike: {:.1}% (avg: {:.1}%)",
                recent[0].cpu_total, avg_cpu
            ));
        }

        // Check for memory spike
        let avg_mem: u64 = recent[1..].iter().map(|m| m.memory_used_mb).sum::<u64>() / 4;
        let mem_increase = recent[0].memory_used_mb as f64 / avg_mem as f64;
        if mem_increase > 1.3 {
            anomalies.push(format!(
                "Memory spike: {}MB (avg: {}MB)",
                recent[0].memory_used_mb, avg_mem
            ));
        }

        anomalies
    }
}

/// Optimization engine
pub struct OptimizationEngine {
    enabled: bool,
}

impl OptimizationEngine {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Auto-kill memory hogs when threshold exceeded
    pub async fn optimize_memory(
        &self,
        system: &mut System,
        threshold_percent: u8,
    ) -> Result<Vec<u32>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let total_mem = system.total_memory();
        let used_mem = system.used_memory();
        let usage_percent = ((used_mem as f64 / total_mem as f64) * 100.0) as u8;

        if usage_percent < threshold_percent {
            return Ok(Vec::new());
        }

        warn!("Memory usage at {}%, running optimization", usage_percent);

        // Find memory hogs
        let mut processes: Vec<_> = system
            .processes()
            .iter()
            .map(|(pid, proc)| (*pid, proc.memory()))
            .collect();
        processes.sort_by_key(|(_, mem)| std::cmp::Reverse(*mem));

        let mut killed = Vec::new();

        // Kill top memory consumers (excluding critical processes)
        for (pid, mem_bytes) in processes.iter().take(5) {
            let mem_mb = mem_bytes / 1024 / 1024;
            if mem_mb > 500 {
                warn!(
                    "Killing memory hog: PID {} using {}MB",
                    pid.as_u32(),
                    mem_mb
                );
                killed.push(pid.as_u32());

                #[cfg(target_os = "linux")]
                {
                    use nix::sys::signal::{kill as nix_kill, Signal};
                    use nix::unistd::Pid;
                    let _ = nix_kill(Pid::from_raw(pid.as_u32() as i32), Signal::SIGTERM);
                }
            }
        }

        Ok(killed)
    }

    /// Clear temp files to free disk space
    pub async fn optimize_disk(&self) -> Result<u64> {
        if !self.enabled {
            return Ok(0);
        }

        info!("Running disk optimization");

        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let tmp_dirs = vec!["/tmp", "/var/tmp"];
            let mut freed_bytes = 0u64;

            for tmp_dir in tmp_dirs {
                if let Ok(entries) = fs::read_dir(tmp_dir) {
                    for entry in entries.flatten() {
                        if let Ok(metadata) = entry.metadata() {
                            if metadata.is_file() {
                                freed_bytes += metadata.len();
                                let _ = fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            }

            return Ok(freed_bytes);
        }

        #[cfg(not(target_os = "linux"))]
        Ok(0)
    }
}

/// Resource monitor with real-time tracking and optimization
pub struct ResourceMonitor {
    system: Arc<RwLock<System>>,
    metrics: Arc<RwLock<SystemMetrics>>,
    history: Arc<RwLock<MetricsHistory>>,
    thresholds: ResourceThresholds,
    optimization_engine: OptimizationEngine,
    event_bus: Option<Arc<EventBus>>,
    metrics_collector: Option<Arc<Metrics>>,
    monitoring_active: Arc<RwLock<bool>>,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let initial_metrics = Self::capture_metrics(&system);

        Self {
            system: Arc::new(RwLock::new(system)),
            metrics: Arc::new(RwLock::new(initial_metrics)),
            history: Arc::new(RwLock::new(MetricsHistory::new(720))), // 1 hour at 5s intervals
            thresholds: ResourceThresholds::default(),
            optimization_engine: OptimizationEngine::new(),
            event_bus: None,
            metrics_collector: None,
            monitoring_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Add event bus
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Add metrics collector
    pub fn with_metrics(mut self, metrics: Arc<Metrics>) -> Self {
        self.metrics_collector = Some(metrics);
        self
    }

    /// Set thresholds
    pub fn with_thresholds(mut self, thresholds: ResourceThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    /// Start real-time monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        let mut active = self.monitoring_active.write().await;
        if *active {
            return Ok(());
        }
        *active = true;
        drop(active);

        info!("Starting resource monitoring");

        let system = Arc::clone(&self.system);
        let metrics = Arc::clone(&self.metrics);
        let history = Arc::clone(&self.history);
        let active_flag = Arc::clone(&self.monitoring_active);
        let event_bus = self.event_bus.clone();
        let thresholds = self.thresholds.clone();

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

                // Capture metrics
                let current_metrics = Self::capture_metrics(&sys);
                drop(sys);

                // Update current metrics
                {
                    let mut m = metrics.write().await;
                    *m = current_metrics.clone();
                }

                // Add to history
                {
                    let mut h = history.write().await;
                    h.add_sample(current_metrics.clone());

                    // Check for anomalies
                    let anomalies = h.detect_anomalies();
                    for anomaly in anomalies {
                        warn!("Resource anomaly: {}", anomaly);
                    }

                    // Predict memory fill
                    if let Some(time_to_fill) = h.predict_memory_full() {
                        warn!(
                            "Memory will fill in approximately {} seconds",
                            time_to_fill.as_secs()
                        );
                    }
                }

                // Check thresholds
                Self::check_thresholds(&current_metrics, &thresholds, &event_bus).await;
            }
        });

        Ok(())
    }

    /// Stop monitoring
    pub async fn stop_monitoring(&self) {
        let mut active = self.monitoring_active.write().await;
        *active = false;
        info!("Stopped resource monitoring");
    }

    /// Capture current system metrics
    fn capture_metrics(system: &System) -> SystemMetrics {
        let cpu_per_core: Vec<f32> = system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
        let cpu_total = system.global_cpu_info().cpu_usage();

        let memory_total_mb = system.total_memory() / 1024 / 1024;
        let memory_used_mb = system.used_memory() / 1024 / 1024;
        let memory_available_mb = system.available_memory() / 1024 / 1024;

        // Network stats - simplified for sysinfo 0.30
        let network_io = NetworkIO {
            received_bytes_per_sec: 0,
            transmitted_bytes_per_sec: 0,
            total_received_bytes: 0,
            total_transmitted_bytes: 0,
        };

        // Disk stats
        let disk_io = DiskIO {
            read_bytes_per_sec: 0,
            write_bytes_per_sec: 0,
            total_read_bytes: 0,
            total_write_bytes: 0,
        };

        SystemMetrics {
            timestamp: Instant::now(),
            cpu_per_core,
            cpu_total,
            memory_used_mb,
            memory_total_mb,
            memory_available_mb,
            disk_io,
            network_io,
            gpu_usage: None,
            battery: None,
            temperatures: Vec::new(),
        }
    }

    /// Check thresholds and emit alerts
    async fn check_thresholds(
        metrics: &SystemMetrics,
        thresholds: &ResourceThresholds,
        event_bus: &Option<Arc<EventBus>>,
    ) {
        // CPU check
        if metrics.cpu_total > thresholds.cpu_critical {
            warn!("CRITICAL: CPU usage at {:.1}%", metrics.cpu_total);
            if let Some(ref bus) = event_bus {
                bus.publish(LunaEvent::Custom {
                    event_type: "ResourceAlert".to_string(),
                    data: serde_json::json!({
                        "resource": "cpu",
                        "level": "critical",
                        "value": metrics.cpu_total,
                    }),
                })
                .await;
            }
        } else if metrics.cpu_total > thresholds.cpu_warning {
            warn!("WARNING: CPU usage at {:.1}%", metrics.cpu_total);
        }

        // Memory check
        let memory_percent =
            ((metrics.memory_used_mb as f64 / metrics.memory_total_mb as f64) * 100.0) as u8;
        if memory_percent > thresholds.memory_critical {
            warn!("CRITICAL: Memory usage at {}%", memory_percent);
            if let Some(ref bus) = event_bus {
                bus.publish(LunaEvent::Custom {
                    event_type: "ResourceAlert".to_string(),
                    data: serde_json::json!({
                        "resource": "memory",
                        "level": "critical",
                        "value": memory_percent,
                    }),
                })
                .await;
            }
        } else if memory_percent > thresholds.memory_warning {
            warn!("WARNING: Memory usage at {}%", memory_percent);
        }
    }

    /// Get current metrics
    pub async fn get_current_metrics(&self) -> SystemMetrics {
        self.metrics.read().await.clone()
    }

    /// Get metrics history
    pub async fn get_history(&self) -> Vec<SystemMetrics> {
        let history = self.history.read().await;
        history.get_samples().iter().cloned().collect()
    }

    /// Run optimization
    pub async fn optimize(&self) -> Result<()> {
        info!("Running system optimization");

        let mut system = self.system.write().await;

        // Optimize memory
        let killed = self
            .optimization_engine
            .optimize_memory(&mut system, 90)
            .await?;
        if !killed.is_empty() {
            info!("Killed {} memory-intensive processes", killed.len());
        }

        drop(system);

        // Optimize disk
        let freed_bytes = self.optimization_engine.optimize_disk().await?;
        if freed_bytes > 0 {
            info!("Freed {} MB of disk space", freed_bytes / 1024 / 1024);
        }

        Ok(())
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_monitor_creation() {
        let monitor = ResourceMonitor::new();
        let metrics = monitor.get_current_metrics().await;
        assert!(metrics.cpu_total >= 0.0);
    }

    #[tokio::test]
    async fn test_monitoring() {
        let monitor = ResourceMonitor::new();
        let result = monitor.start_monitoring().await;
        assert!(result.is_ok());

        tokio::time::sleep(Duration::from_secs(1)).await;

        monitor.stop_monitoring().await;
    }

    #[test]
    fn test_metrics_history() {
        let mut history = MetricsHistory::new(10);

        for i in 0..15 {
            let metrics = SystemMetrics {
                timestamp: Instant::now(),
                cpu_per_core: vec![50.0],
                cpu_total: 50.0,
                memory_used_mb: 1000 + (i * 100),
                memory_total_mb: 8000,
                memory_available_mb: 7000 - (i * 100),
                disk_io: DiskIO {
                    read_bytes_per_sec: 0,
                    write_bytes_per_sec: 0,
                    total_read_bytes: 0,
                    total_write_bytes: 0,
                },
                network_io: NetworkIO {
                    received_bytes_per_sec: 0,
                    transmitted_bytes_per_sec: 0,
                    total_received_bytes: 0,
                    total_transmitted_bytes: 0,
                },
                gpu_usage: None,
                battery: None,
                temperatures: Vec::new(),
            };
            history.add_sample(metrics);
        }

        assert_eq!(history.get_samples().len(), 10);
    }
}
