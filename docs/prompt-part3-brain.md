# LUNA Project - Part 3: Brain/Understanding System

## Overview
Implement the command understanding system that parses text, classifies intent, extracts entities, and plans multi-step tasks.

## Dependencies
```toml
regex = "1.10"
once_cell = "1.19"
strsim = "0.11"  # String similarity
```

## Modules

### 1. Command Parser (`src/brain/command_parser.rs`)
Parse raw text into structured commands using regex patterns and NLP.

```rust
pub struct CommandParser {
    patterns: Vec<CommandPattern>,
}

#[derive(Debug)]
pub struct CommandPattern {
    pub regex: Regex,
    pub intent: IntentType,
    pub extract_entities: fn(&Captures) -> HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum IntentType {
    LaunchApp, CloseApp, FindFile, SystemControl,
    VolumeControl, WindowManagement, MediaControl,
    SearchWeb, Reminder, Note, Question,
}
```

### 2. Intent Classifier (`src/brain/intent_classifier.rs`)
Classify command intent with confidence scores using pattern matching and ML.

```rust
pub struct IntentClassifier {
    rules: Vec<ClassificationRule>,
}

pub struct ClassificationResult {
    pub intent: IntentType,
    pub confidence: f32,
    pub entities: HashMap<String, String>,
}
```

### 3. Entity Extractor (`src/brain/entity_extractor.rs`)
Extract app names, file names, numbers, dates, etc. from commands.

```rust
pub struct EntityExtractor;

impl EntityExtractor {
    pub fn extract_app_name(&self, text: &str) -> Option<String>;
    pub fn extract_file_name(&self, text: &str) -> Option<String>;
    pub fn extract_duration(&self, text: &str) -> Option<Duration>;
    pub fn extract_number(&self, text: &str) -> Option<i32>;
}
```

### 4. Task Planner (`src/brain/task_planner.rs`)
Break complex commands into sequential steps.

```rust
pub struct TaskPlanner;

pub struct TaskPlan {
    pub steps: Vec<ActionStep>,
    pub dependencies: Vec<(usize, usize)>,
}

pub struct ActionStep {
    pub action: ActionType,
    pub params: HashMap<String, String>,
}
```

## Command Patterns (Examples)
```rust
// "open {app}"
r"^(?:open|launch|start)\s+(.+)$"

// "close {app}"  
r"^(?:close|quit|exit)\s+(.+)$"

// "find {file}"
r"^(?:find|search\s+for|locate)\s+(.+)$"

// "volume up/down"
r"^volume\s+(up|down|mute)$"
```

## Success Criteria
- ✅ Parse 50+ command types
- ✅ 95%+ intent accuracy
- ✅ Handle variations
- ✅ Context awareness
