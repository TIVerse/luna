//! Wikipedia integration for quick facts and summaries
//!
//! Provides access to Wikipedia's API for instant answers.

use crate::error::{LunaError, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Wikipedia summary result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSummary {
    /// Article title
    pub title: String,
    /// Short extract/summary
    pub extract: String,
    /// Full URL to article
    pub url: String,
    /// Thumbnail image URL
    pub thumbnail: Option<String>,
    /// Page ID
    pub page_id: Option<u64>,
}

/// Wikipedia client
pub struct WikipediaClient {
    /// HTTP client
    client: reqwest::Client,
    /// Language code (default: "en")
    language: String,
}

impl WikipediaClient {
    /// Create a new Wikipedia client
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Luna/0.1.0 (Rust Voice Assistant)")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            language: "en".to_string(),
        }
    }

    /// Create client with specific language
    pub fn with_language(language: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Luna/0.1.0 (Rust Voice Assistant)")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            language,
        }
    }

    /// Get summary for a topic
    pub async fn get_summary(&self, topic: &str) -> Result<Option<WikiSummary>> {
        info!("📚 Fetching Wikipedia summary for: {}", topic);

        // Use Wikipedia REST API v1
        let url = format!(
            "https://{}.wikipedia.org/api/rest_v1/page/summary/{}",
            self.language,
            urlencoding::encode(topic)
        );

        debug!("Wikipedia API URL: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LunaError::Network(format!("Wikipedia request failed: {}", e)))?;

        if response.status() == 404 {
            debug!("Wikipedia page not found for: {}", topic);
            return Ok(None);
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LunaError::Network(format!("Failed to parse Wikipedia response: {}", e)))?;

        // Parse the response
        let summary = WikiSummary {
            title: json["title"].as_str().unwrap_or(topic).to_string(),
            extract: json["extract"].as_str().unwrap_or("No summary available").to_string(),
            url: json["content_urls"]["desktop"]["page"]
                .as_str()
                .map(String::from)
                .unwrap_or_else(|| format!("https://en.wikipedia.org/wiki/{}", topic)),
            thumbnail: json["thumbnail"]["source"].as_str().map(String::from),
            page_id: json["pageid"].as_u64(),
        };

        info!("✅ Wikipedia summary retrieved for: {}", topic);
        Ok(Some(summary))
    }

    /// Search Wikipedia for topics
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        info!("🔍 Searching Wikipedia for: {}", query);

        let url = format!(
            "https://{}.wikipedia.org/w/api.php?action=opensearch&search={}&limit={}&format=json",
            self.language,
            urlencoding::encode(query),
            limit.min(10)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LunaError::Network(format!("Wikipedia search failed: {}", e)))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LunaError::Network(format!("Failed to parse search results: {}", e)))?;

        // OpenSearch format: [query, [titles], [descriptions], [urls]]
        let mut results = Vec::new();
        if let Some(titles) = json.get(1).and_then(|v| v.as_array()) {
            for title in titles {
                if let Some(title_str) = title.as_str() {
                    results.push(title_str.to_string());
                }
            }
        }

        info!("Found {} Wikipedia results", results.len());
        Ok(results)
    }

    /// Quick fact lookup - searches and gets summary of first result
    pub async fn quick_fact(&self, query: &str) -> Result<Option<WikiSummary>> {
        // First search for the topic
        let results = self.search(query, 1).await?;
        
        if let Some(first_result) = results.first() {
            self.get_summary(first_result).await
        } else {
            Ok(None)
        }
    }

    /// Extract specific section from Wikipedia article
    pub async fn get_section(&self, topic: &str, section: &str) -> Result<Option<String>> {
        info!("📖 Fetching section '{}' from: {}", section, topic);

        let url = format!(
            "https://{}.wikipedia.org/w/api.php?action=parse&page={}&prop=wikitext&section={}&format=json",
            self.language,
            urlencoding::encode(topic),
            urlencoding::encode(section)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LunaError::Network(format!("Wikipedia section request failed: {}", e)))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LunaError::Network(format!("Failed to parse section response: {}", e)))?;

        if let Some(wikitext) = json["parse"]["wikitext"]["*"].as_str() {
            Ok(Some(wikitext.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Check if a page exists
    pub async fn page_exists(&self, title: &str) -> Result<bool> {
        let url = format!(
            "https://{}.wikipedia.org/w/api.php?action=query&titles={}&format=json",
            self.language,
            urlencoding::encode(title)
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LunaError::Network(format!("Wikipedia query failed: {}", e)))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LunaError::Network(format!("Failed to parse query response: {}", e)))?;

        // Check if page has a negative ID (means it doesn't exist)
        if let Some(pages) = json["query"]["pages"].as_object() {
            for (id, _) in pages {
                if let Ok(page_id) = id.parse::<i64>() {
                    return Ok(page_id > 0);
                }
            }
        }

        Ok(false)
    }
}

impl Default for WikipediaClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract clean text from wikitext (basic implementation)
pub fn clean_wikitext(wikitext: &str) -> String {
    // Remove wiki markup (basic cleanup)
    let text = wikitext
        .lines()
        .filter(|line| !line.starts_with("==") && !line.starts_with("[[File:") && !line.starts_with("{|"))
        .collect::<Vec<_>>()
        .join("\n");
    
    // Remove basic wiki syntax
    text.replace("'''", "")
        .replace("''", "")
        .replace("[[", "")
        .replace("]]", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wikipedia_client_creation() {
        let client = WikipediaClient::new();
        assert_eq!(client.language, "en");
    }

    #[test]
    fn test_custom_language() {
        let client = WikipediaClient::with_language("fr".to_string());
        assert_eq!(client.language, "fr");
    }

    #[tokio::test]
    async fn test_search() {
        let client = WikipediaClient::new();
        let results = client.search("Rust programming", 3).await;
        
        // May fail in CI without internet, so we just check it doesn't panic
        if let Ok(res) = results {
            assert!(res.len() <= 3);
        }
    }

    #[test]
    fn test_clean_wikitext() {
        let wikitext = "'''Bold text''' and ''italic text''";
        let cleaned = clean_wikitext(wikitext);
        assert_eq!(cleaned, "Bold text and italic text");
    }
}
