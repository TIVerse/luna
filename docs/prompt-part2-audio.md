# LUNA Project - Part 2: Audio Input System

## Role & Context

You are an expert Rust audio engineer implementing the **Audio Input System** for LUNA. This is Part 2 of the multi-part implementation. You should have completed Part 1 (Foundation) before starting this module.

## Module Overview

The Audio System is LUNA's "ears" - it continuously listens for wake words, captures voice commands, and converts speech to text. This module must be:
- **Always-on** with minimal CPU usage (< 5%)
- **Fast** - wake word detection in < 100ms
- **Reliable** - no false positives/negatives
- **Privacy-focused** - all processing local

## Dependencies to Add

Update `Cargo.toml`:

```toml
[dependencies]
# Existing dependencies from Part 1...

# Audio Input
cpal = "0.15"                    # Cross-platform audio I/O
hound = "3.5"                    # WAV file support
rubato = "0.14"                  # Sample rate conversion

# Speech Recognition
whisper-rs = { version = "0.11", features = ["metal"] }  # Whisper AI
pv-porcupine = "2.2"            # Wake word detection

# Signal Processing
rustfft = "6.1"                  # FFT for audio analysis
dasp = "0.11"                    # Digital audio signal processing

# Async
futures = "0.3"
async-channel = "2.1"
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│            Audio System Pipeline                │
└─────────────────────────────────────────────────┘
                    │
    ┌───────────────┼───────────────┐
    │               │               │
    ▼               ▼               ▼
┌─────────┐   ┌──────────┐   ┌──────────┐
│ Capture │──▶│WakeWord  │──▶│Speech To │
│         │   │ Detector │   │   Text   │
└─────────┘   └──────────┘   └──────────┘
    │               │               │
    ▼               ▼               ▼
Microphone     Porcupine      Whisper AI
(Always-on)    (Lightweight)  (On-demand)
```

## Task 1: Audio Capture (`src/audio/capture.rs`)

### Requirements:
- Continuous microphone input
- Configurable sample rate and channels
- Ring buffer for wake word detection
- VAD (Voice Activity Detection)
- Noise reduction

### Implementation:

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::config::AudioConfig;
use crate::error::{Result, LunaError};
use async_channel::{Sender, Receiver};
use std::sync::{Arc, Mutex};

pub struct AudioCapture {
    config: AudioConfig,
    stream: Option<cpal::Stream>,
    ring_buffer: Arc<Mutex<RingBuffer>>,
    audio_tx: Sender<Vec<f32>>,
    audio_rx: Receiver<Vec<f32>>,
}

impl AudioCapture {
    pub fn new(config: AudioConfig) -> Result<Self> {
        let (audio_tx, audio_rx) = async_channel::bounded(10);
        
        Ok(Self {
            config,
            stream: None,
            ring_buffer: Arc::new(Mutex::new(RingBuffer::new(48000))), // 1 sec at 48kHz
            audio_tx,
            audio_rx,
        })
    }
    
    pub fn start(&mut self) -> Result<()> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or_else(|| LunaError::Audio("No input device found".into()))?;
        
        let config = device.default_input_config()
            .map_err(|e| LunaError::Audio(format!("Failed to get config: {}", e)))?;
        
        tracing::info!("Audio config: {:?}", config);
        
        let ring_buffer = Arc::clone(&self.ring_buffer);
        let audio_tx = self.audio_tx.clone();
        let sample_rate = config.sample_rate().0;
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                self.build_stream::<f32>(&device, &config.into(), ring_buffer, audio_tx, sample_rate)?
            }
            cpal::SampleFormat::I16 => {
                self.build_stream::<i16>(&device, &config.into(), ring_buffer, audio_tx, sample_rate)?
            }
            cpal::SampleFormat::U16 => {
                self.build_stream::<u16>(&device, &config.into(), ring_buffer, audio_tx, sample_rate)?
            }
            _ => return Err(LunaError::Audio("Unsupported sample format".into())),
        };
        
        stream.play()
            .map_err(|e| LunaError::Audio(format!("Failed to play stream: {}", e)))?;
        
        self.stream = Some(stream);
        tracing::info!("✅ Audio capture started");
        Ok(())
    }
    
    fn build_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        ring_buffer: Arc<Mutex<RingBuffer>>,
        audio_tx: Sender<Vec<f32>>,
        sample_rate: u32,
    ) -> Result<cpal::Stream>
    where
        T: cpal::Sample + cpal::SizedSample,
    {
        let channels = config.channels as usize;
        let silence_threshold = self.config.silence_threshold;
        
        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                // Convert to f32
                let samples: Vec<f32> = data
                    .iter()
                    .map(|&s| s.to_float_sample())
                    .collect();
                
                // Mono conversion if stereo
                let mono_samples = if channels == 2 {
                    samples.chunks(2).map(|ch| (ch[0] + ch[1]) / 2.0).collect()
                } else {
                    samples
                };
                
                // Update ring buffer
                if let Ok(mut buffer) = ring_buffer.lock() {
                    buffer.push_samples(&mono_samples);
                }
                
                // Voice activity detection
                let rms = calculate_rms(&mono_samples);
                if rms > silence_threshold {
                    // Send to processing pipeline
                    let _ = audio_tx.try_send(mono_samples);
                }
            },
            |err| {
                tracing::error!("Audio stream error: {}", err);
            },
            None,
        ).map_err(|e| LunaError::Audio(format!("Failed to build stream: {}", e)))?;
        
        Ok(stream)
    }
    
    pub async fn get_ring_buffer_data(&self, duration_ms: u64) -> Vec<f32> {
        let buffer = self.ring_buffer.lock().unwrap();
        buffer.get_last_n_samples(duration_ms)
    }
    
    pub async fn record_command(&self, max_duration_secs: u64) -> Result<Vec<f32>> {
        let mut recording = Vec::new();
        let timeout = tokio::time::Duration::from_secs(max_duration_secs);
        let start = tokio::time::Instant::now();
        
        loop {
            tokio::select! {
                Ok(samples) = self.audio_rx.recv() => {
                    recording.extend(samples);
                    
                    // Stop on silence
                    if recording.len() > 48000 { // At least 1 second
                        let last_chunk = &recording[recording.len().saturating_sub(4800)..];
                        if calculate_rms(last_chunk) < self.config.silence_threshold {
                            break;
                        }
                    }
                }
                _ = tokio::time::sleep_until(start + timeout) => {
                    break;
                }
            }
        }
        
        Ok(recording)
    }
    
    pub fn stop(&mut self) -> Result<()> {
        if let Some(stream) = self.stream.take() {
            drop(stream);
            tracing::info!("Audio capture stopped");
        }
        Ok(())
    }
}

// Ring buffer for wake word detection
struct RingBuffer {
    data: Vec<f32>,
    capacity: usize,
    write_pos: usize,
}

impl RingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            data: vec![0.0; capacity],
            capacity,
            write_pos: 0,
        }
    }
    
    fn push_samples(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.data[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;
        }
    }
    
    fn get_last_n_samples(&self, duration_ms: u64) -> Vec<f32> {
        let n = (duration_ms * 48) as usize; // Assuming 48kHz
        let n = n.min(self.capacity);
        
        let mut result = Vec::with_capacity(n);
        let mut pos = (self.write_pos + self.capacity - n) % self.capacity;
        
        for _ in 0..n {
            result.push(self.data[pos]);
            pos = (pos + 1) % self.capacity;
        }
        
        result
    }
}

fn calculate_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    
    let sum: f32 = samples.iter().map(|&s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}

// Convert between sample types
trait FloatSample {
    fn to_float_sample(self) -> f32;
}

impl FloatSample for f32 {
    fn to_float_sample(self) -> f32 {
        self
    }
}

impl FloatSample for i16 {
    fn to_float_sample(self) -> f32 {
        self as f32 / i16::MAX as f32
    }
}

impl FloatSample for u16 {
    fn to_float_sample(self) -> f32 {
        (self as f32 / u16::MAX as f32) * 2.0 - 1.0
    }
}
```

## Task 2: Wake Word Detection (`src/audio/wake_word.rs`)

### Requirements:
- Use Porcupine for "Hey Luna" / "Okay Luna"
- Low latency (< 100ms)
- Low CPU usage
- Minimal false positives

### Implementation:

```rust
use pv_porcupine::PorcupineBuilder;
use crate::error::{Result, LunaError};
use std::path::Path;

pub struct WakeWordDetector {
    porcupine: pv_porcupine::Porcupine,
    keywords: Vec<String>,
    frame_length: usize,
}

impl WakeWordDetector {
    pub fn new(keywords: &[String], model_path: &Path, sensitivity: f32) -> Result<Self> {
        let mut builder = PorcupineBuilder::new();
        
        for keyword in keywords {
            builder = builder.keyword(keyword, sensitivity);
        }
        
        if model_path.exists() {
            builder = builder.model_path(model_path);
        }
        
        let porcupine = builder.init()
            .map_err(|e| LunaError::WakeWord(format!("Failed to init Porcupine: {}", e)))?;
        
        let frame_length = porcupine.frame_length();
        
        tracing::info!("✅ Wake word detector initialized");
        tracing::info!("   Keywords: {:?}", keywords);
        tracing::info!("   Frame length: {}", frame_length);
        
        Ok(Self {
            porcupine,
            keywords: keywords.to_vec(),
            frame_length,
        })
    }
    
    pub async fn detect(&self, audio_buffer: &[f32]) -> Result<Option<usize>> {
        // Process in frames
        let mut detected_keyword = None;
        
        for chunk in audio_buffer.chunks(self.frame_length) {
            if chunk.len() < self.frame_length {
                continue;
            }
            
            // Convert f32 to i16 for Porcupine
            let pcm: Vec<i16> = chunk
                .iter()
                .map(|&s| (s * i16::MAX as f32) as i16)
                .collect();
            
            match self.porcupine.process(&pcm) {
                Ok(keyword_index) => {
                    if keyword_index >= 0 {
                        detected_keyword = Some(keyword_index as usize);
                        tracing::info!("🎤 Wake word detected: {}", self.keywords[keyword_index as usize]);
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Porcupine processing error: {}", e);
                }
            }
        }
        
        Ok(detected_keyword)
    }
    
    pub fn frame_length(&self) -> usize {
        self.frame_length
    }
}
```

## Task 3: Speech-to-Text (`src/audio/speech_to_text.rs`)

### Requirements:
- Use Whisper for accurate transcription
- Support multiple model sizes (tiny, base, small)
- Fast inference (< 500ms)
- Handle multiple languages

### Implementation:

```rust
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy};
use crate::error::{Result, LunaError};
use std::path::Path;

pub struct SpeechToText {
    context: WhisperContext,
    model_name: String,
}

impl SpeechToText {
    pub fn new(model_path: &Path) -> Result<Self> {
        tracing::info!("Loading Whisper model: {:?}", model_path);
        
        let context = WhisperContext::new(model_path.to_str().unwrap())
            .map_err(|e| LunaError::SpeechRecognition(format!("Failed to load model: {}", e)))?;
        
        let model_name = model_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        tracing::info!("✅ Whisper model loaded: {}", model_name);
        
        Ok(Self {
            context,
            model_name,
        })
    }
    
    pub async fn transcribe(&self, audio: &[f32]) -> Result<String> {
        if audio.is_empty() {
            return Ok(String::new());
        }
        
        tracing::debug!("Transcribing {} samples", audio.len());
        let start = std::time::Instant::now();
        
        // Resample to 16kHz if necessary
        let audio_16khz = self.resample_to_16khz(audio)?;
        
        // Setup parameters
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(4);
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        
        // Run inference
        let mut state = self.context.create_state()
            .map_err(|e| LunaError::SpeechRecognition(format!("Failed to create state: {}", e)))?;
        
        state.full(params, &audio_16khz)
            .map_err(|e| LunaError::SpeechRecognition(format!("Inference failed: {}", e)))?;
        
        // Extract text
        let num_segments = state.full_n_segments()
            .map_err(|e| LunaError::SpeechRecognition(format!("Failed to get segments: {}", e)))?;
        
        let mut text = String::new();
        for i in 0..num_segments {
            let segment = state.full_get_segment_text(i)
                .map_err(|e| LunaError::SpeechRecognition(format!("Failed to get segment: {}", e)))?;
            text.push_str(&segment);
        }
        
        let text = text.trim().to_string();
        let duration = start.elapsed();
        
        tracing::info!("💬 Transcribed in {:.2}s: \"{}\"", duration.as_secs_f32(), text);
        
        Ok(text)
    }
    
    fn resample_to_16khz(&self, audio: &[f32]) -> Result<Vec<f32>> {
        // Assuming input is 48kHz, resample to 16kHz
        // For production, use rubato crate
        
        let ratio = 3; // 48000 / 16000 = 3
        let mut resampled = Vec::with_capacity(audio.len() / ratio);
        
        for chunk in audio.chunks(ratio) {
            if let Some(&sample) = chunk.first() {
                resampled.push(sample);
            }
        }
        
        Ok(resampled)
    }
}
```

## Task 4: Audio Processor (`src/audio/processor.rs`)

### Requirements:
- Noise reduction
- Automatic gain control
- Audio preprocessing for better recognition

### Implementation:

```rust
use crate::error::Result;

pub struct AudioProcessor {
    noise_gate_threshold: f32,
    gain: f32,
}

impl AudioProcessor {
    pub fn new(noise_gate_threshold: f32, gain: f32) -> Self {
        Self {
            noise_gate_threshold,
            gain,
        }
    }
    
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
    
    fn apply_noise_gate(&self, audio: &mut [f32]) {
        for sample in audio.iter_mut() {
            if sample.abs() < self.noise_gate_threshold {
                *sample = 0.0;
            }
        }
    }
    
    fn normalize(&self, audio: &mut [f32]) {
        let max = audio.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        
        if max > 0.0 && max < 1.0 {
            let factor = 0.95 / max;
            for sample in audio.iter_mut() {
                *sample *= factor;
            }
        }
    }
    
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
}
```

## Task 5: Audio Module Integration (`src/audio/mod.rs`)

```rust
pub mod capture;
pub mod wake_word;
pub mod speech_to_text;
pub mod processor;

use crate::config::AudioConfig;
use crate::error::Result;
use std::path::Path;

pub struct AudioSystem {
    capture: capture::AudioCapture,
    wake_word: wake_word::WakeWordDetector,
    stt: speech_to_text::SpeechToText,
    processor: processor::AudioProcessor,
}

impl AudioSystem {
    pub async fn new(config: &AudioConfig) -> Result<Self> {
        tracing::info!("Initializing audio system...");
        
        let capture = capture::AudioCapture::new(config.clone())?;
        
        let wake_word = wake_word::WakeWordDetector::new(
            &config.wake_words,
            Path::new("models/porcupine.ppn"),
            0.5, // sensitivity
        )?;
        
        let stt = speech_to_text::SpeechToText::new(
            Path::new(&config.whisper_model_path)
        )?;
        
        let processor = processor::AudioProcessor::new(
            config.silence_threshold * 0.5,
            1.0, // gain
        );
        
        tracing::info!("✅ Audio system initialized");
        
        Ok(Self {
            capture,
            wake_word,
            stt,
            processor,
        })
    }
    
    pub fn start_listening(&mut self) -> Result<()> {
        self.capture.start()
    }
    
    pub async fn wait_for_wake_word(&self) -> Result<bool> {
        loop {
            // Get last 1 second of audio from ring buffer
            let buffer = self.capture.get_ring_buffer_data(1000).await;
            
            if let Some(_keyword_idx) = self.wake_word.detect(&buffer).await? {
                return Ok(true);
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
    
    pub async fn record_and_transcribe(&self, timeout_secs: u64) -> Result<String> {
        tracing::info!("🎤 Recording command...");
        
        let audio = self.capture.record_command(timeout_secs).await?;
        let processed = self.processor.process(&audio);
        let text = self.stt.transcribe(&processed).await?;
        
        Ok(text)
    }
    
    pub fn stop(&mut self) -> Result<()> {
        self.capture.stop()
    }
}
```

## Testing Requirements

Create `tests/audio_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ring_buffer() {
        // Test ring buffer implementation
    }
    
    #[test]
    fn test_rms_calculation() {
        // Test RMS calculation
    }
    
    #[tokio::test]
    async fn test_audio_capture_init() {
        // Test audio capture initialization
    }
    
    #[tokio::test]
    async fn test_speech_recognition() {
        // Test with sample audio file
    }
}
```

## Performance Targets

- ✅ Wake word detection: **< 100ms latency**
- ✅ Speech recognition: **< 500ms for 3-second clip**
- ✅ CPU usage (idle): **< 5%**
- ✅ Memory usage: **< 200MB**
- ✅ No audio dropouts or glitches

## Success Criteria

✅ **Microphone capture works** - can record audio continuously  
✅ **Wake word detection works** - reliably detects "Hey Luna"  
✅ **Speech-to-text works** - accurately transcribes commands  
✅ **Low latency** - total pipeline < 1 second  
✅ **No crashes** - runs stable for hours  
✅ **Integrated with main loop** - ready for command processing  

## Next Steps

**Proceed to Part 3 (Brain/Understanding System)** once audio system is complete and tested.
