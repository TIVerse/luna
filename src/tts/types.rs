//! TTS type definitions
//!
//! Message kinds, profiles, policies, and configurations.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Priority level for TTS messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Background = 0,
    Info = 1,
    Prompt = 2,
    Critical = 3,
}

/// Kind of TTS message - determines voice profile and behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageKind {
    /// System alerts and critical errors
    Critical,
    /// Error messages
    Error,
    /// Confirmation messages
    Confirmation,
    /// User prompts and questions
    Prompt,
    /// Reading file content or long text
    Reading,
    /// Information and status updates
    Info,
    /// Background notifications
    Background,
}

impl MessageKind {
    /// Get default priority for this message kind
    pub fn default_priority(&self) -> Priority {
        match self {
            Self::Critical => Priority::Critical,
            Self::Error => Priority::Critical,
            Self::Confirmation => Priority::Prompt,
            Self::Prompt => Priority::Prompt,
            Self::Reading => Priority::Info,
            Self::Info => Priority::Info,
            Self::Background => Priority::Background,
        }
    }
    
    /// Can this message be interrupted?
    pub fn is_interruptible(&self) -> bool {
        match self {
            Self::Critical => false,
            Self::Error => false,
            Self::Prompt => true,
            Self::Reading => true,
            Self::Confirmation => true,
            Self::Info => true,
            Self::Background => true,
        }
    }
}

/// Voice profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceProfile {
    /// Voice ID or name
    pub voice: Option<String>,
    /// Speech rate (0.1 - 10.0, 1.0 = normal)
    pub rate: f32,
    /// Speech pitch (0.0 - 2.0, 1.0 = normal)
    pub pitch: f32,
    /// Volume (0.0 - 1.0)
    pub volume: f32,
    /// Pre-utterance earcon
    pub pre_earcon: Option<PathBuf>,
    /// Post-utterance earcon
    pub post_earcon: Option<PathBuf>,
}

impl Default for VoiceProfile {
    fn default() -> Self {
        Self {
            voice: None,
            rate: 1.0,
            pitch: 1.0,
            volume: 1.0,
            pre_earcon: None,
            post_earcon: None,
        }
    }
}

/// TTS policy - maps message kinds to profiles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TtsPolicy {
    pub critical: VoiceProfile,
    pub error: VoiceProfile,
    pub confirmation: VoiceProfile,
    pub prompt: VoiceProfile,
    pub reading: VoiceProfile,
    pub info: VoiceProfile,
    pub background: VoiceProfile,
}

impl TtsPolicy {
    /// Get profile for a message kind
    pub fn profile_for(&self, kind: MessageKind) -> &VoiceProfile {
        match kind {
            MessageKind::Critical => &self.critical,
            MessageKind::Error => &self.error,
            MessageKind::Confirmation => &self.confirmation,
            MessageKind::Prompt => &self.prompt,
            MessageKind::Reading => &self.reading,
            MessageKind::Info => &self.info,
            MessageKind::Background => &self.background,
        }
    }
}

/// TTS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    /// Engine to use: "os", "coqui", etc.
    pub engine: String,
    /// Default voice ID
    pub default_voice: Option<String>,
    /// Default rate
    pub rate: f32,
    /// Default pitch
    pub pitch: f32,
    /// Default volume
    pub volume: f32,
    /// Enable barge-in (interrupt on user speech)
    pub barge_in: bool,
    /// Duck system audio while speaking
    pub duck_system_audio: bool,
    /// Enable earcons
    pub earcons_enabled: bool,
    /// Voice profiles and policies
    pub policy: TtsPolicy,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            engine: "os".to_string(),
            default_voice: None,
            rate: 1.0,
            pitch: 1.0,
            volume: 0.8,
            barge_in: true,
            duck_system_audio: false,
            earcons_enabled: false,
            policy: TtsPolicy::default(),
        }
    }
}

/// Cancellation handle for TTS utterances
#[derive(Debug, Clone)]
pub struct TtsHandle {
    id: u64,
    cancel_tx: tokio::sync::mpsc::UnboundedSender<u64>,
}

impl TtsHandle {
    pub fn new(id: u64, cancel_tx: tokio::sync::mpsc::UnboundedSender<u64>) -> Self {
        Self { id, cancel_tx }
    }
    
    /// Cancel this utterance
    pub fn cancel(&self) {
        let _ = self.cancel_tx.send(self.id);
    }
    
    /// Get utterance ID
    pub fn id(&self) -> u64 {
        self.id
    }
}
