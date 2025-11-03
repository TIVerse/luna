//! Phase 6 TTS and Context Demo
//!
//! Demonstrates god-level TTS and context features.
//!
//! Run with: `cargo run --example phase6_tts_context_demo --features full`

use chrono::Utc;
use luna::brain::command_parser::IntentType;
use luna::brain::types::Entity;
use luna::context::{ActionResult, ContextEntry, UnifiedContext};
use luna::events::EventBus;
use luna::os::health_monitor::HealthMonitor;
use luna::tts::{MessageKind, TtsSystem};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🌙 LUNA Phase 6 - TTS & Context Demo\n");

    // Setup event bus
    let event_bus = Arc::new(EventBus::new());
    let _handle = event_bus.start_processing().await;

    // Subscribe to TTS events
    event_bus
        .subscribe(vec!["tts_started", "tts_completed"], |envelope| {
            println!("📢 Event: {}", envelope.event_type());
        })
        .await;

    demo_tts(&event_bus).await?;
    demo_context().await?;
    demo_health_aware_tts(&event_bus).await?;

    println!("\n✅ Demo complete!");
    Ok(())
}

/// Demo TTS features
async fn demo_tts(event_bus: &Arc<EventBus>) -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════");
    println!("  1. TTS Demo");
    println!("═══════════════════════════════════════\n");

    let mut tts = TtsSystem::new()?.with_event_bus(Arc::clone(event_bus));

    tts.start().await?;

    // Basic speech
    println!("▶ Basic speech");
    tts.speak("Hello! This is LUNA's text to speech system.")
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Message kinds with profiles
    println!("▶ Critical message (high priority)");
    tts.speak_with(MessageKind::Critical, "Alert! This is a critical message.")
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("▶ Confirmation message");
    tts.speak_with(MessageKind::Confirmation, "File opened successfully.")
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // SSML
    println!("▶ SSML with breaks and emphasis");
    tts.speak_ssml(
        r#"
        Please wait <break time="500ms"/> for processing.
        <emphasis>This is emphasized</emphasis> text.
        The code is <say-as interpret-as="characters">API</say-as>.
    "#,
    )
    .await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Coalesced updates
    println!("▶ Coalesced status updates (only latest speaks)");
    for i in (0..=100).step_by(20) {
        tts.enqueue_coalesced(
            MessageKind::Background,
            &format!("Processing... {}%", i),
            "progress",
        )
        .await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Stats
    let stats = tts.stats().await;
    println!("\n📊 TTS Statistics:");
    println!("   Total utterances: {}", stats.total_utterances);
    println!("   Queued: {}", stats.total_queued);
    println!("   Coalesced: {}", stats.total_coalesced);
    println!("   Errors: {}", stats.total_errors);

    tts.stop().await;
    Ok(())
}

/// Demo context management
async fn demo_context() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n═══════════════════════════════════════");
    println!("  2. Context Demo");
    println!("═══════════════════════════════════════\n");

    let mut ctx = UnifiedContext::new(100);

    // Add some entries
    println!("▶ Adding context entries");

    // Entry 1: Open Chrome
    let mut entities1 = HashMap::new();
    entities1.insert("app".to_string(), Entity::App("chrome".to_string()));

    ctx.add_entry(ContextEntry {
        timestamp: Utc::now(),
        user_input: "open chrome".to_string(),
        parsed_intent: IntentType::LaunchApp,
        entities: entities1,
        action_taken: ActionResult::LaunchedApp("chrome".to_string()),
        success: true,
        confidence: 0.95,
    });

    // Entry 2: Open file
    let mut entities2 = HashMap::new();
    entities2.insert("file".to_string(), Entity::File("report.pdf".to_string()));

    ctx.add_entry(ContextEntry {
        timestamp: Utc::now(),
        user_input: "open report.pdf".to_string(),
        parsed_intent: IntentType::FindFile,
        entities: entities2,
        action_taken: ActionResult::OpenedFile(PathBuf::from("report.pdf")),
        success: true,
        confidence: 0.92,
    });

    // Entry 3: Volume control (multiple times)
    for _ in 0..3 {
        ctx.add_entry(ContextEntry {
            timestamp: Utc::now(),
            user_input: "volume up".to_string(),
            parsed_intent: IntentType::VolumeControl,
            entities: HashMap::new(),
            action_taken: ActionResult::VolumeChanged(10),
            success: true,
            confidence: 0.98,
        });
    }

    println!("   Added {} entries", ctx.history.len());

    // Reference resolution
    println!("\n▶ Reference resolution");
    if let Some(Entity::App(name)) = ctx.resolve_reference("app") {
        println!("   'close it' → 'close {}'", name);
    }

    if let Some(Entity::File(name)) = ctx.resolve_reference("file") {
        println!("   'that file' → '{}'", name);
    }

    // Statistics
    println!("\n▶ Command statistics");
    if let Some(stats) = ctx.get_stats("volume up") {
        println!(
            "   'volume up' - Count: {}, Success rate: {:.1}%",
            stats.count,
            stats.success_rate() * 100.0
        );
    }

    // Summary
    println!("\n▶ Context summary");
    println!("   {}", ctx.summary());

    // Persistence demo (without actual file I/O for example)
    println!("\n▶ Persistence");
    println!("   Context can be saved to: data/context.json");
    println!("   Use: ctx.save().await? and UnifiedContext::load(path).await?");

    Ok(())
}

/// Demo health-aware TTS
async fn demo_health_aware_tts(
    event_bus: &Arc<EventBus>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n═══════════════════════════════════════");
    println!("  3. Health-Aware TTS Demo");
    println!("═══════════════════════════════════════\n");

    let health = HealthMonitor::new().with_event_bus(Arc::clone(event_bus));

    health.start_monitoring().await?;

    println!("▶ Checking system health");
    let issues = health.detect_issues().await;

    if issues.is_empty() {
        println!("   ✅ No health issues detected");
    } else {
        println!("   ⚠️  Issues detected: {:?}", issues);

        // Remediate
        for issue in &issues {
            println!("   🔧 Remediating: {:?}", issue);
            health.remediate(issue).await?;
        }
    }

    // Check if we can speak
    println!("\n▶ TTS readiness check");
    if health.can_speak().await {
        println!("   ✅ System ready for TTS");

        // Demonstrate speaking when healthy
        let mut tts = TtsSystem::new()?;
        tts.start().await?;
        tts.speak("System health is normal.").await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        tts.stop().await;
    } else {
        println!("   ⚠️  System under heavy load, TTS would be skipped");
    }

    health.stop_monitoring().await;

    Ok(())
}
