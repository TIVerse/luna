//! Digital Signal Processing (DSP) for audio
//!
//! High-quality audio processing including:
//! - Resampling (using rubato)
//! - Noise suppression (WebRTC NS / RNNoise)
//! - Automatic Gain Control (AGC)
//! - Acoustic Echo Cancellation (AEC)

use crate::error::{LunaError, Result};
use rubato::{FftFixedIn, Resampler};

/// High-quality audio resampler using FFT-based method
pub struct AudioResampler {
    resampler: Option<FftFixedIn<f32>>,
    from_rate: u32,
    to_rate: u32,
    chunk_size: usize,
}

impl AudioResampler {
    /// Create a new resampler
    ///
    /// # Arguments
    /// * `from_rate` - Source sample rate
    /// * `to_rate` - Target sample rate
    /// * `chunk_size` - Processing chunk size
    pub fn new(from_rate: u32, to_rate: u32, chunk_size: usize) -> Result<Self> {
        if from_rate == to_rate {
            // No resampling needed
            return Ok(Self {
                resampler: None,
                from_rate,
                to_rate,
                chunk_size,
            });
        }
        
        let resampler = FftFixedIn::<f32>::new(
            from_rate as usize,
            to_rate as usize,
            chunk_size,
            2, // sub_chunks
            1, // channels (mono)
        ).map_err(|e| LunaError::Audio(format!("Failed to create resampler: {}", e)))?;
        
        Ok(Self {
            resampler: Some(resampler),
            from_rate,
            to_rate,
            chunk_size,
        })
    }
    
    /// Resample audio
    ///
    /// # Arguments
    /// * `input` - Input audio samples
    ///
    /// # Returns
    /// Resampled audio
    pub fn process(&mut self, input: &[f32]) -> Result<Vec<f32>> {
        if let Some(ref mut resampler) = self.resampler {
            // Process in chunks
            let mut output = Vec::new();
            let mut input_pos = 0;
            
            while input_pos < input.len() {
                let chunk_end = (input_pos + self.chunk_size).min(input.len());
                let chunk = &input[input_pos..chunk_end];
                
                // Pad if needed
                let mut chunk_vec = chunk.to_vec();
                if chunk_vec.len() < self.chunk_size {
                    chunk_vec.resize(self.chunk_size, 0.0);
                }
                
                let waves_in = vec![chunk_vec];
                let waves_out = resampler.process(&waves_in, None)
                    .map_err(|e| LunaError::Audio(format!("Resampling failed: {}", e)))?;
                
                if let Some(channel) = waves_out.first() {
                    output.extend_from_slice(channel);
                }
                
                input_pos = chunk_end;
            }
            
            Ok(output)
        } else {
            // No resampling needed
            Ok(input.to_vec())
        }
    }
    
    /// Get output length for given input length
    pub fn output_len(&self, input_len: usize) -> usize {
        if self.from_rate == self.to_rate {
            input_len
        } else {
            (input_len as f64 * self.to_rate as f64 / self.from_rate as f64) as usize
        }
    }
}

/// Automatic Gain Control (AGC)
pub struct AutomaticGainControl {
    target_level: f32,
    max_gain: f32,
    attack_coeff: f32,
    release_coeff: f32,
    current_gain: f32,
}

impl AutomaticGainControl {
    /// Create a new AGC
    ///
    /// # Arguments
    /// * `target_level` - Target RMS level (0.0 - 1.0)
    /// * `max_gain` - Maximum gain multiplier
    pub fn new(target_level: f32, max_gain: f32) -> Self {
        Self {
            target_level,
            max_gain,
            attack_coeff: 0.1,
            release_coeff: 0.01,
            current_gain: 1.0,
        }
    }
    
    /// Process audio with AGC
    ///
    /// # Arguments
    /// * `audio` - Audio samples (modified in-place)
    pub fn process(&mut self, audio: &mut [f32]) {
        if audio.is_empty() {
            return;
        }
        
        // Calculate current RMS
        let rms = calculate_rms(audio);
        
        if rms > 0.0 {
            // Calculate desired gain
            let desired_gain = (self.target_level / rms).min(self.max_gain);
            
            // Smooth gain changes
            if desired_gain > self.current_gain {
                // Attack (increase gain)
                self.current_gain += (desired_gain - self.current_gain) * self.attack_coeff;
            } else {
                // Release (decrease gain)
                self.current_gain += (desired_gain - self.current_gain) * self.release_coeff;
            }
            
            // Apply gain
            for sample in audio.iter_mut() {
                *sample = (*sample * self.current_gain).clamp(-1.0, 1.0);
            }
        }
    }
    
    /// Get current gain value
    pub fn current_gain(&self) -> f32 {
        self.current_gain
    }
    
    /// Reset AGC state
    pub fn reset(&mut self) {
        self.current_gain = 1.0;
    }
}

/// Noise suppression
pub struct NoiseSuppressor {
    threshold: f32,
    enabled: bool,
}

impl NoiseSuppressor {
    /// Create a new noise suppressor
    ///
    /// # Arguments
    /// * `threshold` - Noise gate threshold
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            enabled: true,
        }
    }
    
    /// Process audio with noise suppression
    ///
    /// # Arguments
    /// * `audio` - Audio samples (modified in-place)
    pub fn process(&self, audio: &mut [f32]) {
        if !self.enabled {
            return;
        }
        
        // Simple noise gate for now
        // TODO: Integrate WebRTC NS or RNNoise for production
        for sample in audio.iter_mut() {
            if sample.abs() < self.threshold {
                *sample = 0.0;
            }
        }
    }
    
    /// Enable/disable noise suppression
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Complete DSP processor chain
pub struct DspProcessor {
    resampler: Option<AudioResampler>,
    agc: Option<AutomaticGainControl>,
    noise_suppressor: Option<NoiseSuppressor>,
}

impl DspProcessor {
    /// Create a new DSP processor
    pub fn new() -> Self {
        Self {
            resampler: None,
            agc: None,
            noise_suppressor: None,
        }
    }
    
    /// Add resampler to the chain
    pub fn with_resampler(mut self, from_rate: u32, to_rate: u32, chunk_size: usize) -> Result<Self> {
        self.resampler = Some(AudioResampler::new(from_rate, to_rate, chunk_size)?);
        Ok(self)
    }
    
    /// Add AGC to the chain
    pub fn with_agc(mut self, target_level: f32, max_gain: f32) -> Self {
        self.agc = Some(AutomaticGainControl::new(target_level, max_gain));
        self
    }
    
    /// Add noise suppressor to the chain
    pub fn with_noise_suppression(mut self, threshold: f32) -> Self {
        self.noise_suppressor = Some(NoiseSuppressor::new(threshold));
        self
    }
    
    /// Process audio through the DSP chain
    ///
    /// # Arguments
    /// * `audio` - Input audio samples
    ///
    /// # Returns
    /// Processed audio (may be resampled to different length)
    pub fn process(&mut self, audio: &[f32]) -> Result<Vec<f32>> {
        let mut output = audio.to_vec();
        
        // Apply noise suppression first
        if let Some(ref ns) = self.noise_suppressor {
            ns.process(&mut output);
        }
        
        // Then AGC
        if let Some(ref mut agc) = self.agc {
            agc.process(&mut output);
        }
        
        // Finally resample
        if let Some(ref mut resampler) = self.resampler {
            output = resampler.process(&output)?;
        }
        
        Ok(output)
    }
}

impl Default for DspProcessor {
    fn default() -> Self {
        Self::new()
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
    fn test_agc_creation() {
        let agc = AutomaticGainControl::new(0.5, 3.0);
        assert_eq!(agc.current_gain(), 1.0);
    }

    #[test]
    fn test_agc_amplification() {
        let mut agc = AutomaticGainControl::new(0.5, 3.0);
        
        // Quiet audio should be amplified
        let mut audio = vec![0.1; 160];
        agc.process(&mut audio);
        
        // Gain should have increased
        assert!(agc.current_gain() > 1.0);
    }

    #[test]
    fn test_noise_suppressor() {
        let ns = NoiseSuppressor::new(0.05);
        
        let mut audio = vec![0.01, 0.5, 0.02, 0.8, 0.03];
        ns.process(&mut audio);
        
        // Quiet samples should be zeroed
        assert_eq!(audio[0], 0.0);
        assert_eq!(audio[2], 0.0);
        
        // Loud samples should remain
        assert!(audio[1] > 0.0);
        assert!(audio[3] > 0.0);
    }

    #[test]
    fn test_resampler_passthrough() {
        // Same rate = passthrough
        let mut resampler = AudioResampler::new(16000, 16000, 160).unwrap();
        
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let output = resampler.process(&input).unwrap();
        
        assert_eq!(output, input);
    }

    #[test]
    fn test_resampler_downsampling() {
        // 48kHz -> 16kHz should reduce length by 3x
        let mut resampler = AudioResampler::new(48000, 16000, 480).unwrap();
        
        let input = vec![1.0; 480]; // 10ms at 48kHz
        let output = resampler.process(&input).unwrap();
        
        // Should be approximately 160 samples (10ms at 16kHz)
        assert!((output.len() as i32 - 160).abs() < 5);
    }

    #[test]
    fn test_dsp_processor_chain() {
        let processor = DspProcessor::new()
            .with_agc(0.5, 3.0)
            .with_noise_suppression(0.05);
        
        assert!(processor.agc.is_some());
        assert!(processor.noise_suppressor.is_some());
    }

    #[test]
    fn test_dsp_processor_process() {
        let mut processor = DspProcessor::new()
            .with_agc(0.5, 3.0)
            .with_noise_suppression(0.05);
        
        let audio = vec![0.01, 0.5, 0.02, 0.8];
        let result = processor.process(&audio);
        
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.len(), audio.len());
    }
}
