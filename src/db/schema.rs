//! Database schema definitions for LUNA
//!
//! Defines data structures for applications, files, and other entities.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents an installed application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    /// Unique identifier for the application
    pub id: String,
    
    /// Display name of the application
    pub name: String,
    
    /// Path to the executable file
    pub executable: PathBuf,
    
    /// Alternative names and common misspellings
    pub aliases: Vec<String>,
    
    /// Application category
    pub category: AppCategory,
    
    /// Path to application icon (optional)
    pub icon: Option<PathBuf>,
    
    /// Application description
    pub description: Option<String>,
    
    /// Version string
    pub version: Option<String>,
    
    /// Installation date (ISO 8601 format)
    pub install_date: Option<String>,
}

/// Categories for organizing applications
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppCategory {
    /// Web browsers (Chrome, Firefox, etc.)
    Browser,
    
    /// Integrated Development Environments
    IDE,
    
    /// Text editors (Notepad++, Sublime, etc.)
    TextEditor,
    
    /// Terminal emulators
    Terminal,
    
    /// Media players and editors
    Media,
    
    /// Communication apps (Slack, Discord, etc.)
    Communication,
    
    /// Office productivity (Word, Excel, etc.)
    Office,
    
    /// Games and entertainment
    Games,
    
    /// System utilities
    System,
    
    /// Uncategorized applications
    Other,
}

impl Application {
    /// Create a new application entry
    pub fn new(name: String, executable: PathBuf) -> Self {
        let id = format!("{}_{}", name.to_lowercase().replace(' ', "_"), 
                        executable.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown"));
        
        Self {
            id,
            name,
            executable,
            aliases: Vec::new(),
            category: AppCategory::Other,
            icon: None,
            description: None,
            version: None,
            install_date: None,
        }
    }
    
    /// Add an alias for the application
    pub fn add_alias(&mut self, alias: String) {
        if !self.aliases.contains(&alias) {
            self.aliases.push(alias);
        }
    }
    
    /// Check if application matches a query string
    pub fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        
        self.name.to_lowercase().contains(&query)
            || self.aliases.iter().any(|a| a.to_lowercase().contains(&query))
    }
}

/// Represents a file or directory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Full path to the file
    pub path: PathBuf,
    
    /// File name (without path)
    pub name: String,
    
    /// File extension (if any)
    pub extension: Option<String>,
    
    /// File size in bytes
    pub size: u64,
    
    /// Last modified timestamp (Unix epoch)
    pub modified: i64,
    
    /// Type of file
    pub file_type: FileType,
}

/// Categories for file types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    /// Documents (PDF, DOCX, TXT, etc.)
    Document,
    
    /// Source code files
    Code,
    
    /// Image files (PNG, JPG, etc.)
    Image,
    
    /// Video files
    Video,
    
    /// Audio files
    Audio,
    
    /// Compressed archives (ZIP, TAR, etc.)
    Archive,
    
    /// Directory/folder
    Directory,
    
    /// Other file types
    Other,
}

impl FileEntry {
    /// Create a new file entry from a path
    pub fn from_path(path: PathBuf) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(&path)?;
        
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());
        
        let file_type = if metadata.is_dir() {
            FileType::Directory
        } else {
            FileType::from_extension(extension.as_deref())
        };
        
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        Ok(Self {
            path,
            name,
            extension,
            size: metadata.len(),
            modified,
            file_type,
        })
    }
}

impl FileType {
    /// Determine file type from extension
    pub fn from_extension(ext: Option<&str>) -> Self {
        match ext {
            Some("txt") | Some("pdf") | Some("doc") | Some("docx") | Some("odt") => {
                FileType::Document
            }
            Some("rs") | Some("py") | Some("js") | Some("ts") | Some("java") | Some("c")
            | Some("cpp") | Some("h") | Some("go") => FileType::Code,
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("bmp")
            | Some("svg") => FileType::Image,
            Some("mp4") | Some("avi") | Some("mkv") | Some("mov") | Some("webm") => {
                FileType::Video
            }
            Some("mp3") | Some("wav") | Some("flac") | Some("ogg") | Some("m4a") => {
                FileType::Audio
            }
            Some("zip") | Some("tar") | Some("gz") | Some("rar") | Some("7z") => {
                FileType::Archive
            }
            _ => FileType::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_new() {
        let app = Application::new("Chrome".to_string(), PathBuf::from("/usr/bin/chrome"));
        assert_eq!(app.name, "Chrome");
        assert!(app.aliases.is_empty());
    }

    #[test]
    fn test_application_matches() {
        let mut app = Application::new("Chrome".to_string(), PathBuf::from("/usr/bin/chrome"));
        app.add_alias("google chrome".to_string());
        
        assert!(app.matches("chrome"));
        assert!(app.matches("google"));
        assert!(!app.matches("firefox"));
    }

    #[test]
    fn test_file_type_from_extension() {
        assert_eq!(FileType::from_extension(Some("rs")), FileType::Code);
        assert_eq!(FileType::from_extension(Some("png")), FileType::Image);
        assert_eq!(FileType::from_extension(Some("mp4")), FileType::Video);
        assert_eq!(FileType::from_extension(None), FileType::Other);
    }
}
