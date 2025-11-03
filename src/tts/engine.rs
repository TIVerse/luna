//! TTS engine abstraction
//!
//! Defines pluggable TTS engine interface for multiple backends.

use crate::error::{LunaError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Voice metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    pub id: String,
    pub name: String,
    pub locale: Option<String>,
    pub gender: Option<Gender>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Neutral,
}

/// TTS engine trait - abstraction over different TTS backends
#[async_trait]
pub trait TtsEngine: Send + Sync {
    /// Speak text with default voice (blocking until complete)
    async fn speak(&mut self, text: &str, interrupt: bool) -> Result<()>;

    /// Speak text with SSML markup
    async fn speak_ssml(&mut self, ssml: &str, interrupt: bool) -> Result<()>;

    /// Stop current speech immediately
    fn stop(&mut self) -> Result<()>;

    /// Set voice by ID
    fn set_voice(&mut self, voice_id: &str) -> Result<()>;

    /// Set speech rate (0.1 to 10.0, 1.0 is normal)
    fn set_rate(&mut self, rate: f32) -> Result<()>;

    /// Set speech pitch (0.0 to 2.0, 1.0 is normal)
    fn set_pitch(&mut self, pitch: f32) -> Result<()>;

    /// Set speech volume (0.0 to 1.0)
    fn set_volume(&mut self, volume: f32) -> Result<()>;

    /// Get list of available voices
    fn voices(&self) -> Result<Vec<VoiceInfo>>;

    /// Check if engine is currently speaking
    fn is_speaking(&self) -> bool;

    /// Get engine name
    fn name(&self) -> &str;
}

/// OS TTS Engine using the tts crate
pub struct OsTtsEngine {
    tts: tts::Tts,
    is_speaking: bool,
}

impl OsTtsEngine {
    /// Create a new OS TTS engine
    pub fn new() -> Result<Self> {
        let tts = tts::Tts::default()
            .map_err(|e| LunaError::tts_error(format!("Failed to initialize OS TTS: {}", e)))?;

        Ok(Self {
            tts,
            is_speaking: false,
        })
    }
}

#[async_trait]
impl TtsEngine for OsTtsEngine {
    async fn speak(&mut self, text: &str, interrupt: bool) -> Result<()> {
        self.is_speaking = true;
        let result = self
            .tts
            .speak(text, interrupt)
            .map_err(|e| LunaError::tts_error(format!("Failed to speak: {}", e)));
        self.is_speaking = false;
        result?;
        Ok(())
    }

    async fn speak_ssml(&mut self, ssml: &str, interrupt: bool) -> Result<()> {
        // OS TTS doesn't support SSML directly, strip tags and speak plain text
        let plain = crate::tts::ssml::strip_ssml_tags(ssml);
        self.speak(&plain, interrupt).await
    }

    fn stop(&mut self) -> Result<()> {
        self.is_speaking = false;
        self.tts
            .stop()
            .map_err(|e| LunaError::tts_error(format!("Failed to stop: {}", e)))?;
        Ok(())
    }

    fn set_voice(&mut self, voice_id: &str) -> Result<()> {
        let voices = self
            .tts
            .voices()
            .map_err(|e| LunaError::tts_error(format!("Failed to get voices: {}", e)))?;

        let target = voices.iter().find(|v| {
            v.id() == voice_id || v.name().to_lowercase().contains(&voice_id.to_lowercase())
        });

        if let Some(voice) = target {
            self.tts
                .set_voice(voice)
                .map_err(|e| LunaError::tts_error(format!("Failed to set voice: {}", e)))?;
        }

        Ok(())
    }

    fn set_rate(&mut self, rate: f32) -> Result<()> {
        self.tts
            .set_rate(rate)
            .map_err(|e| LunaError::tts_error(format!("Failed to set rate: {}", e)))?;
        Ok(())
    }

    fn set_pitch(&mut self, pitch: f32) -> Result<()> {
        self.tts
            .set_pitch(pitch)
            .map_err(|e| LunaError::tts_error(format!("Failed to set pitch: {}", e)))?;
        Ok(())
    }

    fn set_volume(&mut self, volume: f32) -> Result<()> {
        self.tts
            .set_volume(volume)
            .map_err(|e| LunaError::tts_error(format!("Failed to set volume: {}", e)))?;
        Ok(())
    }

    fn voices(&self) -> Result<Vec<VoiceInfo>> {
        let voices = self
            .tts
            .voices()
            .map_err(|e| LunaError::tts_error(format!("Failed to get voices: {}", e)))?;

        Ok(voices
            .iter()
            .map(|v| VoiceInfo {
                id: v.id().to_string(),
                name: v.name().to_string(),
                locale: None,
                gender: None,
            })
            .collect())
    }

    fn is_speaking(&self) -> bool {
        self.is_speaking
    }

    fn name(&self) -> &str {
        "os-tts"
    }
}
