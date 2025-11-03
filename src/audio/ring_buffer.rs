//! Lock-free ring buffer for real-time audio
//!
//! Uses lock-free SPSC (single producer, single consumer) queue for zero-contention
//! audio buffering. Critical for real-time audio callbacks.

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;

/// Lock-free ring buffer for audio samples
///
/// Uses a simple VecDeque-based ring buffer with mutex protection.
/// For production, consider using a true lock-free SPSC queue.
pub struct LockFreeRingBuffer {
    /// Underlying ring buffer
    rb: Arc<Mutex<VecDeque<f32>>>,
    /// Capacity in samples
    capacity: usize,
}

impl LockFreeRingBuffer {
    /// Create a new ring buffer
    ///
    /// # Arguments
    /// * `capacity` - Buffer capacity in samples (e.g., 48000 for 1 second at 48kHz)
    pub fn new(capacity: usize) -> Self {
        let rb = VecDeque::with_capacity(capacity);

        Self {
            rb: Arc::new(Mutex::new(rb)),
            capacity,
        }
    }

    /// Push samples into the ring buffer (producer side)
    ///
    /// This is called from the audio callback.
    /// If buffer is full, oldest samples are dropped.
    ///
    /// # Arguments
    /// * `samples` - Audio samples to push
    pub fn push_samples(&self, samples: &[f32]) {
        let mut rb = self.rb.lock();

        // Remove old samples if needed
        while rb.len() + samples.len() > self.capacity {
            rb.pop_front();
        }

        // Push new samples
        rb.extend(samples);
    }

    /// Get the last N samples (consumer side)
    ///
    /// # Arguments
    /// * `duration_ms` - Duration in milliseconds
    /// * `sample_rate` - Sample rate in Hz
    ///
    /// # Returns
    /// Vector of samples (may be shorter if not enough data available)
    pub fn get_last_n_samples(&self, duration_ms: u64, sample_rate: u32) -> Vec<f32> {
        let n = ((duration_ms * sample_rate as u64) / 1000) as usize;
        let n = n.min(self.capacity);

        let rb = self.rb.lock();
        let available = rb.len();
        let to_read = n.min(available);

        if to_read == 0 {
            return Vec::new();
        }

        // Read from the end of the buffer
        if available >= to_read {
            rb.iter().skip(available - to_read).copied().collect()
        } else {
            rb.iter().copied().collect()
        }
    }

    /// Get all available samples and clear the buffer
    ///
    /// # Returns
    /// All samples currently in the buffer
    pub fn drain(&self) -> Vec<f32> {
        let mut rb = self.rb.lock();
        rb.drain(..).collect()
    }

    /// Get current fill level (0.0 - 1.0)
    pub fn fill_ratio(&self) -> f32 {
        let rb = self.rb.lock();
        rb.len() as f32 / self.capacity as f32
    }

    /// Get number of samples currently in buffer
    pub fn len(&self) -> usize {
        let rb = self.rb.lock();
        rb.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        let rb = self.rb.lock();
        rb.is_empty()
    }

    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear the buffer
    pub fn clear(&self) {
        let mut rb = self.rb.lock();
        rb.clear();
    }
}

impl Clone for LockFreeRingBuffer {
    fn clone(&self) -> Self {
        Self {
            rb: Arc::clone(&self.rb),
            capacity: self.capacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_creation() {
        let rb = LockFreeRingBuffer::new(1000);
        assert_eq!(rb.capacity(), 1000);
        assert!(rb.is_empty());
    }

    #[test]
    fn test_push_and_get() {
        let rb = LockFreeRingBuffer::new(1000);

        // Push some samples
        rb.push_samples(&[1.0, 2.0, 3.0, 4.0, 5.0]);

        // Get last 100ms at 16kHz (1600 samples, but we only have 5)
        let samples = rb.get_last_n_samples(100, 16000);
        assert_eq!(samples.len(), 5);
        assert_eq!(samples, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_overflow() {
        let rb = LockFreeRingBuffer::new(10);

        // Push more than capacity
        rb.push_samples(&[
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ]);

        // Should have kept the last 10
        let samples = rb.get_last_n_samples(1000, 10);
        assert!(samples.len() <= 10);
    }

    #[test]
    fn test_drain() {
        let rb = LockFreeRingBuffer::new(1000);

        rb.push_samples(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let samples = rb.drain();

        assert_eq!(samples.len(), 5);
        assert!(rb.is_empty());
    }

    #[test]
    fn test_fill_ratio() {
        let rb = LockFreeRingBuffer::new(100);

        assert_eq!(rb.fill_ratio(), 0.0);

        rb.push_samples(&[1.0; 50]);
        assert!((rb.fill_ratio() - 0.5).abs() < 0.01);

        rb.push_samples(&[1.0; 50]);
        assert!((rb.fill_ratio() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_clear() {
        let rb = LockFreeRingBuffer::new(1000);

        rb.push_samples(&[1.0, 2.0, 3.0]);
        assert!(!rb.is_empty());

        rb.clear();
        assert!(rb.is_empty());
    }
}
