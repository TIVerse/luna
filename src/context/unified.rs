//! Unified context system
//!
//! Bridges ConversationMemory and brain::ConversationContext with persistence.

use crate::brain::command_parser::{IntentType, ParsedCommand};
use crate::brain::types::Entity;
use crate::context::memory::ConversationEntry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, error, info, warn};

/// Unified context that combines both memory systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedContext {
    /// Short-term conversation history
    pub history: VecDeque<ContextEntry>,
    /// Named entity cache
    pub entities: HashMap<String, Entity>,
    /// Command statistics
    pub stats: HashMap<String, CommandStats>,
    /// Maximum history size
    pub max_size: usize,
    /// Persistence path
    #[serde(skip)]
    pub persist_path: Option<PathBuf>,
}

/// Enhanced context entry with typed entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    pub timestamp: DateTime<Utc>,
    pub user_input: String,
    pub parsed_intent: IntentType,
    pub entities: HashMap<String, Entity>,
    pub action_taken: ActionResult,
    pub success: bool,
    pub confidence: f32,
}

/// Typed action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    LaunchedApp(String),
    ClosedApp(String),
    OpenedFile(PathBuf),
    OpenedFolder(PathBuf),
    SearchedWeb(String),
    VolumeChanged(i32),
    SystemControl(String),
    MediaControl(String),
    Reminder(String),
    Note(String),
    Answer(String),
    Failed(String),
    Unknown,
}

/// Command execution statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandStats {
    pub count: usize,
    pub successes: usize,
    pub last_used: Option<DateTime<Utc>>,
    pub avg_confidence: f32,
}

impl CommandStats {
    pub fn success_rate(&self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        self.successes as f32 / self.count as f32
    }

    pub fn update(&mut self, success: bool, confidence: f32) {
        self.count += 1;
        if success {
            self.successes += 1;
        }
        self.last_used = Some(Utc::now());

        // Moving average for confidence
        if self.count == 1 {
            self.avg_confidence = confidence;
        } else {
            self.avg_confidence =
                (self.avg_confidence * (self.count - 1) as f32 + confidence) / self.count as f32;
        }
    }
}

impl UnifiedContext {
    /// Create new unified context
    pub fn new(max_size: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_size),
            entities: HashMap::new(),
            stats: HashMap::new(),
            max_size,
            persist_path: None,
        }
    }

    /// Create with persistence enabled
    pub fn with_persistence(max_size: usize, path: PathBuf) -> Self {
        let mut ctx = Self::new(max_size);
        ctx.persist_path = Some(path);
        ctx
    }

    /// Add a context entry
    pub fn add_entry(&mut self, entry: ContextEntry) {
        // Update entity cache
        for (key, entity) in &entry.entities {
            self.entities.insert(key.clone(), entity.clone());
        }

        // Update stats
        let normalized = Self::normalize_text(&entry.user_input);
        let stats = self
            .stats
            .entry(normalized)
            .or_insert_with(CommandStats::default);
        stats.update(entry.success, entry.confidence);

        // Add to history
        if self.history.len() >= self.max_size {
            self.history.pop_front();
        }
        self.history.push_back(entry);
    }

    /// Convert from legacy ConversationEntry
    pub fn add_legacy_entry(&mut self, entry: ConversationEntry) {
        let ctx_entry = ContextEntry {
            timestamp: DateTime::from_timestamp(entry.timestamp, 0).unwrap_or_else(Utc::now),
            user_input: entry.user_input,
            parsed_intent: entry.parsed_intent,
            entities: HashMap::new(), // Legacy entries don't have typed entities
            action_taken: ActionResult::Unknown,
            success: entry.success,
            confidence: 0.5,
        };

        self.add_entry(ctx_entry);
    }

    /// Get recent entries
    pub fn get_recent(&self, n: usize) -> Vec<&ContextEntry> {
        self.history.iter().rev().take(n).collect()
    }

    /// Get last entry
    pub fn get_last(&self) -> Option<&ContextEntry> {
        self.history.back()
    }

    /// Get last successful entry
    pub fn get_last_successful(&self) -> Option<&ContextEntry> {
        self.history.iter().rev().find(|e| e.success)
    }

    /// Resolve entity reference (e.g., "it", "that file")
    pub fn resolve_reference(&self, reference_type: &str) -> Option<&Entity> {
        // Look in recent history for matching entity type
        for entry in self.history.iter().rev().take(5) {
            if let Some(entity) = entry.entities.get(reference_type) {
                return Some(entity);
            }
        }

        // Check entity cache
        self.entities.get(reference_type)
    }

    /// Get command statistics
    pub fn get_stats(&self, text: &str) -> Option<&CommandStats> {
        let normalized = Self::normalize_text(text);
        self.stats.get(&normalized)
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

    /// Search history by keyword
    pub fn search(&self, keyword: &str) -> Vec<&ContextEntry> {
        let keyword_lower = keyword.to_lowercase();
        self.history
            .iter()
            .filter(|e| e.user_input.to_lowercase().contains(&keyword_lower))
            .collect()
    }

    /// Clear all context
    pub fn clear(&mut self) {
        self.history.clear();
        self.entities.clear();
        // Keep stats for learning
    }

    /// Get context summary
    pub fn summary(&self) -> String {
        let total = self.history.len();
        let successful = self.history.iter().filter(|e| e.success).count();
        let rate = if total > 0 {
            (successful as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        format!(
            "Context: {} entries, {:.1}% success rate, {} entity types cached",
            total,
            rate,
            self.entities.len()
        )
    }

    /// Save context to disk
    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = &self.persist_path {
            debug!("Saving context to {:?}", path);

            // Create parent directory if needed
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }

            let json = serde_json::to_string_pretty(self)?;
            fs::write(path, json).await?;

            info!("Context saved: {} entries", self.history.len());
        }
        Ok(())
    }

    /// Load context from disk
    pub async fn load(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Loading context from {:?}", path);

        if !path.exists() {
            warn!("Context file not found, starting fresh");
            return Ok(Self::with_persistence(500, path));
        }

        let json = fs::read_to_string(&path).await?;
        let mut ctx: Self = serde_json::from_str(&json)?;
        ctx.persist_path = Some(path);

        info!("Context loaded: {} entries", ctx.history.len());
        Ok(ctx)
    }

    /// Summarize old entries and trim history
    pub fn summarize(&mut self) -> Option<String> {
        if self.history.len() < self.max_size * 2 {
            return None;
        }

        // Get oldest half
        let to_summarize: Vec<_> = self.history.iter().take(self.max_size).collect();

        // Generate summary
        let mut intent_counts: HashMap<IntentType, usize> = HashMap::new();
        let mut success_count = 0;

        for entry in &to_summarize {
            *intent_counts
                .entry(entry.parsed_intent.clone())
                .or_insert(0) += 1;
            if entry.success {
                success_count += 1;
            }
        }

        let summary = format!(
            "Summarized {} entries: {:?}, {:.1}% success",
            to_summarize.len(),
            intent_counts,
            (success_count as f32 / to_summarize.len() as f32) * 100.0
        );

        // Keep only recent half
        self.history = self.history.iter().skip(self.max_size).cloned().collect();

        info!("{}", summary);
        Some(summary)
    }

    fn normalize_text(text: &str) -> String {
        text.to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Default for UnifiedContext {
    fn default() -> Self {
        Self::new(500)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_context() {
        let mut ctx = UnifiedContext::new(10);

        let entry = ContextEntry {
            timestamp: Utc::now(),
            user_input: "open chrome".to_string(),
            parsed_intent: IntentType::LaunchApp,
            entities: {
                let mut map = HashMap::new();
                map.insert("app".to_string(), Entity::App("chrome".to_string()));
                map
            },
            action_taken: ActionResult::LaunchedApp("chrome".to_string()),
            success: true,
            confidence: 0.95,
        };

        ctx.add_entry(entry);

        assert_eq!(ctx.history.len(), 1);
        assert_eq!(ctx.entities.len(), 1);
    }

    #[test]
    fn test_resolve_reference() {
        let mut ctx = UnifiedContext::new(10);

        let entry = ContextEntry {
            timestamp: Utc::now(),
            user_input: "open file.txt".to_string(),
            parsed_intent: IntentType::FindFile,
            entities: {
                let mut map = HashMap::new();
                map.insert("file".to_string(), Entity::File("file.txt".to_string()));
                map
            },
            action_taken: ActionResult::OpenedFile(PathBuf::from("file.txt")),
            success: true,
            confidence: 0.9,
        };

        ctx.add_entry(entry);

        let resolved = ctx.resolve_reference("file");
        assert!(resolved.is_some());
    }

    #[test]
    fn test_stats_tracking() {
        let mut ctx = UnifiedContext::new(10);

        for i in 0..5 {
            let entry = ContextEntry {
                timestamp: Utc::now(),
                user_input: "volume up".to_string(),
                parsed_intent: IntentType::VolumeControl,
                entities: HashMap::new(),
                action_taken: ActionResult::VolumeChanged(10),
                success: i < 4, // 4 successes, 1 failure
                confidence: 0.85,
            };
            ctx.add_entry(entry);
        }

        let stats = ctx.get_stats("volume up").unwrap();
        assert_eq!(stats.count, 5);
        assert_eq!(stats.successes, 4);
        assert_eq!(stats.success_rate(), 0.8);
    }
}
