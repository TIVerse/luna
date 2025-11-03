//! Window management (GOD-LEVEL Enhancement)
//!
//! Cross-platform window control for move, resize, maximize, minimize, focus.

use crate::error::{LunaError, Result};
use std::process::Command;
use tracing::{info, warn};

/// Window control handler
pub struct WindowControl;

impl WindowControl {
    /// Create a new window control handler
    pub fn new() -> Self {
        Self
    }

    /// Focus a window by application name
    pub async fn focus_window(&self, app_name: &str) -> Result<String> {
        info!("Focusing window: {}", app_name);

        #[cfg(target_os = "linux")]
        {
            // Try wmctrl first
            let result = Command::new("wmctrl").args(&["-a", app_name]).status();

            if result.is_ok() && result.unwrap().success() {
                return Ok(format!("Focused window: {}", app_name));
            }

            // Fallback to xdotool
            let status = Command::new("xdotool")
                .args(&["search", "--name", app_name, "windowactivate"])
                .status()
                .map_err(|e| {
                    LunaError::SystemOperation(format!("Failed to focus window: {}", e))
                })?;

            if status.success() {
                Ok(format!("Focused window: {}", app_name))
            } else {
                Err(LunaError::SystemOperation(
                    "Window tools not found. Install wmctrl or xdotool".to_string(),
                ))
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Use PowerShell on Windows
            let script = format!(
                r#"Add-Type @"
                    using System;
                    using System.Runtime.InteropServices;
                    public class Window {{
                        [DllImport("user32.dll")]
                        public static extern bool SetForegroundWindow(IntPtr hWnd);
                    }}
"@
$p = Get-Process | Where-Object {{$_.MainWindowTitle -like "*{}*"}} | Select-Object -First 1
if ($p) {{ [Window]::SetForegroundWindow($p.MainWindowHandle) }}"#,
                app_name
            );

            let status = Command::new("powershell")
                .args(&["-Command", &script])
                .status()
                .map_err(|e| {
                    LunaError::SystemOperation(format!("Failed to focus window: {}", e))
                })?;

            if status.success() {
                Ok(format!("Focused window: {}", app_name))
            } else {
                Err(LunaError::SystemOperation(
                    "Failed to focus window".to_string(),
                ))
            }
        }

        #[cfg(target_os = "macos")]
        {
            let status = Command::new("osascript")
                .args(&[
                    "-e",
                    &format!("tell application \"{}\" to activate", app_name),
                ])
                .status()
                .map_err(|e| {
                    LunaError::SystemOperation(format!("Failed to focus window: {}", e))
                })?;

            if status.success() {
                Ok(format!("Focused window: {}", app_name))
            } else {
                Err(LunaError::SystemOperation(
                    "Failed to focus window".to_string(),
                ))
            }
        }
    }

    /// Maximize a window
    pub async fn maximize_window(&self, app_name: &str) -> Result<String> {
        info!("Maximizing window: {}", app_name);

        #[cfg(target_os = "linux")]
        {
            let status = Command::new("wmctrl")
                .args(&["-r", app_name, "-b", "add,maximized_vert,maximized_horz"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to maximize: {}", e)))?;

            if status.success() {
                Ok(format!("Maximized window: {}", app_name))
            } else {
                Err(LunaError::SystemOperation(
                    "wmctrl not found. Install wmctrl".to_string(),
                ))
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            warn!("Maximize not implemented for this platform");
            Err(LunaError::SystemOperation(
                "Maximize not supported on this platform".to_string(),
            ))
        }
    }

    /// Minimize a window
    pub async fn minimize_window(&self, app_name: &str) -> Result<String> {
        info!("Minimizing window: {}", app_name);

        #[cfg(target_os = "linux")]
        {
            let status = Command::new("xdotool")
                .args(&["search", "--name", app_name, "windowminimize"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to minimize: {}", e)))?;

            if status.success() {
                Ok(format!("Minimized window: {}", app_name))
            } else {
                Err(LunaError::SystemOperation(
                    "xdotool not found. Install xdotool".to_string(),
                ))
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            warn!("Minimize not implemented for this platform");
            Err(LunaError::SystemOperation(
                "Minimize not supported on this platform".to_string(),
            ))
        }
    }

    /// Move window to specific position
    pub async fn move_window(&self, app_name: &str, x: i32, y: i32) -> Result<String> {
        info!("Moving window {} to ({}, {})", app_name, x, y);

        #[cfg(target_os = "linux")]
        {
            let status = Command::new("wmctrl")
                .args(&["-r", app_name, "-e", &format!("0,{},{},0,0", x, y)])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to move window: {}", e)))?;

            if status.success() {
                Ok(format!("Moved window {} to ({}, {})", app_name, x, y))
            } else {
                Err(LunaError::SystemOperation(
                    "wmctrl not found. Install wmctrl".to_string(),
                ))
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            warn!("Move not implemented for this platform");
            Err(LunaError::SystemOperation(
                "Move not supported on this platform".to_string(),
            ))
        }
    }

    /// Resize window
    pub async fn resize_window(&self, app_name: &str, width: u32, height: u32) -> Result<String> {
        info!("Resizing window {} to {}x{}", app_name, width, height);

        #[cfg(target_os = "linux")]
        {
            let status = Command::new("wmctrl")
                .args(&["-r", app_name, "-e", &format!("0,0,0,{},{}", width, height)])
                .status()
                .map_err(|e| {
                    LunaError::SystemOperation(format!("Failed to resize window: {}", e))
                })?;

            if status.success() {
                Ok(format!(
                    "Resized window {} to {}x{}",
                    app_name, width, height
                ))
            } else {
                Err(LunaError::SystemOperation(
                    "wmctrl not found. Install wmctrl".to_string(),
                ))
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            warn!("Resize not implemented for this platform");
            Err(LunaError::SystemOperation(
                "Resize not supported on this platform".to_string(),
            ))
        }
    }

    /// Close a window
    pub async fn close_window(&self, app_name: &str) -> Result<String> {
        info!("Closing window: {}", app_name);

        #[cfg(target_os = "linux")]
        {
            let status = Command::new("wmctrl")
                .args(&["-c", app_name])
                .status()
                .map_err(|e| {
                    LunaError::SystemOperation(format!("Failed to close window: {}", e))
                })?;

            if status.success() {
                Ok(format!("Closed window: {}", app_name))
            } else {
                Err(LunaError::SystemOperation(
                    "wmctrl not found. Install wmctrl".to_string(),
                ))
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            warn!("Close not implemented for this platform");
            Err(LunaError::SystemOperation(
                "Close not supported on this platform".to_string(),
            ))
        }
    }
}

impl Default for WindowControl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_window_control_creation() {
        let control = WindowControl::new();
        drop(control);
    }
}
