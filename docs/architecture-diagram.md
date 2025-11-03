# LUNA Architecture Diagrams

## Current Architecture (Phases 1-3)

```
┌─────────────────────────────────────────────────────────────────┐
│                        LUNA Application                         │
└─────────────────────────────────────────────────────────────────┘
                                 │
                ┌────────────────┼────────────────┐
                ▼                ▼                ▼
        ┌──────────────┐  ┌──────────┐  ┌──────────────┐
        │    Audio     │  │  Brain   │  │   Actions    │
        │   System     │  │  System  │  │  (Stubs)     │
        └──────────────┘  └──────────┘  └──────────────┘
                │                │
        ┌───────┴────────┐      │
        ▼                ▼      ▼
    ┌────────┐      ┌────────┐ ┌────────┐
    │Capture │      │Wake    │ │Command │
    │        │      │Word    │ │Parser  │
    └────────┘      └────────┘ └────────┘
        │                │          │
        ▼                ▼          ▼
    ┌────────┐      ┌────────┐ ┌────────┐
    │Process │      │Speech  │ │Intent  │
    │        │      │To Text │ │Classify│
    └────────┘      └────────┘ └────────┘
                                    │
                                    ▼
                               ┌────────┐
                               │Task    │
                               │Planner │
                               └────────┘

Issues:
✗ Direct coupling between components
✗ No event system
✗ Hard to test
✗ Sequential processing
```

## Proposed Architecture (Event-Driven)

```
┌─────────────────────────────────────────────────────────────────┐
│                      LUNA Runtime Core                          │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                      Event Bus                            │  │
│  │  • AudioCaptured    • WakeWordDetected                    │  │
│  │  • CommandTranscribed • CommandParsed                     │  │
│  │  • TaskPlanned      • ActionExecuted                      │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
         ▲              ▲              ▲              ▲
         │              │              │              │
         │ publish      │ subscribe    │ publish      │ subscribe
         │              │              │              │
    ┌────┴────┐    ┌───┴────┐    ┌───┴────┐    ┌───┴────┐
    │ Audio   │    │ Brain  │    │Actions │    │ TTS    │
    │ System  │    │ System │    │Executor│    │ System │
    └─────────┘    └────────┘    └────────┘    └────────┘
         │              │              │              │
    implements     implements     implements     implements
         │              │              │              │
    ┌────▼────────┐ ┌──▼──────────┐ ┌▼──────────┐ ┌▼──────────┐
    │AudioCapture │ │CommandParser│ │AppLauncher│ │Synthesizer│
    │  Trait      │ │   Trait     │ │  Trait    │ │   Trait   │
    └─────────────┘ └─────────────┘ └───────────┘ └───────────┘

Benefits:
✓ Loose coupling via events
✓ Easy to test with mocks
✓ Pluggable components
✓ Parallel event processing
```

## Data Flow: Current vs Proposed

### Current (Sequential)
```
User speaks
    ↓
Microphone captures (blocking)
    ↓
Ring buffer stores
    ↓
Wake word detection (polling 100ms)
    ↓
Speech-to-text transcription (blocking)
    ↓
Command parsing
    ↓
Intent classification
    ↓
Task planning
    ↓
Action execution (Phase 4)

Total latency: 100-500ms
```

### Proposed (Event-Driven + Parallel)
```
User speaks
    ↓
Microphone captures
    ├─────────────────────┬─────────────────────┐
    ▼                     ▼                     ▼
Ring buffer      Wake word (streaming)   VAD analysis
    │                     │                     │
    │                     ├─> WakeWordDetected  │
    │                     │    event            │
    │                     │         │           │
    │                     │         ▼           │
    │                     │    Start STT   ←────┘
    │                     │         │
    ▼                     ▼         ▼
Audio buffer ─────> Transcription
                          │
                          ├─> CommandTranscribed event
                          │         │
                          │         ├──────────┬──────────┐
                          │         ▼          ▼          ▼
                          │    Parse      Extract    Classify
                          │         │          │          │
                          │         └────┬─────┴─────┐    │
                          │              ▼           │    │
                          │         Merge results    │    │
                          │              │           │    │
                          │              ▼           ▼    ▼
                          └────────> Task Planning
                                         │
                                         ├─> TaskPlanned event
                                         │         │
                                         │         ▼
                                         │    Action Execution

Total latency: 50-200ms (2-3x faster)
```

## Component Communication: Proposed

### Trait-Based Interfaces
```rust
┌─────────────────────────────────────────────────────┐
│              Component Traits                       │
├─────────────────────────────────────────────────────┤
│                                                     │
│  AudioCaptureInterface                              │
│    • start() -> Result<()>                          │
│    • stop() -> Result<()>                           │
│    • get_stream() -> Receiver<Vec<f32>>             │
│                                                     │
│  WakeWordDetectorInterface                          │
│    • detect(audio: &[f32]) -> Result<Option<usize>> │
│    • set_sensitivity(value: f32)                    │
│                                                     │
│  SpeechToTextInterface                              │
│    • transcribe(audio: &[f32]) -> Result<String>    │
│    • set_language(lang: &str)                       │
│                                                     │
│  CommandParserInterface                             │
│    • parse(text: &str) -> Result<ParsedCommand>     │
│    • add_pattern(pattern: Pattern)                  │
│                                                     │
│  ActionExecutorInterface                            │
│    • execute(action: Action) -> Result<ActionResult>│
│    • can_execute(action: &Action) -> bool           │
└─────────────────────────────────────────────────────┘
         │              │              │
         ▼              ▼              ▼
    ┌─────────┐    ┌─────────┐    ┌─────────┐
    │  Real   │    │  Mock   │    │  Test   │
    │  Impl   │    │  Impl   │    │  Impl   │
    └─────────┘    └─────────┘    └─────────┘

Benefits: Easy swapping, testing, and extension
```

## Error Handling Flow

### Current
```
Component A
    │
    ├─> Error occurs
    │       │
    │       └─> Propagate to caller
    │               │
    │               └─> Log and return
    │                       │
    └───────────────────────┘ (stops here)
```

### Proposed
```
Component A
    │
    ├─> Error occurs
    │       │
    │       ├─> Check if recoverable
    │       │       │
    │       │       ├─ Yes ─> Retry with backoff
    │       │       │              │
    │       │       │              ├─ Success ─> Continue
    │       │       │              │
    │       │       │              └─ Fail ─> Try fallback
    │       │       │                            │
    │       │       │                            ├─ Success ─> Continue
    │       │       │                            │
    │       │       │                            └─ Fail ─> Emit ErrorEvent
    │       │       │
    │       │       └─ No ─> Emit ErrorEvent
    │       │                      │
    │       └─> Publish to event bus
    │                   │
    │                   ├─> Logger subscribes
    │                   ├─> Metrics subscribes
    │                   ├─> TTS subscribes (user notification)
    │                   └─> Recovery handler subscribes
    │
    └─> Component continues (non-fatal errors)
```

## Testing Architecture

### Current
```
┌─────────────────────┐
│    Unit Test        │
│                     │
│  creates actual     │
│  component          │
│      │              │
│      ▼              │
│  [RealComponent]    │
│      │              │
│      ├─> Hard to    │
│      │   mock deps  │
│      │              │
│      └─> Integration│
│          test only  │
└─────────────────────┘
```

### Proposed
```
┌─────────────────────┐     ┌─────────────────────┐
│    Unit Test        │     │  Integration Test   │
│                     │     │                     │
│  creates mock       │     │  creates real       │
│  via trait          │     │  components         │
│      │              │     │      │              │
│      ▼              │     │      ▼              │
│  [MockComponent]    │     │  [RealComponents]   │
│      │              │     │      │              │
│      ├─> Fast       │     │      ├─> Full flow │
│      ├─> Isolated   │     │      ├─> Real deps │
│      └─> Controlled │     │      └─> End-to-end│
└─────────────────────┘     └─────────────────────┘
```

## Performance Monitoring

```
┌─────────────────────────────────────────────────────┐
│                  Metrics Collector                  │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Command Processing Pipeline:                       │
│  ┌─────────┬─────────┬─────────┬─────────┐         │
│  │ Capture │  STT    │ Parse   │ Execute │         │
│  │  50ms   │ 100ms   │  5ms    │  20ms   │         │
│  └─────────┴─────────┴─────────┴─────────┘         │
│            Total: 175ms                             │
│                                                     │
│  Resource Usage:                                    │
│  • CPU: 15%  [▓▓▓░░░░░░░]                           │
│  • Memory: 120MB                                    │
│  • Audio buffer: 2.5MB                              │
│                                                     │
│  Error Rates:                                       │
│  • Wake word false positive: 0.5%                   │
│  • STT accuracy: 95%                                │
│  • Command success: 92%                             │
│                                                     │
│  Export to:                                         │
│  • Prometheus (metrics endpoint)                    │
│  • Log files (JSON format)                          │
│  • Console (real-time)                              │
└─────────────────────────────────────────────────────┘
```

## Plugin System Architecture

```
┌─────────────────────────────────────────────────────┐
│                  LUNA Core Runtime                  │
│                                                     │
│  ┌──────────────────────────────────────────────┐  │
│  │            Plugin Manager                     │  │
│  │  • load_plugin(path)                          │  │
│  │  • register_handler(event_type, handler)     │  │
│  │  • unload_plugin(name)                        │  │
│  └──────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│   Custom     │ │   Spotify    │ │   Notion     │
│   Commands   │ │   Control    │ │   Notes      │
│   Plugin     │ │   Plugin     │ │   Plugin     │
├──────────────┤ ├──────────────┤ ├──────────────┤
│ • on_command │ │ • on_command │ │ • on_command │
│ • custom     │ │ • play/pause │ │ • create_note│
│   patterns   │ │ • next/prev  │ │ • search     │
└──────────────┘ └──────────────┘ └──────────────┘

Each plugin:
• Implements LunaPlugin trait
• Subscribes to events
• Can add custom commands
• Isolated from core
```

## Deployment Architecture

```
┌───────────────────────────────────────────────────┐
│                User's Computer                    │
│                                                   │
│  ┌──────────────────────────────────────────────┐ │
│  │         LUNA Process                         │ │
│  │                                              │ │
│  │  ┌───────────┐  ┌───────────┐  ┌────────┐    │ │
│  │  │   Audio   │  │   Brain   │  │ Actions│    │ │
│  │  │  Thread   │  │  Thread   │  │ Thread │    │ │
│  │  └───────────┘  └───────────┘  └────────┘    │ │
│  │        │             │              │        │ │
│  │        └─────────────┼──────────────┘        │ │
│  │                      │                       │ │
│  │              ┌───────▼────────┐              │ │
│  │              │   Event Bus    │              │ │
│  │              └────────────────┘              │ │
│  │                      │                       │ │
│  │              ┌───────▼────────┐              │ │
│  │              │  Local SQLite  │              │ │
│  │              └────────────────┘              │ │
│  └──────────────────────────────────────────────┘ │
│                                                   │
│  Local Files:                                     │
│  • ~/.config/luna/config.toml                     │
│  • ~/.local/share/luna/apps.db                    │
│  • ~/.local/share/luna/files.db                   │
│  • ~/.cache/luna/whisper-base.bin                 │
└───────────────────────────────────────────────────┘

All processing runs locally:
✓ No internet required
✓ Privacy preserved
✓ Fast response times
✓ Works offline
```
