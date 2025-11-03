//! Mock implementations for testing
//!
//! Provides mock audio components for unit testing.

use super::traits::*;
use crate::error::Result;
use async_channel::{Receiver, Sender, unbounded};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

/// Mock audio capture for testing
pub struct MockAudioCapture {
    samples: Arc<Mutex<Vec<Vec<f32>>>>,
    active: Arc<Mutex<bool>>,
    rx: Receiver<Vec<f32>>,
    tx: Sender<Vec<f32>>,
    ring_buffer: Arc<Mutex<super::capture::RingBuffer>>,
}

impl MockAudioCapture {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            active: Arc::new(Mutex::new(false)),
            rx,
            tx,
            ring_buffer: Arc::new(Mutex::new(super::capture::RingBuffer::new(48000))),
        }
    }
    
    /// Add mock audio data for testing
    pub fn add_samples(&self, samples: Vec<f32>) {
        self.samples.lock().unwrap().push(samples);
    }
    
    /// Simulate audio capture by sending stored samples
    pub async fn simulate_capture(&self) {
        let samples = self.samples.lock().unwrap().clone();
        for sample in samples.iter() {
            let _ = self.tx.send(sample.clone()).await;
        }
    }
}

impl AudioCaptureInterface for MockAudioCapture {
    fn start(&mut self) -> Result<()> {
        *self.active.lock().unwrap() = true;
        let self_clone = self.clone_for_task();
        tokio::spawn(async move {
            self_clone.simulate_capture().await;
        });
        Ok(())
    }
    
    fn stop(&mut self) -> Result<()> {
        *self.active.lock().unwrap() = false;
        Ok(())
    }
    
    fn get_audio_stream(&self) -> Receiver<Vec<f32>> {
        self.rx.clone()
    }
    
    fn is_active(&self) -> bool {
        *self.active.lock().unwrap()
    }
    
    fn get_ring_buffer(&self) -> &Arc<Mutex<super::capture::RingBuffer>> {
        &self.ring_buffer
    }
}

impl MockAudioCapture {
    /// Mock implementation of record_command
    pub async fn record_command(&self, _max_duration_secs: u64) -> Result<Vec<f32>> {
        // Return first sample set or empty vec
        let samples = self.samples.lock().unwrap();
        Ok(samples.first().cloned().unwrap_or_default())
    }
}

impl MockAudioCapture {
    fn clone_for_task(&self) -> Self {
        Self {
            samples: self.samples.clone(),
            active: self.active.clone(),
            rx: self.rx.clone(),
            tx: self.tx.clone(),
            ring_buffer: self.ring_buffer.clone(),
        }
    }
}

/// Mock wake word detector for testing
pub struct MockWakeWordDetector {
    detections: Arc<Mutex<Vec<bool>>>,
    sensitivity: f32,
}

impl MockWakeWordDetector {
    pub fn new() -> Self {
        Self {
            detections: Arc::new(Mutex::new(Vec::new())),
            sensitivity: 0.5,
        }
    }
    
    /// Queue a detection result (true = detected, false = not detected)
    pub fn queue_detection(&self, detected: bool) {
        self.detections.lock().unwrap().push(detected);
    }
}

#[async_trait]
impl WakeWordDetectorInterface for MockWakeWordDetector {
    async fn detect(&self, _audio: &[f32]) -> Result<Option<usize>> {
        let mut detections = self.detections.lock().unwrap();
        if detections.is_empty() {
            Ok(None)
        } else {
            let detected = detections.remove(0);
            Ok(if detected { Some(0) } else { None })
        }
    }
    
    fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity;
    }
    
    fn get_sensitivity(&self) -> f32 {
        self.sensitivity
    }
}

/// Mock speech-to-text for testing
pub struct MockSpeechToText {
    transcriptions: Arc<Mutex<Vec<String>>>,
}

impl MockSpeechToText {
    pub fn new() -> Self {
        Self {
            transcriptions: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Queue a transcription result
    pub fn queue_transcription(&self, text: String) {
        self.transcriptions.lock().unwrap().push(text);
    }
}

#[async_trait]
impl SpeechToTextInterface for MockSpeechToText {
    async fn transcribe(&self, _audio: &[f32]) -> Result<String> {
        let mut transcriptions = self.transcriptions.lock().unwrap();
        if transcriptions.is_empty() {
            Ok("default transcription".to_string())
        } else {
            Ok(transcriptions.remove(0))
        }
    }
    
    fn is_simulated(&self) -> bool {
        true
    }
    
    fn sample_rate(&self) -> u32 {
        16000
    }
}

/// Mock audio processor for testing
pub struct MockAudioProcessor;

impl MockAudioProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl AudioProcessorInterface for MockAudioProcessor {
    fn apply_noise_gate(&self, _audio: &mut [f32]) {
        // No-op for mock
    }
    
    fn normalize(&self, _audio: &mut [f32]) {
        // No-op for mock
    }
    
    fn apply_high_pass_filter(&self, _audio: &mut [f32], _cutoff: f32, _sample_rate: f32) {
        // No-op for mock
    }
    
    fn apply_low_pass_filter(&self, _audio: &mut [f32], _cutoff: f32, _sample_rate: f32) {
        // No-op for mock
    }
    
    fn calculate_snr(&self, _audio: &[f32]) -> f32 {
        20.0 // Return a reasonable SNR value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_audio_capture() {
        let mut capture = MockAudioCapture::new();
        capture.add_samples(vec![0.1, 0.2, 0.3]);
        
        assert!(!capture.is_active());
        capture.start().unwrap();
        assert!(capture.is_active());
        
        let rx = capture.get_audio_stream();
        let samples = rx.recv().await.unwrap();
        assert_eq!(samples, vec![0.1, 0.2, 0.3]);
    }

    #[tokio::test]
    async fn test_mock_wake_word() {
        let detector = MockWakeWordDetector::new();
        detector.queue_detection(true);
        detector.queue_detection(false);
        
        let result1 = detector.detect(&[]).await.unwrap();
        assert!(result1.is_some());
        
        let result2 = detector.detect(&[]).await.unwrap();
        assert!(result2.is_none());
    }

    #[tokio::test]
    async fn test_mock_stt() {
        let stt = MockSpeechToText::new();
        stt.queue_transcription("test command".to_string());
        
        let result = stt.transcribe(&[]).await.unwrap();
        assert_eq!(result, "test command");
        
        assert!(stt.is_simulated());
    }
}
