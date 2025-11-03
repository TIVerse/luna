# LUNA Project - Part 6: TTS and Context System

## Overview
Implement text-to-speech for responses and conversational context memory.

## Dependencies
```toml
tts = "0.26"
```

## Modules

### 1. TTS Synthesizer (`src/tts/synthesizer.rs`)
Convert text responses to speech.

```rust
pub struct TextToSpeech {
    tts: tts::Tts,
}

impl TextToSpeech {
    pub fn new() -> Result<Self>;
    pub async fn speak(&self, text: &str) -> Result<()>;
    pub async fn speak_with_voice(&self, text: &str, voice: &str) -> Result<()>;
    pub fn stop(&self) -> Result<()>;
}
```

### 2. Context Memory (`src/context/memory.rs`)
Remember conversation history for contextual understanding.

```rust
pub struct ConversationMemory {
    history: VecDeque<ConversationEntry>,
    max_size: usize,
}

#[derive(Debug, Clone)]
pub struct ConversationEntry {
    pub timestamp: i64,
    pub user_input: String,
    pub parsed_intent: IntentType,
    pub action_taken: String,
    pub success: bool,
}

impl ConversationMemory {
    pub fn new(max_size: usize) -> Self;
    pub fn add_entry(&mut self, entry: ConversationEntry);
    pub fn get_context(&self) -> Vec<&ConversationEntry>;
    pub fn get_last_action(&self) -> Option<&ConversationEntry>;
}
```

### 3. State Manager (`src/context/state.rs`)
Track current application state and active windows.

```rust
pub struct StateManager {
    pub active_app: Option<String>,
    pub open_windows: Vec<String>,
    pub last_opened_file: Option<PathBuf>,
}
```

## Success Criteria
- ✅ TTS works with natural voice
- ✅ Context remembered across commands
- ✅ Referential commands work ("close it", "search there")
