//! File indexing and search functionality
//!
//! Maintains an index of files for fast searching.

use crate::db::schema::{FileEntry, FileType};
use crate::error::{LunaError, Result};
use crate::utils::string_matching;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// File index for fast file searching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIndex {
    /// List of indexed files
    files: Vec<FileEntry>,
    
    /// Last update timestamp
    last_updated: i64,
}

impl FileIndex {
    /// Create a new empty file index
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            last_updated: 0,
        }
    }

    /// Load index from disk
    pub async fn load_from_disk<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = tokio::fs::read_to_string(path.as_ref()).await.map_err(|e| {
            LunaError::Database(format!("Failed to load file index: {}", e))
        })?;

        let index: FileIndex = serde_json::from_str(&contents)?;
        Ok(index)
    }

    /// Save index to disk
    pub async fn save_to_disk<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let contents = serde_json::to_string_pretty(self)?;
        
        if let Some(parent) = path.as_ref().parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                LunaError::Database(format!("Failed to create directory: {}", e))
            })?;
        }
        
        tokio::fs::write(path.as_ref(), contents).await.map_err(|e| {
            LunaError::Database(format!("Failed to save file index: {}", e))
        })?;

        Ok(())
    }

    /// Add a file to the index
    pub fn add_file(&mut self, file: FileEntry) {
        self.files.push(file);
    }

    /// Search for files by name
    pub fn search_by_name(&self, query: &str) -> Vec<&FileEntry> {
        let mut results: Vec<(&FileEntry, f32)> = self
            .files
            .iter()
            .filter_map(|file| {
                let score = string_matching::similarity_score(&file.name, query);
                if score > 0.5 {
                    Some((file, score))
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter().map(|(file, _)| file).collect()
    }

    /// Search for files by type
    pub fn search_by_type(&self, file_type: FileType) -> Vec<&FileEntry> {
        self.files
            .iter()
            .filter(|file| file.file_type == file_type)
            .collect()
    }

    /// Get all indexed files
    pub fn all(&self) -> &[FileEntry] {
        &self.files
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.files.clear();
    }

    /// Get number of indexed files
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl Default for FileIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_index_new() {
        let index = FileIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_add_and_search() {
        let mut index = FileIndex::new();
        
        let file = FileEntry {
            path: PathBuf::from("/home/user/document.txt"),
            name: "document.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 1024,
            modified: 0,
            file_type: FileType::Document,
        };
        
        index.add_file(file);
        
        let results = index.search_by_name("document");
        assert_eq!(results.len(), 1);
    }
}
