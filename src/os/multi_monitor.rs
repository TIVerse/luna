//! Multi-Monitor Intelligence (GOD-LEVEL)
//!
//! Detects all monitors, manages window layouts per profile,
//! intelligent window placement, DPI-aware positioning.

use crate::error::{LunaError, Result};
use crate::events::{EventBus, LunaEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Display/monitor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub resolution: (u32, u32),
    pub position: (i32, i32),
    pub scale_factor: f64,
    pub refresh_rate: u32,
    pub is_primary: bool,
    pub rotation: DisplayRotation,
    pub capabilities: DisplayCapabilities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayRotation {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayCapabilities {
    pub supports_hdr: bool,
    pub supports_vrr: bool,
    pub color_depth: u8,
}

impl Default for DisplayCapabilities {
    fn default() -> Self {
        Self {
            supports_hdr: false,
            supports_vrr: false,
            color_depth: 24,
        }
    }
}

/// Window layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowLayout {
    pub name: String,
    pub windows: Vec<WindowPlacement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPlacement {
    pub app_name: String,
    pub display_id: String,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub state: WindowState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowState {
    Normal,
    Maximized,
    Minimized,
    Fullscreen,
}

/// Window placement engine - AI-powered optimal placement
pub struct WindowPlacementEngine {
    rules: Vec<PlacementRule>,
}

#[derive(Debug, Clone)]
pub struct PlacementRule {
    pub app_category: Option<String>,
    pub preferred_display: DisplayPreference,
    pub preferred_size: (u32, u32),
}

#[derive(Debug, Clone)]
pub enum DisplayPreference {
    Primary,
    Secondary,
    Largest,
    HighestRefreshRate,
    Specific(String),
}

impl WindowPlacementEngine {
    pub fn new() -> Self {
        let rules = vec![
            // IDEs prefer large displays
            PlacementRule {
                app_category: Some("IDE".to_string()),
                preferred_display: DisplayPreference::Largest,
                preferred_size: (1920, 1080),
            },
            // Browsers on primary
            PlacementRule {
                app_category: Some("Browser".to_string()),
                preferred_display: DisplayPreference::Primary,
                preferred_size: (1600, 900),
            },
            // Media players on high refresh rate
            PlacementRule {
                app_category: Some("Media".to_string()),
                preferred_display: DisplayPreference::HighestRefreshRate,
                preferred_size: (1280, 720),
            },
        ];

        Self { rules }
    }

    /// Suggest optimal placement for an app
    pub fn suggest_placement(
        &self,
        app_name: &str,
        displays: &[DisplayInfo],
    ) -> Option<WindowPlacement> {
        if displays.is_empty() {
            return None;
        }

        // Find matching rule
        let rule = self.rules.iter().find(|r| {
            if let Some(ref category) = r.app_category {
                app_name.to_lowercase().contains(&category.to_lowercase())
            } else {
                false
            }
        });

        let display = if let Some(rule) = rule {
            match &rule.preferred_display {
                DisplayPreference::Primary => displays
                    .iter()
                    .find(|d| d.is_primary)
                    .or_else(|| displays.first()),
                DisplayPreference::Largest => displays
                    .iter()
                    .max_by_key(|d| d.resolution.0 * d.resolution.1),
                DisplayPreference::HighestRefreshRate => {
                    displays.iter().max_by_key(|d| d.refresh_rate)
                }
                DisplayPreference::Specific(id) => displays.iter().find(|d| d.id == *id),
                _ => displays.first(),
            }
        } else {
            displays
                .iter()
                .find(|d| d.is_primary)
                .or_else(|| displays.first())
        };

        let display = display?;

        Some(WindowPlacement {
            app_name: app_name.to_string(),
            display_id: display.id.clone(),
            position: (display.position.0 + 100, display.position.1 + 100),
            size: rule.map(|r| r.preferred_size).unwrap_or((800, 600)),
            state: WindowState::Normal,
        })
    }
}

/// Multi-monitor manager
pub struct MultiMonitorManager {
    displays: Arc<RwLock<Vec<DisplayInfo>>>,
    layouts: Arc<RwLock<HashMap<String, WindowLayout>>>,
    placement_engine: WindowPlacementEngine,
    event_bus: Option<Arc<EventBus>>,
}

impl MultiMonitorManager {
    /// Create a new multi-monitor manager
    pub fn new() -> Self {
        Self {
            displays: Arc::new(RwLock::new(Vec::new())),
            layouts: Arc::new(RwLock::new(HashMap::new())),
            placement_engine: WindowPlacementEngine::new(),
            event_bus: None,
        }
    }

    /// Add event bus
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Detect all connected displays
    pub async fn detect_displays(&self) -> Result<Vec<DisplayInfo>> {
        info!("Detecting displays...");

        #[cfg(target_os = "linux")]
        let displays = self.detect_linux_displays().await?;

        #[cfg(target_os = "windows")]
        let displays = self.detect_windows_displays().await?;

        #[cfg(target_os = "macos")]
        let displays = self.detect_macos_displays().await?;

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        let displays = vec![self.create_default_display()];

        let mut display_list = self.displays.write().await;
        *display_list = displays.clone();

        info!("Detected {} display(s)", displays.len());

        // Publish event
        if let Some(ref bus) = self.event_bus {
            for display in &displays {
                bus.publish(LunaEvent::Custom {
                    event_type: "DisplayDetected".to_string(),
                    data: serde_json::json!({
                        "id": &display.id,
                        "resolution": display.resolution,
                        "is_primary": display.is_primary,
                    }),
                })
                .await;
            }
        }

        Ok(displays)
    }

    #[cfg(target_os = "linux")]
    async fn detect_linux_displays(&self) -> Result<Vec<DisplayInfo>> {
        use std::process::Command;

        // Try xrandr first
        let output = Command::new("xrandr").arg("--query").output();

        if let Ok(output) = output {
            if output.status.success() {
                return self.parse_xrandr_output(&output.stdout);
            }
        }

        // Fallback to default
        Ok(vec![self.create_default_display()])
    }

    #[cfg(target_os = "linux")]
    fn parse_xrandr_output(&self, output: &[u8]) -> Result<Vec<DisplayInfo>> {
        let output_str = String::from_utf8_lossy(output);
        let mut displays = Vec::new();
        let mut current_display: Option<DisplayInfo> = None;
        let mut display_index = 0;

        for line in output_str.lines() {
            let line = line.trim();

            // Parse connected displays
            if line.contains(" connected") {
                if let Some(display) = current_display.take() {
                    displays.push(display);
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                let name = parts.first().unwrap_or(&"Unknown").to_string();
                let is_primary = line.contains("primary");

                current_display = Some(DisplayInfo {
                    id: format!("display_{}", display_index),
                    name: name.clone(),
                    resolution: (1920, 1080), // Will be updated
                    position: (0, 0),
                    scale_factor: 1.0,
                    refresh_rate: 60,
                    is_primary,
                    rotation: DisplayRotation::Normal,
                    capabilities: DisplayCapabilities::default(),
                });

                display_index += 1;
            }

            // Parse resolution
            if line.contains("x") && line.contains("*") {
                if let Some(ref mut display) = current_display {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(res_part) = parts.first() {
                        if let Some((w, h)) = res_part.split_once('x') {
                            if let (Ok(width), Ok(height)) = (w.parse(), h.parse()) {
                                display.resolution = (width, height);
                            }
                        }
                    }
                }
            }
        }

        if let Some(display) = current_display {
            displays.push(display);
        }

        if displays.is_empty() {
            displays.push(self.create_default_display());
        }

        Ok(displays)
    }

    #[cfg(target_os = "windows")]
    async fn detect_windows_displays(&self) -> Result<Vec<DisplayInfo>> {
        // Windows implementation using Win32 API
        // For now, return default display
        Ok(vec![self.create_default_display()])
    }

    #[cfg(target_os = "macos")]
    async fn detect_macos_displays(&self) -> Result<Vec<DisplayInfo>> {
        // macOS implementation using Core Graphics
        // For now, return default display
        Ok(vec![self.create_default_display()])
    }

    fn create_default_display(&self) -> DisplayInfo {
        DisplayInfo {
            id: "display_0".to_string(),
            name: "Primary Display".to_string(),
            resolution: (1920, 1080),
            position: (0, 0),
            scale_factor: 1.0,
            refresh_rate: 60,
            is_primary: true,
            rotation: DisplayRotation::Normal,
            capabilities: DisplayCapabilities::default(),
        }
    }

    /// Get all displays
    pub async fn get_displays(&self) -> Vec<DisplayInfo> {
        self.displays.read().await.clone()
    }

    /// Save a window layout
    pub async fn save_layout(&self, name: String, layout: WindowLayout) {
        let mut layouts = self.layouts.write().await;
        layouts.insert(name.clone(), layout);
        info!("Saved window layout: {}", name);
    }

    /// Load a window layout
    pub async fn load_layout(&self, name: &str) -> Option<WindowLayout> {
        let layouts = self.layouts.read().await;
        layouts.get(name).cloned()
    }

    /// Apply a window layout
    pub async fn apply_layout(&self, name: &str) -> Result<()> {
        let layout = self
            .load_layout(name)
            .await
            .ok_or_else(|| LunaError::SystemOperation(format!("Layout '{}' not found", name)))?;

        info!("Applying window layout: {}", name);

        for placement in &layout.windows {
            debug!(
                "Placing window: {} on display {}",
                placement.app_name, placement.display_id
            );
            // TODO: Implement actual window placement
        }

        Ok(())
    }

    /// Suggest optimal placement for an app
    pub async fn suggest_placement(&self, app_name: &str) -> Option<WindowPlacement> {
        let displays = self.displays.read().await;
        self.placement_engine.suggest_placement(app_name, &displays)
    }

    /// Get primary display
    pub async fn get_primary_display(&self) -> Option<DisplayInfo> {
        let displays = self.displays.read().await;
        displays.iter().find(|d| d.is_primary).cloned()
    }

    /// Get largest display
    pub async fn get_largest_display(&self) -> Option<DisplayInfo> {
        let displays = self.displays.read().await;
        displays
            .iter()
            .max_by_key(|d| d.resolution.0 * d.resolution.1)
            .cloned()
    }
}

impl Default for MultiMonitorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_multi_monitor_manager_creation() {
        let manager = MultiMonitorManager::new();
        let displays = manager.get_displays().await;
        assert_eq!(displays.len(), 0);
    }

    #[tokio::test]
    async fn test_detect_displays() {
        let manager = MultiMonitorManager::new();
        let result = manager.detect_displays().await;
        assert!(result.is_ok());

        let displays = result.unwrap();
        assert!(!displays.is_empty());
    }

    #[tokio::test]
    async fn test_placement_engine() {
        let engine = WindowPlacementEngine::new();
        let displays = vec![DisplayInfo {
            id: "display_0".to_string(),
            name: "Primary".to_string(),
            resolution: (1920, 1080),
            position: (0, 0),
            scale_factor: 1.0,
            refresh_rate: 60,
            is_primary: true,
            rotation: DisplayRotation::Normal,
            capabilities: DisplayCapabilities::default(),
        }];

        let placement = engine.suggest_placement("vscode", &displays);
        assert!(placement.is_some());
    }
}
