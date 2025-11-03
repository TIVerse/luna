//! Multi-intent and compositional parsing
//!
//! Handles compound commands like "open Chrome and play music" or
//! "after 10 minutes, mute".

use crate::brain::command_parser::{CommandParser, IntentType, ParsedCommand};
use crate::brain::types::Entity;
use crate::error::Result;
use std::collections::HashMap;
use tracing::debug;

/// A multi-intent command with coordination
#[derive(Debug, Clone)]
pub struct MultiIntent {
    /// Individual command segments
    pub segments: Vec<IntentSegment>,

    /// Coordination type
    pub coordination: CoordinationType,

    /// Original full text
    pub original_text: String,
}

/// A single intent segment in a multi-intent command
#[derive(Debug, Clone)]
pub struct IntentSegment {
    /// The parsed command
    pub command: ParsedCommand,

    /// Position in the sequence
    pub position: usize,

    /// Temporal modifier (e.g., "after 5 minutes")
    pub temporal: Option<TemporalModifier>,
}

/// How intents are coordinated
#[derive(Debug, Clone, PartialEq)]
pub enum CoordinationType {
    /// Sequential ("and then", "then")
    Sequential,

    /// Parallel ("and", "also")
    Parallel,

    /// Conditional ("if", "when")
    Conditional,

    /// Temporal ("after", "before", "in")
    Temporal,
}

/// Temporal modifier for delayed execution
#[derive(Debug, Clone)]
pub struct TemporalModifier {
    /// Type of temporal relationship
    pub relation: TemporalRelation,

    /// Duration or time
    pub duration: Option<chrono::Duration>,

    /// Absolute time
    pub time: Option<chrono::NaiveTime>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemporalRelation {
    After,  // "after 5 minutes"
    Before, // "before 3pm"
    In,     // "in 10 seconds"
    At,     // "at 3pm"
}

/// Parser for multi-intent commands
pub struct MultiIntentParser {
    base_parser: CommandParser,
}

impl MultiIntentParser {
    /// Create a new multi-intent parser
    pub fn new() -> Self {
        Self {
            base_parser: CommandParser::new(),
        }
    }

    /// Parse text into multiple intents if present
    pub fn parse(&self, text: &str) -> Result<MultiIntent> {
        debug!("Parsing multi-intent: '{}'", text);

        // Detect coordinators
        let (segments, coordination) = self.segment_text(text)?;

        // Parse each segment
        let mut intent_segments = Vec::new();
        for (i, segment_text) in segments.iter().enumerate() {
            // Extract temporal modifier if present
            let (cleaned_text, temporal) = self.extract_temporal(segment_text)?;

            // Parse the cleaned command
            let command = self.base_parser.parse(&cleaned_text)?;

            intent_segments.push(IntentSegment {
                command,
                position: i,
                temporal,
            });
        }

        Ok(MultiIntent {
            segments: intent_segments,
            coordination,
            original_text: text.to_string(),
        })
    }

    /// Segment text by coordinators
    fn segment_text(&self, text: &str) -> Result<(Vec<String>, CoordinationType)> {
        let normalized = text.to_lowercase();

        // Check for coordinators in order of precedence
        if normalized.contains(" and then ") || normalized.contains(" then ") {
            let parts: Vec<String> = text
                .split(&[',', ';'][..])
                .chain(text.split(" and then "))
                .chain(text.split(" then "))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            return Ok((parts, CoordinationType::Sequential));
        }

        if normalized.contains(" and ") {
            let parts: Vec<String> = text
                .split(" and ")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            return Ok((parts, CoordinationType::Parallel));
        }

        if normalized.contains(" if ") || normalized.contains(" when ") {
            let mut parts = Vec::new();
            if normalized.contains(" if ") {
                parts = text
                    .split(" if ")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            } else {
                parts = text
                    .split(" when ")
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            return Ok((parts, CoordinationType::Conditional));
        }

        if normalized.contains(" after ")
            || normalized.contains(" before ")
            || normalized.contains(" in ")
            || normalized.contains(" at ")
        {
            // For temporal, keep as single segment but mark coordination
            return Ok((vec![text.to_string()], CoordinationType::Temporal));
        }

        // Single intent, no coordination
        Ok((vec![text.to_string()], CoordinationType::Sequential))
    }

    /// Extract temporal modifier from text
    fn extract_temporal(&self, text: &str) -> Result<(String, Option<TemporalModifier>)> {
        use crate::utils::time_helpers;
        use regex::Regex;

        let normalized = text.to_lowercase();

        // "after X" pattern
        if let Some(after_idx) = normalized.find(" after ") {
            let before_text = &text[..after_idx];
            let after_text = &text[after_idx + 7..]; // " after " = 7 chars

            if let Some(duration) = time_helpers::parse_duration(after_text) {
                return Ok((
                    before_text.trim().to_string(),
                    Some(TemporalModifier {
                        relation: TemporalRelation::After,
                        duration: Some(duration),
                        time: None,
                    }),
                ));
            }
        }

        // "in X" pattern
        if let Some(in_idx) = normalized.find(" in ") {
            let before_text = &text[..in_idx];
            let after_text = &text[in_idx + 4..];

            if let Some(duration) = time_helpers::parse_duration(after_text) {
                return Ok((
                    before_text.trim().to_string(),
                    Some(TemporalModifier {
                        relation: TemporalRelation::In,
                        duration: Some(duration),
                        time: None,
                    }),
                ));
            }
        }

        Ok((text.to_string(), None))
    }

    /// Check if text contains multiple intents
    pub fn is_multi_intent(&self, text: &str) -> bool {
        let normalized = text.to_lowercase();

        normalized.contains(" and ")
            || normalized.contains(" then ")
            || normalized.contains(" after ")
            || normalized.contains(" before ")
            || normalized.contains(", ")
    }
}

impl Default for MultiIntentParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_intent() {
        let parser = MultiIntentParser::new();

        let result = parser.parse("open chrome").unwrap();
        assert_eq!(result.segments.len(), 1);
        assert_eq!(result.coordination, CoordinationType::Sequential);
    }

    #[test]
    fn test_parallel_coordination() {
        let parser = MultiIntentParser::new();

        let result = parser.parse("open chrome and play music").unwrap();
        assert_eq!(result.segments.len(), 2);
        assert_eq!(result.coordination, CoordinationType::Parallel);
    }

    #[test]
    fn test_sequential_coordination() {
        let parser = MultiIntentParser::new();

        let result = parser.parse("open chrome and then play music").unwrap();
        assert_eq!(result.segments.len(), 2);
        assert_eq!(result.coordination, CoordinationType::Sequential);
    }

    #[test]
    fn test_temporal_modifier() {
        let parser = MultiIntentParser::new();

        let result = parser.parse("mute after 10 minutes").unwrap();
        assert_eq!(result.segments.len(), 1);

        let segment = &result.segments[0];
        assert!(segment.temporal.is_some());

        let temporal = segment.temporal.as_ref().unwrap();
        assert_eq!(temporal.relation, TemporalRelation::After);
        assert!(temporal.duration.is_some());
    }

    #[test]
    fn test_is_multi_intent() {
        let parser = MultiIntentParser::new();

        assert!(parser.is_multi_intent("open chrome and play music"));
        assert!(parser.is_multi_intent("mute after 5 minutes"));
        assert!(!parser.is_multi_intent("open chrome"));
    }
}
