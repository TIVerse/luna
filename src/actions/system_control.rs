//! System control functions
//!
//! Volume, brightness, power management with cross-platform support.

use crate::error::{LunaError, Result};
use std::process::Command;
use tracing::{debug, info, warn};

/// System control handler
pub struct SystemControl;

impl SystemControl {
    /// Create a new system control handler
    pub fn new() -> Self {
        Self
    }

    /// Set system volume (0-100)
    pub async fn set_volume(&self, level: u8) -> Result<String> {
        info!("Setting volume to {}%", level);

        if level > 100 {
            return Err(LunaError::InvalidParameter(
                "Volume must be 0-100".to_string(),
            ));
        }

        #[cfg(target_os = "linux")]
        {
            // Try PulseAudio first, then ALSA
            let result = Command::new("pactl")
                .args(&["set-sink-volume", "@DEFAULT_SINK@", &format!("{}%", level)])
                .status();

            if result.is_ok() && result.unwrap().success() {
                return Ok(format!("Volume set to {}%", level));
            }

            // Fallback to amixer (ALSA)
            let status = Command::new("amixer")
                .args(&["set", "Master", &format!("{}%", level)])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to set volume: {}", e)))?;

            if status.success() {
                Ok(format!("Volume set to {}%", level))
            } else {
                Err(LunaError::SystemOperation(
                    "Failed to set volume".to_string(),
                ))
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Use nircmd on Windows
            let status = Command::new("nircmd")
                .args(&["setsysvolume", &((level as u32 * 655).to_string())])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to set volume: {}", e)))?;

            if status.success() {
                Ok(format!("Volume set to {}%", level))
            } else {
                Err(LunaError::SystemOperation(
                    "nircmd not found. Please install nircmd".to_string(),
                ))
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Use osascript on macOS
            let status = Command::new("osascript")
                .args(&["-e", &format!("set volume output volume {}", level)])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to set volume: {}", e)))?;

            if status.success() {
                Ok(format!("Volume set to {}%", level))
            } else {
                Err(LunaError::SystemOperation(
                    "Failed to set volume".to_string(),
                ))
            }
        }
    }

    /// Adjust volume by delta (-100 to +100)
    pub async fn adjust_volume(&self, delta: i8) -> Result<String> {
        info!("Adjusting volume by {}", delta);

        #[cfg(target_os = "linux")]
        {
            let sign = if delta > 0 { "+" } else { "" };
            let status = Command::new("pactl")
                .args(&[
                    "set-sink-volume",
                    "@DEFAULT_SINK@",
                    &format!("{}{}%", sign, delta),
                ])
                .status()
                .map_err(|e| {
                    LunaError::SystemOperation(format!("Failed to adjust volume: {}", e))
                })?;

            if status.success() {
                Ok(format!("Volume adjusted by {}", delta))
            } else {
                Err(LunaError::SystemOperation(
                    "Failed to adjust volume".to_string(),
                ))
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(LunaError::SystemOperation(
                "Volume adjustment not implemented for this platform".to_string(),
            ))
        }
    }

    /// Set screen brightness (0-100)
    pub async fn set_brightness(&self, level: u8) -> Result<String> {
        info!("Setting brightness to {}%", level);

        if level > 100 {
            return Err(LunaError::InvalidParameter(
                "Brightness must be 0-100".to_string(),
            ));
        }

        #[cfg(target_os = "linux")]
        {
            // Try xbacklight first
            let result = Command::new("xbacklight")
                .args(&["-set", &level.to_string()])
                .status();

            if result.is_ok() && result.unwrap().success() {
                return Ok(format!("Brightness set to {}%", level));
            }

            // Fallback to brightnessctl
            let status = Command::new("brightnessctl")
                .args(&["set", &format!("{}%", level)])
                .status()
                .map_err(|e| {
                    LunaError::SystemOperation(format!("Failed to set brightness: {}", e))
                })?;

            if status.success() {
                Ok(format!("Brightness set to {}%", level))
            } else {
                Err(LunaError::SystemOperation(
                    "Failed to set brightness. Install xbacklight or brightnessctl".to_string(),
                ))
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows brightness control requires WMI
            warn!("Brightness control not implemented for Windows");
            Err(LunaError::SystemOperation(
                "Brightness control not supported on Windows".to_string(),
            ))
        }

        #[cfg(target_os = "macos")]
        {
            // Use brightness command on macOS
            let status = Command::new("brightness")
                .arg(&format!("{}", level as f32 / 100.0))
                .status()
                .map_err(|e| {
                    LunaError::SystemOperation(format!("Failed to set brightness: {}", e))
                })?;

            if status.success() {
                Ok(format!("Brightness set to {}%", level))
            } else {
                Err(LunaError::SystemOperation(
                    "brightness command not found".to_string(),
                ))
            }
        }
    }

    /// Lock the computer
    pub async fn lock_computer(&self) -> Result<String> {
        info!("Locking computer");

        #[cfg(target_os = "linux")]
        {
            // Try multiple lock commands
            let lock_commands = [
                "gnome-screensaver-command -l",
                "xdg-screensaver lock",
                "loginctl lock-session",
            ];

            for cmd in &lock_commands {
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                let result = Command::new(parts[0]).args(&parts[1..]).status();

                if result.is_ok() && result.unwrap().success() {
                    return Ok("Computer locked".to_string());
                }
            }

            Err(LunaError::SystemOperation(
                "Failed to lock computer. No screen locker found".to_string(),
            ))
        }

        #[cfg(target_os = "windows")]
        {
            let status = Command::new("rundll32")
                .args(&["user32.dll,LockWorkStation"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to lock: {}", e)))?;

            if status.success() {
                Ok("Computer locked".to_string())
            } else {
                Err(LunaError::SystemOperation(
                    "Failed to lock computer".to_string(),
                ))
            }
        }

        #[cfg(target_os = "macos")]
        {
            let status = Command::new("pmset")
                .args(&["displaysleepnow"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to lock: {}", e)))?;

            if status.success() {
                Ok("Computer locked".to_string())
            } else {
                Err(LunaError::SystemOperation(
                    "Failed to lock computer".to_string(),
                ))
            }
        }
    }

    /// Shutdown the computer
    pub async fn shutdown(&self) -> Result<String> {
        info!("Shutting down computer");

        #[cfg(target_os = "linux")]
        {
            let status = Command::new("systemctl")
                .arg("poweroff")
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to shutdown: {}", e)))?;

            if status.success() {
                Ok("Shutting down...".to_string())
            } else {
                // Fallback to shutdown command
                Command::new("shutdown")
                    .args(&["-h", "now"])
                    .status()
                    .map_err(|e| {
                        LunaError::SystemOperation(format!("Failed to shutdown: {}", e))
                    })?;
                Ok("Shutting down...".to_string())
            }
        }

        #[cfg(target_os = "windows")]
        {
            let status = Command::new("shutdown")
                .args(&["/s", "/t", "0"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to shutdown: {}", e)))?;

            if status.success() {
                Ok("Shutting down...".to_string())
            } else {
                Err(LunaError::SystemOperation("Failed to shutdown".to_string()))
            }
        }

        #[cfg(target_os = "macos")]
        {
            let status = Command::new("osascript")
                .args(&["-e", "tell app \"System Events\" to shut down"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to shutdown: {}", e)))?;

            if status.success() {
                Ok("Shutting down...".to_string())
            } else {
                Err(LunaError::SystemOperation("Failed to shutdown".to_string()))
            }
        }
    }

    /// Restart the computer
    pub async fn restart(&self) -> Result<String> {
        info!("Restarting computer");

        #[cfg(target_os = "linux")]
        {
            let status = Command::new("systemctl")
                .arg("reboot")
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to restart: {}", e)))?;

            if status.success() {
                Ok("Restarting...".to_string())
            } else {
                Err(LunaError::SystemOperation("Failed to restart".to_string()))
            }
        }

        #[cfg(target_os = "windows")]
        {
            let status = Command::new("shutdown")
                .args(&["/r", "/t", "0"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to restart: {}", e)))?;

            if status.success() {
                Ok("Restarting...".to_string())
            } else {
                Err(LunaError::SystemOperation("Failed to restart".to_string()))
            }
        }

        #[cfg(target_os = "macos")]
        {
            let status = Command::new("osascript")
                .args(&["-e", "tell app \"System Events\" to restart"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to restart: {}", e)))?;

            if status.success() {
                Ok("Restarting...".to_string())
            } else {
                Err(LunaError::SystemOperation("Failed to restart".to_string()))
            }
        }
    }

    /// Mute/unmute system audio
    pub async fn toggle_mute(&self) -> Result<String> {
        info!("Toggling mute");

        #[cfg(target_os = "linux")]
        {
            let status = Command::new("pactl")
                .args(&["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to toggle mute: {}", e)))?;

            if status.success() {
                Ok("Audio mute toggled".to_string())
            } else {
                Err(LunaError::SystemOperation(
                    "Failed to toggle mute".to_string(),
                ))
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(LunaError::SystemOperation(
                "Mute toggle not implemented for this platform".to_string(),
            ))
        }
    }
}

impl Default for SystemControl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_control_creation() {
        let control = SystemControl::new();
        // Just verify it can be created
        drop(control);
    }

    #[tokio::test]
    async fn test_invalid_volume() {
        let control = SystemControl::new();
        let result = control.set_volume(150).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_brightness() {
        let control = SystemControl::new();
        let result = control.set_brightness(150).await;
        assert!(result.is_err());
    }
}
