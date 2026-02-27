//! Job queue implementation

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tracing::{info, debug};
use boomaga_core::{PrintJobRequest, Error};
use std::time::Instant;

/// Job queue
pub struct JobQueue {
    sender: mpsc::Sender<PrintJobRequest>,
    receiver: mpsc::Receiver<PrintJobRequest>,
    queue_size: Arc<AtomicUsize>,
    max_size: usize,
}

impl JobQueue {
    /// Create a new job queue
    pub fn new(max_size: usize) -> Result<Self, Error> {
        if max_size == 0 {
            return Err(Error::Validation("Queue size must be greater than 0".into()));
        }

        let (sender, receiver) = mpsc::channel(max_size);

        Ok(Self {
            sender,
            receiver,
            queue_size: Arc::new(AtomicUsize::new(0)),
            max_size,
        })
    }

    /// Push a job into the queue
    pub async fn push(&self, request: PrintJobRequest) -> Result<(), Error> {
        if self.queue_size.load(Ordering::Relaxed) >= self.max_size {
            return Err(Error::Validation("Queue is full".into()));
        }

        self.sender.send(request).await
            .map_err(|e| Error::Job(format!("Failed to push job: {}", e)))?;

        self.queue_size.fetch_add(1, Ordering::Relaxed);

        debug!("Job pushed to queue. Current size: {}", self.queue_size.load(Ordering::Relaxed));

        Ok(())
    }

    /// Pop a job from the queue
    pub async fn pop(&mut self) -> Result<PrintJobRequest, Error> {
        self.queue_size.fetch_sub(1, Ordering::Relaxed);

        match self.receiver.recv().await {
            Some(job) => Ok(job),
            None => Err(Error::Job("Queue is empty".into())),
        }
    }

    /// Get current queue size
    pub fn size(&self) -> usize {
        self.queue_size.load(Ordering::Relaxed)
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// Check if queue is full
    pub fn is_full(&self) -> bool {
        self.size() >= self.max_size
    }

    /// Get max queue size
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Clear the queue
    pub async fn clear(&self) {
        // Drain the receiver
        while self.receiver.try_recv().is_ok() {
            self.queue_size.fetch_sub(1, Ordering::Relaxed);
        }

        info!("Queue cleared. Size: {}", self.size());
    }
}

/// Job statistics
pub struct QueueStatistics {
    pub current_size: usize,
    pub max_size: usize,
    pub total_pushed: u64,
    pub total_popped: u64,
    pub avg_processing_time: std::time::Duration,
    pub peak_size: usize,
    pub peak_time: Option<Instant>,
}

impl JobQueue {
    /// Get queue statistics
    pub fn get_statistics(&self) -> QueueStatistics {
        QueueStatistics {
            current_size: self.size(),
            max_size: self.max_size,
            total_pushed: 0, // TODO: Track total pushed
            total_popped: 0, // TODO: Track total popped
            avg_processing_time: std::time::Duration::from_secs(0),
            peak_size: self.size(),
            peak_time: None,
        }
    }
}
