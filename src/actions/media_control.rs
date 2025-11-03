//! Media playback control
//!
//! Control media playback (play, pause, next, etc.) with cross-platform support.

use crate::error::{LunaError, Result};
use std::process::Command;
use tracing::{debug, info};

/// Media control handler
pub struct MediaControl;

impl MediaControl {
    /// Create a new media control handler
    pub fn new() -> Self {
        Self
    }
    
    /// Play/pause media
    pub async fn play_pause(&self) -> Result<String> {
        info!("Toggling play/pause");
        
        #[cfg(target_os = "linux")]
        {
            // Use playerctl (MPRIS)
            let status = Command::new("playerctl")
                .arg("play-pause")
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to control media: {}", e)))?;
            
            if status.success() {
                Ok("Media play/pause toggled".to_string())
            } else {
                Err(LunaError::SystemOperation(
                    "playerctl not found. Install playerctl for media control".to_string()
                ))
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // Use nircmd to simulate media key
            let status = Command::new("nircmd")
                .args(&["sendkeypress", "0xB3"]) // VK_MEDIA_PLAY_PAUSE
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to control media: {}", e)))?;
            
            if status.success() {
                Ok("Media play/pause toggled".to_string())
            } else {
                Err(LunaError::SystemOperation(
                    "nircmd not found. Install nircmd for media control".to_string()
                ))
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            // Use osascript to control Music/iTunes
            let status = Command::new("osascript")
                .args(&["-e", "tell application \"Music\" to playpause"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to control media: {}", e)))?;
            
            if status.success() {
                Ok("Media play/pause toggled".to_string())
            } else {
                Err(LunaError::SystemOperation("Failed to control Music app".to_string()))
            }
        }
    }
    
    /// Next track
    pub async fn next_track(&self) -> Result<String> {
        info!("Skipping to next track");
        
        #[cfg(target_os = "linux")]
        {
            let status = Command::new("playerctl")
                .arg("next")
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to skip track: {}", e)))?;
            
            if status.success() {
                Ok("Skipped to next track".to_string())
            } else {
                Err(LunaError::SystemOperation(
                    "playerctl not found. Install playerctl for media control".to_string()
                ))
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            let status = Command::new("nircmd")
                .args(&["sendkeypress", "0xB0"]) // VK_MEDIA_NEXT_TRACK
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to skip track: {}", e)))?;
            
            if status.success() {
                Ok("Skipped to next track".to_string())
            } else {
                Err(LunaError::SystemOperation(
                    "nircmd not found. Install nircmd for media control".to_string()
                ))
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            let status = Command::new("osascript")
                .args(&["-e", "tell application \"Music\" to next track"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to skip track: {}", e)))?;
            
            if status.success() {
                Ok("Skipped to next track".to_string())
            } else {
                Err(LunaError::SystemOperation("Failed to control Music app".to_string()))
            }
        }
    }
    
    /// Previous track
    pub async fn previous_track(&self) -> Result<String> {
        info!("Going to previous track");
        
        #[cfg(target_os = "linux")]
        {
            let status = Command::new("playerctl")
                .arg("previous")
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to go to previous track: {}", e)))?;
            
            if status.success() {
                Ok("Went to previous track".to_string())
            } else {
                Err(LunaError::SystemOperation(
                    "playerctl not found. Install playerctl for media control".to_string()
                ))
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            let status = Command::new("nircmd")
                .args(&["sendkeypress", "0xB1"]) // VK_MEDIA_PREV_TRACK
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to go to previous track: {}", e)))?;
            
            if status.success() {
                Ok("Went to previous track".to_string())
            } else {
                Err(LunaError::SystemOperation(
                    "nircmd not found. Install nircmd for media control".to_string()
                ))
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            let status = Command::new("osascript")
                .args(&["-e", "tell application \"Music\" to previous track"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to go to previous track: {}", e)))?;
            
            if status.success() {
                Ok("Went to previous track".to_string())
            } else {
                Err(LunaError::SystemOperation("Failed to control Music app".to_string()))
            }
        }
    }
    
    /// Stop playback
    pub async fn stop(&self) -> Result<String> {
        info!("Stopping playback");
        
        #[cfg(target_os = "linux")]
        {
            let status = Command::new("playerctl")
                .arg("stop")
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to stop playback: {}", e)))?;
            
            if status.success() {
                Ok("Playback stopped".to_string())
            } else {
                Err(LunaError::SystemOperation(
                    "playerctl not found. Install playerctl for media control".to_string()
                ))
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            let status = Command::new("nircmd")
                .args(&["sendkeypress", "0xB2"]) // VK_MEDIA_STOP
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to stop playback: {}", e)))?;
            
            if status.success() {
                Ok("Playback stopped".to_string())
            } else {
                Err(LunaError::SystemOperation(
                    "nircmd not found. Install nircmd for media control".to_string()
                ))
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            let status = Command::new("osascript")
                .args(&["-e", "tell application \"Music\" to stop"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to stop playback: {}", e)))?;
            
            if status.success() {
                Ok("Playback stopped".to_string())
            } else {
                Err(LunaError::SystemOperation("Failed to control Music app".to_string()))
            }
        }
    }
    
    /// Get current playback status
    pub async fn get_status(&self) -> Result<String> {
        debug!("Getting playback status");
        
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("playerctl")
                .arg("status")
                .output()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to get status: {}", e)))?;
            
            if output.status.success() {
                let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(format!("Playback status: {}", status))
            } else {
                Err(LunaError::SystemOperation(
                    "playerctl not found or no player active".to_string()
                ))
            }
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            Ok("Playback status unavailable on this platform".to_string())
        }
    }
    
    /// Get currently playing track info
    pub async fn get_current_track(&self) -> Result<String> {
        debug!("Getting current track info");
        
        #[cfg(target_os = "linux")]
        {
            let output = Command::new("playerctl")
                .args(&["metadata", "--format", "{{ artist }} - {{ title }}"])
                .output()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to get track info: {}", e)))?;
            
            if output.status.success() {
                let track = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if track.is_empty() {
                    Ok("No track playing".to_string())
                } else {
                    Ok(format!("Now playing: {}", track))
                }
            } else {
                Err(LunaError::SystemOperation(
                    "playerctl not found or no player active".to_string()
                ))
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("osascript")
                .args(&["-e", "tell application \"Music\" to get name of current track & \" - \" & artist of current track"])
                .output()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to get track info: {}", e)))?;
            
            if output.status.success() {
                let track = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Ok(format!("Now playing: {}", track))
            } else {
                Ok("No track playing".to_string())
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            Ok("Track info unavailable on Windows".to_string())
        }
    }
}

impl Default for MediaControl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_media_control_creation() {
        let control = MediaControl::new();
        // Just verify it can be created
        drop(control);
    }
}
