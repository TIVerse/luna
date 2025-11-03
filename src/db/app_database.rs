//! Application database management
//!
//! Stores and queries installed applications with fuzzy matching.

use crate::db::schema::{AppCategory, Application};
use crate::error::{LunaError, Result};
use crate::utils::string_matching;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Database for managing installed applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppDatabase {
    /// List of all applications
    apps: Vec<Application>,

    /// Index mapping normalized names to app indices
    #[serde(skip)]
    index: HashMap<String, Vec<usize>>,
}

impl AppDatabase {
    /// Create a new empty database
    pub fn new() -> Self {
        Self {
            apps: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// Load database from disk
    pub async fn load_from_disk<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = tokio::fs::read_to_string(path.as_ref())
            .await
            .map_err(|e| LunaError::Database(format!("Failed to load database: {}", e)))?;

        let mut db: AppDatabase = serde_json::from_str(&contents)?;
        db.rebuild_index();
        Ok(db)
    }

    /// Save database to disk
    pub async fn save_to_disk<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let contents = serde_json::to_string_pretty(self)?;

        // Create parent directory if needed
        if let Some(parent) = path.as_ref().parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| LunaError::Database(format!("Failed to create directory: {}", e)))?;
        }

        tokio::fs::write(path.as_ref(), contents)
            .await
            .map_err(|e| LunaError::Database(format!("Failed to save database: {}", e)))?;

        Ok(())
    }

    /// Add an application to the database
    pub fn add_app(&mut self, app: Application) {
        let idx = self.apps.len();

        // Add to index
        let normalized = string_matching::normalize(&app.name);
        self.index.entry(normalized).or_default().push(idx);

        // Index aliases
        for alias in &app.aliases {
            let normalized = string_matching::normalize(alias);
            self.index.entry(normalized).or_default().push(idx);
        }

        self.apps.push(app);
    }

    /// Find application by exact name
    pub fn find_by_name(&self, name: &str) -> Option<&Application> {
        let normalized = string_matching::normalize(name);
        self.index
            .get(&normalized)
            .and_then(|indices| indices.first())
            .and_then(|&idx| self.apps.get(idx))
    }

    /// Find applications by category
    pub fn find_by_category(&self, category: AppCategory) -> Vec<&Application> {
        self.apps
            .iter()
            .filter(|app| app.category == category)
            .collect()
    }

    /// Search for applications with fuzzy matching
    pub fn search(&self, query: &str) -> Vec<&Application> {
        let mut results: Vec<(&Application, f32)> = self
            .apps
            .iter()
            .filter_map(|app| {
                let score = self.calculate_match_score(app, query);
                if score > 0.5 {
                    Some((app, score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        results.into_iter().map(|(app, _)| app).collect()
    }

    /// Calculate match score for an application
    fn calculate_match_score(&self, app: &Application, query: &str) -> f32 {
        let query = string_matching::normalize(query);

        // Exact match gets highest score
        if string_matching::normalize(&app.name) == query {
            return 1.0;
        }

        // Check aliases for exact match
        for alias in &app.aliases {
            if string_matching::normalize(alias) == query {
                return 0.95;
            }
        }

        // Fuzzy match on name
        let name_score = string_matching::similarity_score(&app.name, &query);

        // Fuzzy match on best alias
        let alias_score = app
            .aliases
            .iter()
            .map(|alias| string_matching::similarity_score(alias, &query))
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        name_score.max(alias_score)
    }

    /// Rebuild the search index
    fn rebuild_index(&mut self) {
        self.index.clear();

        for (idx, app) in self.apps.iter().enumerate() {
            let normalized = string_matching::normalize(&app.name);
            self.index.entry(normalized).or_default().push(idx);

            for alias in &app.aliases {
                let normalized = string_matching::normalize(alias);
                self.index.entry(normalized).or_default().push(idx);
            }
        }
    }

    /// Get all applications
    pub fn all(&self) -> &[Application] {
        &self.apps
    }

    /// Get number of applications in database
    pub fn len(&self) -> usize {
        self.apps.len()
    }

    /// Check if database is empty
    pub fn is_empty(&self) -> bool {
        self.apps.is_empty()
    }
}

impl Default for AppDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_add_and_find() {
        let mut db = AppDatabase::new();
        let mut app = Application::new("Chrome".to_string(), PathBuf::from("/usr/bin/chrome"));
        app.add_alias("google chrome".to_string());

        db.add_app(app);

        assert_eq!(db.len(), 1);
        assert!(db.find_by_name("chrome").is_some());
    }

    #[test]
    fn test_fuzzy_search() {
        let mut db = AppDatabase::new();
        let mut app = Application::new(
            "Visual Studio Code".to_string(),
            PathBuf::from("/usr/bin/code"),
        );
        app.add_alias("vscode".to_string());
        app.add_alias("code".to_string());
        db.add_app(app);

        // Search by exact alias match
        let results = db.search("vscode");
        assert!(!results.is_empty());

        // Search by partial name match
        let results = db.search("visual studio");
        assert!(!results.is_empty());

        // Search by another alias
        let results = db.search("code");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_by_category() {
        let mut db = AppDatabase::new();

        let mut app1 = Application::new("Chrome".to_string(), PathBuf::from("/usr/bin/chrome"));
        app1.category = AppCategory::Browser;

        let mut app2 = Application::new("Firefox".to_string(), PathBuf::from("/usr/bin/firefox"));
        app2.category = AppCategory::Browser;

        db.add_app(app1);
        db.add_app(app2);

        let browsers = db.find_by_category(AppCategory::Browser);
        assert_eq!(browsers.len(), 2);
    }
}
