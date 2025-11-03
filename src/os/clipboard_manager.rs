//! Advanced Clipboard Manager (GOD-LEVEL)

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: Uuid,
    pub content: ClipboardContent,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source_app: Option<String>,
    pub format: ClipboardFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardContent {
    Text(String),
    Image(Vec<u8>),
    Files(Vec<std::path::PathBuf>),
    Html(String),
    Rtf(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardFormat {
    PlainText,
    RichText,
    Html,
    Image,
    Files,
}

pub struct ClipboardManager {
    history: VecDeque<ClipboardEntry>,
    max_history: usize,
    favorites: Vec<ClipboardEntry>,
}

impl ClipboardManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
            favorites: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, content: ClipboardContent, format: ClipboardFormat) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }

        let entry = ClipboardEntry {
            id: Uuid::new_v4(),
            content,
            timestamp: chrono::Utc::now(),
            source_app: None,
            format,
        };

        self.history.push_back(entry);
    }

    pub fn get_history(&self) -> Vec<&ClipboardEntry> {
        self.history.iter().collect()
    }

    pub fn search(&self, query: &str) -> Vec<&ClipboardEntry> {
        self.history
            .iter()
            .filter(|entry| {
                if let ClipboardContent::Text(text) = &entry.content {
                    text.contains(query)
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn add_favorite(&mut self, id: Uuid) {
        if let Some(entry) = self.history.iter().find(|e| e.id == id) {
            self.favorites.push(entry.clone());
        }
    }

    pub fn get_favorites(&self) -> &[ClipboardEntry] {
        &self.favorites
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new(100)
    }
}
