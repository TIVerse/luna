//! Linux-specific implementation
//!
//! Linux system integration using D-Bus and .desktop files.

#[cfg(target_os = "linux")]
use crate::error::{Result, LunaError};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

#[cfg(target_os = "linux")]
/// Linux-specific functionality
pub struct LinuxInterface {
    /// Cache of desktop files
    desktop_cache: Vec<DesktopEntry>,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
struct DesktopEntry {
    name: String,
    exec: String,
    icon: Option<String>,
    categories: Vec<String>,
}

#[cfg(target_os = "linux")]
impl LinuxInterface {
    /// Create a new Linux interface
    pub fn new() -> Result<Self> {
        Ok(Self {
            desktop_cache: Vec::new(),
        })
    }
    
    /// Initialize the interface by scanning desktop files
    pub fn init(&mut self) -> Result<()> {
        self.desktop_cache = self.scan_desktop_files()?;
        Ok(())
    }
    
    /// Scan for .desktop files
    fn scan_desktop_files(&self) -> Result<Vec<DesktopEntry>> {
        let mut entries = Vec::new();
        let search_paths = vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
            dirs::home_dir()
                .map(|h| h.join(".local/share/applications"))
                .unwrap_or_else(|| PathBuf::from("/tmp")),
        ];
        
        for search_path in search_paths {
            if !search_path.exists() {
                continue;
            }
            
            if let Ok(entries_iter) = fs::read_dir(&search_path) {
                for entry in entries_iter.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                        if let Ok(desktop_entry) = self.parse_desktop_file(&path) {
                            entries.push(desktop_entry);
                        }
                    }
                }
            }
        }
        
        Ok(entries)
    }
    
    /// Parse a .desktop file
    fn parse_desktop_file(&self, path: &Path) -> Result<DesktopEntry> {
        let contents = fs::read_to_string(path)
            .map_err(|e| LunaError::SystemOperation(format!("Failed to read desktop file: {}", e)))?;
        
        let mut name = String::new();
        let mut exec = String::new();
        let mut icon = None;
        let mut categories = Vec::new();
        let mut in_desktop_entry = false;
        
        for line in contents.lines() {
            let line = line.trim();
            
            if line == "[Desktop Entry]" {
                in_desktop_entry = true;
                continue;
            }
            
            if line.starts_with('[') && line.ends_with(']') {
                in_desktop_entry = false;
                continue;
            }
            
            if !in_desktop_entry {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "Name" => name = value.trim().to_string(),
                    "Exec" => exec = value.trim().to_string(),
                    "Icon" => icon = Some(value.trim().to_string()),
                    "Categories" => {
                        categories = value
                            .split(';')
                            .filter(|s| !s.is_empty())
                            .map(|s| s.trim().to_string())
                            .collect();
                    }
                    _ => {}
                }
            }
        }
        
        if name.is_empty() || exec.is_empty() {
            return Err(LunaError::SystemOperation("Invalid desktop file: missing Name or Exec".to_string()));
        }
        
        Ok(DesktopEntry {
            name,
            exec,
            icon,
            categories,
        })
    }
}

/// Launch an application on Linux
#[cfg(target_os = "linux")]
pub fn launch_app_linux(path: &Path) -> Result<()> {
    // Try xdg-open first (works with .desktop files and paths)
    let output = Command::new("xdg-open")
        .arg(path)
        .spawn();
    
    if output.is_ok() {
        return Ok(());
    }
    
    // Fallback: try to execute directly
    Command::new(path)
        .spawn()
        .map_err(|e| LunaError::SystemOperation(format!("Failed to launch app: {}", e)))?;
    
    Ok(())
}

/// Get current system volume (0-100)
#[cfg(target_os = "linux")]
pub fn get_volume_linux() -> Result<u8> {
    // Try pactl (PulseAudio) first
    if let Ok(output) = Command::new("pactl")
        .args(["get-sink-volume", "@DEFAULT_SINK@"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse output like "Volume: front-left: 65536 / 100% / 0.00 dB"
            if let Some(percent) = stdout.split('/').nth(1) {
                if let Some(num) = percent.trim().strip_suffix('%') {
                    if let Ok(volume) = num.trim().parse::<u8>() {
                        return Ok(volume);
                    }
                }
            }
        }
    }
    
    // Fallback: try amixer (ALSA)
    if let Ok(output) = Command::new("amixer")
        .args(["get", "Master"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse output like "[80%]"
            if let Some(start) = stdout.rfind('[') {
                if let Some(end) = stdout[start..].find('%') {
                    let volume_str = &stdout[start + 1..start + end];
                    if let Ok(volume) = volume_str.parse::<u8>() {
                        return Ok(volume);
                    }
                }
            }
        }
    }
    
    // Default fallback
    Ok(50)
}

/// Set system volume (0-100)
#[cfg(target_os = "linux")]
pub fn set_volume_linux(level: u8) -> Result<()> {
    let level = level.min(100);
    
    // Try pactl (PulseAudio) first
    let pactl_result = Command::new("pactl")
        .args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{}%", level)])
        .status();
    
    if pactl_result.is_ok() && pactl_result.unwrap().success() {
        return Ok(());
    }
    
    // Fallback: try amixer (ALSA)
    let amixer_result = Command::new("amixer")
        .args(["set", "Master", &format!("{}%", level)])
        .status();
    
    if amixer_result.is_ok() && amixer_result.unwrap().success() {
        return Ok(());
    }
    
    Err(LunaError::SystemOperation("Failed to set volume: no audio system available".to_string()))
}

/// Get list of installed packages
#[cfg(target_os = "linux")]
pub fn get_installed_packages() -> Result<Vec<String>> {
    let mut packages = Vec::new();
    
    // Try dpkg (Debian/Ubuntu)
    if let Ok(output) = Command::new("dpkg")
        .args(["-l"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(5) { // Skip header lines
                if let Some(name) = line.split_whitespace().nth(1) {
                    packages.push(name.to_string());
                }
            }
            return Ok(packages);
        }
    }
    
    // Try rpm (RedHat/Fedora)
    if let Ok(output) = Command::new("rpm")
        .args(["-qa"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            packages.extend(stdout.lines().map(|s| s.to_string()));
            return Ok(packages);
        }
    }
    
    Ok(packages)
}

/// Get list of snap packages
#[cfg(target_os = "linux")]
pub fn get_snap_packages() -> Result<Vec<(String, PathBuf)>> {
    let mut packages = Vec::new();
    
    if let Ok(output) = Command::new("snap")
        .args(["list"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) { // Skip header
                if let Some(name) = line.split_whitespace().next() {
                    let path = PathBuf::from(format!("/snap/bin/{}", name));
                    if path.exists() {
                        packages.push((name.to_string(), path));
                    }
                }
            }
        }
    }
    
    Ok(packages)
}

/// Get list of flatpak packages
#[cfg(target_os = "linux")]
pub fn get_flatpak_packages() -> Result<Vec<String>> {
    let mut packages = Vec::new();
    
    if let Ok(output) = Command::new("flatpak")
        .args(["list", "--app"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(name) = line.split('\t').nth(1) {
                    packages.push(name.to_string());
                }
            }
        }
    }
    
    Ok(packages)
}
