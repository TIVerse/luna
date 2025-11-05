//! Enhanced context resolution for multi-turn conversations
//!
//! Handles pronoun resolution, entity linking, and conversation context tracking.

use crate::brain::IntentType;
use crate::context::ConversationEntry;
use crate::error::Result;
use std::collections::HashMap;
use tracing::debug;

/// Context reference type
#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceType {
    /// Pronoun reference (it, that, this, them)
    Pronoun(String),
    /// Definite reference (the file, the app)
    Definite(String),
    /// Previous action result
    PreviousResult,
    /// Ordinal reference (first one, last one, second item)
    Ordinal(usize),
}

/// Resolved context entity
#[derive(Debug, Clone)]
pub struct ResolvedEntity {
    /// Original reference text
    pub reference: String,
    /// Resolved value
    pub resolved_value: String,
    /// Entity type
    pub entity_type: String,
    /// Confidence score
    pub confidence: f32,
}

/// Enhanced context resolver
pub struct ContextResolver {
    /// Pronouns that refer to things
    object_pronouns: Vec<&'static str>,
    /// Definite articles
    definite_articles: Vec<&'static str>,
    /// Ordinal patterns
    ordinal_patterns: Vec<(&'static str, usize)>,
}

impl ContextResolver {
    /// Create a new context resolver
    pub fn new() -> Self {
        Self {
            object_pronouns: vec!["it", "that", "this", "them", "those", "these"],
            definite_articles: vec!["the", "that", "this"],
            ordinal_patterns: vec![
                ("first", 0),
                ("second", 1),
                ("third", 2),
                ("last", usize::MAX),
                ("latest", usize::MAX),
            ],
        }
    }

    /// Detect if command contains references to previous context
    pub fn contains_reference(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        
        // Check for pronouns
        if self.object_pronouns.iter().any(|p| {
            lower.contains(&format!(" {} ", p)) || lower.starts_with(&format!("{} ", p))
        }) {
            return true;
        }

        // Check for ordinals
        if self.ordinal_patterns.iter().any(|(pat, _)| lower.contains(pat)) {
            return true;
        }

        // Check for "the X" where X was mentioned before
        if lower.contains(" the ") {
            return true;
        }

        false
    }

    /// Extract reference type from command
    pub fn extract_references(&self, text: &str) -> Vec<ReferenceType> {
        let mut references = Vec::new();
        let lower = text.to_lowercase();

        // Extract pronouns
        for pronoun in &self.object_pronouns {
            if lower.contains(pronoun) {
                references.push(ReferenceType::Pronoun(pronoun.to_string()));
            }
        }

        // Extract ordinals
        for (pattern, index) in &self.ordinal_patterns {
            if lower.contains(pattern) {
                references.push(ReferenceType::Ordinal(*index));
            }
        }

        // Extract definite references
        if let Some(after_the) = self.extract_after_definite(&lower) {
            references.push(ReferenceType::Definite(after_the));
        }

        references
    }

    /// Resolve references using conversation history
    pub fn resolve(
        &self,
        text: &str,
        conversation_history: &[ConversationEntry],
        last_result: Option<&str>,
    ) -> Result<HashMap<String, String>> {
        let mut resolved = HashMap::new();
        let references = self.extract_references(text);

        debug!("Found {} references in: {}", references.len(), text);

        for reference in references {
            match reference {
                ReferenceType::Pronoun(pronoun) => {
                    if let Some(entity) = self.resolve_pronoun(&pronoun, conversation_history) {
                        resolved.insert(pronoun, entity);
                    }
                }
                ReferenceType::Definite(noun) => {
                    if let Some(entity) = self.resolve_definite(&noun, conversation_history) {
                        resolved.insert(format!("the {}", noun), entity);
                    }
                }
                ReferenceType::PreviousResult => {
                    if let Some(result) = last_result {
                        resolved.insert("result".to_string(), result.to_string());
                    }
                }
                ReferenceType::Ordinal(index) => {
                    if let Some(entity) = self.resolve_ordinal(index, conversation_history) {
                        resolved.insert(format!("item_{}", index), entity);
                    }
                }
            }
        }

        Ok(resolved)
    }

    /// Resolve pronoun to entity from history
    fn resolve_pronoun(&self, _pronoun: &str, history: &[ConversationEntry]) -> Option<String> {
        // Look for most recent entity in last 3 conversations
        for entry in history.iter().rev().take(3) {
            // Extract entities from action taken
            if let Some(entity) = self.extract_main_entity(&entry.action_taken) {
                return Some(entity);
            }
        }
        None
    }

    /// Resolve definite reference (the file, the app)
    fn resolve_definite(&self, noun: &str, history: &[ConversationEntry]) -> Option<String> {
        // Look for mentions of this noun type in recent history
        for entry in history.iter().rev().take(5) {
            let lower = entry.action_taken.to_lowercase();
            if lower.contains(noun) {
                if let Some(entity) = self.extract_entity_near(noun, &entry.action_taken) {
                    return Some(entity);
                }
            }
        }
        None
    }

    /// Resolve ordinal reference (first, last, second)
    fn resolve_ordinal(&self, index: usize, history: &[ConversationEntry]) -> Option<String> {
        // Look for list results in recent history
        for entry in history.iter().rev().take(3) {
            if let Some(items) = self.extract_list(&entry.action_taken) {
                return if index == usize::MAX {
                    // Last item
                    items.last().cloned()
                } else if index < items.len() {
                    Some(items[index].clone())
                } else {
                    None
                };
            }
        }
        None
    }

    /// Extract text after "the", "this", "that"
    fn extract_after_definite(&self, text: &str) -> Option<String> {
        for article in &self.definite_articles {
            if let Some(pos) = text.find(&format!("{} ", article)) {
                let after = &text[pos + article.len() + 1..];
                if let Some(end) = after.find(|c: char| !c.is_alphanumeric() && c != '_') {
                    return Some(after[..end].to_string());
                } else {
                    return Some(after.to_string());
                }
            }
        }
        None
    }

    /// Extract main entity from action text
    fn extract_main_entity(&self, action: &str) -> Option<String> {
        // Simple extraction: look for quoted text or capitalized words
        if let Some(start) = action.find('"') {
            if let Some(end) = action[start + 1..].find('"') {
                return Some(action[start + 1..start + 1 + end].to_string());
            }
        }

        // Look for capitalized entity
        let words: Vec<&str> = action.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            if word.chars().next().map_or(false, |c| c.is_uppercase()) && i > 0 {
                return Some(word.to_string());
            }
        }

        None
    }

    /// Extract entity near a noun
    fn extract_entity_near(&self, _noun: &str, action: &str) -> Option<String> {
        self.extract_main_entity(action)
    }

    /// Extract list items from action result
    fn extract_list(&self, action: &str) -> Option<Vec<String>> {
        // Look for "Found N items:" or numbered lists
        if action.contains("Found") || action.contains("items") {
            // Simple extraction - in real impl, parse structured results
            return Some(vec!["item1".to_string(), "item2".to_string()]);
        }
        None
    }
}

impl Default for ContextResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_reference() {
        let resolver = ContextResolver::new();
        assert!(resolver.contains_reference("open it"));
        assert!(resolver.contains_reference("delete that file"));
        assert!(resolver.contains_reference("show me the first one"));
        assert!(!resolver.contains_reference("open chrome"));
    }

    #[test]
    fn test_extract_references() {
        let resolver = ContextResolver::new();
        let refs = resolver.extract_references("open the file");
        assert!(!refs.is_empty());
    }
}
