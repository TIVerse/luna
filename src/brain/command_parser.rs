//! Command parsing logic
//!
//! Parses natural language text into structured commands using regex patterns.
//! Optimized with RegexSet for parallel pattern matching.

use once_cell::sync::Lazy;
use regex::{Regex, RegexSet, Captures};
use std::collections::HashMap;
use crate::error::{LunaError, Result};
use tracing::{debug, info};
use serde::{Deserialize, Serialize};

/// Intent types that LUNA can understand
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IntentType {
    /// Launch an application
    LaunchApp,
    /// Close an application
    CloseApp,
    /// Find a file or folder
    FindFile,
    /// Open a folder
    OpenFolder,
    /// System control (sleep, shutdown, restart)
    SystemControl,
    /// Volume control (up, down, mute)
    VolumeControl,
    /// Window management (move, resize, minimize, maximize)
    WindowManagement,
    /// Media control (play, pause, next, previous)
    MediaControl,
    /// Search the web
    SearchWeb,
    /// Create a reminder
    Reminder,
    /// Take a note
    Note,
    /// Answer a question
    Question,
    /// Get current time
    GetTime,
    /// Get current date
    GetDate,
    /// Unknown/unrecognized command
    Unknown,
}

/// A parsed command with intent and extracted entities
#[derive(Debug, Clone)]
pub struct ParsedCommand {
    /// The intent/action type
    pub intent: IntentType,
    /// Extracted entities (app_name, file_name, etc.)
    pub entities: HashMap<String, String>,
    /// The original text
    pub original_text: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

/// Command pattern for regex-based matching
struct CommandPattern {
    regex: Regex,
    intent: IntentType,
    extract_entities: fn(&Captures) -> HashMap<String, String>,
}

/// Command parser that converts text to structured commands
pub struct CommandParser {
    patterns: Vec<CommandPattern>,
    regex_set: RegexSet,
}

impl CommandParser {
    /// Create a new command parser with predefined patterns
    pub fn new() -> Self {
        let patterns = Self::build_patterns();
        
        // Build RegexSet for fast pre-filtering
        let pattern_strings: Vec<&str> = patterns
            .iter()
            .map(|p| p.regex.as_str())
            .collect();
        
        let regex_set = RegexSet::new(pattern_strings)
            .expect("Failed to build RegexSet");
        
        Self {
            patterns,
            regex_set,
        }
    }
    
    /// Parse text into a command (optimized with RegexSet)
    pub fn parse(&self, text: &str) -> Result<ParsedCommand> {
        let normalized = self.normalize_text(text);
        debug!("Parsing text: '{}' (normalized: '{}')", text, normalized);
        
        // Fast pre-filter with RegexSet (parallel matching)
        let matches = self.regex_set.matches(&normalized);
        
        // Only evaluate matched patterns (typically 1-2 instead of all 20+)
        for idx in matches.iter() {
            if let Some(captures) = self.patterns[idx].regex.captures(&normalized) {
                let entities = (self.patterns[idx].extract_entities)(&captures);
                let intent = self.patterns[idx].intent.clone();
                
                info!("Matched intent: {:?}, entities: {:?}", intent, entities);
                
                return Ok(ParsedCommand {
                    intent,
                    entities,
                    original_text: text.to_string(),
                    confidence: 0.95, // High confidence for regex matches
                });
            }
        }
        
        // No pattern matched
        info!("No pattern matched for: '{}'", text);
        Ok(ParsedCommand {
            intent: IntentType::Unknown,
            entities: HashMap::new(),
            original_text: text.to_string(),
            confidence: 0.0,
        })
    }
    
    /// Normalize text for better matching
    fn normalize_text(&self, text: &str) -> String {
        text.to_lowercase()
            .trim()
            .to_string()
    }
    
    /// Build all command patterns
    fn build_patterns() -> Vec<CommandPattern> {
        vec![
            // Launch app: "open chrome", "launch firefox", "start vscode"
            CommandPattern {
                regex: Regex::new(r"^(?:open|launch|start|run)\s+(.+)$").unwrap(),
                intent: IntentType::LaunchApp,
                extract_entities: |caps| {
                    let mut map = HashMap::new();
                    map.insert("app_name".to_string(), caps[1].to_string());
                    map
                },
            },
            
            // Close app: "close chrome", "quit firefox", "exit vscode"
            CommandPattern {
                regex: Regex::new(r"^(?:close|quit|exit|kill)\s+(.+)$").unwrap(),
                intent: IntentType::CloseApp,
                extract_entities: |caps| {
                    let mut map = HashMap::new();
                    map.insert("app_name".to_string(), caps[1].to_string());
                    map
                },
            },
            
            // Find file: "find budget.pdf", "search for report", "locate document"
            CommandPattern {
                regex: Regex::new(r"^(?:find|search\s+for|locate)\s+(.+)$").unwrap(),
                intent: IntentType::FindFile,
                extract_entities: |caps| {
                    let mut map = HashMap::new();
                    map.insert("file_name".to_string(), caps[1].to_string());
                    map
                },
            },
            
            // Open folder: "open downloads folder", "show me documents"
            CommandPattern {
                regex: Regex::new(r"^(?:open|show\s+me)\s+(?:the\s+)?(.+?)\s+folder$").unwrap(),
                intent: IntentType::OpenFolder,
                extract_entities: |caps| {
                    let mut map = HashMap::new();
                    map.insert("folder_name".to_string(), caps[1].to_string());
                    map
                },
            },
            
            // Volume control: "volume up", "turn down volume", "mute"
            CommandPattern {
                regex: Regex::new(r"^(?:volume|sound)\s+(up|down|mute|unmute)$").unwrap(),
                intent: IntentType::VolumeControl,
                extract_entities: |caps| {
                    let mut map = HashMap::new();
                    map.insert("action".to_string(), caps[1].to_string());
                    map
                },
            },
            
            CommandPattern {
                regex: Regex::new(r"^(?:turn\s+)?(?:up|down)\s+(?:the\s+)?volume$").unwrap(),
                intent: IntentType::VolumeControl,
                extract_entities: |caps| {
                    let mut map = HashMap::new();
                    let text = caps.get(0).unwrap().as_str();
                    let action = if text.contains("up") { "up" } else { "down" };
                    map.insert("action".to_string(), action.to_string());
                    map
                },
            },
            
            CommandPattern {
                regex: Regex::new(r"^mute$").unwrap(),
                intent: IntentType::VolumeControl,
                extract_entities: |_caps| {
                    let mut map = HashMap::new();
                    map.insert("action".to_string(), "mute".to_string());
                    map
                },
            },
            
            // System control: "lock computer", "shutdown", "restart"
            CommandPattern {
                regex: Regex::new(r"^(?:lock|sleep|shutdown|restart)\s*(?:computer|system)?$").unwrap(),
                intent: IntentType::SystemControl,
                extract_entities: |caps| {
                    let mut map = HashMap::new();
                    let text = caps.get(0).unwrap().as_str();
                    let action = if text.contains("lock") {
                        "lock"
                    } else if text.contains("sleep") {
                        "sleep"
                    } else if text.contains("shutdown") {
                        "shutdown"
                    } else {
                        "restart"
                    };
                    map.insert("action".to_string(), action.to_string());
                    map
                },
            },
            
            // Media control: "play music", "pause", "next song"
            CommandPattern {
                regex: Regex::new(r"^(play|pause|stop|next|previous)(?:\s+(?:song|music|track))?$").unwrap(),
                intent: IntentType::MediaControl,
                extract_entities: |caps| {
                    let mut map = HashMap::new();
                    map.insert("action".to_string(), caps[1].to_string());
                    map
                },
            },
            
            // Search web: "search for rust tutorials", "google machine learning"
            CommandPattern {
                regex: Regex::new(r"^(?:search\s+(?:for|the\s+web\s+for)|google)\s+(.+)$").unwrap(),
                intent: IntentType::SearchWeb,
                extract_entities: |caps| {
                    let mut map = HashMap::new();
                    map.insert("query".to_string(), caps[1].to_string());
                    map
                },
            },
            
            // Reminder: "remind me about meeting in 30 minutes"
            CommandPattern {
                regex: Regex::new(r"^remind\s+me\s+(?:about\s+)?(.+?)\s+in\s+(.+)$").unwrap(),
                intent: IntentType::Reminder,
                extract_entities: |caps| {
                    let mut map = HashMap::new();
                    map.insert("message".to_string(), caps[1].to_string());
                    map.insert("duration".to_string(), caps[2].to_string());
                    map
                },
            },
            
            // Note: "take a note buy milk"
            CommandPattern {
                regex: Regex::new(r"^(?:take\s+a\s+note|note|write\s+down)\s*:?\s*(.+)$").unwrap(),
                intent: IntentType::Note,
                extract_entities: |caps| {
                    let mut map = HashMap::new();
                    map.insert("content".to_string(), caps[1].to_string());
                    map
                },
            },
            
            // Time: "what time is it", "current time"
            CommandPattern {
                regex: Regex::new(r"^(?:what\s+time\s+is\s+it|current\s+time|tell\s+me\s+the\s+time)").unwrap(),
                intent: IntentType::GetTime,
                extract_entities: |_caps| HashMap::new(),
            },
            
            // Date: "what's the date", "today's date"
            CommandPattern {
                regex: Regex::new(r"^(?:what(?:'s|\s+is)\s+the\s+date|today(?:'s|\s+)date|current\s+date)").unwrap(),
                intent: IntentType::GetDate,
                extract_entities: |_caps| HashMap::new(),
            },
            
            // Question: "what is...", "how do I...", "why..."
            CommandPattern {
                regex: Regex::new(r"^(?:what|how|why|when|where|who)\s+.+$").unwrap(),
                intent: IntentType::Question,
                extract_entities: |caps| {
                    let mut map = HashMap::new();
                    map.insert("question".to_string(), caps.get(0).unwrap().as_str().to_string());
                    map
                },
            },
        ]
    }
}

impl Default for CommandParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_launch_app() {
        let parser = CommandParser::new();
        let result = parser.parse("open chrome").unwrap();
        
        assert_eq!(result.intent, IntentType::LaunchApp);
        assert_eq!(result.entities.get("app_name"), Some(&"chrome".to_string()));
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn test_parse_close_app() {
        let parser = CommandParser::new();
        let result = parser.parse("close firefox").unwrap();
        
        assert_eq!(result.intent, IntentType::CloseApp);
        assert_eq!(result.entities.get("app_name"), Some(&"firefox".to_string()));
    }

    #[test]
    fn test_parse_find_file() {
        let parser = CommandParser::new();
        let result = parser.parse("find budget.pdf").unwrap();
        
        assert_eq!(result.intent, IntentType::FindFile);
        assert_eq!(result.entities.get("file_name"), Some(&"budget.pdf".to_string()));
    }

    #[test]
    fn test_parse_volume_control() {
        let parser = CommandParser::new();
        
        let result = parser.parse("volume up").unwrap();
        assert_eq!(result.intent, IntentType::VolumeControl);
        assert_eq!(result.entities.get("action"), Some(&"up".to_string()));
        
        let result = parser.parse("mute").unwrap();
        assert_eq!(result.intent, IntentType::VolumeControl);
        assert_eq!(result.entities.get("action"), Some(&"mute".to_string()));
    }

    #[test]
    fn test_parse_time_date() {
        let parser = CommandParser::new();
        
        let result = parser.parse("what time is it").unwrap();
        assert_eq!(result.intent, IntentType::GetTime);
        
        let result = parser.parse("what's the date").unwrap();
        assert_eq!(result.intent, IntentType::GetDate);
    }

    #[test]
    fn test_parse_unknown() {
        let parser = CommandParser::new();
        let result = parser.parse("blah blah nonsense").unwrap();
        
        assert_eq!(result.intent, IntentType::Unknown);
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn test_case_insensitive() {
        let parser = CommandParser::new();
        let result = parser.parse("OPEN CHROME").unwrap();
        
        assert_eq!(result.intent, IntentType::LaunchApp);
        assert_eq!(result.entities.get("app_name"), Some(&"chrome".to_string()));
    }

    #[test]
    fn test_parse_reminder() {
        let parser = CommandParser::new();
        let result = parser.parse("remind me about meeting in 30 minutes").unwrap();
        
        assert_eq!(result.intent, IntentType::Reminder);
        assert_eq!(result.entities.get("message"), Some(&"meeting".to_string()));
        assert_eq!(result.entities.get("duration"), Some(&"30 minutes".to_string()));
    }
}
