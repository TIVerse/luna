# LUNA Implementation Prompt

You are working on the LUNA Voice Assistant (Rust 2021). Implement the features below in small, well-scoped commits with tests and docs updates. Follow the existing architecture and conventions. Keep code idiomatic, documented, and platform-safe (feature gate where needed).

Repository root: this directory.

Key modules to reference:
- Audio: `src/audio/` (`mod.rs`, `capture.rs`, `speech_to_text.rs`, `processor.rs`, `vad.rs`, `wake_word.rs`, `ring_buffer.rs`, `traits.rs`)
- Brain: `src/brain/` (`mod.rs`, `command_parser.rs`, `intent_classifier.rs`, `task_planner.rs`, `ranking.rs`, `multi_intent.rs`, `contextualizer.rs`, `grammar.rs`, `types.rs`)
- Actions/OS: `src/actions/` (`executor.rs`, `app_launcher.rs`, `file_search.rs`, `system_control.rs`, `media_control.rs`, `reminders.rs`, `capabilities.rs`) and `src/os/`
- Runtime/infra: `src/main.rs`, `src/events.rs`, `src/metrics.rs`, `src/context/memory.rs`, `src/config.rs`, `src/cli.rs`

Deliver the P0 scope as mandatory; P1/P2 as optional if time permits (behind features where appropriate).

---

## P0: Mandatory Features

### 1) Real STT wiring in the runtime loop
- Add to `src/audio/mod.rs`:
  - A new async method on `AudioSystem<...>`:
    - `pub async fn listen_and_transcribe(&mut self, max_secs: u64) -> crate::error::Result<String>`
    - Steps:
      1. Call `self.capture.record_command(max_secs)` to collect audio.
      2. Run basic preprocessing with `self.processor` (noise gate + normalize).
      3. Call `self.stt.transcribe(&processed)`.
      4. If `self.event_bus` is Some, publish `LunaEvent::CommandTranscribed { text, confidence: 1.0 }` (use best effort confidence; STT currently simulated).
      5. Return the text.
- Update `src/main.rs`:
  - After wake word detection, replace the simulated `command {n}` path with:
    - `let text = audio_system.listen_and_transcribe(config.audio.recording_timeout_secs).await?;`
    - `if text.trim().is_empty() { continue; }`
  - Add `MetricTimer` instrumentation (see §6) for STT.

Acceptance:
- Running `cargo run --release` allows speaking after wake word and logs the recognized text instead of the simulated counter.


### 2) Multi-intent integration (parallel + temporal "wait")
- In `src/brain/mod.rs`:
  - Integrate `MultiIntentParser` during `process_async(text)`:
    - If `MultiIntentParser::is_multi_intent(text)`:
      - Use the parser to split segments and extract any `TemporalModifier`.
      - For each cleaned segment, run the existing pipeline (parse → classify).
      - Build a combined `TaskPlan`:
        - Sequential if `CoordinationType::Sequential`.
        - Add a `Wait` step (ActionType::Wait) before a segment when `TemporalModifier` has a duration (convert to seconds string under `params["duration"]`).
        - For `CoordinationType::Parallel`, put the segment steps into a new `parallel_groups` entry (retain existing step numbers and dependencies).
      - Cache the final plan using the same cache strategy.
- In `src/brain/task_planner.rs`:
  - Add a helper like `pub fn plan_multi(&self, items: Vec<(ClassificationResult, Option<std::time::Duration>)>, coordination: CoordinationType) -> TaskPlan` (or equivalent) that merges per-intent steps into a single plan and fills `parallel_groups` accordingly.
  - Ensure `Wait` steps are created with `ActionType::Wait` and `params["duration"]` in seconds.
- In `src/actions/executor.rs`:
  - Already supports `ActionType::Wait`; no changes required beyond using it.

Acceptance:
- `brain/multi_intent.rs` tests remain passing.
- New tests cover:
  - Parallel commands: "open chrome and play music" produce 2 steps in a parallel group.
  - Temporal commands: "mute in 10 minutes" insert a Wait step with duration before the mute/system-control action.


### 3) Use discovered apps to boost classification
- In `src/main.rs` after discovering apps into `app_db_temp` and before using `Brain`:
  - For each `Application` discovered, call `brain.add_known_app(app.name.clone())` (or add a `add_known_apps(Vec<String>)` to avoid N calls).

Acceptance:
- Commands like "open Visual Studio Code" gain confidence (log shows boosted confidence).


### 4) Capability detection at startup
- In `src/main.rs`, after EventBus initialization:
  - Run: `let caps = luna::actions::CapabilityDetector::new().with_event_bus(std::sync::Arc::clone(&event_bus)).detect_all().await;`
  - Log results (already printed by detector) and proceed regardless of outcome.

Acceptance:
- On Linux, missing tools (e.g., `playerctl`, `brightnessctl`) are reported with install hints.


### 5) Declarative grammar YAML + reload CLI
- Add a new file `config/brain_patterns.yaml` with initial patterns (example below).
- Ensure `Brain::try_load_grammar()` (already in `brain/mod.rs`) loads this file if present.
- Extend `src/cli.rs` with subcommand group `Brain`:
  - `brain parse "<text>"` → prints `ParsedCommand`.
  - `brain classify "<text>"` → prints `ClassificationResult`.
  - `brain plan "<text>" [--preview]` → builds `TaskPlan`; if `--preview`, call `TaskExecutor::preview_plan(plan)` and print the dry-run output.
  - `brain grammar-reload` → calls `brain.reload_grammar()` and reports `GrammarReloaded` event.

Acceptance:
- Running `luna brain grammar-reload` logs a successful reload if the YAML exists.
- Other brain CLI tools successfully parse/classify/plan sample inputs.

Example `config/brain_patterns.yaml` starter:
```yaml
version: "1.0"
intents:
  - name: LaunchApp
    priority: 100
    patterns:
      - pattern: "^(?:open|launch|start|run)\\s+(.+)$"
        entities:
          app_name: "$1"
    examples:
      - "open chrome"
  - name: CloseApp
    priority: 100
    patterns:
      - pattern: "^(?:close|quit|exit|kill)\\s+(.+)$"
        entities:
          app_name: "$1"
    examples:
      - "close firefox"
  - name: SearchWeb
    priority: 90
    patterns:
      - pattern: "^(?:search\\s+(?:for|the\\s+web\\s+for)|google)\\s+(.+)$"
        entities:
          query: "$1"
    examples:
      - "search for rust tutorials"
```


### 6) Metrics instrumentation (STT, Parsing, Execution, Total)
- Use `MetricTimer` in `src/main.rs`:
  - Around STT (`MetricPhase::SpeechToText`).
  - Around parsing/brain processing; `MetricPhase::Parsing` is already used in places—ensure it’s consistently applied.
  - Around execution (`MetricPhase::Execution`).
  - Optionally total (`MetricPhase::Total`).

Acceptance:
- Metrics summary prints non-zero average latencies for phases after at least one command.


### 7) Conversation memory persistence
- In `src/context/memory.rs`:
  - Derive `Serialize, Deserialize` for `ConversationEntry` and, if convenient, a serializable wrapper for the history (Vec<ConversationEntry>).
  - Implement:
    - `pub async fn save_to_disk<P: AsRef<std::path::Path>>(&self, path: P) -> crate::error::Result<()>` (JSON with pretty formatting).
    - `pub async fn load_from_disk<P: AsRef<std::path::Path>>(path: P) -> crate::error::Result<Self>` (fallback to empty if missing).
- In `src/main.rs`:
  - On startup, attempt to load from `data_dir/conversation.json`.
  - On graceful shutdown, save to the same path.

Acceptance:
- Conversation persists across runs; `Recent commands` at shutdown reflect previous session after restart.


### 8) Simple clarification loop (runtime)
- In `src/main.rs`, after obtaining a `TaskPlan` and before execution:
  - If `plan.classification.confidence < config.brain.confidence_threshold`:
    - If TTS available, ask: "I’m not confident I understood. Could you clarify?"
    - Call `audio_system.listen_and_transcribe( max 5–8s )`.
    - If non-empty, re-run `brain.process_async(&clarification_text)` and use the new plan.
    - Optionally publish `LunaEvent::ClarificationAnswered { original_command, clarification, resolved_command }`.

Acceptance:
- Low confidence path prompts for clarification and can succeed on a second utterance.


---

## P1: High-Impact Optional Enhancements

- **Window management MVP (Linux)**
  - In `src/os/window_manager.rs` implement basic snap/resize for X11 behind the `x11` feature (Cargo already has `features.x11 = ["x11rb"]`).
  - Provide no-ops on other platforms.

- **Reminders at specific times**
  - Extend `entity_extractor.rs::extract_time_of_day()` usage in parser/classifier to allow "at 3pm" style.
  - In planning, if time-of-day is in the future, schedule via `actions/reminders.rs` using a `tokio::spawn` with computed delay.

- **Brain ranking weights from config**
  - Add ranking weights fields to `BrainConfig` (optional) and thread them into `RankingScorer::with_weights`.

- **Audit JSONL for events**
  - Add an `EventBus` subscriber in `main.rs` when `config.system.log_level` is debug or a new config toggle, writing JSONL to `logs/events.jsonl`.

- **CLI: audio tools**
  - Add `audio transcribe --file <wav>` using existing STT engine.
  - Add `audio calibrate` to suggest `silence_threshold` from a short capture.


## P2: Longer-Term Options (Feature-gated)

- **Linux MPRIS via zbus** for `MediaControl` (replace `playerctl` calls when feature `mpris` is enabled; keep shell fallback otherwise).
- **SQLite backends** for `AppDatabase` and `FileIndex` under the existing `sqlite` feature.
- **Incremental indexer** using `notify` to watch paths and update `FileIndex` incrementally; CLI `index watch`.
- **Local API (HTTP or gRPC)** to submit commands and subscribe to events.
- **Multilingual** via `whatlang` feature to route STT/TTS.

---

## Testing & Validation

- Unit tests for new functions (`audio/mod.rs::listen_and_transcribe`, `task_planner.rs::plan_multi`, memory persistence).
- CLI e2e: `luna brain parse`, `luna brain classify`, `luna brain plan --preview`, `luna brain grammar-reload`.
- Runtime manual test:
  1. Start app, say wake word, speak: "open chrome and play music" → see parallel plan & executor dry-run or actual action.
  2. Say: "mute in 10 seconds" → see Wait then mute.
  3. Say a vague command → clarification prompt and second try.
- Metrics: verify non-zero STT/parsing/execution averages.
- Conversation persistence: restart and see previous entries in shutdown summary.

---

## Non-Functional Requirements
- Keep existing public APIs stable unless necessary; document any breaking changes in code comments and `README.md`.
- Respect platform differences with `cfg(target_os)` and feature flags (`x11`, `prometheus`, `sqlite`, `mpris` if added).
- Logging: use `tracing` consistently; avoid noisy logs in hot paths.
- Performance: avoid blocking the audio callback; follow the existing async patterns.

---

## Documentation Updates
- `README.md` "Current Status" → update Phase 2/3 progress after P0.
- Add a short section on grammar YAML (`config/brain_patterns.yaml`).
- Update `docs/` if there are relevant guides to touch (e.g., audio/brain parts).

---

## Notes & Hints
- Use existing types: `LunaEvent::CommandTranscribed`, `ActionType::Wait`, `MetricTimer`, `ConversationMemory` APIs.
- `IntentType` already has `Serialize, Deserialize`; safe to serialize conversation entries.
- Keep multi-intent merging logic simple: topologically concatenate steps; only use `parallel_groups` when coordination is `Parallel`.
- For temporal modifiers, only support durations now ("after", "in"); ignore absolute times unless covered by P1.

Good luck! Make incremental commits with clear messages, and keep tests green (`cargo test`).
