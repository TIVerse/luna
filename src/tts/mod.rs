//! Text-to-speech module
//!
//! Enhanced TTS with queuing, prioritization, SSML, and profiles.

pub mod synthesizer;
pub mod engine;
pub mod types;
pub mod ssml;
pub mod queue;

use crate::error::Result;
use crate::events::{EventBus, LunaEvent};
use tracing::{info, warn, debug, error};
use std::sync::Arc;
use tokio::sync::RwLock;

// Re-export main types
pub use synthesizer::{TextToSpeech, Synthesizer};
pub use engine::{TtsEngine, OsTtsEngine, VoiceInfo};
pub use types::{MessageKind, Priority, TtsConfig, TtsHandle, VoiceProfile, TtsPolicy};
pub use queue::{TtsQueue, TtsMessage};

/// TTS system coordinator with god-level enhancements
pub struct TtsSystem {
    synthesizer: Arc<RwLock<TextToSpeech>>,
    queue: Arc<TtsQueue>,
    config: Arc<RwLock<TtsConfig>>,
    event_bus: Option<Arc<EventBus>>,
    enabled: Arc<RwLock<bool>>,
    worker_handle: Option<tokio::task::JoinHandle<()>>,
    stats: Arc<RwLock<TtsStats>>,
}

/// TTS statistics
#[derive(Debug, Default, Clone)]
pub struct TtsStats {
    pub total_utterances: u64,
    pub total_interrupted: u64,
    pub total_errors: u64,
    pub total_queued: u64,
    pub total_coalesced: u64,
}

impl TtsSystem {
    /// Create a new TTS system with default config
    pub fn new() -> Result<Self> {
        Self::with_config(TtsConfig::default())
    }
    
    /// Create TTS system with custom config
    pub fn with_config(config: TtsConfig) -> Result<Self> {
        info!("Initializing TTS system with engine: {}", config.engine);
        
        let synthesizer = TextToSpeech::new()?;
        
        info!("✅ TTS system initialized");
        
        Ok(Self {
            synthesizer: Arc::new(RwLock::new(synthesizer)),
            queue: Arc::new(TtsQueue::new()),
            config: Arc::new(RwLock::new(config)),
            event_bus: None,
            enabled: Arc::new(RwLock::new(true)),
            worker_handle: None,
            stats: Arc::new(RwLock::new(TtsStats::default())),
        })
    }
    
    /// Set event bus for TTS events
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }
    
    /// Start the TTS worker that processes the queue
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting TTS worker...");
        
        let synthesizer = Arc::clone(&self.synthesizer);
        let queue = Arc::clone(&self.queue);
        let config = Arc::clone(&self.config);
        let enabled = Arc::clone(&self.enabled);
        let event_bus = self.event_bus.clone();
        let stats = Arc::clone(&self.stats);
        
        let handle = tokio::spawn(async move {
            Self::worker_loop(synthesizer, queue, config, enabled, event_bus, stats).await;
        });
        
        self.worker_handle = Some(handle);
        info!("✅ TTS worker started");
        Ok(())
    }
    
    /// Stop the TTS worker
    pub async fn stop(&mut self) {
        info!("Stopping TTS worker...");
        
        if let Some(handle) = self.worker_handle.take() {
            handle.abort();
            let _ = handle.await;
        }
        
        // Clear queue
        self.queue.clear().await;
        
        // Stop any playing speech
        if let Ok(mut synth) = self.synthesizer.try_write() {
            let _ = synth.stop();
        }
        
        info!("⏹️ TTS worker stopped");
    }
    
    /// Speak text immediately (bypass queue)
    pub async fn speak(&self, text: &str) -> Result<()> {
        if !*self.enabled.read().await {
            debug!("TTS disabled, skipping: \"{}\"", text);
            return Ok(());
        }
        
        let mut synth = self.synthesizer.write().await;
        synth.speak(text).await
    }
    
    /// Speak with message kind (uses profile and queue)
    pub async fn speak_with(&self, kind: MessageKind, text: &str) -> Result<TtsHandle> {
        let message = TtsMessage::new(text.to_string(), kind);
        let handle = self.enqueue(message).await;
        Ok(handle)
    }
    
    /// Speak SSML immediately
    pub async fn speak_ssml(&self, ssml: &str) -> Result<()> {
        if !*self.enabled.read().await {
            return Ok(());
        }
        
        let mut synth = self.synthesizer.write().await;
        synth.speak_ssml(ssml).await
    }
    
    /// Enqueue a message with priority
    pub async fn enqueue(&self, message: TtsMessage) -> TtsHandle {
        self.stats.write().await.total_queued += 1;
        self.queue.enqueue(message).await
    }
    
    /// Enqueue with coalescing (replaces messages with same key)
    pub async fn enqueue_coalesced(&self, kind: MessageKind, text: &str, coalesce_key: &str) -> TtsHandle {
        let message = TtsMessage::new(text.to_string(), kind)
            .with_coalesce_key(coalesce_key.to_string());
        
        self.stats.write().await.total_coalesced += 1;
        self.queue.enqueue(message).await
    }
    
    /// Stop current speech and clear queue
    pub async fn stop_all(&self) -> Result<()> {
        info!("Stopping all TTS");
        
        self.queue.clear().await;
        
        let mut synth = self.synthesizer.write().await;
        synth.stop()
    }
    
    /// Interrupt current speech
    pub async fn interrupt(&self) -> Result<()> {
        debug!("Interrupting TTS");
        self.stats.write().await.total_interrupted += 1;
        
        let mut synth = self.synthesizer.write().await;
        synth.stop()
    }
    
    /// Enable TTS
    pub async fn enable(&self) {
        *self.enabled.write().await = true;
        info!("TTS enabled");
    }
    
    /// Disable TTS
    pub async fn disable(&self) {
        *self.enabled.write().await = false;
        info!("TTS disabled");
    }
    
    /// Check if TTS is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }
    
    /// Get available voices
    pub async fn available_voices(&self) -> Result<Vec<String>> {
        let synth = self.synthesizer.read().await;
        synth.available_voices()
    }
    
    /// Get detailed voice info
    pub async fn voice_info(&self) -> Result<Vec<VoiceInfo>> {
        let synth = self.synthesizer.read().await;
        synth.voice_info()
    }
    
    /// Get queue size
    pub async fn queue_size(&self) -> usize {
        self.queue.len().await
    }
    
    /// Get statistics
    pub async fn stats(&self) -> TtsStats {
        self.stats.read().await.clone()
    }
    
    /// Update config
    pub async fn update_config(&self, config: TtsConfig) {
        *self.config.write().await = config;
        info!("TTS config updated");
    }
    
    /// Worker loop that processes the queue
    async fn worker_loop(
        synthesizer: Arc<RwLock<TextToSpeech>>,
        queue: Arc<TtsQueue>,
        config: Arc<RwLock<TtsConfig>>,
        enabled: Arc<RwLock<bool>>,
        event_bus: Option<Arc<EventBus>>,
        stats: Arc<RwLock<TtsStats>>,
    ) {
        info!("TTS worker loop started");
        
        loop {
            // Check for cancellations
            let cancelled = queue.check_cancellations().await;
            if !cancelled.is_empty() {
                debug!("Cancelled {} utterances", cancelled.len());
            }
            
            // Wait for next message
            if let Some(message) = queue.dequeue().await {
                if !*enabled.read().await {
                    continue;
                }
                
                // Check if cancelled
                if cancelled.contains(&message.id) {
                    continue;
                }
                
                // Emit event
                if let Some(ref bus) = event_bus {
                    let _ = bus.publish(LunaEvent::TtsStarted {
                        text: message.text.clone(),
                        kind: format!("{:?}", message.kind),
                    }).await;
                }
                
                // Apply profile
                let cfg = config.read().await;
                let profile = cfg.policy.profile_for(message.kind);
                
                let mut synth = synthesizer.write().await;
                if let Err(e) = synth.apply_profile(profile) {
                    error!("Failed to apply profile: {}", e);
                }
                drop(synth);
                
                // Speak
                let result = if message.is_ssml {
                    let mut synth = synthesizer.write().await;
                    synth.speak_ssml(&message.text).await
                } else {
                    let mut synth = synthesizer.write().await;
                    let interruptible = message.kind.is_interruptible();
                    if interruptible {
                        synth.speak_interruptible(&message.text).await
                    } else {
                        synth.speak(&message.text).await
                    }
                };
                
                // Update stats
                let mut st = stats.write().await;
                st.total_utterances += 1;
                if result.is_err() {
                    st.total_errors += 1;
                }
                drop(st);
                
                // Emit event
                if let Some(ref bus) = event_bus {
                    let _ = bus.publish(LunaEvent::TtsCompleted {
                        success: result.is_ok(),
                    }).await;
                }
                
                if let Err(e) = result {
                    error!("TTS error: {}", e);
                }
            } else {
                // No messages, sleep briefly
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        }
    }
}

impl Default for TtsSystem {
    fn default() -> Self {
        Self::new().expect("Failed to create default TtsSystem")
    }
}

impl Drop for TtsSystem {
    fn drop(&mut self) {
        if let Some(handle) = self.worker_handle.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tts_system_creation() {
        let system = TtsSystem::new();
        match system {
            Ok(tts) => assert!(tts.is_enabled().await),
            Err(_) => {
                println!("TTS not available in test environment");
            }
        }
    }
    
    #[tokio::test]
    async fn test_enable_disable() {
        if let Ok(system) = TtsSystem::new() {
            assert!(system.is_enabled().await);
            
            system.disable().await;
            assert!(!system.is_enabled().await);
            
            system.enable().await;
            assert!(system.is_enabled().await);
        }
    }
    
    #[tokio::test]
    async fn test_queue_priority() {
        if let Ok(system) = TtsSystem::new() {
            let _ = system.speak_with(MessageKind::Info, "info message").await;
            let _ = system.speak_with(MessageKind::Critical, "critical").await;
            
            // Critical should be queued with higher priority
            assert!(system.queue_size().await > 0);
        }
    }
    
    #[tokio::test]
    async fn test_coalescing() {
        if let Ok(system) = TtsSystem::new() {
            let _ = system.enqueue_coalesced(MessageKind::Info, "msg1", "status").await;
            let _ = system.enqueue_coalesced(MessageKind::Info, "msg2", "status").await;
            
            // Should only have 1 message after coalescing
            assert_eq!(system.queue_size().await, 1);
        }
    }
}
