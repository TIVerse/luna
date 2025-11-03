//! Speech synthesizer
//!
//! Enhanced synthesizer with SSML support, chunking, and profiles.

use crate::error::Result;
use crate::tts::engine::{OsTtsEngine, TtsEngine, VoiceInfo};
use crate::tts::ssml::{chunk_text, parse_ssml, strip_ssml_tags};
use crate::tts::types::VoiceProfile;
use std::time::Duration;
use tracing::{debug, error, info};

/// Text-to-speech synthesizer with enhanced capabilities
pub struct TextToSpeech {
    engine: Box<dyn TtsEngine>,
    current_profile: VoiceProfile,
}

impl TextToSpeech {
    /// Create a new text-to-speech synthesizer with OS engine
    pub fn new() -> Result<Self> {
        info!("Initializing TTS synthesizer...");
        let engine = Box::new(OsTtsEngine::new()?);
        info!(
            "✅ TTS synthesizer initialized with engine: {}",
            engine.name()
        );

        Ok(Self {
            engine,
            current_profile: VoiceProfile::default(),
        })
    }

    /// Create with custom engine
    pub fn with_engine(engine: Box<dyn TtsEngine>) -> Self {
        let engine_name = engine.name().to_string();
        info!("TTS synthesizer using engine: {}", engine_name);

        Self {
            engine,
            current_profile: VoiceProfile::default(),
        }
    }

    /// Apply voice profile settings
    pub fn apply_profile(&mut self, profile: &VoiceProfile) -> Result<()> {
        debug!(
            "Applying voice profile: rate={}, pitch={}, volume={}",
            profile.rate, profile.pitch, profile.volume
        );

        if let Some(voice) = &profile.voice {
            let _ = self.engine.set_voice(voice);
        }

        self.engine.set_rate(profile.rate)?;
        self.engine.set_pitch(profile.pitch)?;
        self.engine.set_volume(profile.volume)?;

        self.current_profile = profile.clone();
        Ok(())
    }

    /// Speak text with default voice
    pub async fn speak(&mut self, text: &str) -> Result<()> {
        info!(
            "🔊 Speaking: \"{}\" (len={})",
            text.chars().take(50).collect::<String>(),
            text.len()
        );

        self.engine.speak(text, false).await
    }

    /// Speak text with interrupt capability
    pub async fn speak_interruptible(&mut self, text: &str) -> Result<()> {
        info!(
            "🔊 Speaking (interruptible): \"{}\"",
            text.chars().take(50).collect::<String>()
        );

        self.engine.speak(text, true).await
    }

    /// Speak text with SSML markup
    pub async fn speak_ssml(&mut self, ssml: &str) -> Result<()> {
        info!(
            "🔊 Speaking SSML: \"{}\" (len={})",
            strip_ssml_tags(ssml).chars().take(50).collect::<String>(),
            ssml.len()
        );

        let chunks = parse_ssml(ssml);

        for chunk in chunks {
            if let Some(break_ms) = chunk.break_ms {
                debug!("SSML break: {}ms", break_ms);
                tokio::time::sleep(Duration::from_millis(break_ms as u64)).await;
                continue;
            }

            if !chunk.text.is_empty() {
                // Apply chunk-specific modifiers
                let orig_rate = self.current_profile.rate;
                let orig_pitch = self.current_profile.pitch;

                if chunk.emphasis || chunk.rate_modifier != 1.0 || chunk.pitch_modifier != 1.0 {
                    let _ = self.engine.set_rate(orig_rate * chunk.rate_modifier);
                    let _ = self.engine.set_pitch(orig_pitch * chunk.pitch_modifier);
                }

                self.engine.speak(&chunk.text, false).await?;

                // Restore
                if chunk.emphasis || chunk.rate_modifier != 1.0 || chunk.pitch_modifier != 1.0 {
                    let _ = self.engine.set_rate(orig_rate);
                    let _ = self.engine.set_pitch(orig_pitch);
                }
            }
        }

        Ok(())
    }

    /// Speak text in chunks for lower latency
    pub async fn speak_chunked(&mut self, text: &str, max_chunk_len: usize) -> Result<()> {
        let chunks = chunk_text(text, max_chunk_len);
        info!("🔊 Speaking {} chunks", chunks.len());

        for (i, chunk) in chunks.iter().enumerate() {
            debug!(
                "Speaking chunk {}/{}: {}",
                i + 1,
                chunks.len(),
                chunk.chars().take(30).collect::<String>()
            );
            self.engine.speak(chunk, true).await?;
        }

        Ok(())
    }

    /// Speak text with a specific voice
    pub async fn speak_with_voice(&mut self, text: &str, voice: &str) -> Result<()> {
        info!("🔊 Speaking with voice '{}': \"{}\"", voice, text);

        self.engine.set_voice(voice)?;
        self.engine.speak(text, false).await
    }

    /// Stop current speech
    pub fn stop(&mut self) -> Result<()> {
        info!("⏹️ Stopping TTS");
        self.engine.stop()
    }

    /// Get list of available voices
    pub fn available_voices(&self) -> Result<Vec<String>> {
        let voices = self.engine.voices()?;
        Ok(voices.iter().map(|v| v.name.clone()).collect())
    }

    /// Get detailed voice information
    pub fn voice_info(&self) -> Result<Vec<VoiceInfo>> {
        self.engine.voices()
    }

    /// Check if currently speaking
    pub fn is_speaking(&self) -> bool {
        self.engine.is_speaking()
    }

    /// Set speech rate (0.1 to 10.0, 1.0 is normal)
    pub fn set_rate(&mut self, rate: f32) -> Result<()> {
        self.current_profile.rate = rate;
        self.engine.set_rate(rate)
    }

    /// Set speech pitch (0.0 to 2.0, 1.0 is normal)
    pub fn set_pitch(&mut self, pitch: f32) -> Result<()> {
        self.current_profile.pitch = pitch;
        self.engine.set_pitch(pitch)
    }

    /// Set speech volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.current_profile.volume = volume;
        self.engine.set_volume(volume)
    }

    /// Get engine name
    pub fn engine_name(&self) -> &str {
        self.engine.name()
    }
}

// Backward compatibility alias
pub type Synthesizer = TextToSpeech;
