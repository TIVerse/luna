//! Windows-specific implementation
//!
//! Windows system integration using Win32 APIs.

#[cfg(target_os = "windows")]
use crate::error::{Result, LunaError};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW};
#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

#[cfg(target_os = "windows")]
/// Windows-specific functionality
pub struct WindowsInterface {
    /// Cache of installed applications
    app_cache: Vec<InstalledApp>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone)]
struct InstalledApp {
    name: String,
    path: PathBuf,
    version: Option<String>,
    publisher: Option<String>,
}

#[cfg(target_os = "windows")]
impl WindowsInterface {
    /// Create a new Windows interface
    pub fn new() -> Result<Self> {
        Ok(Self {
            app_cache: Vec::new(),
        })
    }
    
    /// Initialize the interface by scanning registry and start menu
    pub fn init(&mut self) -> Result<()> {
        self.app_cache = self.scan_installed_apps()?;
        Ok(())
    }
    
    /// Scan for installed applications
    fn scan_installed_apps(&self) -> Result<Vec<InstalledApp>> {
        let mut apps = Vec::new();
        
        // Scan registry
        apps.extend(self.scan_registry()?);
        
        // Scan Start Menu
        apps.extend(self.scan_start_menu()?);
        
        // Scan Program Files
        apps.extend(self.scan_program_files()?);
        
        Ok(apps)
    }
    
    /// Scan Windows Registry for installed applications
    fn scan_registry(&self) -> Result<Vec<InstalledApp>> {
        let mut apps = Vec::new();
        
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let uninstall_path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
        
        if let Ok(uninstall_key) = hklm.open_subkey(uninstall_path) {
            for key_name in uninstall_key.enum_keys().flatten() {
                if let Ok(app_key) = uninstall_key.open_subkey(&key_name) {
                    let name: Result<String, _> = app_key.get_value("DisplayName");
                    let path: Result<String, _> = app_key.get_value("InstallLocation");
                    let version: Result<String, _> = app_key.get_value("DisplayVersion");
                    let publisher: Result<String, _> = app_key.get_value("Publisher");
                    
                    if let (Ok(name), Ok(path_str)) = (name, path) {
                        let path = PathBuf::from(path_str);
                        apps.push(InstalledApp {
                            name,
                            path,
                            version: version.ok(),
                            publisher: publisher.ok(),
                        });
                    }
                }
            }
        }
        
        Ok(apps)
    }
    
    /// Scan Start Menu for shortcuts
    fn scan_start_menu(&self) -> Result<Vec<InstalledApp>> {
        let mut apps = Vec::new();
        
        let start_menu_paths = vec![
            PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs"),
            dirs::home_dir()
                .map(|h| h.join(r"AppData\Roaming\Microsoft\Windows\Start Menu\Programs"))
                .unwrap_or_else(|| PathBuf::from(r"C:\Temp")),
        ];
        
        for start_path in start_menu_paths {
            if start_path.exists() {
                self.scan_directory_for_shortcuts(&start_path, &mut apps)?;
            }
        }
        
        Ok(apps)
    }
    
    /// Recursively scan directory for .lnk files
    fn scan_directory_for_shortcuts(&self, dir: &Path, apps: &mut Vec<InstalledApp>) -> Result<()> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let _ = self.scan_directory_for_shortcuts(&path, apps);
                } else if path.extension().and_then(|s| s.to_str()) == Some("lnk") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        apps.push(InstalledApp {
                            name: name.to_string(),
                            path: path.clone(),
                            version: None,
                            publisher: None,
                        });
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Scan Program Files directories
    fn scan_program_files(&self) -> Result<Vec<InstalledApp>> {
        let mut apps = Vec::new();
        
        let program_dirs = vec![
            PathBuf::from(r"C:\Program Files"),
            PathBuf::from(r"C:\Program Files (x86)"),
        ];
        
        for prog_dir in program_dirs {
            if !prog_dir.exists() {
                continue;
            }
            
            if let Ok(entries) = fs::read_dir(&prog_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                            // Look for common executable names
                            let exe_path = path.join(format!("{}.exe", name));
                            if exe_path.exists() {
                                apps.push(InstalledApp {
                                    name: name.to_string(),
                                    path: exe_path,
                                    version: None,
                                    publisher: None,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(apps)
    }
}

/// Launch an application on Windows
#[cfg(target_os = "windows")]
pub fn launch_app_windows(path: &Path) -> Result<()> {
    Command::new(path)
        .spawn()
        .map_err(|e| LunaError::SystemOperation(format!("Failed to launch app: {}", e)))?;
    Ok(())
}

/// Get current system volume (0-100)
#[cfg(target_os = "windows")]
pub fn get_volume_windows() -> Result<u8> {
    // Use PowerShell to get volume
    let output = Command::new("powershell")
        .args(["-Command", "(Get-AudioDevice -Playback).Volume"])
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
#[cfg(target_os = "windows")]
pub fn set_volume_windows(level: u8) -> Result<()> {
    let level = level.min(100);
    
    // Use nircmd or PowerShell to set volume
    let result = Command::new("powershell")
        .args([
            "-Command",
            &format!("(New-Object -ComObject WScript.Shell).SendKeys([char]174)"),
        ])
        .status();
    
    if result.is_err() {
        return Err(LunaError::SystemOperation("Failed to set volume".to_string()));
    }
    
    Ok(())
}

/// Get list of running processes
#[cfg(target_os = "windows")]
pub fn get_running_processes() -> Result<Vec<String>> {
    let output = Command::new("tasklist")
        .arg("/FO")
        .arg("CSV")
        .arg("/NH")
        .output()
        .map_err(|e| LunaError::SystemOperation(format!("Failed to get processes: {}", e)))?;
    
    let mut processes = Vec::new();
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            // Parse CSV format: "process.exe","PID","Session","Mem Usage"
            if let Some(name) = line.split(',').next() {
                let name = name.trim_matches('"');
                processes.push(name.to_string());
            }
        }
    }
    
    Ok(processes)
}
