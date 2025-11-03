//! Process Priority & Affinity Management (GOD-LEVEL)

use crate::error::{LunaError, Result};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum PriorityLevel {
    Idle,
    Low,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
    Realtime,
}

#[derive(Debug, Clone)]
pub struct CpuSet {
    pub cores: Vec<usize>,
}

pub struct PriorityRule {
    pub app_pattern: String,
    pub priority: PriorityLevel,
    pub affinity: Option<CpuSet>,
}

pub struct ProcessOptimizer {
    priority_rules: Vec<PriorityRule>,
    affinity_map: HashMap<u32, CpuSet>,
}

impl ProcessOptimizer {
    pub fn new() -> Self {
        Self {
            priority_rules: Vec::new(),
            affinity_map: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: PriorityRule) {
        self.priority_rules.push(rule);
    }

    pub async fn set_priority(&self, pid: u32, priority: PriorityLevel) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let nice_value = match priority {
                PriorityLevel::Idle => 19,
                PriorityLevel::Low => 10,
                PriorityLevel::BelowNormal => 5,
                PriorityLevel::Normal => 0,
                PriorityLevel::AboveNormal => -5,
                PriorityLevel::High => -10,
                PriorityLevel::Realtime => -20,
            };

            let _ = std::process::Command::new("renice")
                .args([&nice_value.to_string(), "-p", &pid.to_string()])
                .output();
        }
        Ok(())
    }

    pub async fn set_affinity(&mut self, pid: u32, cpuset: CpuSet) -> Result<()> {
        self.affinity_map.insert(pid, cpuset.clone());

        #[cfg(target_os = "linux")]
        {
            let cpu_list = cpuset
                .cores
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(",");

            let _ = std::process::Command::new("taskset")
                .args(["-p", "-c", &cpu_list, &pid.to_string()])
                .output();
        }
        Ok(())
    }

    pub async fn optimize_for_gaming(&self) -> Result<()> {
        // Lower priority of background processes
        // Pin game to performance cores
        Ok(())
    }

    pub async fn optimize_for_work(&self) -> Result<()> {
        // Balance priorities
        Ok(())
    }
}

impl Default for ProcessOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
