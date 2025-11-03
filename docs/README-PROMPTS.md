# LUNA Project Prompts - Quick Reference

## 📁 Prompt Files Overview

This directory contains a complete, professionally engineered prompt series to build the LUNA voice assistant (100,000+ lines of code).

## 📚 Prompt Files

| File | Focus | Est. Lines | Key Technologies |
|------|-------|-----------|------------------|
| **[prompt-part1-foundation.md](./prompt-part1-foundation.md)** | Project structure, config, errors, utils | ~10,000 | tokio, anyhow, serde, tracing |
| **[prompt-part2-audio.md](./prompt-part2-audio.md)** | Audio capture, wake word, speech-to-text | ~8,000 | cpal, whisper-rs, porcupine |
| **[prompt-part3-brain.md](./prompt-part3-brain.md)** | Command parsing, intent classification | ~12,000 | regex, strsim |
| **[prompt-part4-actions.md](./prompt-part4-actions.md)** | App control, file ops, system control | ~15,000 | open, notify-rust, clipboard |
| **[prompt-part5-os-integration.md](./prompt-part5-os-integration.md)** | Windows/Linux/macOS platform code | ~20,000 | windows, nix, dbus |
| **[prompt-part6-tts-context.md](./prompt-part6-tts-context.md)** | Text-to-speech, conversation memory | ~8,000 | tts |
| **[prompt-part7-integration.md](./prompt-part7-integration.md)** | Main loop, testing, optimization | ~10,000 | All above |
| **[prompt-summary.md](./prompt-summary.md)** | Usage guide and overview | - | Documentation |

**Total Code:** ~83,000 lines + tests/docs = **100,000+ lines**

## 🚀 Quick Start

### Option 1: Sequential Development
```bash
1. Implement Part 1 → Test → Commit
2. Implement Part 2 → Test → Commit
3. Continue through Part 7
```

### Option 2: AI-Assisted Development
```bash
# Feed each prompt to your AI assistant in order
cat prompt-part1-foundation.md | ai-assistant generate
cargo test
git commit -m "Part 1 complete"

cat prompt-part2-audio.md | ai-assistant generate
cargo test
git commit -m "Part 2 complete"

# Continue...
```

### Option 3: Team Development
- **Developer A:** Parts 1-2 (Foundation + Audio)
- **Developer B:** Part 3 (Brain)
- **Developer C:** Parts 4-5 (Actions + OS)
- **Developer D:** Parts 6-7 (TTS + Integration)

## 📋 Prerequisites

Before starting, ensure you have:
- ✅ Rust toolchain (1.70+)
- ✅ System audio libraries
- ✅ Whisper model downloaded
- ✅ Porcupine wake word model

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Linux: Install audio deps
sudo apt install libasound2-dev portaudio19-dev

# macOS: Install audio deps
brew install portaudio

# Download Whisper model
mkdir -p models
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin -O models/whisper-base.bin
```

## 🎯 What Each Part Delivers

### Part 1: Foundation ✨
- Complete project structure (all directories/files)
- Configuration system (TOML-based)
- Error handling framework
- Logging infrastructure
- Core utilities (string matching, path helpers)
- Database schemas

### Part 2: Audio System 🎤
- Microphone capture (continuous, low-power)
- Wake word detection ("Hey Luna")
- Speech-to-text (Whisper integration)
- Audio preprocessing (noise reduction, VAD)

### Part 3: Brain System 🧠
- Command parser (50+ patterns)
- Intent classifier (95%+ accuracy)
- Entity extractor (apps, files, numbers, dates)
- Task planner (multi-step execution)

### Part 4: Actions System 🎬
- App launcher (find and launch any app)
- File search (by name/content)
- System control (volume, brightness, power)
- Window management (move, resize, arrange)
- Media control (play/pause, next/prev)
- Clipboard operations

### Part 5: OS Integration 💻
- Windows support (Win32 APIs, registry)
- Linux support (D-Bus, .desktop files)
- macOS support (Cocoa, LaunchServices)
- App discovery across all platforms

### Part 6: TTS & Context 🗣️
- Text-to-speech (natural voice)
- Conversation memory (context awareness)
- State tracking (active apps, files)

### Part 7: Integration 🔗
- Main event loop
- Component integration
- Performance optimization
- Comprehensive testing
- Documentation

## 📊 Key Metrics

**Performance Targets:**
- Response time: < 1 second
- Wake word detection: < 100ms
- Speech recognition: < 500ms
- CPU (idle): < 5%
- Memory (idle): < 200MB

**Quality Targets:**
- Intent accuracy: 95%+
- Uptime: 24+ hours stable
- Test coverage: 80%+
- Zero panics in production

## 🔧 Prompt Engineering Features

Each prompt includes:
- ✅ **Clear role definition** - Sets expert context
- ✅ **Detailed specifications** - Exact requirements
- ✅ **Code examples** - Concrete implementations
- ✅ **Success criteria** - Testable outcomes
- ✅ **Error handling** - Production-ready patterns
- ✅ **Performance targets** - Measurable goals
- ✅ **Testing requirements** - Quality assurance

## 📖 Documentation Structure

```
docs/
├── brief.md                      # Original project vision
├── prompt-part1-foundation.md    # Part 1 prompt
├── prompt-part2-audio.md         # Part 2 prompt
├── prompt-part3-brain.md         # Part 3 prompt
├── prompt-part4-actions.md       # Part 4 prompt
├── prompt-part5-os-integration.md # Part 5 prompt
├── prompt-part6-tts-context.md   # Part 6 prompt
├── prompt-part7-integration.md   # Part 7 prompt
├── prompt-summary.md             # Comprehensive guide
└── README-PROMPTS.md             # This file
```

## 🎓 Learning Path

If you're new to:
- **Rust async:** Start with Part 1 (tokio basics)
- **Audio DSP:** Focus on Part 2 (signal processing)
- **NLP:** Deep dive Part 3 (parsing, classification)
- **System APIs:** Study Part 5 (OS integration)

## 🤝 Contributing

After building LUNA, you can extend it:
1. Add new command patterns (Part 3)
2. Implement new actions (Part 4)
3. Add platform support (Part 5)
4. Improve accuracy (Parts 2-3)
5. Create plugins (architecture supports it)

## 📞 Support

For issues with:
- **Prompt clarity:** Review `prompt-summary.md`
- **Dependencies:** Check each part's dependency list
- **Architecture:** See `brief.md` for design rationale
- **Implementation:** Each prompt has code examples

## 🚧 Development Checklist

- [ ] Part 1: Foundation complete & tested
- [ ] Part 2: Audio system working
- [ ] Part 3: Brain can parse commands
- [ ] Part 4: Actions execute correctly
- [ ] Part 5: OS integration functional
- [ ] Part 6: TTS and context working
- [ ] Part 7: Full integration complete
- [ ] All tests passing
- [ ] Performance targets met
- [ ] Documentation updated

## 🎉 Final Result

You will have built:
- ✅ Production-ready voice assistant
- ✅ 100,000+ lines of Rust code
- ✅ Cross-platform support
- ✅ Privacy-focused (offline)
- ✅ Sub-second response times
- ✅ Extensible architecture
- ✅ Comprehensive test suite

**Let's build the future of voice control! 🌙**
