//! Wake word detection with multiple engines
//!
//! Supports:
//! - Porcupine (production-grade, feature-gated)
//! - Energy-based (simple fallback)
//! - OpenWakeWord (future: ONNX-based)

use super::traits::WakeWordDetectorInterface;
use crate::error::Result;
use async_trait::async_trait;
use tracing::{info, warn};

#[cfg(feature = "porcupine")]
use pv_porcupine::porcupine::{Porcupine, PorcupineBuilder};

/// Wake word engine type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeWordEngine {
    /// Porcupine (production-grade)
    Porcupine,
    /// Energy-based (simple)
    Energy,
    /// OpenWakeWord (future: ONNX-based)
    OpenWakeWord,
}

impl WakeWordEngine {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "porcupine" => WakeWordEngine::Porcupine,
            "openwakeword" => WakeWordEngine::OpenWakeWord,
            _ => WakeWordEngine::Energy,
        }
    }
}

/// Wake word detector with multiple engine support
pub struct WakeWordDetector {
    engine: WakeWordEngine,
    keywords: Vec<String>,
    sensitivity: f32,
    energy_threshold: f32,

    #[cfg(feature = "porcupine")]
    porcupine: Option<Porcupine>,
}

impl WakeWordDetector {
    /// Create a new wake word detector
    ///
    /// # Arguments
    /// * `engine` - Wake word engine to use
    /// * `keywords` - Wake words to detect
    /// * `sensitivity` - Detection sensitivity (0.0-1.0)
    pub fn new_with_engine(
        engine: WakeWordEngine,
        keywords: Vec<String>,
        sensitivity: f32,
    ) -> Result<Self> {
        match engine {
            #[cfg(feature = "porcupine")]
            WakeWordEngine::Porcupine => Self::new_porcupine(keywords, sensitivity),

            #[cfg(not(feature = "porcupine"))]
            WakeWordEngine::Porcupine => {
                warn!(
                    "Porcupine not available (feature disabled), falling back to energy detection"
                );
                Self::new_energy(keywords, sensitivity)
            }

            WakeWordEngine::Energy => Self::new_energy(keywords, sensitivity),

            WakeWordEngine::OpenWakeWord => {
                warn!("OpenWakeWord not yet implemented, falling back to energy detection");
                Self::new_energy(keywords, sensitivity)
            }
        }
    }

    /// Create energy-based detector
    fn new_energy(keywords: Vec<String>, sensitivity: f32) -> Result<Self> {
        info!("✅ Wake word detector initialized (energy mode)");
        info!("   Keywords: {:?}", keywords);
        info!("   Sensitivity: {}", sensitivity);

        Ok(Self {
            engine: WakeWordEngine::Energy,
            keywords,
            sensitivity,
            energy_threshold: 0.1 * (1.0 - sensitivity),
            #[cfg(feature = "porcupine")]
            porcupine: None,
        })
    }

    /// Create Porcupine-based detector
    #[cfg(feature = "porcupine")]
    fn new_porcupine(keywords: Vec<String>, sensitivity: f32) -> Result<Self> {
        info!("Initializing Porcupine wake word detector...");

        // Build Porcupine with keywords
        let mut builder = PorcupineBuilder::new();

        for keyword in &keywords {
            builder = builder.keyword(keyword, sensitivity);
        }

        let porcupine = builder
            .init()
            .map_err(|e| LunaError::WakeWord(format!("Porcupine init failed: {:?}", e)))?;

        info!("✅ Wake word detector initialized (Porcupine mode)");
        info!("   Keywords: {:?}", keywords);
        info!("   Sensitivity: {}", sensitivity);

        Ok(Self {
            engine: WakeWordEngine::Porcupine,
            keywords,
            sensitivity,
            energy_threshold: 0.0, // Not used in Porcupine mode
            porcupine: Some(porcupine),
        })
    }

    /// Create with default energy engine (backward compatibility)
    pub fn new(keywords: Vec<String>, sensitivity: f32) -> Result<Self> {
        Self::new_energy(keywords, sensitivity)
    }

    /// Detect wake word in audio buffer
    ///
    /// Returns `Some(keyword_index)` if wake word detected, `None` otherwise
    pub async fn detect(&self, audio_buffer: &[f32]) -> Result<Option<usize>> {
        if audio_buffer.is_empty() {
            return Ok(None);
        }

        match self.engine {
            #[cfg(feature = "porcupine")]
            WakeWordEngine::Porcupine => self.detect_porcupine(audio_buffer),

            WakeWordEngine::Energy | WakeWordEngine::OpenWakeWord => {
                self.detect_energy(audio_buffer)
            }

            #[cfg(all(not(feature = "porcupine"), not(any())))]
            _ => self.detect_energy(audio_buffer),
        }
    }

    /// Energy-based detection
    fn detect_energy(&self, audio_buffer: &[f32]) -> Result<Option<usize>> {
        let energy = self.calculate_energy(audio_buffer);

        if energy > self.energy_threshold {
            info!("🎤 Wake word detected (energy: {:.3})", energy);
            return Ok(Some(0)); // Return first keyword
        }

        Ok(None)
    }

    /// Porcupine-based detection
    #[cfg(feature = "porcupine")]
    fn detect_porcupine(&self, audio_buffer: &[f32]) -> Result<Option<usize>> {
        if let Some(ref porcupine) = self.porcupine {
            // Porcupine expects i16 samples
            let pcm: Vec<i16> = audio_buffer
                .iter()
                .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
                .collect();

            // Process frames
            let frame_length = porcupine.frame_length();

            for chunk in pcm.chunks(frame_length) {
                if chunk.len() < frame_length {
                    continue;
                }

                match porcupine.process(chunk) {
                    Ok(keyword_index) => {
                        if keyword_index >= 0 {
                            info!(
                                "🎤 Wake word detected: {}",
                                self.keywords[keyword_index as usize]
                            );
                            return Ok(Some(keyword_index as usize));
                        }
                    }
                    Err(e) => {
                        warn!("Porcupine processing error: {:?}", e);
                    }
                }
            }
        }

        Ok(None)
    }

    /// Calculate audio energy
    fn calculate_energy(&self, audio: &[f32]) -> f32 {
        if audio.is_empty() {
            return 0.0;
        }

        let sum: f32 = audio.iter().map(|&s| s * s).sum();
        (sum / audio.len() as f32).sqrt()
    }

    /// Get list of configured keywords
    pub fn keywords(&self) -> &[String] {
        &self.keywords
    }
}

// Implement WakeWordDetectorInterface trait
#[async_trait]
impl WakeWordDetectorInterface for WakeWordDetector {
    async fn detect(&self, audio: &[f32]) -> Result<Option<usize>> {
        // Call the existing detect method
        self.detect(audio).await
    }

    fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity;
        self.energy_threshold = 0.1 * (1.0 - sensitivity);
    }

    fn get_sensitivity(&self) -> f32 {
        self.sensitivity
    }
}

impl Default for WakeWordDetector {
    fn default() -> Self {
        Self::new(vec!["hey luna".to_string()], 0.5)
            .expect("Failed to create default wake word detector")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wake_word_detector_creation() {
        let detector =
            WakeWordDetector::new(vec!["hey luna".to_string(), "okay luna".to_string()], 0.5);
        assert!(detector.is_ok());
    }

    #[tokio::test]
    async fn test_wake_word_detection() {
        let detector = WakeWordDetector::new(vec!["hey luna".to_string()], 0.5).unwrap();

        // High energy audio should trigger detection
        let loud_audio = vec![0.5; 1000];
        let result = detector.detect(&loud_audio).await.unwrap();
        assert!(result.is_some());

        // Low energy audio should not trigger
        let quiet_audio = vec![0.01; 1000];
        let result = detector.detect(&quiet_audio).await.unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_energy_calculation() {
        let detector = WakeWordDetector::default();

        let audio = vec![0.5, -0.5, 0.3, -0.3];
        let energy = detector.calculate_energy(&audio);
        assert!(energy > 0.0 && energy < 1.0);
    }
}
