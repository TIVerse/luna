//! Application discovery
//!
//! Discovers installed applications across platforms.

use crate::db::schema::{Application, AppCategory};
use crate::error::Result;
use std::path::PathBuf;

#[cfg(target_os = "linux")]
use crate::os::linux::{get_snap_packages, get_flatpak_packages};

/// Discover all installed applications
pub async fn discover_applications() -> Result<Vec<Application>> {
    #[cfg(target_os = "linux")]
    {
        discover_linux_apps().await
    }
    
    #[cfg(target_os = "windows")]
    {
        discover_windows_apps().await
    }
    
    #[cfg(target_os = "macos")]
    {
        discover_macos_apps().await
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Ok(Vec::new())
    }
}

/// Discover applications on Linux
#[cfg(target_os = "linux")]
async fn discover_linux_apps() -> Result<Vec<Application>> {
    use std::fs;
    
    let mut apps = Vec::new();
    
    // Scan .desktop files
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
        
        if let Ok(entries) = fs::read_dir(&search_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                    if let Ok(app) = parse_linux_desktop_file(&path) {
                        apps.push(app);
                    }
                }
            }
        }
    }
    
    // Add snap packages
    if let Ok(snap_packages) = get_snap_packages() {
        for (name, path) in snap_packages {
            let mut app = Application::new(name.clone(), path);
            app.add_alias(format!("snap {}", name));
            apps.push(app);
        }
    }
    
    // Add flatpak packages
    if let Ok(flatpak_packages) = get_flatpak_packages() {
        for name in flatpak_packages {
            let exec_path = PathBuf::from(format!("/var/lib/flatpak/exports/bin/{}", name));
            let mut app = Application::new(name.clone(), exec_path);
            app.add_alias(format!("flatpak {}", name));
            apps.push(app);
        }
    }
    
    Ok(apps)
}

/// Parse a Linux .desktop file
#[cfg(target_os = "linux")]
fn parse_linux_desktop_file(path: &std::path::Path) -> Result<Application> {
    use std::fs;
    use crate::error::LunaError;
    
    let contents = fs::read_to_string(path)
        .map_err(|e| LunaError::SystemOperation(format!("Failed to read desktop file: {}", e)))?;
    
    let mut name = String::new();
    let mut exec = String::new();
    let mut icon = None;
    let mut categories = Vec::new();
    let mut comment = None;
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
                "Icon" => icon = Some(PathBuf::from(value.trim())),
                "Comment" => comment = Some(value.trim().to_string()),
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
    
    // Extract executable path (remove arguments)
    let exec_path = exec
        .split_whitespace()
        .next()
        .unwrap_or(&exec)
        .trim_matches('"');
    
    let mut app = Application::new(name, PathBuf::from(exec_path));
    app.category = categorize_linux_app(&categories);
    app.icon = icon;
    app.description = comment;
    
    Ok(app)
}

/// Categorize Linux app based on desktop categories
#[cfg(target_os = "linux")]
fn categorize_linux_app(categories: &[String]) -> AppCategory {
    for cat in categories {
        match cat.as_str() {
            "WebBrowser" | "Browser" => return AppCategory::Browser,
            "Development" | "IDE" => return AppCategory::IDE,
            "TextEditor" | "Editor" => return AppCategory::TextEditor,
            "TerminalEmulator" | "Terminal" => return AppCategory::Terminal,
            "AudioVideo" | "Audio" | "Video" => return AppCategory::Media,
            "Network" | "InstantMessaging" | "Chat" => return AppCategory::Communication,
            "Office" => return AppCategory::Office,
            "Game" => return AppCategory::Games,
            "System" | "Utility" => return AppCategory::System,
            _ => continue,
        }
    }
    AppCategory::Other
}

/// Discover applications on Windows
#[cfg(target_os = "windows")]
async fn discover_windows_apps() -> Result<Vec<Application>> {
    use std::fs;
    use winreg::enums::*;
    use winreg::RegKey;
    
    let mut apps = Vec::new();
    
    // Scan registry
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let uninstall_path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
    
    if let Ok(uninstall_key) = hklm.open_subkey(uninstall_path) {
        for key_name in uninstall_key.enum_keys().flatten() {
            if let Ok(app_key) = uninstall_key.open_subkey(&key_name) {
                let name: Result<String, _> = app_key.get_value("DisplayName");
                let icon_path: Result<String, _> = app_key.get_value("DisplayIcon");
                let install_location: Result<String, _> = app_key.get_value("InstallLocation");
                let version: Result<String, _> = app_key.get_value("DisplayVersion");
                
                if let Ok(name) = name {
                    // Try to find executable
                    let exe_path = if let Ok(location) = install_location {
                        // Look for common executable names
                        let base_path = PathBuf::from(&location);
                        let possible_exes = vec![
                            base_path.join(format!("{}.exe", &name)),
                            base_path.join("bin").join(format!("{}.exe", &name)),
                        ];
                        
                        possible_exes.into_iter().find(|p| p.exists())
                            .unwrap_or_else(|| base_path.join(format!("{}.exe", &name)))
                    } else if let Ok(icon) = icon_path {
                        // Use icon path as fallback
                        PathBuf::from(icon.split(',').next().unwrap_or(&icon))
                    } else {
                        continue;
                    };
                    
                    let mut app = Application::new(name, exe_path);
                    app.version = version.ok();
                    app.category = categorize_windows_app(&app.name);
                    apps.push(app);
                }
            }
        }
    }
    
    // Scan Start Menu
    let start_menu_paths = vec![
        PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs"),
        dirs::home_dir()
            .map(|h| h.join(r"AppData\Roaming\Microsoft\Windows\Start Menu\Programs"))
            .unwrap_or_else(|| PathBuf::from(r"C:\Temp")),
    ];
    
    for start_path in start_menu_paths {
        if start_path.exists() {
            scan_windows_directory(&start_path, &mut apps)?;
        }
    }
    
    Ok(apps)
}

/// Recursively scan Windows directory for shortcuts
#[cfg(target_os = "windows")]
fn scan_windows_directory(dir: &std::path::Path, apps: &mut Vec<Application>) -> Result<()> {
    use std::fs;
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let _ = scan_windows_directory(&path, apps);
            } else if path.extension().and_then(|s| s.to_str()) == Some("lnk") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    let mut app = Application::new(name.to_string(), path.clone());
                    app.category = categorize_windows_app(name);
                    apps.push(app);
                }
            }
        }
    }
    Ok(())
}

/// Categorize Windows app based on name
#[cfg(target_os = "windows")]
fn categorize_windows_app(name: &str) -> AppCategory {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("chrome") || name_lower.contains("firefox") 
        || name_lower.contains("edge") || name_lower.contains("browser") {
        AppCategory::Browser
    } else if name_lower.contains("visual studio") || name_lower.contains("intellij")
        || name_lower.contains("eclipse") || name_lower.contains("pycharm") {
        AppCategory::IDE
    } else if name_lower.contains("notepad") || name_lower.contains("sublime")
        || name_lower.contains("atom") || name_lower.contains("code") {
        AppCategory::TextEditor
    } else if name_lower.contains("terminal") || name_lower.contains("cmd")
        || name_lower.contains("powershell") {
        AppCategory::Terminal
    } else if name_lower.contains("media") || name_lower.contains("player")
        || name_lower.contains("spotify") || name_lower.contains("vlc") {
        AppCategory::Media
    } else if name_lower.contains("slack") || name_lower.contains("teams")
        || name_lower.contains("discord") || name_lower.contains("zoom") {
        AppCategory::Communication
    } else if name_lower.contains("word") || name_lower.contains("excel")
        || name_lower.contains("powerpoint") || name_lower.contains("office") {
        AppCategory::Office
    } else if name_lower.contains("game") {
        AppCategory::Games
    } else {
        AppCategory::Other
    }
}

/// Discover applications on macOS
#[cfg(target_os = "macos")]
async fn discover_macos_apps() -> Result<Vec<Application>> {
    use std::fs;
    use std::process::Command;
    
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
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        let mut app = Application::new(name.to_string(), path.clone());
                        
                        // Try to read version from Info.plist
                        let info_plist = path.join("Contents/Info.plist");
                        if info_plist.exists() {
                            if let Ok(output) = Command::new("defaults")
                                .args(["read", &info_plist.to_string_lossy(), "CFBundleShortVersionString"])
                                .output()
                            {
                                if output.status.success() {
                                    app.version = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
                                }
                            }
                        }
                        
                        app.category = categorize_macos_app(name);
                        apps.push(app);
                    }
                }
            }
        }
    }
    
    Ok(apps)
}

/// Categorize macOS app based on name
#[cfg(target_os = "macos")]
fn categorize_macos_app(name: &str) -> AppCategory {
    let name_lower = name.to_lowercase();
    
    if name_lower.contains("safari") || name_lower.contains("chrome") 
        || name_lower.contains("firefox") {
        AppCategory::Browser
    } else if name_lower.contains("xcode") || name_lower.contains("intellij")
        || name_lower.contains("pycharm") {
        AppCategory::IDE
    } else if name_lower.contains("textedit") || name_lower.contains("sublime")
        || name_lower.contains("atom") || name_lower.contains("code") {
        AppCategory::TextEditor
    } else if name_lower.contains("terminal") || name_lower.contains("iterm") {
        AppCategory::Terminal
    } else if name_lower.contains("music") || name_lower.contains("spotify")
        || name_lower.contains("vlc") || name_lower.contains("quicktime") {
        AppCategory::Media
    } else if name_lower.contains("messages") || name_lower.contains("slack")
        || name_lower.contains("teams") || name_lower.contains("discord") {
        AppCategory::Communication
    } else if name_lower.contains("pages") || name_lower.contains("numbers")
        || name_lower.contains("keynote") {
        AppCategory::Office
    } else if name_lower.contains("game") {
        AppCategory::Games
    } else {
        AppCategory::Other
    }
}
