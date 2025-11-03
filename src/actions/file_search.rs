//! File search functionality
//!
//! Search for files across the system with fuzzy matching and content search.

use crate::db::FileIndex;
use crate::error::{LunaError, Result};
use crate::utils::string_matching::similarity_score;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// File search with index integration
pub struct FileSearch {
    index: Arc<FileIndex>,
}

impl FileSearch {
    /// Create a new file search
    pub fn new(index: Arc<FileIndex>) -> Self {
        Self { index }
    }
    
    /// Search for files by name
    ///
    /// Uses fuzzy matching to find files with similar names.
    pub async fn search_by_name(&self, query: &str, limit: usize) -> Result<Vec<PathBuf>> {
        info!("Searching for files: {}", query);
        
        let all_files = self.index.all();
        
        if all_files.is_empty() {
            warn!("File index is empty. Consider running an index update.");
            return Ok(Vec::new());
        }
        
        // Score all files by name similarity
        let mut scored_files: Vec<(PathBuf, f32)> = all_files
            .iter()
            .filter_map(|file_info| {
                let file_name = file_info.path
                    .file_name()?
                    .to_str()?;
                
                let score = similarity_score(query, file_name);
                
                // Only include files with reasonable similarity
                if score > 0.3 {
                    Some((file_info.path.clone(), score))
                } else {
                    None
                }
            })
            .collect();
        
        // Sort by score (descending)
        scored_files.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Take top results
        let results: Vec<PathBuf> = scored_files
            .into_iter()
            .take(limit)
            .map(|(path, _)| path)
            .collect();
        
        debug!("Found {} matching files", results.len());
        
        Ok(results)
    }
    
    /// Search for files by content (simple grep-like search)
    pub async fn search_by_content(&self, query: &str, limit: usize) -> Result<Vec<PathBuf>> {
        info!("Searching file contents for: {}", query);
        
        let all_files = self.index.all();
        let mut results = Vec::new();
        
        for file_info in all_files {
            // Only search text files
            if !Self::is_text_file(&file_info.path) {
                continue;
            }
            
            // Read file content
            if let Ok(content) = fs::read_to_string(&file_info.path) {
                if content.to_lowercase().contains(&query.to_lowercase()) {
                    results.push(file_info.path.clone());
                    
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }
        
        debug!("Found {} files containing query", results.len());
        
        Ok(results)
    }
    
    /// Find recently modified files
    pub async fn find_recent(&self, limit: usize) -> Result<Vec<PathBuf>> {
        info!("Finding {} most recent files", limit);
        
        let mut all_files: Vec<_> = self.index.all().iter().cloned().collect();
        
        // Sort by modification time (most recent first)
        all_files.sort_by(|a, b| {
            b.modified
                .cmp(&a.modified)
        });
        
        let results: Vec<PathBuf> = all_files
            .into_iter()
            .take(limit)
            .map(|f| f.path)
            .collect();
        
        debug!("Found {} recent files", results.len());
        
        Ok(results)
    }
    
    /// Find files by extension
    pub async fn find_by_extension(&self, extension: &str, limit: usize) -> Result<Vec<PathBuf>> {
        info!("Finding files with extension: {}", extension);
        
        let all_files = self.index.all();
        
        let results: Vec<PathBuf> = all_files
            .iter()
            .filter(|f| {
                f.path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case(extension))
                    .unwrap_or(false)
            })
            .take(limit)
            .map(|f| f.path.clone())
            .collect();
        
        debug!("Found {} files with extension .{}", results.len(), extension);
        
        Ok(results)
    }
    
    /// Check if a file is likely a text file based on extension
    fn is_text_file(path: &PathBuf) -> bool {
        let text_extensions = [
            "txt", "md", "rs", "py", "js", "ts", "jsx", "tsx",
            "html", "css", "json", "toml", "yaml", "yml", "xml",
            "c", "cpp", "h", "hpp", "java", "go", "sh", "bash",
        ];
        
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| text_extensions.contains(&e))
            .unwrap_or(false)
    }
    
    /// Open a file with the default application
    pub async fn open_file(&self, path: &PathBuf) -> Result<String> {
        info!("Opening file: {}", path.display());
        
        if !path.exists() {
            return Err(LunaError::FileNotFound(path.display().to_string()));
        }
        
        open::that(path)
            .map_err(|e| LunaError::SystemOperation(format!("Failed to open file: {}", e)))?;
        
        Ok(format!("Opened {}", path.file_name().unwrap().to_string_lossy()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::{FileEntry, FileType};
    
    fn create_test_index() -> Arc<FileIndex> {
        let mut index = FileIndex::new();
        
        // Add some test files
        index.add_file(FileEntry {
            path: PathBuf::from("/test/document.txt"),
            name: "document.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 1024,
            modified: 1234567890,
            file_type: FileType::Document,
        });
        
        index.add_file(FileEntry {
            path: PathBuf::from("/test/image.png"),
            name: "image.png".to_string(),
            extension: Some("png".to_string()),
            size: 2048,
            modified: 1234567891,
            file_type: FileType::Image,
        });
        
        index.add_file(FileEntry {
            path: PathBuf::from("/test/notes.md"),
            name: "notes.md".to_string(),
            extension: Some("md".to_string()),
            size: 512,
            modified: 1234567892,
            file_type: FileType::Document,
        });
        
        Arc::new(index)
    }
    
    #[tokio::test]
    async fn test_search_by_name() {
        let index = create_test_index();
        let search = FileSearch::new(index);
        
        let results = search.search_by_name("document", 10).await.unwrap();
        assert!(!results.is_empty());
        assert!(results[0].to_str().unwrap().contains("document"));
    }
    
    #[tokio::test]
    async fn test_find_by_extension() {
        let index = create_test_index();
        let search = FileSearch::new(index);
        
        let results = search.find_by_extension("txt", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].to_str().unwrap().ends_with(".txt"));
    }
    
    #[tokio::test]
    async fn test_find_recent() {
        let index = create_test_index();
        let search = FileSearch::new(index);
        
        let results = search.find_recent(5).await.unwrap();
        assert_eq!(results.len(), 3); // We added 3 files
    }
    
    #[test]
    fn test_is_text_file() {
        assert!(FileSearch::is_text_file(&PathBuf::from("test.txt")));
        assert!(FileSearch::is_text_file(&PathBuf::from("test.rs")));
        assert!(!FileSearch::is_text_file(&PathBuf::from("test.png")));
        assert!(!FileSearch::is_text_file(&PathBuf::from("test.exe")));
    }
}

