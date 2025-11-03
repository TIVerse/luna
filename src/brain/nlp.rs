//! Natural language processing utilities
//!
//! Helper functions for text processing and NLP tasks.

/// NLP utility functions
pub struct NlpUtils;

impl NlpUtils {
    /// Tokenize text into words
    pub fn tokenize(text: &str) -> Vec<String> {
        text.split_whitespace().map(|s| s.to_string()).collect()
    }

    /// Remove punctuation from text
    pub fn remove_punctuation(text: &str) -> String {
        text.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    /// Normalize text (lowercase, trim, remove extra spaces)
    pub fn normalize(text: &str) -> String {
        text.to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Calculate text similarity using simple token overlap
    pub fn text_similarity(text1: &str, text2: &str) -> f32 {
        let tokens1: Vec<String> = Self::tokenize(&Self::normalize(text1));
        let tokens2: Vec<String> = Self::tokenize(&Self::normalize(text2));

        if tokens1.is_empty() || tokens2.is_empty() {
            return 0.0;
        }

        let common = tokens1.iter().filter(|t| tokens2.contains(t)).count();

        let total = tokens1.len() + tokens2.len();
        (2.0 * common as f32) / total as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = NlpUtils::tokenize("open chrome browser");
        assert_eq!(tokens, vec!["open", "chrome", "browser"]);
    }

    #[test]
    fn test_remove_punctuation() {
        let result = NlpUtils::remove_punctuation("Hello, world!");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_normalize() {
        let result = NlpUtils::normalize("  Hello   World  ");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_text_similarity() {
        let sim = NlpUtils::text_similarity("open chrome", "launch chrome");
        assert!(sim > 0.0);

        let sim = NlpUtils::text_similarity("open chrome", "close firefox");
        assert!(sim < 0.5);
    }
}
