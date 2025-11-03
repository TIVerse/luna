//! Event subscribers for LUNA
//!
//! Provides built-in event subscribers for common tasks like logging and metrics.

use crate::events::{EventBus, EventEnvelope, LunaEvent};
use crate::metrics::Metrics;
use std::sync::Arc;
use tracing::{error, info, warn, debug};

/// Setup logging subscriber to log all events
pub async fn setup_logging_subscriber(event_bus: &EventBus) -> usize {
    event_bus.subscribe(vec![], |envelope: &EventEnvelope| {
        match &envelope.event {
            LunaEvent::WakeWordDetected { keyword, confidence } => {
                info!("🎤 Wake word '{}' detected (confidence: {:.2})", keyword, confidence);
            }
            LunaEvent::CommandTranscribed { text, confidence } => {
                info!("📝 Transcribed: '{}' (confidence: {:.2})", text, confidence);
            }
            LunaEvent::CommandParsed { command, intent, .. } => {
                info!("🧠 Parsed command: '{}' -> intent: {}", command, intent);
            }
            LunaEvent::TaskPlanned { plan, .. } => {
                debug!("📋 Task plan created: {}", plan);
            }
            LunaEvent::ActionStarted { action_type, params } => {
                info!("▶️  Starting action: {} with {} params", action_type, params.len());
            }
            LunaEvent::ActionCompleted { action_type, success, duration_ms, .. } => {
                if *success {
                    info!("✅ Action '{}' completed in {}ms", action_type, duration_ms);
                } else {
                    warn!("⚠️  Action '{}' failed after {}ms", action_type, duration_ms);
                }
            }
            LunaEvent::Error { error, error_code, .. } => {
                error!("❌ Error [{}]: {}", error_code, error);
            }
            LunaEvent::StateChanged { from, to } => {
                info!("🔄 State changed: {} -> {}", from, to);
            }
            LunaEvent::AudioCaptured { samples, timestamp } => {
                debug!("🎙️  Audio captured: {} samples at {}", samples.len(), timestamp);
            }
            LunaEvent::ConfigReloaded { .. } => {
                info!("🔄 Configuration reloaded");
            }
            LunaEvent::MetricsSnapshot { .. } => {
                debug!("📊 Metrics snapshot");
            }
            LunaEvent::ClarificationRequested { command, confidence, missing_slots, suggestions } => {
                warn!("❓ Clarification needed for '{}' (confidence: {:.2})", command, confidence);
                if !missing_slots.is_empty() {
                    info!("   Missing: {:?}", missing_slots);
                }
                if !suggestions.is_empty() {
                    info!("   Suggestions: {:?}", suggestions);
                }
            }
            LunaEvent::ClarificationAnswered { original_command, clarification, resolved_command } => {
                info!("✅ Clarification: '{}' + '{}' → '{}'", 
                      original_command, clarification, resolved_command);
            }
            LunaEvent::GrammarReloaded { pattern_count, timestamp } => {
                info!("🔄 Grammar reloaded: {} patterns at timestamp {}", pattern_count, timestamp);
            }
            LunaEvent::CacheInvalidated { cache_type, reason } => {
                info!("🗑️  Cache '{}' invalidated: {}", cache_type, reason);
            }
            LunaEvent::PlanStarted { plan_id, step_count, parallel_groups } => {
                info!("🚀 Plan {} started: {} steps, {} parallel groups", plan_id, step_count, parallel_groups);
            }
            LunaEvent::PlanCompleted { plan_id, success, total_duration_ms, steps_completed, steps_failed } => {
                if *success {
                    info!("✅ Plan {} completed in {}ms: {} steps succeeded", 
                          plan_id, total_duration_ms, steps_completed);
                } else {
                    warn!("⚠️  Plan {} failed after {}ms: {} succeeded, {} failed", 
                          plan_id, total_duration_ms, steps_completed, steps_failed);
                }
            }
            LunaEvent::CapabilityDetected { capability, available, tool_name, version } => {
                if *available {
                    info!("✅ Capability '{}' available: {} {}", 
                          capability, 
                          tool_name.as_ref().unwrap_or(&"unknown".to_string()),
                          version.as_ref().map(|v| format!("({})", v)).unwrap_or_default());
                } else {
                    warn!("❌ Capability '{}' not available: {} not found", 
                          capability,
                          tool_name.as_ref().unwrap_or(&"unknown".to_string()));
                }
            }
            LunaEvent::ActionRetry { action_type, attempt, max_attempts, error } => {
                warn!("🔄 Retrying {} (attempt {}/{}): {}", action_type, attempt, max_attempts, error);
            }
            LunaEvent::PolicyGateTriggered { action_type, requires_confirmation, reason } => {
                if *requires_confirmation {
                    warn!("🔒 Policy gate: {} requires confirmation - {}", action_type, reason);
                } else {
                    info!("🔓 Policy gate: {} - {}", action_type, reason);
                }
            }
            LunaEvent::TtsStarted { text, kind } => {
                debug!("🔊 TTS started ({}): \"{}\"", kind, text.chars().take(50).collect::<String>());
            }
            LunaEvent::TtsCompleted { success } => {
                if *success {
                    debug!("✅ TTS completed successfully");
                } else {
                    warn!("⚠️  TTS completed with errors");
                }
            }
            LunaEvent::TtsInterrupted { reason } => {
                info!("⏹️  TTS interrupted: {}", reason);
            }
            LunaEvent::HealthIssueDetected { issue_type, severity } => {
                match severity.as_str() {
                    "Critical" => error!("🚨 Health issue ({}): {}", severity, issue_type),
                    "Warning" => warn!("⚠️  Health issue ({}): {}", severity, issue_type),
                    _ => info!("ℹ️  Health issue ({}): {}", severity, issue_type),
                }
            }
            LunaEvent::HealthRemediated { issue_type, action_taken } => {
                info!("🔧 Health remediation for {}: {}", issue_type, action_taken);
            }
            LunaEvent::Custom { event_type, data } => {
                debug!("🔧 Custom event '{}': {:?}", event_type, data);
            }
        }
    }).await
}

/// Setup metrics subscriber to track performance
pub async fn setup_metrics_subscriber(event_bus: &EventBus, metrics: Arc<Metrics>) -> usize {
    event_bus.subscribe(vec![], move |envelope: &EventEnvelope| {
        match &envelope.event {
            LunaEvent::WakeWordDetected { .. } => {
                metrics.record_wake_word();
            }
            LunaEvent::CommandTranscribed { .. } => {
                metrics.record_command_processed();
            }
            LunaEvent::ActionCompleted { success, .. } => {
                if *success {
                    metrics.record_command_success();
                } else {
                    metrics.record_command_failure();
                }
            }
            _ => {}
        }
    }).await
}

/// Setup detailed debug subscriber for troubleshooting
pub async fn setup_debug_subscriber(event_bus: &EventBus) -> usize {
    event_bus.subscribe(vec![], |envelope: &EventEnvelope| {
        debug!("🔍 Event: {:?} (ID: {}, Correlation: {:?})", 
               envelope.event, envelope.id, envelope.correlation_id);
    }).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_logging_subscriber() {
        let bus = EventBus::new();
        let handle = bus.start_processing().await;
        
        let _id = setup_logging_subscriber(&bus).await;
        
        bus.publish(LunaEvent::WakeWordDetected {
            keyword: "hey luna".to_string(),
            confidence: 0.95,
        }).await;
        
        sleep(Duration::from_millis(50)).await;
        // If we get here without panicking, the subscriber worked
        
        handle.abort();
    }
    
    #[tokio::test]
    async fn test_metrics_subscriber() {
        let bus = EventBus::new();
        let handle = bus.start_processing().await;
        
        let metrics = Arc::new(Metrics::new());
        let _id = setup_metrics_subscriber(&bus, metrics.clone()).await;
        
        bus.publish(LunaEvent::WakeWordDetected {
            keyword: "hey luna".to_string(),
            confidence: 0.95,
        }).await;
        
        sleep(Duration::from_millis(50)).await;
        
        assert_eq!(metrics.get_wake_words_detected(), 1);
        
        handle.abort();
    }
    
    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let handle = bus.start_processing().await;
        
        let metrics = Arc::new(Metrics::new());
        
        let _log_id = setup_logging_subscriber(&bus).await;
        let _metrics_id = setup_metrics_subscriber(&bus, metrics.clone()).await;
        let _debug_id = setup_debug_subscriber(&bus).await;
        
        bus.publish(LunaEvent::ActionCompleted {
            action_type: "test".to_string(),
            success: true,
            result: "Success".to_string(),
            duration_ms: 100,
        }).await;
        
        sleep(Duration::from_millis(50)).await;
        
        // All subscribers should have processed the event
        assert_eq!(metrics.get_commands_succeeded(), 1);
        
        handle.abort();
    }
}
