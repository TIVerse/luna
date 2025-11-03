//! Integration tests for newly implemented features
//!
//! Tests file_operations, reminders, CLI functions, and EventBus integration

use luna::actions::file_operations::{create_directory, move_file};
use luna::actions::reminders::create_reminder_with_bus;
use luna::events::{EventBus, LunaEvent};
use luna::metrics::Metrics;
use luna::db::{AppDatabase, FileIndex};
use tempfile::tempdir;
use chrono::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Test complete file operation workflow
#[tokio::test]
async fn test_file_operation_workflow() {
    let temp = tempdir().unwrap();
    
    // Create a project structure
    let project_dir = temp.path().join("my_project");
    let src_dir = project_dir.join("src");
    let docs_dir = project_dir.join("docs");
    
    // Create directories
    create_directory(&src_dir).await.unwrap();
    create_directory(&docs_dir).await.unwrap();
    
    assert!(src_dir.exists());
    assert!(docs_dir.exists());
    
    // Create some files
    tokio::fs::write(src_dir.join("main.rs"), b"fn main() {}").await.unwrap();
    tokio::fs::write(docs_dir.join("README.md"), b"# Project").await.unwrap();
    
    // Move files around
    let backup_dir = project_dir.join("backup");
    create_directory(&backup_dir).await.unwrap();
    
    move_file(&docs_dir.join("README.md"), &backup_dir.join("README.md"))
        .await
        .unwrap();
    
    assert!(!docs_dir.join("README.md").exists());
    assert!(backup_dir.join("README.md").exists());
}

/// Test EventBus with multiple reminders
#[tokio::test]
async fn test_eventbus_reminders_integration() {
    let bus = Arc::new(EventBus::new());
    let _handle = bus.start_processing().await;
    
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    
    // Subscribe to reminder events
    bus.subscribe(vec!["custom"], move |envelope| {
        if let LunaEvent::Custom { event_type, .. } = &envelope.event {
            if event_type == "reminder_triggered" {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        }
    })
    .await;
    
    // Create multiple reminders
    for i in 1..=5 {
        let message = format!("Reminder {}", i);
        let delay = Duration::milliseconds(20 * i as i64);
        create_reminder_with_bus(&message, delay, Some(Arc::clone(&bus)))
            .await
            .unwrap();
    }
    
    // Wait for all reminders to trigger
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    let count = counter.load(Ordering::SeqCst);
    assert_eq!(count, 5, "Expected 5 reminders to trigger, got {}", count);
}

/// Test end-to-end scenario: create files and set reminders
#[tokio::test]
async fn test_end_to_end_file_and_reminder() {
    let temp = tempdir().unwrap();
    let bus = Arc::new(EventBus::new());
    let _handle = bus.start_processing().await;
    
    let completed = Arc::new(AtomicBool::new(false));
    let completed_clone = Arc::clone(&completed);
    
    // Subscribe to events
    bus.subscribe(vec!["custom"], move |envelope| {
        if let LunaEvent::Custom { event_type, .. } = &envelope.event {
            if event_type == "reminder_triggered" {
                completed_clone.store(true, Ordering::SeqCst);
            }
        }
    })
    .await;
    
    // Step 1: Create a file
    let file_path = temp.path().join("task.txt");
    tokio::fs::write(&file_path, b"Task to complete").await.unwrap();
    assert!(file_path.exists());
    
    // Step 2: Create a directory for organizing
    let organized_dir = temp.path().join("organized");
    create_directory(&organized_dir).await.unwrap();
    
    // Step 3: Move file to organized directory
    let new_path = organized_dir.join("task.txt");
    move_file(&file_path, &new_path).await.unwrap();
    assert!(new_path.exists());
    assert!(!file_path.exists());
    
    // Step 4: Set reminder to review the task
    create_reminder_with_bus(
        "Review organized tasks",
        Duration::milliseconds(50),
        Some(bus),
    )
    .await
    .unwrap();
    
    // Wait for reminder
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    
    // Verify everything completed
    assert!(completed.load(Ordering::SeqCst));
}

/// Test error recovery: what happens when operations fail?
#[tokio::test]
async fn test_error_recovery_file_ops() {
    let temp = tempdir().unwrap();
    
    // Try to move a nonexistent file
    let result = move_file(
        &temp.path().join("nonexistent.txt"),
        &temp.path().join("destination.txt"),
    )
    .await;
    assert!(result.is_err());
    
    // Try to create directory where file exists
    let file_path = temp.path().join("regular_file");
    std::fs::write(&file_path, b"content").unwrap();
    
    let result = create_directory(&file_path).await;
    assert!(result.is_err());
    
    // System should still be functional after errors
    let valid_dir = temp.path().join("valid_directory");
    let result = create_directory(&valid_dir).await;
    assert!(result.is_ok());
}

/// Test concurrent operations
#[tokio::test]
async fn test_concurrent_file_operations() {
    let temp = tempdir().unwrap();
    
    // Create multiple directories concurrently
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let dir_path = temp.path().join(format!("dir_{}", i));
            tokio::spawn(async move {
                create_directory(&dir_path).await
            })
        })
        .collect();
    
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
    
    // Verify all directories exist
    for i in 0..10 {
        assert!(temp.path().join(format!("dir_{}", i)).exists());
    }
}

/// Test metrics integration with operations
#[tokio::test]
async fn test_metrics_integration() {
    let metrics = Arc::new(Metrics::new());
    
    // Simulate various operations
    for _ in 0..5 {
        metrics.record_command_processed();
        metrics.record_command_success();
    }
    
    for _ in 0..2 {
        metrics.record_command_processed();
        metrics.record_command_failure();
    }
    
    // Check metrics
    assert_eq!(metrics.get_commands_processed(), 7);
    assert_eq!(metrics.get_commands_succeeded(), 5);
    assert_eq!(metrics.get_commands_failed(), 2);
    
    let success_rate = metrics.get_success_rate();
    assert!((success_rate - 71.42).abs() < 0.1); // ~71.42%
}

/// Test database operations
#[tokio::test]
async fn test_database_integration() {
    let temp = tempdir().unwrap();
    
    // Create and use AppDatabase
    let mut app_db = AppDatabase::new();
    let app = luna::db::schema::Application::new(
        "Test App".to_string(),
        temp.path().join("testapp"),
    );
    app_db.add_app(app);
    
    assert_eq!(app_db.len(), 1);
    assert!(app_db.find_by_name("Test App").is_some());
    
    // Test save and load
    let db_path = temp.path().join("app_db.json");
    app_db.save_to_disk(&db_path).await.unwrap();
    assert!(db_path.exists());
    
    let loaded_db = AppDatabase::load_from_disk(&db_path).await.unwrap();
    assert_eq!(loaded_db.len(), 1);
}

/// Test file index operations
#[tokio::test]
async fn test_file_index_integration() {
    let temp = tempdir().unwrap();
    
    // Create some files
    tokio::fs::write(temp.path().join("document.txt"), b"content1").await.unwrap();
    tokio::fs::write(temp.path().join("report.pdf"), b"content2").await.unwrap();
    tokio::fs::write(temp.path().join("image.png"), b"content3").await.unwrap();
    
    // Create index
    let mut index = FileIndex::new();
    
    // Index files
    let mut entries = tokio::fs::read_dir(temp.path()).await.unwrap();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        if entry.path().is_file() {
            let metadata = entry.metadata().await.unwrap();
            let file_entry = luna::db::schema::FileEntry {
                path: entry.path(),
                name: entry.file_name().to_string_lossy().to_string(),
                extension: entry.path().extension()
                    .and_then(|s| s.to_str())
                    .map(String::from),
                size: metadata.len(),
                modified: metadata.modified().unwrap()
                    .duration_since(std::time::UNIX_EPOCH).unwrap()
                    .as_secs() as i64,
                file_type: luna::db::schema::FileType::Document,
            };
            index.add_file(file_entry);
        }
    }
    
    assert!(index.len() >= 3, "Expected at least 3 files, got {}", index.len());
    
    // Test retrieval of all files
    let all_files = index.all();
    assert!(all_files.len() >= 3);
    
    // Test search by full name (should have high similarity)
    let results = index.search_by_name("document.txt");
    assert!(results.len() >= 1, "Expected at least 1 result for 'document.txt', got {}. Files: {:?}", 
        results.len(), 
        all_files.iter().map(|f| &f.name).collect::<Vec<_>>()
    );
    
    // Test save and load
    let index_path = temp.path().join("file_index.json");
    index.save_to_disk(&index_path).await.unwrap();
    assert!(index_path.exists());
    
    let loaded_index = FileIndex::load_from_disk(&index_path).await.unwrap();
    assert_eq!(loaded_index.len(), index.len());
}

/// Test EventBus under load
#[tokio::test]
async fn test_eventbus_load() {
    let bus = Arc::new(EventBus::new());
    let _handle = bus.start_processing().await;
    
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    
    bus.subscribe(vec![], move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    })
    .await;
    
    // Publish many events
    for i in 0..100 {
        bus.publish(LunaEvent::Custom {
            event_type: format!("test_{}", i),
            data: serde_json::json!({ "index": i }),
        })
        .await;
    }
    
    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    let count = counter.load(Ordering::SeqCst);
    assert!(count >= 100, "Expected 100 events, got {}", count);
}

/// Test graceful shutdown scenario
#[tokio::test]
async fn test_graceful_shutdown() {
    let bus = Arc::new(EventBus::new());
    let handle = bus.start_processing().await;
    
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    
    bus.subscribe(vec![], move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    })
    .await;
    
    // Publish some events
    for _ in 0..5 {
        bus.publish(LunaEvent::StateChanged {
            from: "state1".to_string(),
            to: "state2".to_string(),
        })
        .await;
    }
    
    // Abort the handle (simulating shutdown)
    handle.abort();
    
    // System should still be in valid state
    let count = counter.load(Ordering::SeqCst);
    assert!(count <= 5);
}

/// Test memory leak prevention
#[tokio::test]
async fn test_no_memory_leaks() {
    let bus = Arc::new(EventBus::new());
    let _handle = bus.start_processing().await;
    
    // Subscribe and unsubscribe multiple times
    for _ in 0..10 {
        let id = bus.subscribe(vec![], |_| {}).await;
        bus.unsubscribe(id).await;
    }
    
    // Bus should still work
    assert_eq!(bus.subscriber_count().await, 0);
    
    bus.publish(LunaEvent::StateChanged {
        from: "a".to_string(),
        to: "b".to_string(),
    })
    .await;
}

/// Test complex nested directory operations
#[tokio::test]
async fn test_complex_directory_operations() {
    let temp = tempdir().unwrap();
    
    // Create deep nested structure
    let deep_path = temp
        .path()
        .join("a")
        .join("b")
        .join("c")
        .join("d")
        .join("e");
    
    create_directory(&deep_path).await.unwrap();
    assert!(deep_path.exists());
    
    // Create files in various levels
    tokio::fs::write(temp.path().join("a").join("file1.txt"), b"1")
        .await
        .unwrap();
    tokio::fs::write(
        temp.path().join("a").join("b").join("file2.txt"),
        b"2",
    )
    .await
    .unwrap();
    tokio::fs::write(deep_path.join("file3.txt"), b"3")
        .await
        .unwrap();
    
    // Move entire tree
    let backup = temp.path().join("backup");
    move_file(&temp.path().join("a"), &backup).await.unwrap();
    
    assert!(backup.exists());
    assert!(backup.join("file1.txt").exists());
    assert!(backup.join("b").join("file2.txt").exists());
    assert!(backup
        .join("b")
        .join("c")
        .join("d")
        .join("e")
        .join("file3.txt")
        .exists());
}

/// Test reminder edge cases with EventBus
#[tokio::test]
async fn test_reminder_edge_cases() {
    let bus = Arc::new(EventBus::new());
    let _handle = bus.start_processing().await;
    
    // Very short reminder (1ms)
    let triggered = Arc::new(AtomicBool::new(false));
    let triggered_clone = Arc::clone(&triggered);
    
    bus.subscribe(vec!["custom"], move |envelope| {
        if let LunaEvent::Custom { event_type, .. } = &envelope.event {
            if event_type == "reminder_triggered" {
                triggered_clone.store(true, Ordering::SeqCst);
            }
        }
    })
    .await;
    
    create_reminder_with_bus("quick", Duration::milliseconds(1), Some(bus))
        .await
        .unwrap();
    
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    assert!(triggered.load(Ordering::SeqCst));
}
