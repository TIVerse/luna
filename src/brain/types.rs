//! Strongly-typed entities and parameters for the brain system
//!
//! Replaces stringly-typed HashMap<String, String> with proper types
//! for better safety, IDE support, and refactorability.

use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Strongly-typed entity extracted from commands
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Entity {
    /// Application name
    App(String),
    
    /// File name or path
    File(String),
    
    /// Folder/directory name
    Folder(String),
    
    /// Duration (e.g., "30 minutes")
    Duration(Duration),
    
    /// Numeric value
    Number(i64),
    
    /// Percentage (0-100)
    Percent(u8),
    
    /// Time of day (hour, minute)
    TimeOfDay { hour: u8, minute: u8 },
    
    /// Date (year, month, day)
    Date { year: i32, month: u8, day: u8 },
    
    /// Web search query
    Query(String),
    
    /// Free-form text content
    Text(String),
    
    /// Action/command verb
    Action(String),
    
    /// URL
    Url(String),
    
    /// Contact name
    Contact(String),
    
    /// Language code (e.g., "en", "es")
    Language(String),
    
    /// Generic string (fallback)
    String(String),
}

impl Entity {
    /// Get entity as string if possible
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Entity::App(s) | Entity::File(s) | Entity::Folder(s) |
            Entity::Query(s) | Entity::Text(s) | Entity::Action(s) |
            Entity::Url(s) | Entity::Contact(s) | Entity::Language(s) |
            Entity::String(s) => Some(s),
            _ => None,
        }
    }
    
    /// Get entity type name
    pub fn type_name(&self) -> &'static str {
        match self {
            Entity::App(_) => "app",
            Entity::File(_) => "file",
            Entity::Folder(_) => "folder",
            Entity::Duration(_) => "duration",
            Entity::Number(_) => "number",
            Entity::Percent(_) => "percent",
            Entity::TimeOfDay { .. } => "time_of_day",
            Entity::Date { .. } => "date",
            Entity::Query(_) => "query",
            Entity::Text(_) => "text",
            Entity::Action(_) => "action",
            Entity::Url(_) => "url",
            Entity::Contact(_) => "contact",
            Entity::Language(_) => "language",
            Entity::String(_) => "string",
        }
    }
    
    /// Convert to legacy HashMap format for backward compatibility
    pub fn to_legacy_map(&self, key: &str) -> HashMap<String, String> {
        let mut map = HashMap::new();
        match self {
            Entity::App(s) => { map.insert("app_name".to_string(), s.clone()); }
            Entity::File(s) => { map.insert("file_name".to_string(), s.clone()); }
            Entity::Folder(s) => { map.insert("folder_name".to_string(), s.clone()); }
            Entity::Duration(d) => { map.insert("duration".to_string(), format!("{} seconds", d.num_seconds())); }
            Entity::Number(n) => { map.insert("number".to_string(), n.to_string()); }
            Entity::Percent(p) => { map.insert("percent".to_string(), p.to_string()); }
            Entity::TimeOfDay { hour, minute } => { 
                map.insert("time".to_string(), format!("{:02}:{:02}", hour, minute)); 
            }
            Entity::Date { year, month, day } => { 
                map.insert("date".to_string(), format!("{:04}-{:02}-{:02}", year, month, day)); 
            }
            Entity::Query(s) => { map.insert("query".to_string(), s.clone()); }
            Entity::Text(s) => { map.insert("content".to_string(), s.clone()); }
            Entity::Action(s) => { map.insert("action".to_string(), s.clone()); }
            Entity::Url(s) => { map.insert("url".to_string(), s.clone()); }
            Entity::Contact(s) => { map.insert("contact".to_string(), s.clone()); }
            Entity::Language(s) => { map.insert("language".to_string(), s.clone()); }
            Entity::String(s) => { map.insert(key.to_string(), s.clone()); }
        }
        map
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entity::App(s) => write!(f, "App({})", s),
            Entity::File(s) => write!(f, "File({})", s),
            Entity::Folder(s) => write!(f, "Folder({})", s),
            Entity::Duration(d) => write!(f, "Duration({} sec)", d.num_seconds()),
            Entity::Number(n) => write!(f, "Number({})", n),
            Entity::Percent(p) => write!(f, "Percent({}%)", p),
            Entity::TimeOfDay { hour, minute } => write!(f, "Time({:02}:{:02})", hour, minute),
            Entity::Date { year, month, day } => write!(f, "Date({:04}-{:02}-{:02})", year, month, day),
            Entity::Query(s) => write!(f, "Query({})", s),
            Entity::Text(s) => write!(f, "Text({})", s),
            Entity::Action(s) => write!(f, "Action({})", s),
            Entity::Url(s) => write!(f, "Url({})", s),
            Entity::Contact(s) => write!(f, "Contact({})", s),
            Entity::Language(s) => write!(f, "Language({})", s),
            Entity::String(s) => write!(f, "String({})", s),
        }
    }
}

/// Collection of entities extracted from a command
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Entities {
    entities: HashMap<String, Entity>,
}

impl Entities {
    /// Create a new empty entities collection
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Insert an entity
    pub fn insert(&mut self, key: String, entity: Entity) {
        self.entities.insert(key, entity);
    }
    
    /// Get an entity by key
    pub fn get(&self, key: &str) -> Option<&Entity> {
        self.entities.get(key)
    }
    
    /// Get mutable entity by key
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Entity> {
        self.entities.get_mut(key)
    }
    
    /// Check if contains key
    pub fn contains_key(&self, key: &str) -> bool {
        self.entities.contains_key(key)
    }
    
    /// Get number of entities
    pub fn len(&self) -> usize {
        self.entities.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
    
    /// Iterate over entities
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Entity)> {
        self.entities.iter()
    }
    
    /// Convert to legacy HashMap for backward compatibility
    pub fn to_legacy_hashmap(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (key, entity) in &self.entities {
            map.extend(entity.to_legacy_map(key));
        }
        map
    }
    
    /// Create from legacy HashMap
    pub fn from_legacy_hashmap(map: &HashMap<String, String>) -> Self {
        let mut entities = Self::new();
        
        for (key, value) in map {
            let entity = match key.as_str() {
                "app_name" => Entity::App(value.clone()),
                "file_name" => Entity::File(value.clone()),
                "folder_name" => Entity::Folder(value.clone()),
                "query" => Entity::Query(value.clone()),
                "content" => Entity::Text(value.clone()),
                "action" => Entity::Action(value.clone()),
                "url" => Entity::Url(value.clone()),
                "contact" => Entity::Contact(value.clone()),
                _ => {
                    // Try to parse as number or percent
                    if let Ok(n) = value.parse::<i64>() {
                        Entity::Number(n)
                    } else if value.ends_with('%') {
                        if let Ok(p) = value.trim_end_matches('%').parse::<u8>() {
                            Entity::Percent(p)
                        } else {
                            Entity::String(value.clone())
                        }
                    } else {
                        Entity::String(value.clone())
                    }
                }
            };
            entities.insert(key.clone(), entity);
        }
        
        entities
    }
}

/// Confidence score with justification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confidence {
    /// Raw confidence score (0.0 - 1.0)
    pub score: f32,
    
    /// Factors that contributed to the score
    pub factors: Vec<ConfidenceFactor>,
}

impl Confidence {
    /// Create a new confidence score
    pub fn new(score: f32) -> Self {
        Self {
            score: score.clamp(0.0, 1.0),
            factors: Vec::new(),
        }
    }
    
    /// Add a confidence factor
    pub fn add_factor(&mut self, factor: ConfidenceFactor) {
        self.factors.push(factor);
    }
    
    /// Is confidence above threshold?
    pub fn is_confident(&self, threshold: f32) -> bool {
        self.score >= threshold
    }
    
    /// Get the dominant factor
    pub fn dominant_factor(&self) -> Option<&ConfidenceFactor> {
        self.factors.iter().max_by(|a, b| {
            a.weight.partial_cmp(&b.weight).unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

/// Factor contributing to confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceFactor {
    /// Factor name
    pub name: String,
    
    /// Weight of this factor
    pub weight: f32,
    
    /// Description
    pub description: String,
}

impl ConfidenceFactor {
    /// Create a new confidence factor
    pub fn new(name: impl Into<String>, weight: f32, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            weight,
            description: description.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_creation() {
        let app = Entity::App("chrome".to_string());
        assert_eq!(app.type_name(), "app");
        assert_eq!(app.as_string(), Some("chrome"));
    }
    
    #[test]
    fn test_entity_display() {
        let app = Entity::App("firefox".to_string());
        assert_eq!(format!("{}", app), "App(firefox)");
        
        let num = Entity::Number(42);
        assert_eq!(format!("{}", num), "Number(42)");
        
        let time = Entity::TimeOfDay { hour: 15, minute: 30 };
        assert_eq!(format!("{}", time), "Time(15:30)");
    }
    
    #[test]
    fn test_entities_collection() {
        let mut entities = Entities::new();
        entities.insert("app".to_string(), Entity::App("chrome".to_string()));
        entities.insert("number".to_string(), Entity::Number(42));
        
        assert_eq!(entities.len(), 2);
        assert!(entities.contains_key("app"));
        
        let app = entities.get("app").unwrap();
        assert_eq!(app.as_string(), Some("chrome"));
    }
    
    #[test]
    fn test_legacy_conversion() {
        let mut legacy = HashMap::new();
        legacy.insert("app_name".to_string(), "chrome".to_string());
        legacy.insert("action".to_string(), "up".to_string());
        
        let entities = Entities::from_legacy_hashmap(&legacy);
        assert_eq!(entities.len(), 2);
        
        let converted = entities.to_legacy_hashmap();
        assert_eq!(converted.get("app_name"), Some(&"chrome".to_string()));
    }
    
    #[test]
    fn test_confidence() {
        let mut conf = Confidence::new(0.85);
        conf.add_factor(ConfidenceFactor::new("pattern_match", 0.9, "Regex matched"));
        conf.add_factor(ConfidenceFactor::new("known_app", 0.1, "App validated"));
        
        assert!(conf.is_confident(0.7));
        assert!(!conf.is_confident(0.9));
        
        let dominant = conf.dominant_factor().unwrap();
        assert_eq!(dominant.name, "pattern_match");
    }
}
