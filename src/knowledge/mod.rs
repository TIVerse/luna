//! Knowledge and information services
//!
//! Web search, question answering, and information retrieval.

pub mod graph;
pub mod question_answering;
pub mod weather;
pub mod web_search;
pub mod wikipedia;

pub use graph::{Entity, EntityType, Fact, FactConfidence, KnowledgeGraph, Relationship};
pub use question_answering::{Answer, AnswerSource, QuestionAnswerer, QuestionType};
pub use weather::{CurrentWeather, DailyForecast, WeatherCondition, WeatherService};
pub use web_search::{SearchEngine, SearchResult, WebSearcher};
pub use wikipedia::{WikiSummary, WikipediaClient};
