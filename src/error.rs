//! Error types and handling for the LUNA assistant.
//!
//! This module provides comprehensive error types using thiserror,
//! with context-rich messages for debugging and user-friendly display.

use std::collections::HashMap;
use thiserror::Error;

/// Stable error codes for telemetry and error policies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // Audio errors (1000-1099)
    AudioCaptureFailure = 1000,
    AudioDeviceNotFound = 1001,
    AudioFormatUnsupported = 1002,

    // Wake word errors (1100-1199)
    WakeWordModelNotFound = 1100,
    WakeWordDetectionFailed = 1101,

    // Speech recognition errors (1200-1299)
    SttModelNotFound = 1200,
    SttTranscriptionFailed = 1201,
    SttAudioTooShort = 1202,

    // Command parsing errors (1300-1399)
    CommandParseFailure = 1300,
    CommandNotUnderstood = 1301,
    CommandAmbiguous = 1302,

    // Application errors (1400-1499)
    AppNotFound = 1400,
    AppLaunchFailed = 1401,
    AppCloseFailed = 1402,

    // File errors (1500-1599)
    FileNotFound = 1500,
    FileAccessDenied = 1501,
    FileOperationFailed = 1502,

    // System operation errors (1600-1699)
    SystemOperationFailed = 1600,
    SystemPermissionDenied = 1601,

    // Configuration errors (1700-1799)
    ConfigLoadFailed = 1700,
    ConfigInvalid = 1701,
    ConfigSaveFailed = 1702,

    // Database errors (1800-1899)
    DatabaseLoadFailed = 1800,
    DatabaseSaveFailed = 1801,
    DatabaseCorrupted = 1802,

    // Unknown/generic errors (9000+)
    Unknown = 9000,
}

impl ErrorCode {
    /// Get the error code as an integer
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    /// Get error category
    pub fn category(&self) -> &'static str {
        match *self as u32 {
            1000..=1099 => "audio",
            1100..=1199 => "wake_word",
            1200..=1299 => "speech_recognition",
            1300..=1399 => "command_parsing",
            1400..=1499 => "application",
            1500..=1599 => "file",
            1600..=1699 => "system",
            1700..=1799 => "configuration",
            1800..=1899 => "database",
            _ => "unknown",
        }
    }
}

/// Main error type for LUNA operations
#[derive(Error, Debug)]
pub enum LunaError {
    /// Audio system errors (capture, processing, etc.)
    #[error("Audio error: {0}")]
    Audio(String),

    /// Wake word detection failures
    #[error("Wake word detection failed: {0}")]
    WakeWord(String),

    /// Speech-to-text recognition errors
    #[error("Speech recognition error: {0}")]
    SpeechRecognition(String),

    /// Command parsing failures
    #[error("Command parsing error: {0}")]
    CommandParsing(String),

    /// Application not found in database
    #[error("Application not found: '{0}'")]
    AppNotFound(String),

    /// File or directory not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Invalid or missing parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// System operation failures (volume, power, etc.)
    #[error("System operation failed: {0}")]
    SystemOperation(String),

    /// Configuration loading or validation errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Database operations (read, write, search)
    #[error("Database error: {0}")]
    Database(String),

    /// Window management errors
    #[error("Window control error: {0}")]
    WindowControl(String),

    /// TTS (Text-to-Speech) errors
    #[error("TTS error: {0}")]
    Tts(String),

    /// Context or memory management errors
    #[error("Context error: {0}")]
    Context(String),

    /// IO errors (file operations)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML parsing errors
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// WAV file errors
    #[error("WAV file error: {0}")]
    Wav(#[from] hound::Error),

    /// Generic unknown errors
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias for LUNA operations
pub type Result<T> = std::result::Result<T, LunaError>;

// Helper macros for creating specific error types with formatting

/// Create an audio error with formatted message
#[macro_export]
macro_rules! audio_error {
    ($($arg:tt)*) => {
        $crate::error::LunaError::Audio(format!($($arg)*))
    };
}

/// Create a wake word error with formatted message
#[macro_export]
macro_rules! wake_word_error {
    ($($arg:tt)*) => {
        $crate::error::LunaError::WakeWord(format!($($arg)*))
    };
}

/// Create a speech recognition error with formatted message
#[macro_export]
macro_rules! speech_error {
    ($($arg:tt)*) => {
        $crate::error::LunaError::SpeechRecognition(format!($($arg)*))
    };
}

/// Create a command parsing error with formatted message
#[macro_export]
macro_rules! parse_error {
    ($($arg:tt)*) => {
        $crate::error::LunaError::CommandParsing(format!($($arg)*))
    };
}

/// Create a configuration error with formatted message
#[macro_export]
macro_rules! config_error {
    ($($arg:tt)*) => {
        $crate::error::LunaError::Config(format!($($arg)*))
    };
}

/// Create a database error with formatted message
#[macro_export]
macro_rules! db_error {
    ($($arg:tt)*) => {
        $crate::error::LunaError::Database(format!($($arg)*))
    };
}

/// Create a system operation error with formatted message
#[macro_export]
macro_rules! system_error {
    ($($arg:tt)*) => {
        $crate::error::LunaError::SystemOperation(format!($($arg)*))
    };
}

/// Create a TTS error with formatted message
#[macro_export]
macro_rules! tts_error {
    ($($arg:tt)*) => {
        $crate::error::LunaError::Tts(format!($($arg)*))
    };
}

impl LunaError {
    /// Create a TTS error
    pub fn tts_error(msg: impl Into<String>) -> Self {
        LunaError::Tts(msg.into())
    }

    /// Get the stable error code for this error
    pub fn error_code(&self) -> ErrorCode {
        match self {
            LunaError::Audio(_) => ErrorCode::AudioCaptureFailure,
            LunaError::WakeWord(_) => ErrorCode::WakeWordDetectionFailed,
            LunaError::SpeechRecognition(_) => ErrorCode::SttTranscriptionFailed,
            LunaError::CommandParsing(_) => ErrorCode::CommandParseFailure,
            LunaError::AppNotFound(_) => ErrorCode::AppNotFound,
            LunaError::FileNotFound(_) => ErrorCode::FileNotFound,
            LunaError::InvalidParameter(_) => ErrorCode::CommandParseFailure,
            LunaError::SystemOperation(_) => ErrorCode::SystemOperationFailed,
            LunaError::Config(_) => ErrorCode::ConfigInvalid,
            LunaError::Database(_) => ErrorCode::DatabaseLoadFailed,
            LunaError::WindowControl(_) => ErrorCode::SystemOperationFailed,
            LunaError::Tts(_) => ErrorCode::SystemOperationFailed,
            LunaError::Context(_) => ErrorCode::Unknown,
            LunaError::Io(_) => ErrorCode::FileOperationFailed,
            LunaError::Json(_) | LunaError::Toml(_) => ErrorCode::ConfigInvalid,
            LunaError::Wav(_) => ErrorCode::AudioCaptureFailure,
            LunaError::Unknown(_) => ErrorCode::Unknown,
        }
    }

    /// Get error category
    pub fn category(&self) -> &'static str {
        self.error_code().category()
    }

    /// Check if error is recoverable (can retry)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            LunaError::Audio(_)
                | LunaError::WakeWord(_)
                | LunaError::SpeechRecognition(_)
                | LunaError::SystemOperation(_)
        )
    }

    /// Check if error requires user notification
    pub fn requires_notification(&self) -> bool {
        matches!(
            self,
            LunaError::AppNotFound(_)
                | LunaError::FileNotFound(_)
                | LunaError::CommandParsing(_)
                | LunaError::SystemOperation(_)
        )
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            LunaError::AppNotFound(app) => {
                format!("I couldn't find the application '{}'", app)
            }
            LunaError::FileNotFound(file) => {
                format!("I couldn't find the file '{}'", file)
            }
            LunaError::CommandParsing(_) => {
                "I didn't understand that command. Can you try again?".to_string()
            }
            LunaError::SpeechRecognition(_) => {
                "I couldn't hear you clearly. Please try again.".to_string()
            }
            LunaError::SystemOperation(op) => {
                format!("I couldn't perform that operation: {}", op)
            }
            _ => "An error occurred. Please try again.".to_string(),
        }
    }

    /// Get debug context (detailed technical info)
    pub fn debug_context(&self) -> HashMap<String, String> {
        let mut context = HashMap::new();
        context.insert(
            "error_code".to_string(),
            self.error_code().as_u32().to_string(),
        );
        context.insert("category".to_string(), self.category().to_string());
        context.insert("recoverable".to_string(), self.is_recoverable().to_string());
        context.insert("error_message".to_string(), self.to_string());
        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = LunaError::AppNotFound("Chrome".to_string());
        assert_eq!(err.to_string(), "Application not found: 'Chrome'");
    }

    #[test]
    fn test_error_recoverable() {
        assert!(LunaError::Audio("test".to_string()).is_recoverable());
        assert!(!LunaError::Config("test".to_string()).is_recoverable());
    }

    #[test]
    fn test_user_message() {
        let err = LunaError::AppNotFound("VS Code".to_string());
        assert!(err.user_message().contains("VS Code"));
    }

    #[test]
    fn test_error_macros() {
        let err = audio_error!("Failed to capture: {}", "device busy");
        match err {
            LunaError::Audio(msg) => assert!(msg.contains("device busy")),
            _ => panic!("Wrong error type"),
        }
    }
}
