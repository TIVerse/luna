//! Context-aware disambiguation for brain system
//!
//! Tracks recent commands and entities to resolve references like
//! "open it", "that file", "the same app", etc.

use crate::brain::command_parser::{IntentType, ParsedCommand};
use crate::brain::types::Entity;
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, VecDeque};

/// Maximum number of commands to keep in history
const MAX_HISTORY_SIZE: usize = 10;

/// Maximum age of context entries (5 minutes)
const MAX_CONTEXT_AGE: i64 = 300;

/// Conversation context for disambiguation
#[derive(Debug, Clone)]
pub struct ConversationContext {
    /// Recent command history
    history: VecDeque<ContextEntry>,

    /// Named entity references (e.g., "the file" -> last file entity)
    entity_cache: HashMap<String, Entity>,

    /// Success/failure statistics for learning
    stats: HashMap<String, CommandStats>,
}

/// A single context entry
#[derive(Debug, Clone)]
pub struct ContextEntry {
    /// When this command was processed
    pub timestamp: DateTime<Utc>,

    /// The parsed command
    pub command: ParsedCommand,

    /// Extracted entities
    pub entities: HashMap<String, Entity>,

    /// Was the command successful?
    pub success: bool,

    /// Normalized command text
    pub normalized_text: String,
}

/// Statistics for a specific command pattern
#[derive(Debug, Clone, Default)]
pub struct CommandStats {
    /// Number of times executed
    pub count: usize,

    /// Number of successes
    pub successes: usize,

    /// Last execution time
    pub last_used: Option<DateTime<Utc>>,
}

impl CommandStats {
    /// Get success rate (0.0 - 1.0)
    pub fn success_rate(&self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        self.successes as f32 / self.count as f32
    }
}

impl ConversationContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            entity_cache: HashMap::new(),
            stats: HashMap::new(),
        }
    }

    /// Add a command to history
    pub fn add_command(
        &mut self,
        command: ParsedCommand,
        entities: HashMap<String, Entity>,
        success: bool,
    ) {
        // Create entry
        let entry = ContextEntry {
            timestamp: Utc::now(),
            command: command.clone(),
            entities: entities.clone(),
            success,
            normalized_text: Self::normalize_text(&command.original_text),
        };

        // Add to history (FIFO)
        if self.history.len() >= MAX_HISTORY_SIZE {
            self.history.pop_front();
        }
        self.history.push_back(entry);

        // Update entity cache
        self.update_entity_cache(&entities);

        // Update stats
        self.update_stats(&command.original_text, success);

        // Clean old entries
        self.clean_old_entries();
    }

    /// Resolve references in text (e.g., "it", "that", "the file")
    pub fn resolve_references(&self, text: &str) -> Option<HashMap<String, Entity>> {
        let normalized = text.to_lowercase();

        // Detect reference words
        let reference_patterns = [
            ("it", vec!["app", "file", "folder"]),
            ("that", vec!["app", "file", "folder"]),
            ("this", vec!["app", "file", "folder"]),
            ("the same", vec!["app", "file", "folder"]),
            ("same app", vec!["app"]),
            ("same file", vec!["file"]),
            ("that file", vec!["file"]),
            ("that app", vec!["app"]),
            ("that folder", vec!["folder"]),
        ];

        for (pattern, entity_types) in &reference_patterns {
            if normalized.contains(pattern) {
                // Look for most recent entity of matching type
                for entry in self.history.iter().rev() {
                    for entity_type in entity_types {
                        if let Some(entity) = entry.entities.get(*entity_type) {
                            let mut resolved = HashMap::new();
                            resolved.insert(entity_type.to_string(), entity.clone());
                            return Some(resolved);
                        }
                    }
                }
            }
        }

        None
    }

    /// Get the most recent entity of a given type
    pub fn get_recent_entity(&self, entity_type: &str) -> Option<&Entity> {
        for entry in self.history.iter().rev() {
            if let Some(entity) = entry.entities.get(entity_type) {
                return Some(entity);
            }
        }
        None
    }

    /// Get command statistics
    pub fn get_stats(&self, text: &str) -> Option<&CommandStats> {
        let normalized = Self::normalize_text(text);
        self.stats.get(&normalized)
    }

    /// Get success rate for a command pattern
    pub fn success_rate(&self, text: &str) -> f32 {
        self.get_stats(text)
            .map(|s| s.success_rate())
            .unwrap_or(0.0)
    }

    /// Check if command was recently successful
    pub fn was_recently_successful(&self, text: &str) -> bool {
        if let Some(stats) = self.get_stats(text) {
            if let Some(last_used) = stats.last_used {
                let age = Utc::now() - last_used;
                return age.num_seconds() < 60 && stats.success_rate() > 0.7;
            }
        }
        false
    }

    /// Get similar commands from history
    pub fn find_similar_commands(&self, text: &str, threshold: f32) -> Vec<String> {
        use crate::utils::string_matching::similarity_score;

        let normalized = Self::normalize_text(text);
        let mut similar = Vec::new();

        for entry in &self.history {
            let score = similarity_score(&normalized, &entry.normalized_text);
            if score >= threshold {
                similar.push(entry.command.original_text.clone());
            }
        }

        similar
    }

    /// Clear all context
    pub fn clear(&mut self) {
        self.history.clear();
        self.entity_cache.clear();
        self.stats.clear();
    }

    /// Get history size
    pub fn history_size(&self) -> usize {
        self.history.len()
    }

    // Private helpers

    fn normalize_text(text: &str) -> String {
        text.to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn update_entity_cache(&mut self, entities: &HashMap<String, Entity>) {
        for (key, entity) in entities {
            self.entity_cache.insert(key.clone(), entity.clone());
        }
    }

    fn update_stats(&mut self, text: &str, success: bool) {
        let normalized = Self::normalize_text(text);
        let stats = self
            .stats
            .entry(normalized)
            .or_insert_with(CommandStats::default);

        stats.count += 1;
        if success {
            stats.successes += 1;
        }
        stats.last_used = Some(Utc::now());
    }

    fn clean_old_entries(&mut self) {
        let now = Utc::now();
        let cutoff = now - Duration::seconds(MAX_CONTEXT_AGE);

        // Remove old history entries
        while let Some(entry) = self.history.front() {
            if entry.timestamp < cutoff {
                self.history.pop_front();
            } else {
                break;
            }
        }
    }
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_command(text: &str, intent: IntentType) -> ParsedCommand {
        ParsedCommand {
            intent,
            entities: HashMap::new(),
            original_text: text.to_string(),
            confidence: 0.9,
        }
    }

    #[test]
    fn test_context_creation() {
        let ctx = ConversationContext::new();
        assert_eq!(ctx.history_size(), 0);
    }

    #[test]
    fn test_add_command() {
        let mut ctx = ConversationContext::new();

        let cmd = create_test_command("open chrome", IntentType::LaunchApp);
        let mut entities = HashMap::new();
        entities.insert("app".to_string(), Entity::App("chrome".to_string()));

        ctx.add_command(cmd, entities, true);

        assert_eq!(ctx.history_size(), 1);
    }

    #[test]
    fn test_resolve_references() {
        let mut ctx = ConversationContext::new();

        // Add a command with an app entity
        let cmd = create_test_command("open chrome", IntentType::LaunchApp);
        let mut entities = HashMap::new();
        entities.insert("app".to_string(), Entity::App("chrome".to_string()));

        ctx.add_command(cmd, entities, true);

        // Try to resolve "close it"
        let resolved = ctx.resolve_references("close it");
        assert!(resolved.is_some());

        let entities = resolved.unwrap();
        assert!(entities.contains_key("app"));
    }

    #[test]
    fn test_get_recent_entity() {
        let mut ctx = ConversationContext::new();

        let cmd = create_test_command("open file.pdf", IntentType::FindFile);
        let mut entities = HashMap::new();
        entities.insert("file".to_string(), Entity::File("file.pdf".to_string()));

        ctx.add_command(cmd, entities, true);

        let recent = ctx.get_recent_entity("file");
        assert!(recent.is_some());

        match recent.unwrap() {
            Entity::File(name) => assert_eq!(name, "file.pdf"),
            _ => panic!("Wrong entity type"),
        }
    }

    #[test]
    fn test_command_stats() {
        let mut ctx = ConversationContext::new();

        let cmd1 = create_test_command("open chrome", IntentType::LaunchApp);
        ctx.add_command(cmd1.clone(), HashMap::new(), true);
        ctx.add_command(cmd1.clone(), HashMap::new(), true);
        ctx.add_command(cmd1, HashMap::new(), false);

        let stats = ctx.get_stats("open chrome").unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.successes, 2);
        assert!((stats.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_was_recently_successful() {
        let mut ctx = ConversationContext::new();

        let cmd = create_test_command("volume up", IntentType::VolumeControl);
        ctx.add_command(cmd.clone(), HashMap::new(), true);
        ctx.add_command(cmd.clone(), HashMap::new(), true);

        assert!(ctx.was_recently_successful("volume up"));
    }

    #[test]
    fn test_history_limit() {
        let mut ctx = ConversationContext::new();

        // Add more than MAX_HISTORY_SIZE commands
        for i in 0..15 {
            let cmd = create_test_command(&format!("command {}", i), IntentType::Unknown);
            ctx.add_command(cmd, HashMap::new(), true);
        }

        // Should not exceed limit
        assert_eq!(ctx.history_size(), MAX_HISTORY_SIZE);
    }

    #[test]
    fn test_clear() {
        let mut ctx = ConversationContext::new();

        let cmd = create_test_command("test", IntentType::Unknown);
        ctx.add_command(cmd, HashMap::new(), true);

        assert_eq!(ctx.history_size(), 1);

        ctx.clear();
        assert_eq!(ctx.history_size(), 0);
    }
}
