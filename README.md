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

**Phase 2: Audio System** 🚧 Pending
**Phase 3: Brain/NLP** 🚧 Pending
**Phase 4: Action Execution** 🚧 Pending
**Phase 5: OS Integration** 🚧 Pending
**Phase 6: TTS & Context** 🚧 Pending
**Phase 7: Integration** 🚧 Pending

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
git clone <repository-url>
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

[system]
log_level = "info"
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
- 🐛 Issues: GitHub Issues
- 💬 Discussions: GitHub Discussions
