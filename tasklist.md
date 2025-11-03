# LUNA Project Tasklist: Stubs, Simulations, and Placeholders

This file tracks all known stubbed functions, simulated behavior, and placeholders identified in `src/`. Each item is an actionable task with references to the exact files and symbols.

## Critical stubs (panic risk if invoked)

- [x] **COMPLETED** `create_directory(path: &Path)` in `src/actions/file_operations.rs`
  - ✅ Replaced `todo!()` with full implementation.
  - ✅ Creates directory with idempotency using `fs::create_dir_all`.
  - ✅ Proper error handling for non-directories and permissions.

- [x] **COMPLETED** `move_file(from: &Path, to: &Path)` in `src/actions/file_operations.rs`
  - ✅ Replaced `todo!()` with full implementation.
  - ✅ Supports files and directories with `fs::rename` and cross-device fallback.
  - ✅ Uses copy+delete fallback for cross-device moves with recursive directory support.

- [x] **COMPLETED** `create_reminder(message: &str, duration: Duration)` in `src/actions/reminders.rs`
  - ✅ Replaced `todo!()` with tokio timer implementation.
  - ✅ Emits events via `EventBus` and shows OS notifications.
  - ✅ Added tests for validation, positive durations, and event triggering.

- [x] **COMPLETED** `AudioCaptureInterface::get_ring_buffer()` in `src/audio/traits.rs`
  - ✅ Fixed by adding default trait implementation with deprecation notice.
  - ✅ Removed `unimplemented!()` from `capture_new.rs`.
  - ✅ New code should use concrete type's method that returns `&LockFreeRingBuffer`.

## Simulated behavior to replace with production

- [ ] Implement Whisper-based transcription in `src/audio/speech_to_text.rs`
  - Add a real path in `SpeechToText::transcribe(&self, ...)` when running non-simulated.
  - Load model from `BrainConfig.whisper_model_path`.
  - Consider using `whisper-rs` (behind a cargo feature); handle model load failures gracefully.

- [ ] Wire STT selection from config
  - `src/config.rs`: `BrainConfig.stt_engine` defaults to `"simulate"`.
  - Update `SpeechToText` (constructor or factory) to branch on `stt_engine` instead of only file existence.
  - Ensure `audio/mod.rs::ProductionAudioSystem::create(...)` passes the necessary info.

- [ ] Replace main loop command simulation in `src/main.rs`
  - Replace the simulated command counter with real pipeline:
    - Use `AudioCapture.record_command(...)` to capture utterance.
    - Pass audio to `stt.transcribe(&audio)`.
    - Handle empty transcripts and errors.
  - Keep logs; remove `// TODO: Record and transcribe command` and the `simulated_text` path.

- [ ] Keep and document mocks in `src/audio/mocks.rs`
  - Ensure mocks remain usable for unit tests.
  - Add brief README note or doc comments clarifying their testing-only intent.

## Placeholders and CLI completeness

- [x] **COMPLETED** `run_index(...)` in `src/cli.rs`
  - ✅ App index: rebuilds `AppDatabase` via `discover_applications()` and persists to disk.
  - ✅ File index: rebuilds `FileIndex` across configured search paths with exclusions.
  - ✅ Shows real counts, timing, and file paths. Includes helper functions for recursive indexing.

- [x] **COMPLETED** `run_events(...)` in `src/cli.rs`
  - ✅ Connects to `EventBus` and streams events with filtering.
  - ✅ Supports tail mode (`--tail`) with Ctrl+C handling and `--limit` for event count.
  - ✅ Displays events with timestamps and formatted output.

- [x] **COMPLETED** `run_metrics(...)` export in `src/cli.rs`
  - ✅ Exports metrics to JSON or CSV format based on file extension.
  - ✅ Includes all key metrics (commands, latencies, success rate).
  - ✅ Added missing methods to `Metrics` struct for average latency calculations.

- [ ] Implement actual window placement in `src/os/multi_monitor.rs::apply_layout(...)`
  - Replace the `// TODO: Implement actual window placement` loop body.
  - Add per-OS backends (e.g., Linux: `wmctrl`/`xdotool` or a Rust crate; Windows/macOS via platform APIs).
  - Emit `LunaEvent` updates as windows are placed.

## Audio quality and VAD improvements

- [ ] Integrate production noise suppression in `src/audio/dsp.rs`
  - Replace simple noise gate with WebRTC NS or RNNoise (behind a cargo feature).
  - Add config toggles to enable/disable advanced NS; fallback to current gate if unavailable.

- [ ] Implement Silero VAD engine in `src/audio/vad.rs`
  - Fill the `// TODO: Implement Silero VAD` branch.
  - Keep `webrtc-audio` feature gating; when disabled, continue to fallback to RMS.

## Capture architecture consolidation

- [ ] Decide and consolidate: `src/audio/capture.rs` vs `src/audio/capture_new.rs`
  - If adopting `capture_new` (lock-free design), update the trait `AudioCaptureInterface` to match and remove the `unimplemented!` path.
  - Update re-exports in `src/audio/mod.rs` to use the chosen implementation consistently.
  - Ensure `AudioSystem` and CLI tools (`run_audio_*`) work with the final capture.

## Configuration wiring and validation

- [ ] Enforce valid values for `brain.stt_engine` in `src/config.rs`
  - Accept only `"simulate"` or `"whisper"`; warn or error otherwise.
  - Align logs in `main.rs` with actual engine selection and model availability.

## Testing and verification

- [ ] Add unit tests for `file_operations` and `reminders`
  - Cover success and failure paths.

- [ ] Add integration test for the end-to-end audio→wake→record→STT pipeline
  - Use mocks to avoid device dependency; verify `AudioSystem` interacts with `EventBus` and executor.

- [ ] Add CLI tests (where feasible)
  - Validate argument parsing already covered; add tests for `run_index`, `run_events`, `run_metrics` behaviors behind feature flags or using temporary dirs.

---

Generated from code scan of `src/` modules including `audio/`, `actions/`, `cli.rs`, `os/`, and `main.rs`.
