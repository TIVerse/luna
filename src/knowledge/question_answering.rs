//! Question answering system
//!
//! Answers questions using web search, knowledge bases, and reasoning.

use crate::error::Result;
use crate::knowledge::web_search::{SearchEngine, WebSearcher};
use crate::knowledge::weather::WeatherService;
use crate::knowledge::wikipedia::WikipediaClient;
use crate::knowledge::graph::{Entity, EntityType, Fact, FactConfidence, KnowledgeGraph};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Answer source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AnswerSource {
    /// Web search result
    WebSearch,
    /// Local knowledge base
    KnowledgeBase,
    /// Wikipedia
    Wikipedia,
    /// Weather API
    Weather,
    /// Calculation
    Calculation,
    /// Time/Date
    DateTime,
}

/// Answer to a question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    /// The answer text
    pub text: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Source of the answer
    pub source: AnswerSource,
    /// Optional source URL
    pub source_url: Option<String>,
    /// Additional context
    pub context: Option<String>,
}

/// Question types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuestionType {
    /// Factual question (who, what, when, where)
    Factual,
    /// Definition question (what is, define)
    Definition,
    /// How-to question
    HowTo,
    /// Yes/No question
    YesNo,
    /// Numerical/Calculation
    Calculation,
    /// Weather query
    Weather,
    /// Time query
    Time,
    /// Comparison
    Comparison,
}

/// Question answerer
pub struct QuestionAnswerer {
    /// Web searcher
    web_searcher: WebSearcher,
    /// Wikipedia client
    wikipedia: WikipediaClient,
    /// Weather service
    weather: WeatherService,
    /// Knowledge graph for caching
    knowledge_graph: KnowledgeGraph,
}

impl QuestionAnswerer {
    /// Create a new question answerer
    pub fn new() -> Self {
        Self {
            web_searcher: WebSearcher::new(SearchEngine::DuckDuckGo),
            wikipedia: WikipediaClient::new(),
            weather: WeatherService::new(),
            knowledge_graph: KnowledgeGraph::new(),
        }
    }

    /// Get reference to knowledge graph
    pub fn knowledge_graph(&self) -> &KnowledgeGraph {
        &self.knowledge_graph
    }

    /// Answer a question
    pub async fn answer(&self, question: &str) -> Result<Answer> {
        info!("🤔 Answering question: {}", question);

        let question_type = self.classify_question(question);
        debug!("Question type: {:?}", question_type);

        match question_type {
            QuestionType::Calculation => self.answer_calculation(question).await,
            QuestionType::Weather => self.answer_weather(question).await,
            QuestionType::Time => self.answer_time(question).await,
            QuestionType::Definition => self.answer_definition(question).await,
            _ => self.answer_factual(question).await,
        }
    }

    /// Classify question type
    fn classify_question(&self, question: &str) -> QuestionType {
        let lower = question.to_lowercase();

        // Weather patterns
        if lower.contains("weather") || lower.contains("temperature") 
            || lower.contains("forecast") || lower.contains("rain") {
            return QuestionType::Weather;
        }

        // Time patterns
        if lower.contains("time") || lower.contains("date") 
            || lower.contains("day is it") || lower.contains("what day") {
            return QuestionType::Time;
        }

        // Calculation patterns
        if lower.contains("calculate") || lower.contains("what is")
            && (lower.contains('+') || lower.contains('-') || lower.contains('*') 
                || lower.contains('/') || lower.contains("times") || lower.contains("plus")) {
            return QuestionType::Calculation;
        }

        // Definition patterns
        if lower.starts_with("what is") || lower.starts_with("what are") 
            || lower.contains("define") || lower.contains("meaning of") {
            return QuestionType::Definition;
        }

        // How-to patterns
        if lower.starts_with("how to") || lower.starts_with("how do i")
            || lower.starts_with("how can i") {
            return QuestionType::HowTo;
        }

        // Yes/No patterns
        if lower.starts_with("is ") || lower.starts_with("are ") 
            || lower.starts_with("can ") || lower.starts_with("will ") 
            || lower.starts_with("do ") || lower.starts_with("does ") {
            return QuestionType::YesNo;
        }

        QuestionType::Factual
    }

    /// Answer calculation question
    async fn answer_calculation(&self, question: &str) -> Result<Answer> {
        // Simple calculation parsing
        let text = if let Some(expr) = self.extract_math_expression(question) {
            format!("The answer is approximately {}", expr)
        } else {
            "I couldn't parse that calculation".to_string()
        };

        Ok(Answer {
            text,
            confidence: 0.8,
            source: AnswerSource::Calculation,
            source_url: None,
            context: Some("Local calculation".to_string()),
        })
    }

    /// Answer weather question
    async fn answer_weather(&self, question: &str) -> Result<Answer> {
        // Extract location
        let location = self.extract_location(question).unwrap_or("London");
        
        // Try to get real weather data
        match self.weather.get_current_weather(location).await {
            Ok(weather) => {
                // Cache weather data in knowledge graph
                let entity_id = format!("weather_{}", location.to_lowercase().replace(' ', "_"));
                
                // Add weather facts
                self.knowledge_graph.add_fact(Fact {
                    id: uuid::Uuid::new_v4().to_string(),
                    entity_id: entity_id.clone(),
                    predicate: "temperature".to_string(),
                    value: format!("{}°C", weather.temperature),
                    confidence: FactConfidence::new(0.95),
                    source: Some("Open-Meteo API".to_string()),
                    timestamp: chrono::Utc::now(),
                    expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
                });

                Ok(Answer {
                    text: weather.to_string(),
                    confidence: 0.95,
                    source: AnswerSource::Weather,
                    source_url: None,
                    context: Some(format!("Current weather in {}", location)),
                })
            }
            Err(_) => {
                Ok(Answer {
                    text: format!("I couldn't fetch the weather for {}. Please check your internet connection.", location),
                    confidence: 0.3,
                    source: AnswerSource::Weather,
                    source_url: None,
                    context: None,
                })
            }
        }
    }

    /// Answer time question
    async fn answer_time(&self, question: &str) -> Result<Answer> {
        use chrono::Local;
        
        let now = Local::now();
        
        let text = if question.to_lowercase().contains("date") {
            format!("Today is {}", now.format("%A, %B %d, %Y"))
        } else {
            format!("The time is {}", now.format("%I:%M %p"))
        };

        Ok(Answer {
            text,
            confidence: 1.0,
            source: AnswerSource::DateTime,
            source_url: None,
            context: None,
        })
    }

    /// Answer definition question
    async fn answer_definition(&self, question: &str) -> Result<Answer> {
        // Extract the term to define
        let term = self.extract_definition_term(question).unwrap_or(question);
        
        // First, check knowledge graph cache
        if let Some(entity) = self.knowledge_graph.find_entity(term) {
            if let Some(desc) = entity.description {
                info!("Using cached definition from knowledge graph");
                return Ok(Answer {
                    text: desc,
                    confidence: 0.9,
                    source: AnswerSource::KnowledgeBase,
                    source_url: None,
                    context: Some(format!("Cached definition of {}", entity.name)),
                });
            }
        }
        
        // Try Wikipedia first for definitions
        if let Ok(Some(summary)) = self.wikipedia.get_summary(term).await {
            info!("Using Wikipedia for definition");
            
            // Cache in knowledge graph
            let entity = Entity {
                id: uuid::Uuid::new_v4().to_string(),
                name: summary.title.clone(),
                entity_type: EntityType::Concept,
                description: Some(summary.extract.clone()),
                aliases: vec![],
                metadata: std::collections::HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            self.knowledge_graph.add_entity(entity);
            
            return Ok(Answer {
                text: summary.extract,
                confidence: 0.9,
                source: AnswerSource::Wikipedia,
                source_url: Some(summary.url),
                context: Some(summary.title),
            });
        }
        
        // Fallback to web search
        let results = self.web_searcher.search(question, 1).await?;
        
        if let Some(result) = results.first() {
            Ok(Answer {
                text: result.snippet.clone(),
                confidence: 0.75,
                source: AnswerSource::WebSearch,
                source_url: Some(result.url.clone()),
                context: Some(result.title.clone()),
            })
        } else {
            Ok(Answer {
                text: "I couldn't find a definition for that".to_string(),
                confidence: 0.3,
                source: AnswerSource::WebSearch,
                source_url: None,
                context: None,
            })
        }
    }

    /// Answer factual question using web search
    async fn answer_factual(&self, question: &str) -> Result<Answer> {
        // Try Wikipedia first for factual questions
        if let Ok(Some(summary)) = self.wikipedia.quick_fact(question).await {
            info!("Using Wikipedia for factual answer");
            
            // Extract a concise answer from the summary
            let answer_text = if summary.extract.len() > 200 {
                format!("{}...", &summary.extract[..200])
            } else {
                summary.extract.clone()
            };
            
            return Ok(Answer {
                text: answer_text,
                confidence: 0.85,
                source: AnswerSource::Wikipedia,
                source_url: Some(summary.url),
                context: Some(summary.title),
            });
        }
        
        // Fallback to web search
        let results = self.web_searcher.search(question, 3).await?;
        
        if let Some(result) = results.first() {
            Ok(Answer {
                text: result.snippet.clone(),
                confidence: 0.75,
                source: AnswerSource::WebSearch,
                source_url: Some(result.url.clone()),
                context: Some(result.title.clone()),
            })
        } else {
            Ok(Answer {
                text: "I couldn't find an answer to that question".to_string(),
                confidence: 0.2,
                source: AnswerSource::WebSearch,
                source_url: None,
                context: None,
            })
        }
    }

    /// Extract math expression from question
    fn extract_math_expression(&self, question: &str) -> Option<String> {
        // Very simple extraction - in production, use proper parsing
        let lower = question.to_lowercase();
        
        if lower.contains("plus") || lower.contains('+') {
            return Some("sum".to_string());
        }
        if lower.contains("minus") || lower.contains('-') {
            return Some("difference".to_string());
        }
        if lower.contains("times") || lower.contains('*') {
            return Some("product".to_string());
        }
        
        None
    }

    /// Extract location from question
    fn extract_location<'a>(&self, question: &'a str) -> Option<&'a str> {
        let lower = question.to_lowercase();
        
        // Simple location extraction
        if let Some(pos) = lower.find(" in ") {
            let after = &question[pos + 4..];
            return after.split_whitespace().next();
        }
        
        if let Some(pos) = lower.find(" for ") {
            let after = &question[pos + 5..];
            return after.split_whitespace().next();
        }
        
        None
    }

    /// Extract the term to define from a definition question
    fn extract_definition_term<'a>(&self, question: &'a str) -> Option<&'a str> {
        let lower = question.to_lowercase();
        
        // "What is X" -> X
        if let Some(pos) = lower.find("what is ") {
            let after = &question[pos + 8..];
            return Some(after.trim_end_matches('?').trim());
        }
        
        // "What are X" -> X
        if let Some(pos) = lower.find("what are ") {
            let after = &question[pos + 9..];
            return Some(after.trim_end_matches('?').trim());
        }
        
        // "Define X" -> X
        if let Some(pos) = lower.find("define ") {
            let after = &question[pos + 7..];
            return Some(after.trim_end_matches('?').trim());
        }
        
        None
    }
}

impl Default for QuestionAnswerer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_question_classification() {
        let qa = QuestionAnswerer::new();
        
        assert_eq!(
            qa.classify_question("What's the weather in Tokyo?"),
            QuestionType::Weather
        );
        
        assert_eq!(
            qa.classify_question("What time is it?"),
            QuestionType::Time
        );
        
        assert_eq!(
            qa.classify_question("What is Rust?"),
            QuestionType::Definition
        );
    }

    #[tokio::test]
    async fn test_time_answer() {
        let qa = QuestionAnswerer::new();
        let answer = qa.answer_time("What time is it?").await.unwrap();
        assert_eq!(answer.source, AnswerSource::DateTime);
        assert!(answer.confidence > 0.9);
    }
}
