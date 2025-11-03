//! Intent classification
//!
//! Classifies commands with confidence scores using pattern matching and heuristics.

use crate::brain::command_parser::{IntentType, ParsedCommand};
use crate::error::Result;
use crate::utils::string_matching;
use std::collections::HashMap;
use tracing::{debug, info};

/// Classification result with confidence and fallback intents
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    /// Primary intent
    pub intent: IntentType,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Extracted entities
    pub entities: HashMap<String, String>,
    /// Alternative intents with their confidence scores
    pub alternatives: Vec<(IntentType, f32)>,
}

/// Intent classifier with confidence scoring
pub struct IntentClassifier {
    /// Application database for app name validation
    known_apps: Vec<String>,
}

impl IntentClassifier {
    /// Create a new intent classifier
    pub fn new() -> Self {
        Self {
            known_apps: Self::default_known_apps(),
        }
    }

    /// Classify a parsed command and compute confidence
    pub fn classify(&self, command: &ParsedCommand) -> Result<ClassificationResult> {
        let mut confidence = command.confidence;
        let mut alternatives = Vec::new();

        // Boost confidence if entities are validated
        if let Some(app_name) = command.entities.get("app_name") {
            if self.validate_app_name(app_name) {
                confidence = (confidence * 1.1).min(1.0);
                debug!("Boosted confidence for known app: {}", app_name);
            } else {
                // Check if it might be a file instead
                if app_name.contains('.') {
                    alternatives.push((IntentType::FindFile, 0.7));
                    debug!("Might be a file instead of app: {}", app_name);
                }
            }
        }

        // Validate file names
        if let Some(file_name) = command.entities.get("file_name") {
            if self.looks_like_filename(file_name) {
                confidence = (confidence * 1.05).min(1.0);
            }
        }

        // Add context-based alternatives
        alternatives.extend(self.compute_alternatives(&command.intent, &command.entities));

        info!(
            "Classified intent: {:?} with confidence: {:.2}",
            command.intent, confidence
        );

        Ok(ClassificationResult {
            intent: command.intent.clone(),
            confidence,
            entities: command.entities.clone(),
            alternatives,
        })
    }

    /// Validate if a string is likely an app name
    fn validate_app_name(&self, name: &str) -> bool {
        let normalized = name.to_lowercase();

        // Check against known apps
        if self
            .known_apps
            .iter()
            .any(|app| string_matching::fuzzy_match(&normalized, &app.to_lowercase(), 0.3))
        {
            return true;
        }

        // Common app name patterns
        let common_apps = [
            "chrome",
            "firefox",
            "edge",
            "safari",
            "code",
            "vscode",
            "sublime",
            "atom",
            "terminal",
            "iterm",
            "cmd",
            "powershell",
            "slack",
            "discord",
            "teams",
            "zoom",
            "spotify",
            "vlc",
            "itunes",
            "music",
        ];

        common_apps.iter().any(|&app| normalized.contains(app))
    }

    /// Check if string looks like a filename
    fn looks_like_filename(&self, name: &str) -> bool {
        // Has file extension
        if name.contains('.') {
            let parts: Vec<&str> = name.split('.').collect();
            if parts.len() >= 2 {
                let ext = parts.last().unwrap();
                return ext.len() <= 5 && ext.chars().all(|c| c.is_alphanumeric());
            }
        }
        false
    }

    /// Compute alternative intents based on context
    fn compute_alternatives(
        &self,
        intent: &IntentType,
        entities: &HashMap<String, String>,
    ) -> Vec<(IntentType, f32)> {
        let mut alternatives = Vec::new();

        match intent {
            IntentType::LaunchApp => {
                // Might also want to find the app first
                alternatives.push((IntentType::FindFile, 0.3));
            }
            IntentType::FindFile => {
                // Might want to open it instead
                alternatives.push((IntentType::LaunchApp, 0.4));
                alternatives.push((IntentType::OpenFolder, 0.3));
            }
            IntentType::Question => {
                // Might be a web search
                alternatives.push((IntentType::SearchWeb, 0.6));
            }
            _ => {}
        }

        alternatives
    }

    /// Default list of known applications
    fn default_known_apps() -> Vec<String> {
        vec![
            "chrome".to_string(),
            "firefox".to_string(),
            "safari".to_string(),
            "edge".to_string(),
            "vscode".to_string(),
            "code".to_string(),
            "terminal".to_string(),
            "slack".to_string(),
            "discord".to_string(),
            "spotify".to_string(),
        ]
    }

    /// Add a known app name
    pub fn add_known_app(&mut self, app_name: String) {
        if !self.known_apps.contains(&app_name.to_lowercase()) {
            self.known_apps.push(app_name.to_lowercase());
        }
    }

    /// Bulk add known apps
    pub fn add_known_apps(&mut self, apps: Vec<String>) {
        for app in apps {
            self.add_known_app(app);
        }
    }
}

impl Default for IntentClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brain::command_parser::ParsedCommand;

    fn create_test_command(intent: IntentType, entities: HashMap<String, String>) -> ParsedCommand {
        ParsedCommand {
            intent,
            entities,
            original_text: "test".to_string(),
            confidence: 0.9,
        }
    }

    #[test]
    fn test_classify_known_app() {
        let classifier = IntentClassifier::new();
        let mut entities = HashMap::new();
        entities.insert("app_name".to_string(), "chrome".to_string());

        let command = create_test_command(IntentType::LaunchApp, entities);
        let result = classifier.classify(&command).unwrap();

        assert!(result.confidence > 0.9);
        assert_eq!(result.intent, IntentType::LaunchApp);
    }

    #[test]
    fn test_classify_unknown_app() {
        let classifier = IntentClassifier::new();
        let mut entities = HashMap::new();
        entities.insert("app_name".to_string(), "unknownapp123".to_string());

        let command = create_test_command(IntentType::LaunchApp, entities);
        let result = classifier.classify(&command).unwrap();

        // Confidence should not be boosted for unknown apps
        assert_eq!(result.confidence, 0.9);
    }

    #[test]
    fn test_looks_like_filename() {
        let classifier = IntentClassifier::new();

        assert!(classifier.looks_like_filename("document.pdf"));
        assert!(classifier.looks_like_filename("report.docx"));
        assert!(!classifier.looks_like_filename("chrome"));
        assert!(!classifier.looks_like_filename("some text"));
    }

    #[test]
    fn test_add_known_app() {
        let mut classifier = IntentClassifier::new();
        classifier.add_known_app("myapp".to_string());

        assert!(classifier.validate_app_name("myapp"));
        assert!(classifier.validate_app_name("MyApp")); // Case insensitive
    }

    #[test]
    fn test_alternatives() {
        let classifier = IntentClassifier::new();
        let command = create_test_command(IntentType::LaunchApp, HashMap::new());
        let result = classifier.classify(&command).unwrap();

        assert!(!result.alternatives.is_empty());
    }
}
