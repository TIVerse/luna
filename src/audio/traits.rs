//! Trait definitions for audio components
//!
//! Enables dependency injection and mock implementations for testing.

use crate::error::Result;
use async_channel::Receiver;
use async_trait::async_trait;

/// Audio capture interface for microphone input
/// Note: start/stop are synchronous to work with non-Send types like cpal::Stream
pub trait AudioCaptureInterface {
    /// Start capturing audio from the microphone
    fn start(&mut self) -> Result<()>;
    
    /// Stop capturing audio
    fn stop(&mut self) -> Result<()>;
    
    /// Get the audio stream receiver
    fn get_audio_stream(&self) -> Receiver<Vec<f32>>;
    
    /// Check if currently capturing
    fn is_active(&self) -> bool;
    
    /// Get the ring buffer for wake word detection
    /// 
    /// **DEPRECATED**: This method is legacy and only works with the old capture implementation.
    /// New implementations (capture_new.rs) use lock-free ring buffers accessed via direct methods.
    /// This method provides a default panic implementation for new implementations.
    #[deprecated(
        since = "0.1.0",
        note = "Use the concrete type's `get_ring_buffer()` method instead. This trait method is for backward compatibility only."
    )]
    fn get_ring_buffer(&self) -> &std::sync::Arc<std::sync::Mutex<super::capture::RingBuffer>> {
        panic!(
            "get_ring_buffer() trait method is deprecated. \
             For capture_new.rs, use the direct get_ring_buffer() method which returns &LockFreeRingBuffer. \
             This trait method only works with the legacy capture.rs implementation."
        )
    }
}

/// Wake word detection interface
#[async_trait]
pub trait WakeWordDetectorInterface: Send + Sync {
    /// Detect wake word in audio buffer
    /// Returns Some(index) if wake word detected, None otherwise
    async fn detect(&self, audio: &[f32]) -> Result<Option<usize>>;
    
    /// Set detection sensitivity (0.0 - 1.0)
    fn set_sensitivity(&mut self, sensitivity: f32);
    
    /// Get current sensitivity
    fn get_sensitivity(&self) -> f32;
}

/// Speech-to-text interface
#[async_trait]
pub trait SpeechToTextInterface: Send + Sync {
    /// Transcribe audio to text
    async fn transcribe(&self, audio: &[f32]) -> Result<String>;
    
    /// Check if running in simulated mode
    fn is_simulated(&self) -> bool;
    
    /// Get the sample rate used by this STT engine
    fn sample_rate(&self) -> u32;
}

/// Audio processor interface
pub trait AudioProcessorInterface: Send + Sync {
    /// Apply noise gate to remove low-level noise
    fn apply_noise_gate(&self, audio: &mut [f32]);
    
    /// Normalize audio levels
    fn normalize(&self, audio: &mut [f32]);
    
    /// Apply high-pass filter
    fn apply_high_pass_filter(&self, audio: &mut [f32], cutoff: f32, sample_rate: f32);
    
    /// Apply low-pass filter
    fn apply_low_pass_filter(&self, audio: &mut [f32], cutoff: f32, sample_rate: f32);
    
    /// Calculate signal-to-noise ratio
    fn calculate_snr(&self, audio: &[f32]) -> f32;
}
