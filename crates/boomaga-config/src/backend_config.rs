//! Backend service configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Backend service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// IPP service port
    pub ipp_port: u16,

    /// IPC socket path
    pub ipc_socket_path: PathBuf,

    /// D-Bus service name
    pub dbus_service_name: String,

    /// Maximum concurrent job processing
    pub max_concurrent_jobs: usize,

    /// Number of worker threads
    pub worker_threads: usize,

    /// Job queue size
    pub job_queue_size: usize,

    /// Queue timeout (seconds)
    pub queue_timeout: u64,

    /// Enable debug logging
    pub debug: bool,

    /// Enable verbose logging
    pub verbose: bool,

    /// Enable DNS-SD service discovery
    pub dns_sd: bool,

    /// DNS-SD service type
    pub dns_sd_service_type: String,

    /// Job processing timeout (seconds)
    pub job_timeout: u64,

    /// Maximum job size in bytes
    pub max_job_size: u64,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            ipp_port: 631,
            ipc_socket_path: PathBuf::from(boomaga_core::constants::IPC_SOCKET_PATH),
            dbus_service_name: boomaga_core::constants::DBUS_SERVICE_NAME.to_string(),
            max_concurrent_jobs: boomaga_core::constants::MAX_CONCURRENT_JOBS,
            worker_threads: boomaga_core::constants::WORKER_THREADS,
            job_queue_size: boomaga_core::constants::JOB_QUEUE_SIZE,
            queue_timeout: 30,
            debug: false,
            verbose: false,
            dns_sd: true,
            dns_sd_service_type: "ipp".to_string(),
            job_timeout: 300,
            max_job_size: 100 * 1024 * 1024, // 100 MB
        }
    }
}

impl BackendConfig {
    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.max_concurrent_jobs == 0 {
            return Err(anyhow::anyhow!("Max concurrent jobs must be greater than 0"));
        }

        if self.worker_threads == 0 {
            return Err(anyhow::anyhow!("Worker threads must be greater than 0"));
        }

        if self.job_queue_size == 0 {
            return Err(anyhow::anyhow!("Job queue size must be greater than 0"));
        }

        if self.ipp_port < 1024 || self.ipp_port > 65535 {
            return Err(anyhow::anyhow!("IPP port must be between 1024 and 65535"));
        }

        if self.max_job_size == 0 {
            return Err(anyhow::anyhow!("Max job size must be greater than 0"));
        }

        Ok(())
    }

    /// Enable debug mode
    pub fn with_debug(mut self, enabled: bool) -> Self {
        self.debug = enabled;
        self
    }

    /// Enable verbose mode
    pub fn with_verbose(mut self, enabled: bool) -> Self {
        self.verbose = enabled;
        self
    }

    /// Set custom IPC socket path
    pub fn with_ipc_socket(mut self, path: PathBuf) -> Self {
        self.ipc_socket_path = path;
        self
    }

    /// Set custom IPP port
    pub fn with_port(mut self, port: u16) -> Self {
        self.ipp_port = port;
        self
    }
}

impl From<BackendConfig> for boomaga_core::constants::AppConfig {
    fn from(config: BackendConfig) -> Self {
        boomaga_core::constants::AppConfig {
            ipc_socket_path: config.ipc_socket_path,
            dbus_service_name: config.dbus_service_name,
            ipp_port: config.ipp_port,
            max_concurrent_jobs: config.max_concurrent_jobs,
            worker_threads: config.worker_threads,
            job_queue_size: config.job_queue_size,
        }
    }
}
