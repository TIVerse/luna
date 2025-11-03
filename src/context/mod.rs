//! Context and memory module
//!
//! Manages conversation history and application state for contextual command understanding.

pub mod memory;
pub mod state;
pub mod unified;

use crate::error::Result;
use memory::ConversationMemory;
use tracing::info;

// Re-export key types
pub use memory::ConversationEntry;
pub use state::{ReferenceType, StateManager};
pub use unified::{ActionResult, CommandStats, ContextEntry, UnifiedContext};

/// Context manager that coordinates memory and state tracking
pub struct ContextManager {
    /// Conversation memory
    memory: ConversationMemory,
    /// Application state tracker
    state: StateManager,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new() -> Result<Self> {
        info!("Initializing context manager...");

        Ok(Self {
            memory: ConversationMemory::new(),
            state: StateManager::new(),
        })
    }

    /// Create with custom capacities
    pub fn with_capacity(memory_size: usize, state_recent_size: usize) -> Result<Self> {
        Ok(Self {
            memory: ConversationMemory::with_capacity(memory_size),
            state: StateManager::with_capacity(state_recent_size),
        })
    }

    /// Get reference to conversation memory
    pub fn memory(&self) -> &ConversationMemory {
        &self.memory
    }

    /// Get mutable reference to conversation memory
    pub fn memory_mut(&mut self) -> &mut ConversationMemory {
        &mut self.memory
    }

    /// Get reference to state manager
    pub fn state(&self) -> &StateManager {
        &self.state
    }

    /// Get mutable reference to state manager
    pub fn state_mut(&mut self) -> &mut StateManager {
        &mut self.state
    }

    /// Add a conversation entry
    pub fn add_entry(&mut self, entry: memory::ConversationEntry) {
        self.memory.add_entry(entry);
    }

    /// Resolve contextual references in text
    /// E.g., "close it" -> "close chrome" if Chrome was the last opened app
    pub fn resolve_text(&self, text: &str) -> String {
        let text_lower = text.to_lowercase();

        // Check for referential phrases
        if text_lower.contains("it") || text_lower.contains("that") {
            // Try to determine what kind of reference based on context
            if let Some(app) = self.state.get_active_app() {
                return text
                    .replace("it", app)
                    .replace("that", app)
                    .replace("that app", app);
            }

            if let Some(file) = self.state.get_last_opened_file() {
                let file_str = file.to_string_lossy();
                return text
                    .replace("it", &file_str)
                    .replace("that file", &file_str);
            }
        }

        // Check for "there" references (usually for search)
        if text_lower.contains("there") {
            if let Some(query) = self.state.get_last_query() {
                return text.replace("there", query);
            }
        }

        text.to_string()
    }

    /// Clear all context (memory and state)
    pub fn clear(&mut self) {
        self.memory.clear();
        self.state.clear();
    }

    /// Get a summary of current context
    pub fn summary(&self) -> String {
        format!(
            "Memory: {} entries, State: {}",
            self.memory.len(),
            self.state.summary()
        )
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default ContextManager")
    }
}

#[cfg(test)]
mod tests {
    use super::memory;
    use super::*;
    use crate::brain::IntentType;

    #[test]
    fn test_context_manager_creation() {
        let manager = ContextManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_add_entry() {
        let mut manager = ContextManager::new().unwrap();

        let entry = memory::ConversationEntry::new(
            "open chrome".to_string(),
            IntentType::LaunchApp,
            "Launched Chrome".to_string(),
            true,
        );

        manager.add_entry(entry);
        assert_eq!(manager.memory().len(), 1);
    }

    #[test]
    fn test_resolve_text() {
        let mut manager = ContextManager::new().unwrap();

        // Set active app
        manager.state_mut().set_active_app("Chrome".to_string());

        // Test resolution
        let resolved = manager.resolve_text("close it");
        assert!(resolved.contains("Chrome"));
    }

    #[test]
    fn test_clear() {
        let mut manager = ContextManager::new().unwrap();

        manager.state_mut().set_active_app("Firefox".to_string());
        manager.add_entry(memory::ConversationEntry::new(
            "test".to_string(),
            IntentType::LaunchApp,
            "test".to_string(),
            true,
        ));

        assert!(!manager.memory().is_empty());

        manager.clear();

        assert!(manager.memory().is_empty());
        assert!(manager.state().get_active_app().is_none());
    }
}
