//! Configuration management for the LUNA assistant.
//!
//! Loads configuration from TOML files with environment variable overrides.
//! Provides validation and default values for all settings.

use crate::error::{LunaError, Result};
use crate::config_error;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration structure for LUNA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LunaConfig {
    /// Audio system configuration
    pub audio: AudioConfig,
    /// Brain/NLP configuration
    pub brain: BrainConfig,
    /// System-level settings
    pub system: SystemConfig,
    /// Path configurations
    pub paths: PathsConfig,
    /// Performance tuning
    pub performance: PerformanceConfig,
}

/// Audio capture and processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Wake words to listen for (e.g., "Hey Luna", "Okay Luna")
    #[serde(default = "default_wake_words")]
    pub wake_words: Vec<String>,

    /// Sample rate in Hz (typical: 16000 for speech)
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,

    /// Number of audio channels (1 = mono, 2 = stereo)
    #[serde(default = "default_channels")]
    pub channels: u16,

    /// Audio buffer size in frames
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,

    /// Silence detection threshold (0.0 - 1.0)
    #[serde(default = "default_silence_threshold")]
    pub silence_threshold: f32,

    /// Maximum recording duration in seconds
    #[serde(default = "default_recording_timeout")]
    pub recording_timeout_secs: u64,

    // === God-Level Enhancement Settings ===
    
    /// Input device name or ID (empty = default device)
    #[serde(default)]
    pub input_device: String,

    /// Preferred device sample rate (will resample if needed)
    #[serde(default = "default_preferred_sample_rate")]
    pub preferred_sample_rate: u32,

    /// Target sample rate for STT/wake word (will resample from device rate)
    #[serde(default = "default_target_sample_rate")]
    pub target_sample_rate: u32,

    /// Audio frame duration in milliseconds
    #[serde(default = "default_frame_ms")]
    pub frame_ms: u32,

    /// Pre-roll duration in milliseconds (audio before wake word)
    #[serde(default = "default_pre_roll_ms")]
    pub pre_roll_ms: u32,

    /// Post-roll duration in milliseconds (audio after speech ends)
    #[serde(default = "default_post_roll_ms")]
    pub post_roll_ms: u32,

    /// Wake word engine: "porcupine", "openwakeword", "energy"
    #[serde(default = "default_wake_word_engine")]
    pub wake_word_engine: String,

    /// VAD engine: "webrtc", "silero", "rms"
    #[serde(default = "default_vad_engine")]
    pub vad_engine: String,

    /// VAD aggressiveness (0-3 for WebRTC VAD, 0=least aggressive, 3=most)
    #[serde(default = "default_vad_aggressiveness")]
    pub vad_aggressiveness: u8,

    /// Enable noise suppression
    #[serde(default = "default_true")]
    pub noise_suppression: bool,

    /// Enable automatic gain control
    #[serde(default = "default_true")]
    pub agc: bool,

    /// Enable acoustic echo cancellation
    #[serde(default = "default_false")]
    pub aec: bool,

    /// Drop policy for audio frames: "DropOldest", "DropNewest", "Block"
    #[serde(default = "default_drop_policy")]
    pub drop_policy: String,

    /// Ring buffer capacity in samples
    #[serde(default = "default_ring_buffer_capacity")]
    pub ring_buffer_capacity: usize,
}

/// Brain/NLP system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainConfig {
    /// Path to Whisper model file
    #[serde(default = "default_whisper_model_path")]
    pub whisper_model_path: String,

    /// Maximum response time in milliseconds
    #[serde(default = "default_response_timeout")]
    pub response_timeout_ms: u64,

    /// Number of previous commands to remember for context
    #[serde(default = "default_context_window")]
    pub context_window_size: usize,

    /// Minimum confidence threshold for command acceptance (0.0 - 1.0)
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f32,
    
    /// Wake word detection sensitivity (0.0 - 1.0, higher = more sensitive)
    #[serde(default = "default_wake_word_sensitivity")]
    pub wake_word_sensitivity: f32,
    
    /// STT engine: "whisper", "simulate"
    #[serde(default = "default_stt_engine")]
    pub stt_engine: String,
    
    /// Number of threads for STT processing
    #[serde(default = "default_stt_threads")]
    pub stt_threads: usize,
}

/// System-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Log level: "trace", "debug", "info", "warn", "error"
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Directory for persistent data
    #[serde(default = "default_data_dir")]
    pub data_dir: String,

    /// Directory for cache files
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,

    /// Enable telemetry (currently unused, for future)
    #[serde(default)]
    pub enable_telemetry: bool,
}

/// Path configurations for search and discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    /// Directories to search for files
    #[serde(default = "default_search_paths")]
    pub search_paths: Vec<String>,

    /// Paths to exclude from search
    #[serde(default = "default_exclude_paths")]
    pub exclude_paths: Vec<String>,

    /// Common project directory locations
    #[serde(default = "default_project_dirs")]
    pub project_dirs: Vec<String>,
}

/// Performance tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum number of worker threads
    #[serde(default = "default_max_threads")]
    pub max_threads: usize,

    /// Cache size in megabytes
    #[serde(default = "default_cache_size")]
    pub cache_size_mb: usize,

    /// File index update interval in seconds
    #[serde(default = "default_index_update_interval")]
    pub index_update_interval_secs: u64,
}

// Default value functions
fn default_wake_words() -> Vec<String> {
    vec!["hey luna".to_string(), "okay luna".to_string()]
}

fn default_sample_rate() -> u32 {
    16000
}

fn default_channels() -> u16 {
    1
}

fn default_buffer_size() -> usize {
    1024
}

fn default_silence_threshold() -> f32 {
    0.03
}

fn default_recording_timeout() -> u64 {
    5
}

// God-level audio enhancement defaults
fn default_preferred_sample_rate() -> u32 {
    48000
}

fn default_target_sample_rate() -> u32 {
    16000
}

fn default_frame_ms() -> u32 {
    10
}

fn default_pre_roll_ms() -> u32 {
    300
}

fn default_post_roll_ms() -> u32 {
    200
}

fn default_wake_word_engine() -> String {
    "energy".to_string()
}

fn default_vad_engine() -> String {
    "webrtc".to_string()
}

fn default_vad_aggressiveness() -> u8 {
    2
}

fn default_drop_policy() -> String {
    "DropOldest".to_string()
}

fn default_ring_buffer_capacity() -> usize {
    48000 // 1 second at 48kHz
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_whisper_model_path() -> String {
    "models/whisper-base.bin".to_string()
}

fn default_response_timeout() -> u64 {
    1000
}

fn default_context_window() -> usize {
    10
}

fn default_confidence_threshold() -> f32 {
    0.7
}

fn default_wake_word_sensitivity() -> f32 {
    0.5
}

fn default_stt_engine() -> String {
    "simulate".to_string()
}

fn default_stt_threads() -> usize {
    4
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_data_dir() -> String {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("luna")
        .to_string_lossy()
        .to_string()
}

fn default_cache_dir() -> String {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("luna")
        .to_string_lossy()
        .to_string()
}

fn default_search_paths() -> Vec<String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    vec![
        home.join("Documents").to_string_lossy().to_string(),
        home.join("Desktop").to_string_lossy().to_string(),
        home.join("Downloads").to_string_lossy().to_string(),
    ]
}

fn default_exclude_paths() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        ".git".to_string(),
        "target".to_string(),
        ".cache".to_string(),
    ]
}

fn default_project_dirs() -> Vec<String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    vec![
        home.join("Projects").to_string_lossy().to_string(),
        home.join("dev").to_string_lossy().to_string(),
        home.join("workspace").to_string_lossy().to_string(),
    ]
}

fn default_max_threads() -> usize {
    num_cpus::get().max(4)
}

fn default_cache_size() -> usize {
    256
}

fn default_index_update_interval() -> u64 {
    300 // 5 minutes
}

impl Default for LunaConfig {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            brain: BrainConfig::default(),
            system: SystemConfig::default(),
            paths: PathsConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            wake_words: default_wake_words(),
            sample_rate: default_sample_rate(),
            channels: default_channels(),
            buffer_size: default_buffer_size(),
            silence_threshold: default_silence_threshold(),
            recording_timeout_secs: default_recording_timeout(),
            input_device: String::new(),
            preferred_sample_rate: default_preferred_sample_rate(),
            target_sample_rate: default_target_sample_rate(),
            frame_ms: default_frame_ms(),
            pre_roll_ms: default_pre_roll_ms(),
            post_roll_ms: default_post_roll_ms(),
            wake_word_engine: default_wake_word_engine(),
            vad_engine: default_vad_engine(),
            vad_aggressiveness: default_vad_aggressiveness(),
            noise_suppression: default_true(),
            agc: default_true(),
            aec: default_false(),
            drop_policy: default_drop_policy(),
            ring_buffer_capacity: default_ring_buffer_capacity(),
        }
    }
}

impl AudioConfig {
    /// Validate audio configuration
    pub fn validate(&self) -> Result<()> {
        // Sample rate validation
        if !(8000..=48000).contains(&self.sample_rate) {
            return Err(config_error!(
                "Sample rate {} is out of range (8000-48000 Hz)",
                self.sample_rate
            ));
        }
        
        // Channels validation
        if self.channels != 1 && self.channels != 2 {
            return Err(config_error!(
                "Channels must be 1 (mono) or 2 (stereo), got {}",
                self.channels
            ));
        }
        
        // Threshold validation
        if !(0.0..=1.0).contains(&self.silence_threshold) {
            return Err(config_error!(
                "Silence threshold {} must be between 0.0 and 1.0",
                self.silence_threshold
            ));
        }
        
        // Timeout validation
        if self.recording_timeout_secs == 0 || self.recording_timeout_secs > 300 {
            return Err(config_error!(
                "Recording timeout {} is invalid (must be 1-300 seconds)",
                self.recording_timeout_secs
            ));
        }
        
        // Buffer size validation
        if self.buffer_size < 256 || self.buffer_size > 8192 {
            return Err(config_error!(
                "Buffer size {} is invalid (must be 256-8192 frames)",
                self.buffer_size
            ));
        }
        
        Ok(())
    }
}

impl Default for BrainConfig {
    fn default() -> Self {
        Self {
            whisper_model_path: default_whisper_model_path(),
            response_timeout_ms: default_response_timeout(),
            context_window_size: default_context_window(),
            confidence_threshold: default_confidence_threshold(),
            wake_word_sensitivity: default_wake_word_sensitivity(),
            stt_engine: default_stt_engine(),
            stt_threads: default_stt_threads(),
        }
    }
}

impl BrainConfig {
    /// Validate brain configuration
    pub fn validate(&self) -> Result<()> {
        use tracing::warn;
        
        // Model path check (warning if not found, not error)
        let model_path = Path::new(&self.whisper_model_path);
        if !model_path.exists() {
            warn!(
                "Whisper model not found at: {}. Will run in simulated mode.",
                self.whisper_model_path
            );
        }
        
        // Confidence threshold
        if !(0.0..=1.0).contains(&self.confidence_threshold) {
            return Err(config_error!(
                "Confidence threshold {} must be between 0.0 and 1.0",
                self.confidence_threshold
            ));
        }
        
        // Wake word sensitivity
        if !(0.0..=1.0).contains(&self.wake_word_sensitivity) {
            return Err(config_error!(
                "Wake word sensitivity {} must be between 0.0 and 1.0",
                self.wake_word_sensitivity
            ));
        }
        
        // Context window size
        if self.context_window_size == 0 || self.context_window_size > 100 {
            return Err(config_error!(
                "Context window size {} is invalid (must be 1-100)",
                self.context_window_size
            ));
        }
        
        Ok(())
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            data_dir: default_data_dir(),
            cache_dir: default_cache_dir(),
            enable_telemetry: false,
        }
    }
}

impl SystemConfig {
    /// Validate system configuration
    pub fn validate(&self) -> Result<()> {
        // Validate log level
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(config_error!(
                "Invalid log level '{}'. Must be one of: {:?}",
                self.log_level, valid_log_levels
            ));
        }
        
        // Ensure data and cache directories exist or can be created
        std::fs::create_dir_all(&self.data_dir).map_err(|e| {
            config_error!("Cannot create data directory: {}", e)
        })?;
        
        std::fs::create_dir_all(&self.cache_dir).map_err(|e| {
            config_error!("Cannot create cache directory: {}", e)
        })?;
        
        Ok(())
    }
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            search_paths: default_search_paths(),
            exclude_paths: default_exclude_paths(),
            project_dirs: default_project_dirs(),
        }
    }
}

impl PathsConfig {
    /// Validate paths configuration
    pub fn validate(&self) -> Result<()> {
        use tracing::warn;
        
        // Warn about non-existent search paths but don't fail
        for path in &self.search_paths {
            if !Path::new(path).exists() {
                warn!("Search path does not exist: {}", path);
            }
        }
        
        Ok(())
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_threads: default_max_threads(),
            cache_size_mb: default_cache_size(),
            index_update_interval_secs: default_index_update_interval(),
        }
    }
}

impl PerformanceConfig {
    /// Validate performance configuration
    pub fn validate(&self) -> Result<()> {
        // Max threads validation
        if self.max_threads == 0 || self.max_threads > 128 {
            return Err(config_error!(
                "Max threads {} is invalid (must be 1-128)",
                self.max_threads
            ));
        }
        
        // Cache size validation
        if self.cache_size_mb > 10240 {
            return Err(config_error!(
                "Cache size {} MB is too large (maximum 10240 MB)",
                self.cache_size_mb
            ));
        }
        
        // Index update interval validation
        if self.index_update_interval_secs < 60 {
            return Err(config_error!(
                "Index update interval {} seconds is too short (minimum 60 seconds)",
                self.index_update_interval_secs
            ));
        }
        
        Ok(())
    }
}

impl LunaConfig {
    /// Load configuration from default location or create default config
    pub fn load() -> Result<Self> {
        Self::load_from_path("config/default.toml")
    }

    /// Load configuration from a specific file path
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // If file exists, load it; otherwise use defaults
        if path.exists() {
            let contents = std::fs::read_to_string(path).map_err(|e| {
                LunaError::Config(format!("Failed to read config file: {}", e))
            })?;

            let config: LunaConfig = toml::from_str(&contents).map_err(|e| {
                LunaError::Config(format!("Failed to parse config: {}", e))
            })?;

            config.validate()?;
            Ok(config)
        } else {
            // Return default config
            let config = Self::default();
            config.validate()?;
            Ok(config)
        }
    }
    
    /// Load and validate configuration
    pub fn load_and_validate<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = Self::load_from_path(path)?;
        config.validate()?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                LunaError::Config(format!("Failed to create config directory: {}", e))
            })?;
        }

        let contents = toml::to_string_pretty(self).map_err(|e| {
            LunaError::Config(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(path, contents).map_err(|e| {
            LunaError::Config(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        // Validate each subsection
        self.audio.validate()?;
        self.brain.validate()?;
        self.system.validate()?;
        self.paths.validate()?;
        self.performance.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = LunaConfig::default();
        assert_eq!(config.audio.sample_rate, 16000);
        assert_eq!(config.audio.channels, 1);
        assert!(config.brain.confidence_threshold >= 0.0);
    }

    #[test]
    fn test_config_validation() {
        let mut config = LunaConfig::default();
        assert!(config.validate().is_ok());

        // Invalid sample rate
        config.audio.sample_rate = 1000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_save_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.toml");

        let config = LunaConfig::default();
        config.save(&config_path).unwrap();

        let loaded = LunaConfig::load_from_path(&config_path).unwrap();
        assert_eq!(loaded.audio.sample_rate, config.audio.sample_rate);
    }

    #[test]
    fn test_load_nonexistent_file() {
        // Should return default config
        let result = LunaConfig::load_from_path("nonexistent.toml");
        assert!(result.is_ok());
    }
}
