//! Common OS utilities
//!
//! Cross-platform OS utilities.

use crate::error::{Result, LunaError};
use std::env;

/// Get current username
pub fn get_username() -> Result<String> {
    // Try environment variable first
    if let Ok(user) = env::var("USER") {
        return Ok(user);
    }
    
    if let Ok(user) = env::var("USERNAME") {
        return Ok(user);
    }
    
    // Try whoami command as fallback
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("whoami").output() {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
    }
    
    Err(LunaError::SystemOperation("Failed to get username".to_string()))
}

/// Get current hostname
pub fn get_hostname() -> Result<String> {
    if let Ok(hostname) = env::var("HOSTNAME") {
        return Ok(hostname);
    }
    
    if let Ok(hostname) = env::var("COMPUTERNAME") {
        return Ok(hostname);
    }
    
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("hostname").output() {
            if output.status.success() {
                return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
    }
    
    Ok("unknown".to_string())
}

/// Get OS name and version
pub fn get_os_info() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("lsb_release")
            .args(["-ds"])
            .output()
        {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
        "Linux".to_string()
    }
    
    #[cfg(target_os = "windows")]
    {
        "Windows".to_string()
    }
    
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("sw_vers")
            .args(["-productVersion"])
            .output()
        {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return format!("macOS {}", version);
            }
        }
        "macOS".to_string()
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        "Unknown OS".to_string()
    }
}

/// Check if running with elevated privileges
pub fn is_elevated() -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("id").arg("-u").output() {
            if output.status.success() {
                let uid = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return uid == "0";
            }
        }
        false
    }
    
    #[cfg(target_os = "windows")]
    {
        // Check if running as administrator
        // This is a simplified check; full implementation would use Win32 APIs
        env::var("USERNAME").map(|u| u.to_lowercase() == "administrator").unwrap_or(false)
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("id").arg("-u").output() {
            if output.status.success() {
                let uid = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return uid == "0";
            }
        }
        false
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        false
    }
}

/// Get system uptime in seconds
pub fn get_uptime() -> Result<u64> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        let uptime_str = fs::read_to_string("/proc/uptime")
            .map_err(|e| LunaError::SystemOperation(format!("Failed to read uptime: {}", e)))?;
        
        let uptime = uptime_str
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|f| f as u64)
            .ok_or_else(|| LunaError::SystemOperation("Failed to parse uptime".to_string()))?;
        
        Ok(uptime)
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("sysctl")
            .args(["-n", "kern.boottime"])
            .output()
            .map_err(|e| LunaError::SystemOperation(format!("Failed to get uptime: {}", e)))?;
        
        if output.status.success() {
            let boot_time_str = String::from_utf8_lossy(&output.stdout);
            // Parse output like "{ sec = 1234567890, usec = 0 }"
            if let Some(sec_start) = boot_time_str.find("sec = ") {
                if let Some(comma) = boot_time_str[sec_start..].find(',') {
                    let sec_str = &boot_time_str[sec_start + 6..sec_start + comma];
                    if let Ok(boot_time) = sec_str.trim().parse::<u64>() {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        return Ok(now - boot_time);
                    }
                }
            }
        }
        
        Ok(0)
    }
    
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("wmic")
            .args(["os", "get", "lastbootuptime"])
            .output()
            .map_err(|e| LunaError::SystemOperation(format!("Failed to get uptime: {}", e)))?;
        
        if output.status.success() {
            // Parse WMI date format
            // This is simplified; full implementation would parse the exact format
            return Ok(0);
        }
        
        Ok(0)
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_username() {
        // Should not panic
        let _ = get_username();
    }
    
    #[test]
    fn test_get_hostname() {
        let hostname = get_hostname().unwrap();
        assert!(!hostname.is_empty());
    }
    
    #[test]
    fn test_get_os_info() {
        let os_info = get_os_info();
        assert!(!os_info.is_empty());
    }
}
