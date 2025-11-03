# LUNA OS Integration - Developer Guide

## Quick Start

### Basic Usage

```rust
use luna::os::OsInterface;

// Initialize the OS interface
let mut os_interface = OsInterface::new()?;
os_interface.init()?;

// Launch an application
let app_path = std::path::Path::new("/usr/bin/firefox");
os_interface.launch_app(app_path)?;

// Control system volume
let current_volume = os_interface.get_volume()?;
println!("Current volume: {}%", current_volume);

os_interface.set_volume(75)?;
```

### Application Discovery

```rust
use luna::os::discovery::discover_applications;

// Discover all installed applications
let apps = discover_applications().await?;

for app in apps {
    println!("{}: {}", app.name, app.executable.display());
    println!("  Category: {:?}", app.category);
    println!("  Aliases: {:?}", app.aliases);
}
```

### Common Utilities

```rust
use luna::os::common::*;

// Get system information
let username = get_username()?;
let hostname = get_hostname()?;
let os_info = get_os_info();
let uptime = get_uptime()?;

println!("User: {} on {}", username, hostname);
println!("OS: {}", os_info);
println!("Uptime: {} seconds", uptime);

// Check privileges
if is_elevated() {
    println!("Running with elevated privileges");
}
```

## Platform-Specific Features

### Linux

```rust
#[cfg(target_os = "linux")]
use luna::os::linux::*;

// Get package information
let packages = get_installed_packages()?;
let snap_packages = get_snap_packages()?;
let flatpak_packages = get_flatpak_packages()?;

println!("Total packages: {}", packages.len());
```

### Windows

```rust
#[cfg(target_os = "windows")]
use luna::os::windows::*;

// Get running processes
let processes = get_running_processes()?;

for process in processes {
    println!("Process: {}", process);
}
```

### macOS

```rust
#[cfg(target_os = "macos")]
use luna::os::macos::*;

// Spotlight search
let results = spotlight_search("kind:document")?;

for path in results {
    println!("Found: {}", path.display());
}

// Get running applications
let running_apps = get_running_apps()?;
```

## Application Categories

The discovery system automatically categorizes applications:

- **Browser** - Web browsers (Chrome, Firefox, Safari, Edge)
- **IDE** - Development environments (VS Code, IntelliJ, Xcode)
- **TextEditor** - Text editors (Notepad++, Sublime, vim)
- **Terminal** - Terminal emulators (GNOME Terminal, iTerm2)
- **Media** - Media players (VLC, Spotify, Music)
- **Communication** - Chat apps (Slack, Discord, Teams)
- **Office** - Productivity apps (Word, Excel, Pages)
- **Games** - Gaming applications
- **System** - System utilities
- **Other** - Uncategorized applications

## Error Handling

All OS operations return `Result<T, LunaError>`:

```rust
use luna::error::{Result, LunaError};

fn example() -> Result<()> {
    match os_interface.launch_app(path) {
        Ok(_) => println!("App launched successfully"),
        Err(LunaError::SystemOperation(msg)) => {
            eprintln!("Failed to launch app: {}", msg);
        }
        Err(e) => eprintln!("Unexpected error: {}", e),
    }
    Ok(())
}
```

## Integration with LUNA Actions

### App Launcher Action

```rust
use luna::actions::app_launcher::AppLauncher;
use luna::os::OsInterface;

let mut app_launcher = AppLauncher::new(os_interface)?;
app_launcher.launch("firefox")?;
```

### System Control Action

```rust
use luna::actions::system_control::SystemControl;
use luna::os::OsInterface;

let system_control = SystemControl::new(os_interface)?;
system_control.set_volume(50)?;
```

## Performance Considerations

### Application Discovery Caching

Discovery is expensive; cache results:

```rust
use std::time::{Duration, Instant};
use std::sync::RwLock;

struct AppCache {
    apps: RwLock<Vec<Application>>,
    last_update: RwLock<Instant>,
}

impl AppCache {
    async fn get_apps(&self) -> Result<Vec<Application>> {
        let last_update = *self.last_update.read().unwrap();
        let cache_age = last_update.elapsed();
        
        if cache_age > Duration::from_secs(3600) { // 1 hour
            let apps = discover_applications().await?;
            *self.apps.write().unwrap() = apps.clone();
            *self.last_update.write().unwrap() = Instant::now();
            Ok(apps)
        } else {
            Ok(self.apps.read().unwrap().clone())
        }
    }
}
```

### Background Refresh

```rust
use tokio::time::{interval, Duration};

async fn background_refresh(cache: Arc<AppCache>) {
    let mut ticker = interval(Duration::from_secs(3600));
    
    loop {
        ticker.tick().await;
        if let Err(e) = cache.refresh().await {
            eprintln!("Failed to refresh app cache: {}", e);
        }
    }
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_os_interface_creation() {
        let result = OsInterface::new();
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_app_discovery() {
        let apps = discover_applications().await;
        assert!(apps.is_ok());
        
        let apps = apps.unwrap();
        assert!(!apps.is_empty());
    }
    
    #[test]
    fn test_volume_control() {
        let os_interface = OsInterface::new().unwrap();
        
        // Get current volume
        let volume = os_interface.get_volume();
        assert!(volume.is_ok());
        
        // Set volume
        let result = os_interface.set_volume(50);
        assert!(result.is_ok());
    }
}
```

### Integration Tests

```rust
// tests/os_integration.rs
use luna::os::*;

#[tokio::test]
async fn test_full_discovery_flow() {
    // Discover apps
    let apps = discovery::discover_applications().await.unwrap();
    
    // Filter by category
    let browsers: Vec<_> = apps.iter()
        .filter(|a| a.category == AppCategory::Browser)
        .collect();
    
    assert!(!browsers.is_empty(), "Should find at least one browser");
    
    // Test app matching
    if let Some(firefox) = apps.iter().find(|a| a.matches("firefox")) {
        println!("Found Firefox: {}", firefox.executable.display());
    }
}
```

## CLI Commands

```bash
# Discover and index applications
cargo run -- index applications

# Test volume control
cargo run -- system volume get
cargo run -- system volume set 50

# Launch application
cargo run -- open firefox

# System info
cargo run -- system info
```

## Troubleshooting

### Linux

**Issue:** App discovery returns empty list
- Check permissions for `/usr/share/applications`
- Verify desktop files exist: `ls /usr/share/applications/*.desktop`
- Check snap/flatpak installation

**Issue:** Volume control fails
- Verify PulseAudio: `pactl info`
- Or ALSA: `amixer`
- Install missing audio system

### Windows

**Issue:** Registry scanning fails
- Check permissions (may need admin)
- Verify registry path exists

**Issue:** Volume control not working
- Check PowerShell execution policy
- Consider using Windows Core Audio APIs

### macOS

**Issue:** App discovery incomplete
- Check permissions for `/Applications`
- Verify Info.plist files exist in bundles
- Grant Full Disk Access in System Preferences

**Issue:** AppleScript commands fail
- Check System Preferences → Security → Automation
- Grant permissions for osascript

## Best Practices

1. **Initialize once:** Create `OsInterface` at startup
2. **Cache discovery results:** App scanning is expensive
3. **Handle errors gracefully:** OS operations can fail
4. **Test on target platforms:** Each OS has unique behavior
5. **Use fallbacks:** Provide alternatives when features unavailable

## API Reference

### Core Types

- `OsInterface` - Unified cross-platform interface
- `Application` - Discovered application metadata
- `AppCategory` - Application category enum

### Key Functions

- `OsInterface::new()` - Create interface
- `OsInterface::init()` - Initialize and scan
- `OsInterface::launch_app()` - Launch application
- `OsInterface::get_volume()` - Get system volume
- `OsInterface::set_volume()` - Set system volume
- `discover_applications()` - Async app discovery

### Platform-Specific

**Linux:**
- `launch_app_linux()`, `get_volume_linux()`, `set_volume_linux()`
- `get_installed_packages()`, `get_snap_packages()`, `get_flatpak_packages()`

**Windows:**
- `launch_app_windows()`, `get_volume_windows()`, `set_volume_windows()`
- `get_running_processes()`

**macOS:**
- `launch_app_macos()`, `get_volume_macos()`, `set_volume_macos()`
- `get_running_apps()`, `spotlight_search()`

## Future Enhancements

- [ ] Process management (start, stop, monitor)
- [ ] Power management (shutdown, restart, sleep)
- [ ] D-Bus integration for Linux notifications
- [ ] Windows WMI for deeper system info
- [ ] macOS native Cocoa APIs
- [ ] Clipboard monitoring
- [ ] Global keyboard shortcuts
- [ ] Window management APIs

---

For more details, see `PHASE5_OS_INTEGRATION_COMPLETE.md`
