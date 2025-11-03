//! Declarative grammar system for LUNA brain
//!
//! Loads intent patterns from YAML config files and compiles them to RegexSet
//! for fast matching. Supports hot-reloading and validation.

use crate::brain::command_parser::IntentType;
use crate::error::{LunaError, Result};
use regex::{Regex, RegexSet};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// Grammar configuration loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarConfig {
    pub version: String,
    pub intents: Vec<IntentDefinition>,
    
    #[serde(default)]
    pub synonyms: HashMap<String, HashMap<String, Vec<String>>>,
    
    #[serde(default)]
    pub slots: Vec<SlotDefinition>,
}

/// Intent definition with patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentDefinition {
    pub name: String,
    pub priority: u32,
    pub patterns: Vec<PatternDefinition>,
    pub examples: Vec<String>,
}

/// Pattern definition with entity extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDefinition {
    pub pattern: String,
    pub entities: HashMap<String, String>,
}

/// Slot (entity) type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotDefinition {
    pub name: String,
    
    #[serde(rename = "type")]
    pub slot_type: String,
    
    #[serde(default)]
    pub validation: Option<String>,
    
    #[serde(default)]
    pub parser: Option<String>,
    
    #[serde(default)]
    pub values: Vec<String>,
}

/// Compiled grammar ready for matching
pub struct CompiledGrammar {
    /// Fast pre-filter using RegexSet
    regex_set: RegexSet,
    
    /// Individual patterns with metadata
    patterns: Vec<CompiledPattern>,
    
    /// Synonym maps for normalization
    synonyms: HashMap<String, HashMap<String, Vec<String>>>,
    
    /// Slot definitions
    slots: HashMap<String, SlotDefinition>,
}

/// A single compiled pattern
pub struct CompiledPattern {
    pub regex: Regex,
    pub intent: IntentType,
    pub entity_map: HashMap<String, String>,
    pub priority: u32,
    pub original_pattern: String,
}

impl GrammarConfig {
    /// Load grammar from YAML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            warn!("Grammar file not found: {:?}, using built-in patterns", path);
            return Ok(Self::default_grammar());
        }
        
        let contents = std::fs::read_to_string(path)
            .map_err(|e| LunaError::Config(format!("Failed to read grammar file: {}", e)))?;
        
        let config: GrammarConfig = serde_yaml::from_str(&contents)
            .map_err(|e| LunaError::Config(format!("Failed to parse grammar YAML: {}", e)))?;
        
        info!("✅ Loaded grammar config version {} with {} intents", 
              config.version, config.intents.len());
        
        Ok(config)
    }
    
    /// Default built-in grammar as fallback
    pub fn default_grammar() -> Self {
        Self {
            version: "1.0".to_string(),
            intents: vec![
                IntentDefinition {
                    name: "LaunchApp".to_string(),
                    priority: 100,
                    patterns: vec![
                        PatternDefinition {
                            pattern: r"^(?:open|launch|start|run)\s+(.+)$".to_string(),
                            entities: HashMap::from([("app_name".to_string(), "$1".to_string())]),
                        },
                    ],
                    examples: vec!["open chrome".to_string()],
                },
                IntentDefinition {
                    name: "CloseApp".to_string(),
                    priority: 100,
                    patterns: vec![
                        PatternDefinition {
                            pattern: r"^(?:close|quit|exit|kill)\s+(.+)$".to_string(),
                            entities: HashMap::from([("app_name".to_string(), "$1".to_string())]),
                        },
                    ],
                    examples: vec!["close chrome".to_string()],
                },
            ],
            synonyms: HashMap::new(),
            slots: Vec::new(),
        }
    }
    
    /// Compile to executable grammar
    pub fn compile(&self) -> Result<CompiledGrammar> {
        let mut patterns = Vec::new();
        let mut pattern_strings = Vec::new();
        
        // Compile each intent
        for intent_def in &self.intents {
            let intent = Self::parse_intent_name(&intent_def.name)?;
            
            for pattern_def in &intent_def.patterns {
                // Compile regex
                let regex = Regex::new(&pattern_def.pattern)
                    .map_err(|e| LunaError::CommandParsing(
                        format!("Invalid regex pattern '{}': {}", pattern_def.pattern, e)
                    ))?;
                
                pattern_strings.push(pattern_def.pattern.clone());
                
                patterns.push(CompiledPattern {
                    regex: regex.clone(),
                    intent: intent.clone(),
                    entity_map: pattern_def.entities.clone(),
                    priority: intent_def.priority,
                    original_pattern: pattern_def.pattern.clone(),
                });
            }
        }
        
        // Build RegexSet for fast pre-filtering
        let regex_set = RegexSet::new(&pattern_strings)
            .map_err(|e| LunaError::Config(format!("Failed to build RegexSet: {}", e)))?;
        
        // Build slot map
        let slots: HashMap<String, SlotDefinition> = self.slots
            .iter()
            .map(|s| (s.name.clone(), s.clone()))
            .collect();
        
        info!("✅ Compiled {} patterns into grammar", patterns.len());
        
        Ok(CompiledGrammar {
            regex_set,
            patterns,
            synonyms: self.synonyms.clone(),
            slots,
        })
    }
    
    /// Parse intent name string to IntentType
    fn parse_intent_name(name: &str) -> Result<IntentType> {
        match name {
            "LaunchApp" => Ok(IntentType::LaunchApp),
            "CloseApp" => Ok(IntentType::CloseApp),
            "FindFile" => Ok(IntentType::FindFile),
            "OpenFolder" => Ok(IntentType::OpenFolder),
            "SystemControl" => Ok(IntentType::SystemControl),
            "VolumeControl" => Ok(IntentType::VolumeControl),
            "WindowManagement" => Ok(IntentType::WindowManagement),
            "MediaControl" => Ok(IntentType::MediaControl),
            "SearchWeb" => Ok(IntentType::SearchWeb),
            "Reminder" => Ok(IntentType::Reminder),
            "Note" => Ok(IntentType::Note),
            "Question" => Ok(IntentType::Question),
            "GetTime" => Ok(IntentType::GetTime),
            "GetDate" => Ok(IntentType::GetDate),
            _ => {
                warn!("Unknown intent name: {}, defaulting to Unknown", name);
                Ok(IntentType::Unknown)
            }
        }
    }
}

impl CompiledGrammar {
    /// Match text against all patterns
    pub fn match_text(&self, text: &str) -> Vec<(usize, &CompiledPattern)> {
        let matches = self.regex_set.matches(text);
        let mut results: Vec<(usize, &CompiledPattern)> = matches
            .iter()
            .map(|idx| (idx, &self.patterns[idx]))
            .collect();
        
        // Sort by priority (higher first)
        results.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
        
        results
    }
    
    /// Get synonym expansions for a term
    pub fn expand_synonyms(&self, category: &str, term: &str) -> Vec<String> {
        if let Some(cat_map) = self.synonyms.get(category) {
            if let Some(syns) = cat_map.get(term) {
                return syns.clone();
            }
        }
        vec![]
    }
    
    /// Get slot definition
    pub fn get_slot(&self, name: &str) -> Option<&SlotDefinition> {
        self.slots.get(name)
    }
    
    /// Validate entity against slot definition
    pub fn validate_entity(&self, slot_name: &str, value: &str) -> bool {
        if let Some(slot) = self.get_slot(slot_name) {
            // Check allowed values if specified
            if !slot.values.is_empty() {
                return slot.values.iter().any(|v| v.eq_ignore_ascii_case(value));
            }
            // Other validation could go here
        }
        true
    }
    
    /// Get number of patterns
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_default_grammar() {
        let grammar = GrammarConfig::default_grammar();
        assert_eq!(grammar.version, "1.0");
        assert!(!grammar.intents.is_empty());
    }
    
    #[test]
    fn test_compile_grammar() {
        let grammar = GrammarConfig::default_grammar();
        let compiled = grammar.compile().unwrap();
        
        assert!(compiled.pattern_count() > 0);
    }
    
    #[test]
    fn test_match_text() {
        let grammar = GrammarConfig::default_grammar();
        let compiled = grammar.compile().unwrap();
        
        let matches = compiled.match_text("open chrome");
        assert!(!matches.is_empty());
        
        let (_, pattern) = matches[0];
        assert_eq!(pattern.intent, IntentType::LaunchApp);
    }
    
    #[test]
    fn test_load_from_yaml() {
        let yaml = r#"
version: "1.0"
intents:
  - name: LaunchApp
    priority: 100
    patterns:
      - pattern: "open (.+)"
        entities:
          app_name: "$1"
    examples:
      - "open chrome"
synonyms: {}
slots: []
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml.as_bytes()).unwrap();
        
        let grammar = GrammarConfig::load_from_file(temp_file.path()).unwrap();
        assert_eq!(grammar.intents.len(), 1);
        assert_eq!(grammar.intents[0].name, "LaunchApp");
    }
    
    #[test]
    fn test_synonym_expansion() {
        let mut synonyms = HashMap::new();
        let mut apps = HashMap::new();
        apps.insert("chrome".to_string(), vec!["google chrome".to_string()]);
        synonyms.insert("apps".to_string(), apps);
        
        let compiled = CompiledGrammar {
            regex_set: RegexSet::new(&[r"test"]).unwrap(),
            patterns: vec![],
            synonyms,
            slots: HashMap::new(),
        };
        
        let expansions = compiled.expand_synonyms("apps", "chrome");
        assert_eq!(expansions, vec!["google chrome".to_string()]);
    }
    
    #[test]
    fn test_entity_validation() {
        let slot = SlotDefinition {
            name: "action".to_string(),
            slot_type: "string".to_string(),
            validation: None,
            parser: None,
            values: vec!["up".to_string(), "down".to_string()],
        };
        
        let mut slots = HashMap::new();
        slots.insert("action".to_string(), slot);
        
        let compiled = CompiledGrammar {
            regex_set: RegexSet::new(&[r"test"]).unwrap(),
            patterns: vec![],
            synonyms: HashMap::new(),
            slots,
        };
        
        assert!(compiled.validate_entity("action", "up"));
        assert!(compiled.validate_entity("action", "UP"));
        assert!(!compiled.validate_entity("action", "invalid"));
    }
}
