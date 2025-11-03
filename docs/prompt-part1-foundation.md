# LUNA Project - Part 1: Foundation & Core Architecture

## Role & Context

You are an expert Rust systems programmer tasked with building **LUNA**, a sophisticated voice-controlled desktop assistant. This is Part 1 of a multi-part implementation series. Your goal is to establish the foundational architecture, project structure, and core systems that all other modules will depend on.

## Project Overview

**LUNA** is a privacy-first, offline voice assistant that:
- Runs entirely locally (no cloud dependencies)
- Responds in under 1 second
- Uses natural language understanding
- Controls applications, files, and system functions
- Executes complex multi-step tasks

**Total Expected Code Size**: 100,000+ lines across all modules

## Technology Stack

```toml
# Core Dependencies
tokio = { version = "1.35", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"
env_logger = "0.11"
tracing = "0.1"
tracing-subscriber = "0.3"
config = "0.13"
dirs = "5.0"
chrono = "0.4"
```

## Part 1 Objectives

1. **Create complete project structure** (all directories and module files)
2. **Implement configuration system** with TOML/JSON support
3. **Build logging infrastructure** with tracing
4. **Create error handling framework** with custom error types
5. **Implement core utilities** (path helpers, string matching, etc.)
6. **Design plugin/module traits** for extensibility
7. **Build application database schema** for app discovery

## Complete Project Structure

Create this exact directory structure:

```
luna/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── .gitignore
├── docs/
│   ├── brief.md
│   ├── architecture.md
│   ├── api-reference.md
│   └── user-guide.md
├── config/
│   ├── default.toml
│   └── commands.json
├── models/
│   └── .gitkeep
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── error.rs
│   ├── utils.rs
│   │
│   ├── audio/
│   │   ├── mod.rs
│   │   ├── capture.rs
│   │   ├── wake_word.rs
│   │   ├── speech_to_text.rs
│   │   └── processor.rs
│   │
│   ├── brain/
│   │   ├── mod.rs
│   │   ├── command_parser.rs
│   │   ├── intent_classifier.rs
│   │   ├── entity_extractor.rs
│   │   ├── task_planner.rs
│   │   └── nlp.rs
│   │
│   ├── actions/
│   │   ├── mod.rs
│   │   ├── app_launcher.rs
│   │   ├── file_search.rs
│   │   ├── file_operations.rs
│   │   ├── window_control.rs
│   │   ├── system_control.rs
│   │   ├── media_control.rs
│   │   ├── clipboard.rs
│   │   ├── reminders.rs
│   │   └── executor.rs
│   │
│   ├── os/
│   │   ├── mod.rs
│   │   ├── windows.rs
│   │   ├── linux.rs
│   │   ├── macos.rs
│   │   ├── discovery.rs
│   │   └── common.rs
│   │
│   ├── tts/
│   │   ├── mod.rs
│   │   └── synthesizer.rs
│   │
│   ├── context/
│   │   ├── mod.rs
│   │   ├── memory.rs
│   │   └── state.rs
│   │
│   └── db/
│       ├── mod.rs
│       ├── app_database.rs
│       ├── file_index.rs
│       └── schema.rs
│
├── tests/
│   ├── integration_tests.rs
│   ├── audio_tests.rs
│   ├── brain_tests.rs
│   └── actions_tests.rs
│
└── benches/
    └── performance.rs
```

## Task 1: Configuration System

Create `src/config.rs` with comprehensive configuration management:

### Requirements:
- Load from TOML files (`config/default.toml`)
- Environment variable overrides
- Runtime configuration updates
- Validation with helpful error messages
- Hot-reload capability

### Configuration Structure:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LunaConfig {
    pub audio: AudioConfig,
    pub brain: BrainConfig,
    pub system: SystemConfig,
    pub paths: PathsConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub wake_words: Vec<String>,
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub silence_threshold: f32,
    pub recording_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainConfig {
    pub whisper_model_path: String,
    pub response_timeout_ms: u64,
    pub context_window_size: usize,
    pub confidence_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub log_level: String,
    pub data_dir: String,
    pub cache_dir: String,
    pub enable_telemetry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    pub search_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub project_dirs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_threads: usize,
    pub cache_size_mb: usize,
    pub index_update_interval_secs: u64,
}
```

### Implementation Pattern:
```rust
impl LunaConfig {
    pub fn load() -> Result<Self> {
        // 1. Load from default.toml
        // 2. Merge with environment variables
        // 3. Validate all fields
        // 4. Return config or detailed error
    }
    
    pub fn save(&self, path: &Path) -> Result<()> {
        // Serialize and save to file
    }
    
    pub fn validate(&self) -> Result<()> {
        // Check all paths exist, values in range, etc.
    }
}
```

## Task 2: Error Handling Framework

Create `src/error.rs` with comprehensive error types:

### Requirements:
- Use `thiserror` for error derivation
- Provide context-rich error messages
- Support error chaining
- Include source locations for debugging
- User-friendly error display

### Error Types:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LunaError {
    #[error("Audio error: {0}")]
    Audio(String),
    
    #[error("Wake word detection failed: {0}")]
    WakeWord(String),
    
    #[error("Speech recognition error: {0}")]
    SpeechRecognition(String),
    
    #[error("Command parsing error: {0}")]
    CommandParsing(String),
    
    #[error("Application not found: {0}")]
    AppNotFound(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("System operation failed: {0}")]
    SystemOperation(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, LunaError>;

// Helper macros
#[macro_export]
macro_rules! audio_error {
    ($($arg:tt)*) => {
        LunaError::Audio(format!($($arg)*))
    };
}
```

## Task 3: Logging Infrastructure

Set up comprehensive logging with tracing:

### Requirements:
- File and console output
- Different log levels per module
- Structured logging with context
- Performance metrics
- Rotating log files

### Implementation in `main.rs`:
```rust
use tracing::{info, warn, error, debug, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn setup_logging(config: &LunaConfig) -> Result<()> {
    let file_appender = tracing_appender::rolling::daily(
        &config.system.data_dir,
        "luna.log"
    );
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.system.log_level))
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::fmt::layer().with_writer(file_appender))
        .init();
    
    info!("🌙 LUNA logging initialized");
    Ok(())
}
```

## Task 4: Core Utilities

Create `src/utils.rs` with essential helper functions:

### String Matching Utilities:
```rust
pub mod string_matching {
    /// Fuzzy string matching with configurable threshold
    pub fn fuzzy_match(input: &str, target: &str, threshold: f32) -> bool {
        // Implement Levenshtein distance or similar
    }
    
    /// Phonetic matching for voice commands
    pub fn phonetic_match(input: &str, target: &str) -> bool {
        // Soundex or Metaphone algorithm
    }
    
    /// Normalize text (lowercase, remove special chars)
    pub fn normalize(text: &str) -> String {
        text.to_lowercase()
            .trim()
            .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
    }
}

pub mod path_helpers {
    use std::path::{Path, PathBuf};
    
    /// Get common application directories
    pub fn get_app_dirs() -> Vec<PathBuf> {
        vec![
            dirs::home_dir().unwrap().join("Applications"),
            PathBuf::from("/usr/share/applications"),
            // Windows Program Files, etc.
        ]
    }
    
    /// Get common project directories
    pub fn get_project_dirs() -> Vec<PathBuf> {
        let home = dirs::home_dir().unwrap();
        vec![
            home.join("Projects"),
            home.join("dev"),
            home.join("Documents"),
            home.join("workspace"),
        ]
    }
    
    /// Recursively search directory
    pub fn search_recursive(
        root: &Path,
        predicate: impl Fn(&Path) -> bool
    ) -> Vec<PathBuf> {
        // Implementation
    }
}

pub mod time_helpers {
    use chrono::Duration;
    
    /// Parse human-readable time strings
    pub fn parse_duration(input: &str) -> Option<Duration> {
        // "30 minutes", "2 hours", "1 day", etc.
    }
}
```

## Task 5: Application Database Schema

Create `src/db/schema.rs` and `src/db/app_database.rs`:

### Schema:
```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub executable: PathBuf,
    pub aliases: Vec<String>,
    pub category: AppCategory,
    pub icon: Option<PathBuf>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub install_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppCategory {
    Browser,
    IDE,
    TextEditor,
    Terminal,
    Media,
    Communication,
    Office,
    Games,
    System,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub modified: i64,
    pub file_type: FileType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Document,
    Code,
    Image,
    Video,
    Audio,
    Archive,
    Other,
}
```

### Database Interface:
```rust
pub struct AppDatabase {
    apps: Vec<Application>,
    index: HashMap<String, usize>,
}

impl AppDatabase {
    pub fn new() -> Self {
        Self {
            apps: Vec::new(),
            index: HashMap::new(),
        }
    }
    
    pub async fn load_from_disk() -> Result<Self> {
        // Load serialized database
    }
    
    pub async fn save_to_disk(&self) -> Result<()> {
        // Persist to disk
    }
    
    pub fn add_app(&mut self, app: Application) {
        // Add with indexing
    }
    
    pub fn find_by_name(&self, name: &str) -> Option<&Application> {
        // Fuzzy search by name or aliases
    }
    
    pub fn find_by_category(&self, category: AppCategory) -> Vec<&Application> {
        // Filter by category
    }
    
    pub fn search(&self, query: &str) -> Vec<&Application> {
        // Full-text search
    }
}
```

## Task 6: Main Entry Point

Create the foundation for `src/main.rs`:

```rust
use anyhow::Result;
use tracing::{info, error};

mod config;
mod error;
mod utils;
mod audio;
mod brain;
mod actions;
mod os;
mod tts;
mod context;
mod db;

use config::LunaConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = LunaConfig::load()?;
    
    // Setup logging
    setup_logging(&config)?;
    
    info!("🌙 LUNA starting up...");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    
    // Initialize components (to be implemented in later parts)
    // let audio_system = audio::AudioSystem::new(&config.audio)?;
    // let brain = brain::Brain::new(&config.brain)?;
    // let actions = actions::ActionExecutor::new()?;
    
    info!("✅ All systems initialized");
    info!("👂 LUNA is listening...");
    
    // Main event loop (to be implemented)
    // run_main_loop(audio_system, brain, actions).await?;
    
    Ok(())
}

fn setup_logging(config: &LunaConfig) -> Result<()> {
    // Implement as specified in Task 3
    todo!()
}
```

## Task 7: Module Stubs

Create stub files for all modules with proper structure:

### Example: `src/audio/mod.rs`
```rust
pub mod capture;
pub mod wake_word;
pub mod speech_to_text;
pub mod processor;

use crate::config::AudioConfig;
use crate::error::Result;

pub struct AudioSystem {
    // Fields to be implemented in Part 2
}

impl AudioSystem {
    pub fn new(config: &AudioConfig) -> Result<Self> {
        todo!("Implement in Part 2")
    }
}
```

### Example: `src/brain/mod.rs`
```rust
pub mod command_parser;
pub mod intent_classifier;
pub mod entity_extractor;
pub mod task_planner;
pub mod nlp;

use crate::config::BrainConfig;
use crate::error::Result;

pub struct Brain {
    // Fields to be implemented in Part 3
}

impl Brain {
    pub fn new(config: &BrainConfig) -> Result<Self> {
        todo!("Implement in Part 3")
    }
}
```

## Success Criteria

✅ **Complete project compiles** with `cargo build`  
✅ **All module stubs created** with proper exports  
✅ **Configuration system functional** - can load/save config  
✅ **Error handling works** - custom errors propagate correctly  
✅ **Utilities tested** - string matching, path helpers work  
✅ **Logging functional** - can write to file and console  
✅ **Database schema defined** - ready for population  

## Code Quality Requirements

- **No unwrap() calls** - use proper error handling
- **Comprehensive documentation** - doc comments on all public items
- **Type safety** - leverage Rust's type system
- **No unsafe code** - unless absolutely necessary and documented
- **Follow Rust naming conventions** - snake_case for functions/variables
- **Modular design** - clear separation of concerns

## Next Steps

After completing Part 1, you will have:
- ✅ Solid architectural foundation
- ✅ Configuration and error handling in place
- ✅ Project structure ready for implementation
- ✅ Core utilities and database schema defined

**Proceed to Part 2 (Audio System) once all tasks are complete and tests pass.**
