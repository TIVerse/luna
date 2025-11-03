//! LUNA Voice Assistant Library
//!
//! A privacy-first offline voice-controlled desktop assistant written in Rust.
//!
//! # Features
//! - Offline speech recognition using Whisper AI
//! - Wake word detection with Porcupine
//! - Natural language understanding
//! - Application control and automation
//! - File search and management
//! - System control (volume, power, etc.)
//! - Cross-platform support (Windows, Linux, macOS)
//!
//! # Architecture
//! - **Audio**: Microphone capture, wake word detection, speech-to-text
//! - **Brain**: Command parsing, intent classification, task planning (God-level enhanced)
//! - **Actions**: Execute commands (launch apps, search files, etc.)
//! - **OS**: Platform-specific integrations
//! - **TTS**: Text-to-speech for responses
//! - **Context**: Conversation memory and state tracking
//! - **DB**: Application and file databases
//!
//! # God-Level Brain Enhancements (Phase 3+)
//! - **Typed Entities**: Strong typing instead of string-based parameters
//! - **Declarative Grammar**: Hot-reloadable YAML pattern configuration
//! - **LRU Caching**: Fast-path for repeated commands
//! - **Context Awareness**: Resolve "it", "that file" references
//! - **Probabilistic Ranking**: Multi-signal confidence scoring
//! - **Clarification Loop**: Interactive disambiguation for low confidence
//! - **Multi-Intent**: Handle compound commands ("open Chrome and play music")
//! - **Async Pipeline**: Parallel entity extraction and classification
//! - **Advanced Planning**: Preconditions, postconditions, parallel execution
//! - **Knowledge Providers**: Trait-based dependency injection for app/file knowledge

// Public modules
pub mod actions;
pub mod audio;
pub mod brain;
pub mod cli;
pub mod config;
pub mod config_manager;
pub mod context;
pub mod db;
pub mod error;
pub mod events;
pub mod metrics;
pub mod os;
pub mod runtime;
pub mod subscribers;
pub mod tts;
pub mod utils;

// Re-export commonly used types
pub use config::LunaConfig;
pub use config_manager::ConfigManager;
pub use error::{ErrorCode, LunaError, Result};
pub use events::{EventBus, EventEnvelope, LunaEvent, BackpressureStrategy};
pub use metrics::{Metrics, MetricPhase, MetricTimer};
pub use runtime::{Lifecycle, LunaRuntime, RuntimeState};

// Re-export brain enhancements
pub use brain::{
    Brain, IntentType, ActionType,
    // God-level enhancements
    Entity, Entities, Confidence, ConfidenceFactor,
    KnownAppProvider, FileSystemProvider, CompositeProvider,
    BrainCache, GrammarConfig, CompiledGrammar,
    ConversationContext,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = "LUNA Voice Assistant";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_name() {
        assert_eq!(NAME, "LUNA Voice Assistant");
    }
}
