//! Actions module for executing commands (GOD-LEVEL ENHANCED)
//!
//! Launches apps, searches files, controls system, etc.

pub mod app_launcher;
pub mod capabilities;
pub mod clipboard;
pub mod executor;
pub mod file_operations;
pub mod file_search;
pub mod media_control;
pub mod reminders;
pub mod system_control;
pub mod window_control;

// Re-export main structures
pub use app_launcher::AppLauncher;
pub use capabilities::{Capability, CapabilityDetector};
pub use clipboard::Clipboard;
pub use executor::{TaskExecutor, ExecutionPolicy, RetryPolicy};
pub use file_search::FileSearch;
pub use media_control::MediaControl;
pub use system_control::SystemControl;
pub use window_control::WindowControl;
