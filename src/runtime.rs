//! LUNA Runtime orchestration and lifecycle management
//!
//! Provides:
//! - Unified component lifecycle management
//! - Graceful startup and shutdown
//! - Signal handling (SIGINT, SIGTERM)
//! - Component dependency ordering

use crate::error::{LunaError, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Lifecycle trait for all LUNA components
#[async_trait]
pub trait Lifecycle: Send + Sync {
    /// Component name for logging
    fn name(&self) -> &str;

    /// Start the component
    async fn start(&mut self) -> Result<()>;

    /// Stop the component gracefully
    async fn stop(&mut self) -> Result<()>;

    /// Check if component is running
    fn is_running(&self) -> bool;

    /// Health check (optional, returns true by default)
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

/// Runtime state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    /// Not yet started
    Stopped,
    /// Starting up
    Starting,
    /// Running normally
    Running,
    /// Shutting down
    Stopping,
    /// Error state
    Error,
}

/// LUNA runtime that manages component lifecycle
pub struct LunaRuntime {
    /// Registered components in startup order
    components: Vec<Box<dyn Lifecycle>>,

    /// Current runtime state
    state: Arc<RwLock<RuntimeState>>,

    /// Shutdown signal
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl LunaRuntime {
    /// Create a new runtime
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            state: Arc::new(RwLock::new(RuntimeState::Stopped)),
            shutdown_tx: None,
        }
    }

    /// Register a component (added to the end of startup order)
    pub fn register(&mut self, component: Box<dyn Lifecycle>) {
        info!("Registering component: {}", component.name());
        self.components.push(component);
    }

    /// Start all components in order
    pub async fn start(&mut self) -> Result<()> {
        *self.state.write().await = RuntimeState::Starting;
        info!(
            "🚀 Starting LUNA runtime with {} components",
            self.components.len()
        );

        // Create shutdown channel
        let (tx, _rx) = tokio::sync::broadcast::channel(1);
        self.shutdown_tx = Some(tx);

        // Start components in order
        for component in &mut self.components {
            info!("Starting component: {}", component.name());

            match component.start().await {
                Ok(_) => {
                    info!("✅ Component started: {}", component.name());
                }
                Err(e) => {
                    error!("❌ Failed to start component {}: {}", component.name(), e);
                    *self.state.write().await = RuntimeState::Error;

                    // Attempt to stop already-started components
                    self.stop_all_components().await;

                    return Err(e);
                }
            }
        }

        *self.state.write().await = RuntimeState::Running;
        info!("✅ All components started successfully");

        Ok(())
    }

    /// Stop all components in reverse order (graceful shutdown)
    pub async fn stop(&mut self) -> Result<()> {
        *self.state.write().await = RuntimeState::Stopping;
        info!("🛑 Stopping LUNA runtime");

        // Signal shutdown to all listeners
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }

        self.stop_all_components().await;

        *self.state.write().await = RuntimeState::Stopped;
        info!("✅ LUNA runtime stopped");

        Ok(())
    }

    /// Stop all components in reverse startup order
    async fn stop_all_components(&mut self) {
        // Stop in reverse order
        for component in self.components.iter_mut().rev() {
            if component.is_running() {
                info!("Stopping component: {}", component.name());

                match component.stop().await {
                    Ok(_) => {
                        info!("✅ Component stopped: {}", component.name());
                    }
                    Err(e) => {
                        error!("⚠️  Error stopping component {}: {}", component.name(), e);
                        // Continue stopping other components
                    }
                }
            }
        }
    }

    /// Get current runtime state
    pub async fn state(&self) -> RuntimeState {
        *self.state.read().await
    }

    /// Run health checks on all components
    pub async fn health_check(&self) -> Result<bool> {
        let mut all_healthy = true;

        for component in &self.components {
            match component.health_check().await {
                Ok(healthy) => {
                    if !healthy {
                        warn!("Component {} is unhealthy", component.name());
                        all_healthy = false;
                    }
                }
                Err(e) => {
                    error!("Health check failed for {}: {}", component.name(), e);
                    all_healthy = false;
                }
            }
        }

        Ok(all_healthy)
    }

    /// Get shutdown signal receiver
    pub fn shutdown_signal(&self) -> Option<tokio::sync::broadcast::Receiver<()>> {
        self.shutdown_tx.as_ref().map(|tx| tx.subscribe())
    }

    /// Setup signal handlers for graceful shutdown
    pub async fn setup_signal_handlers(&mut self) -> Result<()> {
        use tokio::signal;

        let state = self.state.clone();
        let shutdown_tx = self.shutdown_tx.clone();

        tokio::spawn(async move {
            let ctrl_c = async {
                signal::ctrl_c()
                    .await
                    .expect("Failed to install Ctrl+C handler");
            };

            #[cfg(unix)]
            let terminate = async {
                signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("Failed to install signal handler")
                    .recv()
                    .await;
            };

            #[cfg(not(unix))]
            let terminate = std::future::pending::<()>();

            tokio::select! {
                _ = ctrl_c => {
                    info!("Received Ctrl+C signal");
                }
                _ = terminate => {
                    info!("Received terminate signal");
                }
            }

            *state.write().await = RuntimeState::Stopping;
            if let Some(tx) = shutdown_tx {
                let _ = tx.send(());
            }
        });

        info!("✅ Signal handlers installed");
        Ok(())
    }

    /// Print runtime summary
    pub async fn print_summary(&self) {
        let state = self.state().await;

        println!("\n📊 LUNA Runtime Summary");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  State: {:?}", state);
        println!("  Components: {}", self.components.len());

        for (i, component) in self.components.iter().enumerate() {
            let status = if component.is_running() {
                "✅ Running"
            } else {
                "⏸️  Stopped"
            };
            println!("    {}. {} - {}", i + 1, component.name(), status);
        }

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}

impl Default for LunaRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create a no-op component for testing
#[cfg(test)]
pub struct TestComponent {
    name: String,
    running: bool,
}

#[cfg(test)]
impl TestComponent {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            running: false,
        }
    }
}

#[cfg(test)]
#[async_trait]
impl Lifecycle for TestComponent {
    fn name(&self) -> &str {
        &self.name
    }

    async fn start(&mut self) -> Result<()> {
        self.running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_creation() {
        let runtime = LunaRuntime::new();
        assert_eq!(runtime.state().await, RuntimeState::Stopped);
    }

    #[tokio::test]
    async fn test_component_registration() {
        let mut runtime = LunaRuntime::new();
        runtime.register(Box::new(TestComponent::new("test1")));
        runtime.register(Box::new(TestComponent::new("test2")));

        assert_eq!(runtime.components.len(), 2);
    }

    #[tokio::test]
    async fn test_start_stop() {
        let mut runtime = LunaRuntime::new();
        runtime.register(Box::new(TestComponent::new("test")));

        runtime.start().await.unwrap();
        assert_eq!(runtime.state().await, RuntimeState::Running);

        runtime.stop().await.unwrap();
        assert_eq!(runtime.state().await, RuntimeState::Stopped);
    }

    #[tokio::test]
    async fn test_health_check() {
        let mut runtime = LunaRuntime::new();
        runtime.register(Box::new(TestComponent::new("test")));

        runtime.start().await.unwrap();
        let healthy = runtime.health_check().await.unwrap();
        assert!(healthy);
    }
}
