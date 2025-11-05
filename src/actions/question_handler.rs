//! Question handling action
//!
//! Handles question answering using the knowledge system.

use crate::error::Result;
use crate::knowledge::QuestionAnswerer;
use tracing::info;

/// Question handler
pub struct QuestionHandler {
    /// Question answerer
    qa: QuestionAnswerer,
}

impl QuestionHandler {
    /// Create a new question handler
    pub fn new() -> Self {
        Self {
            qa: QuestionAnswerer::new(),
        }
    }

    /// Answer a question
    pub async fn handle_question(&self, question: &str) -> Result<String> {
        info!("🤔 Processing question: {}", question);
        
        let answer = self.qa.answer(question).await?;
        
        let response = if answer.confidence > 0.7 {
            // High confidence answer
            if let Some(source) = &answer.source_url {
                format!("{}. Source: {}", answer.text, source)
            } else {
                answer.text
            }
        } else if answer.confidence > 0.4 {
            // Medium confidence - provide context
            if let Some(context) = &answer.context {
                format!("Based on {}: {}", context, answer.text)
            } else {
                format!("I think: {}", answer.text)
            }
        } else {
            // Low confidence
            format!("I'm not sure, but {}", answer.text)
        };

        Ok(response)
    }

    /// Perform web search
    pub async fn handle_search(&self, query: &str) -> Result<String> {
        info!("🔍 Searching for: {}", query);
        
        use crate::knowledge::WebSearcher;
        let searcher = WebSearcher::default();
        
        let results = searcher.search(query, 3).await?;
        
        if results.is_empty() {
            return Ok("No results found".to_string());
        }

        let mut response = format!("Found {} results:\n", results.len());
        for (i, result) in results.iter().enumerate().take(3) {
            response.push_str(&format!(
                "\n{}. {}\n   {}\n   {}",
                i + 1,
                result.title,
                result.snippet,
                result.url
            ));
        }

        Ok(response)
    }
}

impl Default for QuestionHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_handler_creation() {
        let handler = QuestionHandler::new();
        assert!(std::mem::size_of_val(&handler) > 0);
    }

    #[tokio::test]
    async fn test_time_question() {
        let handler = QuestionHandler::new();
        let response = handler.handle_question("What time is it?").await.unwrap();
        assert!(response.contains("time") || response.contains(":"));
    }
}
