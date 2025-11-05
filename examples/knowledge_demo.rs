//! Knowledge System Demo
//!
//! Demonstrates the B1 features: Web Search, Question Answering, Wikipedia, and Weather
//!
//! Run with:
//! ```
//! cargo run --example knowledge_demo --features full
//! ```

use luna::knowledge::{
    QuestionAnswerer, WebSearcher, WikipediaClient, WeatherService, KnowledgeGraph,
    Entity, EntityType, Fact, FactConfidence,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for better debugging
    tracing_subscriber::fmt::init();

    println!("🚀 LUNA Knowledge System Demo (Phase B1)");
    println!("=========================================\n");

    // 1. Web Search Demo
    demo_web_search().await?;
    
    // 2. Wikipedia Demo
    demo_wikipedia().await?;
    
    // 3. Weather Demo
    demo_weather().await?;
    
    // 4. Question Answering Demo
    demo_question_answering().await?;
    
    // 5. Knowledge Graph Demo
    demo_knowledge_graph().await?;

    println!("\n✅ All demos completed successfully!");
    Ok(())
}

async fn demo_web_search() -> Result<(), Box<dyn std::error::Error>> {
    println!("📚 1. Web Search Demo");
    println!("-------------------");
    
    let searcher = WebSearcher::default();
    
    let query = "Rust programming language";
    println!("🔍 Searching for: {}", query);
    
    match searcher.search(query, 3).await {
        Ok(results) => {
            println!("Found {} results:", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("\n{}. {}", i + 1, result.title);
                println!("   {}", result.snippet);
                println!("   🔗 {}", result.url);
            }
        }
        Err(e) => {
            println!("⚠️  Search failed: {}", e);
            println!("   (This is expected if you're offline)");
        }
    }
    
    println!();
    Ok(())
}

async fn demo_wikipedia() -> Result<(), Box<dyn std::error::Error>> {
    println!("📖 2. Wikipedia Demo");
    println!("------------------");
    
    let wiki = WikipediaClient::new();
    
    let topic = "Artificial Intelligence";
    println!("📚 Getting summary for: {}", topic);
    
    match wiki.get_summary(topic).await {
        Ok(Some(summary)) => {
            println!("\n📌 {}", summary.title);
            println!("━".repeat(50));
            
            // Print first 300 characters of the extract
            let extract = if summary.extract.len() > 300 {
                format!("{}...", &summary.extract[..300])
            } else {
                summary.extract.clone()
            };
            println!("{}", extract);
            println!("\n🔗 Read more: {}", summary.url);
        }
        Ok(None) => {
            println!("❌ No Wikipedia page found for '{}'", topic);
        }
        Err(e) => {
            println!("⚠️  Wikipedia request failed: {}", e);
            println!("   (This is expected if you're offline)");
        }
    }
    
    println!();
    Ok(())
}

async fn demo_weather() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌤️  3. Weather Demo");
    println!("----------------");
    
    let weather_service = WeatherService::new();
    
    let location = "London";
    println!("🌍 Getting weather for: {}", location);
    
    match weather_service.get_current_weather(location).await {
        Ok(weather) => {
            println!("\n{}", weather.to_string());
            println!("\nDetailed info:");
            println!("  Temperature: {}°C ({}°F)", weather.temperature, weather.temperature_f);
            println!("  Condition: {} {}", weather.condition.emoji(), weather.description);
            println!("  Wind: {} km/h", weather.wind_speed);
            if let Some(humidity) = weather.humidity {
                println!("  Humidity: {}%", humidity);
            }
        }
        Err(e) => {
            println!("⚠️  Weather request failed: {}", e);
            println!("   (This is expected if you're offline)");
        }
    }
    
    // Try forecast
    println!("\n📅 3-day forecast for {}:", location);
    match weather_service.get_forecast(location, 3).await {
        Ok(forecasts) => {
            for forecast in forecasts {
                println!(
                    "  {} {} {:.0}°C - {:.0}°C",
                    forecast.date,
                    forecast.condition.emoji(),
                    forecast.temp_min,
                    forecast.temp_max
                );
            }
        }
        Err(e) => {
            println!("⚠️  Forecast failed: {}", e);
        }
    }
    
    println!();
    Ok(())
}

async fn demo_question_answering() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤔 4. Question Answering Demo");
    println!("---------------------------");
    
    let qa = QuestionAnswerer::new();
    
    let questions = vec![
        "What time is it?",
        "What is Rust?",
        "What's the weather in Tokyo?",
    ];
    
    for question in questions {
        println!("\n❓ Q: {}", question);
        
        match qa.answer(question).await {
            Ok(answer) => {
                println!("💡 A: {}", answer.text);
                println!("   Confidence: {:.0}%", answer.confidence * 100.0);
                println!("   Source: {:?}", answer.source);
                
                if let Some(url) = &answer.source_url {
                    println!("   🔗 {}", url);
                }
            }
            Err(e) => {
                println!("❌ Error: {}", e);
            }
        }
    }
    
    println!();
    Ok(())
}

async fn demo_knowledge_graph() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 5. Knowledge Graph Demo");
    println!("------------------------");
    
    let graph = KnowledgeGraph::new();
    
    // Add an entity
    let rust_entity = Entity {
        id: "rust_lang".to_string(),
        name: "Rust Programming Language".to_string(),
        entity_type: EntityType::Concept,
        description: Some("A systems programming language focused on safety and performance".to_string()),
        aliases: vec!["Rust".to_string(), "rust-lang".to_string()],
        metadata: std::collections::HashMap::new(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    println!("➕ Adding entity: {}", rust_entity.name);
    graph.add_entity(rust_entity);
    
    // Add some facts
    println!("➕ Adding facts...");
    
    graph.add_fact(Fact {
        id: "fact1".to_string(),
        entity_id: "rust_lang".to_string(),
        predicate: "creator".to_string(),
        value: "Graydon Hoare".to_string(),
        confidence: FactConfidence::new(0.95),
        source: Some("Wikipedia".to_string()),
        timestamp: chrono::Utc::now(),
        expires_at: None,
    });
    
    graph.add_fact(Fact {
        id: "fact2".to_string(),
        entity_id: "rust_lang".to_string(),
        predicate: "first_release".to_string(),
        value: "2015".to_string(),
        confidence: FactConfidence::new(1.0),
        source: Some("Official Rust website".to_string()),
        timestamp: chrono::Utc::now(),
        expires_at: None,
    });
    
    // Query the graph
    println!("\n🔍 Finding entity 'Rust'...");
    if let Some(entity) = graph.find_entity("rust") {
        println!("✅ Found: {}", entity.name);
        if let Some(desc) = &entity.description {
            println!("   Description: {}", desc);
        }
    }
    
    println!("\n🔍 Querying facts about 'rust_lang'...");
    let facts = graph.query_facts("rust_lang", None);
    for fact in &facts {
        println!("   • {}: {}", fact.predicate, fact.value);
        if let Some(source) = &fact.source {
            println!("     (Source: {})", source);
        }
    }
    
    // Show stats
    let stats = graph.stats();
    println!("\n📊 Knowledge Graph Stats:");
    println!("   Entities: {}", stats.entity_count);
    println!("   Facts: {}", stats.fact_count);
    println!("   Cache Hit Rate: {:.1}%", stats.hit_rate * 100.0);
    
    println!();
    Ok(())
}
