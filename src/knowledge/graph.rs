//! Knowledge graph for caching factual information
//!
//! Stores entities, relationships, and facts for quick lookup.

use crate::error::{LunaError, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Entity type in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    /// Person entity
    Person,
    /// Place/Location entity
    Place,
    /// Organization entity
    Organization,
    /// Event entity
    Event,
    /// Concept/Abstract entity
    Concept,
    /// Date/Time entity
    DateTime,
    /// Other entity types
    Other(String),
}

/// Fact confidence level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct FactConfidence(pub f32);

impl FactConfidence {
    /// Create a new confidence score (0.0 - 1.0)
    pub fn new(score: f32) -> Self {
        Self(score.clamp(0.0, 1.0))
    }

    /// High confidence threshold
    pub fn is_high(&self) -> bool {
        self.0 >= 0.8
    }

    /// Medium confidence threshold
    pub fn is_medium(&self) -> bool {
        self.0 >= 0.5 && self.0 < 0.8
    }

    /// Low confidence threshold
    pub fn is_low(&self) -> bool {
        self.0 < 0.5
    }
}

/// Knowledge graph entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Entity identifier
    pub id: String,
    /// Entity name/label
    pub name: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Entity description
    pub description: Option<String>,
    /// Aliases/alternative names
    pub aliases: Vec<String>,
    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Relationship type (e.g., "works_at", "located_in")
    pub relation_type: String,
    /// Source entity ID
    pub source: String,
    /// Target entity ID
    pub target: String,
    /// Confidence score
    pub confidence: FactConfidence,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Factual statement about an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    /// Fact ID
    pub id: String,
    /// Entity this fact is about
    pub entity_id: String,
    /// Fact predicate (e.g., "population", "capital")
    pub predicate: String,
    /// Fact value
    pub value: String,
    /// Confidence score
    pub confidence: FactConfidence,
    /// Source URL or reference
    pub source: Option<String>,
    /// Timestamp when fact was added
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Expiry time (for time-sensitive facts like weather)
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Fact {
    /// Check if fact has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }
}

/// Knowledge graph storage
pub struct KnowledgeGraph {
    /// Entities indexed by ID
    entities: Arc<DashMap<String, Entity>>,
    /// Entities indexed by name (lowercase)
    name_index: Arc<DashMap<String, String>>,
    /// Relationships
    relationships: Arc<DashMap<String, Vec<Relationship>>>,
    /// Facts indexed by entity ID
    facts: Arc<DashMap<String, Vec<Fact>>>,
    /// Cache statistics
    hits: Arc<parking_lot::Mutex<u64>>,
    misses: Arc<parking_lot::Mutex<u64>>,
}

impl KnowledgeGraph {
    /// Create a new knowledge graph
    pub fn new() -> Self {
        info!("🧠 Initializing knowledge graph");
        Self {
            entities: Arc::new(DashMap::new()),
            name_index: Arc::new(DashMap::new()),
            relationships: Arc::new(DashMap::new()),
            facts: Arc::new(DashMap::new()),
            hits: Arc::new(parking_lot::Mutex::new(0)),
            misses: Arc::new(parking_lot::Mutex::new(0)),
        }
    }

    /// Add an entity to the graph
    pub fn add_entity(&self, entity: Entity) {
        let id = entity.id.clone();
        let name = entity.name.to_lowercase();
        
        // Index by name and aliases
        self.name_index.insert(name, id.clone());
        for alias in &entity.aliases {
            self.name_index.insert(alias.to_lowercase(), id.clone());
        }
        
        self.entities.insert(id, entity);
    }

    /// Find entity by name (case-insensitive)
    pub fn find_entity(&self, name: &str) -> Option<Entity> {
        let name_lower = name.to_lowercase();
        
        if let Some(id_ref) = self.name_index.get(&name_lower) {
            let id = id_ref.value().clone();
            drop(id_ref);
            
            if let Some(entity_ref) = self.entities.get(&id) {
                *self.hits.lock() += 1;
                return Some(entity_ref.value().clone());
            }
        }
        
        *self.misses.lock() += 1;
        None
    }

    /// Get entity by ID
    pub fn get_entity(&self, id: &str) -> Option<Entity> {
        self.entities.get(id).map(|e| e.value().clone())
    }

    /// Add a relationship between entities
    pub fn add_relationship(&self, relationship: Relationship) {
        self.relationships
            .entry(relationship.source.clone())
            .or_insert_with(Vec::new)
            .push(relationship.clone());
    }

    /// Get relationships for an entity
    pub fn get_relationships(&self, entity_id: &str) -> Vec<Relationship> {
        self.relationships
            .get(entity_id)
            .map(|r| r.value().clone())
            .unwrap_or_default()
    }

    /// Add a fact to the graph
    pub fn add_fact(&self, fact: Fact) {
        debug!("Adding fact: {} = {}", fact.predicate, fact.value);
        self.facts
            .entry(fact.entity_id.clone())
            .or_insert_with(Vec::new)
            .push(fact);
    }

    /// Query facts about an entity
    pub fn query_facts(&self, entity_id: &str, predicate: Option<&str>) -> Vec<Fact> {
        if let Some(facts_ref) = self.facts.get(entity_id) {
            let facts = facts_ref.value();
            
            // Filter by predicate if specified
            let filtered: Vec<Fact> = if let Some(pred) = predicate {
                facts
                    .iter()
                    .filter(|f| f.predicate == pred && !f.is_expired())
                    .cloned()
                    .collect()
            } else {
                facts.iter().filter(|f| !f.is_expired()).cloned().collect()
            };
            
            if !filtered.is_empty() {
                *self.hits.lock() += 1;
            } else {
                *self.misses.lock() += 1;
            }
            
            filtered
        } else {
            *self.misses.lock() += 1;
            Vec::new()
        }
    }

    /// Search entities by type
    pub fn search_by_type(&self, entity_type: &EntityType) -> Vec<Entity> {
        self.entities
            .iter()
            .filter(|e| &e.value().entity_type == entity_type)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Clean up expired facts
    pub fn cleanup_expired(&self) {
        let mut cleaned_count = 0;
        
        for mut facts_ref in self.facts.iter_mut() {
            let facts = facts_ref.value_mut();
            let before = facts.len();
            facts.retain(|f| !f.is_expired());
            cleaned_count += before - facts.len();
        }
        
        if cleaned_count > 0 {
            info!("🧹 Cleaned up {} expired facts", cleaned_count);
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> KnowledgeGraphStats {
        let hits = *self.hits.lock();
        let misses = *self.misses.lock();
        let total = hits + misses;
        
        KnowledgeGraphStats {
            entity_count: self.entities.len(),
            relationship_count: self.relationships.iter().map(|r| r.value().len()).sum(),
            fact_count: self.facts.iter().map(|f| f.value().len()).sum(),
            cache_hits: hits,
            cache_misses: misses,
            hit_rate: if total > 0 {
                hits as f32 / total as f32
            } else {
                0.0
            },
        }
    }

    /// Clear all data (for testing)
    #[cfg(test)]
    pub fn clear(&self) {
        self.entities.clear();
        self.name_index.clear();
        self.relationships.clear();
        self.facts.clear();
        *self.hits.lock() = 0;
        *self.misses.lock() = 0;
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Knowledge graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraphStats {
    /// Number of entities
    pub entity_count: usize,
    /// Number of relationships
    pub relationship_count: usize,
    /// Number of facts
    pub fact_count: usize,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Hit rate (0.0 - 1.0)
    pub hit_rate: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entity(id: &str, name: &str) -> Entity {
        Entity {
            id: id.to_string(),
            name: name.to_string(),
            entity_type: EntityType::Person,
            description: Some(format!("Test entity {}", name)),
            aliases: vec![],
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_add_and_find_entity() {
        let graph = KnowledgeGraph::new();
        let entity = create_test_entity("1", "Rust Programming");
        
        graph.add_entity(entity.clone());
        
        let found = graph.find_entity("Rust Programming");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "1");
    }

    #[test]
    fn test_case_insensitive_search() {
        let graph = KnowledgeGraph::new();
        let entity = create_test_entity("1", "Rust Programming");
        
        graph.add_entity(entity);
        
        let found = graph.find_entity("rust programming");
        assert!(found.is_some());
    }

    #[test]
    fn test_add_fact() {
        let graph = KnowledgeGraph::new();
        
        let fact = Fact {
            id: "f1".to_string(),
            entity_id: "rust".to_string(),
            predicate: "creator".to_string(),
            value: "Graydon Hoare".to_string(),
            confidence: FactConfidence::new(0.95),
            source: Some("Wikipedia".to_string()),
            timestamp: chrono::Utc::now(),
            expires_at: None,
        };
        
        graph.add_fact(fact);
        
        let facts = graph.query_facts("rust", Some("creator"));
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].value, "Graydon Hoare");
    }

    #[test]
    fn test_expired_fact() {
        let graph = KnowledgeGraph::new();
        
        let expired_fact = Fact {
            id: "f1".to_string(),
            entity_id: "weather".to_string(),
            predicate: "temperature".to_string(),
            value: "20°C".to_string(),
            confidence: FactConfidence::new(0.9),
            source: None,
            timestamp: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
        };
        
        graph.add_fact(expired_fact);
        
        let facts = graph.query_facts("weather", Some("temperature"));
        assert_eq!(facts.len(), 0); // Should be filtered out
    }

    #[test]
    fn test_cache_stats() {
        let graph = KnowledgeGraph::new();
        let entity = create_test_entity("1", "Test");
        
        graph.add_entity(entity);
        
        // Hit
        graph.find_entity("Test");
        
        // Miss
        graph.find_entity("NonExistent");
        
        let stats = graph.stats();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }
}
