//! LRU caching for brain system
//!
//! Caches parsed commands and task plans to reduce latency for repeated commands.

use crate::brain::command_parser::ParsedCommand;
use crate::brain::task_planner::TaskPlan;
use lru::LruCache;
use std::num::NonZeroUsize;
use parking_lot::RwLock;

/// Cache for parsed commands and task plans
pub struct BrainCache {
    /// Cache for parsed commands (text -> ParsedCommand)
    parse_cache: RwLock<LruCache<String, ParsedCommand>>,
    
    /// Cache for task plans (text -> TaskPlan)
    plan_cache: RwLock<LruCache<String, TaskPlan>>,
    
    /// Hit/miss statistics
    stats: RwLock<CacheStats>,
}

/// Cache statistics
#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub parse_hits: usize,
    pub parse_misses: usize,
    pub plan_hits: usize,
    pub plan_misses: usize,
}

impl CacheStats {
    /// Get parse hit rate
    pub fn parse_hit_rate(&self) -> f64 {
        let total = self.parse_hits + self.parse_misses;
        if total == 0 {
            return 0.0;
        }
        self.parse_hits as f64 / total as f64
    }
    
    /// Get plan hit rate
    pub fn plan_hit_rate(&self) -> f64 {
        let total = self.plan_hits + self.plan_misses;
        if total == 0 {
            return 0.0;
        }
        self.plan_hits as f64 / total as f64
    }
}

impl BrainCache {
    /// Create a new cache with default capacity (100 entries per cache)
    pub fn new() -> Self {
        Self::with_capacity(100, 100)
    }
    
    /// Create a cache with specific capacities
    pub fn with_capacity(parse_capacity: usize, plan_capacity: usize) -> Self {
        Self {
            parse_cache: RwLock::new(LruCache::new(
                NonZeroUsize::new(parse_capacity).unwrap()
            )),
            plan_cache: RwLock::new(LruCache::new(
                NonZeroUsize::new(plan_capacity).unwrap()
            )),
            stats: RwLock::new(CacheStats::default()),
        }
    }
    
    /// Get a parsed command from cache
    pub fn get_parsed(&self, text: &str) -> Option<ParsedCommand> {
        let normalized = Self::normalize_key(text);
        let mut cache = self.parse_cache.write();
        
        if let Some(parsed) = cache.get(&normalized) {
            self.stats.write().parse_hits += 1;
            Some(parsed.clone())
        } else {
            self.stats.write().parse_misses += 1;
            None
        }
    }
    
    /// Put a parsed command into cache
    pub fn put_parsed(&self, text: &str, parsed: ParsedCommand) {
        let normalized = Self::normalize_key(text);
        self.parse_cache.write().put(normalized, parsed);
    }
    
    /// Get a task plan from cache
    pub fn get_plan(&self, text: &str) -> Option<TaskPlan> {
        let normalized = Self::normalize_key(text);
        let mut cache = self.plan_cache.write();
        
        if let Some(plan) = cache.get(&normalized) {
            self.stats.write().plan_hits += 1;
            Some(plan.clone())
        } else {
            self.stats.write().plan_misses += 1;
            None
        }
    }
    
    /// Put a task plan into cache
    pub fn put_plan(&self, text: &str, plan: TaskPlan) {
        let normalized = Self::normalize_key(text);
        self.plan_cache.write().put(normalized, plan);
    }
    
    /// Clear all caches (e.g., after grammar reload)
    pub fn invalidate_all(&self) {
        self.parse_cache.write().clear();
        self.plan_cache.write().clear();
    }
    
    /// Clear only parse cache
    pub fn invalidate_parse(&self) {
        self.parse_cache.write().clear();
    }
    
    /// Clear only plan cache
    pub fn invalidate_plan(&self) {
        self.plan_cache.write().clear();
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read().clone()
    }
    
    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.write() = CacheStats::default();
    }
    
    /// Normalize cache key (lowercase, trim, collapse whitespace)
    fn normalize_key(text: &str) -> String {
        text.to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Get current cache sizes
    pub fn sizes(&self) -> (usize, usize) {
        (
            self.parse_cache.read().len(),
            self.plan_cache.read().len(),
        )
    }
}

impl Default for BrainCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::command_parser::IntentType;
    use crate::brain::intent_classifier::ClassificationResult;
    use std::collections::HashMap;
    
    fn create_test_parsed(intent: IntentType) -> ParsedCommand {
        ParsedCommand {
            intent,
            entities: HashMap::new(),
            original_text: "test".to_string(),
            confidence: 0.9,
        }
    }
    
    fn create_test_plan() -> TaskPlan {
        TaskPlan {
            steps: Vec::new(),
            dependencies: Vec::new(),
            classification: ClassificationResult {
                intent: IntentType::LaunchApp,
                confidence: 0.9,
                entities: HashMap::new(),
                alternatives: Vec::new(),
            },
            parallel_groups: Vec::new(),
            is_valid: true,
            validation_errors: Vec::new(),
        }
    }
    
    #[test]
    fn test_cache_creation() {
        let cache = BrainCache::new();
        assert_eq!(cache.sizes(), (0, 0));
    }
    
    #[test]
    fn test_parse_cache() {
        let cache = BrainCache::new();
        let parsed = create_test_parsed(IntentType::LaunchApp);
        
        // Miss on first access
        assert!(cache.get_parsed("open chrome").is_none());
        
        // Put and hit
        cache.put_parsed("open chrome", parsed.clone());
        assert!(cache.get_parsed("open chrome").is_some());
        
        // Case insensitive
        assert!(cache.get_parsed("OPEN CHROME").is_some());
        assert!(cache.get_parsed("open  chrome").is_some());
    }
    
    #[test]
    fn test_plan_cache() {
        let cache = BrainCache::new();
        let plan = create_test_plan();
        
        assert!(cache.get_plan("open chrome").is_none());
        
        cache.put_plan("open chrome", plan.clone());
        assert!(cache.get_plan("open chrome").is_some());
    }
    
    #[test]
    fn test_cache_invalidation() {
        let cache = BrainCache::new();
        let parsed = create_test_parsed(IntentType::LaunchApp);
        let plan = create_test_plan();
        
        cache.put_parsed("test", parsed);
        cache.put_plan("test", plan);
        
        assert_eq!(cache.sizes(), (1, 1));
        
        cache.invalidate_all();
        assert_eq!(cache.sizes(), (0, 0));
    }
    
    #[test]
    fn test_cache_stats() {
        let cache = BrainCache::new();
        let parsed = create_test_parsed(IntentType::LaunchApp);
        
        cache.put_parsed("test", parsed);
        
        // One hit
        cache.get_parsed("test");
        
        // Two misses
        cache.get_parsed("other1");
        cache.get_parsed("other2");
        
        let stats = cache.stats();
        assert_eq!(stats.parse_hits, 1);
        assert_eq!(stats.parse_misses, 2);
        assert!((stats.parse_hit_rate() - 0.333).abs() < 0.01);
    }
    
    #[test]
    fn test_lru_eviction() {
        let cache = BrainCache::with_capacity(2, 2);
        
        cache.put_parsed("cmd1", create_test_parsed(IntentType::LaunchApp));
        cache.put_parsed("cmd2", create_test_parsed(IntentType::CloseApp));
        cache.put_parsed("cmd3", create_test_parsed(IntentType::FindFile));
        
        // cmd1 should be evicted
        assert!(cache.get_parsed("cmd1").is_none());
        assert!(cache.get_parsed("cmd2").is_some());
        assert!(cache.get_parsed("cmd3").is_some());
    }
    
    #[test]
    fn test_key_normalization() {
        let key1 = BrainCache::normalize_key("  Open   Chrome  ");
        let key2 = BrainCache::normalize_key("open chrome");
        let key3 = BrainCache::normalize_key("OPEN CHROME");
        
        assert_eq!(key1, key2);
        assert_eq!(key2, key3);
    }
}
