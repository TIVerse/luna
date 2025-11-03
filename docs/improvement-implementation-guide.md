# Implementation Guide: Architecture Improvements

This guide provides step-by-step instructions for implementing the high-priority architecture improvements.

---

## 🎯 Phase 3.5: Pre-Phase 4 Improvements

### Improvement 1: Trait-Based Dependency Injection (2 days)

#### Step 1: Define Core Traits

**Create** `src/audio/traits.rs`:
```rust
use crate::error::Result;
use async_channel::{Receiver, Sender};

/// Audio capture interface
#[async_trait::async_trait]
pub trait AudioCaptureInterface: Send + Sync {
    /// Start capturing audio
    async fn start(&mut self) -> Result<()>;
    
    /// Stop capturing audio
    async fn stop(&mut self) -> Result<()>;
    
    /// Get audio stream receiver
    fn get_audio_stream(&self) -> Receiver<Vec<f32>>;
    
    /// Check if currently capturing
    fn is_active(&self) -> bool;
}

/// Wake word detection interface
#[async_trait::async_trait]
pub trait WakeWordDetectorInterface: Send + Sync {
    /// Detect wake word in audio buffer
    async fn detect(&self, audio: &[f32]) -> Result<Option<usize>>;
    
    /// Set detection sensitivity
    fn set_sensitivity(&mut self, sensitivity: f32);
}

/// Speech-to-text interface
#[async_trait::async_trait]
pub trait SpeechToTextInterface: Send + Sync {
    /// Transcribe audio to text
    async fn transcribe(&self, audio: &[f32]) -> Result<String>;
    
    /// Check if running in simulated mode
    fn is_simulated(&self) -> bool;
}
```

**Add to** `Cargo.toml`:
```toml
async-trait = "0.1"
```

#### Step 2: Implement Traits for Existing Components

**Update** `src/audio/capture.rs`:
```rust
use super::traits::AudioCaptureInterface;

#[async_trait::async_trait]
impl AudioCaptureInterface for AudioCapture {
    async fn start(&mut self) -> Result<()> {
        // Existing implementation
        self.start_internal()
    }
    
    async fn stop(&mut self) -> Result<()> {
        // Existing implementation  
        self.stop_internal()
    }
    
    fn get_audio_stream(&self) -> Receiver<Vec<f32>> {
        self.audio_rx.clone()
    }
    
    fn is_active(&self) -> bool {
        self.stream.is_some()
    }
}
```

#### Step 3: Create Mock Implementations

**Create** `src/audio/mocks.rs`:
```rust
use super::traits::*;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MockAudioCapture {
    samples: Arc<Mutex<Vec<Vec<f32>>>>,
    active: Arc<Mutex<bool>>,
    rx: Receiver<Vec<f32>>,
    tx: Sender<Vec<f32>>,
}

impl MockAudioCapture {
    pub fn new() -> Self {
        let (tx, rx) = async_channel::bounded(100);
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            active: Arc::new(Mutex::new(false)),
            rx,
            tx,
        }
    }
    
    /// Add mock audio data
    pub async fn add_samples(&self, samples: Vec<f32>) {
        self.samples.lock().await.push(samples);
    }
    
    /// Simulate audio capture
    pub async fn simulate_capture(&self) {
        let samples = self.samples.lock().await;
        for sample in samples.iter() {
            let _ = self.tx.send(sample.clone()).await;
        }
    }
}

#[async_trait::async_trait]
impl AudioCaptureInterface for MockAudioCapture {
    async fn start(&mut self) -> Result<()> {
        *self.active.lock().await = true;
        self.simulate_capture().await;
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<()> {
        *self.active.lock().await = false;
        Ok(())
    }
    
    fn get_audio_stream(&self) -> Receiver<Vec<f32>> {
        self.rx.clone()
    }
    
    fn is_active(&self) -> bool {
        // Simplified for mock
        true
    }
}
```

#### Step 4: Refactor AudioSystem to Use Traits

**Update** `src/audio/mod.rs`:
```rust
use crate::audio::traits::*;

pub struct AudioSystem<C, W, S>
where
    C: AudioCaptureInterface,
    W: WakeWordDetectorInterface,
    S: SpeechToTextInterface,
{
    capture: C,
    wake_word: W,
    stt: S,
    processor: AudioProcessor,
}

impl<C, W, S> AudioSystem<C, W, S>
where
    C: AudioCaptureInterface,
    W: WakeWordDetectorInterface,
    S: SpeechToTextInterface,
{
    pub fn new(
        capture: C,
        wake_word: W,
        stt: S,
        processor: AudioProcessor,
    ) -> Self {
        Self {
            capture,
            wake_word,
            stt,
            processor,
        }
    }
    
    // Rest of implementation...
}

// Type alias for production use
pub type ProductionAudioSystem = AudioSystem<
    AudioCapture,
    WakeWordDetector,
    SpeechToText,
>;

// Factory for easy creation
impl ProductionAudioSystem {
    pub async fn create(
        audio_config: &AudioConfig,
        brain_config: &BrainConfig,
    ) -> Result<Self> {
        let capture = AudioCapture::new(audio_config)?;
        let wake_word = WakeWordDetector::new(
            audio_config.wake_words.clone(),
            brain_config.wake_word_sensitivity,
        )?;
        let stt = SpeechToText::new(&brain_config.whisper_model_path)?;
        let processor = AudioProcessor::new(/* ... */);
        
        Ok(Self::new(capture, wake_word, stt, processor))
    }
}
```

#### Step 5: Write Tests with Mocks

**Create** `src/audio/tests.rs`:
```rust
use super::*;
use super::mocks::*;

#[tokio::test]
async fn test_audio_system_with_mocks() {
    // Create mock components
    let mut capture = MockAudioCapture::new();
    let wake_word = MockWakeWordDetector::new();
    let stt = MockSpeechToText::new();
    let processor = AudioProcessor::new(0.01, 1.0);
    
    // Add test data
    capture.add_samples(vec![0.1, 0.2, 0.3]).await;
    
    // Create system with mocks
    let mut system = AudioSystem::new(capture, wake_word, stt, processor);
    
    // Test functionality
    system.start_listening().unwrap();
    let detected = system.wait_for_wake_word().await.unwrap();
    assert!(detected);
}
```

---

### Improvement 2: Event Bus Implementation (3 days)

#### Step 1: Define Event Types

**Create** `src/events.rs`:
```rust
use crate::brain::{IntentType, ParsedCommand, TaskPlan};
use crate::error::LunaError;
use std::any::Any;

#[derive(Debug, Clone)]
pub enum LunaEvent {
    /// Raw audio captured from microphone
    AudioCaptured {
        samples: Vec<f32>,
        timestamp: u64,
    },
    
    /// Wake word detected
    WakeWordDetected {
        keyword: String,
        confidence: f32,
    },
    
    /// Speech transcribed to text
    CommandTranscribed {
        text: String,
        confidence: f32,
    },
    
    /// Command parsed into structured format
    CommandParsed {
        command: ParsedCommand,
    },
    
    /// Task plan created
    TaskPlanned {
        plan: TaskPlan,
    },
    
    /// Action execution started
    ActionStarted {
        action_type: String,
        params: std::collections::HashMap<String, String>,
    },
    
    /// Action execution completed
    ActionCompleted {
        action_type: String,
        success: bool,
        duration_ms: u64,
    },
    
    /// Error occurred
    Error {
        error: LunaError,
        context: String,
    },
}

impl LunaEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::AudioCaptured { .. } => "audio_captured",
            Self::WakeWordDetected { .. } => "wake_word_detected",
            Self::CommandTranscribed { .. } => "command_transcribed",
            Self::CommandParsed { .. } => "command_parsed",
            Self::TaskPlanned { .. } => "task_planned",
            Self::ActionStarted { .. } => "action_started",
            Self::ActionCompleted { .. } => "action_completed",
            Self::Error { .. } => "error",
        }
    }
}
```

#### Step 2: Implement Event Bus

**Add to** `src/events.rs`:
```rust
use async_channel::{Sender, Receiver, unbounded};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type EventHandler = Arc<dyn Fn(&LunaEvent) + Send + Sync>;

pub struct EventBus {
    subscribers: Arc<RwLock<Vec<Subscriber>>>,
    tx: Sender<LunaEvent>,
    rx: Receiver<LunaEvent>,
}

struct Subscriber {
    id: usize,
    event_types: Vec<&'static str>,
    handler: EventHandler,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
            tx,
            rx,
        }
    }
    
    /// Subscribe to specific event types
    pub async fn subscribe<F>(&self, event_types: Vec<&'static str>, handler: F) -> usize
    where
        F: Fn(&LunaEvent) + Send + Sync + 'static,
    {
        let mut subs = self.subscribers.write().await;
        let id = subs.len();
        subs.push(Subscriber {
            id,
            event_types,
            handler: Arc::new(handler),
        });
        id
    }
    
    /// Unsubscribe by ID
    pub async fn unsubscribe(&self, id: usize) {
        let mut subs = self.subscribers.write().await;
        subs.retain(|s| s.id != id);
    }
    
    /// Publish an event
    pub async fn publish(&self, event: LunaEvent) {
        let _ = self.tx.send(event).await;
    }
    
    /// Start event processing loop
    pub async fn start_processing(&self) {
        let rx = self.rx.clone();
        let subscribers = self.subscribers.clone();
        
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let event_type = event.event_type();
                let subs = subscribers.read().await;
                
                for subscriber in subs.iter() {
                    if subscriber.event_types.is_empty() 
                        || subscriber.event_types.contains(&event_type) 
                    {
                        (subscriber.handler)(&event);
                    }
                }
            }
        });
    }
}
```

#### Step 3: Integrate Event Bus

**Update** `src/lib.rs`:
```rust
pub mod events;
pub use events::{EventBus, LunaEvent};
```

**Update** `src/audio/mod.rs`:
```rust
use crate::events::{EventBus, LunaEvent};

impl<C, W, S> AudioSystem<C, W, S> {
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }
    
    pub async fn wait_for_wake_word(&self) -> Result<bool> {
        // ... existing code ...
        
        if detected {
            // Publish event
            if let Some(bus) = &self.event_bus {
                bus.publish(LunaEvent::WakeWordDetected {
                    keyword: "luna".to_string(),
                    confidence: 0.95,
                }).await;
            }
        }
        
        Ok(detected)
    }
}
```

#### Step 4: Add Event Subscribers

**Create** `src/subscribers/logger.rs`:
```rust
use crate::events::{EventBus, LunaEvent};
use tracing::{info, error};

pub async fn setup_logging_subscriber(event_bus: &EventBus) {
    event_bus.subscribe(vec![], |event| {
        match event {
            LunaEvent::WakeWordDetected { keyword, confidence } => {
                info!("🎤 Wake word '{}' detected (confidence: {:.2})", keyword, confidence);
            }
            LunaEvent::CommandTranscribed { text, confidence } => {
                info!("📝 Transcribed: '{}' (confidence: {:.2})", text, confidence);
            }
            LunaEvent::Error { error, context } => {
                error!("❌ Error in {}: {}", context, error);
            }
            _ => {}
        }
    }).await;
}
```

---

### Improvement 3: Configuration Validation (1 day)

**Update** `src/config.rs`:
```rust
impl LunaConfig {
    pub fn load_and_validate<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = Self::load(path)?;
        config.validate()?;
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<()> {
        self.audio.validate()?;
        self.brain.validate()?;
        self.system.validate()?;
        self.paths.validate()?;
        self.performance.validate()?;
        Ok(())
    }
}

impl AudioConfig {
    pub fn validate(&self) -> Result<()> {
        // Sample rate validation
        if !(8000..=48000).contains(&self.sample_rate) {
            return Err(config_error!(
                "Sample rate {} is out of range (8000-48000 Hz)",
                self.sample_rate
            ));
        }
        
        // Channels validation
        if self.channels != 1 && self.channels != 2 {
            return Err(config_error!(
                "Channels must be 1 (mono) or 2 (stereo), got {}",
                self.channels
            ));
        }
        
        // Threshold validation
        if !(0.0..=1.0).contains(&self.silence_threshold) {
            return Err(config_error!(
                "Silence threshold {} must be between 0.0 and 1.0",
                self.silence_threshold
            ));
        }
        
        // Timeout validation
        if self.recording_timeout_secs == 0 || self.recording_timeout_secs > 300 {
            return Err(config_error!(
                "Recording timeout {} is invalid (must be 1-300 seconds)",
                self.recording_timeout_secs
            ));
        }
        
        Ok(())
    }
}

impl BrainConfig {
    pub fn validate(&self) -> Result<()> {
        // Model path check (warning if not found, not error)
        let model_path = Path::new(&self.whisper_model_path);
        if !model_path.exists() {
            tracing::warn!(
                "Whisper model not found at: {}. Will run in simulated mode.",
                self.whisper_model_path
            );
        }
        
        // Confidence threshold
        if !(0.0..=1.0).contains(&self.confidence_threshold) {
            return Err(config_error!(
                "Confidence threshold {} must be between 0.0 and 1.0",
                self.confidence_threshold
            ));
        }
        
        // Wake word sensitivity
        if !(0.0..=1.0).contains(&self.wake_word_sensitivity) {
            return Err(config_error!(
                "Wake word sensitivity {} must be between 0.0 and 1.0",
                self.wake_word_sensitivity
            ));
        }
        
        Ok(())
    }
}
```

---

### Improvement 4: Metrics Collection (1-2 days)

**Create** `src/metrics.rs`:
```rust
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

pub struct Metrics {
    // Counters
    commands_processed: AtomicUsize,
    commands_succeeded: AtomicUsize,
    commands_failed: AtomicUsize,
    wake_words_detected: AtomicUsize,
    
    // Latency (microseconds)
    total_processing_latency: AtomicU64,
    audio_capture_latency: AtomicU64,
    stt_latency: AtomicU64,
    parsing_latency: AtomicU64,
    execution_latency: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            commands_processed: AtomicUsize::new(0),
            commands_succeeded: AtomicUsize::new(0),
            commands_failed: AtomicUsize::new(0),
            wake_words_detected: AtomicUsize::new(0),
            total_processing_latency: AtomicU64::new(0),
            audio_capture_latency: AtomicU64::new(0),
            stt_latency: AtomicU64::new(0),
            parsing_latency: AtomicU64::new(0),
            execution_latency: AtomicU64::new(0),
        }
    }
    
    pub fn record_command_processed(&self) {
        self.commands_processed.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_command_success(&self) {
        self.commands_succeeded.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_latency(&self, phase: MetricPhase, duration: std::time::Duration) {
        let micros = duration.as_micros() as u64;
        
        match phase {
            MetricPhase::AudioCapture => {
                self.audio_capture_latency.fetch_add(micros, Ordering::Relaxed);
            }
            MetricPhase::SpeechToText => {
                self.stt_latency.fetch_add(micros, Ordering::Relaxed);
            }
            MetricPhase::Parsing => {
                self.parsing_latency.fetch_add(micros, Ordering::Relaxed);
            }
            MetricPhase::Execution => {
                self.execution_latency.fetch_add(micros, Ordering::Relaxed);
            }
            MetricPhase::Total => {
                self.total_processing_latency.fetch_add(micros, Ordering::Relaxed);
            }
        }
    }
    
    pub fn print_summary(&self) {
        let processed = self.commands_processed.load(Ordering::Relaxed);
        let succeeded = self.commands_succeeded.load(Ordering::Relaxed);
        let failed = self.commands_failed.load(Ordering::Relaxed);
        
        println!("\n📊 LUNA Metrics Summary");
        println!("  Commands processed: {}", processed);
        println!("  Success rate: {:.1}%", 
            if processed > 0 { succeeded as f64 / processed as f64 * 100.0 } else { 0.0 }
        );
        
        if processed > 0 {
            let avg_total = self.total_processing_latency.load(Ordering::Relaxed) / processed as u64;
            println!("\n  Average latencies:");
            println!("    Total: {}ms", avg_total / 1000);
            println!("    Audio: {}ms", 
                self.audio_capture_latency.load(Ordering::Relaxed) / processed as u64 / 1000);
            println!("    STT: {}ms", 
                self.stt_latency.load(Ordering::Relaxed) / processed as u64 / 1000);
            println!("    Parse: {}ms", 
                self.parsing_latency.load(Ordering::Relaxed) / processed as u64 / 1000);
        }
    }
}

pub enum MetricPhase {
    AudioCapture,
    SpeechToText,
    Parsing,
    Execution,
    Total,
}

// Helper to measure execution time
pub struct MetricTimer {
    start: Instant,
    metrics: Arc<Metrics>,
    phase: MetricPhase,
}

impl MetricTimer {
    pub fn new(metrics: Arc<Metrics>, phase: MetricPhase) -> Self {
        Self {
            start: Instant::now(),
            metrics,
            phase,
        }
    }
}

impl Drop for MetricTimer {
    fn drop(&mut self) {
        self.metrics.record_latency(self.phase, self.start.elapsed());
    }
}
```

**Usage**:
```rust
impl Brain {
    pub fn process(&self, text: &str) -> Result<TaskPlan> {
        let _timer = MetricTimer::new(self.metrics.clone(), MetricPhase::Total);
        
        // Processing...
        
        self.metrics.record_command_processed();
        Ok(plan)
    }
}
```

---

## 🧪 Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_with_mocks() {
        let capture = MockAudioCapture::new();
        let wake_word = MockWakeWordDetector::new();
        let stt = MockSpeechToText::new();
        let processor = AudioProcessor::new(0.01, 1.0);
        
        let system = AudioSystem::new(capture, wake_word, stt, processor);
        
        // Test logic
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_end_to_end_with_events() {
    let event_bus = Arc::new(EventBus::new());
    event_bus.start_processing().await;
    
    // Subscribe to events
    let events_received = Arc::new(Mutex::new(Vec::new()));
    let events_clone = events_received.clone();
    
    event_bus.subscribe(vec![], move |event| {
        events_clone.lock().unwrap().push(event.clone());
    }).await;
    
    // Run system
    let audio_system = ProductionAudioSystem::create(&config.audio, &config.brain)
        .await
        .unwrap()
        .with_event_bus(event_bus.clone());
    
    // Verify events were published
    let events = events_received.lock().unwrap();
    assert!(events.len() > 0);
}
```

---

## 📝 Migration Checklist

- [ ] Add `async-trait` dependency
- [ ] Create trait definitions for all major components
- [ ] Implement traits for existing components
- [ ] Create mock implementations
- [ ] Refactor components to use traits
- [ ] Update main.rs to use new constructors
- [ ] Run all tests to ensure nothing broke
- [ ] Add event bus implementation
- [ ] Integrate event bus with components
- [ ] Add logging subscriber
- [ ] Add configuration validation
- [ ] Add metrics collection
- [ ] Update tests to use new architecture
- [ ] Update documentation

---

## 🎯 Success Criteria

After implementing these improvements, you should have:

1. **Testable Components**
   - All major components implement traits
   - Mock implementations available
   - Easy to unit test in isolation

2. **Loose Coupling**
   - Components communicate via events
   - Easy to add new components
   - No circular dependencies

3. **Observable System**
   - Metrics for all operations
   - Event logging
   - Performance tracking

4. **Reliable Configuration**
   - Validation on startup
   - Clear error messages
   - Prevents invalid states

---

## Next: Proceed with Phase 4

With these improvements in place, Phase 4 (Action Execution) will be much easier to implement and test!
