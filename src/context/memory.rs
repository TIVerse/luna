//! Conversation memory
//!
//! Stores and retrieves conversation history for contextual understanding.

use crate::brain::IntentType;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Entry in the conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    /// Timestamp of the command
    pub timestamp: i64,
    /// Original user input
    pub user_input: String,
    /// Parsed intent from the command
    pub parsed_intent: IntentType,
    /// Action that was taken
    pub action_taken: String,
    /// Whether the action succeeded
    pub success: bool,
}

impl ConversationEntry {
    /// Create a new conversation entry
    pub fn new(
        user_input: String,
        parsed_intent: IntentType,
        action_taken: String,
        success: bool,
    ) -> Self {
        Self {
            timestamp: Utc::now().timestamp(),
            user_input,
            parsed_intent,
            action_taken,
            success,
        }
    }
}

/// Conversation memory manager
pub struct ConversationMemory {
    /// History of conversation entries
    history: VecDeque<ConversationEntry>,
    /// Maximum number of entries to keep
    max_size: usize,
}

impl ConversationMemory {
    /// Create a new conversation memory with default size (100 entries)
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    /// Create a new conversation memory with specified capacity
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Add an entry to the conversation history
    pub fn add_entry(&mut self, entry: ConversationEntry) {
        // If at capacity, remove oldest entry
        if self.history.len() >= self.max_size {
            self.history.pop_front();
        }

        self.history.push_back(entry);
    }

    /// Get the full conversation context (all entries)
    pub fn get_context(&self) -> Vec<&ConversationEntry> {
        self.history.iter().collect()
    }

    /// Get the last N entries
    pub fn get_recent(&self, n: usize) -> Vec<&ConversationEntry> {
        self.history.iter().rev().take(n).collect()
    }

    /// Get the last action performed
    pub fn get_last_action(&self) -> Option<&ConversationEntry> {
        self.history.back()
    }

    /// Get the last successful action
    pub fn get_last_successful_action(&self) -> Option<&ConversationEntry> {
        self.history.iter().rev().find(|e| e.success)
    }

    /// Get all entries matching a specific intent
    pub fn get_by_intent(&self, intent: &IntentType) -> Vec<&ConversationEntry> {
        self.history
            .iter()
            .filter(|e| &e.parsed_intent == intent)
            .collect()
    }

    /// Get success rate for a specific intent
    pub fn success_rate_for_intent(&self, intent: &IntentType) -> f32 {
        let matching: Vec<_> = self.get_by_intent(intent);
        if matching.is_empty() {
            return 0.0;
        }

        let successful = matching.iter().filter(|e| e.success).count();
        successful as f32 / matching.len() as f32
    }

    /// Clear all conversation history
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// Get the number of entries in history
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Find entries related to a specific keyword (in user input or action)
    pub fn search(&self, keyword: &str) -> Vec<&ConversationEntry> {
        let keyword_lower = keyword.to_lowercase();
        self.history
            .iter()
            .filter(|e| {
                e.user_input.to_lowercase().contains(&keyword_lower)
                    || e.action_taken.to_lowercase().contains(&keyword_lower)
            })
            .collect()
    }

    /// Save conversation history to disk
    pub async fn save_to_disk<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> crate::error::Result<()> {
        let entries: Vec<ConversationEntry> = self.history.iter().cloned().collect();
        let json = serde_json::to_string_pretty(&entries)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    /// Load conversation history from disk
    pub async fn load_from_disk<P: AsRef<std::path::Path>>(
        path: P,
    ) -> crate::error::Result<Self> {
        let path = path.as_ref();
        
        // If file doesn't exist, return empty memory
        if !path.exists() {
            return Ok(Self::new());
        }

        let json = tokio::fs::read_to_string(path).await?;
        let entries: Vec<ConversationEntry> = serde_json::from_str(&json)?;
        
        let mut memory = Self::new();
        for entry in entries {
            memory.add_entry(entry);
        }
        
        Ok(memory)
    }
}

impl Default for ConversationMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_entry() {
        let mut memory = ConversationMemory::new();
        let entry = ConversationEntry::new(
            "open chrome".to_string(),
            IntentType::LaunchApp,
            "Launched Chrome".to_string(),
            true,
        );

        memory.add_entry(entry);
        assert_eq!(memory.len(), 1);
    }

    #[test]
    fn test_get_last_action() {
        let mut memory = ConversationMemory::new();

        memory.add_entry(ConversationEntry::new(
            "open chrome".to_string(),
            IntentType::LaunchApp,
            "Launched Chrome".to_string(),
            true,
        ));

        memory.add_entry(ConversationEntry::new(
            "close chrome".to_string(),
            IntentType::CloseApp,
            "Closed Chrome".to_string(),
            true,
        ));

        let last = memory.get_last_action().unwrap();
        assert_eq!(last.user_input, "close chrome");
    }

    #[test]
    fn test_max_size() {
        let mut memory = ConversationMemory::with_capacity(2);

        for i in 0..3 {
            memory.add_entry(ConversationEntry::new(
                format!("command {}", i),
                IntentType::LaunchApp,
                format!("action {}", i),
                true,
            ));
        }

        assert_eq!(memory.len(), 2);
        // First entry should be removed
        assert!(memory.get_context()[0].user_input == "command 1");
    }

    #[test]
    fn test_search() {
        let mut memory = ConversationMemory::new();

        memory.add_entry(ConversationEntry::new(
            "open chrome".to_string(),
            IntentType::LaunchApp,
            "Launched Chrome".to_string(),
            true,
        ));

        memory.add_entry(ConversationEntry::new(
            "open firefox".to_string(),
            IntentType::LaunchApp,
            "Launched Firefox".to_string(),
            true,
        ));

        let results = memory.search("chrome");
        assert_eq!(results.len(), 1);
        assert!(results[0].user_input.contains("chrome"));
    }
}
