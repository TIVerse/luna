//! Core utilities for the LUNA assistant.
//!
//! Provides string matching, path helpers, time parsing, and other common utilities.

use crate::error::{LunaError, Result};
use std::path::{Path, PathBuf};

/// String matching utilities for fuzzy search and phonetic matching
pub mod string_matching {
    use strsim::levenshtein;

    /// Fuzzy string matching using Levenshtein distance
    ///
    /// # Arguments
    /// * `input` - The input string to match
    /// * `target` - The target string to compare against
    /// * `threshold` - Maximum distance ratio (0.0 = exact, 1.0 = any)
    ///
    /// # Returns
    /// `true` if strings are similar within threshold
    pub fn fuzzy_match(input: &str, target: &str, threshold: f32) -> bool {
        let input = normalize(input);
        let target = normalize(target);

        if input == target {
            return true;
        }

        let distance = levenshtein(&input, &target);
        let max_len = input.len().max(target.len()) as f32;

        if max_len == 0.0 {
            return false;
        }

        let similarity = 1.0 - (distance as f32 / max_len);
        similarity >= (1.0 - threshold)
    }

    /// Check if input contains target as substring (normalized)
    pub fn contains_match(input: &str, target: &str) -> bool {
        normalize(input).contains(&normalize(target))
    }

    /// Phonetic matching using simplified Soundex algorithm
    ///
    /// # Arguments
    /// * `input` - The input string
    /// * `target` - The target string
    ///
    /// # Returns
    /// `true` if strings sound similar
    pub fn phonetic_match(input: &str, target: &str) -> bool {
        soundex(input) == soundex(target)
    }

    /// Normalize text for comparison (lowercase, alphanumeric only)
    ///
    /// # Arguments
    /// * `text` - The text to normalize
    ///
    /// # Returns
    /// Normalized string
    pub fn normalize(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' || c == '_' {
                    ' '
                } else {
                    ' '
                }
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Simplified Soundex algorithm for phonetic encoding
    fn soundex(s: &str) -> String {
        let s = s.to_uppercase();
        let chars: Vec<char> = s.chars().filter(|c| c.is_alphabetic()).collect();

        if chars.is_empty() {
            return String::from("0000");
        }

        let mut code = String::new();
        code.push(chars[0]);

        let get_code = |c: char| -> char {
            match c {
                'B' | 'F' | 'P' | 'V' => '1',
                'C' | 'G' | 'J' | 'K' | 'Q' | 'S' | 'X' | 'Z' => '2',
                'D' | 'T' => '3',
                'L' => '4',
                'M' | 'N' => '5',
                'R' => '6',
                _ => '0',
            }
        };

        let mut prev_code = get_code(chars[0]);

        for &c in &chars[1..] {
            let curr_code = get_code(c);
            if curr_code != '0' && curr_code != prev_code {
                code.push(curr_code);
                if code.len() >= 4 {
                    break;
                }
            }
            prev_code = curr_code;
        }

        // Pad with zeros
        while code.len() < 4 {
            code.push('0');
        }

        code.truncate(4);
        code
    }

    /// Calculate similarity score between two strings (0.0 - 1.0)
    pub fn similarity_score(input: &str, target: &str) -> f32 {
        let input = normalize(input);
        let target = normalize(target);

        if input == target {
            return 1.0;
        }

        let distance = levenshtein(&input, &target);
        let max_len = input.len().max(target.len()) as f32;

        if max_len == 0.0 {
            return 0.0;
        }

        1.0 - (distance as f32 / max_len)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_fuzzy_match() {
            assert!(fuzzy_match("chrome", "chrome", 0.2));
            assert!(fuzzy_match("vscode", "vs code", 0.4));
            assert!(!fuzzy_match("firefox", "chrome", 0.2));
        }

        #[test]
        fn test_normalize() {
            assert_eq!(normalize("  Hello   World!  "), "hello world");
            assert_eq!(normalize("VS-Code"), "vs code");
        }

        #[test]
        fn test_phonetic_match() {
            assert!(phonetic_match("smith", "smyth"));
            assert!(phonetic_match("john", "jon"));
        }

        #[test]
        fn test_similarity_score() {
            assert!(similarity_score("test", "test") == 1.0);
            assert!(similarity_score("test", "text") > 0.7);
            assert!(similarity_score("abc", "xyz") < 0.3);
        }
    }
}

/// Path and file system utilities
pub mod path_helpers {
    use super::*;

    /// Get common application directories for the current platform
    pub fn get_app_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        #[cfg(target_os = "linux")]
        {
            dirs.extend(vec![
                PathBuf::from("/usr/share/applications"),
                PathBuf::from("/usr/local/share/applications"),
            ]);

            if let Some(home) = dirs::home_dir() {
                dirs.push(home.join(".local/share/applications"));
            }
        }

        #[cfg(target_os = "windows")]
        {
            dirs.extend(vec![
                PathBuf::from("C:\\Program Files"),
                PathBuf::from("C:\\Program Files (x86)"),
            ]);

            if let Some(home) = dirs::home_dir() {
                dirs.push(home.join("AppData\\Local\\Programs"));
                dirs.push(home.join("AppData\\Roaming\\Microsoft\\Windows\\Start Menu"));
            }
        }

        #[cfg(target_os = "macos")]
        {
            dirs.push(PathBuf::from("/Applications"));
            if let Some(home) = dirs::home_dir() {
                dirs.push(home.join("Applications"));
            }
        }

        dirs
    }

    /// Get common project directories
    pub fn get_project_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        if let Some(home) = dirs::home_dir() {
            dirs.extend(vec![
                home.join("Projects"),
                home.join("dev"),
                home.join("Documents"),
                home.join("workspace"),
                home.join("code"),
            ]);
        }

        // Filter to only existing directories
        dirs.into_iter().filter(|p| p.exists()).collect()
    }

    /// Get common document directories
    pub fn get_document_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        if let Some(home) = dirs::home_dir() {
            dirs.extend(vec![
                home.join("Documents"),
                home.join("Desktop"),
                home.join("Downloads"),
            ]);
        }

        dirs.into_iter().filter(|p| p.exists()).collect()
    }

    /// Recursively search directory for files matching a predicate
    ///
    /// # Arguments
    /// * `root` - Root directory to search
    /// * `predicate` - Function to test each path
    /// * `max_depth` - Maximum recursion depth (None for unlimited)
    pub fn search_recursive<F>(root: &Path, predicate: F, max_depth: Option<usize>) -> Vec<PathBuf>
    where
        F: Fn(&Path) -> bool + Copy,
    {
        search_recursive_impl(root, predicate, max_depth, 0)
    }

    fn search_recursive_impl<F>(
        root: &Path,
        predicate: F,
        max_depth: Option<usize>,
        current_depth: usize,
    ) -> Vec<PathBuf>
    where
        F: Fn(&Path) -> bool + Copy,
    {
        let mut results = Vec::new();

        if let Some(max) = max_depth {
            if current_depth >= max {
                return results;
            }
        }

        let entries = match std::fs::read_dir(root) {
            Ok(e) => e,
            Err(_) => return results,
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if predicate(&path) {
                results.push(path.clone());
            }

            if path.is_dir() {
                results.extend(search_recursive_impl(
                    &path,
                    predicate,
                    max_depth,
                    current_depth + 1,
                ));
            }
        }

        results
    }

    /// Check if path should be excluded from search
    pub fn should_exclude(path: &Path) -> bool {
        let exclude_names = [
            "node_modules",
            ".git",
            "target",
            ".cache",
            "__pycache__",
            ".venv",
            "venv",
        ];

        path.file_name()
            .and_then(|n| n.to_str())
            .map(|name| exclude_names.iter().any(|&ex| name == ex))
            .unwrap_or(false)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_get_app_dirs() {
            let dirs = get_app_dirs();
            assert!(!dirs.is_empty());
        }

        #[test]
        fn test_should_exclude() {
            assert!(should_exclude(Path::new("/home/user/project/node_modules")));
            assert!(should_exclude(Path::new("/home/user/project/.git")));
            assert!(!should_exclude(Path::new("/home/user/project/src")));
        }
    }
}

/// Time and duration parsing utilities
pub mod time_helpers {
    use chrono::Duration;

    /// Parse human-readable duration strings
    ///
    /// # Supported formats
    /// - "30 seconds", "30s", "30 sec"
    /// - "5 minutes", "5m", "5 min"
    /// - "2 hours", "2h", "2 hr"
    /// - "1 day", "1d"
    ///
    /// # Arguments
    /// * `input` - The duration string to parse
    ///
    /// # Returns
    /// `Some(Duration)` if parsing succeeds, `None` otherwise
    pub fn parse_duration(input: &str) -> Option<Duration> {
        let input = input.trim().to_lowercase();
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return None;
        }

        // Try to parse number
        let number: i64 = if parts.len() == 1 {
            // Format like "30s"
            let digits: String = parts[0].chars().take_while(|c| c.is_numeric()).collect();
            digits.parse().ok()?
        } else {
            // Format like "30 seconds"
            parts[0].parse().ok()?
        };

        // Determine unit
        let unit = if parts.len() > 1 {
            parts[1]
        } else {
            parts[0].trim_start_matches(|c: char| c.is_numeric())
        };

        match unit {
            "s" | "sec" | "second" | "seconds" => Some(Duration::seconds(number)),
            "m" | "min" | "minute" | "minutes" => Some(Duration::minutes(number)),
            "h" | "hr" | "hour" | "hours" => Some(Duration::hours(number)),
            "d" | "day" | "days" => Some(Duration::days(number)),
            _ => None,
        }
    }

    /// Format duration in human-readable form
    pub fn format_duration(duration: Duration) -> String {
        let seconds = duration.num_seconds();

        if seconds < 60 {
            format!("{} seconds", seconds)
        } else if seconds < 3600 {
            let minutes = seconds / 60;
            format!("{} minute{}", minutes, if minutes != 1 { "s" } else { "" })
        } else if seconds < 86400 {
            let hours = seconds / 3600;
            format!("{} hour{}", hours, if hours != 1 { "s" } else { "" })
        } else {
            let days = seconds / 86400;
            format!("{} day{}", days, if days != 1 { "s" } else { "" })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_duration() {
            assert_eq!(
                parse_duration("30 seconds"),
                Some(Duration::seconds(30))
            );
            assert_eq!(parse_duration("5 minutes"), Some(Duration::minutes(5)));
            assert_eq!(parse_duration("2 hours"), Some(Duration::hours(2)));
            assert_eq!(parse_duration("30s"), Some(Duration::seconds(30)));
            assert_eq!(parse_duration("5m"), Some(Duration::minutes(5)));
        }

        #[test]
        fn test_format_duration() {
            assert_eq!(format_duration(Duration::seconds(30)), "30 seconds");
            assert_eq!(format_duration(Duration::minutes(5)), "5 minutes");
            assert_eq!(format_duration(Duration::hours(2)), "2 hours");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_matching_module() {
        assert!(string_matching::fuzzy_match("test", "test", 0.2));
    }

    #[test]
    fn test_path_helpers_module() {
        let dirs = path_helpers::get_app_dirs();
        assert!(!dirs.is_empty());
    }

    #[test]
    fn test_time_helpers_module() {
        assert!(time_helpers::parse_duration("30 seconds").is_some());
    }
}
