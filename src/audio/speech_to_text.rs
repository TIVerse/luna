//! Speech-to-text transcription
//!
//! Converts audio to text. Stub implementation returns simulated transcriptions.
//! Can be upgraded to use Whisper AI when models are available.

use super::traits::SpeechToTextInterface;
use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;
use tracing::{debug, info};

/// Speech-to-text engine (stub implementation)
///
/// **Note**: This is a stub implementation for testing the audio pipeline.
/// In production, integrate Whisper with:
/// ```text
/// whisper-rs = { version = "0.11", features = ["metal"] }
/// ```
pub struct SpeechToText {
    model_path: String,
    simulated_mode: bool,
}

impl SpeechToText {
    /// Create a new speech-to-text instance
    ///
    /// # Arguments
    /// * `model_path` - Path to Whisper model file
    ///
    /// # Note
    /// Currently returns a stub implementation. When Whisper model is available,
    /// this will load the actual model for transcription.
    pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self> {
        let model_path_str = model_path.as_ref().display().to_string();

        // Check if model exists
        let simulated_mode = !model_path.as_ref().exists();

        if simulated_mode {
            info!("⚠️  Whisper model not found at: {}", model_path_str);
            info!("✅ Speech-to-text initialized (simulated mode)");
            info!("   For production, download Whisper model:");
            info!(
                "   wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
            );
        } else {
            info!("✅ Speech-to-text initialized");
            info!("   Model: {}", model_path_str);
        }

        Ok(Self {
            model_path: model_path_str,
            simulated_mode,
        })
    }

    /// Transcribe audio to text
    ///
    /// # Arguments
    /// * `audio` - Audio samples (f32, typically 16kHz sample rate)
    ///
    /// # Returns
    /// Transcribed text string
    ///
    /// # Note
    /// Currently returns simulated transcriptions based on audio characteristics.
    /// When Whisper is integrated, this will return actual transcriptions.
    pub async fn transcribe(&self, audio: &[f32]) -> Result<String> {
        if audio.is_empty() {
            return Ok(String::new());
        }

        debug!("Transcribing {} samples", audio.len());
        let start = std::time::Instant::now();

        let text = if self.simulated_mode {
            // Simulated transcription based on audio characteristics
            self.simulate_transcription(audio)
        } else {
            // TODO: Actual Whisper transcription when model is available
            self.simulate_transcription(audio)
        };

        let duration = start.elapsed();
        info!(
            "💬 Transcribed in {:.2}s: \"{}\"",
            duration.as_secs_f32(),
            text
        );

        Ok(text)
    }

    /// Simulate transcription for testing
    fn simulate_transcription(&self, audio: &[f32]) -> String {
        // Analyze audio characteristics for simulation
        let duration_secs = audio.len() as f32 / 16000.0;
        let energy = self.calculate_energy(audio);

        // Generate simulated transcription based on characteristics
        if duration_secs < 0.5 {
            "hey".to_string()
        } else if duration_secs < 1.5 {
            if energy > 0.2 {
                "hey luna".to_string()
            } else {
                "hello".to_string()
            }
        } else if duration_secs < 3.0 {
            if energy > 0.3 {
                "hey luna open chrome".to_string()
            } else if energy > 0.15 {
                "open chrome".to_string()
            } else {
                "what time is it".to_string()
            }
        } else {
            // Longer audio
            "hey luna can you please open chrome and search for rust programming tutorials"
                .to_string()
        }
    }

    /// Calculate audio energy
    fn calculate_energy(&self, audio: &[f32]) -> f32 {
        if audio.is_empty() {
            return 0.0;
        }

        let sum: f32 = audio.iter().map(|&s| s * s).sum();
        (sum / audio.len() as f32).sqrt()
    }

    /// Check if running in simulated mode
    pub fn is_simulated(&self) -> bool {
        self.simulated_mode
    }
}

// Implement SpeechToTextInterface trait
#[async_trait]
impl SpeechToTextInterface for SpeechToText {
    async fn transcribe(&self, audio: &[f32]) -> Result<String> {
        // Call the existing transcribe method
        self.transcribe(audio).await
    }

    fn is_simulated(&self) -> bool {
        self.simulated_mode
    }

    fn sample_rate(&self) -> u32 {
        16000
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_speech_to_text_creation() {
        let stt = SpeechToText::new(PathBuf::from("models/whisper-base.bin"));
        assert!(stt.is_ok());
    }

    #[tokio::test]
    async fn test_transcription() {
        let stt = SpeechToText::new(PathBuf::from("models/whisper-base.bin")).unwrap();

        // Short audio
        let audio = vec![0.2; 8000]; // ~0.5 seconds at 16kHz
        let text = stt.transcribe(&audio).await.unwrap();
        assert!(!text.is_empty());

        // Longer audio
        let audio = vec![0.3; 32000]; // ~2 seconds
        let text = stt.transcribe(&audio).await.unwrap();
        assert!(!text.is_empty());
    }

    #[tokio::test]
    async fn test_empty_audio() {
        let stt = SpeechToText::new(PathBuf::from("models/whisper-base.bin")).unwrap();
        let text = stt.transcribe(&[]).await.unwrap();
        assert_eq!(text, "");
    }

    #[test]
    fn test_energy_calculation() {
        let stt = SpeechToText::new(PathBuf::from("models/whisper-base.bin")).unwrap();
        let audio = vec![0.5, -0.5, 0.3, -0.3];
        let energy = stt.calculate_energy(&audio);
        assert!(energy > 0.0);
    }
}
