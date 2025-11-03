//! Probabilistic ranking with feature weights
//!
//! Multi-signal scoring system that combines pattern matching, entity validation,
//! context, and history to produce better confidence scores.

use crate::brain::command_parser::ParsedCommand;
use crate::brain::intent_classifier::ClassificationResult;
use crate::brain::contextualizer::ConversationContext;
use crate::brain::providers::CompositeProvider;
use crate::brain::types::{Confidence, ConfidenceFactor};
use crate::config::BrainConfig;
use std::collections::HashMap;

/// Feature weights for ranking
#[derive(Debug, Clone)]
pub struct RankingWeights {
    /// Weight for regex pattern match
    pub pattern_match: f32,
    
    /// Weight for known entity validation
    pub entity_validation: f32,
    
    /// Weight for context/history match
    pub context_match: f32,
    
    /// Weight for synonym overlap
    pub synonym_overlap: f32,
    
    /// Weight for recent success
    pub recent_success: f32,
    
    /// Weight for pattern priority
    pub pattern_priority: f32,
}

impl Default for RankingWeights {
    fn default() -> Self {
        Self {
            pattern_match: 0.40,      // 40% - core pattern matching
            entity_validation: 0.20,   // 20% - entity validation
            context_match: 0.15,       // 15% - historical context
            synonym_overlap: 0.10,     // 10% - synonym matching
            recent_success: 0.10,      // 10% - recent success rate
            pattern_priority: 0.05,    // 5% - pattern priority
        }
    }
}

impl RankingWeights {
    /// Load from config
    pub fn from_config(config: &BrainConfig) -> Self {
        // Could load from config in future
        Self::default()
    }
    
    /// Validate weights sum to ~1.0
    pub fn validate(&self) -> bool {
        let sum = self.pattern_match + self.entity_validation + 
                  self.context_match + self.synonym_overlap +
                  self.recent_success + self.pattern_priority;
        
        (sum - 1.0).abs() < 0.01
    }
}

/// Feature-based ranking scorer
pub struct RankingScorer {
    weights: RankingWeights,
}

impl RankingScorer {
    /// Create a new ranking scorer with default weights
    pub fn new() -> Self {
        Self {
            weights: RankingWeights::default(),
        }
    }
    
    /// Create with custom weights
    pub fn with_weights(weights: RankingWeights) -> Self {
        Self { weights }
    }
    
    /// Score a classification result using multiple features
    pub fn score(
        &self,
        parsed: &ParsedCommand,
        classification: &ClassificationResult,
        providers: &CompositeProvider,
        context: Option<&ConversationContext>,
    ) -> Confidence {
        let mut confidence = Confidence::new(classification.confidence);
        let mut total_score = 0.0;
        
        // 1. Pattern match score (base confidence)
        let pattern_score = classification.confidence * self.weights.pattern_match;
        total_score += pattern_score;
        confidence.add_factor(ConfidenceFactor::new(
            "pattern_match",
            pattern_score,
            format!("Regex pattern matched with {:.2} confidence", classification.confidence),
        ));
        
        // 2. Entity validation score
        let entity_score = self.score_entity_validation(parsed, providers);
        total_score += entity_score * self.weights.entity_validation;
        confidence.add_factor(ConfidenceFactor::new(
            "entity_validation",
            entity_score * self.weights.entity_validation,
            format!("Entities validated: {:.2}", entity_score),
        ));
        
        // 3. Context match score
        if let Some(ctx) = context {
            let context_score = self.score_context_match(&parsed.original_text, ctx);
            total_score += context_score * self.weights.context_match;
            confidence.add_factor(ConfidenceFactor::new(
                "context_match",
                context_score * self.weights.context_match,
                format!("Context similarity: {:.2}", context_score),
            ));
        }
        
        // 4. Recent success score
        if let Some(ctx) = context {
            let success_score = ctx.success_rate(&parsed.original_text);
            total_score += success_score * self.weights.recent_success;
            
            if success_score > 0.0 {
                confidence.add_factor(ConfidenceFactor::new(
                    "recent_success",
                    success_score * self.weights.recent_success,
                    format!("Historical success rate: {:.1}%", success_score * 100.0),
                ));
            }
        }
        
        // Set final score
        confidence.score = total_score.clamp(0.0, 1.0);
        
        confidence
    }
    
    /// Score entity validation
    fn score_entity_validation(&self, parsed: &ParsedCommand, providers: &CompositeProvider) -> f32 {
        let mut validation_score = 0.0;
        let mut total_entities = 0;
        
        for (key, value) in &parsed.entities {
            total_entities += 1;
            
            match key.as_str() {
                "app_name" => {
                    if providers.app_provider().is_known(value) {
                        validation_score += 1.0;
                    } else {
                        validation_score += 0.3; // Partial credit for unknown apps
                    }
                }
                "file_name" => {
                    if providers.fs_provider().is_file_like(value) {
                        validation_score += 1.0;
                    } else {
                        validation_score += 0.5;
                    }
                }
                "folder_name" => {
                    if providers.fs_provider().is_folder_like(value) {
                        validation_score += 1.0;
                    } else {
                        validation_score += 0.5;
                    }
                }
                _ => {
                    // Other entities get neutral score
                    validation_score += 0.7;
                }
            }
        }
        
        if total_entities == 0 {
            return 1.0; // No entities to validate = perfect score
        }
        
        validation_score / total_entities as f32
    }
    
    /// Score context/history match
    fn score_context_match(&self, text: &str, context: &ConversationContext) -> f32 {
        use crate::utils::string_matching::similarity_score;
        
        // Find similar commands in history
        let similar = context.find_similar_commands(text, 0.7);
        
        if similar.is_empty() {
            return 0.0;
        }
        
        // Average similarity of top 3 matches
        let top_similar: Vec<f32> = similar
            .iter()
            .take(3)
            .map(|cmd| similarity_score(text, cmd))
            .collect();
        
        if top_similar.is_empty() {
            return 0.0;
        }
        
        top_similar.iter().sum::<f32>() / top_similar.len() as f32
    }
    
    /// Get weights
    pub fn weights(&self) -> &RankingWeights {
        &self.weights
    }
    
    /// Set weights
    pub fn set_weights(&mut self, weights: RankingWeights) {
        self.weights = weights;
    }
}

impl Default for RankingScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::command_parser::IntentType;
    use crate::brain::providers::DefaultAppProvider;
    use std::sync::Arc;
    
    fn create_test_parsed(entities: HashMap<String, String>) -> ParsedCommand {
        ParsedCommand {
            intent: IntentType::LaunchApp,
            entities,
            original_text: "test".to_string(),
            confidence: 0.9,
        }
    }
    
    fn create_test_classification(confidence: f32) -> ClassificationResult {
        ClassificationResult {
            intent: IntentType::LaunchApp,
            confidence,
            entities: HashMap::new(),
            alternatives: Vec::new(),
        }
    }
    
    #[test]
    fn test_weights_validation() {
        let weights = RankingWeights::default();
        assert!(weights.validate());
    }
    
    #[test]
    fn test_ranking_scorer_creation() {
        let scorer = RankingScorer::new();
        assert!(scorer.weights().validate());
    }
    
    #[test]
    fn test_entity_validation_scoring() {
        let scorer = RankingScorer::new();
        let providers = CompositeProvider::default();
        
        let mut entities = HashMap::new();
        entities.insert("app_name".to_string(), "chrome".to_string());
        
        let parsed = create_test_parsed(entities);
        let score = scorer.score_entity_validation(&parsed, &providers);
        
        // Chrome is a known app, should score high
        assert!(score > 0.8);
    }
    
    #[test]
    fn test_full_scoring() {
        let scorer = RankingScorer::new();
        let providers = CompositeProvider::default();
        
        let mut entities = HashMap::new();
        entities.insert("app_name".to_string(), "chrome".to_string());
        
        let parsed = create_test_parsed(entities);
        let classification = create_test_classification(0.95);
        
        let confidence = scorer.score(&parsed, &classification, &providers, None);
        
        assert!(confidence.score > 0.0);
        assert!(confidence.score <= 1.0);
        assert!(!confidence.factors.is_empty());
    }
    
    #[test]
    fn test_with_context() {
        let scorer = RankingScorer::new();
        let providers = CompositeProvider::default();
        let mut context = ConversationContext::new();
        
        // Add some history
        let cmd = create_test_parsed(HashMap::new());
        context.add_command(cmd.clone(), HashMap::new(), true);
        context.add_command(cmd, HashMap::new(), true);
        
        let parsed = create_test_parsed(HashMap::new());
        let classification = create_test_classification(0.9);
        
        let confidence = scorer.score(&parsed, &classification, &providers, Some(&context));
        
        // Should have context factors
        assert!(confidence.factors.len() > 2);
    }
}
