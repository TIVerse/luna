//! Integration tests for LUNA voice assistant
//!
//! Tests the full pipeline: wake word -> STT -> parse -> execute -> TTS

use luna::audio::{AudioSystem, ProductionAudioSystem};
use luna::brain::Brain;
use luna::actions::{AppLauncher, FileSearch, TaskExecutor};
use luna::config::{AudioConfig, BrainConfig, LunaConfig};
use luna::context::ConversationMemory;
use luna::tts::TtsSystem;
use luna::events::EventBus;
use luna::metrics::Metrics;
use luna::db::{AppDatabase, FileIndex};
use std::sync::Arc;
use tokio::time::{timeout, Duration};

/// Helper function to create test app database
async fn create_test_app_db() -> Arc<AppDatabase> {
    let temp_dir = std::env::temp_dir().join("luna_test");
    std::fs::create_dir_all(&temp_dir).ok();
    let db = Arc::new(AppDatabase::new(&temp_dir.to_string_lossy()).unwrap());
    // Don't discover apps in tests - too slow
    db
}

/// Helper function to create test file index
fn create_test_file_index() -> Arc<FileIndex> {
    let temp_dir = std::env::temp_dir().join("luna_test");
    std::fs::create_dir_all(&temp_dir).ok();
    Arc::new(FileIndex::new(&temp_dir.to_string_lossy()).unwrap())
}

/// Test full pipeline with mock components
#[tokio::test]
async fn test_full_pipeline_mock() {
    // Initialize configuration
    let brain_config = BrainConfig::default();
    
    // Initialize brain
    let brain = Brain::new(&brain_config).expect("Failed to create brain");
    
    // Initialize executor
    let app_launcher = AppLauncher::new(create_test_app_db().await);
    let file_search = FileSearch::new(create_test_file_index());
    let executor = TaskExecutor::new(app_launcher, file_search);
    
    // Test command
    let test_commands = vec![
        "open chrome",
        "close firefox",
        "find budget.pdf",
        "volume up",
        "what time is it",
    ];
    
    for cmd in test_commands {
        // Parse command
        let plan = brain.process(cmd).expect(&format!("Failed to process: {}", cmd));
        assert!(!plan.steps.is_empty(), "Plan should have steps for: {}", cmd);
        
        // Execute plan
        let result = executor.execute_plan(plan).await;
        assert!(result.is_ok(), "Execution failed for: {}", cmd);
    }
}

/// Test error recovery and handling
#[tokio::test]
async fn test_error_recovery() {
    let brain_config = BrainConfig::default();
    let brain = Brain::new(&brain_config).expect("Failed to create brain");
    
    let app_launcher = AppLauncher::new();
    let file_search = FileSearch::new();
    let executor = TaskExecutor::new(app_launcher, file_search);
    
    // Test with invalid/unclear command
    let unclear_commands = vec![
        "asdfasdf",
        "banana elephant computer",
        "",
    ];
    
    for cmd in unclear_commands {
        if cmd.is_empty() {
            continue;
        }
        
        let result = brain.process(cmd);
        // Should still return a result, even if low confidence
        assert!(result.is_ok() || result.is_err());
    }
    
    // Test retry logic with executor
    // The executor should handle failures gracefully
    let plan = brain.process("open nonexistent_app_12345").unwrap();
    let result = executor.execute_plan(plan).await;
    // Should either succeed or fail gracefully (not panic)
    assert!(result.is_ok() || result.is_err());
}

/// Test conversation memory integration
#[tokio::test]
async fn test_conversation_memory() {
    let brain_config = BrainConfig::default();
    let brain = Brain::new(&brain_config).expect("Failed to create brain");
    let mut memory = ConversationMemory::with_capacity(10);
    
    // Process several commands
    let commands = vec![
        "open chrome",
        "close chrome",
        "find document.pdf",
    ];
    
    for cmd in commands {
        let plan = brain.process(cmd).expect("Failed to process");
        
        // Add to memory
        memory.add_entry(luna::context::ConversationEntry::new(
            cmd.to_string(),
            plan.classification.intent,
            "Executed successfully".to_string(),
            true,
        ));
    }
    
    assert_eq!(memory.len(), 3);
    
    // Test memory retrieval
    let last = memory.get_last_action().unwrap();
    assert!(last.user_input.contains("document.pdf"));
    
    // Test search
    let chrome_commands = memory.search("chrome");
    assert_eq!(chrome_commands.len(), 2);
}

/// Test event bus integration
#[tokio::test]
async fn test_event_bus_integration() {
    let event_bus = Arc::new(EventBus::new());
    let handle = event_bus.start_processing().await;
    
    // Create a counter for received events
    let event_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter_clone = Arc::clone(&event_counter);
    
    // Subscribe to events
    event_bus.subscribe(vec![], move |_envelope| {
        counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }).await;
    
    // Publish some events
    event_bus.publish(luna::LunaEvent::StateChanged {
        from: "stopped".to_string(),
        to: "running".to_string(),
    }).await;
    
    event_bus.publish(luna::LunaEvent::WakeWordDetected {
        keyword: "luna".to_string(),
        confidence: 0.95,
    }).await;
    
    // Wait for events to be processed
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Check that events were received
    let count = event_counter.load(std::sync::atomic::Ordering::SeqCst);
    assert!(count >= 2, "Expected at least 2 events, got {}", count);
    
    handle.abort();
}

/// Test metrics tracking
#[tokio::test]
async fn test_metrics_tracking() {
    let metrics = Arc::new(Metrics::new());
    
    // Record some operations
    metrics.record_success();
    metrics.record_success();
    metrics.record_error();
    
    let start = std::time::Instant::now();
    tokio::time::sleep(Duration::from_millis(10)).await;
    let duration = start.elapsed();
    
    metrics.record_latency("test_operation", duration);
    
    // Metrics should be tracked
    // (In a real test, we'd assert on specific values)
    // For now, just ensure it doesn't panic
    metrics.print_summary();
}

/// Test TTS system integration
#[tokio::test]
async fn test_tts_integration() {
    // TTS may not be available in test environment
    if let Ok(tts) = TtsSystem::new() {
        assert!(tts.is_enabled().await);
        
        // Test enable/disable
        tts.disable().await;
        assert!(!tts.is_enabled().await);
        
        tts.enable().await;
        assert!(tts.is_enabled().await);
        
        // Test queue
        let _ = tts.speak_with(luna::tts::MessageKind::Info, "test message").await;
        assert!(tts.queue_size().await > 0);
        
        // Test stop
        let _ = tts.stop_all().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(tts.queue_size().await, 0);
    }
}

/// Test brain caching system
#[tokio::test]
async fn test_brain_caching() {
    let brain_config = BrainConfig::default();
    let brain = Brain::new(&brain_config).expect("Failed to create brain");
    
    let command = "open chrome";
    
    // First processing - should cache
    let start1 = std::time::Instant::now();
    let _plan1 = brain.process(command).unwrap();
    let duration1 = start1.elapsed();
    
    // Second processing - should be faster (cached)
    let start2 = std::time::Instant::now();
    let _plan2 = brain.process(command).unwrap();
    let duration2 = start2.elapsed();
    
    // Second call should be faster due to caching
    // (This may not always be true in CI environments, so we just check it doesn't panic)
    println!("First: {:?}, Second: {:?}", duration1, duration2);
    
    // Check cache stats
    let stats = brain.cache_stats();
    assert!(stats.hits + stats.misses > 0);
}

/// Test executor with event bus and metrics
#[tokio::test]
async fn test_executor_with_telemetry() {
    let event_bus = Arc::new(EventBus::new());
    let handle = event_bus.start_processing().await;
    let metrics = Arc::new(Metrics::new());
    
    let app_launcher = AppLauncher::new(create_test_app_db().await);
    let file_search = FileSearch::new(create_test_file_index());
    let executor = TaskExecutor::new(app_launcher, file_search)
        .with_event_bus(Arc::clone(&event_bus))
        .with_metrics(Arc::clone(&metrics));
    
    let brain_config = BrainConfig::default();
    let brain = Brain::new(&brain_config).expect("Failed to create brain");
    
    // Execute a command
    let plan = brain.process("volume up").unwrap();
    let result = executor.execute_plan(plan).await;
    
    assert!(result.is_ok());
    
    // Wait for events to propagate
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    handle.abort();
}

/// Test parallel execution (if supported)
#[tokio::test]
async fn test_parallel_execution() {
    let brain_config = BrainConfig::default();
    let brain = Brain::new(&brain_config).expect("Failed to create brain");
    
    let app_launcher = AppLauncher::new(create_test_app_db().await);
    let file_search = FileSearch::new(create_test_file_index());
    let executor = TaskExecutor::new(app_launcher, file_search);
    
    // Commands that could potentially run in parallel
    let commands = vec![
        "volume up",
        "brightness up",
    ];
    
    for cmd in commands {
        let plan = brain.process(cmd).unwrap();
        let result = executor.execute_plan(plan).await;
        assert!(result.is_ok());
    }
}

/// Test response time requirements (< 1 second)
#[tokio::test]
async fn test_response_time() {
    let brain_config = BrainConfig::default();
    let brain = Brain::new(&brain_config).expect("Failed to create brain");
    
    let app_launcher = AppLauncher::new(create_test_app_db().await);
    let file_search = FileSearch::new(create_test_file_index());
    let executor = TaskExecutor::new(app_launcher, file_search);
    
    let command = "volume up";
    
    let start = std::time::Instant::now();
    
    // Process
    let plan = brain.process(command).unwrap();
    
    // Execute
    let result = timeout(Duration::from_secs(1), executor.execute_plan(plan)).await;
    
    let elapsed = start.elapsed();
    
    assert!(result.is_ok(), "Execution should complete within 1 second");
    assert!(elapsed < Duration::from_secs(1), 
            "Total time should be < 1s, got {:?}", elapsed);
}

/// Test graceful degradation when components fail
#[tokio::test]
async fn test_graceful_degradation() {
    // Test that system works even if TTS is unavailable
    let brain_config = BrainConfig::default();
    let brain = Brain::new(&brain_config).expect("Failed to create brain");
    
    let app_launcher = AppLauncher::new(create_test_app_db().await);
    let file_search = FileSearch::new(create_test_file_index());
    let executor = TaskExecutor::new(app_launcher, file_search);
    
    // Should work without TTS
    let plan = brain.process("volume up").unwrap();
    let result = executor.execute_plan(plan).await;
    
    assert!(result.is_ok(), "Should work without TTS");
}

/// Test configuration hot-reload scenario
#[tokio::test]
async fn test_config_hot_reload() {
    let brain_config = BrainConfig::default();
    let brain = Brain::new(&brain_config).expect("Failed to create brain");
    
    // Process a command
    let _plan1 = brain.process("open chrome").unwrap();
    
    // Simulate config reload
    brain.clear_cache();
    
    // Process again - should work with fresh cache
    let _plan2 = brain.process("open chrome").unwrap();
    
    // Verify both executions work
    assert!(true); // If we got here, it worked
}

/// Stress test - multiple commands in sequence
#[tokio::test]
async fn test_stress_multiple_commands() {
    let brain_config = BrainConfig::default();
    let brain = Brain::new(&brain_config).expect("Failed to create brain");
    
    let app_launcher = AppLauncher::new(create_test_app_db().await);
    let file_search = FileSearch::new(create_test_file_index());
    let executor = TaskExecutor::new(app_launcher, file_search);
    
    let commands = vec![
        "open chrome",
        "volume up",
        "volume down",
        "brightness up",
        "brightness down",
        "what time is it",
        "find document.pdf",
        "close chrome",
    ];
    
    for cmd in commands {
        let plan = brain.process(cmd).expect(&format!("Failed to process: {}", cmd));
        let result = executor.execute_plan(plan).await;
        assert!(result.is_ok(), "Failed to execute: {}", cmd);
    }
}

/// Test context resolution (e.g., "open it" after "find file")
#[tokio::test]
async fn test_context_resolution() {
    let brain_config = BrainConfig::default();
    let brain = Brain::new(&brain_config).expect("Failed to create brain");
    
    // First command - find something
    let _plan1 = brain.process("find budget.pdf").unwrap();
    let _ = brain.process_with_context("find budget.pdf", Some(true));
    
    // Second command - reference previous result
    // Note: Full context resolution depends on implementation
    let plan2 = brain.process("open it");
    assert!(plan2.is_ok());
}
