//! Web search integration
//!
//! Provides web search capabilities using multiple search engines.

use crate::error::{LunaError, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Search engine types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchEngine {
    /// DuckDuckGo (privacy-focused, no API key needed)
    DuckDuckGo,
    /// Google (requires API key)
    Google,
    /// Brave Search (privacy-focused)
    Brave,
}

/// Web search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Result title
    pub title: String,
    /// Result URL
    pub url: String,
    /// Snippet/description
    pub snippet: String,
    /// Result rank
    pub rank: usize,
    /// Optional favicon URL
    pub favicon: Option<String>,
}

/// Web searcher
pub struct WebSearcher {
    /// Search engine to use
    engine: SearchEngine,
    /// HTTP client
    client: reqwest::Client,
    /// API key (if required)
    api_key: Option<String>,
}

impl WebSearcher {
    /// Create a new web searcher
    pub fn new(engine: SearchEngine) -> Self {
        Self {
            engine,
            client: reqwest::Client::builder()
                .user_agent("Luna/0.1.0")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            api_key: None,
        }
    }

    /// Set API key for search engine
    pub fn with_api_key(mut self, key: String) -> Self {
        self.api_key = Some(key);
        self
    }

    /// Perform web search
    pub async fn search(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        info!("🔍 Searching for: {}", query);
        
        match self.engine {
            SearchEngine::DuckDuckGo => self.search_duckduckgo(query, max_results).await,
            SearchEngine::Google => self.search_google(query, max_results).await,
            SearchEngine::Brave => self.search_brave(query, max_results).await,
        }
    }

    /// Search using DuckDuckGo Instant Answer API
    async fn search_duckduckgo(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1",
            urlencoding::encode(query)
        );

        debug!("DuckDuckGo search URL: {}", url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| LunaError::Network(format!("Search request failed: {}", e)))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LunaError::Network(format!("Failed to parse response: {}", e)))?;

        let mut results = Vec::new();

        // Extract abstract
        if let Some(abstract_text) = json["AbstractText"].as_str() {
            if !abstract_text.is_empty() {
                results.push(SearchResult {
                    title: json["Heading"]
                        .as_str()
                        .unwrap_or("DuckDuckGo Result")
                        .to_string(),
                    url: json["AbstractURL"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    snippet: abstract_text.to_string(),
                    rank: 0,
                    favicon: None,
                });
            }
        }

        // Extract related topics
        if let Some(topics) = json["RelatedTopics"].as_array() {
            for (i, topic) in topics.iter().enumerate().take(max_results.saturating_sub(1)) {
                if let Some(text) = topic["Text"].as_str() {
                    results.push(SearchResult {
                        title: topic["Text"].as_str().unwrap_or("").to_string(),
                        url: topic["FirstURL"].as_str().unwrap_or("").to_string(),
                        snippet: text.to_string(),
                        rank: i + 1,
                        favicon: topic["Icon"]["URL"].as_str().map(String::from),
                    });
                }
            }
        }

        info!("Found {} DuckDuckGo results", results.len());
        Ok(results)
    }

    /// Search using Google Custom Search API
    async fn search_google(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| LunaError::Config("Google API key not configured".to_string()))?;

        let url = format!(
            "https://www.googleapis.com/customsearch/v1?key={}&q={}&num={}",
            api_key,
            urlencoding::encode(query),
            max_results.min(10)
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| LunaError::Network(format!("Google search failed: {}", e)))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LunaError::Network(format!("Failed to parse Google response: {}", e)))?;

        let mut results = Vec::new();

        if let Some(items) = json["items"].as_array() {
            for (i, item) in items.iter().enumerate() {
                results.push(SearchResult {
                    title: item["title"].as_str().unwrap_or("").to_string(),
                    url: item["link"].as_str().unwrap_or("").to_string(),
                    snippet: item["snippet"].as_str().unwrap_or("").to_string(),
                    rank: i,
                    favicon: None,
                });
            }
        }

        info!("Found {} Google results", results.len());
        Ok(results)
    }

    /// Search using Brave Search API
    async fn search_brave(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| LunaError::Config("Brave API key not configured".to_string()))?;

        let url = format!(
            "https://api.search.brave.com/res/v1/web/search?q={}&count={}",
            urlencoding::encode(query),
            max_results.min(20)
        );

        let response = self.client
            .get(&url)
            .header("X-Subscription-Token", api_key)
            .send()
            .await
            .map_err(|e| LunaError::Network(format!("Brave search failed: {}", e)))?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LunaError::Network(format!("Failed to parse Brave response: {}", e)))?;

        let mut results = Vec::new();

        if let Some(web_results) = json["web"]["results"].as_array() {
            for (i, item) in web_results.iter().enumerate() {
                results.push(SearchResult {
                    title: item["title"].as_str().unwrap_or("").to_string(),
                    url: item["url"].as_str().unwrap_or("").to_string(),
                    snippet: item["description"].as_str().unwrap_or("").to_string(),
                    rank: i,
                    favicon: item["favicon"].as_str().map(String::from),
                });
            }
        }

        info!("Found {} Brave results", results.len());
        Ok(results)
    }

    /// Quick search - returns just the top result
    pub async fn quick_search(&self, query: &str) -> Result<Option<SearchResult>> {
        let results = self.search(query, 1).await?;
        Ok(results.into_iter().next())
    }
}

impl Default for WebSearcher {
    fn default() -> Self {
        Self::new(SearchEngine::DuckDuckGo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_searcher_creation() {
        let searcher = WebSearcher::new(SearchEngine::DuckDuckGo);
        assert_eq!(searcher.engine, SearchEngine::DuckDuckGo);
    }
}
