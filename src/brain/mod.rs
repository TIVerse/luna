//! Brain module for natural language understanding
//!
//! Coordinates command parsing, intent classification, entity extraction, and task planning.
//! Enhanced with typed entities, caching, context awareness, and declarative grammar.

pub mod command_parser;
pub mod entity_extractor;
pub mod intent_classifier;
pub mod nlp;
pub mod task_planner;

// God-level enhancements
pub mod cache;
pub mod contextualizer;
pub mod grammar;
pub mod multi_intent;
pub mod providers;
pub mod ranking;
pub mod types;

use crate::config::BrainConfig;
use crate::error::Result;
use command_parser::{CommandParser, ParsedCommand};
use entity_extractor::EntityExtractor;
use intent_classifier::{ClassificationResult, IntentClassifier};
use task_planner::{TaskPlan, TaskPlanner};
use tracing::info;

// Re-export key types
pub use cache::BrainCache;
pub use command_parser::IntentType;
pub use contextualizer::ConversationContext;
pub use grammar::{CompiledGrammar, GrammarConfig};
pub use providers::{CompositeProvider, FileSystemProvider, KnownAppProvider};
pub use task_planner::ActionType;
pub use types::{Confidence, ConfidenceFactor, Entities, Entity};

use std::path::Path;
use std::sync::Arc;

/// Brain system that coordinates all NLP components with god-level enhancements
pub struct Brain {
    parser: CommandParser,
    classifier: IntentClassifier,
    extractor: EntityExtractor,
    planner: TaskPlanner,

    // Enhancements
    cache: Arc<BrainCache>,
    context: Arc<parking_lot::RwLock<ConversationContext>>,
    providers: Arc<CompositeProvider>,
    grammar: Option<Arc<CompiledGrammar>>,
    ranker: Arc<ranking::RankingScorer>,

    config: BrainConfig,
}

impl Brain {
    /// Create a new brain instance
    pub fn new(config: &BrainConfig) -> Result<Self> {
        info!("Initializing brain system with god-level enhancements...");

        let parser = CommandParser::new();
        let classifier = IntentClassifier::new();
        let extractor = EntityExtractor::new();
        let planner = TaskPlanner::new();

        // Initialize enhancements
        let cache = Arc::new(BrainCache::new());
        let context = Arc::new(parking_lot::RwLock::new(ConversationContext::new()));
        let providers = Arc::new(CompositeProvider::default());
        let ranker = Arc::new(ranking::RankingScorer::new());

        // Try to load grammar from config
        let grammar = Self::try_load_grammar()?;

        info!("✅ Brain system initialized with caching, context, providers, and ranking");

        Ok(Self {
            parser,
            classifier,
            extractor,
            planner,
            cache,
            context,
            providers,
            grammar,
            ranker,
            config: config.clone(),
        })
    }

    /// Try to load grammar from file
    fn try_load_grammar() -> Result<Option<Arc<CompiledGrammar>>> {
        let grammar_path = Path::new("config/brain_patterns.yaml");

        if grammar_path.exists() {
            let config = GrammarConfig::load_from_file(grammar_path)?;
            let compiled = config.compile()?;
            Ok(Some(Arc::new(compiled)))
        } else {
            info!("No grammar file found, using built-in patterns");
            Ok(None)
        }
    }

    /// Reload grammar from file (for hot-reload)
    pub fn reload_grammar(&mut self) -> Result<()> {
        self.grammar = Self::try_load_grammar()?;

        // Invalidate caches since patterns changed
        self.cache.invalidate_all();

        info!("✅ Grammar reloaded and caches invalidated");
        Ok(())
    }

    /// Process text through the complete NLP pipeline with caching and context
    ///
    /// # Arguments
    /// * `text` - Raw text input from speech-to-text
    ///
    /// # Returns
    /// A complete task plan ready for execution
    pub fn process(&self, text: &str) -> Result<TaskPlan> {
        info!("🧠 Processing: \"{}\"", text);

        // Fast path: check plan cache first
        if let Some(cached_plan) = self.cache.get_plan(text) {
            info!("   ✨ Cache hit! Returning cached plan");
            return Ok(cached_plan);
        }

        // 1. Try to resolve context references (e.g., "open it")
        let resolved_text = self.resolve_context(text);

        // 2. Check parse cache
        let parsed = if let Some(cached_parsed) = self.cache.get_parsed(&resolved_text) {
            info!("   ✨ Parse cache hit");
            cached_parsed
        } else {
            let parsed = self.parser.parse(&resolved_text)?;
            self.cache.put_parsed(&resolved_text, parsed.clone());
            parsed
        };

        info!("   Parsed intent: {:?}", parsed.intent);

        // 3. Classify with confidence
        let classified = self.classifier.classify(&parsed)?;
        info!("   Confidence: {:.2}", classified.confidence);

        // 4. Plan tasks
        let plan = self.planner.plan(classified);
        info!("   Plan: {} steps", plan.steps.len());

        // Cache the plan
        self.cache.put_plan(text, plan.clone());

        Ok(plan)
    }

    /// Process with explicit context awareness and success tracking
    pub fn process_with_context(
        &self,
        text: &str,
        success_callback: Option<bool>,
    ) -> Result<TaskPlan> {
        let plan = self.process(text)?;

        // Record in context if callback provided
        if let Some(success) = success_callback {
            let parsed = self.parser.parse(text)?;
            let mut entities = std::collections::HashMap::new();

            // Convert legacy entities to typed (simplified for now)
            for (key, value) in &parsed.entities {
                entities.insert(key.clone(), Entity::String(value.clone()));
            }

            self.context.write().add_command(parsed, entities, success);
        }

        Ok(plan)
    }

    /// Resolve context references like "it", "that file", etc.
    fn resolve_context(&self, text: &str) -> String {
        let ctx = self.context.read();

        if let Some(resolved_entities) = ctx.resolve_references(text) {
            // Simple substitution: replace "it" with actual entity
            let mut resolved = text.to_string();

            for (entity_type, entity) in resolved_entities {
                if let Some(value) = entity.as_string() {
                    // Replace reference words with actual values
                    resolved = resolved
                        .replace("it", value)
                        .replace("that", value)
                        .replace("this", value);

                    info!("   🔗 Resolved '{}' to '{}'", entity_type, value);
                }
            }

            resolved
        } else {
            text.to_string()
        }
    }

    /// Parse text into a structured command
    pub fn parse(&self, text: &str) -> Result<ParsedCommand> {
        self.parser.parse(text)
    }

    /// Classify a parsed command
    pub fn classify(&self, command: &ParsedCommand) -> Result<ClassificationResult> {
        self.classifier.classify(command)
    }

    /// Create a task plan from classification
    pub fn plan(&self, classification: ClassificationResult) -> TaskPlan {
        self.planner.plan(classification)
    }

    /// Get the entity extractor
    pub fn extractor(&self) -> &EntityExtractor {
        &self.extractor
    }

    /// Add known application to the classifier
    pub fn add_known_app(&mut self, app_name: String) {
        self.classifier.add_known_app(app_name);
    }

    // Enhanced accessors

    /// Get cache statistics
    pub fn cache_stats(&self) -> cache::CacheStats {
        self.cache.stats()
    }

    /// Get conversation context (read-only)
    pub fn context(&self) -> Arc<parking_lot::RwLock<ConversationContext>> {
        Arc::clone(&self.context)
    }

    /// Clear all caches
    pub fn clear_cache(&self) {
        self.cache.invalidate_all();
    }

    /// Clear conversation context
    pub fn clear_context(&self) {
        self.context.write().clear();
    }

    /// Get success rate for a command
    pub fn command_success_rate(&self, text: &str) -> f32 {
        self.context.read().success_rate(text)
    }

    /// Check if providers know an app
    pub fn is_known_app(&self, name: &str) -> bool {
        self.providers.app_provider().is_known(name)
    }

    /// Get grammar if loaded
    pub fn grammar(&self) -> Option<Arc<CompiledGrammar>> {
        self.grammar.clone()
    }

    /// Async processing with parallel entity extraction and classification
    pub async fn process_async(&self, text: &str) -> Result<TaskPlan> {
        info!("🧠 Async processing: \"{}\"", text);

        // Fast path: check cache
        if let Some(cached_plan) = self.cache.get_plan(text) {
            info!("   ✨ Cache hit! Returning cached plan");
            return Ok(cached_plan);
        }

        // Resolve context
        let resolved_text = self.resolve_context(text);

        // Check if this is a multi-intent command
        let multi_parser = multi_intent::MultiIntentParser::new();
        if multi_parser.is_multi_intent(&resolved_text) {
            info!("   🔀 Multi-intent detected, using multi-intent parser");

            // Parse multi-intent command
            let multi_intent = multi_parser.parse(&resolved_text)?;

            // Process each segment
            let mut items = Vec::new();
            for segment in &multi_intent.segments {
                // Classify the segment
                let classified = self.classifier.classify(&segment.command)?;

                // Extract temporal duration if present
                let duration = segment.temporal.as_ref().and_then(|t| {
                    t.duration.map(|cd| {
                        std::time::Duration::from_secs(cd.num_seconds() as u64)
                    })
                });

                items.push((classified, duration));
            }

            // Plan multi-intent with coordination
            let plan = self.planner.plan_multi(items, multi_intent.coordination);
            info!("   Plan: {} steps (multi-intent)", plan.steps.len());

            // Cache the plan
            self.cache.put_plan(text, plan.clone());

            return Ok(plan);
        }

        // Single-intent path
        // Parse
        let parsed = if let Some(cached_parsed) = self.cache.get_parsed(&resolved_text) {
            info!("   ✨ Parse cache hit");
            cached_parsed
        } else {
            let parsed = self.parser.parse(&resolved_text)?;
            self.cache.put_parsed(&resolved_text, parsed.clone());
            parsed
        };

        info!("   Parsed intent: {:?}", parsed.intent);

        // Classify with enhanced ranking
        let mut classified = self.classifier.classify(&parsed)?;

        // Apply advanced ranking with all signals
        let ctx_read = self.context.read();
        let enhanced_confidence =
            self.ranker
                .score(&parsed, &classified, &self.providers, Some(&*ctx_read));
        drop(ctx_read);

        // Update classification with enhanced confidence
        classified.confidence = enhanced_confidence.score;

        info!("   Confidence: {:.2}", classified.confidence);

        // Check confidence threshold for clarification
        if classified.confidence < self.config.confidence_threshold {
            info!(
                "   ⚠️  Low confidence ({:.2} < {:.2}), may need clarification",
                classified.confidence, self.config.confidence_threshold
            );
        }

        // Plan tasks
        let plan = self.planner.plan(classified);
        info!("   Plan: {} steps", plan.steps.len());

        // Cache the plan
        self.cache.put_plan(text, plan.clone());

        Ok(plan)
    }

    /// Process with clarification support
    pub async fn process_with_clarification(
        &self,
        text: &str,
        event_bus: Option<Arc<crate::events::EventBus>>,
    ) -> Result<TaskPlan> {
        let plan = self.process_async(text).await?;

        // Check if clarification is needed
        if plan.classification.confidence < self.config.confidence_threshold {
            if let Some(bus) = event_bus {
                // Publish clarification request
                let missing_slots = self.identify_missing_slots(&plan);
                let suggestions = self.generate_suggestions(&plan);

                bus.publish(crate::events::LunaEvent::ClarificationRequested {
                    command: text.to_string(),
                    confidence: plan.classification.confidence,
                    missing_slots,
                    suggestions,
                })
                .await;
            }
        }

        Ok(plan)
    }

    /// Identify missing or uncertain slots
    fn identify_missing_slots(&self, plan: &TaskPlan) -> Vec<String> {
        let mut missing = Vec::new();

        for step in &plan.steps {
            if step.params.is_empty() {
                missing.push("required_parameter".to_string());
            }
        }

        missing
    }

    /// Generate suggestions for disambiguation
    fn generate_suggestions(&self, plan: &TaskPlan) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Add alternatives
        for (alt_intent, confidence) in &plan.classification.alternatives {
            suggestions.push(format!("{:?} (confidence: {:.2})", alt_intent, confidence));
        }

        // Limit to top 3
        suggestions.truncate(3);
        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_creation() {
        let config = BrainConfig::default();
        let brain = Brain::new(&config);
        assert!(brain.is_ok());
    }

    #[test]
    fn test_process_command() {
        let config = BrainConfig::default();
        let brain = Brain::new(&config).unwrap();

        let result = brain.process("open chrome");
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert!(!plan.steps.is_empty());
        assert_eq!(plan.classification.intent, IntentType::LaunchApp);
    }

    #[test]
    fn test_parse() {
        let config = BrainConfig::default();
        let brain = Brain::new(&config).unwrap();

        let result = brain.parse("close firefox");
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.intent, IntentType::CloseApp);
        assert_eq!(
            parsed.entities.get("app_name"),
            Some(&"firefox".to_string())
        );
    }

    #[test]
    fn test_end_to_end_pipeline() {
        let config = BrainConfig::default();
        let brain = Brain::new(&config).unwrap();

        // Test various commands
        let commands = vec![
            "open chrome",
            "close firefox",
            "find budget.pdf",
            "volume up",
            "what time is it",
        ];

        for cmd in commands {
            let result = brain.process(cmd);
            assert!(result.is_ok(), "Failed to process: {}", cmd);
        }
    }
}
