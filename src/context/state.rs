//! Application state tracking
//!
//! Tracks active applications, windows, and file operations for contextual commands.

use std::path::PathBuf;

/// State manager for tracking current application state
#[derive(Debug, Clone, Default)]
pub struct StateManager {
    /// Currently active application
    pub active_app: Option<String>,
    /// List of currently open window titles
    pub open_windows: Vec<String>,
    /// Last opened file path
    pub last_opened_file: Option<PathBuf>,
    /// Last searched query
    pub last_query: Option<String>,
    /// Recently accessed files (for quick reference)
    recent_files: Vec<PathBuf>,
    /// Recently used applications
    recent_apps: Vec<String>,
    /// Maximum number of recent items to track
    max_recent: usize,
}

impl StateManager {
    /// Create a new state manager with default capacity
    pub fn new() -> Self {
        Self {
            active_app: None,
            open_windows: Vec::new(),
            last_opened_file: None,
            last_query: None,
            recent_files: Vec::new(),
            recent_apps: Vec::new(),
            max_recent: 10,
        }
    }

    /// Create a state manager with specified capacity for recent items
    pub fn with_capacity(max_recent: usize) -> Self {
        Self {
            active_app: None,
            open_windows: Vec::new(),
            last_opened_file: None,
            last_query: None,
            recent_files: Vec::with_capacity(max_recent),
            recent_apps: Vec::with_capacity(max_recent),
            max_recent,
        }
    }

    /// Set the currently active application
    pub fn set_active_app(&mut self, app_name: String) {
        self.active_app = Some(app_name.clone());
        self.add_recent_app(app_name);
    }

    /// Clear the active application
    pub fn clear_active_app(&mut self) {
        self.active_app = None;
    }

    /// Get the currently active application
    pub fn get_active_app(&self) -> Option<&str> {
        self.active_app.as_deref()
    }

    /// Add a window to the list of open windows
    pub fn add_window(&mut self, window_title: String) {
        if !self.open_windows.contains(&window_title) {
            self.open_windows.push(window_title);
        }
    }

    /// Remove a window from the list
    pub fn remove_window(&mut self, window_title: &str) {
        self.open_windows.retain(|w| w != window_title);
    }

    /// Get all open windows
    pub fn get_open_windows(&self) -> &[String] {
        &self.open_windows
    }

    /// Check if a window with title is open
    pub fn has_window(&self, window_title: &str) -> bool {
        self.open_windows.iter().any(|w| w.contains(window_title))
    }

    /// Set the last opened file
    pub fn set_last_opened_file(&mut self, file_path: PathBuf) {
        self.last_opened_file = Some(file_path.clone());
        self.add_recent_file(file_path);
    }

    /// Get the last opened file
    pub fn get_last_opened_file(&self) -> Option<&PathBuf> {
        self.last_opened_file.as_ref()
    }

    /// Set the last search query
    pub fn set_last_query(&mut self, query: String) {
        self.last_query = Some(query);
    }

    /// Get the last search query
    pub fn get_last_query(&self) -> Option<&str> {
        self.last_query.as_deref()
    }

    /// Add a file to recent files list
    fn add_recent_file(&mut self, file_path: PathBuf) {
        // Remove if already exists
        self.recent_files.retain(|f| f != &file_path);

        // Add to front
        self.recent_files.insert(0, file_path);

        // Trim to max size
        if self.recent_files.len() > self.max_recent {
            self.recent_files.truncate(self.max_recent);
        }
    }

    /// Get recent files
    pub fn get_recent_files(&self) -> &[PathBuf] {
        &self.recent_files
    }

    /// Add an app to recent apps list
    fn add_recent_app(&mut self, app_name: String) {
        // Remove if already exists
        self.recent_apps.retain(|a| a != &app_name);

        // Add to front
        self.recent_apps.insert(0, app_name);

        // Trim to max size
        if self.recent_apps.len() > self.max_recent {
            self.recent_apps.truncate(self.max_recent);
        }
    }

    /// Get recent applications
    pub fn get_recent_apps(&self) -> &[String] {
        &self.recent_apps
    }

    /// Resolve contextual reference (e.g., "it", "that file")
    pub fn resolve_reference(&self, reference_type: ReferenceType) -> Option<String> {
        match reference_type {
            ReferenceType::App => self.active_app.clone(),
            ReferenceType::File => self
                .last_opened_file
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            ReferenceType::Query => self.last_query.clone(),
            ReferenceType::Window => self.open_windows.last().cloned(),
        }
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.active_app = None;
        self.open_windows.clear();
        self.last_opened_file = None;
        self.last_query = None;
        self.recent_files.clear();
        self.recent_apps.clear();
    }

    /// Get a summary of current state
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(app) = &self.active_app {
            parts.push(format!("Active app: {}", app));
        }

        if !self.open_windows.is_empty() {
            parts.push(format!("{} open windows", self.open_windows.len()));
        }

        if let Some(file) = &self.last_opened_file {
            parts.push(format!("Last file: {}", file.display()));
        }

        if parts.is_empty() {
            "No active state".to_string()
        } else {
            parts.join(", ")
        }
    }
}

/// Type of contextual reference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceType {
    /// Reference to an application ("it", "that app")
    App,
    /// Reference to a file ("it", "that file")
    File,
    /// Reference to a search query ("that", "the search")
    Query,
    /// Reference to a window ("it", "that window")
    Window,
}

// Backward compatibility alias
pub type StateTracker = StateManager;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_app() {
        let mut state = StateManager::new();
        assert!(state.get_active_app().is_none());

        state.set_active_app("Chrome".to_string());
        assert_eq!(state.get_active_app(), Some("Chrome"));

        state.clear_active_app();
        assert!(state.get_active_app().is_none());
    }

    #[test]
    fn test_windows() {
        let mut state = StateManager::new();

        state.add_window("Window 1".to_string());
        state.add_window("Window 2".to_string());

        assert_eq!(state.get_open_windows().len(), 2);
        assert!(state.has_window("Window 1"));

        state.remove_window("Window 1");
        assert_eq!(state.get_open_windows().len(), 1);
        assert!(!state.has_window("Window 1"));
    }

    #[test]
    fn test_recent_files() {
        let mut state = StateManager::with_capacity(2);

        state.set_last_opened_file(PathBuf::from("/tmp/file1.txt"));
        state.set_last_opened_file(PathBuf::from("/tmp/file2.txt"));
        state.set_last_opened_file(PathBuf::from("/tmp/file3.txt"));

        // Should only keep 2 most recent
        assert_eq!(state.get_recent_files().len(), 2);
        assert_eq!(state.get_recent_files()[0], PathBuf::from("/tmp/file3.txt"));
    }

    #[test]
    fn test_resolve_reference() {
        let mut state = StateManager::new();
        state.set_active_app("Firefox".to_string());
        state.set_last_query("rust programming".to_string());

        assert_eq!(
            state.resolve_reference(ReferenceType::App),
            Some("Firefox".to_string())
        );
        assert_eq!(
            state.resolve_reference(ReferenceType::Query),
            Some("rust programming".to_string())
        );
    }

    #[test]
    fn test_summary() {
        let mut state = StateManager::new();
        assert_eq!(state.summary(), "No active state");

        state.set_active_app("VS Code".to_string());
        state.add_window("Window 1".to_string());

        let summary = state.summary();
        assert!(summary.contains("VS Code"));
        assert!(summary.contains("1 open windows"));
    }
}
