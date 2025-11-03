//! Reminders and notifications
//!
//! Create reminders and display notifications.

use crate::error::{LunaError, Result};
use crate::events::{EventBus, LunaEvent};
use chrono::Duration;
use notify_rust::Notification;
use std::sync::Arc;
use tokio::time;
use tracing::{debug, info, warn};

/// Create a reminder
/// 
/// Schedules a timer that will trigger an OS notification and emit an event
/// when the duration expires.
/// 
/// # Arguments
/// * `message` - The reminder message to display
/// * `duration` - How long to wait before triggering the reminder
/// 
/// # Errors
/// Returns `LunaError::InvalidParameter` if duration is negative or zero.
/// Returns `LunaError::SystemOperation` if notification fails to display.
pub async fn create_reminder(message: &str, duration: Duration) -> Result<()> {
    create_reminder_with_bus(message, duration, None).await
}

/// Create a reminder with EventBus for notifications
/// 
/// This is the full implementation that can optionally emit events.
/// 
/// # Arguments
/// * `message` - The reminder message
/// * `duration` - How long to wait
/// * `event_bus` - Optional EventBus for emitting events
pub async fn create_reminder_with_bus(
    message: &str,
    duration: Duration,
    event_bus: Option<Arc<EventBus>>,
) -> Result<()> {
    debug!("Creating reminder: '{}' in {:?}", message, duration);
    
    // Validate duration
    if duration.num_milliseconds() <= 0 {
        return Err(LunaError::InvalidParameter(
            "Reminder duration must be positive".to_string(),
        ));
    }
    
    let message = message.to_string();
    let millis = duration.num_milliseconds() as u64;
    
    info!("⏰ Reminder set: '{}' in {}ms", message, millis);
    
    // Spawn a background task for the reminder
    tokio::spawn(async move {
        // Wait for the specified duration
        time::sleep(time::Duration::from_millis(millis)).await;
        
        debug!("Reminder triggered: '{}'", message);
        
        // Show OS notification
        if let Err(e) = show_notification(&message) {
            warn!("Failed to show notification: {}", e);
        }
        
        // Emit event if EventBus is available
        if let Some(bus) = event_bus {
            bus.publish(LunaEvent::Custom {
                event_type: "reminder_triggered".to_string(),
                data: serde_json::json!({
                    "message": message,
                    "timestamp": chrono::Utc::now().timestamp(),
                }),
            })
            .await;
        }
        
        info!("✅ Reminder completed: '{}'", message);
    });
    
    Ok(())
}

/// Show an OS notification
fn show_notification(message: &str) -> Result<()> {
    #[cfg(feature = "notifications")]
    {
        Notification::new()
            .summary("LUNA Reminder")
            .body(message)
            .icon("dialog-information")
            .timeout(notify_rust::Timeout::Milliseconds(5000))
            .show()
            .map_err(|e| {
                LunaError::SystemOperation(format!("Failed to show notification: {}", e))
            })?;
        Ok(())
    }
    
    #[cfg(not(feature = "notifications"))]
    {
        // Fallback: just log if notifications feature is disabled
        info!("🔔 Reminder: {}", message);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    #[tokio::test]
    async fn test_create_reminder_validates_duration() {
        // Zero duration should fail
        let result = create_reminder("test", Duration::zero()).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("positive"));
        
        // Negative duration should fail
        let result = create_reminder("test", Duration::milliseconds(-100)).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("positive"));
    }
    
    #[tokio::test]
    async fn test_create_reminder_accepts_positive_duration() {
        // Positive duration should succeed
        let result = create_reminder("test", Duration::milliseconds(10)).await;
        assert!(result.is_ok());
        
        // Very small positive duration should also succeed
        let result = create_reminder("test", Duration::milliseconds(1)).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_reminder_actually_triggers() {
        use std::sync::atomic::{AtomicBool, Ordering};
        
        let triggered = Arc::new(AtomicBool::new(false));
        let triggered_clone = Arc::clone(&triggered);
        
        let bus = Arc::new(EventBus::new());
        let _handle = bus.start_processing().await;
        
        // Subscribe to reminder events
        let bus_clone = bus.clone();
        bus.subscribe(vec!["custom"], move |envelope| {
            if let LunaEvent::Custom { event_type, .. } = &envelope.event {
                if event_type == "reminder_triggered" {
                    triggered_clone.store(true, Ordering::SeqCst);
                }
            }
        })
        .await;
        
        // Create reminder with very short duration
        create_reminder_with_bus("test", Duration::milliseconds(50), Some(bus_clone))
            .await
            .unwrap();
        
        // Wait for reminder to trigger
        time::sleep(time::Duration::from_millis(150)).await;
        
        // Verify it triggered
        assert!(triggered.load(Ordering::SeqCst));
    }
    
    #[tokio::test]
    async fn test_reminder_contains_correct_message() {
        use std::sync::Mutex;
        
        let captured_message = Arc::new(Mutex::new(String::new()));
        let message_clone = Arc::clone(&captured_message);
        
        let bus = Arc::new(EventBus::new());
        let _handle = bus.start_processing().await;
        
        // Subscribe and capture the message
        let bus_clone = bus.clone();
        bus.subscribe(vec!["custom"], move |envelope| {
            if let LunaEvent::Custom { event_type, data } = &envelope.event {
                if event_type == "reminder_triggered" {
                    if let Some(msg) = data.get("message").and_then(|v| v.as_str()) {
                        *message_clone.lock().unwrap() = msg.to_string();
                    }
                }
            }
        })
        .await;
        
        let test_message = "Important reminder message";
        create_reminder_with_bus(test_message, Duration::milliseconds(50), Some(bus_clone))
            .await
            .unwrap();
        
        time::sleep(time::Duration::from_millis(150)).await;
        
        let captured = captured_message.lock().unwrap();
        assert_eq!(*captured, test_message);
    }
    
    #[tokio::test]
    async fn test_multiple_reminders_trigger_independently() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);
        
        let bus = Arc::new(EventBus::new());
        let _handle = bus.start_processing().await;
        
        bus.subscribe(vec!["custom"], move |envelope| {
            if let LunaEvent::Custom { event_type, .. } = &envelope.event {
                if event_type == "reminder_triggered" {
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                }
            }
        })
        .await;
        
        // Create multiple reminders with different delays
        let bus_clone = bus.clone();
        create_reminder_with_bus("reminder 1", Duration::milliseconds(30), Some(bus_clone.clone()))
            .await
            .unwrap();
        
        create_reminder_with_bus("reminder 2", Duration::milliseconds(50), Some(bus_clone.clone()))
            .await
            .unwrap();
        
        create_reminder_with_bus("reminder 3", Duration::milliseconds(70), Some(bus_clone))
            .await
            .unwrap();
        
        // Wait for all to trigger
        time::sleep(time::Duration::from_millis(150)).await;
        
        // All three should have triggered
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
    
    #[tokio::test]
    async fn test_reminder_with_long_message() {
        use std::sync::atomic::{AtomicBool, Ordering};
        
        let triggered = Arc::new(AtomicBool::new(false));
        let triggered_clone = Arc::clone(&triggered);
        
        let bus = Arc::new(EventBus::new());
        let _handle = bus.start_processing().await;
        
        bus.subscribe(vec!["custom"], move |envelope| {
            if let LunaEvent::Custom { event_type, .. } = &envelope.event {
                if event_type == "reminder_triggered" {
                    triggered_clone.store(true, Ordering::SeqCst);
                }
            }
        })
        .await;
        
        let long_message = "This is a very long reminder message with lots of text. ".repeat(10);
        
        create_reminder_with_bus(&long_message, Duration::milliseconds(50), Some(bus))
            .await
            .unwrap();
        
        time::sleep(time::Duration::from_millis(150)).await;
        
        assert!(triggered.load(Ordering::SeqCst));
    }
    
    #[tokio::test]
    async fn test_reminder_with_special_characters() {
        use std::sync::Mutex;
        
        let captured = Arc::new(Mutex::new(String::new()));
        let captured_clone = Arc::clone(&captured);
        
        let bus = Arc::new(EventBus::new());
        let _handle = bus.start_processing().await;
        
        bus.subscribe(vec!["custom"], move |envelope| {
            if let LunaEvent::Custom { event_type, data } = &envelope.event {
                if event_type == "reminder_triggered" {
                    if let Some(msg) = data.get("message").and_then(|v| v.as_str()) {
                        *captured_clone.lock().unwrap() = msg.to_string();
                    }
                }
            }
        })
        .await;
        
        let special_message = "Reminder with special chars: <>&\"'\\n\\t 你好 🚀";
        
        create_reminder_with_bus(special_message, Duration::milliseconds(50), Some(bus))
            .await
            .unwrap();
        
        time::sleep(time::Duration::from_millis(150)).await;
        
        let result = captured.lock().unwrap();
        assert_eq!(*result, special_message);
    }
    
    #[tokio::test]
    async fn test_reminder_without_event_bus() {
        // Should succeed even without EventBus
        let result = create_reminder("test without bus", Duration::milliseconds(50)).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_reminder_with_various_durations() {
        // Test different duration units
        let result1 = create_reminder("1ms", Duration::milliseconds(1)).await;
        assert!(result1.is_ok());
        
        let result2 = create_reminder("1sec", Duration::seconds(1)).await;
        assert!(result2.is_ok());
        
        let result3 = create_reminder("1min", Duration::minutes(1)).await;
        assert!(result3.is_ok());
        
        let result4 = create_reminder("1hour", Duration::hours(1)).await;
        assert!(result4.is_ok());
    }
    
    #[tokio::test]
    async fn test_reminder_includes_timestamp() {
        use std::sync::Mutex;
        
        let captured_timestamp = Arc::new(Mutex::new(0i64));
        let timestamp_clone = Arc::clone(&captured_timestamp);
        
        let bus = Arc::new(EventBus::new());
        let _handle = bus.start_processing().await;
        
        bus.subscribe(vec!["custom"], move |envelope| {
            if let LunaEvent::Custom { event_type, data } = &envelope.event {
                if event_type == "reminder_triggered" {
                    if let Some(ts) = data.get("timestamp").and_then(|v| v.as_i64()) {
                        *timestamp_clone.lock().unwrap() = ts;
                    }
                }
            }
        })
        .await;
        
        let before = chrono::Utc::now().timestamp();
        
        create_reminder_with_bus("test", Duration::milliseconds(50), Some(bus))
            .await
            .unwrap();
        
        time::sleep(time::Duration::from_millis(150)).await;
        
        let after = chrono::Utc::now().timestamp();
        let captured = *captured_timestamp.lock().unwrap();
        
        // Timestamp should be between before and after
        assert!(captured >= before && captured <= after);
    }
}
