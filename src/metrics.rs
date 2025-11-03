//! Metrics collection for LUNA
//!
//! Tracks performance metrics, success rates, and latency measurements.
//! Supports both atomic counters and external metrics crate integration.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

// Re-export metrics macros when feature is enabled
#[cfg(feature = "prometheus")]
pub use metrics::{counter, histogram, gauge};

/// Performance metrics collector
pub struct Metrics {
    // Counters
    commands_processed: AtomicUsize,
    commands_succeeded: AtomicUsize,
    commands_failed: AtomicUsize,
    wake_words_detected: AtomicUsize,
    
    // Latency (microseconds)
    total_processing_latency: AtomicU64,
    audio_capture_latency: AtomicU64,
    stt_latency: AtomicU64,
    parsing_latency: AtomicU64,
    execution_latency: AtomicU64,
    
    // Counters for averaging
    audio_capture_count: AtomicUsize,
    stt_count: AtomicUsize,
    parsing_count: AtomicUsize,
    execution_count: AtomicUsize,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            commands_processed: AtomicUsize::new(0),
            commands_succeeded: AtomicUsize::new(0),
            commands_failed: AtomicUsize::new(0),
            wake_words_detected: AtomicUsize::new(0),
            total_processing_latency: AtomicU64::new(0),
            audio_capture_latency: AtomicU64::new(0),
            stt_latency: AtomicU64::new(0),
            parsing_latency: AtomicU64::new(0),
            execution_latency: AtomicU64::new(0),
            audio_capture_count: AtomicUsize::new(0),
            stt_count: AtomicUsize::new(0),
            parsing_count: AtomicUsize::new(0),
            execution_count: AtomicUsize::new(0),
        }
    }
    
    /// Record a command was processed
    pub fn record_command_processed(&self) {
        self.commands_processed.fetch_add(1, Ordering::Relaxed);
        
        #[cfg(feature = "prometheus")]
        counter!("luna_commands_processed_total").increment(1);
    }
    
    /// Record a successful command execution
    pub fn record_command_success(&self) {
        self.commands_succeeded.fetch_add(1, Ordering::Relaxed);
        
        #[cfg(feature = "prometheus")]
        counter!("luna_commands_total", "status" => "success").increment(1);
    }
    
    /// Record a failed command execution
    pub fn record_command_failure(&self) {
        self.commands_failed.fetch_add(1, Ordering::Relaxed);
        
        #[cfg(feature = "prometheus")]
        counter!("luna_commands_total", "status" => "failure").increment(1);
    }
    
    /// Record a wake word detection
    pub fn record_wake_word(&self) {
        self.wake_words_detected.fetch_add(1, Ordering::Relaxed);
        
        #[cfg(feature = "prometheus")]
        counter!("luna_wake_words_detected_total").increment(1);
    }
    
    /// Record wake word with confidence
    pub fn record_wake_word_confidence(&self, confidence: f32) {
        self.wake_words_detected.fetch_add(1, Ordering::Relaxed);
        
        #[cfg(feature = "prometheus")]
        {
            counter!("luna_wake_words_detected_total").increment(1);
            histogram!("luna_wake_confidence").record(confidence as f64);
        }
    }
    
    /// Record audio frame drop
    pub fn record_audio_frame_drop(&self) {
        #[cfg(feature = "prometheus")]
        counter!("luna_audio_dropped_frames_total").increment(1);
    }
    
    /// Record ring buffer fill ratio
    pub fn record_ring_fill_ratio(&self, ratio: f32) {
        #[cfg(feature = "prometheus")]
        gauge!("luna_audio_ring_fill_ratio").set(ratio as f64);
    }
    
    /// Record VAD trigger
    pub fn record_vad_trigger(&self, is_speech: bool) {
        #[cfg(feature = "prometheus")]
        {
            if is_speech {
                counter!("luna_vad_speech_total").increment(1);
            } else {
                counter!("luna_vad_silence_total").increment(1);
            }
        }
    }
    
    /// Record audio processing duration
    pub fn record_audio_processing(&self, duration_ms: f64) {
        #[cfg(feature = "prometheus")]
        histogram!("luna_audio_processing_duration_ms").record(duration_ms);
    }
    
    /// Record an error by category
    pub fn record_error(&self, error_type: &str, module: &str) {
        #[cfg(feature = "prometheus")]
        counter!("luna_errors_total", "type" => error_type, "module" => module).increment(1);
    }
    
    /// Record latency for a specific phase
    pub fn record_latency(&self, phase: MetricPhase, duration: std::time::Duration) {
        let micros = duration.as_micros() as u64;
        let millis = duration.as_millis() as f64;
        
        // Record in atomic counters
        match phase {
            MetricPhase::AudioCapture => {
                self.audio_capture_latency.fetch_add(micros, Ordering::Relaxed);
                self.audio_capture_count.fetch_add(1, Ordering::Relaxed);
                
                #[cfg(feature = "prometheus")]
                histogram!("luna_audio_capture_duration_ms").record(millis);
            }
            MetricPhase::SpeechToText => {
                self.stt_latency.fetch_add(micros, Ordering::Relaxed);
                self.stt_count.fetch_add(1, Ordering::Relaxed);
                
                #[cfg(feature = "prometheus")]
                histogram!("luna_stt_duration_ms").record(millis);
            }
            MetricPhase::Parsing => {
                self.parsing_latency.fetch_add(micros, Ordering::Relaxed);
                self.parsing_count.fetch_add(1, Ordering::Relaxed);
                
                #[cfg(feature = "prometheus")]
                histogram!("luna_parsing_duration_ms").record(millis);
            }
            MetricPhase::Execution => {
                self.execution_latency.fetch_add(micros, Ordering::Relaxed);
                self.execution_count.fetch_add(1, Ordering::Relaxed);
                
                #[cfg(feature = "prometheus")]
                histogram!("luna_execution_duration_ms").record(millis);
            }
            MetricPhase::Total => {
                self.total_processing_latency.fetch_add(micros, Ordering::Relaxed);
                
                #[cfg(feature = "prometheus")]
                histogram!("luna_total_processing_duration_ms").record(millis);
            }
        }
    }
    
    /// Get the number of commands processed
    pub fn get_commands_processed(&self) -> usize {
        self.commands_processed.load(Ordering::Relaxed)
    }
    
    /// Get the success rate as a percentage
    pub fn get_success_rate(&self) -> f64 {
        let processed = self.commands_processed.load(Ordering::Relaxed);
        if processed == 0 {
            return 0.0;
        }
        let succeeded = self.commands_succeeded.load(Ordering::Relaxed);
        (succeeded as f64 / processed as f64) * 100.0
    }
    
    /// Get the number of wake words detected
    pub fn get_wake_words_detected(&self) -> usize {
        self.wake_words_detected.load(Ordering::Relaxed)
    }
    
    /// Get the number of commands succeeded
    pub fn get_commands_succeeded(&self) -> usize {
        self.commands_succeeded.load(Ordering::Relaxed)
    }
    
    /// Get the number of commands failed
    pub fn get_commands_failed(&self) -> usize {
        self.commands_failed.load(Ordering::Relaxed)
    }
    
    /// Get average audio capture latency in milliseconds
    pub fn get_avg_audio_capture_ms(&self) -> f64 {
        let count = self.audio_capture_count.load(Ordering::Relaxed);
        if count > 0 {
            (self.audio_capture_latency.load(Ordering::Relaxed) / count as u64) as f64 / 1000.0
        } else {
            0.0
        }
    }
    
    /// Get average STT latency in milliseconds
    pub fn get_avg_stt_ms(&self) -> f64 {
        let count = self.stt_count.load(Ordering::Relaxed);
        if count > 0 {
            (self.stt_latency.load(Ordering::Relaxed) / count as u64) as f64 / 1000.0
        } else {
            0.0
        }
    }
    
    /// Get average parsing latency in milliseconds
    pub fn get_avg_parsing_ms(&self) -> f64 {
        let count = self.parsing_count.load(Ordering::Relaxed);
        if count > 0 {
            (self.parsing_latency.load(Ordering::Relaxed) / count as u64) as f64 / 1000.0
        } else {
            0.0
        }
    }
    
    /// Get average execution latency in milliseconds
    pub fn get_avg_execution_ms(&self) -> f64 {
        let count = self.execution_count.load(Ordering::Relaxed);
        if count > 0 {
            (self.execution_latency.load(Ordering::Relaxed) / count as u64) as f64 / 1000.0
        } else {
            0.0
        }
    }
    
    /// Print a summary of collected metrics
    pub fn print_summary(&self) {
        let processed = self.commands_processed.load(Ordering::Relaxed);
        let succeeded = self.commands_succeeded.load(Ordering::Relaxed);
        let failed = self.commands_failed.load(Ordering::Relaxed);
        let wake_words = self.wake_words_detected.load(Ordering::Relaxed);
        
        println!("\n📊 LUNA Metrics Summary");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  Commands processed: {}", processed);
        println!("  Commands succeeded: {}", succeeded);
        println!("  Commands failed:    {}", failed);
        println!("  Wake words detected: {}", wake_words);
        println!("  Success rate:       {:.1}%", 
            if processed > 0 { succeeded as f64 / processed as f64 * 100.0 } else { 0.0 }
        );
        
        if processed > 0 {
            println!("\n  Average latencies:");
            
            let total_count = self.commands_processed.load(Ordering::Relaxed);
            if total_count > 0 {
                let avg_total = self.total_processing_latency.load(Ordering::Relaxed) / total_count as u64;
                println!("    Total:     {}ms", avg_total / 1000);
            }
            
            let audio_count = self.audio_capture_count.load(Ordering::Relaxed);
            if audio_count > 0 {
                let avg_audio = self.audio_capture_latency.load(Ordering::Relaxed) / audio_count as u64;
                println!("    Audio:     {}ms", avg_audio / 1000);
            }
            
            let stt_count = self.stt_count.load(Ordering::Relaxed);
            if stt_count > 0 {
                let avg_stt = self.stt_latency.load(Ordering::Relaxed) / stt_count as u64;
                println!("    STT:       {}ms", avg_stt / 1000);
            }
            
            let parse_count = self.parsing_count.load(Ordering::Relaxed);
            if parse_count > 0 {
                let avg_parse = self.parsing_latency.load(Ordering::Relaxed) / parse_count as u64;
                println!("    Parse:     {}ms", avg_parse / 1000);
            }
            
            let exec_count = self.execution_count.load(Ordering::Relaxed);
            if exec_count > 0 {
                let avg_exec = self.execution_latency.load(Ordering::Relaxed) / exec_count as u64;
                println!("    Execution: {}ms", avg_exec / 1000);
            }
        }
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
    
    /// Reset all metrics
    pub fn reset(&self) {
        self.commands_processed.store(0, Ordering::Relaxed);
        self.commands_succeeded.store(0, Ordering::Relaxed);
        self.commands_failed.store(0, Ordering::Relaxed);
        self.wake_words_detected.store(0, Ordering::Relaxed);
        self.total_processing_latency.store(0, Ordering::Relaxed);
        self.audio_capture_latency.store(0, Ordering::Relaxed);
        self.stt_latency.store(0, Ordering::Relaxed);
        self.parsing_latency.store(0, Ordering::Relaxed);
        self.execution_latency.store(0, Ordering::Relaxed);
        self.audio_capture_count.store(0, Ordering::Relaxed);
        self.stt_count.store(0, Ordering::Relaxed);
        self.parsing_count.store(0, Ordering::Relaxed);
        self.execution_count.store(0, Ordering::Relaxed);
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Metric collection phases
#[derive(Debug, Clone, Copy)]
pub enum MetricPhase {
    AudioCapture,
    SpeechToText,
    Parsing,
    Execution,
    Total,
}

/// Helper to automatically measure execution time
pub struct MetricTimer {
    start: Instant,
    metrics: Arc<Metrics>,
    phase: MetricPhase,
}

impl MetricTimer {
    /// Create a new metric timer
    pub fn new(metrics: Arc<Metrics>, phase: MetricPhase) -> Self {
        Self {
            start: Instant::now(),
            metrics,
            phase,
        }
    }
}

impl Drop for MetricTimer {
    fn drop(&mut self) {
        self.metrics.record_latency(self.phase, self.start.elapsed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics::new();
        assert_eq!(metrics.get_commands_processed(), 0);
        assert_eq!(metrics.get_success_rate(), 0.0);
    }
    
    #[test]
    fn test_command_tracking() {
        let metrics = Metrics::new();
        
        metrics.record_command_processed();
        metrics.record_command_success();
        
        assert_eq!(metrics.get_commands_processed(), 1);
        assert_eq!(metrics.get_success_rate(), 100.0);
        
        metrics.record_command_processed();
        metrics.record_command_failure();
        
        assert_eq!(metrics.get_commands_processed(), 2);
        assert_eq!(metrics.get_success_rate(), 50.0);
    }
    
    #[test]
    fn test_latency_recording() {
        let metrics = Metrics::new();
        
        metrics.record_latency(MetricPhase::AudioCapture, Duration::from_millis(100));
        metrics.record_latency(MetricPhase::SpeechToText, Duration::from_millis(500));
        
        // Just verify no panics; actual values checked in print_summary
        assert_eq!(metrics.audio_capture_count.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.stt_count.load(Ordering::Relaxed), 1);
    }
    
    #[test]
    fn test_wake_word_tracking() {
        let metrics = Metrics::new();
        
        metrics.record_wake_word();
        metrics.record_wake_word();
        
        assert_eq!(metrics.wake_words_detected.load(Ordering::Relaxed), 2);
    }
    
    #[test]
    fn test_reset() {
        let metrics = Metrics::new();
        
        metrics.record_command_processed();
        metrics.record_command_success();
        metrics.record_wake_word();
        
        assert_eq!(metrics.get_commands_processed(), 1);
        
        metrics.reset();
        
        assert_eq!(metrics.get_commands_processed(), 0);
        assert_eq!(metrics.wake_words_detected.load(Ordering::Relaxed), 0);
    }
    
    #[test]
    fn test_metric_timer() {
        let metrics = Arc::new(Metrics::new());
        
        {
            let _timer = MetricTimer::new(metrics.clone(), MetricPhase::AudioCapture);
            std::thread::sleep(Duration::from_millis(10));
        }
        
        // Timer should have recorded something
        assert!(metrics.audio_capture_count.load(Ordering::Relaxed) > 0);
    }
}

/// Prometheus exporter integration
#[cfg(feature = "prometheus")]
pub mod prometheus {
    use metrics_exporter_prometheus::PrometheusBuilder;
    use std::net::SocketAddr;
    use crate::error::Result;
    
    /// Start Prometheus metrics exporter
    ///
    /// Starts an HTTP server on the given address that serves metrics
    /// in Prometheus format at /metrics endpoint.
    ///
    /// # Arguments
    /// * `addr` - Address to bind to (e.g., "127.0.0.1:9090")
    ///
    /// # Example
    /// ```no_run
    /// # use luna::metrics::prometheus;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// prometheus::start_exporter("127.0.0.1:9090").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_exporter(addr: &str) -> Result<()> {
        let socket_addr: SocketAddr = addr.parse()
            .map_err(|e| crate::error::LunaError::Config(
                format!("Invalid metrics address: {}", e)
            ))?;
        
        PrometheusBuilder::new()
            .with_http_listener(socket_addr)
            .install()
            .map_err(|e| crate::error::LunaError::Config(
                format!("Failed to install Prometheus exporter: {}", e)
            ))?;
        
        tracing::info!("📊 Prometheus metrics exporter started on http://{}/metrics", addr);
        
        Ok(())
    }
    
    /// Initialize Prometheus metrics with defaults
    pub fn init_metrics() {
        use metrics::{describe_counter, describe_histogram, Unit};
        
        // Describe counters
        describe_counter!("luna_commands_processed_total", "Total number of commands processed");
        describe_counter!("luna_commands_total", "Commands by status");
        describe_counter!("luna_wake_words_detected_total", "Total wake words detected");
        describe_counter!("luna_errors_total", "Errors by type and module");
        
        // Describe histograms
        describe_histogram!("luna_audio_capture_duration_ms", Unit::Milliseconds, "Audio capture duration");
        describe_histogram!("luna_stt_duration_ms", Unit::Milliseconds, "Speech-to-text duration");
        describe_histogram!("luna_parsing_duration_ms", Unit::Milliseconds, "Command parsing duration");
        describe_histogram!("luna_execution_duration_ms", Unit::Milliseconds, "Action execution duration");
        describe_histogram!("luna_total_processing_duration_ms", Unit::Milliseconds, "Total processing duration");
    }
}
