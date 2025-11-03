//! Application launcher
//!
//! Launch and close applications with fuzzy matching and error handling.

use crate::db::AppDatabase;
use crate::error::{LunaError, Result};
use std::process::Command;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Application launcher with database integration
pub struct AppLauncher {
    app_db: Arc<AppDatabase>,
}

impl AppLauncher {
    /// Create a new app launcher
    pub fn new(app_db: Arc<AppDatabase>) -> Self {
        Self { app_db }
    }
    
    /// Launch an application by name
    ///
    /// Uses fuzzy matching to find the best application match.
    pub async fn launch(&self, app_name: &str) -> Result<String> {
        info!("Launching app: {}", app_name);
        
        // Find the application
        let app = self.find_app(app_name)?;
        
        debug!("Found app: {} at {}", app.name, app.executable.display());
        
        // Launch the application
        self.launch_app_by_path(&app.executable.to_string_lossy(), &[])?;
        
        Ok(format!("Launched {}", app.name))
    }
    
    /// Launch an application with arguments
    pub async fn launch_with_args(&self, app_name: &str, args: &[String]) -> Result<String> {
        info!("Launching app: {} with args: {:?}", app_name, args);
        
        let app = self.find_app(app_name)?;
        self.launch_app_by_path(&app.executable.to_string_lossy(), args)?;
        
        Ok(format!("Launched {} with arguments", app.name))
    }
    
    /// Close a running application
    ///
    /// Platform-specific implementation to terminate processes
    pub async fn close(&self, app_name: &str) -> Result<String> {
        info!("Closing app: {}", app_name);
        
        let app = self.find_app(app_name)?;
        
        #[cfg(target_os = "linux")]
        {
            // Use pkill on Linux
            let status = Command::new("pkill")
                .arg("-f")
                .arg(&app.name)
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to close app: {}", e)))?;
            
            if status.success() {
                Ok(format!("Closed {}", app.name))
            } else {
                warn!("Application {} may not have been running", app.name);
                Ok(format!("Application {} was not running", app.name))
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // Use taskkill on Windows
            let status = Command::new("taskkill")
                .args(&["/IM", &format!("{}.exe", app.name), "/F"])
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to close app: {}", e)))?;
            
            if status.success() {
                Ok(format!("Closed {}", app.name))
            } else {
                warn!("Application {} may not have been running", app.name);
                Ok(format!("Application {} was not running", app.name))
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            // Use killall on macOS
            let status = Command::new("killall")
                .arg(&app.name)
                .status()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to close app: {}", e)))?;
            
            if status.success() {
                Ok(format!("Closed {}", app.name))
            } else {
                warn!("Application {} may not have been running", app.name);
                Ok(format!("Application {} was not running", app.name))
            }
        }
    }
    
    /// Find an application using fuzzy matching
    fn find_app(&self, app_name: &str) -> Result<crate::db::schema::Application> {
        let apps = self.app_db.search(app_name);
        
        if apps.is_empty() {
            return Err(LunaError::AppNotFound(app_name.to_string()));
        }
        
        // AppDatabase::search() already returns sorted by best match
        // So we just take the first one
        let best_match = apps[0];
        
        debug!("Best match: {} (score based on internal matching)", best_match.name);
        
        Ok(best_match.clone())
    }
    
    /// Launch an application by its full path
    fn launch_app_by_path(&self, path: &str, args: &[String]) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let mut cmd = Command::new(path);
            cmd.args(args);
            cmd.spawn()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to launch app: {}", e)))?;
        }
        
        #[cfg(target_os = "windows")]
        {
            let mut cmd = Command::new("cmd");
            cmd.args(&["/C", "start", "", path]);
            for arg in args {
                cmd.arg(arg);
            }
            cmd.spawn()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to launch app: {}", e)))?;
        }
        
        #[cfg(target_os = "macos")]
        {
            let mut cmd = Command::new("open");
            cmd.arg(path);
            if !args.is_empty() {
                cmd.arg("--args");
                cmd.args(args);
            }
            cmd.spawn()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to launch app: {}", e)))?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::Application;
    use std::path::PathBuf;
    
    fn create_test_db() -> Arc<AppDatabase> {
        let mut db = AppDatabase::new();
        
        // Add some test applications
        let mut chrome = Application::new(
            "Chrome".to_string(),
            PathBuf::from("/usr/bin/google-chrome")
        );
        chrome.add_alias("browser".to_string());
        chrome.add_alias("web".to_string());
        chrome.category = crate::db::schema::AppCategory::Browser;
        db.add_app(chrome);
        
        let mut firefox = Application::new(
            "Firefox".to_string(),
            PathBuf::from("/usr/bin/firefox")
        );
        firefox.add_alias("browser".to_string());
        firefox.add_alias("web".to_string());
        firefox.category = crate::db::schema::AppCategory::Browser;
        db.add_app(firefox);
        
        Arc::new(db)
    }
    
    #[tokio::test]
    async fn test_find_app() {
        let db = create_test_db();
        let launcher = AppLauncher::new(db);
        
        // Exact match
        let app = launcher.find_app("Chrome").unwrap();
        assert_eq!(app.name, "Chrome");
        
        // Fuzzy match
        let app = launcher.find_app("chrom").unwrap();
        assert_eq!(app.name, "Chrome");
    }
    
    #[tokio::test]
    async fn test_app_not_found() {
        let db = create_test_db();
        let launcher = AppLauncher::new(db);
        
        let result = launcher.find_app("NonexistentApp");
        assert!(result.is_err());
    }
}
