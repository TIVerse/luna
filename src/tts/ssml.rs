//! SSML-lite parser
//!
//! Simplified SSML support for enhanced speech synthesis.

use regex::Regex;
use once_cell::sync::Lazy;

static BREAK_TAG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"<break\s+(?:time|strength)="([^"]+)"\s*/>"#).unwrap()
});

static EMPHASIS_TAG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"<emphasis[^>]*>(.*?)</emphasis>"#).unwrap()
});

static SAY_AS_TAG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"<say-as\s+interpret-as="([^"]+)"[^>]*>(.*?)</say-as>"#).unwrap()
});

static PROSODY_TAG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"<prosody[^>]*>(.*?)</prosody>"#).unwrap()
});

/// Parse SSML-lite text into chunks with metadata
#[derive(Debug, Clone)]
pub struct SsmlChunk {
    pub text: String,
    pub break_ms: Option<u32>,
    pub rate_modifier: f32,
    pub pitch_modifier: f32,
    pub emphasis: bool,
}

impl Default for SsmlChunk {
    fn default() -> Self {
        Self {
            text: String::new(),
            break_ms: None,
            rate_modifier: 1.0,
            pitch_modifier: 1.0,
            emphasis: false,
        }
    }
}

/// Parse SSML-lite markup into chunks
pub fn parse_ssml(text: &str) -> Vec<SsmlChunk> {
    let mut chunks = Vec::new();
    let mut remaining = text.to_string();
    
    // Simple sequential processing - handle breaks first
    while let Some(caps) = BREAK_TAG.captures(&remaining) {
        let before = &remaining[..caps.get(0).unwrap().start()];
        if !before.is_empty() {
            chunks.push(SsmlChunk {
                text: before.to_string(),
                ..Default::default()
            });
        }
        
        // Parse break duration
        let duration = caps.get(1).unwrap().as_str();
        let break_ms = parse_break_duration(duration);
        chunks.push(SsmlChunk {
            text: String::new(),
            break_ms: Some(break_ms),
            ..Default::default()
        });
        
        remaining = remaining[caps.get(0).unwrap().end()..].to_string();
    }
    
    // Handle emphasis
    if let Some(caps) = EMPHASIS_TAG.captures(&remaining) {
        let emphasized = caps.get(1).unwrap().as_str();
        chunks.push(SsmlChunk {
            text: emphasized.to_string(),
            emphasis: true,
            rate_modifier: 0.9,
            pitch_modifier: 1.1,
            ..Default::default()
        });
        remaining = EMPHASIS_TAG.replace(&remaining, "$1").to_string();
    }
    
    // Handle say-as
    while let Some(caps) = SAY_AS_TAG.captures(&remaining) {
        let interpret_as = caps.get(1).unwrap().as_str();
        let content = caps.get(2).unwrap().as_str();
        
        let processed = match interpret_as {
            "characters" | "spell-out" => content.chars()
                .map(|c| format!("{} ", c))
                .collect::<String>(),
            "digits" => content.chars()
                .filter(|c| c.is_numeric())
                .map(|c| format!("{} ", c))
                .collect::<String>(),
            _ => content.to_string(),
        };
        
        remaining = SAY_AS_TAG.replace(&remaining, &processed).to_string();
    }
    
    // Add final chunk if any text remains
    if !remaining.is_empty() {
        let clean = strip_ssml_tags(&remaining);
        if !clean.is_empty() {
            chunks.push(SsmlChunk {
                text: clean,
                ..Default::default()
            });
        }
    }
    
    chunks
}

/// Parse break duration string to milliseconds
fn parse_break_duration(duration: &str) -> u32 {
    if duration.ends_with("ms") {
        duration.trim_end_matches("ms")
            .parse()
            .unwrap_or(200)
    } else if duration.ends_with("s") {
        duration.trim_end_matches("s")
            .parse::<f32>()
            .unwrap_or(0.2)
            .mul_add(1000.0, 0.0) as u32
    } else {
        // Named strengths
        match duration {
            "none" => 0,
            "x-weak" => 50,
            "weak" => 100,
            "medium" => 200,
            "strong" => 400,
            "x-strong" => 800,
            _ => 200,
        }
    }
}

/// Strip all SSML tags and return plain text
pub fn strip_ssml_tags(text: &str) -> String {
    let mut result = text.to_string();
    
    // Remove all tags
    let tag_regex = Regex::new(r"<[^>]+>").unwrap();
    result = tag_regex.replace_all(&result, "").to_string();
    
    // Collapse whitespace
    let ws_regex = Regex::new(r"\s+").unwrap();
    result = ws_regex.replace_all(&result, " ").to_string();
    
    result.trim().to_string()
}

/// Split text into natural sentence/phrase chunks for streaming
pub fn chunk_text(text: &str, max_chunk_len: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    
    // Split on sentence boundaries
    for sentence in text.split_inclusive(&['.', '!', '?', ';']) {
        if current.len() + sentence.len() > max_chunk_len && !current.is_empty() {
            chunks.push(current.trim().to_string());
            current = String::new();
        }
        current.push_str(sentence);
    }
    
    if !current.is_empty() {
        chunks.push(current.trim().to_string());
    }
    
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_strip_tags() {
        let ssml = r#"Hello <break time="500ms"/> world!"#;
        let plain = strip_ssml_tags(ssml);
        assert_eq!(plain, "Hello world!");
    }
    
    #[test]
    fn test_parse_break() {
        assert_eq!(parse_break_duration("500ms"), 500);
        assert_eq!(parse_break_duration("1s"), 1000);
        assert_eq!(parse_break_duration("medium"), 200);
    }
    
    #[test]
    fn test_chunk_text() {
        let text = "This is sentence one. This is sentence two! And three?";
        let chunks = chunk_text(text, 30);
        assert_eq!(chunks.len(), 3);
    }
    
    #[test]
    fn test_parse_ssml_with_break() {
        let ssml = r#"Hello<break time="500ms"/>world"#;
        let chunks = parse_ssml(ssml);
        assert!(chunks.len() >= 2);
        assert!(chunks.iter().any(|c| c.break_ms == Some(500)));
    }
}
