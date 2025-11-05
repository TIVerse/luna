# 🌙 LUNA Voice Assistant

A privacy-first, offline voice-controlled desktop assistant written in Rust.

## Features

- 🎤 **Always Listening**: Wake word detection with "Hey Luna"
- 🧠 **Natural Language**: Understands conversational commands
- 🔒 **Privacy First**: 100% offline, zero cloud dependencies
- ⚡ **Fast**: Sub-second response times
- 🖥️ **Cross-Platform**: Windows, Linux, and macOS support

## Current Status

**Phase 1: Foundation** ✅ **COMPLETE**
- Configuration system with TOML support
- Comprehensive error handling framework
- Logging infrastructure with file rotation
- Core utilities (string matching, path helpers)
- Database schemas for apps and files
- Complete project structure with module stubs

**Phase 2: Audio System** ✅ **COMPLETE**
- Real-time audio capture with lock-free ring buffers
- Wake word detection (Porcupine + energy-based)
- Voice Activity Detection (WebRTC VAD)
- Speech-to-text integration (Whisper + simulation mode)
- Audio processing pipeline (noise gate, normalization)
- **P0.1**: STT wiring in runtime loop with metrics

**Phase 3: Brain/NLP** ✅ **ENHANCED**
- Command parsing and intent classification
- Entity extraction with typed entities
- Task planning with dependency management
- **P0.2**: Multi-intent parsing (parallel + temporal coordination)
- **P0.3**: App discovery integration for classification boosting
- **P0.5**: Declarative grammar YAML with hot-reload
- Context awareness and conversation tracking
- Caching system for performance

**Phase 4: Action Execution** ✅ **COMPLETE**
- Task executor with retry logic
- App launching and closing
- File search and management
- System control (volume, lock, shutdown)
- Media control integration
- **P0.4**: Capability detection at startup

**Phase 5: OS Integration** ✅ **COMPLETE**
- Cross-platform OS abstractions
- Application discovery (Linux, macOS, Windows)
- System control backends
- Window management APIs

**Phase 6: TTS & Context** ✅ **COMPLETE**
- Text-to-speech system with message types
- **P0.7**: Conversation memory persistence (JSON)
- **P0.8**: Clarification loop for low-confidence commands
- Context resolution and reference tracking

**Phase 7: Integration** ✅ **COMPLETE**
- Full runtime loop with all components
- Event bus with pub-sub architecture
- **P0.6**: Comprehensive metrics instrumentation
- Configuration hot-reload
- CLI tools for diagnostics and testing

## Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Linux dependencies
sudo apt install libasound2-dev portaudio19-dev

# macOS dependencies
brew install portaudio
```

### Build and Run

```bash
# Clone repository
git clone https://github.com/TIVerse/luna.git
cd luna

# Build
cargo build --release

# Run
cargo run --release
```

## Configuration

Edit `config/default.toml` to customize settings:

```toml
[audio]
wake_words = ["hey luna", "okay luna"]
sample_rate = 16000
recording_timeout_secs = 10

[brain]
confidence_threshold = 0.7
whisper_model_path = "models/ggml-base.en.bin"

[system]
log_level = "info"
data_dir = "~/.local/share/luna"
```

### Grammar Patterns

Customize intent patterns in `config/brain_patterns.yaml`:

```yaml
version: "1.0"
intents:
  - name: LaunchApp
    priority: 100
    patterns:
      - pattern: "^(?:open|launch|start|run)\\s+(.+)$"
        entities:
          app_name: "$1"
```

## Project Structure

```
luna/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library root
│   ├── config.rs            # Configuration ✅
│   ├── error.rs             # Error handling ✅
│   ├── utils.rs             # Utilities ✅
│   ├── audio/               # Audio system (Phase 2)
│   ├── brain/               # NLP system (Phase 3)
│   ├── actions/             # Action executor (Phase 4)
│   ├── os/                  # OS integration (Phase 5)
│   ├── tts/                 # Text-to-speech (Phase 6)
│   ├── context/             # Context manager (Phase 6)
│   └── db/                  # Databases ✅
├── config/
│   └── default.toml         # Configuration file
├── models/                  # AI models directory
└── docs/                    # Documentation

✅ = Implemented in Phase 1
```

## CLI Tools

LUNA includes powerful CLI tools for testing and diagnostics:

```bash
# System diagnostics
luna doctor

# Brain/NLP tools
luna brain parse "open chrome"
luna brain classify "play music"
luna brain plan "open chrome and play music" --preview
luna brain grammar-reload

# Audio tools
luna audio devices
luna audio monitor --duration 10
luna audio test-wake --duration 30

# Index management
luna index --apps --files

# Metrics and events
luna metrics --detailed
luna events --tail
```

## Development

### Run Tests

```bash
cargo test
```

### Build Documentation

```bash
cargo doc --open
```

### Check Code

```bash
cargo clippy
cargo fmt --check
```

## Phase 1 Implementation

Phase 1 establishes the foundation:

- ✅ Complete project structure (40+ files)
- ✅ Configuration system with validation
- ✅ Error handling with custom types
- ✅ Logging with tracing
- ✅ Core utilities (fuzzy matching, path helpers, time parsing)
- ✅ Database schemas (Application, FileEntry)
- ✅ Module stubs for future phases

## Next Steps

Proceed to **Phase 2: Audio System** to implement:
- Microphone capture with cpal
- Wake word detection with Porcupine
- Speech-to-text with Whisper AI
- Audio preprocessing (VAD, noise reduction)

See `docs/prompt-part2-audio.md` for detailed instructions.

## License

MIT

## Contributing

Contributions welcome! Please read CONTRIBUTING.md first.

## Support

- 📚 Documentation: `docs/`
- 🐛 Issues: [GitHub Issues](https://github.com/TIVerse/luna/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/TIVerse/luna/discussions)

## Author

**Eshan Roy** - [eshanized](https://github.com/eshanized)
- Organization: [TIVerse](https://github.com/TIVerse)
- Email: m.eshanized@gmail.com
