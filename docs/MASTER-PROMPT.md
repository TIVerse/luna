# 🌙 LUNA Voice Assistant - Master Implementation Prompt

## Executive Summary

You are tasked with building **LUNA**, a sophisticated, privacy-first voice-controlled desktop assistant written in Rust. This is a complete, production-ready system with 100,000+ lines of code organized into 7 modular components.

## Project Vision

**LUNA** is an offline voice assistant that:
- Listens continuously for wake words ("Hey Luna")
- Understands natural language commands
- Executes complex multi-step tasks
- Controls applications, files, and system functions
- Responds in under 1 second
- Runs entirely locally (zero cloud dependencies)
- Works on Windows, Linux, and macOS

## Architecture Overview

```
┌─────────────────────────────────────────────┐
│              LUNA ASSISTANT                  │
└─────────────────────────────────────────────┘
         │
    ┌────┴────┬──────────┬──────────┐
    │         │          │          │
    ▼         ▼          ▼          ▼
┌────────┐┌────────┐┌────────┐┌────────┐
│ Audio  ││ Brain  ││Actions ││Context │
│ System ││ System ││Executor││Manager │
└────────┘└────────┘└────────┘└────────┘
    │         │          │          │
    ▼         ▼          ▼          ▼
Microphone  NLP/AI   OS APIs    Memory
```

## Implementation Sequence

### **Phase 1: Foundation** (`prompt-part1-foundation.md`)
**Timeline:** Week 1-2 | **Lines:** ~10,000

**Deliverables:**
- Complete project structure (40+ files)
- Configuration system (TOML + env vars)
- Error handling framework (custom types)
- Logging infrastructure (tracing)
- Core utilities (string matching, paths)
- Database schemas (apps, files)

**Key Outputs:**
```
luna/src/
├── main.rs           # Entry point
├── config.rs         # Configuration
├── error.rs          # Error types
├── utils.rs          # Utilities
└── [module stubs]    # All module directories
```

---

### **Phase 2: Audio Input** (`prompt-part2-audio.md`)
**Timeline:** Week 3-4 | **Lines:** ~8,000

**Deliverables:**
- Microphone capture (cpal, continuous)
- Wake word detection (Porcupine)
- Speech-to-text (Whisper AI)
- Audio preprocessing (VAD, noise reduction)

**Key Outputs:**
```
luna/src/audio/
├── capture.rs         # Mic input + ring buffer
├── wake_word.rs       # Porcupine integration
├── speech_to_text.rs  # Whisper integration
└── processor.rs       # Audio preprocessing
```

---

### **Phase 3: Brain/Understanding** (`prompt-part3-brain.md`)
**Timeline:** Week 5-6 | **Lines:** ~12,000

**Deliverables:**
- Command parser (50+ regex patterns)
- Intent classifier (machine learning)
- Entity extractor (apps, files, numbers)
- Task planner (multi-step breakdown)

**Key Outputs:**
```
luna/src/brain/
├── command_parser.rs    # Text → Intent
├── intent_classifier.rs # Classification
├── entity_extractor.rs  # Extract entities
└── task_planner.rs      # Plan execution
```

---

### **Phase 4: Action Execution** (`prompt-part4-actions.md`)
**Timeline:** Week 7-8 | **Lines:** ~15,000

**Deliverables:**
- App launcher (find and launch apps)
- File search (name/content)
- System control (volume, power)
- Window management (move, resize)
- Media control (play/pause)
- Clipboard operations

**Key Outputs:**
```
luna/src/actions/
├── app_launcher.rs      # Launch apps
├── file_search.rs       # Find files
├── system_control.rs    # System settings
├── window_control.rs    # Window mgmt
├── media_control.rs     # Media playback
└── executor.rs          # Task execution
```

---

### **Phase 5: OS Integration** (`prompt-part5-os-integration.md`)
**Timeline:** Week 9-10 | **Lines:** ~20,000

**Deliverables:**
- Windows implementation (Win32 APIs)
- Linux implementation (D-Bus, .desktop)
- macOS implementation (Cocoa, LaunchServices)
- Cross-platform app discovery

**Key Outputs:**
```
luna/src/os/
├── windows.rs    # Windows-specific
├── linux.rs      # Linux-specific
├── macos.rs      # macOS-specific
└── discovery.rs  # App detection
```

---

### **Phase 6: TTS & Context** (`prompt-part6-tts-context.md`)
**Timeline:** Week 11 | **Lines:** ~8,000

**Deliverables:**
- Text-to-speech synthesis
- Conversation memory (context)
- State tracking (active apps)

**Key Outputs:**
```
luna/src/tts/
└── synthesizer.rs    # Voice output

luna/src/context/
├── memory.rs         # Conversation history
└── state.rs          # Current state
```

---

### **Phase 7: Integration** (`prompt-part7-integration.md`)
**Timeline:** Week 12 | **Lines:** ~10,000

**Deliverables:**
- Main event loop (async)
- Component integration
- Error recovery
- Performance optimization
- Comprehensive testing
- User documentation

**Key Outputs:**
- Fully integrated system
- Test suite (80%+ coverage)
- Benchmarks
- README and docs

---

## Technical Specifications

### Core Dependencies
```toml
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Audio
cpal = "0.15"
whisper-rs = "0.11"
pv-porcupine = "2.2"

# Utilities
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
regex = "1.10"
tracing = "0.1"

# Platform-specific
[target.'cfg(windows)'.dependencies]
windows = "0.52"
```

### Performance Requirements
- **Response time:** < 1000ms (wake word → action → response)
- **Wake word detection:** < 100ms
- **Speech recognition:** < 500ms
- **CPU (idle):** < 5%
- **Memory (idle):** < 200MB
- **Uptime:** 24+ hours stable

### Quality Requirements
- **Intent accuracy:** 95%+
- **Test coverage:** 80%+
- **Zero panics** in production code
- **Error handling** on all operations
- **Documentation** on all public APIs

## Code Quality Standards

### Rust Best Practices
```rust
// ✅ DO: Use Result types
pub async fn launch_app(name: &str) -> Result<()> { }

// ❌ DON'T: Use unwrap/panic
pub fn bad_function() {
    let x = some_option.unwrap(); // ❌
}

// ✅ DO: Comprehensive error context
.map_err(|e| LunaError::Audio(format!("Failed to capture: {}", e)))?

// ✅ DO: Documentation
/// Launches an application by name.
/// 
/// # Arguments
/// * `name` - Application name or alias
pub async fn launch_app(name: &str) -> Result<()>
```

### Module Organization
- One concept per file
- Clear public interfaces
- Private implementation details
- Comprehensive tests per module

## Testing Strategy

```rust
// Unit tests per module
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_command() { }
    
    #[tokio::test]
    async fn test_launch_app() { }
}

// Integration tests
tests/integration_tests.rs

// Benchmarks
benches/performance.rs
```

## Example Commands (50+ total)

**Application Control:**
- "Luna, open VS Code"
- "Luna, close Chrome"
- "Luna, switch to Firefox"

**File Operations:**
- "Luna, find my budget report"
- "Luna, search documents for presentation"
- "Luna, open the Downloads folder"

**System Control:**
- "Luna, volume up"
- "Luna, lock computer"
- "Luna, what's my CPU usage?"

**Multi-Step:**
- "Luna, open VS Code and the Luna project"
- "Luna, close all Chrome tabs except YouTube"
- "Luna, organize my downloads folder"

## Success Criteria

### Technical Metrics
- ✅ All phases complete (1-7)
- ✅ 100,000+ lines of code
- ✅ All tests passing
- ✅ Performance targets met
- ✅ Zero critical bugs

### Functional Metrics
- ✅ 50+ commands working
- ✅ Cross-platform support
- ✅ Offline operation
- ✅ Context awareness
- ✅ Error recovery

### Quality Metrics
- ✅ Clean code (no warnings)
- ✅ Documented APIs
- ✅ Comprehensive tests
- ✅ Production-ready

## Getting Started

### Step 1: Read Documentation
```bash
1. Read docs/brief.md (project vision)
2. Read docs/prompt-summary.md (overview)
3. Read docs/README-PROMPTS.md (guide)
```

### Step 2: Sequential Implementation
```bash
1. Implement Phase 1 (Foundation)
   → cargo build
   → cargo test
   → git commit

2. Implement Phase 2 (Audio)
   → cargo build
   → cargo test
   → git commit

# Continue through all phases...
```

### Step 3: Testing & Integration
```bash
cargo test --all
cargo build --release
./target/release/luna
```

## Expected Timeline

**Solo Developer:**
- 12 weeks full-time
- 24 weeks part-time

**Team of 4:**
- 6 weeks parallel development
- 2 weeks integration/testing

## Final Deliverable

A production-ready voice assistant featuring:
- ✅ Natural language understanding
- ✅ Application control
- ✅ File management
- ✅ System automation
- ✅ Multi-step task execution
- ✅ Privacy-focused (offline)
- ✅ Fast response (< 1s)
- ✅ Cross-platform support

## Next Actions

1. **Review** all prompt files (part 1-7)
2. **Setup** development environment
3. **Download** required models (Whisper, Porcupine)
4. **Start** with Part 1 (Foundation)
5. **Test** after each phase
6. **Integrate** incrementally

## Resources

- **Prompts:** `docs/prompt-part*.md` (7 files)
- **Summary:** `docs/prompt-summary.md`
- **Guide:** `docs/README-PROMPTS.md`
- **Vision:** `docs/brief.md`

---

## 🚀 Let's Build LUNA!

This is your complete guide to building a sophisticated, production-ready voice assistant. Each prompt is carefully engineered to produce high-quality, maintainable Rust code.

Follow the phases sequentially, test thoroughly, and you'll have a powerful voice assistant that rivals commercial offerings while maintaining complete privacy and offline capability.

**The future of voice control is local, private, and powerful. Let's build it! 🌙**
