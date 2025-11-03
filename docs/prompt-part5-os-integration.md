# LUNA Project - Part 5: OS Integration

## Overview
Implement OS-specific functionality for Windows, Linux, and macOS.

## Dependencies
```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = ["Win32_System_Threading", "Win32_UI_WindowsAndMessaging"] }
winapi = { version = "0.3", features = ["winuser", "shellapi"] }

[target.'cfg(target_os = "linux")'.dependencies]
nix = { version = "0.27", features = ["process"] }
dbus = "0.9"

[target.'cfg(target_os = "macos")'.dependencies]
core-foundation = "0.9"
cocoa = "0.25"
```

## Modules

### 1. Application Discovery (`src/os/discovery.rs`)
Scan system for installed applications.

**Windows:**
- Start Menu: `C:\ProgramData\Microsoft\Windows\Start Menu`
- Registry: `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall`
- Program Files

**Linux:**
- Desktop files: `/usr/share/applications/*.desktop`
- Package managers: `dpkg -l`, `rpm -qa`
- Snap/Flatpak

**macOS:**
- Applications folder: `/Applications`
- LaunchServices database

### 2. OS-Specific Implementations

#### Windows (`src/os/windows.rs`)
```rust
pub fn launch_app_windows(path: &Path) -> Result<()>;
pub fn get_volume_windows() -> Result<u8>;
pub fn set_volume_windows(level: u8) -> Result<()>;
```

#### Linux (`src/os/linux.rs`)
```rust
pub fn launch_app_linux(path: &Path) -> Result<()>;
pub fn get_volume_linux() -> Result<u8>;
pub fn set_volume_linux(level: u8) -> Result<()>;
```

#### macOS (`src/os/macos.rs`)
```rust
pub fn launch_app_macos(path: &Path) -> Result<()>;
pub fn get_volume_macos() -> Result<u8>;
pub fn set_volume_macos(level: u8) -> Result<()>;
```

## Success Criteria
- ✅ App discovery works on all platforms
- ✅ Platform-specific features utilized
- ✅ Fallback for unsupported features
