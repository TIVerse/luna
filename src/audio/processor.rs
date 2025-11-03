//! Audio preprocessing
//!
//! Noise reduction, automatic gain control, and filtering for better speech recognition.

use super::traits::AudioProcessorInterface;

/// Audio processor for preprocessing audio before speech recognition
pub struct AudioProcessor {
    noise_gate_threshold: f32,
    gain: f32,
}

impl AudioProcessor {
    /// Create a new audio processor
    ///
    /// # Arguments
    /// * `noise_gate_threshold` - Threshold below which audio is silenced (0.0-1.0)
    /// * `gain` - Amplification factor (typically 1.0-2.0)
    pub fn new(noise_gate_threshold: f32, gain: f32) -> Self {
        Self {
            noise_gate_threshold,
            gain,
        }
    }

    /// Process audio samples with noise reduction and normalization
    pub fn process(&self, audio: &[f32]) -> Vec<f32> {
        let mut processed = audio.to_vec();

        // Apply noise gate
        self.apply_noise_gate(&mut processed);

        // Normalize audio
        self.normalize(&mut processed);

        // Apply gain
        for sample in &mut processed {
            *sample *= self.gain;
            *sample = sample.clamp(-1.0, 1.0);
        }

        processed
    }

    /// Apply noise gate to remove low-level noise
    fn apply_noise_gate(&self, audio: &mut [f32]) {
        for sample in audio.iter_mut() {
            if sample.abs() < self.noise_gate_threshold {
                *sample = 0.0;
            }
        }
    }

    /// Normalize audio to consistent volume
    fn normalize(&self, audio: &mut [f32]) {
        let max = audio.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

        if max > 0.0 && max < 1.0 {
            let factor = 0.95 / max;
            for sample in audio.iter_mut() {
                *sample *= factor;
            }
        }
    }

    /// Apply high-pass filter to remove low-frequency noise
    pub fn apply_high_pass_filter(&self, audio: &mut [f32], cutoff_hz: f32, sample_rate: f32) {
        // Simple high-pass filter to remove low-frequency noise
        let rc = 1.0 / (cutoff_hz * 2.0 * std::f32::consts::PI);
        let dt = 1.0 / sample_rate;
        let alpha = rc / (rc + dt);

        let mut prev_input = 0.0;
        let mut prev_output = 0.0;

        for sample in audio.iter_mut() {
            let output = alpha * (prev_output + *sample - prev_input);
            prev_input = *sample;
            prev_output = output;
            *sample = output;
        }
    }

    /// Apply low-pass filter to remove high-frequency noise
    pub fn apply_low_pass_filter(&self, audio: &mut [f32], cutoff_hz: f32, sample_rate: f32) {
        let rc = 1.0 / (cutoff_hz * 2.0 * std::f32::consts::PI);
        let dt = 1.0 / sample_rate;
        let alpha = dt / (rc + dt);

        let mut prev_output = 0.0;

        for sample in audio.iter_mut() {
            let output = prev_output + alpha * (*sample - prev_output);
            prev_output = output;
            *sample = output;
        }
    }

    /// Calculate signal-to-noise ratio
    pub fn calculate_snr(&self, audio: &[f32]) -> f32 {
        if audio.is_empty() {
            return 0.0;
        }

        // Simple SNR estimation: compare signal power to noise floor
        let signal_power: f32 = audio.iter().map(|&s| s * s).sum::<f32>() / audio.len() as f32;

        // Estimate noise as samples below threshold
        let noise_samples: Vec<f32> = audio
            .iter()
            .filter(|&&s| s.abs() < self.noise_gate_threshold)
            .copied()
            .collect();

        if noise_samples.is_empty() {
            return 100.0; // Very high SNR
        }

        let noise_power: f32 =
            noise_samples.iter().map(|&s| s * s).sum::<f32>() / noise_samples.len() as f32;

        if noise_power == 0.0 {
            return 100.0;
        }

        10.0 * (signal_power / noise_power).log10()
    }
}

impl Default for AudioProcessor {
    fn default() -> Self {
        Self::new(0.01, 1.0)
    }
}

// Implement AudioProcessorInterface trait
impl AudioProcessorInterface for AudioProcessor {
    fn apply_noise_gate(&self, audio: &mut [f32]) {
        // Call the existing private method
        self.apply_noise_gate(audio)
    }

    fn normalize(&self, audio: &mut [f32]) {
        // Call the existing private method
        self.normalize(audio)
    }

    fn apply_high_pass_filter(&self, audio: &mut [f32], cutoff: f32, sample_rate: f32) {
        // Call the existing public method
        self.apply_high_pass_filter(audio, cutoff, sample_rate)
    }

    fn apply_low_pass_filter(&self, audio: &mut [f32], cutoff: f32, sample_rate: f32) {
        // Call the existing public method
        self.apply_low_pass_filter(audio, cutoff, sample_rate)
    }

    fn calculate_snr(&self, audio: &[f32]) -> f32 {
        // Call the existing public method
        self.calculate_snr(audio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_gate() {
        let processor = AudioProcessor::new(0.1, 1.0);
        let audio = vec![0.05, 0.15, -0.05, 0.3, -0.08];
        let processed = processor.process(&audio);

        // Samples below threshold should be zeroed
        assert_eq!(processed[0], 0.0);
        assert_eq!(processed[2], 0.0);
        assert_eq!(processed[4], 0.0);

        // Samples above threshold should be preserved (and possibly normalized)
        assert!(processed[1].abs() > 0.1);
        assert!(processed[3].abs() > 0.1);
    }

    #[test]
    fn test_normalization() {
        let processor = AudioProcessor::new(0.0, 1.0);
        let audio = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let processed = processor.process(&audio);

        // Find max value
        let max = processed.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

        // Should be normalized close to 0.95
        assert!(max > 0.9 && max <= 1.0);
    }

    #[test]
    fn test_high_pass_filter() {
        let processor = AudioProcessor::new(0.0, 1.0);
        let mut audio = vec![0.5; 100]; // DC signal

        processor.apply_high_pass_filter(&mut audio, 100.0, 48000.0);

        // High-pass should reduce DC component
        let final_avg: f32 = audio.iter().sum::<f32>() / audio.len() as f32;
        assert!(final_avg.abs() < 0.5);
    }

    #[test]
    fn test_snr_calculation() {
        let processor = AudioProcessor::new(0.01, 1.0);

        // High signal
        let good_audio = vec![0.5, 0.6, 0.4, 0.7, 0.5];
        let snr_good = processor.calculate_snr(&good_audio);
        assert!(snr_good > 10.0);

        // Mostly noise
        let noisy_audio = vec![0.005, 0.008, 0.003, 0.007, 0.004];
        let snr_noisy = processor.calculate_snr(&noisy_audio);
        assert!(snr_noisy < 10.0);
    }

    #[test]
    fn test_default_processor() {
        let processor = AudioProcessor::default();
        assert_eq!(processor.noise_gate_threshold, 0.01);
        assert_eq!(processor.gain, 1.0);
    }
}
