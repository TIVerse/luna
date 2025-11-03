//! Operating system integration module (PHASE 5 GOD-LEVEL)
//!
//! Platform-specific code for Windows, Linux, and macOS.
//! Includes 18 god-level enhancements for enterprise-grade system management.

// Core OS modules
pub mod common;
pub mod discovery;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

// GOD-LEVEL ENHANCEMENTS (Phase 5)

// P0: Critical Features
pub mod process_manager;      // Smart Process Lifecycle Management
pub mod multi_monitor;         // Multi-Monitor Intelligence
pub mod resource_monitor;      // Real-Time Resource Monitor & Optimizer

// P1: High-Impact Features
pub mod window_manager;        // Window Manipulation Suite
pub mod smart_app_index;       // ML-Powered Application Indexing
pub mod automation;            // Desktop Automation Engine
pub mod health_monitor;        // System Health Monitor with Auto-Remediation

// P2: Important Features
pub mod power_manager;         // Smart Power Management
pub mod virtual_desktop;       // Virtual Desktop Management
pub mod process_priority;      // Process Priority & Affinity Management
pub mod clipboard_manager;     // Advanced Clipboard Manager

// P3: Supporting Features
pub mod desktop_environment;   // Desktop Environment Detection
pub mod app_watcher;           // Real-Time App Change Detection
pub mod performance_optimizer; // System Performance Optimizer

// Unified Manager - Single Entry Point
pub mod unified_manager;       // Unified OS Manager (Integration Layer)

// Re-exports for convenience
pub use unified_manager::{UnifiedOsManager, SystemHealth};

use crate::error::Result;
use std::path::Path;

/// OS abstraction layer - provides unified cross-platform interface
pub struct OsInterface {
    #[cfg(target_os = "linux")]
    platform: linux::LinuxInterface,
    
    #[cfg(target_os = "windows")]
    platform: windows::WindowsInterface,
    
    #[cfg(target_os = "macos")]
    platform: macos::MacOsInterface,
}

impl OsInterface {
    /// Create a new OS interface
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "linux")]
        {
            Ok(Self {
                platform: linux::LinuxInterface::new()?,
            })
        }
        
        #[cfg(target_os = "windows")]
        {
            Ok(Self {
                platform: windows::WindowsInterface::new()?,
            })
        }
        
        #[cfg(target_os = "macos")]
        {
            Ok(Self {
                platform: macos::MacOsInterface::new()?,
            })
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err(crate::error::LunaError::SystemOperation("Unsupported platform".to_string()))
        }
    }
    
    /// Initialize the interface (scan for applications, etc.)
    pub fn init(&mut self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            self.platform.init()
        }
        
        #[cfg(target_os = "windows")]
        {
            self.platform.init()
        }
        
        #[cfg(target_os = "macos")]
        {
            self.platform.init()
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Ok(())
        }
    }
    
    /// Launch an application
    pub fn launch_app(&self, path: &Path) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            linux::launch_app_linux(path)
        }
        
        #[cfg(target_os = "windows")]
        {
            windows::launch_app_windows(path)
        }
        
        #[cfg(target_os = "macos")]
        {
            macos::launch_app_macos(path)
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err(crate::error::LunaError::SystemOperation("Unsupported platform".to_string()))
        }
    }
    
    /// Get system volume (0-100)
    pub fn get_volume(&self) -> Result<u8> {
        #[cfg(target_os = "linux")]
        {
            linux::get_volume_linux()
        }
        
        #[cfg(target_os = "windows")]
        {
            windows::get_volume_windows()
        }
        
        #[cfg(target_os = "macos")]
        {
            macos::get_volume_macos()
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Ok(50) // Fallback
        }
    }
    
    /// Set system volume (0-100)
    pub fn set_volume(&self, level: u8) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            linux::set_volume_linux(level)
        }
        
        #[cfg(target_os = "windows")]
        {
            windows::set_volume_windows(level)
        }
        
        #[cfg(target_os = "macos")]
        {
            macos::set_volume_macos(level)
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Err(crate::error::LunaError::SystemOperation("Unsupported platform".to_string()))
        }
    }
}

impl Default for OsInterface {
    fn default() -> Self {
        Self::new().expect("Failed to initialize OS interface")
    }
}
