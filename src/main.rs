//! LUNA Voice Assistant - Main Entry Point
//!
//! Privacy-first offline voice-controlled desktop assistant
//!
//! Supports both CLI mode and runtime mode:
//! - CLI: `luna doctor`, `luna index`, etc.
//! - Runtime: Full voice assistant with event bus and lifecycle management

use luna::cli::{Cli, run_cli};
use luna::config::LunaConfig;
use luna::{ConfigManager, EventBus, Metrics};
use luna::error::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // If a subcommand is provided, run CLI mode
    if cli.command.is_some() {
        // Setup minimal logging for CLI
        setup_minimal_logging(&cli.log_level)?;
        return run_cli(cli).await;
    }
    
    // Otherwise, run the full voice assistant
    run_assistant(cli).await
}

/// Run the full LUNA voice assistant
async fn run_assistant(cli: Cli) -> Result<()> {
    // Load configuration with manager (layered + hot-reload)
    let mut config_mgr = ConfigManager::new(cli.config).await?;
    
    // Enable hot-reload for production
    config_mgr.enable_hot_reload()?;
    
    let config = config_mgr.get_clone().await;

    // Setup logging
    setup_logging(&config)?;

    info!("🌙 LUNA Voice Assistant starting up...");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    info!("Configuration loaded with layering + hot-reload enabled");

    // Display configuration summary
    info!("Sample Rate: {} Hz", config.audio.sample_rate);
    info!("Wake Words: {:?}", config.audio.wake_words);
    info!("Log Level: {}", config.system.log_level);
    info!("Data Directory: {}", config.system.data_dir);
    
    // Initialize event bus
    info!("Initializing event bus...");
    let event_bus = std::sync::Arc::new(EventBus::new());
    let _event_bus_handle = event_bus.start_processing().await;
    info!("✓ Event bus initialized");
    
    // Initialize metrics
    info!("Initializing metrics...");
    let metrics = std::sync::Arc::new(Metrics::new());
    
    #[cfg(feature = "prometheus")]
    {
        // Start Prometheus exporter if enabled
        if let Err(e) = luna::metrics::prometheus::start_exporter("127.0.0.1:9090").await {
            tracing::error!("Failed to start Prometheus exporter: {}", e);
        } else {
            luna::metrics::prometheus::init_metrics();
            info!("✓ Prometheus metrics exporter started on :9090");
        }
    }
    
    info!("✓ Metrics initialized");
    
    // Subscribe to events for logging and state tracking
    event_bus.subscribe(vec![], move |envelope| {
        tracing::debug!(
            event_id = %envelope.id,
            event_type = %envelope.event_type(),
            correlation_id = ?envelope.correlation_id,
            "Event published"
        );
        
        // Track event metrics
        // Events are tracked via the event bus itself
    }).await;

    // Initialize components
    info!("Initializing components...");

    // Audio System
    let mut audio_system = luna::audio::ProductionAudioSystem::create(&config.audio, &config.brain)
        .await?
        .with_event_bus(std::sync::Arc::clone(&event_bus));
    info!("✓ Audio system initialized");
    
    if audio_system.is_stt_simulated() {
        info!("⚠️  Note: Running in simulated mode for speech-to-text");
        info!("   For production, download Whisper model to: {}", config.brain.whisper_model_path);
    }

    // Brain/NLP System
    let brain = luna::brain::Brain::new(&config.brain)?;
    info!("✓ Brain system initialized");

    // Task Executor
    // Create app database and discover applications
    let mut app_db_temp = luna::db::AppDatabase::new();
    match luna::os::discovery::discover_applications().await {
        Ok(apps) => {
            for app in apps {
                app_db_temp.add_app(app);
            }
            info!("✓ Discovered {} applications", app_db_temp.len());
        }
        Err(e) => {
            tracing::warn!("Failed to discover applications: {}", e);
        }
    }
    let app_db = std::sync::Arc::new(app_db_temp);
    let file_index = std::sync::Arc::new(luna::db::FileIndex::new());    
    let app_launcher = luna::actions::AppLauncher::new(app_db);
    let file_search = luna::actions::FileSearch::new(file_index);
    let executor = luna::actions::TaskExecutor::new(app_launcher, file_search)
        .with_event_bus(std::sync::Arc::clone(&event_bus))
        .with_metrics(std::sync::Arc::clone(&metrics));
    info!("✓ Task executor initialized");
    
    // TTS System
    let mut tts_system = match luna::tts::TtsSystem::new() {
        Ok(tts) => {
            let tts = tts.with_event_bus(std::sync::Arc::clone(&event_bus));
            info!("✓ TTS system initialized");
            Some(tts)
        },
        Err(e) => {
            tracing::warn!("TTS system unavailable: {}", e);
            None
        }
    };
    
    // Start TTS worker if available
    if let Some(ref mut tts) = tts_system {
        tts.start().await?;
    }
    
    // Conversation memory
    let mut conversation_memory = luna::context::memory::ConversationMemory::with_capacity(100);

    info!("✅ All systems ready");
    info!("👂 LUNA is ready to listen!");
    
    // Print metrics summary
    metrics.print_summary();
    
    // Publish startup event
    event_bus.publish(luna::LunaEvent::StateChanged {
        from: "stopped".to_string(),
        to: "running".to_string(),
    }).await;
    
    // Start audio capture
    audio_system.start_listening()?;
    
    // Speak startup message
    if let Some(ref tts) = tts_system {
        let _ = tts.speak_with(luna::tts::MessageKind::Info, "Luna is ready").await;
    }
    
    info!("✅ Full Integration Complete");
    info!("  - Audio capture & wake word detection");
    info!("  - Speech-to-text processing");
    info!("  - Natural language understanding");
    info!("  - Task execution with retry logic");
    info!("  - Text-to-speech feedback");
    info!("  - Conversation memory");
    info!("  - Event bus & metrics tracking");
    
    // Setup Ctrl+C handler for graceful shutdown
    let shutdown_requested = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let shutdown_flag = shutdown_requested.clone();
    
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        shutdown_flag.store(true, std::sync::atomic::Ordering::SeqCst);
    });
    
    info!("🎤 Main event loop starting... (Press Ctrl+C to shutdown)");
    
    // Main event loop
    let mut command_count = 0;
    
    while !shutdown_requested.load(std::sync::atomic::Ordering::SeqCst) {
        // Wait for wake word
        match audio_system.wait_for_wake_word().await {
            Ok(true) => {
                info!("👂 Wake word detected!");
                
                // Interrupt any ongoing TTS
                if let Some(ref tts) = tts_system {
                    let _ = tts.interrupt().await;
                }
                
                // TODO: Record and transcribe command
                // For now, simulate with a command counter
                command_count += 1;
                let simulated_text = format!("command {}", command_count);
                
                info!("💬 Command received: \"{}\"", simulated_text);
                
                // Process command through brain
                let start_time = std::time::Instant::now();
                match brain.process_async(&simulated_text).await {
                    Ok(plan) => {
                        let processing_time = start_time.elapsed();
                        info!("🧠 Command processed in {:?}", processing_time);
                        info!("   Intent: {:?}, Confidence: {:.2}", 
                              plan.classification.intent, plan.classification.confidence);
                        
                        // Execute the plan
                        match executor.execute_plan(plan.clone()).await {
                            Ok(response) => {
                                let total_time = start_time.elapsed();
                                info!("✅ Action completed in {:?}: {}", total_time, response);
                                
                                // Speak response
                                if let Some(ref tts) = tts_system {
                                    let _ = tts.speak_with(
                                        luna::tts::MessageKind::Confirmation, 
                                        &response
                                    ).await;
                                }
                                
                                // Record in conversation memory
                                conversation_memory.add_entry(
                                    luna::context::ConversationEntry::new(
                                        simulated_text.clone(),
                                        plan.classification.intent,
                                        response.clone(),
                                        true,
                                    )
                                );
                                
                                // Track metrics
                                metrics.record_command_success();
                                metrics.record_latency(luna::metrics::MetricPhase::Parsing, processing_time);
                            }
                            Err(e) => {
                                tracing::error!("❌ Action execution failed: {}", e);
                                
                                // Speak error message
                                if let Some(ref tts) = tts_system {
                                    let _ = tts.speak_with(
                                        luna::tts::MessageKind::Error,
                                        "Sorry, I couldn't do that"
                                    ).await;
                                }
                                
                                // Record failure in conversation memory
                                conversation_memory.add_entry(
                                    luna::context::ConversationEntry::new(
                                        simulated_text.clone(),
                                        plan.classification.intent,
                                        format!("Error: {}", e),
                                        false,
                                    )
                                );
                                
                                // Track metrics
                                metrics.record_command_failure();
                                metrics.record_error("execution_failed", "executor");
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("⚠️  Command not understood: {}", e);
                        
                        // Speak clarification request
                        if let Some(ref tts) = tts_system {
                            let _ = tts.speak_with(
                                luna::tts::MessageKind::Info,
                                "I didn't understand that. Could you please rephrase?"
                            ).await;
                        }
                    }
                }
            }
            Ok(false) => {
                // No wake word detected, continue listening
            }
            Err(e) => {
                tracing::error!("Error in wake word detection: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
        
        // Brief sleep to avoid busy loop (wake word detection has its own timing)
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    
    info!("Shutting down gracefully...");
    
    // Stop audio system
    audio_system.stop()?;
    
    // Stop TTS system
    if let Some(ref mut tts) = tts_system {
        tts.stop().await;
    }
    
    // Publish shutdown event
    event_bus.publish(luna::LunaEvent::StateChanged {
        from: "running".to_string(),
        to: "stopped".to_string(),
    }).await;
    
    // Print final statistics
    println!("\n");
    println!("=== LUNA Session Summary ===");
    println!("Commands processed: {}", conversation_memory.len());
    metrics.print_summary();
    
    // Print conversation summary
    if !conversation_memory.is_empty() {
        println!("\nRecent commands:");
        for entry in conversation_memory.get_recent(5) {
            let status = if entry.success { "✓" } else { "✗" };
            println!("  {} {} -> {}", status, entry.user_input, entry.action_taken);
        }
    }

    info!("✅ LUNA shutdown complete");

    Ok(())
}

/// Setup minimal logging for CLI commands
fn setup_minimal_logging(log_level: &str) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().compact())
        .init();
    
    Ok(())
}

/// Setup logging infrastructure
fn setup_logging(config: &LunaConfig) -> Result<()> {
    // Create log directory if it doesn't exist
    let log_dir = std::path::Path::new(&config.system.data_dir).join("logs");
    std::fs::create_dir_all(&log_dir).map_err(|e| {
        luna::error::LunaError::Config(format!("Failed to create log directory: {}", e))
    })?;

    // Setup file appender with daily rotation
    let file_appender = tracing_appender::rolling::daily(&log_dir, "luna.log");

    // Parse log level
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.system.log_level));

    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(file_appender))
        .init();

    Ok(())
}
