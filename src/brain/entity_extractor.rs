//! Entity extraction
//!
//! Extracts specific entities (app names, file names, numbers, durations) from text.

use crate::error::Result;
use crate::utils::time_helpers;
use chrono::Duration;
use regex::Regex;
use once_cell::sync::Lazy;

static NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(\d+)\b").unwrap());
static APP_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b([a-zA-Z][a-zA-Z0-9_-]*(?:\s+[a-zA-Z][a-zA-Z0-9_-]*)*)\b").unwrap()
});

/// Entity extractor for parsing specific information from text
#[derive(Clone)]
pub struct EntityExtractor;

impl EntityExtractor {
    /// Create a new entity extractor
    pub fn new() -> Self {
        Self
    }
    
    /// Extract app name from text
    pub fn extract_app_name(&self, text: &str) -> Option<String> {
        // Remove common words that aren't part of app names
        let filtered = self.remove_common_words(text);
        
        if filtered.is_empty() {
            return None;
        }
        
        Some(filtered.trim().to_string())
    }
    
    /// Extract file name from text
    pub fn extract_file_name(&self, text: &str) -> Option<String> {
        let text = text.trim();
        
        // If it has an extension, it's likely a file
        if text.contains('.') {
            return Some(text.to_string());
        }
        
        // Otherwise, try to extract the filename part
        let filtered = self.remove_common_words(text);
        if !filtered.is_empty() {
            Some(filtered)
        } else {
            Some(text.to_string())
        }
    }
    
    /// Extract duration from text (e.g., "30 minutes", "2 hours")
    pub fn extract_duration(&self, text: &str) -> Option<Duration> {
        time_helpers::parse_duration(text)
    }
    
    /// Extract a number from text
    pub fn extract_number(&self, text: &str) -> Option<i32> {
        NUMBER_REGEX
            .captures(text)
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }
    
    /// Extract percentage from text (e.g., "50%", "75 percent")
    pub fn extract_percentage(&self, text: &str) -> Option<u8> {
        // Pattern: "50%" or "50 percent"
        let percent_regex = Regex::new(r"(\d+)\s*(?:%|percent)").unwrap();
        
        percent_regex
            .captures(text)
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse().ok())
            .map(|n: u8| n.min(100))
    }
    
    /// Extract folder name from text
    pub fn extract_folder_name(&self, text: &str) -> Option<String> {
        let filtered = self.remove_common_words(text);
        
        if filtered.is_empty() {
            None
        } else {
            Some(filtered)
        }
    }
    
    /// Extract search query from text
    pub fn extract_search_query(&self, text: &str) -> Option<String> {
        if text.trim().is_empty() {
            None
        } else {
            Some(text.trim().to_string())
        }
    }
    
    /// Remove common filler words that aren't part of entity names
    fn remove_common_words(&self, text: &str) -> String {
        let common_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at",
            "to", "for", "of", "with", "by", "from", "up", "about",
            "into", "through", "during", "before", "after", "above",
            "below", "between", "under", "again", "further", "then",
            "once", "application", "app", "program", "file", "folder",
            "directory", "document",
        ];
        
        text.split_whitespace()
            .filter(|word| {
                let lower = word.to_lowercase();
                !common_words.contains(&lower.as_str())
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Extract time of day (e.g., "3pm", "14:30")
    pub fn extract_time_of_day(&self, text: &str) -> Option<(u8, u8)> {
        // Pattern: "3pm", "3:30pm", "15:30"
        let time_regex = Regex::new(r"(\d{1,2})(?::(\d{2}))?\s*(am|pm)?").unwrap();
        
        if let Some(caps) = time_regex.captures(text) {
            let hour: u8 = caps.get(1)?.as_str().parse().ok()?;
            let minute: u8 = caps.get(2)
                .map(|m| m.as_str().parse().ok())
                .flatten()
                .unwrap_or(0);
            
            let am_pm = caps.get(3).map(|m| m.as_str().to_lowercase());
            
            let hour = match am_pm.as_deref() {
                Some("pm") if hour < 12 => hour + 12,
                Some("am") if hour == 12 => 0,
                _ => hour,
            };
            
            if hour < 24 && minute < 60 {
                return Some((hour, minute));
            }
        }
        
        None
    }
}

impl Default for EntityExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_app_name() {
        let extractor = EntityExtractor::new();
        
        assert_eq!(
            extractor.extract_app_name("chrome"),
            Some("chrome".to_string())
        );
        
        assert_eq!(
            extractor.extract_app_name("visual studio code"),
            Some("visual studio code".to_string())
        );
        
        // Should filter common words
        assert_eq!(
            extractor.extract_app_name("the chrome application"),
            Some("chrome".to_string())
        );
    }

    #[test]
    fn test_extract_file_name() {
        let extractor = EntityExtractor::new();
        
        assert_eq!(
            extractor.extract_file_name("budget.pdf"),
            Some("budget.pdf".to_string())
        );
        
        assert_eq!(
            extractor.extract_file_name("my document"),
            Some("my".to_string())
        );
    }

    #[test]
    fn test_extract_duration() {
        let extractor = EntityExtractor::new();
        
        assert_eq!(
            extractor.extract_duration("30 minutes"),
            Some(Duration::minutes(30))
        );
        
        assert_eq!(
            extractor.extract_duration("2 hours"),
            Some(Duration::hours(2))
        );
        
        assert_eq!(
            extractor.extract_duration("5m"),
            Some(Duration::minutes(5))
        );
    }

    #[test]
    fn test_extract_number() {
        let extractor = EntityExtractor::new();
        
        assert_eq!(extractor.extract_number("set volume to 50"), Some(50));
        assert_eq!(extractor.extract_number("open 3 windows"), Some(3));
        assert_eq!(extractor.extract_number("no numbers here"), None);
    }

    #[test]
    fn test_extract_percentage() {
        let extractor = EntityExtractor::new();
        
        assert_eq!(extractor.extract_percentage("50%"), Some(50));
        assert_eq!(extractor.extract_percentage("75 percent"), Some(75));
        assert_eq!(extractor.extract_percentage("150%"), Some(100)); // Capped at 100
    }

    #[test]
    fn test_extract_time_of_day() {
        let extractor = EntityExtractor::new();
        
        assert_eq!(extractor.extract_time_of_day("3pm"), Some((15, 0)));
        assert_eq!(extractor.extract_time_of_day("3:30pm"), Some((15, 30)));
        assert_eq!(extractor.extract_time_of_day("14:30"), Some((14, 30)));
        assert_eq!(extractor.extract_time_of_day("12am"), Some((0, 0)));
    }

    #[test]
    fn test_remove_common_words() {
        let extractor = EntityExtractor::new();
        
        let result = extractor.remove_common_words("the chrome browser");
        assert_eq!(result, "chrome browser");
        
        let result = extractor.remove_common_words("open the file manager");
        assert_eq!(result, "open manager");
    }
}
