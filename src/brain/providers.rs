//! Knowledge providers for brain system
//!
//! Trait-based dependency injection for app databases, file indices, and other
//! knowledge sources. Decouples classification logic from data sources.

use phf::phf_map;
use std::sync::Arc;

/// Trait for providing application knowledge
pub trait KnownAppProvider: Send + Sync {
    /// Check if an application is known
    fn is_known(&self, name: &str) -> bool;

    /// Get aliases for an application
    fn aliases(&self, name: &str) -> Vec<String>;

    /// Get all known applications
    fn all_apps(&self) -> Vec<String>;

    /// Add a new application dynamically
    fn add_app(&mut self, name: String, aliases: Vec<String>);
}

/// Default in-memory app provider with common applications
pub struct DefaultAppProvider {
    custom_apps: Vec<(String, Vec<String>)>,
}

// Static perfect hash map for common apps - O(1) lookup
static COMMON_APPS: phf::Map<&'static str, &'static [&'static str]> = phf_map! {
    "chrome" => &["google chrome", "chrome browser"],
    "firefox" => &["mozilla firefox", "firefox browser"],
    "safari" => &["safari browser"],
    "edge" => &["microsoft edge", "edge browser"],
    "vscode" => &["visual studio code", "code", "vs code"],
    "sublime" => &["sublime text", "subl"],
    "atom" => &["atom editor"],
    "terminal" => &["term", "console", "shell"],
    "iterm" => &["iterm2"],
    "slack" => &["slack app"],
    "discord" => &["discord app"],
    "teams" => &["microsoft teams"],
    "zoom" => &["zoom app"],
    "spotify" => &["spotify music", "spotify app"],
    "vlc" => &["vlc player", "vlc media player"],
    "itunes" => &["apple music"],
    "photoshop" => &["adobe photoshop", "ps"],
    "illustrator" => &["adobe illustrator", "ai"],
    "word" => &["microsoft word", "ms word"],
    "excel" => &["microsoft excel", "ms excel"],
    "powerpoint" => &["microsoft powerpoint", "ms powerpoint", "ppt"],
    "outlook" => &["microsoft outlook", "ms outlook"],
    "thunderbird" => &["mozilla thunderbird"],
    "notion" => &["notion app"],
    "obsidian" => &["obsidian app"],
};

impl DefaultAppProvider {
    /// Create a new default app provider
    pub fn new() -> Self {
        Self {
            custom_apps: Vec::new(),
        }
    }
}

impl Default for DefaultAppProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl KnownAppProvider for DefaultAppProvider {
    fn is_known(&self, name: &str) -> bool {
        let normalized = name.to_lowercase();

        // Check static map
        if COMMON_APPS.contains_key(&normalized.as_str()) {
            return true;
        }

        // Check static map aliases
        for (_, aliases) in COMMON_APPS.entries() {
            if aliases.iter().any(|a| a.to_lowercase() == normalized) {
                return true;
            }
        }

        // Check custom apps
        for (app, aliases) in &self.custom_apps {
            if app.to_lowercase() == normalized {
                return true;
            }
            if aliases.iter().any(|a| a.to_lowercase() == normalized) {
                return true;
            }
        }

        false
    }

    fn aliases(&self, name: &str) -> Vec<String> {
        let normalized = name.to_lowercase();
        let mut result = Vec::new();

        // Check static map
        if let Some(aliases) = COMMON_APPS.get(&normalized.as_str()) {
            result.extend(aliases.iter().map(|s| s.to_string()));
        }

        // Check custom apps
        for (app, aliases) in &self.custom_apps {
            if app.to_lowercase() == normalized {
                result.extend(aliases.clone());
            }
        }

        result
    }

    fn all_apps(&self) -> Vec<String> {
        let mut apps: Vec<String> = COMMON_APPS.keys().map(|k| k.to_string()).collect();

        apps.extend(self.custom_apps.iter().map(|(name, _)| name.clone()));
        apps
    }

    fn add_app(&mut self, name: String, aliases: Vec<String>) {
        self.custom_apps.push((name, aliases));
    }
}

/// Trait for providing file/folder knowledge
pub trait FileSystemProvider: Send + Sync {
    /// Check if a path looks like a file
    fn is_file_like(&self, name: &str) -> bool;

    /// Check if a path looks like a folder
    fn is_folder_like(&self, name: &str) -> bool;

    /// Get common folder names
    fn common_folders(&self) -> Vec<String>;
}

/// Default filesystem provider with heuristics
pub struct DefaultFileSystemProvider;

impl DefaultFileSystemProvider {
    /// Create a new default filesystem provider
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultFileSystemProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemProvider for DefaultFileSystemProvider {
    fn is_file_like(&self, name: &str) -> bool {
        // Has extension
        if name.contains('.') {
            let parts: Vec<&str> = name.split('.').collect();
            if parts.len() >= 2 {
                let ext = parts.last().unwrap();
                return ext.len() <= 5 && ext.chars().all(|c| c.is_alphanumeric());
            }
        }
        false
    }

    fn is_folder_like(&self, name: &str) -> bool {
        let normalized = name.to_lowercase();

        // Common folder keywords
        let folder_keywords = [
            "folder",
            "directory",
            "dir",
            "downloads",
            "documents",
            "desktop",
            "pictures",
            "videos",
            "music",
            "projects",
        ];

        folder_keywords.iter().any(|k| normalized.contains(k))
    }

    fn common_folders(&self) -> Vec<String> {
        vec![
            "downloads".to_string(),
            "documents".to_string(),
            "desktop".to_string(),
            "pictures".to_string(),
            "videos".to_string(),
            "music".to_string(),
            "projects".to_string(),
        ]
    }
}

/// Composite provider that combines multiple knowledge sources
pub struct CompositeProvider {
    app_provider: Arc<dyn KnownAppProvider>,
    fs_provider: Arc<dyn FileSystemProvider>,
}

impl CompositeProvider {
    /// Create a new composite provider
    pub fn new(
        app_provider: Arc<dyn KnownAppProvider>,
        fs_provider: Arc<dyn FileSystemProvider>,
    ) -> Self {
        Self {
            app_provider,
            fs_provider,
        }
    }

    /// Get the app provider
    pub fn app_provider(&self) -> &Arc<dyn KnownAppProvider> {
        &self.app_provider
    }

    /// Get the filesystem provider
    pub fn fs_provider(&self) -> &Arc<dyn FileSystemProvider> {
        &self.fs_provider
    }
}

impl Default for CompositeProvider {
    fn default() -> Self {
        Self::new(
            Arc::new(DefaultAppProvider::new()),
            Arc::new(DefaultFileSystemProvider::new()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_app_provider() {
        let provider = DefaultAppProvider::new();

        assert!(provider.is_known("chrome"));
        assert!(provider.is_known("Chrome"));
        assert!(provider.is_known("google chrome"));
        assert!(provider.is_known("vscode"));
        assert!(provider.is_known("visual studio code"));
        assert!(!provider.is_known("unknownapp123"));
    }

    #[test]
    fn test_app_aliases() {
        let provider = DefaultAppProvider::new();

        let aliases = provider.aliases("chrome");
        assert!(!aliases.is_empty());
        assert!(aliases.iter().any(|a| a.contains("google")));
    }

    #[test]
    fn test_add_custom_app() {
        let mut provider = DefaultAppProvider::new();

        provider.add_app("myapp".to_string(), vec!["my custom app".to_string()]);

        assert!(provider.is_known("myapp"));
        assert!(provider.is_known("my custom app"));
    }

    #[test]
    fn test_filesystem_provider() {
        let provider = DefaultFileSystemProvider::new();

        assert!(provider.is_file_like("document.pdf"));
        assert!(provider.is_file_like("report.docx"));
        assert!(!provider.is_file_like("chrome"));

        assert!(provider.is_folder_like("downloads folder"));
        assert!(provider.is_folder_like("my documents"));
    }

    #[test]
    fn test_composite_provider() {
        let provider = CompositeProvider::default();

        assert!(provider.app_provider().is_known("chrome"));
        assert!(provider.fs_provider().is_file_like("test.pdf"));
    }
}
