//! Priority queue for TTS messages
//!
//! Manages queuing, prioritization, and cancellation of TTS utterances.

use super::types::{MessageKind, Priority, TtsHandle};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// A queued TTS message
#[derive(Debug, Clone)]
pub struct TtsMessage {
    pub id: u64,
    pub text: String,
    pub kind: MessageKind,
    pub priority: Priority,
    pub is_ssml: bool,
    pub coalesce_key: Option<String>,
}

impl TtsMessage {
    pub fn new(text: String, kind: MessageKind) -> Self {
        Self {
            id: NEXT_ID.fetch_add(1, AtomicOrdering::Relaxed),
            text,
            kind,
            priority: kind.default_priority(),
            is_ssml: false,
            coalesce_key: None,
        }
    }

    pub fn with_ssml(mut self, is_ssml: bool) -> Self {
        self.is_ssml = is_ssml;
        self
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_coalesce_key(mut self, key: String) -> Self {
        self.coalesce_key = Some(key);
        self
    }
}

/// Priority queue wrapper for BinaryHeap
#[derive(Debug)]
struct PriorityMessage(TtsMessage);

impl PartialEq for PriorityMessage {
    fn eq(&self, other: &Self) -> bool {
        self.0.priority == other.0.priority && self.0.id == other.0.id
    }
}

impl Eq for PriorityMessage {}

impl PartialOrd for PriorityMessage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityMessage {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority comes first, then FIFO (lower ID first)
        match self.0.priority.cmp(&other.0.priority) {
            Ordering::Equal => other.0.id.cmp(&self.0.id), // Reverse for FIFO
            other => other,
        }
    }
}

/// TTS queue manager
pub struct TtsQueue {
    queue: Arc<RwLock<BinaryHeap<PriorityMessage>>>,
    cancel_tx: mpsc::UnboundedSender<u64>,
    cancel_rx: Arc<RwLock<mpsc::UnboundedReceiver<u64>>>,
}

impl TtsQueue {
    pub fn new() -> Self {
        let (cancel_tx, cancel_rx) = mpsc::unbounded_channel();

        Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            cancel_tx,
            cancel_rx: Arc::new(RwLock::new(cancel_rx)),
        }
    }

    /// Enqueue a message and return a handle
    pub async fn enqueue(&self, message: TtsMessage) -> TtsHandle {
        let id = message.id;

        // Check for coalescing
        if let Some(key) = &message.coalesce_key {
            let mut queue = self.queue.write().await;
            // Remove existing messages with same coalesce key
            let filtered: Vec<_> = queue
                .drain()
                .filter(|m| m.0.coalesce_key.as_ref() != Some(key))
                .collect();

            *queue = filtered.into_iter().collect();
        }

        self.queue.write().await.push(PriorityMessage(message));
        TtsHandle::new(id, self.cancel_tx.clone())
    }

    /// Dequeue highest priority message
    pub async fn dequeue(&self) -> Option<TtsMessage> {
        self.queue.write().await.pop().map(|m| m.0)
    }

    /// Check if a message should be cancelled
    pub async fn check_cancellations(&self) -> Vec<u64> {
        let mut cancelled = Vec::new();
        let mut rx = self.cancel_rx.write().await;

        while let Ok(id) = rx.try_recv() {
            cancelled.push(id);
        }

        cancelled
    }

    /// Clear all messages
    pub async fn clear(&self) {
        self.queue.write().await.clear();
    }

    /// Get queue size
    pub async fn len(&self) -> usize {
        self.queue.read().await.len()
    }

    /// Check if queue is empty
    pub async fn is_empty(&self) -> bool {
        self.queue.read().await.is_empty()
    }
}

impl Default for TtsQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_queue_priority() {
        let queue = TtsQueue::new();

        // Enqueue in random order
        queue
            .enqueue(TtsMessage::new("info".to_string(), MessageKind::Info))
            .await;
        queue
            .enqueue(TtsMessage::new(
                "critical".to_string(),
                MessageKind::Critical,
            ))
            .await;
        queue
            .enqueue(TtsMessage::new(
                "background".to_string(),
                MessageKind::Background,
            ))
            .await;

        // Should dequeue by priority
        let msg1 = queue.dequeue().await.unwrap();
        assert_eq!(msg1.kind, MessageKind::Critical);

        let msg2 = queue.dequeue().await.unwrap();
        assert_eq!(msg2.kind, MessageKind::Info);

        let msg3 = queue.dequeue().await.unwrap();
        assert_eq!(msg3.kind, MessageKind::Background);
    }

    #[tokio::test]
    async fn test_coalescing() {
        let queue = TtsQueue::new();

        let msg1 = TtsMessage::new("working 1".to_string(), MessageKind::Info)
            .with_coalesce_key("status".to_string());
        let msg2 = TtsMessage::new("working 2".to_string(), MessageKind::Info)
            .with_coalesce_key("status".to_string());

        queue.enqueue(msg1).await;
        queue.enqueue(msg2).await;

        // Should only have one message
        assert_eq!(queue.len().await, 1);

        let msg = queue.dequeue().await.unwrap();
        assert_eq!(msg.text, "working 2");
    }

    #[tokio::test]
    async fn test_cancellation() {
        let queue = TtsQueue::new();

        let handle = queue
            .enqueue(TtsMessage::new("test".to_string(), MessageKind::Info))
            .await;

        handle.cancel();

        let cancelled = queue.check_cancellations().await;
        assert!(cancelled.contains(&handle.id()));
    }
}
