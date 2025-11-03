//! Capability Detection (GOD-LEVEL Enhancement)
//!
//! Detects available system tools and capabilities at startup,
//! provides proactive guidance for missing dependencies.

use crate::error::Result;
use crate::events::{EventBus, LunaEvent};
use std::process::Command;
use std::sync::Arc;
use tracing::{info, warn};

/// System capability information
#[derive(Debug, Clone)]
pub struct Capability {
    pub name: String,
    pub tool_name: String,
    pub available: bool,
    pub version: Option<String>,
    pub install_hint: Option<String>,
}

/// Capability detector
pub struct CapabilityDetector {
    event_bus: Option<Arc<EventBus>>,
}

impl CapabilityDetector {
    /// Create a new capability detector
    pub fn new() -> Self {
        Self { event_bus: None }
    }

    /// Add event bus for publishing detection events
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Detect all system capabilities
    pub async fn detect_all(&self) -> Vec<Capability> {
        info!("Detecting system capabilities...");

        let mut capabilities = Vec::new();

        // Audio control
        capabilities.push(
            self.detect_tool(
                "pactl",
                "Audio Control (PulseAudio)",
                Some("sudo apt install pulseaudio-utils"),
            )
            .await,
        );
        capabilities.push(
            self.detect_tool(
                "amixer",
                "Audio Control (ALSA)",
                Some("sudo apt install alsa-utils"),
            )
            .await,
        );

        // Media control
        capabilities.push(
            self.detect_tool(
                "playerctl",
                "Media Control (MPRIS)",
                Some("sudo apt install playerctl"),
            )
            .await,
        );

        // Brightness control
        capabilities.push(
            self.detect_tool(
                "xbacklight",
                "Brightness Control (X11)",
                Some("sudo apt install xbacklight"),
            )
            .await,
        );
        capabilities.push(
            self.detect_tool(
                "brightnessctl",
                "Brightness Control",
                Some("sudo apt install brightnessctl"),
            )
            .await,
        );

        // Screen locking
        capabilities.push(
            self.detect_tool(
                "gnome-screensaver-command",
                "Screen Lock (GNOME)",
                Some("Pre-installed on GNOME"),
            )
            .await,
        );
        capabilities.push(
            self.detect_tool(
                "xdg-screensaver",
                "Screen Lock (XDG)",
                Some("sudo apt install xdg-utils"),
            )
            .await,
        );

        // Window management
        capabilities.push(
            self.detect_tool(
                "wmctrl",
                "Window Management",
                Some("sudo apt install wmctrl"),
            )
            .await,
        );
        capabilities.push(
            self.detect_tool(
                "xdotool",
                "Window Management (Advanced)",
                Some("sudo apt install xdotool"),
            )
            .await,
        );

        // System control
        capabilities.push(
            self.detect_tool("systemctl", "System Control (systemd)", None)
                .await,
        );

        // Publish events for each capability
        if let Some(ref bus) = self.event_bus {
            for cap in &capabilities {
                bus.publish(LunaEvent::CapabilityDetected {
                    capability: cap.name.clone(),
                    available: cap.available,
                    tool_name: Some(cap.tool_name.clone()),
                    version: cap.version.clone(),
                })
                .await;
            }
        }

        self.print_summary(&capabilities);

        capabilities
    }

    /// Detect a specific tool
    async fn detect_tool(
        &self,
        tool: &str,
        capability_name: &str,
        install_hint: Option<&str>,
    ) -> Capability {
        let version = self.get_tool_version(tool).await;
        let available = version.is_some();

        if !available {
            warn!(
                "Capability '{}' not available: {} not found",
                capability_name, tool
            );
            if let Some(hint) = install_hint {
                info!("  Install with: {}", hint);
            }
        }

        Capability {
            name: capability_name.to_string(),
            tool_name: tool.to_string(),
            available,
            version,
            install_hint: install_hint.map(|s| s.to_string()),
        }
    }

    /// Get tool version if available
    async fn get_tool_version(&self, tool: &str) -> Option<String> {
        // Try --version
        if let Ok(output) = Command::new(tool).arg("--version").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                let first_line = version.lines().next().unwrap_or("").to_string();
                return Some(first_line);
            }
        }

        // Try -v
        if let Ok(output) = Command::new(tool).arg("-v").output() {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                let first_line = version.lines().next().unwrap_or("").to_string();
                return Some(first_line);
            }
        }

        // Just check if tool exists
        if Command::new("which")
            .arg(tool)
            .output()
            .ok()?
            .status
            .success()
        {
            return Some("available".to_string());
        }

        None
    }

    /// Print capability summary
    fn print_summary(&self, capabilities: &[Capability]) {
        let available = capabilities.iter().filter(|c| c.available).count();
        let total = capabilities.len();

        info!("\n🔍 Capability Detection Summary");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("  Available: {}/{}", available, total);
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for cap in capabilities {
            let status = if cap.available { "✅" } else { "❌" };
            info!("  {} {}: {}", status, cap.name, cap.tool_name);
        }

        // Show missing capabilities with install hints
        let missing: Vec<_> = capabilities.iter().filter(|c| !c.available).collect();
        if !missing.is_empty() {
            info!("\n💡 Missing Capabilities:");
            for cap in missing {
                if let Some(hint) = &cap.install_hint {
                    info!("  • {}: {}", cap.name, hint);
                }
            }
        }

        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}

impl Default for CapabilityDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capability_detector() {
        let detector = CapabilityDetector::new();
        let capabilities = detector.detect_all().await;

        // Should detect at least some capabilities on any system
        assert!(!capabilities.is_empty());
    }
}
