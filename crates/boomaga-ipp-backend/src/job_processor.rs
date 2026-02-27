//! Print job processor

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinSet;
use tracing::{info, error, debug};
use boomaga_core::{PrintJobRequest, JobStatus, Error};
use crate::job_queue::JobQueue;

/// Job processor
pub struct JobProcessor {
    queue: Arc<JobQueue>,
    max_concurrent: usize,
    worker_threads: usize,
    jobs: Arc<RwLock<HashMap<String, JobStatus>>>,
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
    ) -> Result<Self, Error> {
        if max_concurrent == 0 {
            return Err(Error::Validation("Max concurrent jobs must be greater than 0".into()));
        }

        if worker_threads == 0 {
            return Err(Error::Validation("Worker threads must be greater than 0".into()));
        }

        Ok(Self {
            queue,
            max_concurrent,
            worker_threads,
            jobs: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Add a job to the queue
    pub async fn add_job(&self, request: PrintJobRequest) -> Result<(), Error> {
        request.options.validate()?;

        let job_id = request.job_id.to_string();

        info!("Adding job {} to queue", job_id);

        // Add to queue
        self.queue.push(request).await?;

        // Update job status
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_id, JobStatus::Queued);
        }

        // Spawn worker task
        for _ in 0..self.worker_threads {
            let queue = Arc::clone(&self.queue);
            let jobs = Arc::clone(&self.jobs);

            tokio::spawn(async move {
                Self::process_queue(queue, jobs).await;
            });
        }

        Ok(())
    }

    /// Process job queue
    async fn process_queue(queue: Arc<JobQueue>, jobs: Arc<RwLock<HashMap<String, JobStatus>>>) {
        let mut running = true;

        while running {
            // Wait for job to be available
            match queue.pop().await {
                Ok(request) => {
                    let job_id = request.job_id.to_string();

                    // Update status to processing
                    {
                        let mut jobs = jobs.write().await;
                        jobs.insert(job_id.clone(), JobStatus::Processing);
                    }

                    info!("Processing job {}", job_id);

                    // Process job
                    match Self::process_job(request).await {
                        Ok(_) => {
                            info!("Job {} completed successfully", job_id);
                            {
                                let mut jobs = jobs.write().await;
                                jobs.insert(job_id, JobStatus::Completed);
                            }
                        }
                        Err(e) => {
                            error!("Job {} failed: {}", job_id, e);
                            {
                                let mut jobs = jobs.write().await;
                                jobs.insert(job_id, JobStatus::Failed);
                            }
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
