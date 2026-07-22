//! Print job processor

use crate::job_queue::JobQueue;
use boomaga_core::{Error, JobId, JobStatus, PrintJobRequest};
use boomaga_ipc::{Message, MessageDestination, MessagePayload, MessageSource, NotificationSender};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinSet;
use tracing::{debug, error, info};

/// Job processor
#[derive(Clone)]
pub struct JobProcessor {
    queue: Arc<JobQueue>,
    max_concurrent: usize,
    worker_threads: usize,
    jobs: Arc<RwLock<HashMap<String, JobStatus>>>,
    notifications: NotificationSender,
}

/// Job processing context
struct JobContext {
    job_id: String,
    request: PrintJobRequest,
}

impl JobProcessor {
    /// Create a new job processor
    pub fn new(
        queue: Arc<JobQueue>,
        max_concurrent: usize,
        worker_threads: usize,
        notifications: NotificationSender,
    ) -> Result<Self, Error> {
        if max_concurrent == 0 {
            return Err(Error::Validation(
                "Max concurrent jobs must be greater than 0".into(),
            ));
        }

        if worker_threads == 0 {
            return Err(Error::Validation(
                "Worker threads must be greater than 0".into(),
            ));
        }

        Ok(Self {
            queue,
            max_concurrent,
            worker_threads,
            jobs: Arc::new(RwLock::new(HashMap::new())),
            notifications,
        })
    }

    /// Add a job to the queue
    pub async fn add_job(&self, request: PrintJobRequest) -> Result<(), Error> {
        request.options.validate()?;

        let job_id = request.job_id.to_string();
        let notification_job_id = request.job_id.clone();

        info!("Adding job {} to queue", job_id);

        // Add to queue
        let queue_clone = Arc::clone(&self.queue);
        queue_clone.push(request).await?;

        // Update job status
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_id, JobStatus::Queued);
        }
        Self::notify(&self.notifications, notification_job_id, JobStatus::Queued);

        // Spawn worker task
        for _ in 0..self.worker_threads {
            let queue = Arc::clone(&self.queue);
            let jobs = Arc::clone(&self.jobs);
            let notifications = self.notifications.clone();

            tokio::spawn(async move {
                Self::process_queue(queue, jobs, notifications).await;
            });
        }

        Ok(())
    }

    /// Process job queue
    async fn process_queue(
        queue: Arc<JobQueue>,
        jobs: Arc<RwLock<HashMap<String, JobStatus>>>,
        notifications: NotificationSender,
    ) {
        let mut running = true;

        while running {
            // Wait for job to be available
            let queue_clone = Arc::clone(&queue);
            match queue_clone.pop().await {
                Ok(request) => {
                    let job_id = request.job_id.to_string();
                    let notification_job_id = request.job_id.clone();

                    // Update status to processing
                    {
                        let mut jobs = jobs.write().await;
                        jobs.insert(job_id.clone(), JobStatus::Processing);
                    }
                    Self::notify(
                        &notifications,
                        notification_job_id.clone(),
                        JobStatus::Processing,
                    );

                    info!("Processing job {}", job_id);

                    // Process job
                    match Self::process_job(request).await {
                        Ok(_) => {
                            info!("Job {} completed successfully", job_id);
                            {
                                let mut jobs = jobs.write().await;
                                jobs.insert(job_id, JobStatus::Completed);
                            }
                            Self::notify(&notifications, notification_job_id, JobStatus::Completed);
                        }
                        Err(e) => {
                            error!("Job {} failed: {}", job_id, e);
                            {
                                let mut jobs = jobs.write().await;
                                jobs.insert(job_id, JobStatus::Failed);
                            }
                            Self::notify(&notifications, notification_job_id, JobStatus::Failed);
                        }
                    }
                }
                Err(_) => {
                    // Queue is empty
                    running = false;
                }
            }
        }
    }

    fn notify(sender: &NotificationSender, job_id: JobId, status: JobStatus) {
        let _ = sender.send(Message::new_notification(
            MessageSource::Backend,
            MessageDestination::Preview,
            MessagePayload::PrintJobStatus { job_id, status },
        ));
    }

    /// Process a single job
    async fn process_job(request: PrintJobRequest) -> Result<(), Error> {
        // Simulate job processing
        // In production, this would:
        // 1. Parse document
        // 2. Render pages
        // 3. Apply layout transformations
        // 4. Create preview window
        // 5. Wait for user action

        debug!("Processing job: {:?}", request);

        // Simulate processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(())
    }

    /// Get job status
    pub async fn get_status(&self, job_id: String) -> Option<JobStatus> {
        let jobs = self.jobs.read().await;
        jobs.get(&job_id).copied()
    }

    /// Get all jobs
    pub async fn get_all_jobs(&self) -> Vec<(String, JobStatus)> {
        let jobs = self.jobs.read().await;
        jobs.iter().map(|(k, v)| (k.clone(), *v)).collect()
    }

    /// Cancel a job
    pub async fn cancel_job(&self, job_id: String) -> Result<(), Error> {
        // TODO: Implement job cancellation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use boomaga_core::{FileType, PrintOptions};
    use std::path::PathBuf;

    #[tokio::test]
    async fn emits_job_status_notifications_in_order() {
        let queue = Arc::new(JobQueue::new(4).unwrap());
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let processor = JobProcessor::new(queue, 1, 1, sender).unwrap();
        let job_id: JobId =
            serde_json::from_str("\"f7f04d62-a28d-4f7c-a55a-cf35dc913918\"").unwrap();

        processor
            .add_job(PrintJobRequest {
                job_id,
                file_path: PathBuf::from("test.pdf"),
                file_type: FileType::Pdf,
                printer_name: None,
                options: PrintOptions::default(),
            })
            .await
            .unwrap();

        for expected in [
            JobStatus::Queued,
            JobStatus::Processing,
            JobStatus::Completed,
        ] {
            let message =
                tokio::time::timeout(tokio::time::Duration::from_secs(1), receiver.recv())
                    .await
                    .unwrap()
                    .unwrap();
            match message.payload {
                MessagePayload::PrintJobStatus { status, .. } => assert_eq!(status, expected),
                payload => panic!("unexpected payload: {payload:?}"),
            }
        }
    }
}
