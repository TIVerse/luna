//! macOS-specific implementation
//!
//! macOS system integration using Cocoa APIs.

#[cfg(target_os = "macos")]
use crate::error::{LunaError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "macos")]
/// macOS-specific functionality
pub struct MacOsInterface {
    /// Cache of installed applications
    app_cache: Vec<MacApp>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
struct MacApp {
    name: String,
    bundle_path: PathBuf,
    bundle_id: Option<String>,
    version: Option<String>,
}

#[cfg(target_os = "macos")]
impl MacOsInterface {
    /// Create a new macOS interface
    pub fn new() -> Result<Self> {
        Ok(Self {
            app_cache: Vec::new(),
        })
    }

    /// Initialize the interface by scanning Applications folder
    pub fn init(&mut self) -> Result<()> {
        self.app_cache = self.scan_applications()?;
        Ok(())
    }

    /// Scan for installed applications
    fn scan_applications(&self) -> Result<Vec<MacApp>> {
        let mut apps = Vec::new();

        let app_dirs = vec![
            PathBuf::from("/Applications"),
            PathBuf::from("/System/Applications"),
            dirs::home_dir()
                .map(|h| h.join("Applications"))
                .unwrap_or_else(|| PathBuf::from("/tmp")),
        ];

        for app_dir in app_dirs {
            if !app_dir.exists() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(&app_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("app") {
                        if let Ok(app) = self.parse_app_bundle(&path) {
                            apps.push(app);
                        }
                    }
                }
            }
        }

        Ok(apps)
    }

    /// Parse an .app bundle
    fn parse_app_bundle(&self, bundle_path: &Path) -> Result<MacApp> {
        let name = bundle_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| LunaError::SystemOperation("Invalid app bundle name".to_string()))?
            .to_string();

        // Try to read Info.plist
        let info_plist = bundle_path.join("Contents/Info.plist");
        let mut bundle_id = None;
        let mut version = None;

        if info_plist.exists() {
            // Use plutil to convert plist to JSON
            if let Ok(output) = Command::new("plutil")
                .args(["-convert", "json", "-o", "-"])
                .arg(&info_plist)
                .output()
            {
                if output.status.success() {
                    let json = String::from_utf8_lossy(&output.stdout);

                    // Simple JSON parsing for CFBundleIdentifier and CFBundleShortVersionString
                    if let Some(id_start) = json.find("\"CFBundleIdentifier\"") {
                        if let Some(value_start) = json[id_start..].find(':') {
                            let rest = &json[id_start + value_start + 1..];
                            if let Some(quote_start) = rest.find('"') {
                                if let Some(quote_end) = rest[quote_start + 1..].find('"') {
                                    bundle_id = Some(
                                        rest[quote_start + 1..quote_start + 1 + quote_end]
                                            .to_string(),
                                    );
                                }
                            }
                        }
                    }

                    if let Some(ver_start) = json.find("\"CFBundleShortVersionString\"") {
                        if let Some(value_start) = json[ver_start..].find(':') {
                            let rest = &json[ver_start + value_start + 1..];
                            if let Some(quote_start) = rest.find('"') {
                                if let Some(quote_end) = rest[quote_start + 1..].find('"') {
                                    version = Some(
                                        rest[quote_start + 1..quote_start + 1 + quote_end]
                                            .to_string(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(MacApp {
            name,
            bundle_path: bundle_path.to_path_buf(),
            bundle_id,
            version,
        })
    }
}

/// Launch an application on macOS
#[cfg(target_os = "macos")]
pub fn launch_app_macos(path: &Path) -> Result<()> {
    // Use 'open' command which handles .app bundles properly
    Command::new("open")
        .arg(path)
        .spawn()
        .map_err(|e| LunaError::SystemOperation(format!("Failed to launch app: {}", e)))?;

    Ok(())
}

/// Get current system volume (0-100)
#[cfg(target_os = "macos")]
pub fn get_volume_macos() -> Result<u8> {
    let output = Command::new("osascript")
        .args(["-e", "output volume of (get volume settings)"])
        .output()
        .map_err(|e| LunaError::SystemOperation(format!("Failed to get volume: {}", e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(volume) = stdout.trim().parse::<u8>() {
            return Ok(volume);
        }
    }

    // Fallback
    Ok(50)
}

/// Set system volume (0-100)
#[cfg(target_os = "macos")]
pub fn set_volume_macos(level: u8) -> Result<()> {
    let level = level.min(100);

    Command::new("osascript")
        .args(["-e", &format!("set volume output volume {}", level)])
        .status()
        .map_err(|e| LunaError::SystemOperation(format!("Failed to set volume: {}", e)))?;

    Ok(())
}

/// Get list of running applications
#[cfg(target_os = "macos")]
pub fn get_running_apps() -> Result<Vec<String>> {
    let output = Command::new("osascript")
        .args([
            "-e",
            "tell application \"System Events\" to get name of every application process",
        ])
        .output()
        .map_err(|e| LunaError::SystemOperation(format!("Failed to get running apps: {}", e)))?;

    let mut apps = Vec::new();

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        apps = stdout.split(',').map(|s| s.trim().to_string()).collect();
    }

    Ok(apps)
}

/// Use Spotlight to search for files
#[cfg(target_os = "macos")]
pub fn spotlight_search(query: &str) -> Result<Vec<PathBuf>> {
    let output = Command::new("mdfind")
        .arg(query)
        .output()
        .map_err(|e| LunaError::SystemOperation(format!("Spotlight search failed: {}", e)))?;

    let mut results = Vec::new();

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            results.push(PathBuf::from(line));
        }
    }

    Ok(results)
}
