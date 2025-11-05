//! God-level audio system module for LUNA
//!
//! Comprehensive audio pipeline with:
//! - Lock-free real-time capture
//! - Production-grade VAD and DSP
//! - Device management and hotplug
//! - Wake word detection (Porcupine + energy)
//! - Speech-to-text (Whisper + simulation)

pub mod capture;
pub mod device;
pub mod dsp;
pub mod mocks;
pub mod processor;
pub mod ring_buffer;
pub mod speech_to_text;
pub mod traits;
pub mod vad;
pub mod wake_word;

// Re-export commonly used types
pub use capture::AudioCapture;
pub use device::{AudioDeviceInfo, AudioDeviceManager};
pub use dsp::{AudioResampler, AutomaticGainControl, DspProcessor, NoiseSuppressor};
pub use processor::AudioProcessor;
pub use ring_buffer::LockFreeRingBuffer;
pub use speech_to_text::SpeechToText;
pub use traits::*;
pub use vad::{VadEngine, VoiceActivityDetector};
pub use wake_word::{WakeWordDetector, WakeWordEngine};

use crate::config::{AudioConfig, BrainConfig};
use crate::error::Result;
use crate::events::EventBus;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// Generic audio system coordinator with dependency injection
pub struct AudioSystem<C, W, S, P>
where
    C: AudioCaptureInterface,
    W: WakeWordDetectorInterface,
    S: SpeechToTextInterface,
    P: AudioProcessorInterface,
{
    capture: C,
    wake_word: W,
    stt: S,
    processor: P,
    event_bus: Option<Arc<EventBus>>,
}

impl<C, W, S, P> AudioSystem<C, W, S, P>
where
    C: AudioCaptureInterface,
    W: WakeWordDetectorInterface,
    S: SpeechToTextInterface,
    P: AudioProcessorInterface,
{
    /// Create a new audio system with injected dependencies
    pub fn new(capture: C, wake_word: W, stt: S, processor: P) -> Self {
        Self {
            capture,
            wake_word,
            stt,
            processor,
            event_bus: None,
        }
    }

    /// Add event bus for publishing events
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Start listening for audio
    pub fn start_listening(&mut self) -> Result<()> {
        self.capture.start()
    }

    /// Wait for wake word detection
    ///
    /// Continuously monitors audio buffer for wake word.
    /// Returns `true` when wake word is detected.
    pub async fn wait_for_wake_word(&mut self) -> Result<bool> {
        loop {
            // Get last 1 second of audio from ring buffer
            let ring_buffer = self.capture.get_ring_buffer();
            let buffer = {
                let buf = ring_buffer.lock().unwrap();
                buf.get_last_n_samples(1000)
            };

            if let Some(_keyword_idx) = self.wake_word.detect(&buffer).await? {
                // Publish event if event bus is configured
                if let Some(ref bus) = self.event_bus {
                    bus.publish(crate::events::LunaEvent::WakeWordDetected {
                        keyword: "luna".to_string(),
                        confidence: 0.95,
                    })
                    .await;
                }
                return Ok(true);
            }

            // Check every 100ms
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Stop the audio system
    pub fn stop(&mut self) -> Result<()> {
        self.capture.stop()
    }

    /// Check if speech-to-text is in simulated mode
    pub fn is_stt_simulated(&self) -> bool {
        self.stt.is_simulated()
    }

    /// Get reference to event bus (for testing)
    pub fn event_bus(&self) -> &Option<Arc<EventBus>> {
        &self.event_bus
    }

    /// Listen and transcribe command audio
    ///
    /// Records audio for up to `max_secs` seconds, processes it, and transcribes.
    /// Publishes `CommandTranscribed` event if event bus is configured.
    ///
    /// # Arguments
    /// * `max_secs` - Maximum recording duration in seconds
    ///
    /// # Returns
    /// Transcribed text from the audio
    pub async fn listen_and_transcribe(&mut self, max_secs: u64) -> crate::error::Result<String>
    where
        C: AudioCaptureInterface,
    {
        // 1. Collect audio from the stream for specified duration
        let mut audio = Vec::new();
        let mut rx = self.capture.get_audio_stream();
        let timeout = tokio::time::Duration::from_secs(max_secs);
        let start = tokio::time::Instant::now();

        loop {
            tokio::select! {
                Ok(samples) = rx.recv() => {
                    audio.extend(samples);
                    
                    // Stop on silence after at least 1 second of audio
                    if audio.len() > 48000 && start.elapsed() > tokio::time::Duration::from_secs(1) {
                        let last_chunk_size = 4800;
                        if audio.len() >= last_chunk_size {
                            let last_chunk = &audio[audio.len() - last_chunk_size..];
                            let rms: f32 = {
                                let sum: f32 = last_chunk.iter().map(|&s| s * s).sum();
                                (sum / last_chunk.len() as f32).sqrt()
                            };
                            if rms < 0.01 {
                                break;
                            }
                        }
                    }
                }
                _ = tokio::time::sleep_until(start + timeout) => {
                    break;
                }
            }
        }

        if audio.is_empty() {
            return Ok(String::new());
        }

        // 2. Process audio (noise gate + normalize)
        let mut processed = audio.clone();
        self.processor.apply_noise_gate(&mut processed);
        self.processor.normalize(&mut processed);

        // 3. Transcribe
        let text = self.stt.transcribe(&processed).await?;

        // 4. Publish event if event bus is configured
        if let Some(ref bus) = self.event_bus {
            bus.publish(crate::events::LunaEvent::CommandTranscribed {
                text: text.clone(),
                confidence: 1.0, // Best effort confidence; STT currently doesn't provide this
            })
            .await;
        }

        Ok(text)
    }
}

/// Type alias for production audio system
pub type ProductionAudioSystem =
    AudioSystem<AudioCapture, WakeWordDetector, SpeechToText, AudioProcessor>;

/// Factory for creating production audio system
impl ProductionAudioSystem {
    /// Create a production audio system from configuration
    pub async fn create(audio_config: &AudioConfig, brain_config: &BrainConfig) -> Result<Self> {
        info!("Initializing audio system...");

        let capture = AudioCapture::new(audio_config.clone())?;

        let wake_word = WakeWordDetector::new(
            audio_config.wake_words.clone(),
            brain_config.wake_word_sensitivity,
        )?;

        let stt = SpeechToText::new(Path::new(&brain_config.whisper_model_path))?;

        let processor = AudioProcessor::new(
            audio_config.silence_threshold * 0.5,
            1.0, // gain
        );

        info!("✅ Audio system initialized");

        Ok(Self::new(capture, wake_word, stt, processor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AudioConfig, BrainConfig};
    use mocks::*;

    #[tokio::test]
    async fn test_audio_system_creation() {
        let audio_config = AudioConfig::default();
        let brain_config = BrainConfig::default();

        let system = ProductionAudioSystem::create(&audio_config, &brain_config).await;
        assert!(system.is_ok());
    }

    #[tokio::test]
    async fn test_audio_system_with_mocks() {
        // Create mock components
        let capture = MockAudioCapture::new();
        let wake_word = MockWakeWordDetector::new();
        let stt = MockSpeechToText::new();
        let processor = MockAudioProcessor::new();

        // Create system with mocks
        let system = AudioSystem::new(capture, wake_word, stt, processor);

        assert!(system.is_stt_simulated());
    }

    #[tokio::test]
    async fn test_audio_system_with_event_bus() {
        let capture = MockAudioCapture::new();
        let wake_word = MockWakeWordDetector::new();
        let stt = MockSpeechToText::new();
        let processor = MockAudioProcessor::new();

        let event_bus = Arc::new(EventBus::new());
        let handle = event_bus.start_processing().await;

        let system = AudioSystem::new(capture, wake_word, stt, processor).with_event_bus(event_bus);

        assert!(system.event_bus().is_some());

        handle.abort();
    }
}
