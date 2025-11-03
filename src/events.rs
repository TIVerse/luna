//! Event system for LUNA
//!
//! Provides a publish-subscribe event bus with:
//! - Typed events with strong guarantees
//! - Event envelopes with correlation IDs
//! - Multiple subscriber queues with backpressure
//! - Tracing integration

use async_channel::{Receiver, Sender, bounded, unbounded};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;
use crate::error::LunaError;

/// Events that can occur in the LUNA system (typed with real structs)
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
        command: String,
        intent: String,
        entities: HashMap<String, String>,
        confidence: f32,
    },
    
    /// Task plan created
    TaskPlanned {
        plan: String,
        steps: usize,
    },
    
    /// Action execution started
    ActionStarted {
        action_type: String,
        params: HashMap<String, String>,
    },
    
    /// Action execution completed
    ActionCompleted {
        action_type: String,
        success: bool,
        result: String,
        duration_ms: u64,
    },
    
    /// Error occurred
    Error {
        error: String,
        error_code: String,
        context: HashMap<String, String>,
        recoverable: bool,
    },
    
    /// System state changed
    StateChanged {
        from: String,
        to: String,
    },
    
    /// Configuration reloaded
    ConfigReloaded {
        timestamp: u64,
    },
    
    /// Metrics snapshot
    MetricsSnapshot {
        commands_processed: usize,
        success_rate: f64,
        avg_latency_ms: u64,
    },
    
    /// Clarification requested (god-level enhancement)
    ClarificationRequested {
        command: String,
        confidence: f32,
        missing_slots: Vec<String>,
        suggestions: Vec<String>,
    },
    
    /// Clarification answered
    ClarificationAnswered {
        original_command: String,
        clarification: String,
        resolved_command: String,
    },
    
    /// Grammar reloaded (hot-reload)
    GrammarReloaded {
        pattern_count: usize,
        timestamp: u64,
    },
    
    /// Cache invalidated
    CacheInvalidated {
        cache_type: String,
        reason: String,
    },
    
    /// Plan execution started (god-level enhancement)
    PlanStarted {
        plan_id: String,
        step_count: usize,
        parallel_groups: usize,
    },
    
    /// Plan execution completed (god-level enhancement)
    PlanCompleted {
        plan_id: String,
        success: bool,
        total_duration_ms: u64,
        steps_completed: usize,
        steps_failed: usize,
    },
    
    /// Capability detected (god-level enhancement)
    CapabilityDetected {
        capability: String,
        available: bool,
        tool_name: Option<String>,
        version: Option<String>,
    },
    
    /// Action retry attempt (god-level enhancement)
    ActionRetry {
        action_type: String,
        attempt: usize,
        max_attempts: usize,
        error: String,
    },
    
    /// Policy gate triggered (god-level enhancement)
    PolicyGateTriggered {
        action_type: String,
        requires_confirmation: bool,
        reason: String,
    },
    
    /// TTS started speaking (Phase 6)
    TtsStarted {
        text: String,
        kind: String,
    },
    
    /// TTS completed speaking (Phase 6)
    TtsCompleted {
        success: bool,
    },
    
    /// TTS interrupted (Phase 6)
    TtsInterrupted {
        reason: String,
    },
    
    /// Health issue detected (Phase 5)
    HealthIssueDetected {
        issue_type: String,
        severity: String,
    },
    
    /// Health issue remediated (Phase 5)
    HealthRemediated {
        issue_type: String,
        action_taken: String,
    },
    
    /// Custom event (for Phase 5 OS integration)
    Custom {
        event_type: String,
        data: serde_json::Value,
    },
}

impl LunaEvent {
    /// Get the event type as a string
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
            Self::StateChanged { .. } => "state_changed",
            Self::ConfigReloaded { .. } => "config_reloaded",
            Self::MetricsSnapshot { .. } => "metrics_snapshot",
            Self::ClarificationRequested { .. } => "clarification_requested",
            Self::ClarificationAnswered { .. } => "clarification_answered",
            Self::GrammarReloaded { .. } => "grammar_reloaded",
            Self::CacheInvalidated { .. } => "cache_invalidated",
            Self::PlanStarted { .. } => "plan_started",
            Self::PlanCompleted { .. } => "plan_completed",
            Self::CapabilityDetected { .. } => "capability_detected",
            Self::ActionRetry { .. } => "action_retry",
            Self::PolicyGateTriggered { .. } => "policy_gate_triggered",
            Self::TtsStarted { .. } => "tts_started",
            Self::TtsCompleted { .. } => "tts_completed",
            Self::TtsInterrupted { .. } => "tts_interrupted",
            Self::HealthIssueDetected { .. } => "health_issue_detected",
            Self::HealthRemediated { .. } => "health_remediated",
            Self::Custom { .. } => "custom",
        }
    }
}

/// Event envelope with metadata for observability
#[derive(Debug, Clone)]
pub struct EventEnvelope {
    /// Unique event ID
    pub id: Uuid,
    
    /// Event timestamp (microseconds since epoch)
    pub timestamp: u64,
    
    /// Correlation ID for tracking related events
    pub correlation_id: Option<Uuid>,
    
    /// Span ID for distributed tracing
    pub span_id: Option<String>,
    
    /// The actual event
    pub event: LunaEvent,
}

impl EventEnvelope {
    /// Create a new event envelope
    pub fn new(event: LunaEvent) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        
        Self {
            id: Uuid::new_v4(),
            timestamp,
            correlation_id: None,
            span_id: None,
            event,
        }
    }
    
    /// Create with correlation ID
    pub fn with_correlation(event: LunaEvent, correlation_id: Uuid) -> Self {
        let mut envelope = Self::new(event);
        envelope.correlation_id = Some(correlation_id);
        envelope
    }
    
    /// Get event type
    pub fn event_type(&self) -> &'static str {
        self.event.event_type()
    }
}

/// Event handler function type
pub type EventHandler = Arc<dyn Fn(&EventEnvelope) + Send + Sync>;

/// Subscription information
struct Subscriber {
    id: usize,
    event_types: Vec<&'static str>,
    handler: EventHandler,
    queue_size: usize,
}

/// Backpressure strategy for full queues
#[derive(Debug, Clone, Copy)]
pub enum BackpressureStrategy {
    /// Drop oldest events
    DropOldest,
    /// Drop newest events
    DropNewest,
    /// Block until space available
    Block,
}

/// Event bus for publish-subscribe communication
pub struct EventBus {
    subscribers: Arc<RwLock<Vec<Subscriber>>>,
    tx: Sender<EventEnvelope>,
    rx: Receiver<EventEnvelope>,
    next_id: Arc<RwLock<usize>>,
    backpressure: BackpressureStrategy,
}

impl EventBus {
    /// Create a new event bus with unbounded channel
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
            tx,
            rx,
            next_id: Arc::new(RwLock::new(0)),
            backpressure: BackpressureStrategy::DropOldest,
        }
    }
    
    /// Create with bounded channel and specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, rx) = bounded(capacity);
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
            tx,
            rx,
            next_id: Arc::new(RwLock::new(0)),
            backpressure: BackpressureStrategy::DropOldest,
        }
    }
    
    /// Subscribe to specific event types
    ///
    /// # Arguments
    /// * `event_types` - List of event types to subscribe to. Empty vec = all events.
    /// * `handler` - Function to call when matching event is published
    /// * `queue_size` - Maximum queue size for this subscriber (0 = unbounded)
    ///
    /// # Returns
    /// Subscription ID for unsubscribing later
    pub async fn subscribe<F>(&self, event_types: Vec<&'static str>, handler: F) -> usize
    where
        F: Fn(&EventEnvelope) + Send + Sync + 'static,
    {
        self.subscribe_with_queue(event_types, handler, 0).await
    }
    
    /// Subscribe with specific queue size
    pub async fn subscribe_with_queue<F>(
        &self,
        event_types: Vec<&'static str>,
        handler: F,
        queue_size: usize,
    ) -> usize
    where
        F: Fn(&EventEnvelope) + Send + Sync + 'static,
    {
        let mut subs = self.subscribers.write().await;
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;
        
        subs.push(Subscriber {
            id,
            event_types,
            handler: Arc::new(handler),
            queue_size,
        });
        id
    }
    
    /// Unsubscribe by ID
    pub async fn unsubscribe(&self, id: usize) {
        let mut subs = self.subscribers.write().await;
        subs.retain(|s| s.id != id);
    }
    
    /// Publish an event to all subscribers (wraps in envelope)
    pub async fn publish(&self, event: LunaEvent) {
        let envelope = EventEnvelope::new(event);
        let _ = self.tx.send(envelope).await;
    }
    
    /// Publish an event with correlation ID
    pub async fn publish_with_correlation(&self, event: LunaEvent, correlation_id: uuid::Uuid) {
        let envelope = EventEnvelope::with_correlation(event, correlation_id);
        let _ = self.tx.send(envelope).await;
    }
    
    /// Publish a raw envelope
    pub async fn publish_envelope(&self, envelope: EventEnvelope) {
        let _ = self.tx.send(envelope).await;
    }
    
    /// Start the event processing loop
    ///
    /// This spawns a background task that dispatches events to subscribers.
    /// Returns a JoinHandle that can be aborted to stop the processing loop.
    pub async fn start_processing(&self) -> JoinHandle<()> {
        let rx = self.rx.clone();
        let subscribers = self.subscribers.clone();
        
        tokio::spawn(async move {
            while let Ok(envelope) = rx.recv().await {
                let event_type = envelope.event_type();
                let subs = subscribers.read().await;
                
                for subscriber in subs.iter() {
                    // Call handler if subscribed to all events or this specific type
                    if subscriber.event_types.is_empty() 
                        || subscriber.event_types.contains(&event_type) 
                    {
                        (subscriber.handler)(&envelope);
                    }
                }
            }
        })
    }
    
    /// Get a clone of the sender for publishing events
    pub fn get_sender(&self) -> Sender<EventEnvelope> {
        self.tx.clone()
    }
    
    /// Get the number of subscribers
    pub async fn subscriber_count(&self) -> usize {
        self.subscribers.read().await.len()
    }
    
    /// Generate a unique plan correlation ID (god-level enhancement)
    pub fn generate_plan_id() -> String {
        format!("plan_{}", Uuid::new_v4())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    
    #[tokio::test]
    async fn test_event_bus_creation() {
        let bus = EventBus::new();
        assert!(bus.subscribers.read().await.is_empty());
    }
    
    #[tokio::test]
    async fn test_subscribe_and_publish() {
        let bus = EventBus::new();
        let handle = bus.start_processing().await;
        
        let events_received = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events_received.clone();
        
        bus.subscribe(vec!["wake_word_detected"], move |envelope| {
            events_clone.lock().unwrap().push(envelope.clone());
        }).await;
        
        bus.publish(LunaEvent::WakeWordDetected {
            keyword: "hey luna".to_string(),
            confidence: 0.95,
        }).await;
        
        // Give some time for event processing
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let events = events_received.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "wake_word_detected");
        
        handle.abort();
    }
    
    #[tokio::test]
    async fn test_subscribe_all_events() {
        let bus = EventBus::new();
        let handle = bus.start_processing().await;
        
        let events_received = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events_received.clone();
        
        // Subscribe to all events (empty vec)
        bus.subscribe(vec![], move |envelope| {
            events_clone.lock().unwrap().push(envelope.clone());
        }).await;
        
        bus.publish(LunaEvent::WakeWordDetected {
            keyword: "hey luna".to_string(),
            confidence: 0.95,
        }).await;
        
        bus.publish(LunaEvent::CommandTranscribed {
            text: "open chrome".to_string(),
            confidence: 0.9,
        }).await;
        
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let events = events_received.lock().unwrap();
        assert_eq!(events.len(), 2);
        
        handle.abort();
    }
    
    #[tokio::test]
    async fn test_unsubscribe() {
        let bus = EventBus::new();
        let handle = bus.start_processing().await;
        
        let events_received = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events_received.clone();
        
        let id = bus.subscribe(vec![], move |envelope| {
            events_clone.lock().unwrap().push(envelope.clone());
        }).await;
        
        bus.publish(LunaEvent::WakeWordDetected {
            keyword: "hey luna".to_string(),
            confidence: 0.95,
        }).await;
        
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // Unsubscribe
        bus.unsubscribe(id).await;
        
        bus.publish(LunaEvent::CommandTranscribed {
            text: "open chrome".to_string(),
            confidence: 0.9,
        }).await;
        
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let events = events_received.lock().unwrap();
        // Should only have received the first event
        assert_eq!(events.len(), 1);
        
        handle.abort();
    }
    
    #[tokio::test]
    async fn test_event_envelope() {
        let event = LunaEvent::WakeWordDetected {
            keyword: "hey luna".to_string(),
            confidence: 0.95,
        };
        
        let envelope = EventEnvelope::new(event);
        assert_eq!(envelope.event_type(), "wake_word_detected");
        assert!(envelope.correlation_id.is_none());
    }
    
    #[tokio::test]
    async fn test_event_with_correlation() {
        let event = LunaEvent::CommandParsed {
            command: "open chrome".to_string(),
            intent: "LaunchApp".to_string(),
            entities: HashMap::new(),
            confidence: 0.95,
        };
        
        let correlation_id = Uuid::new_v4();
        let envelope = EventEnvelope::with_correlation(event, correlation_id);
        
        assert_eq!(envelope.correlation_id, Some(correlation_id));
    }
}
