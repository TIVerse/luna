//! Clipboard operations (GOD-LEVEL Enhancement)
//!
//! Cross-platform clipboard read/write with desktop notifications.

use crate::error::{LunaError, Result};
use tracing::{info, warn};

#[cfg(feature = "clipboard")]
use clipboard::{ClipboardContext, ClipboardProvider};

/// Clipboard handler
pub struct Clipboard;

impl Clipboard {
    /// Create a new clipboard handler
    pub fn new() -> Self {
        Self
    }

    /// Read clipboard content
    pub async fn read(&self) -> Result<String> {
        info!("Reading clipboard");

        #[cfg(feature = "clipboard")]
        {
            let mut ctx: ClipboardContext = ClipboardProvider::new().map_err(|e| {
                LunaError::SystemOperation(format!("Failed to access clipboard: {}", e))
            })?;

            ctx.get_contents()
                .map_err(|e| LunaError::SystemOperation(format!("Failed to read clipboard: {}", e)))
        }

        #[cfg(not(feature = "clipboard"))]
        {
            warn!("Clipboard feature not enabled. Enable with --features clipboard");
            Err(LunaError::SystemOperation(
                "Clipboard support not compiled in. Rebuild with --features clipboard".to_string(),
            ))
        }
    }

    /// Write text to clipboard
    pub async fn write(&self, text: &str) -> Result<String> {
        info!("Writing to clipboard: {} chars", text.len());

        #[cfg(feature = "clipboard")]
        {
            let mut ctx: ClipboardContext = ClipboardProvider::new().map_err(|e| {
                LunaError::SystemOperation(format!("Failed to access clipboard: {}", e))
            })?;

            ctx.set_contents(text.to_string()).map_err(|e| {
                LunaError::SystemOperation(format!("Failed to write clipboard: {}", e))
            })?;

            // Send desktop notification
            self.notify_clipboard_action("Copied to clipboard", text)
                .await;

            Ok(format!("Copied {} characters to clipboard", text.len()))
        }

        #[cfg(not(feature = "clipboard"))]
        {
            warn!("Clipboard feature not enabled. Enable with --features clipboard");
            Err(LunaError::SystemOperation(
                "Clipboard support not compiled in. Rebuild with --features clipboard".to_string(),
            ))
        }
    }

    /// Clear clipboard
    pub async fn clear(&self) -> Result<String> {
        info!("Clearing clipboard");
        self.write("").await?;
        Ok("Clipboard cleared".to_string())
    }

    /// Send desktop notification for clipboard action
    async fn notify_clipboard_action(&self, summary: &str, body: &str) {
        #[cfg(all(feature = "notifications", not(target_os = "macos")))]
        {
            use notify_rust::Notification;

            let body_preview = if body.len() > 50 {
                format!("{}...", &body[..50])
            } else {
                body.to_string()
            };

            if let Err(e) = Notification::new()
                .summary(summary)
                .body(&body_preview)
                .icon("edit-copy")
                .timeout(3000)
                .show()
            {
                warn!("Failed to show notification: {}", e);
            }
        }
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

/// Read clipboard content (convenience function)
pub async fn read_clipboard() -> Result<String> {
    Clipboard::new().read().await
}

/// Write to clipboard (convenience function)
pub async fn write_clipboard(content: &str) -> Result<()> {
    Clipboard::new().write(content).await?;
    Ok(())
}
