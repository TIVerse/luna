//! Voice Activity Detection (VAD)
//!
//! Detects speech vs silence in audio streams using multiple engines:
//! - WebRTC VAD (production-grade)
//! - RMS-based (simple fallback)
//! - Silero VAD (future: ML-based)

use crate::error::{LunaError, Result};

#[cfg(feature = "webrtc-audio")]
use webrtc_vad::{Vad, VadMode, SampleRate};

/// VAD engine type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadEngine {
    /// WebRTC VAD (production-grade)
    WebRtc,
    /// RMS-based VAD (simple)
    Rms,
    /// Silero VAD (ML-based, future)
    Silero,
}

impl VadEngine {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "webrtc" => VadEngine::WebRtc,
            "silero" => VadEngine::Silero,
            _ => VadEngine::Rms,
        }
    }
}

/// Voice activity detector
pub struct VoiceActivityDetector {
    engine: VadEngine,
    #[cfg(feature = "webrtc-audio")]
    webrtc_vad: Option<Vad>,
    rms_threshold: f32,
    hangover_frames: usize,
    current_hangover: usize,
    was_speech: bool,
}

impl VoiceActivityDetector {
    /// Create a new VAD
    ///
    /// # Arguments
    /// * `engine` - VAD engine to use
    /// * `aggressiveness` - Aggressiveness level (0-3 for WebRTC, threshold for RMS)
    /// * `sample_rate` - Sample rate in Hz (must be 8000, 16000, 32000, or 48000 for WebRTC)
    pub fn new(engine: VadEngine, aggressiveness: u8, sample_rate: u32) -> Result<Self> {
        #[cfg(feature = "webrtc-audio")]
        let webrtc_vad = if engine == VadEngine::WebRtc {
            let mode = match aggressiveness {
                0 => VadMode::Quality,
                1 => VadMode::LowBitrate,
                2 => VadMode::Aggressive,
                _ => VadMode::VeryAggressive,
            };
            
            let mut vad = Vad::new();
            vad.set_mode(mode);
            
            // Convert sample rate to WebRTC VAD enum
            let vad_sample_rate = match sample_rate {
                8000 => SampleRate::Rate8kHz,
                16000 => SampleRate::Rate16kHz,
                32000 => SampleRate::Rate32kHz,
                48000 => SampleRate::Rate48kHz,
                _ => return Err(LunaError::Audio(
                    format!("Unsupported sample rate for WebRTC VAD: {}. Use 8000, 16000, 32000, or 48000", sample_rate)
                )),
            };
            vad.set_sample_rate(vad_sample_rate);
            
            Some(vad)
        } else {
            None
        };
        
        #[cfg(not(feature = "webrtc-audio"))]
        let webrtc_vad: Option<i32> = None;  // Placeholder type when feature disabled
        
        let rms_threshold = match aggressiveness {
            0 => 0.05,
            1 => 0.04,
            2 => 0.03,
            _ => 0.02,
        };
        
        // Hangover: keep speech active for N frames after speech ends
        // Prevents choppy detection on brief pauses
        let hangover_frames = 10; // ~100ms at 10ms frames
        
        Ok(Self {
            engine,
            #[cfg(feature = "webrtc-audio")]
            webrtc_vad,
            rms_threshold,
            hangover_frames,
            current_hangover: 0,
            was_speech: false,
        })
    }
    
    /// Detect speech in audio frame
    ///
    /// # Arguments
    /// * `frame` - Audio frame (must be 10, 20, or 30ms of audio for WebRTC)
    ///
    /// # Returns
    /// `true` if speech detected, `false` otherwise
    pub fn is_speech(&mut self, frame: &[f32]) -> Result<bool> {
        if frame.is_empty() {
            return Ok(false);
        }
        
        let is_speech = match self.engine {
            #[cfg(feature = "webrtc-audio")]
            VadEngine::WebRtc => {
                if let Some(ref mut vad) = self.webrtc_vad {
                    // WebRTC VAD expects i16 samples
                    let pcm: Vec<i16> = frame
                        .iter()
                        .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
                        .collect();
                    
                    vad.is_voice_segment(&pcm)
                        .map_err(|e| LunaError::Audio(format!("WebRTC VAD error: {:?}", e)))?
                } else {
                    self.detect_rms(frame)
                }
            }
            #[cfg(not(feature = "webrtc-audio"))]
            VadEngine::WebRtc => self.detect_rms(frame),
            
            VadEngine::Rms => self.detect_rms(frame),
            
            VadEngine::Silero => {
                // TODO: Implement Silero VAD
                self.detect_rms(frame)
            }
        };
        
        // Hangover logic: keep speech active for a few frames after it stops
        if is_speech {
            self.current_hangover = self.hangover_frames;
            self.was_speech = true;
            Ok(true)
        } else if self.current_hangover > 0 {
            self.current_hangover -= 1;
            Ok(true)
        } else {
            self.was_speech = false;
            Ok(false)
        }
    }
    
    /// RMS-based speech detection (fallback)
    fn detect_rms(&self, frame: &[f32]) -> bool {
        let rms = calculate_rms(frame);
        rms > self.rms_threshold
    }
    
    /// Check if we just transitioned from silence to speech
    pub fn speech_started(&self) -> bool {
        self.was_speech && self.current_hangover == self.hangover_frames
    }
    
    /// Check if we just transitioned from speech to silence
    pub fn speech_ended(&self) -> bool {
        !self.was_speech && self.current_hangover == 0
    }
    
    /// Reset VAD state
    pub fn reset(&mut self) {
        self.current_hangover = 0;
        self.was_speech = false;
        
        #[cfg(feature = "webrtc-audio")]
        if let Some(ref mut vad) = self.webrtc_vad {
            let _ = vad.reset();
        }
    }
}

/// Calculate RMS (Root Mean Square) of audio samples
fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    
    let sum: f32 = samples.iter().map(|&s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vad_creation() {
        let vad = VoiceActivityDetector::new(VadEngine::Rms, 2, 16000);
        assert!(vad.is_ok());
    }

    #[test]
    fn test_rms_detection() {
        let mut vad = VoiceActivityDetector::new(VadEngine::Rms, 2, 16000).unwrap();
        
        // Loud audio should be detected as speech
        let loud_frame = vec![0.5; 160]; // 10ms at 16kHz
        assert!(vad.is_speech(&loud_frame).unwrap());
        
        // Quiet audio should not
        let quiet_frame = vec![0.01; 160];
        // May still be true due to hangover, so let's reset first
        vad.reset();
        assert!(!vad.is_speech(&quiet_frame).unwrap());
    }

    #[test]
    fn test_hangover() {
        let mut vad = VoiceActivityDetector::new(VadEngine::Rms, 2, 16000).unwrap();
        
        // Speech detected
        let loud_frame = vec![0.5; 160];
        assert!(vad.is_speech(&loud_frame).unwrap());
        
        // Silence, but hangover keeps it active
        let quiet_frame = vec![0.01; 160];
        assert!(vad.is_speech(&quiet_frame).unwrap());
        
        // After enough frames, should become silence
        for _ in 0..15 {
            let _ = vad.is_speech(&quiet_frame);
        }
        assert!(!vad.is_speech(&quiet_frame).unwrap());
    }

    #[test]
    fn test_engine_from_str() {
        assert_eq!(VadEngine::from_str("webrtc"), VadEngine::WebRtc);
        assert_eq!(VadEngine::from_str("rms"), VadEngine::Rms);
        assert_eq!(VadEngine::from_str("silero"), VadEngine::Silero);
        assert_eq!(VadEngine::from_str("unknown"), VadEngine::Rms);
    }

    #[test]
    fn test_rms_calculation() {
        let samples = vec![0.5, -0.5, 0.3, -0.3];
        let rms = calculate_rms(&samples);
        assert!(rms > 0.0 && rms < 1.0);
        
        let silent = vec![0.0; 100];
        assert_eq!(calculate_rms(&silent), 0.0);
    }

    #[cfg(feature = "webrtc-audio")]
    #[test]
    fn test_webrtc_vad() {
        let mut vad = VoiceActivityDetector::new(VadEngine::WebRtc, 2, 16000).unwrap();
        
        // Create a 10ms frame at 16kHz (160 samples)
        let frame = vec![0.0; 160];
        let result = vad.is_speech(&frame);
        assert!(result.is_ok());
    }
}
